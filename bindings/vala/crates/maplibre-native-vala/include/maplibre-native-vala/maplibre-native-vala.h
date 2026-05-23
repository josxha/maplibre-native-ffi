#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MLN_VALA_DIAGNOSTIC_CAPACITY 512

/*
 * Scaffold-only result for the first Rust adapter slice.
 *
 * The generated GObject ABI will expose status-returning operations as
 * gboolean-returning functions with GError**. Keep this struct private to
 * scaffold tests and generator experiments.
 */
typedef struct mln_vala_status_result {
  int32_t status;
  uint32_t value;
  size_t diagnostic_len;
  uint8_t diagnostic[MLN_VALA_DIAGNOSTIC_CAPACITY];
} mln_vala_status_result;

uint32_t mln_vala_c_version(void);
uint32_t mln_vala_supported_render_backend_mask(void);
mln_vala_status_result mln_vala_network_status_get(void);
mln_vala_status_result mln_vala_network_status_set(uint32_t raw_status);

#ifdef __cplusplus
}
#endif
