"""Shared Python handle lifecycle helpers."""

from __future__ import annotations

from collections.abc import Callable
import warnings as _warnings


def warn_unclosed(
    handle_name: str,
    closed: bool,
    _warn: Callable[..., None] = _warnings.warn,
) -> None:
    """Report an unclosed handle without destroying thread-affine native state."""
    if closed:
        return
    try:
        _warn(
            f"{handle_name} was not closed; native state was intentionally leaked",
            ResourceWarning,
            stacklevel=2,
        )
    except BaseException:
        # Finalizers may run during interpreter shutdown. Leak reporting must not
        # raise noisy unraisable exceptions when Python globals are half torn down.
        return
