buildscript {
    repositories {
        mavenCentral()
    }
    dependencies {
        classpath("xyz.baaad.machikado:java:3.0.1")
    }
}

import android.databinding.tool.ext.capitalizeUS
import java.nio.file.Files
import java.security.MessageDigest
import org.apache.commons.codec.binary.Hex
import org.apache.tools.ant.filters.FixCrLfFilter
import org.apache.tools.ant.filters.ReplaceTokens
import xyz.baaad.machikado.Sign
import xyz.baaad.machikado.Utils

plugins {
    alias(libs.plugins.agp.lib)
}

val moduleId: String = rootProject.extra["moduleId"] as String
val moduleName: String = rootProject.extra["moduleName"] as String
val verCode: Int = rootProject.extra["verCode"] as Int
val verName: String = rootProject.extra["verName"] as String
val minKsuVersion: Int = rootProject.extra["minKsuVersion"] as Int
val minKsudVersion: Int = rootProject.extra["minKsudVersion"] as Int
val minMagiskVersion: Int = rootProject.extra["minMagiskVersion"] as Int
val minApatchVersion: Int = rootProject.extra["minApatchVersion"] as Int
val kpmId: String = "znnext"
val commitHash: String = rootProject.extra["commitHash"] as String
val updateUrl: String = rootProject.extra["updateUrl"] as String
val androidMinSdkVersion: Int = rootProject.extra["androidMinSdkVersion"] as Int

android {
    androidResources {
        enable = false
    }
    buildFeatures {
        buildConfig = false
    }
}

val xzMinEntrySize = 1024L

fun runProcess(command: List<String>, workingDirectory: File? = null): Pair<Int, String> {
    val process = ProcessBuilder(command)
        .apply { if (workingDirectory != null) directory(workingDirectory) }
        .redirectErrorStream(true)
        .start()
    val output = process.inputStream.bufferedReader().use { it.readText() }
    return process.waitFor() to output
}

fun resolveSevenZip(probeRoot: File): String {
    val candidates = listOfNotNull(
        findProperty("sevenZip.path") as? String,
        "7zz",
        "7z",
        "7za",
    )
    val probeDir = File(probeRoot, "probe").apply { deleteRecursively(); mkdirs() }
    File(probeDir, "probe.txt").writeText("z".repeat(4096))
    try {
        for (candidate in candidates) {
            File(probeDir, "probe.zip").delete()
            try {
                val add = runProcess(
                    listOf(candidate, "a", "-tzip", "-mm=XZ", "probe.zip", "probe.txt"), probeDir
                )
                if (add.first != 0) continue
                val listing = runProcess(listOf(candidate, "l", "-slt", "probe.zip"), probeDir)
                if (listing.first == 0 && "Method = xz" in listing.second) return candidate
            } catch (_: Exception) {
            }
        }
    } finally {
        probeDir.deleteRecursively()
    }
    throw GradleException(
        "No 7-Zip found"
    )
}

val licenseOutputDir = layout.buildDirectory.dir("licenses")
val externalDir = rootProject.layout.projectDirectory.dir("loader/src/external")
val externalLicenseFiles = rootProject.fileTree("loader/src/external") {
    include(
        "*/LICENSE", "*/LICENSE.*", "*/LICENSE-*",
        "*/COPYING", "*/COPYING.*", "*/COPYING-*",
    )
}

