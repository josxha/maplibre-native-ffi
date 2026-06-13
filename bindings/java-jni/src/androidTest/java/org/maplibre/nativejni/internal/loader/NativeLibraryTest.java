package org.maplibre.nativejni.internal.loader;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;
import org.maplibre.nativejni.Maplibre;

final class NativeLibraryTest {
  @Test
  void exposesDocumentedLookupInputs() {
    assertFalse(NativeLibrary.LIBRARY_PATH_PROPERTY.isBlank());
    assertFalse(NativeLibrary.LIBRARY_PATH_ENV.isBlank());
    assertEquals("jniMaplibreNativeC", NativeLibrary.LIBRARY_NAME);
    assertEquals("libjniMaplibreNativeC.so", System.mapLibraryName(NativeLibrary.LIBRARY_NAME));
  }

  @Test
  void loadedLibraryServesCAbiCalls() {
    assertTrue(Maplibre.cVersion() >= 0);
    NativeLibrary.ensureLoaded();
  }
}
