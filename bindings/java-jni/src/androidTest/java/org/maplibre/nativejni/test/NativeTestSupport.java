package org.maplibre.nativejni.test;

import org.maplibre.nativejni.Maplibre;

public final class NativeTestSupport {
  private NativeTestSupport() {}

  public static void loadNativeLibrary() {
    Maplibre.loadNativeLibrary();
  }
}
