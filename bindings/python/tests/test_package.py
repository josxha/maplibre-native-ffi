import threading

import pytest

import maplibre_native as mln
from maplibre_native import _native
from maplibre_native import camera, geo, json, log, query, render, resource, style


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


def test_map_handle_context_manager_closes_once() -> None:
    with mln.RuntimeHandle() as runtime:
        with mln.MapHandle(runtime, mln.MapOptions(width=128, height=64)) as map_handle:
            assert not map_handle.closed
            map_handle.request_repaint()

        assert map_handle.closed
        map_handle.close()
        assert map_handle.closed


def test_runtime_rejects_close_while_map_is_live() -> None:
    runtime = mln.RuntimeHandle()
    map_handle = runtime.create_map(mln.MapOptions(width=64, height=64))
    try:
        with pytest.raises(mln.InvalidStateError) as raised:
            runtime.close()

        assert raised.value.status == mln.MaplibreStatus.INVALID_STATE
        assert (
            raised.value.native_status_code
            == mln.MaplibreStatus.INVALID_STATE.native_code
        )
    finally:
        map_handle.close()
        runtime.close()


def test_map_create_from_closed_runtime_reports_invalid_argument() -> None:
    runtime = mln.RuntimeHandle()
    runtime.close()

    with pytest.raises(mln.InvalidArgumentError):
        runtime.create_map()


def test_camera_snapshot_and_jump_round_trip_public_values() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            target = camera.CameraOptions(
                center=geo.LatLng(10.0, 20.0),
                zoom=2.0,
                bearing=15.0,
                pitch=10.0,
                padding=camera.EdgeInsets(top=1.0, left=2.0, bottom=3.0, right=4.0),
                anchor=camera.ScreenPoint(x=16.0, y=8.0),
            )
            map_handle.jump_to(target)
            snapshot = map_handle.get_camera()
            map_handle.move_by(1.0, 1.0)
            map_handle.cancel_transitions()

            assert snapshot.center is not None
            assert snapshot.center.latitude == pytest.approx(10.0)
            assert snapshot.center.longitude == pytest.approx(20.0)
            assert snapshot.zoom == pytest.approx(2.0)
            assert snapshot.bearing == pytest.approx(15.0)
            assert snapshot.pitch == pytest.approx(10.0)
            assert snapshot.padding == target.padding


def test_poll_event_returns_none_when_queue_is_empty() -> None:
    with mln.RuntimeHandle() as runtime:
        assert runtime.poll_event() is None


def test_poll_event_returns_copied_map_event() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            with pytest.raises((mln.InvalidArgumentError, mln.NativeError)):
                map_handle.set_style_json("{")

            loading_failed = None
            for _ in range(8):
                event = runtime.poll_event()
                if event is None:
                    break
                if event.event_type == mln.RuntimeEventType.MAP_LOADING_FAILED:
                    loading_failed = event
                    break

            assert loading_failed is not None
            copied_message = loading_failed.message
            runtime.poll_event()

            assert loading_failed.event_type == mln.RuntimeEventType.MAP_LOADING_FAILED
            assert loading_failed.source.source_type == mln.RuntimeEventSourceType.MAP
            assert copied_message == loading_failed.message
            assert loading_failed.message


def test_render_descriptors_are_public_python_values() -> None:
    extent = render.RenderTargetExtent(width=320, height=240, scale_factor=2.0)
    pointer = render.NativePointer(0x1234)
    metal = render.MetalOwnedTextureDescriptor(
        extent=extent,
        context=render.MetalContextDescriptor(device=pointer),
    )
    vulkan = render.VulkanBorrowedTextureDescriptor(
        extent=extent,
        context=render.VulkanContextDescriptor(graphics_queue_family_index=7),
        image=pointer,
        image_view=render.NativePointer(0x5678),
        format=44,
        initial_layout=1,
        final_layout=2,
    )

    assert metal.extent == extent
    assert metal.context.device.address == 0x1234
    assert vulkan.context.graphics_queue_family_index == 7
    assert vulkan.image_view.address == 0x5678
    assert vulkan.format == 44


