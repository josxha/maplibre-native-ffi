import math
from pathlib import Path
import threading

import pytest

import maplibre_native as mln
from maplibre_native import _native
from maplibre_native import (
    camera,
    geo,
    json,
    log,
    map as map_module,
    offline,
    query,
    render,
    resource,
    style,
)


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


def test_still_image_request_uses_map_mode_validation() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map(mln.MapOptions(mode=mln.MapMode.STATIC)) as static_map:
            static_map.request_still_image()

    with mln.RuntimeHandle() as runtime:
        with runtime.create_map(
            mln.MapOptions(mode=mln.MapMode.CONTINUOUS)
        ) as map_handle:
            with pytest.raises(mln.InvalidStateError) as raised:
                map_handle.request_still_image()

    assert raised.value.status == mln.MaplibreStatus.INVALID_STATE


def test_map_create_from_closed_runtime_reports_invalid_argument() -> None:
    runtime = mln.RuntimeHandle()
    runtime.close()

    with pytest.raises(mln.InvalidArgumentError):
        runtime.create_map()


def test_map_debug_and_status_options_round_trip_public_values() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            debug_options = (
                map_module.MapDebugOptions.TILE_BORDERS
                | map_module.MapDebugOptions.PARSE_STATUS
            )
            map_handle.set_debug_options(debug_options)
            map_handle.set_rendering_stats_view_enabled(True)

            assert map_handle.get_debug_options() == debug_options
            assert map_handle.get_rendering_stats_view_enabled() is True
            assert isinstance(map_handle.is_fully_loaded(), bool)

            map_handle.set_debug_options(map_module.MapDebugOptions.NONE)
            map_handle.set_rendering_stats_view_enabled(False)
            assert map_handle.get_debug_options() == map_module.MapDebugOptions.NONE
            assert map_handle.get_rendering_stats_view_enabled() is False


def test_style_url_reports_native_status_for_invalid_url() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            with pytest.raises(mln.InvalidArgumentError):
                map_handle.set_style_url("bad\0url")


