package org.maplibre.nativeffi.render

import org.bytedeco.javacpp.BytePointer
import org.bytedeco.javacpp.Pointer
import org.bytedeco.javacpp.PointerPointer
import org.maplibre.nativeffi.NativeAccess
import org.maplibre.nativeffi.error.MaplibreStatus
import org.maplibre.nativeffi.geo.Feature
import org.maplibre.nativeffi.internal.javacpp.MaplibreNativeC
import org.maplibre.nativeffi.internal.lifecycle.HandleStateCore
import org.maplibre.nativeffi.internal.status.Status
import org.maplibre.nativeffi.json.JsonValue
import org.maplibre.nativeffi.map.MapHandle
import org.maplibre.nativeffi.query.FeatureExtensionResult
import org.maplibre.nativeffi.query.FeatureStateSelector
import org.maplibre.nativeffi.query.QueriedFeature
import org.maplibre.nativeffi.query.RenderedFeatureQueryOptions
import org.maplibre.nativeffi.query.RenderedQueryGeometry
import org.maplibre.nativeffi.query.SourceFeatureQueryOptions

/** Owned Android JNI render session handle. Close it on the map owner thread. */
public actual class RenderSessionHandle
private constructor(private val map: MapHandle, private val handleAddress: Long) : AutoCloseable {
  private val mapRetention = map.retainChild()
  private val core = HandleStateCore("RenderSessionHandle", handleAddress, map)

  public actual val isClosed: Boolean
    get() = core.isReleased()

  public actual fun map(): MapHandle = map

  public actual fun resize(width: Int, height: Int, scaleFactor: Double) {
    NativeAccess.ensureLoaded()
    Status.check(
      MaplibreNativeC.mln_render_session_resize(
        renderSession(requireLiveAddress()),
        width,
        height,
        scaleFactor,
      )
    )
  }

  public actual fun renderUpdate() {
    NativeAccess.ensureLoaded()
    Status.check(
      MaplibreNativeC.mln_render_session_render_update(renderSession(requireLiveAddress()))
    )
  }

  public actual fun detach() {
    NativeAccess.ensureLoaded()
    Status.check(MaplibreNativeC.mln_render_session_detach(renderSession(requireLiveAddress())))
  }

  public actual fun reduceMemoryUse() {
    NativeAccess.ensureLoaded()
    Status.check(
      MaplibreNativeC.mln_render_session_reduce_memory_use(renderSession(requireLiveAddress()))
    )
  }

  public actual fun clearData() {
    NativeAccess.ensureLoaded()
    Status.check(MaplibreNativeC.mln_render_session_clear_data(renderSession(requireLiveAddress())))
  }

  public actual fun dumpDebugLogs() {
    NativeAccess.ensureLoaded()
    Status.check(
      MaplibreNativeC.mln_render_session_dump_debug_logs(renderSession(requireLiveAddress()))
    )
  }

  public actual fun setFeatureState(selector: FeatureStateSelector, value: JsonValue) {
    unsupportedRenderSessionHandle()
  }

  public actual fun getFeatureState(selector: FeatureStateSelector): JsonValue =
    unsupportedRenderSessionHandle()

  public actual fun removeFeatureState(selector: FeatureStateSelector) {
    unsupportedRenderSessionHandle()
  }

  public actual fun queryRenderedFeatures(
    geometry: RenderedQueryGeometry,
    options: RenderedFeatureQueryOptions?,
  ): List<QueriedFeature> = unsupportedRenderSessionHandle()

  public actual fun querySourceFeatures(
    sourceId: String,
    options: SourceFeatureQueryOptions?,
  ): List<QueriedFeature> = unsupportedRenderSessionHandle()

  public actual fun queryFeatureExtension(
    sourceId: String,
    feature: Feature,
    extension: String,
    extensionField: String,
    arguments: JsonValue?,
  ): FeatureExtensionResult = unsupportedRenderSessionHandle()

  public actual fun textureImageInfo(): TextureImageInfo {
    NativeAccess.ensureLoaded()
    val outInfo = MaplibreNativeC.mln_texture_image_info_default()
    val status =
      MaplibreNativeC.mln_texture_read_premultiplied_rgba8(
        renderSession(requireLiveAddress()),
        null as BytePointer?,
        0L,
        outInfo,
      )
    val info = textureImageInfo(outInfo)
    if (
      status == MaplibreStatus.OK.nativeCode ||
        (status == MaplibreStatus.INVALID_ARGUMENT.nativeCode && info.byteLength > 0)
    ) {
      return info
    }
    Status.check(status)
    error("unreachable")
  }

  public actual fun readPremultipliedRgba8(buffer: NativeBuffer): TextureImageInfo {
    NativeAccess.ensureLoaded()
    val outInfo = MaplibreNativeC.mln_texture_image_info_default()
    Status.check(
      MaplibreNativeC.mln_texture_read_premultiplied_rgba8(
        renderSession(requireLiveAddress()),
        buffer.borrowBuffer(),
        buffer.byteLength(),
        outInfo,
      )
    )
    return textureImageInfo(outInfo)
  }

  public actual fun acquireMetalOwnedTextureFrame(): MetalOwnedTextureFrameHandle =
    unsupportedRenderSessionHandle()

  public actual fun acquireVulkanOwnedTextureFrame(): VulkanOwnedTextureFrameHandle =
    unsupportedRenderSessionHandle()

  public actual fun acquireOpenGLOwnedTextureFrame(): OpenGLOwnedTextureFrameHandle =
    unsupportedRenderSessionHandle()

  public actual override fun close() {
    NativeAccess.ensureLoaded()
    core.closeOnce(
      destroy = { MaplibreNativeC.mln_render_session_destroy(renderSession(handleAddress)) },
      afterSuccess = { mapRetention.close() },
    )
  }

  private fun requireLiveAddress(): Long {
    core.requireLive()
    return handleAddress
  }

  public companion object {
    internal fun attachMetalOwnedTexture(
      map: MapHandle,
      descriptor: MetalOwnedTextureDescriptor,
    ): RenderSessionHandle =
      attach(map) { outSession ->
        MaplibreNativeC.mln_metal_owned_texture_attach(
          map(map.nativeAddress()),
          metalOwnedTextureDescriptor(descriptor),
          outSession,
        )
      }

    internal fun attachMetalBorrowedTexture(
      map: MapHandle,
      descriptor: MetalBorrowedTextureDescriptor,
    ): RenderSessionHandle =
      attach(map) { outSession ->
        MaplibreNativeC.mln_metal_borrowed_texture_attach(
          map(map.nativeAddress()),
          metalBorrowedTextureDescriptor(descriptor),
          outSession,
        )
      }

    internal fun attachVulkanOwnedTexture(
      map: MapHandle,
      descriptor: VulkanOwnedTextureDescriptor,
    ): RenderSessionHandle =
      attach(map) { outSession ->
        MaplibreNativeC.mln_vulkan_owned_texture_attach(
          map(map.nativeAddress()),
          vulkanOwnedTextureDescriptor(descriptor),
          outSession,
        )
      }

    internal fun attachVulkanBorrowedTexture(
      map: MapHandle,
      descriptor: VulkanBorrowedTextureDescriptor,
    ): RenderSessionHandle =
      attach(map) { outSession ->
        MaplibreNativeC.mln_vulkan_borrowed_texture_attach(
          map(map.nativeAddress()),
          vulkanBorrowedTextureDescriptor(descriptor),
          outSession,
        )
      }

    internal fun attachOpenGLOwnedTexture(
      map: MapHandle,
      descriptor: OpenGLOwnedTextureDescriptor,
    ): RenderSessionHandle =
      attach(map) { outSession ->
        MaplibreNativeC.mln_opengl_owned_texture_attach(
          map(map.nativeAddress()),
          openglOwnedTextureDescriptor(descriptor),
          outSession,
        )
      }

    internal fun attachOpenGLBorrowedTexture(
      map: MapHandle,
      descriptor: OpenGLBorrowedTextureDescriptor,
    ): RenderSessionHandle =
      attach(map) { outSession ->
        MaplibreNativeC.mln_opengl_borrowed_texture_attach(
          map(map.nativeAddress()),
          openglBorrowedTextureDescriptor(descriptor),
          outSession,
        )
      }

    internal fun attachMetalSurface(
      map: MapHandle,
      descriptor: MetalSurfaceDescriptor,
    ): RenderSessionHandle =
      attach(map) { outSession ->
        MaplibreNativeC.mln_metal_surface_attach(
          map(map.nativeAddress()),
          metalSurfaceDescriptor(descriptor),
          outSession,
        )
      }

    internal fun attachVulkanSurface(
      map: MapHandle,
      descriptor: VulkanSurfaceDescriptor,
    ): RenderSessionHandle =
      attach(map) { outSession ->
        MaplibreNativeC.mln_vulkan_surface_attach(
          map(map.nativeAddress()),
          vulkanSurfaceDescriptor(descriptor),
          outSession,
        )
      }

    internal fun attachOpenGLSurface(
      map: MapHandle,
      descriptor: OpenGLSurfaceDescriptor,
    ): RenderSessionHandle =
      attach(map) { outSession ->
        MaplibreNativeC.mln_opengl_surface_attach(
          map(map.nativeAddress()),
          openglSurfaceDescriptor(descriptor),
          outSession,
        )
      }

    private fun attach(
      map: MapHandle,
      block: (PointerPointer<MaplibreNativeC.mln_render_session>) -> Int,
    ): RenderSessionHandle {
      NativeAccess.ensureLoaded()
      PointerPointer<MaplibreNativeC.mln_render_session>(1).use { outSession ->
        outSession.put(0, null as Pointer?)
        Status.check(block(outSession))
        val session = outSession.get(MaplibreNativeC.mln_render_session::class.java, 0)
        val address = if (session == null || session.isNull) 0L else session.address()
        require(address != 0L) { "render session attach returned a null session" }
        return RenderSessionHandle(map, address)
      }
    }
  }
}