def test_invalid_render_target_attach_reports_native_status() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            with pytest.raises(
                (mln.InvalidArgumentError, mln.UnsupportedFeatureError)
            ) as raised:
                map_handle.attach_metal_owned_texture(
                    render.MetalOwnedTextureDescriptor()
                )

            assert raised.value.status in {
                mln.MaplibreStatus.INVALID_ARGUMENT,
                mln.MaplibreStatus.UNSUPPORTED,
            }


def test_query_descriptors_and_results_preserve_public_shape() -> None:
    point = camera.ScreenPoint(1.0, 2.0)
    geometry = query.RenderedQueryGeometry.point_geometry(point)
    box_geometry = query.RenderedQueryGeometry.box_geometry(
        query.ScreenBox(camera.ScreenPoint(0.0, 0.0), camera.ScreenPoint(10.0, 10.0))
    )
    line_geometry = query.RenderedQueryGeometry.line_string_geometry(
        (camera.ScreenPoint(0.0, 0.0), camera.ScreenPoint(5.0, 5.0))
    )
    rendered_options = query.RenderedFeatureQueryOptions(
        layer_ids=("landuse",),
        filter=json.json_array(("==", "class", "park")),
    )
    source_options = query.SourceFeatureQueryOptions(source_layer_ids=("landuse",))
    selector = query.FeatureStateSelector(
        source_id="source",
        source_layer_id="layer",
        feature_id="feature-1",
        state_key="hover",
    )
    feature = geo.Feature(geo.point(0.0, 0.0))
    queried = query.QueriedFeature(
        feature=feature,
        source_id="source",
        source_layer_id="layer",
        state=json.json_object((json.JsonMember("hover", True),)),
    )
    extension = query.FeatureExtensionResult.feature_collection_result((feature,))

    assert geometry.point == point
    assert box_geometry.box is not None
    assert line_geometry.line_string == (
        camera.ScreenPoint(0.0, 0.0),
        camera.ScreenPoint(5.0, 5.0),
    )
    assert rendered_options.layer_ids == ("landuse",)
    assert source_options.source_layer_ids == ("landuse",)
    assert selector.state_key == "hover"
    assert queried.source_id == "source"
    assert extension.feature_collection == (feature,)


def test_query_selector_rejects_state_key_without_feature_id() -> None:
    with pytest.raises(ValueError, match="state_key requires feature_id"):
        query.FeatureStateSelector(source_id="source", state_key="hover")


def test_process_global_logging_receiver_copies_native_records() -> None:
    receiver = log.set_log_callback(max_queued_records=8, consume=True)
    try:
        log.set_async_log_severity_mask(log.LogSeverityMask.DEFAULT)
        with mln.RuntimeHandle() as runtime:
            with runtime.create_map() as map_handle:
                with pytest.raises((mln.InvalidArgumentError, mln.NativeError)):
                    map_handle.set_style_json("{")
                map_handle.dump_debug_logs()

        records = []
        while (record := receiver.poll_record()) is not None:
            records.append(record)

        assert records
        assert any(record.message for record in records)
        assert all(isinstance(record.severity, log.LogSeverity) for record in records)
        assert all(isinstance(record.event, log.LogEvent) for record in records)
    finally:
        log.clear_log_callback()
        log.set_async_log_severity_mask(log.LogSeverityMask.DEFAULT)


def test_log_receiver_reports_dropped_records() -> None:
    receiver = log.set_log_callback(max_queued_records=1, consume=True)
    try:
        with mln.RuntimeHandle() as runtime:
            with runtime.create_map() as map_handle:
                with pytest.raises((mln.InvalidArgumentError, mln.NativeError)):
                    map_handle.set_style_json("{")
                map_handle.dump_debug_logs()

        assert receiver.poll_record() is not None
        assert receiver.dropped_record_count >= 0
    finally:
        log.clear_log_callback()


def test_json_values_preserve_order_duplicates_and_numeric_shape() -> None:
    value = json.json_object(
        (
            json.JsonMember("same", json.json_uint(1)),
            json.JsonMember("same", json.json_int(-1)),
            json.JsonMember("nested", json.json_array((True, json.json_double(1.5)))),
        )
    )

    assert json.to_python(value) == [
        ("same", 1),
        ("same", -1),
        ("nested", [True, 1.5]),
    ]
    assert value.members[0].value == json.JsonUInt(1)
    assert value.members[1].value == json.JsonInt(-1)


