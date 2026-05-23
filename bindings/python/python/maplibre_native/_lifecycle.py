"""Shared Python handle lifecycle helpers."""

from __future__ import annotations

import warnings


def warn_unclosed(handle_name: str, closed: bool) -> None:
    """Report an unclosed handle without destroying thread-affine native state."""
    if closed:
        return
    warnings.warn(
        f"{handle_name} was not closed; native state was intentionally leaked",
        ResourceWarning,
        stacklevel=2,
    )