def test_style_source_url_metadata_and_removal_public_api() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.set_style_json('{"version":8,"sources":{},"layers":[]}')
            map_handle.add_style_source_json(
                "style-json-points",
                json.from_python(
                    {
                        "type": "geojson",
                        "data": {
                            "type": "FeatureCollection",
                            "features": [],
                        },
                    }
                ),
            )
            map_handle.add_geojson_source_url(
                "points", "https://example.test/points.geojson"
            )
            inline_points = geo.FeatureCollection(
                (
                    geo.Feature(
                        geometry=geo.point(1.0, 2.0),
                        properties=(json.JsonMember("name", "one"),),
                        identifier=geo.FeatureIdentifierString("point-1"),
                    ),
                )
            )
            map_handle.add_geojson_source_data("inline-points", inline_points)
            map_handle.set_geojson_source_url(
                "inline-points",
                "https://example.test/inline-points.geojson",
            )
            map_handle.set_geojson_source_data("inline-points", inline_points)
            map_handle.add_vector_source_url(
                "vector-tiles",
                "https://example.test/vector.json",
                style.TileSourceOptions(
                    min_zoom=1.0,
                    max_zoom=10.0,
                    vector_encoding=style.VectorTileEncoding.MVT,
                ),
            )
            map_handle.add_raster_source_url(
                "raster-tiles",
                "https://example.test/raster.json",
                style.TileSourceOptions(tile_size=256),
            )
            map_handle.add_raster_dem_source_url(
                "dem-tiles",
                "https://example.test/dem.json",
                style.TileSourceOptions(
                    tile_size=512,
                    raster_dem_encoding=style.RasterDemEncoding.MAPBOX,
                ),
            )
            map_handle.add_vector_source_tiles(
                "vector-inline",
                ("https://example.test/vector/{z}/{x}/{y}.pbf",),
            )
            map_handle.add_raster_source_tiles(
                "raster-inline",
                ("https://example.test/raster/{z}/{x}/{y}.png",),
            )
            map_handle.add_raster_dem_source_tiles(
                "dem-inline",
                ("https://example.test/dem/{z}/{x}/{y}.png",),
            )

            assert map_handle.style_source_exists("points") is True
            assert map_handle.style_source_exists("missing") is False
            assert (
                map_handle.get_style_source_type("style-json-points")
                == style.StyleSourceType.GEOJSON
            )
            assert (
                map_handle.get_style_source_type("points")
                == style.StyleSourceType.GEOJSON
            )
            assert (
                map_handle.get_style_source_type("inline-points")
                == style.StyleSourceType.GEOJSON
            )
            assert (
                map_handle.get_style_source_type("vector-tiles")
                == style.StyleSourceType.VECTOR
            )
            assert (
                map_handle.get_style_source_type("raster-tiles")
                == style.StyleSourceType.RASTER
            )
            assert (
                map_handle.get_style_source_type("dem-tiles")
                == style.StyleSourceType.RASTER_DEM
            )
            assert (
                map_handle.get_style_source_type("vector-inline")
                == style.StyleSourceType.VECTOR
            )
            assert (
                map_handle.get_style_source_type("raster-inline")
                == style.StyleSourceType.RASTER
            )
            assert (
                map_handle.get_style_source_type("dem-inline")
                == style.StyleSourceType.RASTER_DEM
            )
            assert map_handle.get_style_source_type("missing") is None
            source_ids = map_handle.list_style_source_ids()
            assert "style-json-points" in source_ids
            assert "points" in source_ids
            assert "inline-points" in source_ids
            assert "vector-tiles" in source_ids
            assert "raster-tiles" in source_ids
            assert "dem-tiles" in source_ids
            assert "vector-inline" in source_ids
            assert "raster-inline" in source_ids
            assert "dem-inline" in source_ids

            info = map_handle.get_style_source_info("points")
            assert info is not None
            assert info.source_type == style.StyleSourceType.GEOJSON
            assert info.attribution is None
            assert map_handle.get_style_source_info("missing") is None

            assert map_handle.remove_style_source("style-json-points") is True
            assert map_handle.remove_style_source("points") is True
            assert map_handle.remove_style_source("points") is False
            assert map_handle.remove_style_source("inline-points") is True
            assert map_handle.remove_style_source("vector-tiles") is True
            assert map_handle.remove_style_source("raster-tiles") is True
            assert map_handle.remove_style_source("dem-tiles") is True
            assert map_handle.remove_style_source("vector-inline") is True
            assert map_handle.remove_style_source("raster-inline") is True
            assert map_handle.remove_style_source("dem-inline") is True
            source_ids = map_handle.list_style_source_ids()
            assert "style-json-points" not in source_ids
            assert "points" not in source_ids
            assert "inline-points" not in source_ids
            assert "vector-tiles" not in source_ids
            assert "raster-tiles" not in source_ids
            assert "dem-tiles" not in source_ids
            assert "vector-inline" not in source_ids
            assert "raster-inline" not in source_ids
            assert "dem-inline" not in source_ids


