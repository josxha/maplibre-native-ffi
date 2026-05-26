/* Private raw VAPI for the MapLibre Native C API. */

[CCode (cprefix = "mln_", lower_case_cprefix = "mln_", cheader_filename = "maplibre_native_c/base.h,maplibre_native_c/diagnostics.h,maplibre_native_c/logging.h,maplibre_native_c/runtime.h,maplibre_native_c/map.h,maplibre_native_c/camera.h,maplibre_native_c/style.h,maplibre_native_c/query.h,maplibre_native_c/render_target.h,maplibre_native_c/texture.h,maplibre_native_c/render_session.h,maplibre_native_c/projection.h")]
namespace MaplibreNative.Raw {
    [CCode (cname = "mln_status", cprefix = "MLN_STATUS_", has_type_id = false)]
    public enum Status {
        OK,
        INVALID_ARGUMENT,
        INVALID_STATE,
        WRONG_THREAD,
        UNSUPPORTED,
        NATIVE_ERROR
    }

    [CCode (cname = "mln_render_backend_flag", cprefix = "MLN_RENDER_BACKEND_FLAG_", has_type_id = false)]
    [Flags]
    public enum RenderBackendFlag {
        METAL,
        VULKAN
    }

    [CCode (cname = "mln_network_status", cprefix = "MLN_NETWORK_STATUS_", has_type_id = false)]
    public enum NetworkStatus {
        ONLINE,
        OFFLINE
    }

    [CCode (cname = "mln_map_mode", cprefix = "MLN_MAP_MODE_", has_type_id = false)]
    public enum MapMode {
        CONTINUOUS,
        STATIC,
        TILE
    }

    [CCode (cname = "mln_ambient_cache_operation", cprefix = "MLN_AMBIENT_CACHE_OPERATION_", has_type_id = false)]
    public enum AmbientCacheOperation {
        RESET_DATABASE,
        PACK_DATABASE,
        INVALIDATE,
        CLEAR
    }

    [CCode (cname = "mln_runtime_event_type", cprefix = "MLN_RUNTIME_EVENT_", has_type_id = false)]
    public enum RuntimeEventType {
        MAP_CAMERA_WILL_CHANGE,
        MAP_CAMERA_IS_CHANGING,
        MAP_CAMERA_DID_CHANGE,
        MAP_STYLE_LOADED,
        MAP_LOADING_STARTED,
        MAP_LOADING_FINISHED,
        MAP_LOADING_FAILED,
        MAP_IDLE,
        MAP_RENDER_UPDATE_AVAILABLE,
        MAP_RENDER_ERROR,
        MAP_STILL_IMAGE_FINISHED,
        MAP_STILL_IMAGE_FAILED,
        MAP_RENDER_FRAME_STARTED,
        MAP_RENDER_FRAME_FINISHED,
        MAP_RENDER_MAP_STARTED,
        MAP_RENDER_MAP_FINISHED,
        MAP_STYLE_IMAGE_MISSING,
        MAP_TILE_ACTION,
        OFFLINE_REGION_STATUS_CHANGED,
        OFFLINE_REGION_RESPONSE_ERROR,
        OFFLINE_REGION_TILE_COUNT_LIMIT_EXCEEDED,
        OFFLINE_OPERATION_COMPLETED
    }

    [CCode (cname = "mln_runtime_event_source_type", cprefix = "MLN_RUNTIME_EVENT_SOURCE_", has_type_id = false)]
    public enum RuntimeEventSourceType {
        RUNTIME,
        MAP
    }

    [CCode (cname = "mln_log_severity", cprefix = "MLN_LOG_SEVERITY_", has_type_id = false)]
    public enum LogSeverity {
        INFO,
        WARNING,
        ERROR
    }

    [CCode (cname = "mln_resource_kind", cprefix = "MLN_RESOURCE_KIND_", has_type_id = false)]
    public enum ResourceKind {
        UNKNOWN,
        STYLE,
        SOURCE,
        TILE,
        GLYPHS,
        SPRITE_IMAGE,
        SPRITE_JSON,
        IMAGE
    }

    [CCode (cname = "mln_rendered_query_geometry_type", cprefix = "MLN_RENDERED_QUERY_GEOMETRY_TYPE_", has_type_id = false)]
    public enum RenderedQueryGeometryType {
        POINT,
        BOX,
        LINE_STRING
    }

