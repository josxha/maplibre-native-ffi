import org.gradle.api.file.Directory
import org.gradle.api.provider.Provider
import org.gradle.api.tasks.compile.JavaCompile
import org.gradle.api.tasks.javadoc.Javadoc
import org.gradle.external.javadoc.StandardJavadocDocletOptions

plugins { id("com.android.library") version "9.1.1" }

repositories {
  google()
  mavenCentral()
}

data class AndroidJniAbi(val abi: String, val javaCppPlatform: String, val ndkClangTriple: String)

val androidAbis =
  listOf(
    AndroidJniAbi("arm64-v8a", "android-arm64", "aarch64-linux-android"),
    AndroidJniAbi("x86_64", "android-x86_64", "x86_64-linux-android"),
  )

val curlVersion = "8.8.0"
val androidNativeApiLevel = "30"
val androidNdkVersion = "28.1.13356709"

val androidApiLevel =
  providers
    .environmentVariable("MLN_FFI_ANDROID_PLATFORM")
    .map { it.removePrefix("android-") }
    .orElse(androidNativeApiLevel)

android {
  namespace = "org.maplibre.nativejni"
  compileSdk = 34
  ndkVersion = androidNdkVersion

  buildFeatures { prefab = true }

  defaultConfig {
    minSdk = 33
    testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    testInstrumentationRunnerArguments["runnerBuilder"] =
      "de.mannodermaus.junit5.AndroidJUnit5Builder"

    ndk { abiFilters += androidAbis.map { it.abi } }

    // TODO(android): JNI hardcodes EGL/OpenGL here because Android is not wired into the
    // root mise variant matrix yet. Long term, android-* mise envs should drive
    // MLN_FFI_RENDER_BACKEND and related CMake args, root `mise run build` should
    // compile the shared native library for Android, and multiple bindings (not only
    // JNI) should consume that output instead of each Gradle module pinning backend
    // flags inline.
    externalNativeBuild {
      cmake {
        arguments +=
          listOf(
            "-DANDROID_STL=c++_static",
            "-DANDROID_PLATFORM=android-$androidNativeApiLevel",
            "-DMLN_FFI_RENDER_BACKEND=opengl",
            "-DMLN_FFI_OPENGL_CONTEXT_PROVIDER=egl",
            "-DMLN_FFI_ENABLE_CLANG_TIDY=OFF",
          )
        targets += "maplibre_native_c"
      }
    }
  }

  externalNativeBuild {
    cmake {
      version = "3.24.0+"
      path = rootProject.file("CMakeLists.txt")
    }
  }

  compileOptions {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
  }

  testOptions { animationsDisabled = true }

  packaging {
    jniLibs { pickFirsts += "**/libc++_shared.so" }
    resources {
      excludes += "/META-INF/LICENSE.md"
      excludes += "/META-INF/LICENSE-notice.md"
    }
  }
}

val javacppVersion = "1.5.13"

dependencies {
  implementation("io.github.vvb2060.ndk:curl:$curlVersion")
  implementation("org.bytedeco:javacpp:$javacppVersion")

  androidTestImplementation(platform("org.junit:junit-bom:5.11.4"))
  androidTestImplementation("org.junit.jupiter:junit-jupiter")
  androidTestRuntimeOnly("org.junit.platform:junit-platform-launcher")
  androidTestImplementation("androidx.test:runner:1.6.2")
  androidTestImplementation("androidx.test:rules:1.6.1")
  androidTestImplementation("de.mannodermaus.junit5:android-test-core:1.7.0")
  androidTestRuntimeOnly("de.mannodermaus.junit5:android-test-runner:1.7.0")
}

val javacppHostClasspath =
  configurations.detachedConfiguration(dependencies.create("org.bytedeco:javacpp:$javacppVersion"))