def test_image_source_url_image_and_coordinates_public_api() -> None:
    coordinates = (
        geo.LatLng(1.0, 2.0),
        geo.LatLng(1.0, 3.0),
        geo.LatLng(0.0, 3.0),
        geo.LatLng(0.0, 2.0),
    )
    updated_coordinates = (
        geo.LatLng(2.0, 2.0),
        geo.LatLng(2.0, 3.0),
        geo.LatLng(1.0, 3.0),
        geo.LatLng(1.0, 2.0),
    )
    image = render.PremultipliedRgba8Image(
        info=render.TextureImageInfo(width=1, height=1, stride=4, byte_length=4),
        data=bytes([0, 255, 0, 255]),
    )

    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.set_style_json('{"version":8,"sources":{},"layers":[]}')
            map_handle.add_image_source_url(
                "overlay-url",
                coordinates,
                "https://example.test/overlay.png",
            )
            map_handle.add_image_source_image("overlay-inline", coordinates, image)

            assert (
                map_handle.get_style_source_type("overlay-url")
                == style.StyleSourceType.IMAGE
            )
            assert (
                map_handle.get_style_source_type("overlay-inline")
                == style.StyleSourceType.IMAGE
            )
            assert map_handle.get_image_source_coordinates("overlay-url") == coordinates
            assert map_handle.get_image_source_coordinates("missing") is None

            map_handle.set_image_source_url(
                "overlay-url",
                "https://example.test/overlay-2.png",
            )
            map_handle.set_image_source_image("overlay-url", image)
            map_handle.set_image_source_coordinates("overlay-url", updated_coordinates)
            assert (
                map_handle.get_image_source_coordinates("overlay-url")
                == updated_coordinates
            )

            assert map_handle.remove_style_source("overlay-url") is True
            assert map_handle.remove_style_source("overlay-inline") is True


def test_style_json_light_layer_property_and_filter_public_api() -> None:
    background = json.from_python({"id": "json-background", "type": "background"})
    circle = json.from_python(
        {"id": "json-circle", "type": "circle", "source": "points"}
    )
    filter_value = json.json_array(
        (
            "==",
            json.json_array(("get", "kind")),
            "park",
        )
    )
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.set_style_json('{"version":8,"sources":{},"layers":[]}')
            map_handle.add_geojson_source_url(
                "points",
                "https://example.test/points.geojson",
            )
            map_handle.add_style_layer_json(background)
            map_handle.add_style_layer_json(circle)
            map_handle.set_layer_property(
                "json-background",
                "background-color",
                "#ff0000",
            )
            map_handle.set_layer_filter("json-circle", filter_value)
            map_handle.set_style_light_json(json.from_python({"anchor": "viewport"}))
            map_handle.set_style_light_property("intensity", json.json_double(0.5))

            layer_json = map_handle.get_style_layer_json("json-background")
            assert layer_json is not None
            assert ("id", "json-background") in json.to_python(layer_json)
            assert map_handle.get_style_layer_json("missing") is None
            background_color = map_handle.get_layer_property(
                "json-background",
                "background-color",
            )
            assert isinstance(background_color, json.JsonArray)
            assert background_color.values[0] == "rgba"
            assert map_handle.get_layer_filter("json-circle") == filter_value
            assert map_handle.get_style_light_property("anchor") == "viewport"
            assert map_handle.get_style_light_property("intensity") == json.json_double(
                0.5
            )

            map_handle.set_layer_filter("json-circle", None)
            assert map_handle.get_layer_filter("json-circle") is None


def test_style_image_metadata_copy_and_removal_public_api() -> None:
    image = render.PremultipliedRgba8Image(
        info=render.TextureImageInfo(width=1, height=1, stride=4, byte_length=4),
        data=bytes([255, 0, 0, 255]),
    )
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.set_style_json('{"version":8,"sources":{},"layers":[]}')
            map_handle.set_style_image(
                "marker",
                image,
                style.StyleImageOptions(pixel_ratio=2.0, sdf=True),
            )

            assert map_handle.style_image_exists("marker") is True
            assert map_handle.style_image_exists("missing") is False
            info = map_handle.get_style_image_info("marker")
            assert info is not None
            assert info.width == 1
            assert info.height == 1
            assert info.stride == 4
            assert info.byte_length == 4
            assert info.pixel_ratio == pytest.approx(2.0)
            assert info.sdf is True
            assert map_handle.get_style_image_info("missing") is None

            copied = map_handle.copy_style_image_premultiplied_rgba8("marker")
            assert copied is not None
            assert copied.image == image
            assert copied.pixel_ratio == pytest.approx(2.0)
            assert copied.sdf is True
            assert map_handle.copy_style_image_premultiplied_rgba8("missing") is None

            assert map_handle.remove_style_image("marker") is True
            assert map_handle.remove_style_image("marker") is False