    [CCode (cname = "mln_style_source_type", cprefix = "MLN_STYLE_SOURCE_TYPE_", has_type_id = false)]
    public enum StyleSourceType {
        UNKNOWN,
        VECTOR,
        RASTER,
        RASTER_DEM,
        GEOJSON,
        IMAGE,
        VIDEO,
        ANNOTATIONS,
        CUSTOM_VECTOR
    }

    [CCode (cname = "mln_log_event", cprefix = "MLN_LOG_EVENT_", has_type_id = false)]
    public enum LogEvent {
        GENERAL,
        SETUP,
        SHADER,
        PARSE_STYLE,
        PARSE_TILE,
        RENDER,
        STYLE,
        DATABASE,
        HTTP_REQUEST,
        SPRITE,
        IMAGE,
        OPENGL,
        JNI,
        ANDROID,
        CRASH,
        GLYPH,
        TIMING
    }

    [Compact]
    [CCode (cname = "mln_runtime", free_function = "")]
    public class Runtime {}

    [Compact]
    [CCode (cname = "mln_map", free_function = "")]
    public class Map {}

    [Compact]
    [CCode (cname = "mln_render_session", free_function = "")]
    public class RenderSession {}

    [Compact]
    [CCode (cname = "mln_map_projection", free_function = "")]
    public class MapProjection {}

    [Compact]
    [CCode (cname = "mln_style_id_list", free_function = "")]
    public class StyleIdList {}

    [Compact]
    [CCode (cname = "mln_feature_query_result", free_function = "")]
    public class FeatureQueryResult {}

    [CCode (cname = "mln_runtime_options", has_type_id = false)]
    public struct RuntimeOptions {
        public uint32 size;
        public uint32 flags;
        public unowned string? asset_path;
        public unowned string? cache_path;
        public uint64 maximum_cache_size;
    }

    [CCode (cname = "mln_map_options", has_type_id = false)]
    public struct MapOptions {
        public uint32 size;
        public uint32 width;
        public uint32 height;
        public double scale_factor;
        public uint32 map_mode;
    }

    [SimpleType]
    [CCode (cname = "mln_string_view", has_type_id = false)]
    public struct StringView {
        public char* data;
        public size_t size;
    }

    [SimpleType]
    [CCode (cname = "mln_lat_lng", has_type_id = false)]
    public struct LatLng {
        public double latitude;
        public double longitude;
    }

    [SimpleType]
    [CCode (cname = "mln_screen_point", has_type_id = false)]
    public struct ScreenPoint {
        public double x;
        public double y;
    }

    [SimpleType]
    [CCode (cname = "mln_edge_insets", has_type_id = false)]
    public struct EdgeInsets {
        public double top;
        public double left;
        public double bottom;
        public double right;
    }

    [SimpleType]
    [CCode (cname = "mln_projected_meters", has_type_id = false)]
    public struct ProjectedMeters {
        public double northing;
        public double easting;
    }

    [SimpleType]
    [CCode (cname = "mln_screen_box", has_type_id = false)]
    public struct ScreenBox {
        public ScreenPoint min;
        public ScreenPoint max;
    }

    [SimpleType]
    [CCode (cname = "mln_rendered_query_geometry", has_type_id = false)]
    public struct RenderedQueryGeometry {
        public uint32 size;
        public uint32 type;
        public ScreenBox data;
    }

    [CCode (cname = "mln_camera_options", has_type_id = false)]
    public struct CameraOptions {
        public uint32 size;
        public uint32 fields;
        public double latitude;
        public double longitude;
        public double center_altitude;
        public EdgeInsets padding;
        public ScreenPoint anchor;
        public double zoom;
        public double bearing;
        public double pitch;
        public double roll;
        public double field_of_view;
    }

    [CCode (cname = "mln_render_target_extent", has_type_id = false)]
    public struct RenderTargetExtent {
        public uint32 size;
        public uint32 width;
        public uint32 height;
        public double scale_factor;
    }

    [CCode (cname = "mln_metal_context_descriptor", has_type_id = false)]
    public struct MetalContextDescriptor {
        public uint32 size;
        public void* device;
    }

    [CCode (cname = "mln_metal_owned_texture_descriptor", has_type_id = false)]
    public struct MetalOwnedTextureDescriptor {
        public uint32 size;
        public RenderTargetExtent extent;
        public MetalContextDescriptor context;
    }

