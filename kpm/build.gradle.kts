import org.gradle.internal.os.OperatingSystem

val kpmVersion: String = rootProject.extra["kpmVersion"] as String
val ndkVersion: String = rootProject.extra["androidNdkVersion"] as String

val requestedMode: String? = (findProperty("kpm") as? String)?.trim()?.lowercase()
val kpmDisabled = requestedMode == "false" || requestedMode == "off" || requestedMode == "no"

val manifestFile = file("version.properties")

val compilerFlags = listOf(
    "-O2",
    "-ffreestanding",
    "-nostdlib",
    "-fno-stack-protector",
    "-fno-pic",
    "-fno-builtin",
    "-fno-asynchronous-unwind-tables",
    "-fno-unwind-tables",
    "-fvisibility=hidden",
    "-mno-outline-atomics",
)

val includeDirectories = listOf(
    "external/kpatch/include",
    "external/kpatch/patch/include",
    "external/kpatch/linux/include",
    "external/kpatch/linux/arch/arm64/include",
)

val syncKpmVersion = tasks.register("syncKpmVersion") {
    group = "kpm"
    description = "Writes the module version into the KPM manifest."
    val manifest = manifestFile
    val version = kpmVersion
    inputs.property("version", version)
    outputs.file(manifest)
    doLast {
        val contents = manifest.readText()
        val versioned = Regex("(?m)^version\\s*=.*$")
            .replaceFirst(contents, "version=$version")
        if (versioned != contents) manifest.writeText(versioned)
    }
}

fun manifestVersion(): String {
    val line = manifestFile.readLines()
        .firstOrNull { it.trimStart().startsWith("version") }
        .orEmpty()
    return line.substringAfter('=', "").trim()
}

fun hostTag(): String = when {
    OperatingSystem.current().isLinux -> "linux-x86_64"
    OperatingSystem.current().isMacOsX -> "darwin-x86_64"
    else -> "windows-x86_64"
}

fun sdkDirectories(): List<File> {
    val directories = mutableListOf<File>()
    for (variable in listOf("ANDROID_SDK_ROOT", "ANDROID_HOME")) {
        System.getenv(variable)?.let { directories.add(File(it)) }
    }
    val localProperties = rootProject.file("local.properties")
    if (localProperties.isFile) {
        val configured = localProperties.readLines()
            .firstOrNull { it.startsWith("sdk.dir=") }
            ?.removePrefix("sdk.dir=")
            ?.trim()
        if (!configured.isNullOrEmpty()) directories.add(File(configured))
    }
    return directories
}

fun ndkCandidates(): List<File> {
    val candidates = mutableListOf<File>()
    for (variable in listOf("ANDROID_NDK_HOME", "ANDROID_NDK_ROOT")) {
        System.getenv(variable)?.let { candidates.add(File(it)) }
    }
    for (sdk in sdkDirectories()) {
        candidates.add(File(sdk, "ndk/$ndkVersion"))
        File(sdk, "ndk").listFiles()
            ?.sortedByDescending { it.name }
            ?.let { candidates.addAll(it) }
    }
    return candidates.filter { it.isDirectory }.distinct()
}

class KpmToolchain(val ndk: File, val clang: File, val linker: File)

fun resolveToolchain(): KpmToolchain? {
    for (ndk in ndkCandidates()) {
        val binaries = File(ndk, "toolchains/llvm/prebuilt/${hostTag()}/bin")
        if (!binaries.isDirectory) continue
        val clang = binaries.listFiles { file ->
            file.isFile &&
                file.name.startsWith("aarch64-linux-android") &&
                file.name.endsWith("-clang") &&
                file.name.removePrefix("aarch64-linux-android").removeSuffix("-clang").all { it.isDigit() }
        }?.sortedBy { it.name } ?: continue
        val preferred = clang.firstOrNull { it.name == "aarch64-linux-android26-clang" } ?: clang.first()
        val linker = File(binaries, "ld.lld").takeIf { it.isFile }
            ?: File(binaries, "aarch64-linux-android-ld").takeIf { it.isFile }
            ?: continue
        return KpmToolchain(ndk, preferred, linker)
    }
    return null
}