def test_builtin_style_layers_and_location_indicator_public_api() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.set_style_json('{"version":8,"sources":{},"layers":[]}')
            map_handle.add_raster_dem_source_url(
                "dem",
                "https://example.test/dem.json",
                style.TileSourceOptions(
                    tile_size=512,
                    raster_dem_encoding=style.RasterDemEncoding.MAPBOX,
                ),
            )
            map_handle.add_hillshade_layer("hillshade", "dem")
            map_handle.add_color_relief_layer("relief", "dem")
            map_handle.add_location_indicator_layer("location")
            map_handle.set_location_indicator_location(
                "location",
                geo.LatLng(1.0, 2.0),
                3.0,
            )
            map_handle.set_location_indicator_bearing("location", 45.0)
            map_handle.set_location_indicator_accuracy_radius("location", 5.0)
            map_handle.set_location_indicator_image_name(
                "location",
                style.LocationIndicatorImageKind.TOP,
                "marker",
            )

            assert map_handle.get_style_layer_type("hillshade") == "hillshade"
            assert map_handle.get_style_layer_type("relief") == "color-relief"
            assert map_handle.get_style_layer_type("location") == "location-indicator"
            assert map_handle.remove_style_layer("hillshade") is True
            assert map_handle.remove_style_layer("relief") is True
            assert map_handle.remove_style_layer("location") is True


def test_style_layer_metadata_move_and_removal_public_api() -> None:
    style_json = """
    {
      "version": 8,
      "sources": {},
      "layers": [
        {"id": "background-a", "type": "background"},
        {"id": "background-b", "type": "background"}
      ]
    }
    """
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.set_style_json(style_json)

            layer_ids = map_handle.list_style_layer_ids()
            assert "background-a" in layer_ids
            assert "background-b" in layer_ids
            assert layer_ids.index("background-a") < layer_ids.index("background-b")
            assert map_handle.style_layer_exists("background-a") is True
            assert map_handle.style_layer_exists("missing") is False
            assert map_handle.get_style_layer_type("background-a") == "background"
            assert map_handle.get_style_layer_type("missing") is None

            map_handle.move_style_layer("background-b", "background-a")
            layer_ids = map_handle.list_style_layer_ids()
            assert layer_ids.index("background-b") < layer_ids.index("background-a")

            assert map_handle.remove_style_layer("background-b") is True
            assert map_handle.remove_style_layer("background-b") is False
            assert "background-b" not in map_handle.list_style_layer_ids()


def test_map_viewport_and_tile_options_round_trip_public_values() -> None:
    viewport = map_module.MapViewportOptions(
        north_orientation=map_module.NorthOrientation.RIGHT,
        constrain_mode=map_module.ConstrainMode.WIDTH_AND_HEIGHT,
        viewport_mode=map_module.ViewportMode.DEFAULT,
        frustum_offset=camera.EdgeInsets(top=1.0, left=2.0, bottom=3.0, right=4.0),
    )
    tile = map_module.MapTileOptions(
        prefetch_zoom_delta=1,
        lod_min_radius=1.0,
        lod_scale=1.0,
        lod_pitch_threshold=30.0,
        lod_zoom_shift=0.0,
        lod_mode=map_module.TileLodMode.DEFAULT,
    )

    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.set_viewport_options(viewport)
            map_handle.set_tile_options(tile)

            assert map_handle.get_viewport_options() == viewport
            assert map_handle.get_tile_options() == tile


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


