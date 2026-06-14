package org.maplibre.nativejni.render;

import org.maplibre.nativejni.internal.lifecycle.NativeCleaner;

final class FrameHandleLeakReport implements Runnable {
  private static final NativeCleaner CLEANER = NativeCleaner.create();

  private final String typeName;
  private volatile boolean closed;

  private FrameHandleLeakReport(String typeName) {
    this.typeName = typeName;
  }

  static Registration register(Object owner, String typeName) {
    var report = new FrameHandleLeakReport(typeName);
    return new Registration(report, CLEANER.register(owner, report));
  }

  void markClosed() {
    closed = true;
  }

  @Override
  public void run() {
    if (!closed) {
      System.err.printf(
          "Leaked %s; close frame handles explicitly on the render session owner thread.%n",
          typeName);
    }
  }

  record Registration(FrameHandleLeakReport report, NativeCleaner.Cleanable cleanable) {}
}
