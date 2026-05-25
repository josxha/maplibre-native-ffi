use super::*;
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static PROVIDER_DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn passthrough_transform(
    _kind: u32,
    _url: *const c_char,
    _user_data: *mut c_void,
) -> *mut c_char {
    ptr::null_mut()
}

unsafe extern "C" fn replacing_transform(
    _kind: u32,
    _url: *const c_char,
    _user_data: *mut c_void,
) -> *mut c_char {
    crate::glib::copy_string_view(sys::mln_string_view {
        data: c"asset://replacement".as_ptr(),
        size: "asset://replacement".len(),
    })
    .expect("test replacement URL allocation should succeed")
}

unsafe extern "C" fn counted_transform_destroy_notify(user_data: *mut c_void) {
    assert!(!user_data.is_null());
    // SAFETY: Tests pass a live `AtomicUsize` pointer as user_data and
    // trigger destroy-notify before the counter leaves scope.
    unsafe { &*(user_data as *const AtomicUsize) }.fetch_add(1, Ordering::SeqCst);
}

unsafe extern "C" fn passthrough_provider(
    _request: *const sys::mln_resource_request,
    _handle: *mut ResourceRequestHandle,
    _user_data: *mut c_void,
) -> u32 {
    sys::MLN_RESOURCE_PROVIDER_DECISION_PASS_THROUGH
}

unsafe extern "C" fn provider_destroy_notify(_user_data: *mut c_void) {
    PROVIDER_DESTROY_COUNT.fetch_add(1, Ordering::SeqCst);
}

unsafe extern "C" fn custom_geometry_tile_callback(
    _user_data: *mut c_void,
    _tile_id: sys::mln_canonical_tile_id,
) {
}

unsafe extern "C" fn counted_custom_geometry_tile_delegate(
    _tile_id: sys::mln_canonical_tile_id,
    _user_data: *mut c_void,
) {
}

unsafe extern "C" fn counted_custom_geometry_destroy_notify(user_data: *mut c_void) {
    assert!(!user_data.is_null());
    // SAFETY: Tests pass a live `AtomicUsize` pointer as user_data and
    // trigger destroy-notify before the counter leaves scope.
    unsafe { &*(user_data as *const AtomicUsize) }.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn runtime_handle_create_run_and_close() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());

    assert_eq!(
        mln_vala_runtime_handle_run_once(runtime, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );

    glib::unref_object(runtime);
}

#[test]
fn runtime_and_map_options_create_handles() {
    // SAFETY: Zeroed storage is immediately initialized by default helpers.
    let mut runtime_options: sys::mln_runtime_options = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_runtime_options_default(&mut runtime_options, ptr::null_mut()),
        GTRUE
    );
    let runtime = mln_vala_runtime_handle_new_with_options(&runtime_options, ptr::null_mut());
    assert!(!runtime.is_null());

    // SAFETY: Zeroed storage is immediately initialized by default helpers.
    let mut map_options: sys::mln_map_options = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_map_options_default(&mut map_options, ptr::null_mut()),
        GTRUE
    );
    map_options.width = 128;
    map_options.height = 128;
    map_options.scale_factor = 1.0;
    let map = mln_vala_map_handle_new_with_options(runtime, &map_options, ptr::null_mut());
    assert!(!map.is_null());

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );

    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn map_owner_thread_errors_propagate_as_gerror() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 128, 128, 1.0, ptr::null_mut());
    assert!(!map.is_null());

    let map_bits = map as usize;
    let wrong_thread_failed = std::thread::spawn(move || {
        let map = map_bits as *mut MapHandle;
        let mut loaded = GFALSE;
        let mut error = ptr::null_mut();
        let ok = mln_vala_map_handle_is_fully_loaded(map, &mut loaded, &mut error);
        ok == GFALSE && !error.is_null()
    })
    .join()
    .expect("wrong-thread probe should not panic");
    assert!(wrong_thread_failed);

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );
    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn gobject_finalize_releases_runtime_callback_state() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let destroy_count = AtomicUsize::new(0);

    assert_eq!(
        mln_vala_runtime_handle_set_resource_transform(
            runtime,
            Some(passthrough_transform),
            &destroy_count as *const AtomicUsize as *mut c_void,
            Some(counted_transform_destroy_notify),
            ptr::null_mut(),
        ),
        GTRUE
    );
    glib::unref_object(runtime);
    assert_eq!(destroy_count.load(Ordering::SeqCst), 1);
}

#[test]
fn gobject_finalize_releases_map_before_runtime_close() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 128, 128, 1.0, ptr::null_mut());
    assert!(!map.is_null());

    glib::unref_object(map);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );
    glib::unref_object(runtime);
}