fun runCommand(command: List<String>, workingDirectory: File): Pair<Int, String> {
    val process = ProcessBuilder(command)
        .directory(workingDirectory)
        .redirectErrorStream(true)
        .start()
    val output = process.inputStream.bufferedReader().use { it.readText() }
    return process.waitFor() to output
}

fun toolchainHint(): String = buildString {
    append("no usable NDK with an aarch64-linux-android<api>-clang and ld.lld was found; searched ")
    val candidates = ndkCandidates()
    if (candidates.isEmpty()) {
        append("no candidate directories at all")
    } else {
        append(candidates.joinToString(", ") { it.absolutePath })
    }
    append(" (host tag ${hostTag()}). ")
    append("Set ANDROID_NDK_HOME to a complete NDK, or put sdk.dir=<sdk> in local.properties ")
    append("so that <sdk>/ndk/$ndkVersion resolves, or pass -Pkpm=false to skip this optional backend.")
}

fun buildKpm(toolchain: KpmToolchain, outputDirectory: File): File {
    outputDirectory.mkdirs()
    val source = file("src/znn.c")
    val objectFile = File(outputDirectory, "znn.o")
    val artifact = File(outputDirectory, "znn.kpm")
    val version = manifestVersion()
    if (version.isEmpty()) throw GradleException("$manifestFile has no version= entry")

    val compile = buildList {
        add(toolchain.clang.absolutePath)
        addAll(compilerFlags)
        add("-DZNN_KPM_VERSION=\"$version\"")
        includeDirectories.forEach { add("-I${file(it).absolutePath}") }
        add("-c")
        add(source.absolutePath)
        add("-o")
        add(objectFile.absolutePath)
    }
    val (compileStatus, compileOutput) = runCommand(compile, projectDir)
    if (compileStatus != 0 || !objectFile.isFile) {
        throw GradleException(
            "KPM compile failed (${toolchain.clang.absolutePath}, exit $compileStatus):\n$compileOutput"
        )
    }

    val link = listOf(
        toolchain.linker.absolutePath, "-r",
        objectFile.absolutePath, "-o", artifact.absolutePath,
    )
    val (linkStatus, linkOutput) = runCommand(link, projectDir)
    if (linkStatus != 0 || !artifact.isFile) {
        throw GradleException(
            "KPM relocatable link failed (${toolchain.linker.absolutePath}, exit $linkStatus):\n$linkOutput"
        )
    }
    return artifact
}

listOf("debug", "release").forEach { variant ->
    val variantCapped = variant.replaceFirstChar { it.uppercase() }
    val outputDirectory = layout.buildDirectory.dir("kpm/$variant")

    tasks.register("assemble$variantCapped") {
        group = "kpm"
        description = "Builds the KernelPatch backend for the $variant build type (arm64 only)."
        dependsOn(syncKpmVersion)
        inputs.dir(file("src"))
        inputs.dir(file("external/kpatch"))
        inputs.file(manifestFile)
        inputs.property("kpm", requestedMode ?: "auto")
        outputs.dir(outputDirectory)

        doLast {
            if (kpmDisabled) {
                logger.lifecycle("kpm ($variant): disabled by -Pkpm=false")
                return@doLast
            }
            val toolchain = resolveToolchain()
            if (toolchain == null) {
                throw GradleException("cannot build the KernelPatch backend: ${toolchainHint()}")
            }
            val artifact = buildKpm(toolchain, outputDirectory.get().asFile)
            logger.lifecycle(
                "kpm ($variant): ${artifact.absolutePath} " +
                    "(${artifact.length()} bytes, ${toolchain.clang.name}, NDK ${toolchain.ndk.name})"
            )
        }
    }
}