def test_free_camera_and_projection_mode_round_trip_public_values() -> None:
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            free_camera = map_handle.get_free_camera_options()
            assert isinstance(free_camera, camera.FreeCameraOptions)

            projection = camera.ProjectionMode(
                axonometric=True,
                x_skew=0.1,
                y_skew=0.2,
            )
            map_handle.set_projection_mode(projection)
            snapshot = map_handle.get_projection_mode()

            assert snapshot.axonometric is True
            assert snapshot.x_skew == pytest.approx(0.1)
            assert snapshot.y_skew == pytest.approx(0.2)


def test_camera_transition_commands_accept_public_values() -> None:
    animation = camera.AnimationOptions(
        duration_ms=0.0,
        velocity=1.0,
        min_zoom=0.0,
        easing=camera.UnitBezier(0.0, 0.0, 1.0, 1.0),
    )
    target = camera.CameraOptions(center=geo.LatLng(0.0, 0.0), zoom=1.0)
    first = camera.ScreenPoint(0.0, 0.0)
    second = camera.ScreenPoint(1.0, 1.0)

    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.ease_to(target, animation)
            map_handle.fly_to(target, animation)
            map_handle.move_by_animated(1.0, 1.0, animation)
            map_handle.scale_by(1.0, first)
            map_handle.scale_by_animated(1.0, first, animation)
            map_handle.rotate_by(first, second)
            map_handle.rotate_by_animated(first, second, animation)
            map_handle.pitch_by(0.0)
            map_handle.pitch_by_animated(0.0, animation)
            map_handle.cancel_transitions()


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


def test_render_session_query_public_api_uses_query_and_geojson_wire_values() -> None:
    class FakeNativeRenderSession:
        closed = False
        detached = False

        def __init__(self) -> None:
            self.rendered_call = None
            self.source_call = None
            self.extension_call = None

        def query_rendered_features(
            self,
            geometry: object,
            layer_ids: tuple[str, ...] | None,
            filter_: object,
        ) -> list[dict[str, object]]:
            self.rendered_call = (geometry, layer_ids, filter_)
            return [queried_feature_wire()]

        def query_source_features(
            self,
            source_id: str,
            source_layer_ids: tuple[str, ...] | None,
            filter_: object,
        ) -> list[dict[str, object]]:
            self.source_call = (source_id, source_layer_ids, filter_)
            return [queried_feature_wire()]

        def query_feature_extensions(
            self,
            source_id: str,
            feature: object,
            extension: str,
            extension_field: str,
            arguments: object,
        ) -> dict[str, object]:
            self.extension_call = (
                source_id,
                feature,
                extension,
                extension_field,
                arguments,
            )
            return {"type": 1, "value": {"type": "uint", "value": 7}}

    def queried_feature_wire() -> dict[str, object]:
        return {
            "feature": {
                "geometry": {
                    "type": "point",
                    "coordinate": {"latitude": 1.0, "longitude": 2.0},
                },
                "properties": [("name", "one")],
                "identifier": {"type": "string", "value": "feature-1"},
            },
            "source_id": "points",
            "source_layer_id": None,
            "state": {"type": "object", "members": [("hover", True)]},
        }

    fake_native = FakeNativeRenderSession()
    session = render.RenderSessionHandle(fake_native, object())
    geometry = query.RenderedQueryGeometry.point_geometry(camera.ScreenPoint(1.0, 2.0))
    rendered_options = query.RenderedFeatureQueryOptions(
        layer_ids=("circle",),
        filter=json.from_python(["==", ["get", "kind"], "park"]),
    )
    source_options = query.SourceFeatureQueryOptions(
        source_layer_ids=("landuse",),
        filter=json.from_python(["==", ["get", "kind"], "park"]),
    )
    feature = geo.Feature(
        geometry=geo.point(1.0, 2.0),
        properties=(json.JsonMember("name", "one"),),
        identifier=geo.FeatureIdentifierString("feature-1"),
    )

    rendered = session.query_rendered_features(geometry, rendered_options)
    source = session.query_source_features("points", source_options)
    extension = session.query_feature_extensions(
        "points",
        feature,
        "supercluster",
        "leaves",
        json.JsonObject.from_pairs([("limit", json.JsonUInt(10))]),
    )

    assert fake_native.rendered_call == (
        {"type": "point", "point": (1.0, 2.0)},
        ("circle",),
        {
            "type": "array",
            "values": [
                "==",
                {"type": "array", "values": ["get", "kind"]},
                "park",
            ],
        },
    )
    assert fake_native.source_call[0] == "points"
    assert fake_native.source_call[1] == ("landuse",)
    assert rendered[0].feature == feature
    assert source[0].state == json.JsonObject.from_pairs([("hover", True)])
    assert fake_native.extension_call == (
        "points",
        {
            "geometry": {"type": "point", "coordinate": (1.0, 2.0)},
            "properties": [("name", "one")],
            "identifier": {"type": "string", "value": "feature-1"},
        },
        "supercluster",
        "leaves",
        {"type": "object", "members": [("limit", {"type": "uint", "value": 10})]},
    )
    assert extension == query.FeatureExtensionResult.value_result(json.JsonUInt(7))