def test_geojson_values_preserve_geometry_and_properties() -> None:
    feature = geo.Feature(
        geometry=geo.line_string((geo.LatLng(1.0, 2.0), geo.LatLng(3.0, 4.0))),
        properties=(
            json.JsonMember("name", "road"),
            json.JsonMember("name", "duplicate"),
        ),
        identifier=geo.FeatureIdentifierString("feature-1"),
    )
    collection = geo.FeatureCollection((feature,))

    assert collection.features[0].geometry == geo.LineString(
        (geo.LatLng(1.0, 2.0), geo.LatLng(3.0, 4.0))
    )
    assert [member.key for member in collection.features[0].properties] == [
        "name",
        "name",
    ]
    assert collection.features[0].identifier == geo.FeatureIdentifierString("feature-1")


def test_resource_values_preserve_native_shape() -> None:
    request = resource.ResourceRequest.from_native(
        {
            "url": "https://example.test/tile.pbf",
            "kind": resource.ResourceKind.TILE.native_code,
            "loading_method": resource.ResourceLoadingMethod.NETWORK_ONLY,
            "priority": resource.ResourcePriority.LOW,
            "usage": resource.ResourceUsage.OFFLINE,
            "storage_policy": resource.ResourceStoragePolicy.VOLATILE,
            "range": {"start": 5, "end": 10},
            "prior_modified_unix_ms": 123,
            "prior_expires_unix_ms": 456,
            "prior_etag": "abc",
            "prior_data": b"old",
        }
    )
    response = resource.ResourceResponse.error(
        resource.ResourceErrorReason.NOT_FOUND,
        "missing",
    )

    assert request.kind == resource.ResourceKind.TILE
    assert request.range == resource.ByteRange(5, 10)
    assert request.prior_data == b"old"
    assert response.to_native()["status"] == resource.ResourceResponseStatus.ERROR
    assert (
        response.to_native()["error_reason"] == resource.ResourceErrorReason.NOT_FOUND
    )


def test_resource_transform_registers_and_clears() -> None:
    seen: list[resource.ResourceTransformRequest] = []

    def transform(request: resource.ResourceTransformRequest) -> str | None:
        seen.append(request)
        return None

    with mln.RuntimeHandle() as runtime:
        runtime.set_resource_transform(transform, max_pending_callbacks=1)
        with runtime.create_map():
            runtime.set_resource_transform(transform, max_pending_callbacks=1)
            runtime.clear_resource_transform()


def test_resource_callback_registration_validates_bounds_and_lifecycle() -> None:
    def provider(
        request: resource.ResourceRequest,
        handle: resource.ResourceRequestHandle,
    ) -> resource.ResourceProviderDecision:
        handle.close()
        return resource.ResourceProviderDecision.PASS_THROUGH

    with mln.RuntimeHandle() as runtime:
        with pytest.raises(mln.InvalidArgumentError):
            runtime.set_resource_provider(provider, max_pending_callbacks=0)

        runtime.set_resource_provider(provider, max_pending_callbacks=1)
        with runtime.create_map():
            with pytest.raises(mln.InvalidStateError):
                runtime.set_resource_provider(provider, max_pending_callbacks=1)


def test_custom_geometry_source_scaffolding_queues_copied_events() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.set_style_json('{"version":8,"sources":{},"layers":[]}')
            source = map_handle.add_custom_geometry_source(
                "custom",
                style.CustomGeometrySourceOptions(
                    has_cancel_tile=True,
                    max_queued_events=1,
                ),
            )

            source._native.push_fetch_for_test(1, 2, 3)
            source._native.push_cancel_for_test(4, 5, 6)

            event = source.poll_event()
            assert event == style.CustomGeometrySourceEvent(
                style.CustomGeometrySourceEventType.FETCH_TILE,
                style.CanonicalTileId(1, 2, 3),
            )
            assert source.poll_event() is None
            assert source.dropped_event_count == 1

            map_handle.set_style_json('{"version":8,"sources":{},"layers":[]}')
            assert source.closed


def test_custom_geometry_source_rejects_empty_queue_capacity() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.set_style_json('{"version":8,"sources":{},"layers":[]}')
            with pytest.raises(mln.InvalidArgumentError):
                map_handle.add_custom_geometry_source(
                    "custom",
                    style.CustomGeometrySourceOptions(max_queued_events=0),
                )
