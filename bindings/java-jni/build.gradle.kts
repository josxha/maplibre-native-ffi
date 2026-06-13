import org.gradle.api.tasks.compile.JavaCompile
import org.gradle.api.tasks.javadoc.Javadoc
import org.gradle.external.javadoc.StandardJavadocDocletOptions
import org.gradle.internal.os.OperatingSystem

plugins { id("com.android.library") }

repositories {
  google()
  mavenCentral()
}

val javaCppPlatformName = "android-arm64"
val androidAbi = "arm64-v8a"
val androidApiLevel =
  providers
    .environmentVariable("MLN_FFI_ANDROID_PLATFORM")
    .map { it.removePrefix("android-") }
    .orElse("30")

android {
  namespace = "org.maplibre.nativejni"
  compileSdk = 34

  defaultConfig {
    minSdk = 30
    testInstrumentationRunner = "de.mannodermaus.junit5.AndroidJUnit5Runner"

    ndk { abiFilters += androidAbi }
  }

  compileOptions {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
  }

  testOptions { animationsDisabled = true }

  packaging { jniLibs { pickFirsts += "**/libc++_shared.so" } }
}

dependencies {
  implementation("org.bytedeco:javacpp:1.5.11")

  androidTestImplementation(platform("org.junit:junit-bom:6.0.3"))
  androidTestImplementation("org.junit.jupiter:junit-jupiter")
  androidTestImplementation("androidx.test:runner:1.6.2")
  androidTestImplementation("androidx.test:rules:1.6.1")
  androidTestImplementation("de.mannodermaus.junit5:android-test-core:1.7.0")
  androidTestRuntimeOnly("de.mannodermaus.junit5:android-test-runner:1.7.0")
}

val javacppHostClasspath =
  configurations.detachedConfiguration(dependencies.create("org.bytedeco:javacpp:1.5.11"))
val generatedJavaCppSources = layout.buildDirectory.dir("generated/sources/javacpp/main/java")
val javaCppConfigClasses = layout.buildDirectory.dir("classes/javacppConfig")
val jniLibsRoot = layout.buildDirectory.dir("generated/jniLibs")
val jniLibsDir = jniLibsRoot.map { it.dir(androidAbi) }

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
      "-properties",
      javaCppPlatformName,
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

val nativeBuildDir = providers.environmentVariable("MLN_FFI_BUILD_DIR")
val androidNdkRoot = providers.environmentVariable("ANDROID_NDK_ROOT")

fun ndkPrebuiltHost(): String {
  val os = OperatingSystem.current()
  return when {
    os.isLinux -> "linux-x86_64"
    os.isMacOsX ->
      if (System.getProperty("os.arch") == "aarch64") "darwin-arm64" else "darwin-x86_64"
    else -> throw GradleException("Android JNI builds require a Linux or macOS host")
  }
}

val debugJavaClasses =
  layout.buildDirectory.dir("intermediates/javac/debug/compileDebugJavaWithJavac/classes")
val jniBridgeBuildDir =
  layout.buildDirectory.dir(
    "intermediates/javacpp/$javaCppPlatformName/org/maplibre/nativejni/internal/javacpp"
  )
val jniBridgeLibrary = jniBridgeBuildDir.map {
  it.file(System.mapLibraryName("jniMaplibreNativeC"))
}

val javaCppCompatInclude = layout.projectDirectory.dir("cpp")

val buildJavaCppNative =
  tasks.register<JavaExec>("buildJavaCppNative") {
    group = "build"
    description = "Builds the JavaCPP JNI bridge for Android arm64."
    mainClass = "org.bytedeco.javacpp.tools.Builder"
    doFirst {
      require(androidNdkRoot.orNull?.isNotBlank() == true) {
        "ANDROID_NDK_ROOT is required to build the JNI bridge"
      }
      require(nativeBuildDir.orNull?.isNotBlank() == true) {
        "MLN_FFI_BUILD_DIR is required to build the JNI bridge"
      }
      classpath = files(debugJavaClasses) + javacppHostClasspath
      val ndkRoot = androidNdkRoot.get()
      val compiler =
        "$ndkRoot/toolchains/llvm/prebuilt/${ndkPrebuiltHost()}/bin/aarch64-linux-android${androidApiLevel.get()}-clang++"
      args(
        "-classpath",
        debugJavaClasses.get().asFile.absolutePath,
        "-properties",
        javaCppPlatformName,
        "-Dplatform.root=$ndkRoot",
        "-Dplatform.includepath=${javaCppCompatInclude.asFile.absolutePath}",
        "-Dplatform.linkpath=${nativeBuildDir.get()}",
        "-Dplatform.compiler=$compiler",
        "-Dplatform.compiler.default=-O3 -s -include javacpp_char_traits.hpp",
        "-d",
        jniBridgeBuildDir.get().asFile.absolutePath,
        "org.maplibre.nativejni.internal.javacpp.MaplibreNativeC",
      )
    }
    inputs.dir(debugJavaClasses)
    inputs.dir(rootProject.layout.projectDirectory.dir("include"))
    inputs.dir(nativeBuildDir)
    outputs.file(jniBridgeLibrary)
  }

val packageNativeLibraries =
  tasks.register<Copy>("packageNativeLibraries") {
    group = "build"
    description = "Packages Android JNI libraries for the AAR."
    dependsOn(buildJavaCppNative)
    into(jniLibsDir)
    from(jniBridgeLibrary) { rename { System.mapLibraryName("jniMaplibreNativeC") } }
    from(nativeBuildDir) {
      include("libmaplibre-native-c.so")
      onlyIf { nativeBuildDir.isPresent }
    }
    doFirst {
      require(nativeBuildDir.isPresent) {
        "MLN_FFI_BUILD_DIR is required to package native libraries"
      }
      val coreLibrary = file("${nativeBuildDir.get()}/libmaplibre-native-c.so")
      require(coreLibrary.isFile) {
        "Missing ${coreLibrary.absolutePath}; run mise -E android-arm64-egl run build first"
      }
    }
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
  buildJavaCppNative.configure { dependsOn(compileJavaTask) }
  tasks.named("mergeDebugJniLibFolders").configure { dependsOn(packageNativeLibraries) }
  tasks.named("mergeReleaseJniLibFolders").configure { dependsOn(packageNativeLibraries) }
}

tasks.register<Javadoc>("javadoc") {
  dependsOn(generateJavaCppBindings)
  val mainSources = layout.projectDirectory.dir("src/main/java")
  source = fileTree(mainSources) { exclude("org/maplibre/nativejni/internal/**") }
  doFirst {
    val sdkRoot =
      System.getenv("ANDROID_SDK_ROOT")
        ?: System.getenv("ANDROID_HOME")
        ?: error("ANDROID_SDK_ROOT or ANDROID_HOME is required for javadoc")
    classpath = files("$sdkRoot/platforms/android-34/android.jar")
  }
  isFailOnError = true
  options.encoding = "UTF-8"
  (options as StandardJavadocDocletOptions).links("https://developer.android.com/reference/")
}
