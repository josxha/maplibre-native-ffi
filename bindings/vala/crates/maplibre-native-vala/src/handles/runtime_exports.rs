use super::*;

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_new(error_out: *mut *mut GError) -> *mut RuntimeHandle {
    gerror_pointer(create_runtime_handle(ptr::null()), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_new_with_options(
    options: *const sys::mln_runtime_options,
    error_out: *mut *mut GError,
) -> *mut RuntimeHandle {
    gerror_pointer(create_runtime_handle(options), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_close(
    handle: *mut RuntimeHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(close_runtime_handle(handle), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_run_once(
    handle: *mut RuntimeHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        runtime_native(handle).and_then(|runtime| {
            // SAFETY: `runtime_native` returns a live native runtime pointer.
            error::check(unsafe { sys::mln_runtime_run_once(runtime) })
        }),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_poll_event(
    handle: *mut RuntimeHandle,
    out_event: *mut *mut RuntimeEvent,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(poll_runtime_event(handle, out_event), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_set_resource_transform(
    handle: *mut RuntimeHandle,
    callback: Option<ResourceTransformCallback>,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_resource_transform(handle, callback, user_data, destroy_notify),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_clear_resource_transform(
    handle: *mut RuntimeHandle,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(clear_resource_transform(handle), error_out)
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_set_resource_provider(
    handle: *mut RuntimeHandle,
    callback: Option<ResourceProviderCallback>,
    user_data: *mut c_void,
    destroy_notify: GDestroyNotify,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        set_resource_provider(handle, callback, user_data, destroy_notify),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_run_ambient_cache_operation_start(
    handle: *mut RuntimeHandle,
    operation: u32,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        run_ambient_cache_operation_start(handle, operation, out_operation_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_create_start(
    handle: *mut RuntimeHandle,
    definition: *const OfflineRegionDefinition,
    metadata: *const u8,
    metadata_size: usize,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_region_create_start(
            handle,
            definition,
            metadata,
            metadata_size,
            out_operation_id,
        ),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_get_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_region_get_start(handle, region_id, out_operation_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_regions_list_start(
    handle: *mut RuntimeHandle,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_regions_list_start(handle, out_operation_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_regions_merge_database_start(
    handle: *mut RuntimeHandle,
    side_database_path: *const c_char,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_regions_merge_database_start(handle, side_database_path, out_operation_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_update_metadata_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    metadata: *const u8,
    metadata_size: usize,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_region_update_metadata_start(
            handle,
            region_id,
            metadata,
            metadata_size,
            out_operation_id,
        ),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_get_status_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_region_get_status_start(handle, region_id, out_operation_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_set_observed_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    observed: GBoolean,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_region_set_observed_start(handle, region_id, observed != GFALSE, out_operation_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_set_download_state_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    state: u32,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_region_set_download_state_start(handle, region_id, state, out_operation_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_invalidate_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_region_invalidate_start(handle, region_id, out_operation_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_delete_start(
    handle: *mut RuntimeHandle,
    region_id: i64,
    out_operation_id: *mut u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_region_delete_start(handle, region_id, out_operation_id),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_create_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionInfo {
    match offline_region_create_take_result(handle, operation_id) {
        Ok(info) => Box::into_raw(Box::new(info)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_get_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    out_found: *mut GBoolean,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionInfo {
    match offline_region_get_take_result(handle, operation_id, out_found) {
        Ok(info) => info.map_or(ptr::null_mut(), |info| Box::into_raw(Box::new(info))),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_regions_list_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionInfoList {
    match offline_regions_list_take_result(handle, operation_id) {
        Ok(list) => Box::into_raw(Box::new(list)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_regions_merge_database_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionInfoList {
    match offline_regions_merge_database_take_result(handle, operation_id) {
        Ok(list) => Box::into_raw(Box::new(list)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_update_metadata_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    error_out: *mut *mut GError,
) -> *mut OfflineRegionInfo {
    match offline_region_update_metadata_take_result(handle, operation_id) {
        Ok(info) => Box::into_raw(Box::new(info)),
        Err(error) => {
            glib::set_error(error_out, error);
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_region_get_status_take_result(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    out_status: *mut sys::mln_offline_region_status,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(
        offline_region_get_status_take_result(handle, operation_id, out_status),
        error_out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn mln_vala_runtime_handle_offline_operation_discard(
    handle: *mut RuntimeHandle,
    operation_id: u64,
    error_out: *mut *mut GError,
) -> GBoolean {
    gerror_bool(offline_operation_discard(handle, operation_id), error_out)
}
