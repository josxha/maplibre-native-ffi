namespace MaplibreNative {
    internal void check_status (Raw.Status status) throws Error {
        if (status == Raw.Status.OK) {
            return;
        }

        var message = Raw.thread_last_error_message ();
        if (message == null || message.length == 0) {
            message = "MapLibre Native operation failed";
        }

        switch (status) {
            case Raw.Status.INVALID_ARGUMENT:
                throw new Error.INVALID_ARGUMENT ("%s", message);
            case Raw.Status.INVALID_STATE:
                throw new Error.INVALID_STATE ("%s", message);
            case Raw.Status.WRONG_THREAD:
                throw new Error.WRONG_THREAD ("%s", message);
            case Raw.Status.UNSUPPORTED:
                throw new Error.UNSUPPORTED ("%s", message);
            case Raw.Status.NATIVE_ERROR:
                throw new Error.NATIVE_ERROR ("%s", message);
            default:
                throw new Error.UNKNOWN_STATUS ("%s", message);
        }
    }

    internal string copy_c_string (char* value) {
        if (value == null) {
            return "";
        }
        return (string) value;
    }

    internal string copy_c_string_bytes (char* value, size_t size) {
        if (value == null || size == 0) {
            return "";
        }
        return ((string) value).substring (0, (long) size);
    }

    internal Raw.StringView string_view (string value) throws Error {
        return Raw.StringView () { data = (char*) value, size = value.length };
    }

    internal string copy_string_view (Raw.StringView view) throws Error {
        if (view.data == null && view.size == 0) {
            return "";
        }
        if (view.data == null) {
            throw new Error.INVALID_ARGUMENT ("string view data is null");
        }
        return ((string) view.data).substring (0, (long) view.size);
    }

    internal uint8[]? copy_bytes (uint8* data, size_t size) {
        if (data == null || size == 0) {
            return null;
        }
        uint8[] copied = new uint8[size];
        for (size_t index = 0; index < size; index++) {
            copied[index] = data[index];
        }
        return copied;
    }

    internal StringList copy_style_id_list (owned Raw.StyleIdList list) throws Error {
        try {
            size_t count;
            check_status (Raw.style_id_list_count (list, out count));
            string[] values = new string[count];
            for (size_t index = 0; index < count; index++) {
                Raw.StringView item;
                check_status (Raw.style_id_list_get (list, index, out item));
                values[index] = copy_string_view (item);
            }
            return new StringList ((owned) values);
        } finally {
            Raw.style_id_list_destroy (list);
        }
    }

    internal JsonValue? copy_json_snapshot (owned Raw.JsonSnapshot? snapshot) throws Error {
        if (snapshot == null) {
            return null;
        }
        try {
            Raw.JsonValue* value;
            check_status (Raw.json_snapshot_get (snapshot, out value));
            if (value == null) {
                return null;
            }
            return JsonValue.from_native (value[0]);
        } finally {
            Raw.json_snapshot_destroy (snapshot);
        }
    }

    internal NetworkStatus network_status_from_raw (uint32 raw_status) {
        switch (raw_status) {
            case 1:
                return NetworkStatus.ONLINE;
            case 2:
                return NetworkStatus.OFFLINE;
            default:
                return NetworkStatus.UNKNOWN;
        }
    }

    internal RuntimeEventType runtime_event_type_from_raw (uint32 raw_type) {
        if (raw_type >= 1 && raw_type <= 22) {
            return (RuntimeEventType) raw_type;
        }
        return RuntimeEventType.UNKNOWN;
    }

    internal RuntimeEventSourceType runtime_event_source_type_from_raw (uint32 raw_type) {
        if (raw_type <= 1) {
            return (RuntimeEventSourceType) raw_type;
        }
        return RuntimeEventSourceType.UNKNOWN;
    }

    internal LogSeverity log_severity_from_raw (uint32 raw_severity) {
        if (raw_severity >= 1 && raw_severity <= 3) {
            return (LogSeverity) raw_severity;
        }
        return LogSeverity.UNKNOWN;
    }

    internal StyleSourceType style_source_type_from_raw (uint32 raw_type) {
        if (raw_type <= 8) {
            return (StyleSourceType) raw_type;
        }
        return StyleSourceType.UNKNOWN;
    }

    internal ResourceKind resource_kind_from_raw (uint32 raw_kind) {
        if (raw_kind <= 7) {
            return (ResourceKind) raw_kind;
        }
        return ResourceKind.UNKNOWN;
    }

    internal ResourceLoadingMethod resource_loading_method_from_raw (uint32 raw_method) {
        if (raw_method <= 2) {
            return (ResourceLoadingMethod) raw_method;
        }
        return ResourceLoadingMethod.UNKNOWN;
    }

    internal ResourcePriority resource_priority_from_raw (uint32 raw_priority) {
        if (raw_priority <= 1) {
            return (ResourcePriority) raw_priority;
        }
        return ResourcePriority.UNKNOWN;
    }

    internal ResourceUsage resource_usage_from_raw (uint32 raw_usage) {
        if (raw_usage <= 1) {
            return (ResourceUsage) raw_usage;
        }
        return ResourceUsage.UNKNOWN;
    }

    internal ResourceStoragePolicy resource_storage_policy_from_raw (uint32 raw_policy) {
        if (raw_policy <= 1) {
            return (ResourceStoragePolicy) raw_policy;
        }
        return ResourceStoragePolicy.UNKNOWN;
    }

    internal ResourceErrorReason resource_error_reason_from_raw (uint32 raw_reason) {
        if (raw_reason <= 5) {
            return (ResourceErrorReason) raw_reason;
        }
        return ResourceErrorReason.UNKNOWN;
    }

    internal RuntimeEventPayloadType runtime_event_payload_type_from_raw (uint32 raw_type) {
        if (raw_type <= 8) {
            return (RuntimeEventPayloadType) raw_type;
        }
        return RuntimeEventPayloadType.UNKNOWN;
    }

    internal RenderMode render_mode_from_raw (uint32 raw_mode) {
        if (raw_mode <= 1) {
            return (RenderMode) raw_mode;
        }
        return RenderMode.UNKNOWN;
    }

    internal TileOperation tile_operation_from_raw (uint32 raw_operation) {
        if (raw_operation <= 8) {
            return (TileOperation) raw_operation;
        }
        return TileOperation.UNKNOWN;
    }

    internal OfflineRegionDownloadState offline_region_download_state_from_raw (uint32 raw_state) {
        if (raw_state <= 1) {
            return (OfflineRegionDownloadState) raw_state;
        }
        return OfflineRegionDownloadState.UNKNOWN;
    }

    internal OfflineOperationKind offline_operation_kind_from_raw (uint32 raw_kind) {
        if (raw_kind >= 1 && raw_kind <= 11) {
            return (OfflineOperationKind) raw_kind;
        }
        return OfflineOperationKind.UNKNOWN;
    }

    internal OfflineOperationResultKind offline_operation_result_kind_from_raw (uint32 raw_kind) {
        if (raw_kind <= 4) {
            return (OfflineOperationResultKind) raw_kind;
        }
        return OfflineOperationResultKind.UNKNOWN;
    }

    internal LogEvent log_event_from_raw (uint32 raw_event) {
        if (raw_event <= 16) {
            return (LogEvent) raw_event;
        }
        return LogEvent.UNKNOWN;
    }
}