    [CCode (cname = "mln_texture_image_info", has_type_id = false)]
    public struct TextureImageInfo {
        public uint32 size;
        public uint32 width;
        public uint32 height;
        public uint32 stride;
        public size_t byte_length;
    }

    [CCode (cname = "mln_premultiplied_rgba8_image", has_type_id = false)]
    public struct PremultipliedRgba8Image {
        public uint32 size;
        public uint32 width;
        public uint32 height;
        public uint32 stride;
        public uint8* pixels;
        public size_t byte_length;
    }

    [CCode (cname = "mln_style_source_info", has_type_id = false)]
    public struct StyleSourceInfo {
        public uint32 size;
        public uint32 type;
        public size_t id_size;
        public bool is_volatile;
        public bool has_attribution;
        public size_t attribution_size;
    }

    [CCode (cname = "mln_style_image_options", has_type_id = false)]
    public struct StyleImageOptions {
        public uint32 size;
        public uint32 fields;
        public float pixel_ratio;
        public bool sdf;
    }

    [CCode (cname = "mln_style_image_info", has_type_id = false)]
    public struct StyleImageInfo {
        public uint32 size;
        public uint32 width;
        public uint32 height;
        public uint32 stride;
        public size_t byte_length;
        public float pixel_ratio;
        public bool sdf;
    }

    [CCode (cname = "mln_metal_owned_texture_frame", has_type_id = false)]
    public struct MetalOwnedTextureFrame {
        public uint32 size;
        public uint64 generation;
        public uint32 width;
        public uint32 height;
        public double scale_factor;
        public uint64 frame_id;
        public void* texture;
        public void* device;
        public uint64 pixel_format;
    }

    [CCode (cname = "mln_resource_transform_response", has_type_id = false)]
    public struct ResourceTransformResponse {
        public uint32 size;
        public unowned string? url;
    }

    [CCode (cname = "mln_resource_transform_callback", has_target = false)]
    public delegate Status ResourceTransformCallback (void* user_data, uint32 kind, string url, ResourceTransformResponse* out_response);

    [CCode (cname = "mln_resource_transform", has_type_id = false)]
    public struct ResourceTransform {
        public uint32 size;
        public ResourceTransformCallback callback;
        public void* user_data;
    }

    [CCode (cname = "mln_runtime_event", has_type_id = false)]
    public struct RuntimeEvent {
        public uint32 size;
        public uint32 type;
        public uint32 source_type;
        public void* source;
        public int32 code;
        public uint32 payload_type;
        public void* payload;
        public size_t payload_size;
        public char* message;
        public size_t message_size;
    }

    [CCode (cname = "mln_log_callback", has_target = false)]
    public delegate uint32 LogCallback (void* user_data, uint32 severity, uint32 event, int64 code, string? message);

    [CCode (cname = "mln_c_version")]
    public static uint32 c_version ();

    [CCode (cname = "mln_supported_render_backend_mask")]
    public static uint32 supported_render_backend_mask ();

    [CCode (cname = "mln_thread_last_error_message")]
    public static unowned string thread_last_error_message ();

    [CCode (cname = "mln_network_status_get")]
    public static Status network_status_get (out uint32 out_status);

    [CCode (cname = "mln_network_status_set")]
    public static Status network_status_set (uint32 status);

    [CCode (cname = "mln_log_set_callback")]
    public static Status log_set_callback (LogCallback? callback, void* user_data);

    [CCode (cname = "mln_log_clear_callback")]
    public static Status log_clear_callback ();

    [CCode (cname = "mln_runtime_options_default")]
    public static RuntimeOptions runtime_options_default ();

    [CCode (cname = "mln_runtime_create")]
    public static Status runtime_create (RuntimeOptions* options, out Runtime runtime);

    [CCode (cname = "mln_runtime_destroy")]
    public static Status runtime_destroy (Runtime runtime);

    [CCode (cname = "mln_runtime_run_once")]
    public static Status runtime_run_once (Runtime runtime);

    [CCode (cname = "mln_runtime_poll_event")]
    public static Status runtime_poll_event (Runtime runtime, RuntimeEvent* out_event, out bool out_has_event);

    [CCode (cname = "mln_runtime_set_resource_transform")]
    public static Status runtime_set_resource_transform (Runtime runtime, ResourceTransform* transform);

