package org.maplibre.nativejni.test;

import android.opengl.EGL14;
import android.opengl.EGLConfig;
import android.opengl.EGLContext;
import android.opengl.EGLDisplay;
import android.opengl.EGLExt;
import android.opengl.EGLObjectHandle;
import android.opengl.EGLSurface;
import android.opengl.GLES20;
import android.opengl.GLES30;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import org.maplibre.nativejni.Maplibre;
import org.maplibre.nativejni.map.MapHandle;
import org.maplibre.nativejni.map.MapOptions;
import org.maplibre.nativejni.render.EglContextDescriptor;
import org.maplibre.nativejni.render.NativePointer;
import org.maplibre.nativejni.render.OpenGLBorrowedTextureDescriptor;
import org.maplibre.nativejni.render.OpenGLContextDescriptor;
import org.maplibre.nativejni.render.OpenGLContextProvider;
import org.maplibre.nativejni.render.OpenGLOwnedTextureDescriptor;
import org.maplibre.nativejni.render.OpenGLSurfaceDescriptor;
import org.maplibre.nativejni.render.RenderSessionHandle;
import org.maplibre.nativejni.render.RenderTargetExtent;
import org.maplibre.nativejni.runtime.RuntimeHandle;

/** Shared helpers for render-target tests that need a small local map. */
public final class RenderTargetTestSupport implements AutoCloseable {
  private static final int GL_TEXTURE_2D = 0x0DE1;
  private static final int GL_RGBA = 0x1908;
  private static final int GL_RGBA8 = 0x8058;
  private static final int GL_UNSIGNED_BYTE = 0x1401;
  private static final int GL_NEAREST = 0x2600;
  private static final int GL_TEXTURE_MIN_FILTER = 0x2801;
  private static final int GL_TEXTURE_MAG_FILTER = 0x2800;
  private static final int GL_FRAMEBUFFER = 0x8D40;
  private static final int GL_COLOR_ATTACHMENT0 = 0x8CE0;
  private static final int GL_FRAMEBUFFER_COMPLETE = 0x8B5B;

  private final RenderSessionHandle session;
  private final AutoCloseable context;
  private boolean closed;

  private RenderTargetTestSupport(RenderSessionHandle session, AutoCloseable context) {
    this.session = session;
    this.context = context;
  }

  public static MapHandle createSmallMap(RuntimeHandle runtime) {
    return MapHandle.create(runtime, new MapOptions().size(64, 64));
  }

  public static RenderTargetTestSupport attachOpenGLOwnedTexture(
      MapHandle map, RenderTargetExtent extent) {
    var context = OpenGLTestContext.create(8, 8);
    try {
      return new RenderTargetTestSupport(
          map.attachOpenGLOwnedTexture(
              new OpenGLOwnedTextureDescriptor().extent(extent).context(context.descriptor())),
          context);
    } catch (RuntimeException | Error error) {
      closeContextAfterAttachFailure(context, error);
      throw error;
    }
  }

  public static RenderTargetTestSupport attachOpenGLBorrowedTexture(
      MapHandle map, RenderTargetExtent extent) {
    var context = OpenGLBorrowedTextureContext.create(extent.width(), extent.height());
    try {
      return new RenderTargetTestSupport(
          map.attachOpenGLBorrowedTexture(
              new OpenGLBorrowedTextureDescriptor()
                  .extent(extent)
                  .context(context.descriptor())
                  .texture(context.texture())
                  .target(GL_TEXTURE_2D)),
          context);
    } catch (RuntimeException | Error error) {
      closeContextAfterAttachFailure(context, error);
      throw error;
    }
  }

  public static RenderTargetTestSupport attachOpenGLSurface(
      MapHandle map, RenderTargetExtent extent) {
    var context = OpenGLTestContext.create(extent.width(), extent.height());
    try {
      return new RenderTargetTestSupport(
          map.attachOpenGLSurface(
              new OpenGLSurfaceDescriptor()
                  .extent(extent)
                  .context(context.descriptor())
                  .surface(context.surface())),
          context);
    } catch (RuntimeException | Error error) {
      closeContextAfterAttachFailure(context, error);
      throw error;
    }
  }

  private static void closeContextAfterAttachFailure(AutoCloseable context, Throwable failure) {
    try {
      context.close();
    } catch (Exception closeError) {
      failure.addSuppressed(closeError);
    }
  }

  public RenderSessionHandle session() {
    if (closed) {
      throw new IllegalStateException("RenderTargetTestSupport is closed");
    }
    return session;
  }

  public byte[] readOpenGLBorrowedTextureRgba() {
    if (closed) {
      throw new IllegalStateException("RenderTargetTestSupport is closed");
    }
    if (context instanceof OpenGLBorrowedTextureContext openglContext) {
      return openglContext.readRgba();
    }
    throw new IllegalStateException("Render target is not an OpenGL borrowed texture");
  }