def test_render_session_feature_state_public_api_uses_json_wire_values() -> None:
    class FakeNativeRenderSession:
        closed = False
        detached = False

        def __init__(self) -> None:
            self.set_call = None
            self.remove_call = None

        def set_feature_state(
            self,
            source_id: str,
            source_layer_id: str | None,
            feature_id: str | None,
            state_key: str | None,
            state: object,
        ) -> None:
            self.set_call = (
                source_id,
                source_layer_id,
                feature_id,
                state_key,
                state,
            )

        def get_feature_state(
            self,
            source_id: str,
            source_layer_id: str | None,
            feature_id: str | None,
            state_key: str | None,
        ) -> object:
            assert (source_id, source_layer_id, feature_id, state_key) == (
                "points",
                "symbols",
                "feature-1",
                "hover",
            )
            return {
                "type": "object",
                "members": [("hover", {"type": "bool", "value": True})],
            }

        def remove_feature_state(
            self,
            source_id: str,
            source_layer_id: str | None,
            feature_id: str | None,
            state_key: str | None,
        ) -> None:
            self.remove_call = (source_id, source_layer_id, feature_id, state_key)

    fake_native = FakeNativeRenderSession()
    session = render.RenderSessionHandle(fake_native, object())
    selector = query.FeatureStateSelector(
        source_id="points",
        source_layer_id="symbols",
        feature_id="feature-1",
        state_key="hover",
    )
    state = json.JsonObject.from_pairs([("hover", True)])

    session.set_feature_state(selector, state)
    returned = session.get_feature_state(selector)
    session.remove_feature_state(selector)

    assert fake_native.set_call == (
        "points",
        "symbols",
        "feature-1",
        "hover",
        {"type": "object", "members": [("hover", True)]},
    )
    assert returned == state
    assert fake_native.remove_call == ("points", "symbols", "feature-1", "hover")


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


def test_map_coordinate_conversions_round_trip_public_values() -> None:
    coordinate = geo.LatLng(0.0, 0.0)
    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.jump_to(camera.CameraOptions(center=coordinate, zoom=1.0))
            point = map_handle.pixel_for_lat_lng(coordinate)
            projected = map_handle.lat_lng_for_pixel(point)
            points = map_handle.pixels_for_lat_lngs((coordinate, geo.LatLng(1.0, 1.0)))
            coordinates = map_handle.lat_lngs_for_pixels(points)

            assert isinstance(point, camera.ScreenPoint)
            assert math.isfinite(projected.latitude)
            assert math.isfinite(projected.longitude)
            assert len(points) == 2
            assert len(coordinates) == 2
            assert all(isinstance(item, camera.ScreenPoint) for item in points)
            assert all(isinstance(item, geo.LatLng) for item in coordinates)


