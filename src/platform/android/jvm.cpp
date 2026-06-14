#include "jni.hpp"

extern "C" JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM* vm, void* /*reserved*/) {
  mbgl::android::theJVM = vm;
  return JNI_VERSION_1_6;
}
