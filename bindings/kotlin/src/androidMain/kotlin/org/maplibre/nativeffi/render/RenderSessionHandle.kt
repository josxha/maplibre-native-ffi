package org.maplibre.nativeffi.render

import java.nio.charset.StandardCharsets
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
    NativeAccess.ensureLoaded()
    FeatureStateSelectorScope(selector).use { nativeSelector ->
      JsonScope(value).use { nativeValue ->
        Status.check(
          MaplibreNativeC.mln_render_session_set_feature_state(
            renderSession(requireLiveAddress()),
            nativeSelector.selector,
            nativeValue.value,
          )
        )
      }
    }
  }

  public actual fun getFeatureState(selector: FeatureStateSelector): JsonValue {
    NativeAccess.ensureLoaded()
    FeatureStateSelectorScope(selector).use { nativeSelector ->
      PointerPointer<Pointer>(1).use { outState ->
        outState.put(0, null as Pointer?)
        Status.check(
          MaplibreNativeC.mln_render_session_get_feature_state(
            renderSession(requireLiveAddress()),
            nativeSelector.selector,
            outState,
          )
        )
        return jsonSnapshot(outState) ?: JsonValue.ObjectValue(emptyList())
      }
    }
  }

  public actual fun removeFeatureState(selector: FeatureStateSelector) {
    NativeAccess.ensureLoaded()
    FeatureStateSelectorScope(selector).use { nativeSelector ->
      Status.check(
        MaplibreNativeC.mln_render_session_remove_feature_state(
          renderSession(requireLiveAddress()),
          nativeSelector.selector,
        )
      )
    }
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

private fun jsonSnapshot(outSnapshot: PointerPointer<Pointer>): JsonValue? {
  val snapshotPointer = outSnapshot.get(Pointer::class.java, 0) ?: return null
  if (snapshotPointer.isNull) return null
  val snapshot = MaplibreNativeC.mln_json_snapshot(snapshotPointer)
  return try {
    PointerPointer<Pointer>(1).use { outValue ->
      outValue.put(0, null as Pointer?)
      Status.check(MaplibreNativeC.mln_json_snapshot_get(snapshot, outValue))
      val valuePointer = outValue.get(Pointer::class.java, 0) ?: return null
      if (valuePointer.isNull) null else jsonValue(MaplibreNativeC.mln_json_value(valuePointer))
    }
  } finally {
    MaplibreNativeC.mln_json_snapshot_destroy(snapshot)
  }
}

private fun jsonValue(value: MaplibreNativeC.mln_json_value): JsonValue =
  when (value.type()) {
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_NULL -> JsonValue.Null
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_BOOL -> JsonValue.Bool(value.data_bool_value())
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_UINT -> JsonValue.UInt(value.data_uint_value())
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_INT -> JsonValue.Int(value.data_int_value())
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_DOUBLE -> JsonValue.DoubleValue(value.data_double_value())
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_STRING ->
      JsonValue.StringValue(stringView(value.data_string_value()))
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_ARRAY -> jsonArray(value.data_array_value())
    MaplibreNativeC.MLN_JSON_VALUE_TYPE_OBJECT -> jsonObject(value.data_object_value())
    else -> JsonValue.Unknown(value.type(), value.size())
  }

private fun jsonArray(array: MaplibreNativeC.mln_json_array): JsonValue.Array {
  val nativeValues = array.values()
  return JsonValue.Array(
    List(Math.toIntExact(array.value_count())) { index ->
      jsonValue(nativeValues.getPointer(index.toLong()))
    }
  )
}

private fun jsonObject(obj: MaplibreNativeC.mln_json_object): JsonValue.ObjectValue {
  val nativeMembers = obj.members()
  return JsonValue.ObjectValue(
    List(Math.toIntExact(obj.member_count())) { index ->
      val member = nativeMembers.getPointer(index.toLong())
      JsonValue.Member(stringView(member.key()), jsonValue(member.value()))
    }
  )
}

private fun stringView(value: MaplibreNativeC.mln_string_view): String {
  val size = Math.toIntExact(value.size())
  if (size == 0) return ""
  val bytes = ByteArray(size)
  value.data().get(bytes, 0, size)
  return String(bytes, StandardCharsets.UTF_8)
}

private fun map(address: Long): MaplibreNativeC.mln_map =
  MaplibreNativeC.mln_map(AddressPointer(address))

private fun renderSession(address: Long): MaplibreNativeC.mln_render_session =
  MaplibreNativeC.mln_render_session(AddressPointer(address))

private fun pointerOrNull(pointer: NativePointer): Pointer? =
  if (pointer.isNull) null else AddressPointer(pointer.address)

private class FeatureStateSelectorScope(value: FeatureStateSelector) : AutoCloseable {
  private val sourceId = StringViewScope(value.sourceId)
  private val sourceLayerId = value.sourceLayerId?.let(::StringViewScope)
  private val featureId = value.featureId?.let(::StringViewScope)
  private val stateKey = value.stateKey?.let(::StringViewScope)
  val selector: MaplibreNativeC.mln_feature_state_selector =
    MaplibreNativeC.mln_feature_state_selector()

  init {
    selector.size(selector.sizeof())
    selector.source_id(sourceId.view)
    var fields = 0
    sourceLayerId?.let {
      fields = fields or MaplibreNativeC.MLN_FEATURE_STATE_SELECTOR_SOURCE_LAYER_ID
      selector.source_layer_id(it.view)
    }
    featureId?.let {
      fields = fields or MaplibreNativeC.MLN_FEATURE_STATE_SELECTOR_FEATURE_ID
      selector.feature_id(it.view)
    }
    stateKey?.let {
      fields = fields or MaplibreNativeC.MLN_FEATURE_STATE_SELECTOR_STATE_KEY
      selector.state_key(it.view)
    }
    selector.fields(fields)
  }

  override fun close() {
    selector.close()
    stateKey?.close()
    featureId?.close()
    sourceLayerId?.close()
    sourceId.close()
  }
}

private class JsonScope(value: JsonValue) : AutoCloseable {
  private val owned = mutableListOf<Pointer>()
  private val strings = mutableListOf<StringViewScope>()
  val value: MaplibreNativeC.mln_json_value = jsonValue(value)

  override fun close() {
    owned.asReversed().forEach(Pointer::close)
    strings.asReversed().forEach(StringViewScope::close)
  }

  private fun <T : Pointer> own(pointer: T): T {
    owned += pointer
    return pointer
  }

  private fun string(value: String): MaplibreNativeC.mln_string_view {
    val scope = StringViewScope(value)
    strings += scope
    return scope.view
  }

  private fun jsonValue(value: JsonValue, depth: Int = 0): MaplibreNativeC.mln_json_value {
    require(depth <= JsonValue.MAX_DESCRIPTOR_DEPTH) {
      "JSON descriptor depth exceeds ${JsonValue.MAX_DESCRIPTOR_DEPTH}"
    }
    val out = own(MaplibreNativeC.mln_json_value())
    out.size(out.sizeof())
    when (value) {
      JsonValue.Null -> out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_NULL)
      is JsonValue.Bool ->
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_BOOL).data_bool_value(value.value)
      is JsonValue.UInt ->
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_UINT).data_uint_value(value.value)
      is JsonValue.Int ->
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_INT).data_int_value(value.value)
      is JsonValue.DoubleValue ->
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_DOUBLE).data_double_value(value.value)
      is JsonValue.StringValue ->
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_STRING).data_string_value(string(value.value))
      is JsonValue.Array -> {
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_ARRAY)
        val array = own(MaplibreNativeC.mln_json_array())
        if (value.values.isNotEmpty()) {
          val nativeValues = own(MaplibreNativeC.mln_json_value(value.values.size.toLong()))
          value.values.forEachIndexed { index, child ->
            nativeValues
              .position(index.toLong())
              .put<MaplibreNativeC.mln_json_value>(jsonValue(child, depth + 1))
          }
          nativeValues.position(0)
          array.values(nativeValues)
        }
        array.value_count(value.values.size.toLong())
        out.data_array_value(array)
      }
      is JsonValue.ObjectValue -> {
        out.type(MaplibreNativeC.MLN_JSON_VALUE_TYPE_OBJECT)
        val obj = own(MaplibreNativeC.mln_json_object())
        if (value.members.isNotEmpty()) {
          val nativeMembers = own(MaplibreNativeC.mln_json_member(value.members.size.toLong()))
          value.members.forEachIndexed { index, member ->
            nativeMembers.position(index.toLong())
            nativeMembers.key(string(member.key))
            nativeMembers.value(jsonValue(member.value, depth + 1))
          }
          nativeMembers.position(0)
          obj.members(nativeMembers)
        }
        obj.member_count(value.members.size.toLong())
        out.data_object_value(obj)
      }
      is JsonValue.Unknown ->
        throw IllegalArgumentException("unknown JSON values cannot be used as input")
    }
    return out
  }
}

private class StringViewScope(value: String) : AutoCloseable {
  private val bytes: BytePointer
  val view: MaplibreNativeC.mln_string_view = MaplibreNativeC.mln_string_view()

  init {
    val utf8 = value.toByteArray(StandardCharsets.UTF_8)
    bytes = BytePointer(Math.max(utf8.size, 1).toLong())
    if (utf8.isNotEmpty()) bytes.put(utf8, 0, utf8.size)
    view.data(if (utf8.isEmpty()) null else bytes)
    view.size(utf8.size.toLong())
  }

  override fun close() {
    view.close()
    bytes.close()
  }
}

private class AddressPointer(address: Long) : Pointer(null as Pointer?) {
  init {
    this.address = address
  }
}

private fun unsupportedRenderSessionHandle(): Nothing =
  throw UnsupportedOperationException(
    "This RenderSessionHandle operation is not available until the Android render bridge is completed"
  )