private fun metalOwnedTextureDescriptor(
  descriptor: MetalOwnedTextureDescriptor
): MaplibreNativeC.mln_metal_owned_texture_descriptor =
  MaplibreNativeC.mln_metal_owned_texture_descriptor_default().apply {
    setExtent(extent(), descriptor.extent)
    context().device(pointerOrNull(descriptor.context.device))
  }

private fun metalBorrowedTextureDescriptor(
  descriptor: MetalBorrowedTextureDescriptor
): MaplibreNativeC.mln_metal_borrowed_texture_descriptor =
  MaplibreNativeC.mln_metal_borrowed_texture_descriptor_default().apply {
    setExtent(extent(), descriptor.extent)
    texture(pointerOrNull(descriptor.texture))
  }

private fun metalSurfaceDescriptor(
  descriptor: MetalSurfaceDescriptor
): MaplibreNativeC.mln_metal_surface_descriptor =
  MaplibreNativeC.mln_metal_surface_descriptor_default().apply {
    setExtent(extent(), descriptor.extent)
    context().device(pointerOrNull(descriptor.context.device))
    layer(pointerOrNull(descriptor.layer))
  }

private fun vulkanOwnedTextureDescriptor(
  descriptor: VulkanOwnedTextureDescriptor
): MaplibreNativeC.mln_vulkan_owned_texture_descriptor =
  MaplibreNativeC.mln_vulkan_owned_texture_descriptor_default().apply {
    setExtent(extent(), descriptor.extent)
    setVulkanContext(context(), descriptor.context)
  }

