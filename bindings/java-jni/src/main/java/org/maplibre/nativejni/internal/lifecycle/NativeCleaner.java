package org.maplibre.nativejni.internal.lifecycle;

import java.lang.ref.PhantomReference;
import java.lang.ref.ReferenceQueue;
import java.util.Objects;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Minimal {@link java.lang.ref.Cleaner} replacement for Android API levels below 33.
 *
 * <p>Registers phantom-reachability cleanup actions and supports explicit {@link Cleanable#clean()}
 * calls.
 */
public final class NativeCleaner {
  private final ReferenceQueue<Object> queue = new ReferenceQueue<>();
  private final ConcurrentHashMap<PhantomReference<Object>, ActionRef> actions =
      new ConcurrentHashMap<>();

  private NativeCleaner() {
    var thread = new Thread(this::runLoop, "maplibre-nativejni-cleaner");
    thread.setDaemon(true);
    thread.start();
  }

  public static NativeCleaner create() {
    return new NativeCleaner();
  }

  public Cleanable register(Object object, Runnable action) {
    Objects.requireNonNull(object, "object");
    var actionRef = new ActionRef(Objects.requireNonNull(action, "action"));
    var reference = new PhantomReference<>(object, queue);
    actions.put(reference, actionRef);
    return () -> {
      actions.remove(reference);
      reference.clear();
      actionRef.runOnce();
    };
  }

  private void runLoop() {
    while (true) {
      try {
        @SuppressWarnings("unchecked")
        var reference = (PhantomReference<Object>) queue.remove();
        var actionRef = actions.remove(reference);
        if (actionRef != null) {
          actionRef.runOnce();
        }
      } catch (InterruptedException interrupted) {
        Thread.currentThread().interrupt();
        return;
      }
    }
  }

  @FunctionalInterface
  public interface Cleanable {
    void clean();
  }

  private static final class ActionRef {
    private final Runnable action;
    private final AtomicBoolean ran = new AtomicBoolean();

    private ActionRef(Runnable action) {
      this.action = action;
    }

    private void runOnce() {
      if (ran.compareAndSet(false, true)) {
        action.run();
      }
    }
  }
}
