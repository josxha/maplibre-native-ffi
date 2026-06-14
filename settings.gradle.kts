pluginManagement {
  repositories {
    google()
    mavenCentral()
    gradlePluginPortal()
  }
  plugins {
    id("com.android.library") version "9.1.1"
    id("org.jetbrains.kotlin.multiplatform") version "2.2.21"
    id("de.infolektuell.jextract") version "1.4.0"
  }
}

rootProject.name = "maplibre-native-ffi"

include(":bindings:java-ffm")

include(":bindings:java-jni")

include(":bindings:kotlin-native")

include(":examples:lwjgl-map")