    [CCode (cname = "mln_runtime_clear_resource_transform")]
    public static Status runtime_clear_resource_transform (Runtime runtime);

    [CCode (cname = "mln_runtime_run_ambient_cache_operation_start")]
    public static Status runtime_run_ambient_cache_operation_start (Runtime runtime, uint32 operation, out uint64 out_operation_id);

    [CCode (cname = "mln_runtime_offline_operation_discard")]
    public static Status runtime_offline_operation_discard (Runtime runtime, uint64 operation_id);

    [CCode (cname = "mln_map_options_default")]
    public static MapOptions map_options_default ();

    [CCode (cname = "mln_map_create")]
    public static Status map_create (Runtime runtime, MapOptions* options, out Map map);

    [CCode (cname = "mln_map_destroy")]
    public static Status map_destroy (Map map);

    [CCode (cname = "mln_map_request_repaint")]
    public static Status map_request_repaint (Map map);

    [CCode (cname = "mln_map_request_still_image")]
    public static Status map_request_still_image (Map map);

    [CCode (cname = "mln_map_set_style_url")]
    public static Status map_set_style_url (Map map, string url);

    [CCode (cname = "mln_map_set_style_json")]
    public static Status map_set_style_json (Map map, string json);

    [CCode (cname = "mln_map_set_debug_options")]
    public static Status map_set_debug_options (Map map, uint32 options);

    [CCode (cname = "mln_map_get_debug_options")]
    public static Status map_get_debug_options (Map map, out uint32 out_options);

    [CCode (cname = "mln_map_set_rendering_stats_view_enabled")]
    public static Status map_set_rendering_stats_view_enabled (Map map, bool enabled);

    [CCode (cname = "mln_map_get_rendering_stats_view_enabled")]
    public static Status map_get_rendering_stats_view_enabled (Map map, out bool out_enabled);

    [CCode (cname = "mln_map_is_fully_loaded")]
    public static Status map_is_fully_loaded (Map map, out bool out_loaded);

    [CCode (cname = "mln_map_dump_debug_logs")]
    public static Status map_dump_debug_logs (Map map);

    [CCode (cname = "mln_camera_options_default")]
    public static CameraOptions camera_options_default ();

    [CCode (cname = "mln_map_get_camera")]
    public static Status map_get_camera (Map map, CameraOptions* out_camera);

    [CCode (cname = "mln_map_jump_to")]
    public static Status map_jump_to (Map map, CameraOptions* camera);

    [CCode (cname = "mln_style_id_list_count")]
    public static Status style_id_list_count (StyleIdList list, out size_t out_count);

    [CCode (cname = "mln_style_id_list_get")]
    public static Status style_id_list_get (StyleIdList list, size_t index, out StringView out_id);

    [CCode (cname = "mln_style_id_list_destroy")]
    public static void style_id_list_destroy (StyleIdList list);

    [CCode (cname = "mln_map_add_geojson_source_url")]
    public static Status map_add_geojson_source_url (Map map, StringView source_id, StringView url);

    [CCode (cname = "mln_map_set_geojson_source_url")]
    public static Status map_set_geojson_source_url (Map map, StringView source_id, StringView url);

    [CCode (cname = "mln_map_remove_style_source")]
    public static Status map_remove_style_source (Map map, StringView source_id, out bool out_removed);

    [CCode (cname = "mln_map_style_source_exists")]
    public static Status map_style_source_exists (Map map, StringView source_id, out bool out_exists);

    [CCode (cname = "mln_map_get_style_source_type")]
    public static Status map_get_style_source_type (Map map, StringView source_id, out uint32 out_source_type, out bool out_found);

    [CCode (cname = "mln_map_get_style_source_info")]
    public static Status map_get_style_source_info (Map map, StringView source_id, StyleSourceInfo* out_info, out bool out_found);

    [CCode (cname = "mln_map_list_style_source_ids")]
    public static Status map_list_style_source_ids (Map map, out StyleIdList out_source_ids);

    [CCode (cname = "mln_map_add_location_indicator_layer")]
    public static Status map_add_location_indicator_layer (Map map, StringView layer_id, StringView before_layer_id);

    [CCode (cname = "mln_map_remove_style_layer")]
    public static Status map_remove_style_layer (Map map, StringView layer_id, out bool out_removed);

    [CCode (cname = "mln_map_style_layer_exists")]
    public static Status map_style_layer_exists (Map map, StringView layer_id, out bool out_exists);

