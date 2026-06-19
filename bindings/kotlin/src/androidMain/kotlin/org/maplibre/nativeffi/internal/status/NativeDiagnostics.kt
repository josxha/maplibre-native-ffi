package org.maplibre.nativeffi.internal.status

import java.nio.charset.StandardCharsets
import org.bytedeco.javacpp.BytePointer
import org.maplibre.nativeffi.internal.javacpp.MaplibreNativeC

internal actual object NativeDiagnostics {
  actual fun currentDiagnostic(): String = cString(MaplibreNativeC.mln_thread_last_error_message())

  private fun cString(pointer: BytePointer?): String =
    if (pointer == null || pointer.isNull) "" else pointer.getString(StandardCharsets.UTF_8)
}