  public byte[] readOpenGLSurfaceRgba(int width, int height) {
    if (closed) {
      throw new IllegalStateException("RenderTargetTestSupport is closed");
    }
    if (context instanceof OpenGLTestContext openglContext) {
      return openglContext.readSurfaceRgba(width, height);
    }
    throw new IllegalStateException("Render target is not an OpenGL surface");
  }

  @Override
  public void close() throws Exception {
    if (closed) {
      return;
    }
    closed = true;
    try {
      if (!session.isClosed()) {
        session.close();
      }
    } finally {
      context.close();
    }
  }

  private static class OpenGLTestContext implements AutoCloseable {
    protected EGLDisplay eglDisplay;
    protected EGLConfig eglConfig;
    protected EGLSurface eglSurface;
    protected EGLContext eglContext;
    protected boolean closed;

    static OpenGLTestContext create(int width, int height) {
      if (!Maplibre.supportedOpenGLContextProviders().contains(OpenGLContextProvider.EGL)) {
        throw new IllegalStateException("Native library does not support EGL");
      }

      var context = new OpenGLTestContext();
      context.eglDisplay = EGL14.eglGetDisplay(EGL14.EGL_DEFAULT_DISPLAY);
      if (context.eglDisplay == EGL14.EGL_NO_DISPLAY) {
        throw new IllegalStateException("EGL display unavailable");
      }
      var version = new int[2];
      if (!EGL14.eglInitialize(context.eglDisplay, version, 0, version, 1)) {
        throw new IllegalStateException("EGL initialization failed");
      }
      if (!EGL14.eglBindAPI(EGL14.EGL_OPENGL_ES_API)) {
        throw new IllegalStateException("EGL OpenGL ES API binding failed");
      }

      var configAttributes =
          new int[] {
            EGL14.EGL_SURFACE_TYPE,
            EGL14.EGL_PBUFFER_BIT,
            EGL14.EGL_RENDERABLE_TYPE,
            EGLExt.EGL_OPENGL_ES3_BIT_KHR,
            EGL14.EGL_RED_SIZE,
            8,
            EGL14.EGL_GREEN_SIZE,
            8,
            EGL14.EGL_BLUE_SIZE,
            8,
            EGL14.EGL_ALPHA_SIZE,
            8,
            EGL14.EGL_DEPTH_SIZE,
            24,
            EGL14.EGL_STENCIL_SIZE,
            8,
            EGL14.EGL_NONE
          };
      var configs = new EGLConfig[1];
      var configCount = new int[1];
      if (!EGL14.eglChooseConfig(
              context.eglDisplay, configAttributes, 0, configs, 0, configs.length, configCount, 0)
          || configCount[0] == 0) {
        throw new IllegalStateException("EGL config unavailable");
      }
      context.eglConfig = configs[0];

      var contextAttributes = new int[] {EGL14.EGL_CONTEXT_CLIENT_VERSION, 3, EGL14.EGL_NONE};
      context.eglContext =
          EGL14.eglCreateContext(
              context.eglDisplay, context.eglConfig, EGL14.EGL_NO_CONTEXT, contextAttributes, 0);
      if (context.eglContext == EGL14.EGL_NO_CONTEXT) {
        throw new IllegalStateException("EGL context creation failed");
      }

      var surfaceAttributes =
          new int[] {EGL14.EGL_WIDTH, width, EGL14.EGL_HEIGHT, height, EGL14.EGL_NONE};
      context.eglSurface =
          EGL14.eglCreatePbufferSurface(
              context.eglDisplay, context.eglConfig, surfaceAttributes, 0);
      if (context.eglSurface == EGL14.EGL_NO_SURFACE) {
        throw new IllegalStateException("EGL pbuffer creation failed");
      }

      context.makeCurrent();
      return context;
    }

    final OpenGLContextDescriptor descriptor() {
      return new EglContextDescriptor(
          NativePointer.ofAddress(nativeHandle(eglDisplay)),
          NativePointer.ofAddress(eglConfigHandle(eglConfig)),
          NativePointer.ofAddress(nativeHandle(eglContext)));
    }

    final NativePointer surface() {
      return NativePointer.ofAddress(nativeHandle(eglSurface));
    }

    final void makeCurrent() {
      if (closed) {
        throw new IllegalStateException("OpenGL test context is closed");
      }
      if (!EGL14.eglMakeCurrent(eglDisplay, eglSurface, eglSurface, eglContext)) {
        throw new IllegalStateException("EGL make-current failed");
      }
    }

    final void checkGlError(String operation) {
      var error = GLES20.glGetError();
      if (error != GLES20.GL_NO_ERROR) {
        throw new IllegalStateException(
            String.format("%s failed with OpenGL error 0x%x", operation, error));
      }
    }