val generatedJavaCppSources = layout.buildDirectory.dir("generated/sources/javacpp/main/java")
val javaCppConfigClasses = layout.buildDirectory.dir("classes/javacppConfig")
val jniLibsRoot = layout.buildDirectory.dir("generated/jniLibs")

fun coreNativeLibDir(abi: String): Provider<Directory> =
  layout.buildDirectory.dir(
    "intermediates/merged_native_libs/debug/mergeDebugNativeLibs/out/lib/$abi"
  )

val compileJavaCppConfig =
  tasks.register<JavaCompile>("compileJavaCppConfig") {
    source("src/main/java/org/maplibre/nativejni/internal/javacpp/MaplibreNativeCConfig.java")
    classpath = javacppHostClasspath
    destinationDirectory = javaCppConfigClasses
    options.release = 21
  }

val generateJavaCppBindings =
  tasks.register<JavaExec>("generateJavaCppBindings") {
    group = "build"
    description = "Generates JavaCPP declarations for the MapLibre Native C ABI."
    dependsOn(compileJavaCppConfig)
    classpath = files(javaCppConfigClasses) + javacppHostClasspath
    mainClass = "org.bytedeco.javacpp.tools.Builder"
    args(
      "-classpath",
      classpath.asPath,
      // Header parsing uses one Android preset; native code is built per ABI in
      // buildJavaCppNative*. MaplibreNativeCConfig defines identical headers for
      // every Android @Platform, so the choice does not affect generated Java.
      "-properties",
      androidAbis.first().javaCppPlatform,
      "-Dplatform.includepath=${rootProject.layout.projectDirectory.dir("include").asFile.absolutePath}",
      "-d",
      generatedJavaCppSources.get().asFile.absolutePath,
      "-nogenerate",
      "org.maplibre.nativejni.internal.javacpp.MaplibreNativeCConfig",
    )
    inputs.file("src/main/java/org/maplibre/nativejni/internal/javacpp/MaplibreNativeCConfig.java")
    inputs.dir(rootProject.layout.projectDirectory.dir("include"))
    outputs.file(
      generatedJavaCppSources.map {
        it.file("org/maplibre/nativejni/internal/javacpp/MaplibreNativeC.java")
      }
    )
  }

val androidNdkRoot = android.sdkComponents.ndkDirectory

// AGP wires externalNativeBuild/cmake itself. JavaCPP compiles the JNI bridge in a
// separate JavaExec, so we point it at the NDK LLVM bin directory from the same
// install AGP selected (one host toolchain per NDK package).
fun ndkLlvmBinDir(ndkRoot: java.io.File): java.io.File {
  val prebuilt = ndkRoot.resolve("toolchains/llvm/prebuilt")
  val hostToolchains =
    prebuilt.listFiles { file -> file.isDirectory }?.sortedBy { it.name }.orEmpty()
  check(hostToolchains.size == 1) {
    "Expected one NDK host toolchain under $prebuilt, found: ${hostToolchains.map { it.name }}"
  }
  val bin = hostToolchains.single().resolve("bin")
  check(bin.isDirectory) { "Missing NDK LLVM bin directory: $bin" }
  return bin
}

val debugJavaClasses =
  layout.buildDirectory.dir("intermediates/javac/debug/compileDebugJavaWithJavac/classes")

val packageNativeLibraries =
  tasks.register("packageNativeLibraries") {
    group = "build"
    description = "Packages Android JNI libraries for all ABIs into the AAR."
  }