def test_map_projection_converts_coordinates_and_closes() -> None:
    coordinate = geo.LatLng(0.0, 0.0)
    meters = map_module.projected_meters_for_lat_lng(coordinate)
    round_tripped = map_module.lat_lng_for_projected_meters(meters)

    assert isinstance(meters, map_module.ProjectedMeters)
    assert math.isclose(round_tripped.latitude, coordinate.latitude, abs_tol=1e-6)
    assert math.isclose(round_tripped.longitude, coordinate.longitude, abs_tol=1e-6)

    with mln.RuntimeHandle() as runtime:
        with runtime.create_map() as map_handle:
            map_handle.jump_to(camera.CameraOptions(center=coordinate, zoom=1.0))
            with map_handle.create_projection() as projection:
                point = projection.pixel_for_lat_lng(coordinate)
                projected = projection.lat_lng_for_pixel(point)
                projection.set_camera(camera.CameraOptions(center=coordinate, zoom=2.0))
                projection.set_visible_coordinates(
                    (geo.LatLng(-1.0, -1.0), geo.LatLng(1.0, 1.0)),
                    camera.EdgeInsets(),
                )

                assert not projection.closed
                assert isinstance(projection.get_camera(), camera.CameraOptions)
                assert isinstance(point, camera.ScreenPoint)
                assert math.isfinite(projected.latitude)
                assert math.isfinite(projected.longitude)

            assert projection.closed


def test_ambient_cache_operation_starts_and_discards_through_public_api(
    tmp_path: Path,
) -> None:
    with mln.RuntimeHandle(mln.RuntimeOptions(cache_path=str(tmp_path))) as runtime:
        operation = runtime.run_ambient_cache_operation(
            offline.AmbientCacheOperation.CLEAR,
        )

        assert isinstance(operation, offline.OfflineOperationHandle)
        assert operation.operation_id > 0
        assert not operation.closed
        operation.close()
        assert operation.closed
        operation.close()


def test_offline_values_wrap_runtime_event_payload_shape() -> None:
    bounds = geo.LatLngBounds(geo.LatLng(1.0, 2.0), geo.LatLng(3.0, 4.0))
    definition = offline.OfflineTilePyramidRegionDefinition(
        style_url="https://example.test/style.json",
        bounds=bounds,
        min_zoom=1.0,
        max_zoom=3.0,
        pixel_ratio=2.0,
    )
    status = offline.OfflineRegionStatus(
        download_state=offline.OfflineRegionDownloadState.ACTIVE,
        completed_resource_count=1,
        completed_resource_size=2,
        completed_tile_count=3,
        required_tile_count=4,
        completed_tile_size=5,
        required_resource_count=6,
        required_resource_count_is_precise=True,
        complete=False,
    )
    completed = offline.OfflineOperationCompleted.from_runtime_payload(
        {
            "operation_id": 7,
            "operation_kind": offline.OfflineOperationKind.REGION_CREATE.native_code,
            "result_kind": offline.OfflineOperationResultKind.REGION,
            "result_status": mln.MaplibreStatus.OK.native_code,
            "found": True,
        }
    )

    assert (
        definition.definition_type == offline.OfflineRegionDefinitionType.TILE_PYRAMID
    )
    assert status.download_state == offline.OfflineRegionDownloadState.ACTIVE
    assert completed.operation_kind == offline.OfflineOperationKind.REGION_CREATE
    assert completed.result_kind == offline.OfflineOperationResultKind.REGION


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

            tile = style.CanonicalTileId(0, 0, 0)
            data = geo.FeatureCollection((geo.Feature(geometry=geo.point(0.0, 0.0)),))
            bounds = geo.LatLngBounds(
                southwest=geo.LatLng(-1.0, -1.0),
                northeast=geo.LatLng(1.0, 1.0),
            )
            map_handle.set_custom_geometry_source_tile_data("custom", tile, data)
            map_handle.invalidate_custom_geometry_source_tile("custom", tile)
            map_handle.invalidate_custom_geometry_source_region("custom", bounds)

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