    [CCode (cname = "mln_map_list_style_layer_ids")]
    public static Status map_list_style_layer_ids (Map map, out StyleIdList out_layer_ids);

    [CCode (cname = "mln_map_set_style_image")]
    public static Status map_set_style_image (Map map, StringView image_id, PremultipliedRgba8Image* image, StyleImageOptions* options);

    [CCode (cname = "mln_map_remove_style_image")]
    public static Status map_remove_style_image (Map map, StringView image_id, out bool out_removed);

    [CCode (cname = "mln_map_style_image_exists")]
    public static Status map_style_image_exists (Map map, StringView image_id, out bool out_exists);

    [CCode (cname = "mln_map_get_style_image_info")]
    public static Status map_get_style_image_info (Map map, StringView image_id, StyleImageInfo* out_info, out bool out_found);

    [CCode (cname = "mln_map_copy_style_image_premultiplied_rgba8")]
    public static Status map_copy_style_image_premultiplied_rgba8 (Map map, StringView image_id, uint8* out_pixels, size_t pixel_capacity, out size_t out_byte_length, out bool out_found);

    [CCode (cname = "mln_rendered_query_geometry_point")]
    public static RenderedQueryGeometry rendered_query_geometry_point (ScreenPoint point);

    [CCode (cname = "mln_render_session_query_rendered_features")]
    public static Status render_session_query_rendered_features (RenderSession session, RenderedQueryGeometry* geometry, void* options, out FeatureQueryResult out_result);

    [CCode (cname = "mln_feature_query_result_count")]
    public static Status feature_query_result_count (FeatureQueryResult result, out size_t out_count);

    [CCode (cname = "mln_feature_query_result_destroy")]
    public static void feature_query_result_destroy (FeatureQueryResult result);

    [CCode (cname = "mln_metal_owned_texture_attach")]
    public static Status metal_owned_texture_attach (Map map, MetalOwnedTextureDescriptor* descriptor, out RenderSession session);

    [CCode (cname = "mln_render_session_render_update")]
    public static Status render_session_render_update (RenderSession session);

    [CCode (cname = "mln_render_session_destroy")]
    public static Status render_session_destroy (RenderSession session);

    [CCode (cname = "mln_texture_read_premultiplied_rgba8")]
    public static Status texture_read_premultiplied_rgba8 (RenderSession session, uint8* out_data, size_t out_data_capacity, TextureImageInfo* out_info);

    [CCode (cname = "mln_metal_owned_texture_acquire_frame")]
    public static Status metal_owned_texture_acquire_frame (RenderSession session, MetalOwnedTextureFrame* out_frame);

    [CCode (cname = "mln_metal_owned_texture_release_frame")]
    public static Status metal_owned_texture_release_frame (RenderSession session, MetalOwnedTextureFrame* frame);

    [CCode (cname = "mln_map_projection_create")]
    public static Status map_projection_create (Map map, out MapProjection projection);

    [CCode (cname = "mln_map_projection_destroy")]
    public static Status map_projection_destroy (MapProjection projection);

    [CCode (cname = "mln_map_projection_get_camera")]
    public static Status map_projection_get_camera (MapProjection projection, CameraOptions* out_camera);

    [CCode (cname = "mln_map_projection_set_camera")]
    public static Status map_projection_set_camera (MapProjection projection, CameraOptions* camera);

    [CCode (cname = "mln_map_projection_pixel_for_lat_lng")]
    public static Status map_projection_pixel_for_lat_lng (MapProjection projection, LatLng coordinate, out ScreenPoint out_point);

    [CCode (cname = "mln_map_projection_lat_lng_for_pixel")]
    public static Status map_projection_lat_lng_for_pixel (MapProjection projection, ScreenPoint point, out LatLng out_coordinate);

    [CCode (cname = "mln_map_projection_set_visible_coordinates")]
    public static Status map_projection_set_visible_coordinates (MapProjection projection, LatLng* coordinates, size_t coordinate_count, EdgeInsets padding);

    [CCode (cname = "mln_projected_meters_for_lat_lng")]
    public static Status projected_meters_for_lat_lng (LatLng coordinate, out ProjectedMeters out_meters);

    [CCode (cname = "mln_lat_lng_for_projected_meters")]
    public static Status lat_lng_for_projected_meters (ProjectedMeters meters, out LatLng out_coordinate);
}
