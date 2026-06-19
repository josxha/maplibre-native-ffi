package org.maplibre.nativeffi.internal.loader

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue
import org.maplibre.nativeffi.error.AbiVersionMismatchException

class NativeAccessTest {
  @Test
  fun abiVersionMismatchReportsActualAndExpectedVersions() {
    val error =
      assertFailsWith<AbiVersionMismatchException> {
        NativeAccess.checkAbiVersion(NativeAccess.EXPECTED_C_ABI_VERSION + 1)
      }

    assertEquals(NativeAccess.EXPECTED_C_ABI_VERSION + 1, error.actualVersion)
    assertEquals(NativeAccess.EXPECTED_C_ABI_VERSION, error.expectedVersion)
  }

  @Test
  fun nativeAccessFailureIsWrappedWithJvmFlagGuidance() {
    val error =
      assertFailsWith<IllegalStateException> {
        NativeAccess.checkNativeAccessAndAbi { throw IllegalCallerException("native access") }
      }

    assertTrue(error.message.orEmpty().contains("--enable-native-access=ALL-UNNAMED"))
  }

  @Test
  fun missingSymbolFailureIsWrappedAsUnsatisfiedLinkError() {
    val error =
      assertFailsWith<UnsatisfiedLinkError> {
        NativeAccess.checkNativeAccessAndAbi { throw NoSuchElementException("mln_c_version") }
      }

    assertTrue(error.message.orEmpty().contains("Maplibre C ABI symbols"))
  }
}