    final byte[] readSurfaceRgba(int width, int height) {
      makeCurrent();
      var pixels = ByteBuffer.allocateDirect(width * height * 4).order(ByteOrder.nativeOrder());
      GLES20.glReadPixels(0, 0, width, height, GL_RGBA, GL_UNSIGNED_BYTE, pixels);
      checkGlError("read OpenGL surface");
      var bytes = new byte[pixels.capacity()];
      pixels.rewind();
      pixels.get(bytes);
      return bytes;
    }

    @Override
    public void close() {
      if (closed) {
        return;
      }
      closed = true;
      if (eglDisplay != EGL14.EGL_NO_DISPLAY) {
        EGL14.eglMakeCurrent(
            eglDisplay, EGL14.EGL_NO_SURFACE, EGL14.EGL_NO_SURFACE, EGL14.EGL_NO_CONTEXT);
        if (eglSurface != EGL14.EGL_NO_SURFACE) {
          EGL14.eglDestroySurface(eglDisplay, eglSurface);
          eglSurface = EGL14.EGL_NO_SURFACE;
        }
        if (eglContext != EGL14.EGL_NO_CONTEXT) {
          EGL14.eglDestroyContext(eglDisplay, eglContext);
          eglContext = EGL14.EGL_NO_CONTEXT;
        }
        EGL14.eglTerminate(eglDisplay);
        eglDisplay = EGL14.EGL_NO_DISPLAY;
      }
    }

    private static long nativeHandle(EGLObjectHandle handle) {
      return handle.getNativeHandle();
    }

    private static long eglConfigHandle(EGLConfig config) {
      try {
        var field = config.getClass().getDeclaredField("mEGLConfig");
        field.setAccessible(true);
        return field.getLong(config);
      } catch (ReflectiveOperationException error) {
        throw new IllegalStateException("Unable to read EGLConfig native handle", error);
      }
    }
  }

  private static final class OpenGLBorrowedTextureContext extends OpenGLTestContext {
    private int texture;
    private int width;
    private int height;

    static OpenGLBorrowedTextureContext create(int width, int height) {
      var base = OpenGLTestContext.create(width, height);
      var context = new OpenGLBorrowedTextureContext();
      context.eglDisplay = base.eglDisplay;
      context.eglConfig = base.eglConfig;
      context.eglSurface = base.eglSurface;
      context.eglContext = base.eglContext;
      base.eglDisplay = EGL14.EGL_NO_DISPLAY;
      base.eglConfig = null;
      base.eglSurface = EGL14.EGL_NO_SURFACE;
      base.eglContext = EGL14.EGL_NO_CONTEXT;
      base.closed = true;
      context.width = width;
      context.height = height;
      try {
        context.createTexture();
        return context;
      } catch (RuntimeException error) {
        context.close();
        throw error;
      }
    }

    int texture() {
      return texture;
    }

    byte[] readRgba() {
      makeCurrent();
      var pixels = ByteBuffer.allocateDirect(width * height * 4).order(ByteOrder.nativeOrder());
      bindTexture(texture);
      readTexture(pixels);
      bindTexture(0);
      checkGlError("read OpenGL borrowed texture");
      var bytes = new byte[pixels.capacity()];
      pixels.rewind();
      pixels.get(bytes);
      return bytes;
    }

    @Override
    public void close() {
      if (texture != 0) {
        makeCurrent();
        GLES20.glDeleteTextures(1, new int[] {texture}, 0);
        texture = 0;
      }
      super.close();
    }

    private void createTexture() {
      makeCurrent();
      var textures = new int[1];
      GLES20.glGenTextures(1, textures, 0);
      texture = textures[0];
      bindTexture(texture);
      GLES20.glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
      GLES20.glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
      GLES30.glTexImage2D(
          GL_TEXTURE_2D, 0, GL_RGBA8, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE, null);
      bindTexture(0);
      checkGlError("create OpenGL borrowed texture");
    }

    private void bindTexture(int value) {
      GLES20.glBindTexture(GL_TEXTURE_2D, value);
    }

    private void readTexture(ByteBuffer pixels) {
      var framebuffer = new int[1];
      GLES20.glGenFramebuffers(1, framebuffer, 0);
      GLES20.glBindFramebuffer(GL_FRAMEBUFFER, framebuffer[0]);
      try {
        GLES20.glFramebufferTexture2D(
            GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, texture, 0);
        if (GLES20.glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE) {
          throw new IllegalStateException("OpenGL ES borrowed texture framebuffer incomplete");
        }
        GLES20.glReadPixels(0, 0, width, height, GL_RGBA, GL_UNSIGNED_BYTE, pixels);
      } finally {
        GLES20.glBindFramebuffer(GL_FRAMEBUFFER, 0);
        GLES20.glDeleteFramebuffers(1, framebuffer, 0);
      }
    }
  }
}
