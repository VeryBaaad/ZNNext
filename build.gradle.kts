import com.android.build.api.dsl.LibraryExtension
import org.gradle.internal.os.OperatingSystem

plugins {
    alias(libs.plugins.agp.lib) apply false
}

fun git(vararg args: String): String {
    return try {
        providers.exec {
            commandLine("git", *args)
            workingDir = layout.projectDirectory.asFile
        }.standardOutput.asText.get().trim()
    } catch (_: Exception) {
        ""
    }
}

val gitCommitCount = git("rev-list", "HEAD", "--count").toIntOrNull() ?: 1
val gitCommitHash = git("rev-parse", "--verify", "--short", "HEAD").ifEmpty { "unknown" }

// Module metadata, exposed to :loader and :module through rootProject.extra.
val moduleId = "zygisknextsu"
val moduleName = "ZNNext"
val verName = "v2.6.0"
val verCode = gitCommitCount
val commitHash = gitCommitHash
val updateUrl = "https://raw.githubusercontent.com/VeryBaaad/ZNNext/refs/heads/update/update.json"
val kpmVersion = verName.removePrefix("v")

val minKsuVersion = 10940
val minKsudVersion = 11425
val minMagiskVersion = 26402
val minApatchVersion = 10700

extra.set("moduleId", moduleId)
extra.set("moduleName", moduleName)
extra.set("verName", verName)
extra.set("verCode", verCode)
extra.set("commitHash", commitHash)
extra.set("updateUrl", updateUrl)
extra.set("kpmVersion", kpmVersion)

extra.set("minKsuVersion", minKsuVersion)
extra.set("minKsudVersion", minKsudVersion)
extra.set("minMagiskVersion", minMagiskVersion)
extra.set("minApatchVersion", minApatchVersion)

val defaultCFlags = arrayOf(
    "-Wall", "-Wextra",
    "-fno-rtti", "-fno-exceptions",
    "-fno-stack-protector", "-fomit-frame-pointer",
    "-Wno-builtin-macro-redefined", "-D__FILE__=__FILE_NAME__"
)

val optimizedCFlags = arrayOf(
    "-flto",
    "-Wno-unused", "-Wno-unused-parameter",
    "-fvisibility=hidden", "-fvisibility-inlines-hidden",
    "-fno-unwind-tables", "-fno-asynchronous-unwind-tables",
)

val debugCFlags = arrayOf("-Os", *optimizedCFlags)
val releaseCFlags = arrayOf("-Oz", *optimizedCFlags)

val commonLinkerFlags = "-Wl,--exclude-libs,ALL -Wl,--gc-sections -Wl,--icf=safe"
val debugLinkerFlags = "-Os $commonLinkerFlags"
val releaseLinkerFlags = "-Oz $commonLinkerFlags -Wl,--strip-all"

// ccache is used whenever it is available on PATH, or through -Pccache.path=...
val ccachePath: String? = run {
    val executable = "ccache" + if (OperatingSystem.current().isWindows) ".exe" else ""
    System.getenv("PATH")?.split(File.pathSeparator).orEmpty()
        .map { File(it, executable) }
        .firstOrNull { it.exists() }
        ?.absolutePath
        ?: findProperty("ccache.path") as? String
}

extra.set("defaultCFlags", defaultCFlags)
extra.set("debugCFlags", debugCFlags)
extra.set("debugLinkerFlags", debugLinkerFlags)
extra.set("releaseCFlags", releaseCFlags)
extra.set("releaseLinkerFlags", releaseLinkerFlags)
extra.set("ccachePath", ccachePath)

// Android build configuration (used only by this script).
val androidMinSdkVersion = 21
val androidCompileSdkVersion = 37
val androidCompileSdkMinorVersion = 2
val androidBuildToolsVersion = "37.0.0"
val androidCompileNdkVersion = "30.0.16248370"

extra.set("androidMinSdkVersion", androidMinSdkVersion)
extra.set("androidNdkVersion", androidCompileNdkVersion)

tasks.register("Delete", Delete::class) {
    delete(layout.buildDirectory)
}

fun Project.configureBaseExtension() {
    extensions.configure<LibraryExtension> {
        namespace = "xyz.baaad.znn"
        compileSdk {
            version = release(androidCompileSdkVersion) {
                minorApiLevel = androidCompileSdkMinorVersion
            }
        }
        defaultConfig {
            minSdk = androidMinSdkVersion
        }
        ndkVersion = androidCompileNdkVersion
        buildToolsVersion = androidBuildToolsVersion
        lint {
            abortOnError = true
        }
    }
}

subprojects {
    plugins.withId("com.android.library") {
        configureBaseExtension()
    }
}