androidAbis.forEach { abiConfig ->
  val jniBridgeBuildDir =
    layout.buildDirectory.dir(
      "intermediates/javacpp/${abiConfig.javaCppPlatform}/org/maplibre/nativejni/internal/javacpp"
    )
  val jniBridgeLibrary = jniBridgeBuildDir.map {
    it.file(System.mapLibraryName("jniMaplibreNativeC"))
  }
  val jniLibsDir = jniLibsRoot.map { it.dir(abiConfig.abi) }

  val buildJavaCppNative =
    tasks.register<JavaExec>("buildJavaCppNative${abiConfig.abi}") {
      group = "build"
      description = "Builds the JavaCPP JNI bridge for ${abiConfig.abi}."
      dependsOn("mergeDebugNativeLibs")
      mainClass = "org.bytedeco.javacpp.tools.Builder"
      doFirst {
        val coreBuildDir = coreNativeLibDir(abiConfig.abi).get()
        val coreLibrary = coreBuildDir.file("libmaplibre-native-c.so").asFile
        require(coreLibrary.isFile) {
          "Missing ${coreLibrary.absolutePath}; run :bindings:java-jni:assembleDebug first"
        }
        classpath = files(debugJavaClasses) + javacppHostClasspath
        val ndkRootFile = androidNdkRoot.get().asFile
        val ndkRoot = ndkRootFile.absolutePath
        val compiler =
          ndkLlvmBinDir(ndkRootFile)
            .resolve("${abiConfig.ndkClangTriple}${androidApiLevel.get()}-clang++")
            .absolutePath
        args(
          "-classpath",
          debugJavaClasses.get().asFile.absolutePath,
          "-properties",
          abiConfig.javaCppPlatform,
          "-Dplatform.root=$ndkRoot",
          "-Dplatform.linkpath=${coreBuildDir.asFile.absolutePath}",
          "-Dplatform.compiler=$compiler",
          "-d",
          jniBridgeBuildDir.get().asFile.absolutePath,
          "org.maplibre.nativejni.internal.javacpp.MaplibreNativeC",
        )
      }
      inputs.dir(debugJavaClasses)
      inputs.dir(androidNdkRoot)
      inputs.dir(rootProject.layout.projectDirectory.dir("include"))
      inputs.dir(coreNativeLibDir(abiConfig.abi))
      outputs.file(jniBridgeLibrary)
    }

  val packageAbiNativeLibraries =
    tasks.register<Copy>("packageNativeLibraries${abiConfig.abi}") {
      group = "build"
      description = "Packages ${abiConfig.abi} JNI bridge libraries for the AAR."
      dependsOn(buildJavaCppNative)
      into(jniLibsDir)
      from(jniBridgeLibrary) { rename { System.mapLibraryName("jniMaplibreNativeC") } }
    }

  packageNativeLibraries.configure { dependsOn(packageAbiNativeLibraries) }
}

afterEvaluate {
  android.sourceSets.named("main") {
    java.srcDir(generatedJavaCppSources.get().asFile)
    jniLibs.srcDir(jniLibsRoot.get().asFile)
  }

  val compileJavaTask = tasks.named("compileDebugJavaWithJavac")
  compileJavaTask.configure { dependsOn(generateJavaCppBindings) }
  tasks
    .matching { it.name.contains("Annotations") }
    .configureEach { dependsOn(generateJavaCppBindings) }
  androidAbis.forEach { abiConfig ->
    tasks.named("buildJavaCppNative${abiConfig.abi}").configure { dependsOn(compileJavaTask) }
  }
  tasks.named("mergeDebugJniLibFolders").configure { dependsOn(packageNativeLibraries) }
  tasks.named("mergeReleaseJniLibFolders").configure { dependsOn(packageNativeLibraries) }
}

tasks.register<Javadoc>("javadoc") {
  dependsOn(generateJavaCppBindings)
  val mainSources = layout.projectDirectory.dir("src/main/java")
  source = fileTree(mainSources) { exclude("org/maplibre/nativejni/internal/**") }
  doFirst {
    val sdkRoot = android.sdkComponents.sdkDirectory.get().asFile
    classpath = files(sdkRoot.resolve("platforms/android-34/android.jar"))
  }
  isFailOnError = true
  options.encoding = "UTF-8"
  (options as StandardJavadocDocletOptions).links("https://developer.android.com/reference/")
}
