plugins { id("com.android.library") }

repositories {
  google()
  mavenCentral()
}

val curlVersion = "8.8.0"
val androidNativeApiLevel = "30"
val androidNdkVersion = "28.1.13356709"

android {
  namespace = "org.maplibre.nativeffi"
  compileSdk = 34
  ndkVersion = androidNdkVersion

  buildFeatures { prefab = true }

  defaultConfig {
    minSdk = androidNativeApiLevel.toInt()
    ndk { abiFilters += listOf("arm64-v8a", "x86_64") }

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

  packaging { jniLibs { pickFirsts += "**/libc++_shared.so" } }
}

dependencies { implementation("io.github.vvb2060.ndk:curl:$curlVersion") }
