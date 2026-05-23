import threading

import pytest

import maplibre_native as mln
from maplibre_native import _native


def test_c_version_matches_expected_abi_version() -> None:
    assert mln.c_version() == mln.EXPECTED_C_ABI_VERSION


def test_supported_render_backends_returns_flag_value() -> None:
    assert isinstance(mln.supported_render_backends(), mln.RenderBackend)


def test_native_pointer_is_opaque_value() -> None:
    pointer = mln.NativePointer(0)

    assert pointer.is_null
    assert pointer == mln.NativePointer.null()


def test_network_status_round_trips_through_public_api() -> None:
    original = mln.network_status()
    try:
        mln.set_network_status(mln.NetworkStatus.OFFLINE)
        assert mln.network_status() == mln.NetworkStatus.OFFLINE
        mln.set_network_status(mln.NetworkStatus.ONLINE)
        assert mln.network_status() == mln.NetworkStatus.ONLINE
    finally:
        mln.set_network_status(original)


def test_network_status_preserves_unknown_raw_values() -> None:
    status = mln.NetworkStatus(999_001)

    assert status.is_unknown
    assert status.native_code == 999_001


def test_unknown_network_status_setter_raises_invalid_argument() -> None:
    with pytest.raises(mln.InvalidArgumentError) as raised:
        mln.set_network_status(mln.NetworkStatus(999_001))

    assert raised.value.status == mln.MaplibreStatus.INVALID_ARGUMENT
    assert raised.value.native_status_code is None
    assert "cannot be set" in raised.value.diagnostic


def test_native_status_conversion_preserves_status_and_diagnostic() -> None:
    with pytest.raises(mln.InvalidArgumentError) as raised:
        _native.set_network_status_raw_unchecked_for_test(999_001)

    assert raised.value.status == mln.MaplibreStatus.INVALID_ARGUMENT
    assert (
        raised.value.native_status_code
        == mln.MaplibreStatus.INVALID_ARGUMENT.native_code
    )
    assert "network status" in raised.value.diagnostic


def test_runtime_handle_context_manager_closes_once() -> None:
    with mln.RuntimeHandle() as runtime:
        assert not runtime.closed
        runtime.run_once()

    assert runtime.closed
    runtime.close()
    assert runtime.closed


def test_duplicate_runtime_reports_invalid_state() -> None:
    runtime = mln.RuntimeHandle()
    try:
        with pytest.raises(mln.InvalidStateError) as raised:
            mln.RuntimeHandle()

        assert raised.value.status == mln.MaplibreStatus.INVALID_STATE
        assert (
            raised.value.native_status_code
            == mln.MaplibreStatus.INVALID_STATE.native_code
        )
    finally:
        runtime.close()


def test_runtime_close_from_wrong_thread_reports_wrong_thread() -> None:
    runtime = mln.RuntimeHandle()
    raised_error: list[BaseException] = []

    def close_runtime() -> None:
        try:
            runtime.close()
        except BaseException as error:
            raised_error.append(error)

    thread = threading.Thread(target=close_runtime)
    thread.start()
    thread.join()

    try:
        assert len(raised_error) == 1
        assert isinstance(raised_error[0], mln.WrongThreadError)
        assert raised_error[0].status == mln.MaplibreStatus.WRONG_THREAD
    finally:
        runtime.close()
