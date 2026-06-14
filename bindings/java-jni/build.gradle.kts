import org.gradle.api.file.Directory
import org.gradle.api.tasks.compile.JavaCompile
import org.gradle.api.tasks.javadoc.Javadoc
import org.gradle.external.javadoc.StandardJavadocDocletOptions
import org.gradle.internal.os.OperatingSystem

plugins { id("com.android.library") }

repositories {
  google()
  mavenCentral()
}

data class AndroidJniAbi(
  val abi: String,
  val javaCppPlatform: String,
  val ndkClangTriple: String,
  val nativeBuildDirName: String,
)

val androidAbis =
  listOf(
    AndroidJniAbi("arm64-v8a", "android-arm64", "aarch64-linux-android", "android-arm64-egl"),
    AndroidJniAbi("x86_64", "android-x86_64", "x86_64-linux-android", "android-x64-egl"),
  )

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
    testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    testInstrumentationRunnerArguments["runnerBuilder"] =
      "de.mannodermaus.junit5.AndroidJUnit5Builder"

    ndk { abiFilters += androidAbis.map { it.abi } }
  }

  compileOptions {
    isCoreLibraryDesugaringEnabled = true
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
  coreLibraryDesugaring("com.android.tools:desugar_jdk_libs:2.1.4")
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

fun nativeBuildDir(config: AndroidJniAbi): Directory =
  rootProject.layout.projectDirectory.dir("build/${config.nativeBuildDirName}")

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
      mainClass = "org.bytedeco.javacpp.tools.Builder"
      doFirst {
        require(androidNdkRoot.orNull?.isNotBlank() == true) {
          "ANDROID_NDK_ROOT is required to build the JNI bridge"
        }
        val coreBuildDir = nativeBuildDir(abiConfig)
        require(coreBuildDir.asFile.isDirectory) {
          "Missing ${coreBuildDir.asFile.absolutePath}; run mise -E ${abiConfig.nativeBuildDirName} run build first"
        }
        val coreLibrary = coreBuildDir.file("libmaplibre-native-c.so").asFile
        require(coreLibrary.isFile) {
          "Missing ${coreLibrary.absolutePath}; run mise -E ${abiConfig.nativeBuildDirName} run build first"
        }
        classpath = files(debugJavaClasses) + javacppHostClasspath
        val ndkRoot = androidNdkRoot.get()
        val compiler =
          "$ndkRoot/toolchains/llvm/prebuilt/${ndkPrebuiltHost()}/bin/${abiConfig.ndkClangTriple}${androidApiLevel.get()}-clang++"
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
      inputs.dir(rootProject.layout.projectDirectory.dir("include"))
      inputs.dir(nativeBuildDir(abiConfig))
      outputs.file(jniBridgeLibrary)
    }

  val packageAbiNativeLibraries =
    tasks.register<Copy>("packageNativeLibraries${abiConfig.abi}") {
      group = "build"
      description = "Packages ${abiConfig.abi} JNI libraries for the AAR."
      dependsOn(buildJavaCppNative)
      into(jniLibsDir)
      from(jniBridgeLibrary) { rename { System.mapLibraryName("jniMaplibreNativeC") } }
      from(nativeBuildDir(abiConfig)) { include("libmaplibre-native-c.so") }
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
