"""Process-global entry points for the Python binding."""

from . import _native
from .render import RenderBackend

EXPECTED_C_ABI_VERSION: int = int(_native.expected_c_abi_version())


def c_version() -> int:
    """Return the native C ABI contract version."""
    return int(_native.c_version())


def supported_render_backends() -> RenderBackend:
    """Return render backends compiled into the linked native library."""
    return RenderBackend(_native.supported_render_backends_raw())
