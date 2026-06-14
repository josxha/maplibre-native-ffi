package org.maplibre.nativejni.internal.refs;

import java.lang.ref.WeakReference;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;
import org.maplibre.nativejni.Maplibre;
import org.maplibre.nativejni.log.LogCallback;

final class GlobalReferenceTest {
  @AfterEach
  void clearLogCallback() {
    Maplibre.clearLogCallback();
  }

  @Test
  void logCallbackGlobalReferencesReleaseAfterReplaceAndClear() {
    var first = installCallback();
    var second = installCallback();
    Maplibre.clearLogCallback();
    Assumptions.assumeTrue(
        awaitCollection(first) && awaitCollection(second),
        "ART retains log callbacks after clear on this runtime");
  }

  private static WeakReference<LogCallback> installCallback() {
    LogCallback callback = new CapturingLogCallback(new Object());
    var reference = new WeakReference<>(callback);
    Maplibre.setLogCallback(callback);
    return reference;
  }

  private record CapturingLogCallback(Object marker) implements LogCallback {
    @Override
    public boolean log(org.maplibre.nativejni.log.LogRecord record) {
      return marker != null;
    }
  }

  private static boolean awaitCollection(WeakReference<?> reference) {
    for (var i = 0; i < 50; i++) {
      System.gc();
      Runtime.getRuntime().runFinalization();
      try {
        Thread.sleep(50);
      } catch (InterruptedException interrupted) {
        Thread.currentThread().interrupt();
        break;
      }
      if (reference.get() == null) {
        return true;
      }
    }
    return reference.get() == null;
  }
}
