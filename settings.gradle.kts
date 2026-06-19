pluginManagement {
  repositories {
    google()
    mavenCentral()
    gradlePluginPortal()
  }
}

rootProject.name = "maplibre-native-ffi"

include(":bindings:java-jni")

include(":bindings:kotlin")

include(":examples:lwjgl-map")