private fun vulkanBorrowedTextureDescriptor(
  descriptor: VulkanBorrowedTextureDescriptor
): MaplibreNativeC.mln_vulkan_borrowed_texture_descriptor =
  MaplibreNativeC.mln_vulkan_borrowed_texture_descriptor_default().apply {
    setExtent(extent(), descriptor.extent)
    setVulkanContext(context(), descriptor.context)
    image(pointerOrNull(descriptor.image))
    image_view(pointerOrNull(descriptor.imageView))
    format(descriptor.format)
    initial_layout(descriptor.initialLayout)
    descriptor.finalLayout?.let { final_layout(it) }
  }

private fun vulkanSurfaceDescriptor(
  descriptor: VulkanSurfaceDescriptor
): MaplibreNativeC.mln_vulkan_surface_descriptor =
  MaplibreNativeC.mln_vulkan_surface_descriptor_default().apply {
    setExtent(extent(), descriptor.extent)
    setVulkanContext(context(), descriptor.context)
    surface(pointerOrNull(descriptor.surface))
  }

private fun openglOwnedTextureDescriptor(
  descriptor: OpenGLOwnedTextureDescriptor
): MaplibreNativeC.mln_opengl_owned_texture_descriptor =
  MaplibreNativeC.mln_opengl_owned_texture_descriptor_default().apply {
    setExtent(extent(), descriptor.extent)
    setOpenGLContext(context(), descriptor.context)
  }

