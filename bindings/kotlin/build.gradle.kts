import org.gradle.api.tasks.JavaExec
import org.gradle.api.tasks.compile.JavaCompile
import org.gradle.api.tasks.testing.Test
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget

plugins {
  kotlin("multiplatform") version "2.2.21"
  id("com.android.kotlin.multiplatform.library") version "9.1.1"
}

repositories {
  google()
  mavenCentral()
}

val nativeBuildDir = providers.environmentVariable("MLN_FFI_BUILD_DIR").orNull
val nativeLibraryPathForTests =
  providers.environmentVariable("MLN_FFI_BUILD_DIR").map {
    "$it/${System.mapLibraryName("maplibre-native-c")}"
  }
val javaCppVersion = "1.5.11"
val javaCppToolClasspath =
  configurations.detachedConfiguration(dependencies.create("org.bytedeco:javacpp:$javaCppVersion"))
val generatedJavaCppSources =
  layout.buildDirectory.dir("generated/sources/javacpp/androidMain/java")
val javaCppConfigClasses = layout.buildDirectory.dir("classes/javacppConfig")
val hostOs = System.getProperty("os.name").lowercase()
val hostArch = System.getProperty("os.arch").lowercase()

kotlin {
  jvmToolchain(25)

  compilerOptions { freeCompilerArgs.add("-Xexpect-actual-classes") }

  jvm { compilerOptions { jvmTarget.set(JvmTarget.JVM_24) } }

  android {
    namespace = "org.maplibre.nativeffi"
    compileSdk = 36
    minSdk = 24

    withJava()

    compilerOptions { jvmTarget.set(JvmTarget.JVM_17) }
  }

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

  sourceSets {
    androidMain.dependencies { implementation("org.bytedeco:javacpp:$javaCppVersion") }

    commonTest.dependencies { implementation(kotlin("test")) }
  }
}

androidComponents {
  onVariants { variant ->
    variant.sources.java?.addStaticSourceDirectory(
      generatedJavaCppSources.get().asFile.absolutePath
    )
  }
}

val compileJavaCppConfig =
  tasks.register<JavaCompile>("compileAndroidJavaCppConfig") {
    source(
      "src/androidMain/java/org/maplibre/nativeffi/internal/javacpp/MaplibreNativeCConfig.java"
    )
    classpath = javaCppToolClasspath
    destinationDirectory = javaCppConfigClasses
    options.release = 17
  }

val generateJavaCppBindings =
  tasks.register<JavaExec>("generateAndroidJavaCppBindings") {
    group = "build"
    description = "Generates JavaCPP declarations for the Android MapLibre Native C ABI."
    dependsOn(compileJavaCppConfig)
    classpath = files(javaCppConfigClasses) + javaCppToolClasspath
    mainClass = "org.bytedeco.javacpp.tools.Builder"
    args(
      "-classpath",
      classpath.asPath,
      "-Dplatform.includepath=${rootProject.layout.projectDirectory.dir("include").asFile.absolutePath}",
      "-d",
      generatedJavaCppSources.get().asFile.absolutePath,
      "-nogenerate",
      "org.maplibre.nativeffi.internal.javacpp.MaplibreNativeCConfig",
    )
    inputs.file(
      "src/androidMain/java/org/maplibre/nativeffi/internal/javacpp/MaplibreNativeCConfig.java"
    )
    inputs.dir(rootProject.layout.projectDirectory.dir("include"))
    outputs.file(
      generatedJavaCppSources.map {
        it.file("org/maplibre/nativeffi/internal/javacpp/MaplibreNativeC.java")
      }
    )
  }

tasks
  .matching { it.name == "compileAndroidMainJavaWithJavac" }
  .configureEach { dependsOn(generateJavaCppBindings) }

tasks
  .matching { it.name == "compileAndroidMain" || it.name == "extractAndroidMainAnnotations" }
  .configureEach { dependsOn(generateJavaCppBindings) }

tasks.withType<Test>().configureEach {
  if (name == "jvmTest") {
    jvmArgs("--enable-native-access=ALL-UNNAMED")
    systemProperty("org.maplibre.nativeffi.library.path", nativeLibraryPathForTests.get())
    inputs.file(nativeLibraryPathForTests).withPropertyName("maplibreNativeCLibrary")
  }
}

tasks.register("nativeTest") {
  group = "verification"
  description = "Runs tests for the host Kotlin/Native target."
  dependsOn(kotlin.targets.withType<KotlinNativeTarget>().map { "${it.name}Test" })
}

tasks.register("androidBuild") {
  group = "build"
  description = "Builds the Android variant of the Kotlin Multiplatform binding."
  dependsOn("assembleAndroidMain")
}
