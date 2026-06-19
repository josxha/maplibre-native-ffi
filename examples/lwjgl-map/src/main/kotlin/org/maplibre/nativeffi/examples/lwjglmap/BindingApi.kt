@file:JvmName("BindingApi")

package org.maplibre.nativeffi.examples.lwjglmap

import org.maplibre.nativeffi.camera.CameraOptions
import org.maplibre.nativeffi.log.LogRecord
import org.maplibre.nativeffi.map.MapHandle
import org.maplibre.nativeffi.map.MapMode
import org.maplibre.nativeffi.map.MapOptions
import org.maplibre.nativeffi.render.MetalBorrowedTextureDescriptor
import org.maplibre.nativeffi.render.MetalOwnedTextureDescriptor
import org.maplibre.nativeffi.render.MetalSurfaceDescriptor
import org.maplibre.nativeffi.render.NativePointer
import org.maplibre.nativeffi.render.OpenGLBorrowedTextureDescriptor
import org.maplibre.nativeffi.render.OpenGLOwnedTextureDescriptor
import org.maplibre.nativeffi.render.OpenGLSurfaceDescriptor
import org.maplibre.nativeffi.render.RenderSessionHandle
import org.maplibre.nativeffi.render.VulkanBorrowedTextureDescriptor
import org.maplibre.nativeffi.render.VulkanOwnedTextureDescriptor
import org.maplibre.nativeffi.render.VulkanSurfaceDescriptor
import org.maplibre.nativeffi.runtime.RuntimeEvent
import org.maplibre.nativeffi.runtime.RuntimeEventPayload
import org.maplibre.nativeffi.runtime.RuntimeEventType
import org.maplibre.nativeffi.runtime.RuntimeHandle
import org.maplibre.nativeffi.runtime.RuntimeOptions

fun nativePointer(address: Long): NativePointer = NativePointer.ofAddress(address)

fun nullNativePointer(): NativePointer = NativePointer.NULL

fun nativeAddress(pointer: NativePointer): Long = pointer.address

fun continuousMapMode(): MapMode = MapMode.CONTINUOUS

fun setContinuousMapMode(options: MapOptions) {
  options.mapMode = MapMode.CONTINUOUS
}

fun createRuntime(options: RuntimeOptions): RuntimeHandle = RuntimeHandle.create(options)

fun createMap(runtime: RuntimeHandle, options: MapOptions): MapHandle =
  MapHandle.create(runtime, options)

fun currentCamera(map: MapHandle): CameraOptions = map.camera

fun isMapRenderUpdateAvailable(event: RuntimeEvent, map: MapHandle): Boolean =
  event.type == RuntimeEventType.MAP_RENDER_UPDATE_AVAILABLE && event.mapSource == map

fun isRepaintNeededRenderFrame(event: RuntimeEvent, map: MapHandle): Boolean =
  event.type == RuntimeEventType.MAP_RENDER_FRAME_FINISHED &&
    event.mapSource == map &&
    (event.payload as? RuntimeEventPayload.RenderFrame)?.needsRepaint == true

fun logSeverity(record: LogRecord): Any = record.severity

fun logEvent(record: LogRecord): Any = record.event

fun logCode(record: LogRecord): Long = record.code

fun logMessage(record: LogRecord): String = record.message

fun attachMetalOwnedTexture(
  map: MapHandle,
  descriptor: MetalOwnedTextureDescriptor,
): RenderSessionHandle = map.attachMetalOwnedTexture(descriptor)

fun attachMetalBorrowedTexture(
  map: MapHandle,
  descriptor: MetalBorrowedTextureDescriptor,
): RenderSessionHandle = map.attachMetalBorrowedTexture(descriptor)

fun attachMetalSurface(map: MapHandle, descriptor: MetalSurfaceDescriptor): RenderSessionHandle =
  map.attachMetalSurface(descriptor)

fun attachVulkanOwnedTexture(
  map: MapHandle,
  descriptor: VulkanOwnedTextureDescriptor,
): RenderSessionHandle = map.attachVulkanOwnedTexture(descriptor)

fun attachVulkanBorrowedTexture(
  map: MapHandle,
  descriptor: VulkanBorrowedTextureDescriptor,
): RenderSessionHandle = map.attachVulkanBorrowedTexture(descriptor)

fun attachVulkanSurface(map: MapHandle, descriptor: VulkanSurfaceDescriptor): RenderSessionHandle =
  map.attachVulkanSurface(descriptor)

fun attachOpenGLOwnedTexture(
  map: MapHandle,
  descriptor: OpenGLOwnedTextureDescriptor,
): RenderSessionHandle = map.attachOpenGLOwnedTexture(descriptor)

fun attachOpenGLBorrowedTexture(
  map: MapHandle,
  descriptor: OpenGLBorrowedTextureDescriptor,
): RenderSessionHandle = map.attachOpenGLBorrowedTexture(descriptor)

fun attachOpenGLSurface(map: MapHandle, descriptor: OpenGLSurfaceDescriptor): RenderSessionHandle =
  map.attachOpenGLSurface(descriptor)
