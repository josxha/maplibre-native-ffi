pluginManagement {
  repositories {
    google()
    mavenCentral()
    gradlePluginPortal()
  }
}

fun resolveAndroidSdkDir(): String? {
  System.getenv("ANDROID_HOME")?.let {
    return it
  }
  System.getenv("ANDROID_SDK_ROOT")?.let {
    return it
  }
  val localProperties = java.util.Properties()
  val localPropertiesFile = file("local.properties")
  if (!localPropertiesFile.isFile) {
    return null
  }
  localPropertiesFile.inputStream().use { localProperties.load(it) }
  return localProperties.getProperty("sdk.dir")
}

rootProject.name = "maplibre-native-ffi"

include(":bindings:java-ffm")

if (resolveAndroidSdkDir() != null) {
  include(":bindings:java-jni")
}

include(":bindings:kotlin-native")

include(":examples:lwjgl-map")
