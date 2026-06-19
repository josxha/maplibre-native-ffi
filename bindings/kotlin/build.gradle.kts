import org.gradle.api.tasks.testing.Test
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget

plugins { kotlin("multiplatform") version "2.2.21" }

repositories { mavenCentral() }

val nativeBuildDir = providers.environmentVariable("MLN_FFI_BUILD_DIR").orNull
val nativeLibraryPathForTests =
  providers.environmentVariable("MLN_FFI_BUILD_DIR").map {
    "$it/${System.mapLibraryName("maplibre-native-c")}"
  }
val hostOs = System.getProperty("os.name").lowercase()
val hostArch = System.getProperty("os.arch").lowercase()

kotlin {
  jvmToolchain(25)

  jvm { compilerOptions { jvmTarget.set(JvmTarget.JVM_24) } }

  when {
    hostOs.contains("mac") && (hostArch == "aarch64" || hostArch == "arm64") -> macosArm64()
    hostOs.contains("mac") -> macosX64()
    hostOs.contains("linux") && (hostArch == "aarch64" || hostArch == "arm64") -> linuxArm64()
    hostOs.contains("linux") -> linuxX64()
  }

  targets.withType<KotlinNativeTarget>().configureEach {
    if (nativeBuildDir != null) {
      binaries.all {
        linkerOpts("-L$nativeBuildDir", "-lmaplibre-native-c")
        if (hostOs.contains("mac") || hostOs.contains("linux")) {
          linkerOpts("-Wl,-rpath,$nativeBuildDir")
        }
        if (hostOs.contains("mac")) {
          linkerOpts("-framework", "Foundation", "-framework", "Metal", "-framework", "QuartzCore")
        }
      }
    }

    compilations.getByName("main") {
      cinterops {
        val maplibreNativeC by creating {
          defFile(project.file("src/nativeInterop/cinterop/maplibreNativeC.def"))
          includeDirs.headerFilterOnly(rootProject.file("include"))
          compilerOpts("-I${rootProject.file("include").absolutePath}")
        }
      }
    }
  }

  sourceSets { commonTest.dependencies { implementation(kotlin("test")) } }
}

tasks.withType<Test>().configureEach {
  if (name == "jvmTest") {
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    systemProperty("org.maplibre.nativeffi.library.path", nativeLibraryPathForTests.get())
    inputs.file(nativeLibraryPathForTests).withPropertyName("maplibreNativeCLibrary")
  }
}
