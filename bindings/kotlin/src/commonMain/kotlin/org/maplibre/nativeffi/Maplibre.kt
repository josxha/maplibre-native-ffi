package org.maplibre.nativeffi

import org.maplibre.nativeffi.geo.LatLng
import org.maplibre.nativeffi.geo.ProjectedMeters
import org.maplibre.nativeffi.render.OpenGLContextProvider
import org.maplibre.nativeffi.render.RenderBackend
import org.maplibre.nativeffi.runtime.NetworkStatus

/** Process-global entry points for the MapLibre Native FFI binding. */
public expect object Maplibre {
  /** C ABI contract version expected by this binding. */
  public val EXPECTED_C_ABI_VERSION: Long

  /** Loads or verifies access to the native library for the current platform. */
  public fun loadNativeLibrary()

  /** Returns the native C ABI contract version. */
  public fun cVersion(): Long

  /** Returns the render backends compiled into the loaded native library. */
  public fun supportedRenderBackends(): Set<RenderBackend>

  /** Returns the OpenGL context providers compiled into the loaded native library. */
  public fun supportedOpenGLContextProviders(): Set<OpenGLContextProvider>

  /** Reads Maplibre Native's process-global network status. */
  public val networkStatus: NetworkStatus

  /** Sets Maplibre Native's process-global network status. */
  public fun setNetworkStatus(status: NetworkStatus)

  /** Converts a geographic coordinate to spherical Mercator projected meters. */
  public fun projectedMetersForLatLng(coordinate: LatLng): ProjectedMeters

  /** Converts spherical Mercator projected meters to a geographic coordinate. */
  public fun latLngForProjectedMeters(meters: ProjectedMeters): LatLng
}