val prepareLicensesTask = tasks.register("prepareLicenses") {
    group = "module"
    description = "Collects the project and external dependency licenses for the module zip."
    inputs.file(rootProject.file("LICENSE")).withPropertyName("projectLicense")
    inputs.files(externalLicenseFiles).withPropertyName("externalLicenses")
    outputs.dir(licenseOutputDir)

    doLast {
        val output = licenseOutputDir.get().asFile
        output.deleteRecursively()
        output.mkdirs()

        fun sanitize(value: String): String =
            value.uppercase().replace(Regex("[^A-Z0-9]+"), "_").trim('_')

        fun licenseSuffix(fileName: String): String {
            val upper = fileName.uppercase()
            val suffix = when {
                upper.startsWith("LICENSE-") -> upper.removePrefix("LICENSE-")
                upper.startsWith("LICENSE.") -> upper.removePrefix("LICENSE.")
                upper.startsWith("COPYING-") -> upper.removePrefix("COPYING-")
                upper.startsWith("COPYING.") -> upper.removePrefix("COPYING.")
                else -> ""
            }
            return sanitize(suffix)
        }

        val used = mutableSetOf("LICENSE-ZNNext")
        rootProject.file("LICENSE").copyTo(File(output, "LICENSE-ZNNext"), overwrite = true)
        var count = 1

        externalLicenseFiles.files
            .sortedBy { it.invariantSeparatorsPath }
            .forEach { license ->
                val componentDir = license.parentFile?.parentFile ?: return@forEach
                if (componentDir != externalDir.asFile) return@forEach

                val component = sanitize(license.parentFile.name).ifEmpty { "EXTERNAL" }
                val suffix = licenseSuffix(license.name)
                val base = "LICENSE-$component" + if (suffix.isEmpty()) "" else "-$suffix"
                var target = base
                var duplicate = 2
                while (!used.add(target)) {
                    target = "$base-$duplicate"
                    duplicate++
                }
                license.copyTo(File(output, target), overwrite = true)
                count++
            }

        logger.lifecycle("licenses: packaged $count files into licenses/")
    }
}