private fun openglBorrowedTextureDescriptor(
  descriptor: OpenGLBorrowedTextureDescriptor
): MaplibreNativeC.mln_opengl_borrowed_texture_descriptor =
  MaplibreNativeC.mln_opengl_borrowed_texture_descriptor_default().apply {
    setExtent(extent(), descriptor.extent)
    setOpenGLContext(context(), descriptor.context)
    texture(descriptor.texture)
    target(descriptor.target)
  }

private fun openglSurfaceDescriptor(
  descriptor: OpenGLSurfaceDescriptor
): MaplibreNativeC.mln_opengl_surface_descriptor =
  MaplibreNativeC.mln_opengl_surface_descriptor_default().apply {
    setExtent(extent(), descriptor.extent)
    setOpenGLContext(context(), descriptor.context)
    surface(pointerOrNull(descriptor.surface))
  }

private fun setExtent(out: MaplibreNativeC.mln_render_target_extent, extent: RenderTargetExtent) {
  out.width(extent.width)
  out.height(extent.height)
  out.scale_factor(extent.scaleFactor)
}

private fun setVulkanContext(
  out: MaplibreNativeC.mln_vulkan_context_descriptor,
  context: VulkanContextDescriptor,
) {
  out.instance(pointerOrNull(context.instance))
  out.physical_device(pointerOrNull(context.physicalDevice))
  out.device(pointerOrNull(context.device))
  out.graphics_queue(pointerOrNull(context.graphicsQueue))
  out.graphics_queue_family_index(context.graphicsQueueFamilyIndex)
  out.get_instance_proc_addr(pointerOrNull(context.getInstanceProcAddr))
  out.get_device_proc_addr(pointerOrNull(context.getDeviceProcAddr))
}

private fun setOpenGLContext(
  out: MaplibreNativeC.mln_opengl_context_descriptor,
  context: OpenGLContextDescriptor,
) {
  out.size(out.sizeof())
  when (context) {
    is WglContextDescriptor -> {
      out.platform(MaplibreNativeC.MLN_OPENGL_CONTEXT_PLATFORM_WGL)
      out.data_wgl().apply {
        size(sizeof())
        device_context(pointerOrNull(context.deviceContext))
        share_context(pointerOrNull(context.shareContext))
        get_proc_address(pointerOrNull(context.getProcAddress))
      }
    }
    is EglContextDescriptor -> {
      out.platform(MaplibreNativeC.MLN_OPENGL_CONTEXT_PLATFORM_EGL)
      out.data_egl().apply {
        size(sizeof())
        display(pointerOrNull(context.display))
        config(pointerOrNull(context.config))
        share_context(pointerOrNull(context.shareContext))
        get_proc_address(pointerOrNull(context.getProcAddress))
      }
    }
  }
}

private fun textureImageInfo(info: MaplibreNativeC.mln_texture_image_info): TextureImageInfo =
  TextureImageInfo(info.width(), info.height(), info.stride(), info.byte_length())

private fun map(address: Long): MaplibreNativeC.mln_map =
  MaplibreNativeC.mln_map(AddressPointer(address))

private fun renderSession(address: Long): MaplibreNativeC.mln_render_session =
  MaplibreNativeC.mln_render_session(AddressPointer(address))

private fun pointerOrNull(pointer: NativePointer): Pointer? =
  if (pointer.isNull) null else AddressPointer(pointer.address)

private class AddressPointer(address: Long) : Pointer(null as Pointer?) {
  init {
    this.address = address
  }
}

private fun unsupportedRenderSessionHandle(): Nothing =
  throw UnsupportedOperationException(
    "This RenderSessionHandle operation is not available until the Android render bridge is completed"
  )