#[test]
fn resource_transform_replacement_and_clear_run_destroy_notify() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let destroy_count = AtomicUsize::new(0);

    assert_eq!(
        mln_vala_runtime_handle_set_resource_transform(
            runtime,
            Some(passthrough_transform),
            &destroy_count as *const AtomicUsize as *mut c_void,
            Some(counted_transform_destroy_notify),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(destroy_count.load(Ordering::SeqCst), 0);

    assert_eq!(
        mln_vala_runtime_handle_set_resource_transform(
            runtime,
            Some(passthrough_transform),
            &destroy_count as *const AtomicUsize as *mut c_void,
            Some(counted_transform_destroy_notify),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(destroy_count.load(Ordering::SeqCst), 1);

    assert_eq!(
        mln_vala_runtime_handle_clear_resource_transform(runtime, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(destroy_count.load(Ordering::SeqCst), 2);

    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );
    glib::unref_object(runtime);
}

#[test]
fn resource_transform_retains_non_null_replacement_urls_until_destroy() {
    let state = Box::into_raw(Box::new(ResourceTransformState {
        callback: replacing_transform,
        user_data: ptr::null_mut(),
        destroy_notify: None,
        returned_urls: Mutex::new(Vec::new()),
    }));
    // SAFETY: Zeroed storage is initialized by the trampoline before use.
    let mut first_response: sys::mln_resource_transform_response = unsafe { std::mem::zeroed() };
    // SAFETY: Zeroed storage is initialized by the trampoline before use.
    let mut second_response: sys::mln_resource_transform_response = unsafe { std::mem::zeroed() };

    // SAFETY: `state` is valid callback state, input URLs are static
    // NUL-terminated strings, and response pointers are writable.
    assert_eq!(
        unsafe {
            resource_transform_trampoline(
                state.cast::<c_void>(),
                sys::MLN_RESOURCE_KIND_STYLE,
                c"asset://one".as_ptr(),
                &mut first_response,
            )
        },
        sys::MLN_STATUS_OK
    );
    assert_eq!(
        unsafe {
            resource_transform_trampoline(
                state.cast::<c_void>(),
                sys::MLN_RESOURCE_KIND_STYLE,
                c"asset://two".as_ptr(),
                &mut second_response,
            )
        },
        sys::MLN_STATUS_OK
    );

    assert!(!first_response.url.is_null());
    assert!(!second_response.url.is_null());
    // SAFETY: The transform state retains both GLib-allocated URLs until
    // `destroy_resource_transform_state` below.
    assert_eq!(
        unsafe { CStr::from_ptr(first_response.url) },
        c"asset://replacement"
    );
    // SAFETY: The second response URL is likewise retained until teardown.
    assert_eq!(
        unsafe { CStr::from_ptr(second_response.url) },
        c"asset://replacement"
    );
    assert_eq!(
        unsafe { &*state }
            .returned_urls
            .lock()
            .expect("resource transform URL storage lock poisoned")
            .len(),
        2
    );

    destroy_resource_transform_state(state);
}

#[test]
fn resource_provider_registration_runs_destroy_notify_on_close() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    PROVIDER_DESTROY_COUNT.store(0, Ordering::SeqCst);

    assert_eq!(
        mln_vala_runtime_handle_set_resource_provider(
            runtime,
            Some(passthrough_provider),
            ptr::null_mut(),
            Some(provider_destroy_notify),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(PROVIDER_DESTROY_COUNT.load(Ordering::SeqCst), 0);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(PROVIDER_DESTROY_COUNT.load(Ordering::SeqCst), 1);
    glib::unref_object(runtime);
}

#[test]
fn offline_operation_wrappers_report_invalid_inputs() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());

    let mut operation_id = 0;
    let mut error = ptr::null_mut();
    assert_eq!(
        mln_vala_runtime_handle_run_ambient_cache_operation_start(
            runtime,
            0,
            &mut operation_id,
            &mut error,
        ),
        GFALSE
    );
    assert!(!error.is_null());

    error = ptr::null_mut();
    assert_eq!(
        mln_vala_runtime_handle_offline_regions_list_start(runtime, ptr::null_mut(), &mut error),
        GFALSE
    );
    assert!(!error.is_null());

    error = ptr::null_mut();
    assert_eq!(
        mln_vala_runtime_handle_offline_regions_merge_database_start(
            runtime,
            ptr::null(),
            &mut operation_id,
            &mut error,
        ),
        GFALSE
    );
    assert!(!error.is_null());

    error = ptr::null_mut();
    assert_eq!(
        mln_vala_runtime_handle_offline_region_update_metadata_start(
            runtime,
            1,
            ptr::null(),
            1,
            &mut operation_id,
            &mut error,
        ),
        GFALSE
    );
    assert!(!error.is_null());

    error = ptr::null_mut();
    assert_eq!(
        mln_vala_runtime_handle_offline_region_get_status_take_result(
            runtime,
            0,
            ptr::null_mut(),
            &mut error,
        ),
        GFALSE
    );
    assert!(!error.is_null());

    error = ptr::null_mut();
    assert_eq!(
        mln_vala_offline_region_snapshot_handle_get(ptr::null_mut(), ptr::null_mut(), &mut error),
        GFALSE
    );
    assert!(!error.is_null());

    error = ptr::null_mut();
    assert_eq!(
        mln_vala_offline_region_list_handle_get(ptr::null_mut(), 0, ptr::null_mut(), &mut error),
        GFALSE
    );
    assert!(!error.is_null());

    error = ptr::null_mut();
    assert_eq!(
        mln_vala_runtime_handle_offline_operation_discard(runtime, 0, &mut error),
        GFALSE
    );
    assert!(!error.is_null());

    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );
    glib::unref_object(runtime);
}

#[test]
fn map_handle_retains_runtime_until_close() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );

    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn offline_region_definition_initializer_sets_nested_sizes() {
    // SAFETY: Zeroed storage is initialized by the adapter helper before
    // being passed to the C ABI.
    let mut tile_definition: sys::mln_offline_region_definition = unsafe { std::mem::zeroed() };
    tile_definition.type_ = sys::MLN_OFFLINE_REGION_DEFINITION_TILE_PYRAMID;
    initialize_offline_region_definition(&mut tile_definition);
    assert_eq!(
        tile_definition.size,
        std::mem::size_of::<sys::mln_offline_region_definition>() as u32
    );
    assert_eq!(
        unsafe { tile_definition.data.tile_pyramid.size },
        std::mem::size_of::<sys::mln_offline_tile_pyramid_region_definition>() as u32
    );

    // SAFETY: Zeroed storage is initialized by the adapter helper before
    // being passed to the C ABI.
    let mut geometry_definition: sys::mln_offline_region_definition = unsafe { std::mem::zeroed() };
    geometry_definition.type_ = sys::MLN_OFFLINE_REGION_DEFINITION_GEOMETRY;
    initialize_offline_region_definition(&mut geometry_definition);
    assert_eq!(
        geometry_definition.size,
        std::mem::size_of::<sys::mln_offline_region_definition>() as u32
    );
    assert_eq!(
        unsafe { geometry_definition.data.geometry.size },
        std::mem::size_of::<sys::mln_offline_geometry_region_definition>() as u32
    );
}

#[test]
fn owned_offline_region_definitions_copy_and_expose_safe_accessors() {
    let point = LatLng {
        latitude: 1.0,
        longitude: 2.0,
    };
    let geometry = values::mln_vala_geometry_new_point(&point, ptr::null_mut());
    assert!(!geometry.is_null());

    let definition = mln_vala_offline_region_definition_new_geometry(
        c"asset://style.json".as_ptr(),
        geometry,
        0.0,
        4.0,
        1.0,
        false,
        ptr::null_mut(),
    );
    assert!(!definition.is_null());
    assert_eq!(
        mln_vala_offline_region_definition_get_definition_type(definition),
        sys::MLN_OFFLINE_REGION_DEFINITION_GEOMETRY
    );
    assert_eq!(
        mln_vala_offline_region_definition_get_min_zoom(definition),
        0.0
    );
    assert_eq!(
        mln_vala_offline_region_definition_get_max_zoom(definition),
        4.0
    );
    assert_eq!(
        mln_vala_offline_region_definition_get_pixel_ratio(definition),
        1.0
    );
    assert!(!mln_vala_offline_region_definition_get_include_ideographs(
        definition
    ));
    let style_url = mln_vala_offline_region_definition_dup_style_url(definition);
    assert!(!style_url.is_null());
    // SAFETY: String was allocated with GLib allocation by the wrapper.
    unsafe { glib::free(style_url.cast::<c_void>()) };
    let copied_geometry = mln_vala_offline_region_definition_get_geometry(definition);
    assert!(!copied_geometry.is_null());
    values::mln_vala_geometry_free(copied_geometry);

    let bounds = sys::mln_lat_lng_bounds {
        southwest: sys::mln_lat_lng {
            latitude: -1.0,
            longitude: -2.0,
        },
        northeast: sys::mln_lat_lng {
            latitude: 1.0,
            longitude: 2.0,
        },
    };
    let tile_definition = mln_vala_offline_region_definition_new_tile_pyramid(
        c"asset://style.json".as_ptr(),
        &bounds,
        1.0,
        8.0,
        2.0,
        true,
        ptr::null_mut(),
    );
    assert!(!tile_definition.is_null());
    assert_eq!(
        mln_vala_offline_region_definition_get_definition_type(tile_definition),
        sys::MLN_OFFLINE_REGION_DEFINITION_TILE_PYRAMID
    );
    let mut copied_bounds: sys::mln_lat_lng_bounds = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_offline_region_definition_get_bounds(tile_definition, &mut copied_bounds),
        GTRUE
    );
    assert_eq!(copied_bounds.southwest.latitude, -1.0);
    assert!(mln_vala_offline_region_definition_get_geometry(tile_definition).is_null());

    mln_vala_offline_region_definition_free(tile_definition);
    mln_vala_offline_region_definition_free(definition);
    values::mln_vala_geometry_free(geometry);
}