androidComponents.onVariants { variant ->
    val variantLowered = variant.name.lowercase()
    val variantCapped = variant.name.capitalizeUS()
    val buildTypeLowered = variant.buildType?.lowercase()

    val moduleDir = layout.buildDirectory.dir("outputs/module/$variantLowered")
    val zipFileName = "$moduleName-$verName-$verCode-$commitHash-$buildTypeLowered.zip".replace(' ', '-')

    val prepareModuleFilesTask = tasks.register<Sync>("prepareModuleFiles$variantCapped") {
        group = "module"
        dependsOn(
            ":loader:assemble$variantCapped",
            ":daemon:assemble$variantCapped",
            ":kpm:assemble$variantCapped",
            ":webui:buildWebui",
            prepareLicensesTask,
        )
        inputs.dir(project(":kpm").layout.buildDirectory.dir("kpm/$variantLowered"))
            .withPropertyName("kpmBackend")
            .optional()
        into(moduleDir)
        from("${rootProject.projectDir}/README.md")
        from(licenseOutputDir) {
            into("licenses")
        }
        from("$projectDir/src") {
            exclude("module.prop", "customize.sh", "late-load.sh", "post-fs-data.sh", "mazoku")
            filter<FixCrLfFilter>("eol" to FixCrLfFilter.CrLf.newInstance("lf"))
        }
        // `mazoku` is a binary signature blob and must be copied verbatim.
        from("$projectDir/src") {
            include("mazoku")
        }
        from(rootProject.file("module/webroot")) {
            into("webroot")
        }
        from("$projectDir/src") {
            include("module.prop")
            expand(
                "moduleId" to moduleId,
                "moduleName" to moduleName,
                "versionName" to "$verName ($verCode-$commitHash-$variantLowered)",
                "versionCode" to verCode,
                "updateUrl" to updateUrl
            )
        }
        from("$projectDir/src") {
            include("customize.sh", "late-load.sh", "post-fs-data.sh")
            val tokens = mapOf(
                "DEBUG" to if (buildTypeLowered == "debug") "true" else "false",
                "MIN_SDK" to "$androidMinSdkVersion",
                "MIN_KSU_VERSION" to "$minKsuVersion",
                "MIN_KSUD_VERSION" to "$minKsudVersion",
                "MIN_MAGISK_VERSION" to "$minMagiskVersion",
                "MIN_APATCH_VERSION" to "$minApatchVersion",
                "KPM_ID" to kpmId,
            )
            filter<ReplaceTokens>("tokens" to tokens)
            filter<FixCrLfFilter>("eol" to FixCrLfFilter.CrLf.newInstance("lf"))
        }

        val cmakeBuildType = if (buildTypeLowered == "debug") "Debug" else "RelWithDebInfo"

        inputs.dir(project(":daemon").layout.buildDirectory.dir("rust/$variantLowered"))
            .withPropertyName("daemonArtifacts")
            .optional()
        inputs.files(
            fileTree(project(":loader").layout.buildDirectory.dir("intermediates/cxx/$cmakeBuildType")) {
                include("**/libzn_loader.so")
            },
        ).withPropertyName("loaderArtifacts").optional()

        doLast {
            val dstRoot = moduleDir.get().asFile

            fun cmakeObjDir(projectPath: String): File? {
                val cxxDir = project(projectPath).layout.buildDirectory
                    .dir("intermediates/cxx/$cmakeBuildType").get().asFile
                val hashDir = cxxDir.listFiles()
                    ?.filter { it.isDirectory && File(it, "obj").isDirectory }
                    ?.maxByOrNull { it.lastModified() }
                    ?: return null
                return File(hashDir, "obj")
            }

            fun collectArtifacts(projectPath: String, artifactName: String, destOf: (String) -> File) {
                cmakeObjDir(projectPath)?.listFiles()?.forEach { abiDir ->
                    val artifact = File(abiDir, artifactName)
                    if (!abiDir.isDirectory || !artifact.isFile) return@forEach
                    val dest = destOf(abiDir.name)
                    dest.parentFile.mkdirs()
                    artifact.copyTo(dest, overwrite = true)
                }
            }

            collectArtifacts(":loader", "libzn_loader.so") { abi -> File(dstRoot, "lib/$abi/libzn_loader.so") }

            val daemonVariants = project(":daemon").layout.buildDirectory
                .dir("rust/$variantLowered").get().asFile
            daemonVariants.listFiles()?.forEach { abiDir ->
                val artifact = File(abiDir, "znnd")
                if (!abiDir.isDirectory || !artifact.isFile) return@forEach
                val destination = File(dstRoot, "bin/${abiDir.name}/znnd")
                destination.parentFile.mkdirs()
                artifact.copyTo(destination, overwrite = true)
            }

            val kpmProperty = (findProperty("kpm") as? String)?.trim()?.lowercase()
            val kpmSkipped = kpmProperty == "false" || kpmProperty == "off" || kpmProperty == "no"
            val kpmArtifact = File(
                project(":kpm").layout.buildDirectory.dir("kpm/$variantLowered").get().asFile,
                "znn.kpm"
            )
            if (kpmArtifact.isFile) {
                kpmArtifact.copyTo(File(dstRoot, "znn.kpm"), overwrite = true)
                logger.lifecycle("Packaging KernelPatch backend: znn.kpm")
            } else if (kpmSkipped) {
                logger.warn("kpm: -Pkpm=false, the zip will not contain znn.kpm")
            } else {
                throw GradleException(
                    "znn.kpm was not built; the module zip always ships it " +
                        "(build with a complete NDK, or pass -Pkpm=false to ship without it)"
                )
            }
        }

        val root = moduleDir.get()

        doLast {
            if (file("private_key").exists()) {
                println("=== Guards the peace of Machikado ===")
                val privateKey = file("private_key").readBytes()
                fun getSign(name: String, abi: String, abi32: String, abi64: String) {
                    val mapping = Utils.FileMapping()
                    mapping.insert("module.prop.orig", "module.prop")
                    mapping.insert("post-fs-data.sh", null)
                    mapping.insert("late-load.sh", null)
                    mapping.insert("emulated-soft-reboot.sh", null)
                    mapping.insert("sepolicy.rule", null)
                    mapping.insert("mazoku", null)
                    mapping.insert("bin/znnd", "bin/$abi/znnd")
                    mapping.insert("lib/libzn_loader.so", "lib/$abi32/libzn_loader.so")
                    mapping.insert("lib64/libzn_loader.so", "lib/$abi64/libzn_loader.so")
                    root.asFile.resolve("webroot").walkTopDown()
                        .filter { it.isFile }
                        .forEach { webrootFile ->
                            val relative = webrootFile.relativeTo(root.asFile).invariantSeparatorsPath
                            mapping.insert(relative, relative)
                        }
                    val entries: MutableList<Sign.FileEntry> = Utils.loadFromMapping(
                        root.asFile.toPath(),
                        mapping
                    )
                    val machikado = Sign.signFileEntries(entries, privateKey)
                    Files.write(root.file(name).asFile.toPath(), machikado.asBytes())
                }
                getSign("machikado.arm64", "arm64-v8a", "armeabi-v7a", "arm64-v8a")
                getSign("machikado.x64", "x86_64", "x86", "x86_64")
                getSign("machikado.arm", "armeabi-v7a", "armeabi-v7a", "arm64-v8a")
                getSign("machikado.x86", "x86", "x86", "x86_64")
                getSign("machikado.riscv64", "riscv64", "riscv64", "riscv64")
            } else {
                println("no private_key found, this build will not be signed")
                root.file("machikado.arm64").asFile.createNewFile()
                root.file("machikado.x64").asFile.createNewFile()
                root.file("machikado.arm").asFile.createNewFile()
                root.file("machikado.x86").asFile.createNewFile()
                root.file("machikado.riscv64").asFile.createNewFile()
            }

            fileTree(moduleDir).visit {
                if (isDirectory) return@visit
                val md = MessageDigest.getInstance("SHA-256")
                file.forEachBlock(4096) { bytes, size ->
                    md.update(bytes, 0, size)
                }
                file(file.path + ".sha256").writeText(Hex.encodeHexString(md.digest()))
            }
        }
    }

    val releaseDir = layout.buildDirectory.dir("outputs/release").get().asFile
    val archiveFile = File(releaseDir, zipFileName)

    val zipTask = tasks.register("zip$variantCapped") {
        group = "module"
        description = "Packs the $variantLowered module into $zipFileName with XZ-compressed entries."
        dependsOn(prepareModuleFilesTask)
        inputs.dir(moduleDir)
        outputs.file(archiveFile)

        doLast {
            val sevenZip = resolveSevenZip(layout.buildDirectory.dir("sevenzip").get().asFile)
            val root = moduleDir.get().asFile

            releaseDir.mkdirs()
            archiveFile.delete()

            val entries = fileTree(root).files
                .filter { it.isFile }
                .map { it.relativeTo(root).invariantSeparatorsPath }
                .sorted()
            val (deflated, xz) = entries.partition { File(root, it).length() < xzMinEntrySize }

            fun append(method: String, members: List<String>) {
                if (members.isEmpty()) return
                val (status, output) = runProcess(
                    listOf(
                        sevenZip, "a", "-tzip", "-mx=9", "-mm=$method", "-bso0", "-bsp0",
                        archiveFile.absolutePath,
                    ) + members,
                    root,
                )
                if (status != 0) {
                    throw GradleException("7-Zip failed: $method ($status):\n$output")
                }
            }

            append("Deflate", deflated)
            append("XZ", xz)

            logger.lifecycle(
                "$zipFileName: ${archiveFile.length()} bytes " +
                    "(${xz.size} xz + ${deflated.size} deflate entries)"
            )
        }
    }

    val pushTask = tasks.register<Exec>("push$variantCapped") {
        group = "module"
        dependsOn(zipTask)
        commandLine("adb", "push", zipTask.get().outputs.files.singleFile.path, "/data/local/tmp")
    }

    val installKsuTask = tasks.register("installKsu$variantCapped") {
        group = "module"
        dependsOn(pushTask)
        doLast {
            fun run(vararg cmd: String) {
                ProcessBuilder(*cmd).redirectErrorStream(true).start().waitFor()
            }
            run(
                "adb", "shell", "echo",
                "/data/adb/ksud module install /data/local/tmp/$zipFileName",
                "> /data/local/tmp/install.sh"
            )
            run("adb", "shell", "chmod", "755", "/data/local/tmp/install.sh")
            run("adb", "shell", "su", "-c", "/data/local/tmp/install.sh")
        }
    }

    val installMagiskTask = tasks.register<Exec>("installMagisk$variantCapped") {
        group = "module"
        dependsOn(pushTask)
        commandLine("adb", "shell", "su", "-M", "-c", "magisk --install-module /data/local/tmp/$zipFileName")
    }

    tasks.register<Exec>("installKsuAndReboot$variantCapped") {
        group = "module"
        dependsOn(installKsuTask)
        commandLine("adb", "reboot")
    }

    tasks.register<Exec>("installMagiskAndReboot$variantCapped") {
        group = "module"
        dependsOn(installMagiskTask)
        commandLine("adb", "reboot")
    }
}