#[test]
fn custom_geometry_destroy_waits_for_in_flight_callback() {
    let destroy_count = AtomicUsize::new(0);
    let state = Box::into_raw(Box::new(CustomGeometrySourceCallbackState {
        source_id: CString::new("custom-geometry-source").unwrap(),
        fetch_tile: counted_custom_geometry_tile_delegate,
        fetch_user_data: &destroy_count as *const AtomicUsize as *mut c_void,
        fetch_destroy_notify: Some(counted_custom_geometry_destroy_notify),
        cancel_tile: None,
        cancel_user_data: ptr::null_mut(),
        cancel_destroy_notify: None,
        lifecycle: Mutex::new(CustomGeometrySourceLifecycle {
            closing: false,
            active_callbacks: 0,
        }),
        idle: Condvar::new(),
    }));
    let guard = custom_geometry_enter_callback(unsafe { &*state })
        .expect("callback guard should enter before teardown starts");

    let teardown_started = std::sync::Arc::new(AtomicBool::new(false));
    let teardown_started_on_thread = teardown_started.clone();
    let state_bits = state as usize;
    let teardown = std::thread::spawn(move || {
        teardown_started_on_thread.store(true, Ordering::SeqCst);
        destroy_custom_geometry_state(state_bits as *mut CustomGeometrySourceCallbackState);
    });

    while !teardown_started.load(Ordering::SeqCst) {
        std::thread::yield_now();
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut observed_closing = false;
    while Instant::now() < deadline {
        let closing = unsafe { &*state }
            .lifecycle
            .lock()
            .expect("custom geometry lifecycle lock should not be poisoned")
            .closing;
        if closing {
            observed_closing = true;
            break;
        }
        std::thread::yield_now();
    }
    assert!(observed_closing);
    assert_eq!(destroy_count.load(Ordering::SeqCst), 0);

    drop(guard);
    teardown
        .join()
        .expect("custom geometry teardown should finish after callback exits");
    assert_eq!(destroy_count.load(Ordering::SeqCst), 1);
}

#[test]
fn custom_geometry_callbacks_release_on_source_removal() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());
    assert_eq!(
        mln_vala_map_handle_set_style_json(
            map,
            c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let destroy_count = AtomicUsize::new(0);
    assert_eq!(
        mln_vala_map_handle_add_custom_geometry_source_with_callbacks(
            map,
            c"custom-geometry-source".as_ptr(),
            Some(counted_custom_geometry_tile_delegate),
            &destroy_count as *const AtomicUsize as *mut c_void,
            Some(counted_custom_geometry_destroy_notify),
            None,
            ptr::null_mut(),
            None,
            ptr::null(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let mut removed = GFALSE;
    assert_eq!(
        mln_vala_map_handle_remove_style_source(
            map,
            c"custom-geometry-source".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(removed, GTRUE);
    assert_eq!(destroy_count.load(Ordering::SeqCst), 1);

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );
    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn custom_geometry_callbacks_block_url_style_replacement() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());
    assert_eq!(
        mln_vala_map_handle_set_style_json(
            map,
            c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let destroy_count = AtomicUsize::new(0);
    assert_eq!(
        mln_vala_map_handle_add_custom_geometry_source_with_callbacks(
            map,
            c"custom-geometry-source".as_ptr(),
            Some(counted_custom_geometry_tile_delegate),
            &destroy_count as *const AtomicUsize as *mut c_void,
            Some(counted_custom_geometry_destroy_notify),
            None,
            ptr::null_mut(),
            None,
            ptr::null(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let mut error = ptr::null_mut();
    assert_eq!(
        mln_vala_map_handle_set_style_url(map, c"asset://style.json".as_ptr(), &mut error),
        GFALSE
    );
    assert!(!error.is_null());
    assert_eq!(destroy_count.load(Ordering::SeqCst), 0);

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(destroy_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );
    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn custom_geometry_callbacks_release_on_inline_style_replacement() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());
    assert_eq!(
        mln_vala_map_handle_set_style_json(
            map,
            c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let destroy_count = AtomicUsize::new(0);
    assert_eq!(
        mln_vala_map_handle_add_custom_geometry_source_with_callbacks(
            map,
            c"custom-geometry-source".as_ptr(),
            Some(counted_custom_geometry_tile_delegate),
            &destroy_count as *const AtomicUsize as *mut c_void,
            Some(counted_custom_geometry_destroy_notify),
            None,
            ptr::null_mut(),
            None,
            ptr::null(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(destroy_count.load(Ordering::SeqCst), 0);
    assert_eq!(
        mln_vala_map_handle_set_style_json(
            map,
            c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(destroy_count.load(Ordering::SeqCst), 1);

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );
    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn geojson_source_url_lifecycle_round_trips_existence() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());

    assert_eq!(
        mln_vala_map_handle_set_style_json(
            map,
            c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_add_geojson_source_url(
            map,
            c"fixture-source".as_ptr(),
            c"asset://fixture.geojson".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let mut exists = GFALSE;
    assert_eq!(
        mln_vala_map_handle_style_source_exists(
            map,
            c"fixture-source".as_ptr(),
            &mut exists,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(exists, GTRUE);
    let mut source_type = sys::MLN_STYLE_SOURCE_TYPE_UNKNOWN;
    let mut found = GFALSE;
    assert_eq!(
        mln_vala_map_handle_get_style_source_type(
            map,
            c"fixture-source".as_ptr(),
            &mut source_type,
            &mut found,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(found, GTRUE);
    assert_eq!(source_type, sys::MLN_STYLE_SOURCE_TYPE_GEOJSON);
    let mut source_info = sys::mln_style_source_info {
        size: std::mem::size_of::<sys::mln_style_source_info>() as u32,
        type_: sys::MLN_STYLE_SOURCE_TYPE_UNKNOWN,
        id_size: 0,
        is_volatile: false,
        has_attribution: false,
        attribution_size: 0,
    };
    assert_eq!(
        mln_vala_map_handle_get_style_source_info(
            map,
            c"fixture-source".as_ptr(),
            &mut source_info,
            &mut found,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(found, GTRUE);
    assert_eq!(source_info.type_, sys::MLN_STYLE_SOURCE_TYPE_GEOJSON);
    assert_eq!(
        mln_vala_map_handle_set_geojson_source_url(
            map,
            c"fixture-source".as_ptr(),
            c"asset://fixture-updated.geojson".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let mut removed = GFALSE;
    assert_eq!(
        mln_vala_map_handle_remove_style_source(
            map,
            c"fixture-source".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(removed, GTRUE);

    let geojson = GeoJson {
        value: maplibre_native_core::GeoJson::Geometry(maplibre_native_core::Geometry::Point(
            maplibre_native_core::LatLng::new(0.0, 0.0),
        )),
    };
    assert_eq!(
        mln_vala_map_handle_add_geojson_source_data(
            map,
            c"fixture-data-source".as_ptr(),
            &geojson,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_geojson_source_data(
            map,
            c"fixture-data-source".as_ptr(),
            &geojson,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_remove_style_source(
            map,
            c"fixture-data-source".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(removed, GTRUE);

    let mut custom_options = unsafe { sys::mln_custom_geometry_source_options_default() };
    assert_eq!(
        mln_vala_custom_geometry_source_options_default(&mut custom_options, ptr::null_mut()),
        GTRUE
    );
    custom_options.fetch_tile = Some(custom_geometry_tile_callback);
    assert_eq!(
        mln_vala_map_handle_add_custom_geometry_source(
            map,
            c"custom-geometry-source".as_ptr(),
            &custom_options,
            ptr::null_mut(),
        ),
        GTRUE
    );
    let tile_id = sys::mln_canonical_tile_id { z: 0, x: 0, y: 0 };
    assert_eq!(
        mln_vala_map_handle_set_custom_geometry_source_tile_data(
            map,
            c"custom-geometry-source".as_ptr(),
            &tile_id,
            &geojson,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_invalidate_custom_geometry_source_tile(
            map,
            c"custom-geometry-source".as_ptr(),
            &tile_id,
            ptr::null_mut(),
        ),
        GTRUE
    );
    let bounds = sys::mln_lat_lng_bounds {
        southwest: sys::mln_lat_lng {
            latitude: -1.0,
            longitude: -1.0,
        },
        northeast: sys::mln_lat_lng {
            latitude: 1.0,
            longitude: 1.0,
        },
    };
    assert_eq!(
        mln_vala_map_handle_invalidate_custom_geometry_source_region(
            map,
            c"custom-geometry-source".as_ptr(),
            &bounds,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_remove_style_source(
            map,
            c"custom-geometry-source".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(removed, GTRUE);

    let mut tile_options = unsafe { sys::mln_style_tile_source_options_default() };
    assert_eq!(
        mln_vala_style_tile_source_options_default(&mut tile_options, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_add_vector_source_url(
            map,
            c"vector-source".as_ptr(),
            c"asset://vector-source.json".as_ptr(),
            &tile_options,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_remove_style_source(
            map,
            c"vector-source".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );
    let vector_tile_url = c"asset://vector/{z}/{x}/{y}.pbf";
    let vector_tile_values = [vector_tile_url.as_ptr()];
    let vector_tiles = crate::string_list::mln_vala_string_list_new(
        vector_tile_values.as_ptr(),
        vector_tile_values.len(),
        ptr::null_mut(),
    );
    assert!(!vector_tiles.is_null());
    assert_eq!(
        mln_vala_map_handle_add_vector_source_tiles(
            map,
            c"vector-tiles-source".as_ptr(),
            vector_tiles,
            &tile_options,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_remove_style_source(
            map,
            c"vector-tiles-source".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );
    let raster_tile_url = c"asset://raster/{z}/{x}/{y}.png";
    let raster_tile_values = [raster_tile_url.as_ptr()];
    let raster_tiles = crate::string_list::mln_vala_string_list_new(
        raster_tile_values.as_ptr(),
        raster_tile_values.len(),
        ptr::null_mut(),
    );
    assert!(!raster_tiles.is_null());
    assert_eq!(
        mln_vala_map_handle_add_raster_source_tiles(
            map,
            c"raster-tiles-source".as_ptr(),
            raster_tiles,
            &tile_options,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_remove_style_source(
            map,
            c"raster-tiles-source".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );
    let dem_tile_url = c"asset://dem/{z}/{x}/{y}.png";
    let dem_tile_values = [dem_tile_url.as_ptr()];
    let dem_tiles = crate::string_list::mln_vala_string_list_new(
        dem_tile_values.as_ptr(),
        dem_tile_values.len(),
        ptr::null_mut(),
    );
    assert!(!dem_tiles.is_null());
    assert_eq!(
        mln_vala_map_handle_add_raster_dem_source_tiles(
            map,
            c"dem-tiles-source".as_ptr(),
            dem_tiles,
            &tile_options,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_remove_style_source(
            map,
            c"dem-tiles-source".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );

    crate::string_list::mln_vala_string_list_free(vector_tiles);
    crate::string_list::mln_vala_string_list_free(raster_tiles);
    crate::string_list::mln_vala_string_list_free(dem_tiles);

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );

    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn style_image_lifecycle_round_trips_metadata_and_pixels() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());

    assert_eq!(
        mln_vala_map_handle_set_style_json(
            map,
            c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );

    let pixels = [255_u8, 0, 0, 255];
    // SAFETY: Zeroed storage is immediately initialized by the public init
    // helper before use.
    let mut image: sys::mln_premultiplied_rgba8_image = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_premultiplied_rgba8_image_init(
            &mut image,
            1,
            1,
            4,
            pixels.as_ptr(),
            pixels.len(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(image.byte_length, pixels.len());

    // SAFETY: Zeroed storage is immediately initialized by default helpers.
    let mut options: sys::mln_style_image_options = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_style_image_options_default(&mut options, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_style_image(
            map,
            c"fixture-image".as_ptr(),
            &image,
            &options,
            ptr::null_mut(),
        ),
        GTRUE
    );

    let mut exists = GFALSE;
    assert_eq!(
        mln_vala_map_handle_style_image_exists(
            map,
            c"fixture-image".as_ptr(),
            &mut exists,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(exists, GTRUE);

    // SAFETY: Zeroed storage is immediately initialized by the wrapper.
    let mut info: sys::mln_style_image_info = unsafe { std::mem::zeroed() };
    let mut found = GFALSE;
    assert_eq!(
        mln_vala_map_handle_get_style_image_info(
            map,
            c"fixture-image".as_ptr(),
            &mut info,
            &mut found,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(found, GTRUE);
    assert_eq!(info.width, 1);
    assert_eq!(info.byte_length, pixels.len());

    let mut copied = [0_u8; 4];
    let mut byte_length = 0;
    assert_eq!(
        mln_vala_map_handle_copy_style_image_premultiplied_rgba8(
            map,
            c"fixture-image".as_ptr(),
            copied.as_mut_ptr(),
            copied.len(),
            &mut byte_length,
            &mut found,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(found, GTRUE);
    assert_eq!(byte_length, pixels.len());
    assert_eq!(copied, pixels);

    let mut removed = GFALSE;
    assert_eq!(
        mln_vala_map_handle_remove_style_image(
            map,
            c"fixture-image".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(removed, GTRUE);

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );

    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn image_source_lifecycle_round_trips_coordinates() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());

    assert_eq!(
        mln_vala_map_handle_set_style_json(
            map,
            c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );

    let coordinates = [
        sys::mln_lat_lng {
            latitude: 0.0,
            longitude: 0.0,
        },
        sys::mln_lat_lng {
            latitude: 0.0,
            longitude: 1.0,
        },
        sys::mln_lat_lng {
            latitude: 1.0,
            longitude: 1.0,
        },
        sys::mln_lat_lng {
            latitude: 1.0,
            longitude: 0.0,
        },
    ];
    assert_eq!(
        mln_vala_map_handle_add_image_source_url(
            map,
            c"image-source".as_ptr(),
            coordinates.as_ptr(),
            coordinates.len(),
            c"asset://image.png".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_image_source_url(
            map,
            c"image-source".as_ptr(),
            c"asset://image-updated.png".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_image_source_coordinates(
            map,
            c"image-source".as_ptr(),
            coordinates.as_ptr(),
            coordinates.len(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let mut copied_coordinates = [sys::mln_lat_lng {
        latitude: 0.0,
        longitude: 0.0,
    }; 4];
    let mut coordinate_count = 0;
    let mut found = GFALSE;
    assert_eq!(
        mln_vala_map_handle_get_image_source_coordinates(
            map,
            c"image-source".as_ptr(),
            copied_coordinates.as_mut_ptr(),
            copied_coordinates.len(),
            &mut coordinate_count,
            &mut found,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(found, GTRUE);
    assert_eq!(coordinate_count, 4);
    assert_eq!(copied_coordinates[2].latitude, 1.0);
    let mut removed = GFALSE;
    assert_eq!(
        mln_vala_map_handle_remove_style_source(
            map,
            c"image-source".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(removed, GTRUE);

    let pixels = [255_u8, 0, 0, 255];
    // SAFETY: Zeroed storage is immediately initialized by the public init helper.
    let mut image: sys::mln_premultiplied_rgba8_image = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_premultiplied_rgba8_image_init(
            &mut image,
            1,
            1,
            4,
            pixels.as_ptr(),
            pixels.len(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_add_image_source_image(
            map,
            c"inline-image-source".as_ptr(),
            coordinates.as_ptr(),
            coordinates.len(),
            &image,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_image_source_image(
            map,
            c"inline-image-source".as_ptr(),
            &image,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_remove_style_source(
            map,
            c"inline-image-source".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );

    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn location_indicator_layer_lifecycle_round_trips_existence() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());

    assert_eq!(
        mln_vala_map_handle_set_style_json(
            map,
            c"{\"version\":8,\"sources\":{},\"layers\":[]}".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_add_location_indicator_layer(
            map,
            c"location-layer".as_ptr(),
            c"".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let coordinate = sys::mln_lat_lng {
        latitude: 37.7749,
        longitude: -122.4194,
    };
    assert_eq!(
        mln_vala_map_handle_set_location_indicator_location(
            map,
            c"location-layer".as_ptr(),
            &coordinate,
            0.0,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_location_indicator_bearing(
            map,
            c"location-layer".as_ptr(),
            15.0,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_location_indicator_accuracy_radius(
            map,
            c"location-layer".as_ptr(),
            10.0,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_location_indicator_image_name(
            map,
            c"location-layer".as_ptr(),
            sys::MLN_LOCATION_INDICATOR_IMAGE_KIND_TOP,
            c"fixture-image".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let mut exists = GFALSE;
    assert_eq!(
        mln_vala_map_handle_style_layer_exists(
            map,
            c"location-layer".as_ptr(),
            &mut exists,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(exists, GTRUE);

    let mut layer_type = ptr::null_mut();
    let mut found = GFALSE;
    assert_eq!(
        mln_vala_map_handle_get_style_layer_type(
            map,
            c"location-layer".as_ptr(),
            &mut layer_type,
            &mut found,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(found, GTRUE);
    assert!(!layer_type.is_null());
    // SAFETY: Layer type was allocated with GLib allocation by the wrapper.
    unsafe { glib::free(layer_type.cast::<c_void>()) };

    let layer_ids = mln_vala_map_handle_list_style_layer_ids(map, ptr::null_mut());
    assert!(!layer_ids.is_null());
    let layer_count = crate::string_list::mln_vala_string_list_count(layer_ids);
    assert_ne!(layer_count, 0);
    let first_layer_id =
        crate::string_list::mln_vala_string_list_get(layer_ids, 0, ptr::null_mut());
    assert!(!first_layer_id.is_null());
    // SAFETY: ID was allocated with GLib allocation by the wrapper.
    unsafe { glib::free(first_layer_id.cast::<c_void>()) };
    crate::string_list::mln_vala_string_list_free(layer_ids);

    let source_ids = mln_vala_map_handle_list_style_source_ids(map, ptr::null_mut());
    assert!(!source_ids.is_null());
    let source_count = crate::string_list::mln_vala_string_list_count(source_ids);
    if source_count > 0 {
        let first_source_id =
            crate::string_list::mln_vala_string_list_get(source_ids, 0, ptr::null_mut());
        assert!(!first_source_id.is_null());
        // SAFETY: ID was allocated with GLib allocation by the wrapper.
        unsafe { glib::free(first_source_id.cast::<c_void>()) };
    }
    crate::string_list::mln_vala_string_list_free(source_ids);

    assert_eq!(
        mln_vala_map_handle_move_style_layer(
            map,
            c"location-layer".as_ptr(),
            c"".as_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let mut removed = GFALSE;
    assert_eq!(
        mln_vala_map_handle_remove_style_layer(
            map,
            c"location-layer".as_ptr(),
            &mut removed,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(removed, GTRUE);

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );

    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn map_debug_and_rendering_stats_options_round_trip() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());

    assert_eq!(
        mln_vala_map_handle_set_debug_options(
            map,
            sys::MLN_MAP_DEBUG_TILE_BORDERS,
            ptr::null_mut(),
        ),
        GTRUE
    );
    let mut debug_options = 0;
    assert_eq!(
        mln_vala_map_handle_get_debug_options(map, &mut debug_options, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(debug_options, sys::MLN_MAP_DEBUG_TILE_BORDERS);

    assert_eq!(
        mln_vala_map_handle_set_rendering_stats_view_enabled(map, GTRUE, ptr::null_mut()),
        GTRUE
    );
    let mut rendering_stats_enabled = GFALSE;
    assert_eq!(
        mln_vala_map_handle_get_rendering_stats_view_enabled(
            map,
            &mut rendering_stats_enabled,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(rendering_stats_enabled, GTRUE);

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );

    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn map_camera_commands_accept_default_descriptors() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());

    // SAFETY: Zeroed storage is immediately initialized through the public
    // default entry point before use.
    let mut camera: sys::mln_camera_options = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_camera_options_default(&mut camera, ptr::null_mut()),
        GTRUE
    );
    assert_ne!(camera.size, 0);
    assert_eq!(
        mln_vala_map_handle_get_camera(map, &mut camera, ptr::null_mut()),
        GTRUE
    );

    // SAFETY: Zeroed storage is immediately initialized through the public
    // default entry point before use.
    let mut animation: sys::mln_animation_options = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_animation_options_default(&mut animation, ptr::null_mut()),
        GTRUE
    );
    assert_ne!(animation.size, 0);

    // SAFETY: Zeroed storage is immediately initialized through the public
    // default entry point before use.
    let mut fit_options: sys::mln_camera_fit_options = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_camera_fit_options_default(&mut fit_options, ptr::null_mut()),
        GTRUE
    );
    assert_ne!(fit_options.size, 0);
    let bounds = sys::mln_lat_lng_bounds {
        southwest: sys::mln_lat_lng {
            latitude: -1.0,
            longitude: -1.0,
        },
        northeast: sys::mln_lat_lng {
            latitude: 1.0,
            longitude: 1.0,
        },
    };
    assert_eq!(
        mln_vala_map_handle_camera_for_lat_lng_bounds(
            map,
            &bounds,
            &fit_options,
            &mut camera,
            ptr::null_mut(),
        ),
        GTRUE
    );
    let fit_coordinates = [
        LatLng {
            latitude: -1.0,
            longitude: -1.0,
        },
        LatLng {
            latitude: 1.0,
            longitude: 1.0,
        },
    ];
    assert_eq!(
        mln_vala_map_handle_camera_for_lat_lngs(
            map,
            fit_coordinates.as_ptr(),
            fit_coordinates.len(),
            &fit_options,
            &mut camera,
            ptr::null_mut(),
        ),
        GTRUE
    );
    // SAFETY: Zeroed storage is overwritten through the public output API.
    let mut fitted_bounds: sys::mln_lat_lng_bounds = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_map_handle_lat_lng_bounds_for_camera(
            map,
            &camera,
            &mut fitted_bounds,
            ptr::null_mut(),
        ),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_lat_lng_bounds_for_camera_unwrapped(
            map,
            &camera,
            &mut fitted_bounds,
            ptr::null_mut(),
        ),
        GTRUE
    );

    assert_eq!(
        mln_vala_map_handle_jump_to(map, &camera, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_ease_to(map, &camera, &animation, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_move_by(map, 0.0, 0.0, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_scale_by(map, 1.0, ptr::null(), ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_pitch_by(map, 0.0, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_cancel_transitions(map, ptr::null_mut()),
        GTRUE
    );

    let coordinate = LatLng {
        latitude: 0.0,
        longitude: 0.0,
    };
    let mut point = ScreenPoint { x: 0.0, y: 0.0 };
    assert_eq!(
        mln_vala_map_handle_pixel_for_lat_lng(map, &coordinate, &mut point, ptr::null_mut()),
        GTRUE
    );
    let mut round_trip = LatLng {
        latitude: 999.0,
        longitude: 999.0,
    };
    assert_eq!(
        mln_vala_map_handle_lat_lng_for_pixel(map, &point, &mut round_trip, ptr::null_mut()),
        GTRUE
    );
    let coordinates = [coordinate];
    let mut points = [ScreenPoint { x: 0.0, y: 0.0 }];
    assert_eq!(
        mln_vala_map_handle_pixels_for_lat_lngs(
            map,
            coordinates.as_ptr(),
            coordinates.len(),
            points.as_mut_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );
    let mut round_trips = [LatLng {
        latitude: 0.0,
        longitude: 0.0,
    }];
    assert_eq!(
        mln_vala_map_handle_lat_lngs_for_pixels(
            map,
            points.as_ptr(),
            points.len(),
            round_trips.as_mut_ptr(),
            ptr::null_mut(),
        ),
        GTRUE
    );

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );

    glib::unref_object(map);
    glib::unref_object(runtime);
}

#[test]
fn map_state_option_descriptors_round_trip() {
    let runtime = mln_vala_runtime_handle_new(ptr::null_mut());
    assert!(!runtime.is_null());
    let map = mln_vala_map_handle_new(runtime, 512, 512, 1.0, ptr::null_mut());
    assert!(!map.is_null());

    // SAFETY: Zeroed storage is immediately initialized by default helpers.
    let mut viewport: sys::mln_map_viewport_options = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_map_viewport_options_default(&mut viewport, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_get_viewport_options(map, &mut viewport, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_viewport_options(map, &viewport, ptr::null_mut()),
        GTRUE
    );

    // SAFETY: Zeroed storage is immediately initialized by default helpers.
    let mut tile: sys::mln_map_tile_options = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_map_tile_options_default(&mut tile, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_get_tile_options(map, &mut tile, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_tile_options(map, &tile, ptr::null_mut()),
        GTRUE
    );

    // SAFETY: Zeroed storage is immediately initialized by default helpers.
    let mut bounds: sys::mln_bound_options = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_bound_options_default(&mut bounds, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_get_bounds(map, &mut bounds, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_bounds(map, &bounds, ptr::null_mut()),
        GTRUE
    );

    // SAFETY: Zeroed storage is immediately initialized by default helpers.
    let mut free_camera: sys::mln_free_camera_options = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_free_camera_options_default(&mut free_camera, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_get_free_camera_options(map, &mut free_camera, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_free_camera_options(map, &free_camera, ptr::null_mut()),
        GTRUE
    );

    // SAFETY: Zeroed storage is immediately initialized by default helpers.
    let mut projection_mode: sys::mln_projection_mode = unsafe { std::mem::zeroed() };
    assert_eq!(
        mln_vala_projection_mode_default(&mut projection_mode, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_get_projection_mode(map, &mut projection_mode, ptr::null_mut()),
        GTRUE
    );
    assert_eq!(
        mln_vala_map_handle_set_projection_mode(map, &projection_mode, ptr::null_mut()),
        GTRUE
    );

    assert_eq!(mln_vala_map_handle_close(map, ptr::null_mut()), GTRUE);
    assert_eq!(
        mln_vala_runtime_handle_close(runtime, ptr::null_mut()),
        GTRUE
    );

    glib::unref_object(map);
    glib::unref_object(runtime);
}
