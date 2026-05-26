namespace MaplibreNative {
    public errordomain Error {
        INVALID_ARGUMENT,
        INVALID_STATE,
        WRONG_THREAD,
        UNSUPPORTED,
        NATIVE_ERROR,
        UNKNOWN_STATUS
    }

    [Flags]
    public enum RenderBackendFlags {
        METAL = 1 << 0,
        VULKAN = 1 << 1
    }

    public enum NetworkStatus {
        ONLINE = 1,
        OFFLINE = 2,
        UNKNOWN = 0
    }

    public enum MapMode {
        CONTINUOUS = 0,
        STATIC = 1,
        TILE = 2
    }

    public enum AmbientCacheOperation {
        RESET_DATABASE = 1,
        PACK_DATABASE = 2,
        INVALIDATE = 3,
        CLEAR = 4
    }

    public enum RuntimeEventType {
        MAP_CAMERA_WILL_CHANGE = 1,
        MAP_CAMERA_IS_CHANGING = 2,
        MAP_CAMERA_DID_CHANGE = 3,
        MAP_STYLE_LOADED = 4,
        MAP_LOADING_STARTED = 5,
        MAP_LOADING_FINISHED = 6,
        MAP_LOADING_FAILED = 7,
        MAP_IDLE = 8,
        MAP_RENDER_UPDATE_AVAILABLE = 9,
        MAP_RENDER_ERROR = 10,
        MAP_STILL_IMAGE_FINISHED = 11,
        MAP_STILL_IMAGE_FAILED = 12,
        MAP_RENDER_FRAME_STARTED = 13,
        MAP_RENDER_FRAME_FINISHED = 14,
        MAP_RENDER_MAP_STARTED = 15,
        MAP_RENDER_MAP_FINISHED = 16,
        MAP_STYLE_IMAGE_MISSING = 17,
        MAP_TILE_ACTION = 18,
        OFFLINE_REGION_STATUS_CHANGED = 19,
        OFFLINE_REGION_RESPONSE_ERROR = 20,
        OFFLINE_REGION_TILE_COUNT_LIMIT_EXCEEDED = 21,
        OFFLINE_OPERATION_COMPLETED = 22,
        UNKNOWN = 0
    }

    public enum RuntimeEventSourceType {
        RUNTIME = 0,
        MAP = 1,
        UNKNOWN = 2
    }

    public enum LogSeverity {
        INFO = 1,
        WARNING = 2,
        ERROR = 3,
        UNKNOWN = 0
    }

    public enum ResourceKind {
        UNKNOWN = 0,
        STYLE = 1,
        SOURCE = 2,
        TILE = 3,
        GLYPHS = 4,
        SPRITE_IMAGE = 5,
        SPRITE_JSON = 6,
        IMAGE = 7
    }

    public enum StyleSourceType {
        UNKNOWN = 0,
        VECTOR = 1,
        RASTER = 2,
        RASTER_DEM = 3,
        GEOJSON = 4,
        IMAGE = 5,
        VIDEO = 6,
        ANNOTATIONS = 7,
        CUSTOM_VECTOR = 8
    }

    public enum LogEvent {
        GENERAL = 0,
        SETUP = 1,
        SHADER = 2,
        PARSE_STYLE = 3,
        PARSE_TILE = 4,
        RENDER = 5,
        STYLE = 6,
        DATABASE = 7,
        HTTP_REQUEST = 8,
        SPRITE = 9,
        IMAGE = 10,
        OPENGL = 11,
        JNI = 12,
        ANDROID = 13,
        CRASH = 14,
        GLYPH = 15,
        TIMING = 16,
        UNKNOWN = 255
    }

    public struct LatLng {
        public double latitude;
        public double longitude;

        public LatLng (double latitude, double longitude) {
            this.latitude = latitude;
            this.longitude = longitude;
        }

        internal Raw.LatLng to_native () {
            return Raw.LatLng () { latitude = latitude, longitude = longitude };
        }

        internal static LatLng from_native (Raw.LatLng native) {
            return LatLng (native.latitude, native.longitude);
        }
    }

    public struct ScreenPoint {
        public double x;
        public double y;

        public ScreenPoint (double x, double y) {
            this.x = x;
            this.y = y;
        }

        internal Raw.ScreenPoint to_native () {
            return Raw.ScreenPoint () { x = x, y = y };
        }

        internal static ScreenPoint from_native (Raw.ScreenPoint native) {
            return ScreenPoint (native.x, native.y);
        }
    }

    public struct EdgeInsets {
        public double top;
        public double left;
        public double bottom;
        public double right;

        public EdgeInsets (double top, double left, double bottom, double right) {
            this.top = top;
            this.left = left;
            this.bottom = bottom;
            this.right = right;
        }

        internal Raw.EdgeInsets to_native () {
            return Raw.EdgeInsets () { top = top, left = left, bottom = bottom, right = right };
        }
    }

    public struct ProjectedMeters {
        public double northing;
        public double easting;

        public ProjectedMeters (double northing, double easting) {
            this.northing = northing;
            this.easting = easting;
        }

        internal Raw.ProjectedMeters to_native () {
            return Raw.ProjectedMeters () { northing = northing, easting = easting };
        }

        internal static ProjectedMeters from_native (Raw.ProjectedMeters native) {
            return ProjectedMeters (native.northing, native.easting);
        }
    }

    public struct NativePointer {
        public size_t bits;

        public NativePointer (size_t bits) {
            this.bits = bits;
        }

        public bool is_null () {
            return bits == 0;
        }

        internal void* to_native () throws Error {
            if (bits == 0) {
                throw new Error.INVALID_ARGUMENT ("native pointer is null");
            }
            return (void*) bits;
        }
    }

    public class StringList {
        private string[] values;

        internal StringList (owned string[] values) {
            this.values = (owned) values;
        }

        public uint length {
            get { return values.length; }
        }

        public string get (uint index) throws Error {
            if (index >= values.length) {
                throw new Error.INVALID_ARGUMENT ("string list index is out of range");
            }
            return values[index];
        }

        public string[] to_array () {
            return values;
        }

        public bool contains (string value) {
            foreach (var item in values) {
                if (item == value) {
                    return true;
                }
            }
            return false;
        }
    }

    public class TextureImageInfo {
        public uint32 width { get; private set; }
        public uint32 height { get; private set; }
        public uint32 stride { get; private set; }
        public size_t byte_length { get; private set; }

        internal TextureImageInfo (Raw.TextureImageInfo native) {
            width = native.width;
            height = native.height;
            stride = native.stride;
            byte_length = native.byte_length;
        }
    }

    public class PremultipliedRgba8Image {
        public uint32 width { get; private set; }
        public uint32 height { get; private set; }
        public uint32 stride { get; private set; }
        private uint8[] pixels;

        public PremultipliedRgba8Image (uint32 width, uint32 height, uint32 stride, owned uint8[] pixels) {
            this.width = width;
            this.height = height;
            this.stride = stride;
            this.pixels = (owned) pixels;
        }

        internal Raw.PremultipliedRgba8Image to_native () {
            Raw.PremultipliedRgba8Image image = {};
            image.size = (uint32) sizeof (Raw.PremultipliedRgba8Image);
            image.width = width;
            image.height = height;
            image.stride = stride;
            image.pixels = pixels;
            image.byte_length = pixels.length;
            return image;
        }

        public uint8[] copy_pixels () {
            uint8[] copied = new uint8[pixels.length];
            for (int index = 0; index < pixels.length; index++) {
                copied[index] = pixels[index];
            }
            return copied;
        }
    }

    public class StyleImageOptions {
        public float? pixel_ratio { get; set; }
        public bool? sdf { get; set; }

        internal Raw.StyleImageOptions to_native () {
            Raw.StyleImageOptions options = {};
            options.size = (uint32) sizeof (Raw.StyleImageOptions);
            if (pixel_ratio != null) {
                options.pixel_ratio = pixel_ratio;
                options.fields |= 1U << 0;
            }
            if (sdf != null) {
                options.sdf = sdf;
                options.fields |= 1U << 1;
            }
            return options;
        }
    }

    public class StyleSourceInfo {
        public StyleSourceType source_type { get; private set; }
        public size_t id_size { get; private set; }
        public bool is_volatile { get; private set; }
        public bool has_attribution { get; private set; }
        public size_t attribution_size { get; private set; }

        internal StyleSourceInfo (Raw.StyleSourceInfo native) {
            source_type = style_source_type_from_raw (native.type);
            id_size = native.id_size;
            is_volatile = native.is_volatile;
            has_attribution = native.has_attribution;
            attribution_size = native.attribution_size;
        }
    }

    public class StyleImageInfo {
        public uint32 width { get; private set; }
        public uint32 height { get; private set; }
        public uint32 stride { get; private set; }
        public size_t byte_length { get; private set; }
        public float pixel_ratio { get; private set; }
        public bool sdf { get; private set; }

        internal StyleImageInfo (Raw.StyleImageInfo native) {
            width = native.width;
            height = native.height;
            stride = native.stride;
            byte_length = native.byte_length;
            pixel_ratio = native.pixel_ratio;
            sdf = native.sdf;
        }
    }

    public class MetalOwnedTextureDescriptor {
        public uint32 width { get; set; default = 64; }
        public uint32 height { get; set; default = 64; }
        public double scale_factor { get; set; default = 1.0; }
        public NativePointer device { get; set; }

        public MetalOwnedTextureDescriptor (NativePointer device) {
            this.device = device;
        }

        internal Raw.MetalOwnedTextureDescriptor to_native () throws Error {
            Raw.MetalOwnedTextureDescriptor descriptor = {};
            descriptor.size = (uint32) sizeof (Raw.MetalOwnedTextureDescriptor);
            descriptor.extent.size = (uint32) sizeof (Raw.RenderTargetExtent);
            descriptor.extent.width = width;
            descriptor.extent.height = height;
            descriptor.extent.scale_factor = scale_factor;
            descriptor.context.size = (uint32) sizeof (Raw.MetalContextDescriptor);
            descriptor.context.device = device.to_native ();
            return descriptor;
        }
    }

    public class RuntimeOptions {
        public string? asset_path { get; set; }
        public string? cache_path { get; set; }
        public uint64? maximum_cache_size { get; set; }

        internal Raw.RuntimeOptions to_native () {
            Raw.RuntimeOptions options = {};
            options.size = (uint32) sizeof (Raw.RuntimeOptions);
            options.asset_path = asset_path;
            options.cache_path = cache_path;
            if (maximum_cache_size != null) {
                options.maximum_cache_size = maximum_cache_size;
                options.flags |= 1U << 0;
            }
            return options;
        }
    }

    public class MapOptions {
        public uint32 width { get; set; default = 64; }
        public uint32 height { get; set; default = 64; }
        public double scale_factor { get; set; default = 1.0; }
        public MapMode mode { get; set; default = MapMode.CONTINUOUS; }

        internal Raw.MapOptions to_native () {
            Raw.MapOptions options = {};
            options.size = (uint32) sizeof (Raw.MapOptions);
            options.width = width;
            options.height = height;
            options.scale_factor = scale_factor;
            options.map_mode = (uint32) mode;
            return options;
        }
    }

    public class CameraOptions {
        private bool has_center;
        private LatLng center_value;
        private bool has_zoom;
        private double zoom_value;
        private bool has_bearing;
        private double bearing_value;
        private bool has_pitch;
        private double pitch_value;

        public CameraOptions () {
        }

        public void set_center (LatLng center) {
            center_value = center;
            has_center = true;
        }

        public void set_zoom (double zoom) {
            zoom_value = zoom;
            has_zoom = true;
        }

        public void set_bearing (double bearing) {
            bearing_value = bearing;
            has_bearing = true;
        }

        public void set_pitch (double pitch) {
            pitch_value = pitch;
            has_pitch = true;
        }

        public bool get_center (out LatLng center) {
            center = center_value;
            return has_center;
        }

        public bool get_zoom (out double zoom) {
            zoom = zoom_value;
            return has_zoom;
        }

        public bool get_bearing (out double bearing) {
            bearing = bearing_value;
            return has_bearing;
        }

        public bool get_pitch (out double pitch) {
            pitch = pitch_value;
            return has_pitch;
        }

        internal Raw.CameraOptions to_native () {
            Raw.CameraOptions options = {};
            options.size = (uint32) sizeof (Raw.CameraOptions);
            if (has_center) {
                options.latitude = center_value.latitude;
                options.longitude = center_value.longitude;
                options.fields |= 1U << 0;
            }
            if (has_zoom) {
                options.zoom = zoom_value;
                options.fields |= 1U << 1;
            }
            if (has_bearing) {
                options.bearing = bearing_value;
                options.fields |= 1U << 2;
            }
            if (has_pitch) {
                options.pitch = pitch_value;
                options.fields |= 1U << 3;
            }
            return options;
        }

        internal static CameraOptions from_native (Raw.CameraOptions native) {
            var camera = new CameraOptions ();
            if ((native.fields & (1U << 0)) != 0) {
                camera.set_center (LatLng (native.latitude, native.longitude));
            }
            if ((native.fields & (1U << 1)) != 0) {
                camera.set_zoom (native.zoom);
            }
            if ((native.fields & (1U << 2)) != 0) {
                camera.set_bearing (native.bearing);
            }
            if ((native.fields & (1U << 3)) != 0) {
                camera.set_pitch (native.pitch);
            }
            return camera;
        }
    }

    public class RuntimeEvent {
        public RuntimeEventType event_type { get; private set; }
        public RuntimeEventSourceType source_type { get; private set; }
        public int32 code { get; private set; }
        public string message { get; private set; }

        internal RuntimeEvent (Raw.RuntimeEvent native) {
            event_type = runtime_event_type_from_raw (native.type);
            source_type = runtime_event_source_type_from_raw (native.source_type);
            code = native.code;
            message = copy_c_string (native.message);
        }
    }

    public delegate bool LogCallback (LogSeverity severity, LogEvent event, int64 code, string? message);
    public delegate string? ResourceTransformCallback (ResourceKind kind, string url);

    private LogCallback? current_log_callback;

    private uint32 log_trampoline (void* user_data, uint32 severity, uint32 event, int64 code, string? message) {
        if (current_log_callback == null) {
            return 0;
        }
        return current_log_callback (log_severity_from_raw (severity), log_event_from_raw (event), code, message) ? 1U : 0U;
    }

    private Raw.Status resource_transform_trampoline (void* user_data, uint32 kind, string url, Raw.ResourceTransformResponse* out_response) {
        if (user_data == null) {
            return Raw.Status.INVALID_ARGUMENT;
        }
        unowned RuntimeHandle runtime = (RuntimeHandle) user_data;
        return runtime.invoke_resource_transform (kind, url, out_response);
    }

    public uint32 c_version () {
        return Raw.c_version ();
    }

    public RenderBackendFlags supported_render_backends () {
        return (RenderBackendFlags) Raw.supported_render_backend_mask ();
    }

    public NetworkStatus network_status () throws Error {
        uint32 raw_status;
        check_status (Raw.network_status_get (out raw_status));
        return network_status_from_raw (raw_status);
    }

    public void set_network_status (NetworkStatus status) throws Error {
        check_status (Raw.network_status_set ((uint32) status));
    }

    public void set_log_callback (owned LogCallback callback) throws Error {
        current_log_callback = (owned) callback;
        check_status (Raw.log_set_callback (log_trampoline, null));
    }

    public void clear_log_callback () throws Error {
        check_status (Raw.log_clear_callback ());
        current_log_callback = null;
    }

    public ProjectedMeters projected_meters_for_lat_lng (LatLng coordinate) throws Error {
        Raw.ProjectedMeters meters;
        check_status (Raw.projected_meters_for_lat_lng (coordinate.to_native (), out meters));
        return ProjectedMeters.from_native (meters);
    }

    public LatLng lat_lng_for_projected_meters (ProjectedMeters meters) throws Error {
        Raw.LatLng coordinate;
        check_status (Raw.lat_lng_for_projected_meters (meters.to_native (), out coordinate));
        return LatLng.from_native (coordinate);
    }

    public class RuntimeHandle {
        private Raw.Runtime? native;
        private ResourceTransformCallback? resource_transform;
        private string? resource_transform_url;

        public bool closed { get { return native == null; } }

        public RuntimeHandle (RuntimeOptions? options = null) throws Error {
            var native_options = (options ?? new RuntimeOptions ()).to_native ();
            Raw.Runtime created;
            check_status (Raw.runtime_create (&native_options, out created));
            native = (owned) created;
        }

        ~RuntimeHandle () {
            if (native != null) {
                warning ("RuntimeHandle finalized while live; call close() on the owner thread");
            }
        }

        internal unowned Raw.Runtime require_live () throws Error {
            if (native == null) {
                throw new Error.INVALID_STATE ("runtime handle is closed");
            }
            return native;
        }

        public void close () throws Error {
            if (native == null) {
                return;
            }
            unowned Raw.Runtime closing = native;
            check_status (Raw.runtime_destroy (closing));
            native = null;
        }

        public void run_once () throws Error {
            check_status (Raw.runtime_run_once (require_live ()));
        }

        public RuntimeEvent? poll_event () throws Error {
            Raw.RuntimeEvent raw_event = {};
            raw_event.size = (uint32) sizeof (Raw.RuntimeEvent);
            bool has_event;
            check_status (Raw.runtime_poll_event (require_live (), &raw_event, out has_event));
            if (!has_event) {
                return null;
            }
            return new RuntimeEvent (raw_event);
        }

        public void set_resource_transform (owned ResourceTransformCallback callback) throws Error {
            resource_transform = (owned) callback;
            Raw.ResourceTransform transform = {};
            transform.size = (uint32) sizeof (Raw.ResourceTransform);
            transform.callback = resource_transform_trampoline;
            transform.user_data = this;
            check_status (Raw.runtime_set_resource_transform (require_live (), &transform));
        }

        public void clear_resource_transform () throws Error {
            check_status (Raw.runtime_clear_resource_transform (require_live ()));
            resource_transform = null;
            resource_transform_url = null;
        }

        internal Raw.Status invoke_resource_transform (uint32 raw_kind, string url, Raw.ResourceTransformResponse* out_response) {
            if (out_response == null || resource_transform == null) {
                return Raw.Status.INVALID_ARGUMENT;
            }
            out_response->size = (uint32) sizeof (Raw.ResourceTransformResponse);
            var replacement = resource_transform (resource_kind_from_raw (raw_kind), url);
            resource_transform_url = replacement;
            out_response->url = resource_transform_url;
            return Raw.Status.OK;
        }

        public uint64 run_ambient_cache_operation_start (AmbientCacheOperation operation) throws Error {
            uint64 operation_id;
            check_status (Raw.runtime_run_ambient_cache_operation_start (require_live (), (uint32) operation, out operation_id));
            return operation_id;
        }

        public void discard_offline_operation (uint64 operation_id) throws Error {
            check_status (Raw.runtime_offline_operation_discard (require_live (), operation_id));
        }
    }

    public class RenderedQueryGeometry {
        internal Raw.RenderedQueryGeometry native;

        private RenderedQueryGeometry (Raw.RenderedQueryGeometry native) {
            this.native = native;
        }

        public static RenderedQueryGeometry point (ScreenPoint point) {
            return new RenderedQueryGeometry (Raw.rendered_query_geometry_point (point.to_native ()));
        }
    }

    public class FeatureQueryResultHandle {
        private Raw.FeatureQueryResult? native;

        public bool closed { get { return native == null; } }

        internal FeatureQueryResultHandle (owned Raw.FeatureQueryResult native) {
            this.native = (owned) native;
        }

        ~FeatureQueryResultHandle () {
            if (native != null) {
                warning ("FeatureQueryResultHandle finalized while live; call close()");
            }
        }

        internal unowned Raw.FeatureQueryResult require_live () throws Error {
            if (native == null) {
                throw new Error.INVALID_STATE ("feature query result handle is closed");
            }
            return native;
        }

        public void close () {
            if (native == null) {
                return;
            }
            unowned Raw.FeatureQueryResult closing = native;
            Raw.feature_query_result_destroy (closing);
            native = null;
        }

        public size_t count () throws Error {
            size_t result_count;
            check_status (Raw.feature_query_result_count (require_live (), out result_count));
            return result_count;
        }
    }

    public class RenderSessionHandle {
        private MapHandle map;
        private Raw.RenderSession? native;
        private bool frame_acquired;

        public bool closed { get { return native == null; } }

        internal RenderSessionHandle (MapHandle map, owned Raw.RenderSession native) {
            this.map = map;
            this.native = (owned) native;
        }

        ~RenderSessionHandle () {
            if (native != null) {
                warning ("RenderSessionHandle finalized while live; call close() on the owner thread");
            }
        }

        internal unowned Raw.RenderSession require_live () throws Error {
            if (native == null) {
                throw new Error.INVALID_STATE ("render session handle is closed");
            }
            return native;
        }

        internal void begin_frame_borrow () throws Error {
            if (frame_acquired) {
                throw new Error.INVALID_STATE ("render session already has an acquired frame");
            }
            frame_acquired = true;
        }

        internal void finish_frame_borrow () {
            frame_acquired = false;
        }

        public void close () throws Error {
            if (native == null) {
                return;
            }
            if (frame_acquired) {
                throw new Error.INVALID_STATE ("render session has an acquired frame");
            }
            unowned Raw.RenderSession closing = native;
            check_status (Raw.render_session_destroy (closing));
            native = null;
        }

        public void render_update () throws Error {
            if (frame_acquired) {
                throw new Error.INVALID_STATE ("render session has an acquired frame");
            }
            check_status (Raw.render_session_render_update (require_live ()));
        }

        public TextureImageInfo read_premultiplied_rgba8 (uint8[] out_data) throws Error {
            if (out_data.length == 0) {
                throw new Error.INVALID_ARGUMENT ("readback buffer is empty");
            }
            Raw.TextureImageInfo info = {};
            info.size = (uint32) sizeof (Raw.TextureImageInfo);
            check_status (Raw.texture_read_premultiplied_rgba8 (require_live (), out_data, out_data.length, &info));
            return new TextureImageInfo (info);
        }

        public FeatureQueryResultHandle query_rendered_features (RenderedQueryGeometry geometry) throws Error {
            Raw.FeatureQueryResult result;
            check_status (Raw.render_session_query_rendered_features (require_live (), &geometry.native, null, out result));
            return new FeatureQueryResultHandle ((owned) result);
        }

        public MetalOwnedTextureFrameHandle acquire_metal_owned_texture_frame () throws Error {
            begin_frame_borrow ();
            Raw.MetalOwnedTextureFrame frame = {};
            frame.size = (uint32) sizeof (Raw.MetalOwnedTextureFrame);
            try {
                check_status (Raw.metal_owned_texture_acquire_frame (require_live (), &frame));
                return new MetalOwnedTextureFrameHandle (this, frame);
            } catch (Error error) {
                finish_frame_borrow ();
                throw error;
            }
        }
    }

    public class MetalOwnedTextureFrameHandle {
        private RenderSessionHandle session;
        private Raw.MetalOwnedTextureFrame frame;
        private bool closed;

        internal MetalOwnedTextureFrameHandle (RenderSessionHandle session, Raw.MetalOwnedTextureFrame frame) {
            this.session = session;
            this.frame = frame;
        }

        ~MetalOwnedTextureFrameHandle () {
            if (!closed) {
                warning ("MetalOwnedTextureFrameHandle finalized while live; call close() on the owner thread");
            }
        }

        private void require_live () throws Error {
            if (closed) {
                throw new Error.INVALID_STATE ("metal texture frame is closed");
            }
        }

        public void close () throws Error {
            if (closed) {
                return;
            }
            check_status (Raw.metal_owned_texture_release_frame (session.require_live (), &frame));
            closed = true;
            session.finish_frame_borrow ();
        }

        public uint32 get_width () throws Error {
            require_live ();
            return frame.width;
        }

        public uint32 get_height () throws Error {
            require_live ();
            return frame.height;
        }

        public double get_scale_factor () throws Error {
            require_live ();
            return frame.scale_factor;
        }

        public uint64 get_generation () throws Error {
            require_live ();
            return frame.generation;
        }

        public uint64 get_frame_id () throws Error {
            require_live ();
            return frame.frame_id;
        }

        public NativePointer get_texture () throws Error {
            require_live ();
            return NativePointer ((size_t) frame.texture);
        }

        public NativePointer get_device () throws Error {
            require_live ();
            return NativePointer ((size_t) frame.device);
        }

        public uint64 get_pixel_format () throws Error {
            require_live ();
            return frame.pixel_format;
        }
    }

    public class MapProjectionHandle {
        private Raw.MapProjection? native;

        public bool closed { get { return native == null; } }

        internal MapProjectionHandle (owned Raw.MapProjection native) {
            this.native = (owned) native;
        }

        ~MapProjectionHandle () {
            if (native != null) {
                warning ("MapProjectionHandle finalized while live; call close() on the owner thread");
            }
        }

        internal unowned Raw.MapProjection require_live () throws Error {
            if (native == null) {
                throw new Error.INVALID_STATE ("map projection handle is closed");
            }
            return native;
        }

        public void close () throws Error {
            if (native == null) {
                return;
            }
            unowned Raw.MapProjection closing = native;
            check_status (Raw.map_projection_destroy (closing));
            native = null;
        }

        public CameraOptions get_camera () throws Error {
            Raw.CameraOptions native_camera = {};
            native_camera.size = (uint32) sizeof (Raw.CameraOptions);
            check_status (Raw.map_projection_get_camera (require_live (), &native_camera));
            return CameraOptions.from_native (native_camera);
        }

        public void set_camera (CameraOptions camera) throws Error {
            var native_camera = camera.to_native ();
            check_status (Raw.map_projection_set_camera (require_live (), &native_camera));
        }

        public ScreenPoint pixel_for_lat_lng (LatLng coordinate) throws Error {
            Raw.ScreenPoint point;
            check_status (Raw.map_projection_pixel_for_lat_lng (require_live (), coordinate.to_native (), out point));
            return ScreenPoint.from_native (point);
        }

        public LatLng lat_lng_for_pixel (ScreenPoint point) throws Error {
            Raw.LatLng coordinate;
            check_status (Raw.map_projection_lat_lng_for_pixel (require_live (), point.to_native (), out coordinate));
            return LatLng.from_native (coordinate);
        }

        public void set_visible_coordinates (LatLng[] coordinates, EdgeInsets padding) throws Error {
            if (coordinates.length == 0) {
                throw new Error.INVALID_ARGUMENT ("visible coordinates are empty");
            }
            Raw.LatLng[] native_coordinates = new Raw.LatLng[coordinates.length];
            for (var i = 0; i < coordinates.length; i++) {
                native_coordinates[i] = coordinates[i].to_native ();
            }
            check_status (Raw.map_projection_set_visible_coordinates (require_live (), native_coordinates, native_coordinates.length, padding.to_native ()));
        }
    }

    public class MapHandle {
        private RuntimeHandle runtime;
        private Raw.Map? native;

        public bool closed { get { return native == null; } }

        public MapHandle (RuntimeHandle runtime, MapOptions? options = null) throws Error {
            this.runtime = runtime;
            var native_options = (options ?? new MapOptions ()).to_native ();
            Raw.Map created;
            check_status (Raw.map_create (runtime.require_live (), &native_options, out created));
            native = (owned) created;
        }

        ~MapHandle () {
            if (native != null) {
                warning ("MapHandle finalized while live; call close() on the owner thread");
            }
        }

        internal unowned Raw.Map require_live () throws Error {
            if (native == null) {
                throw new Error.INVALID_STATE ("map handle is closed");
            }
            return native;
        }

        public void close () throws Error {
            if (native == null) {
                return;
            }
            unowned Raw.Map closing = native;
            check_status (Raw.map_destroy (closing));
            native = null;
        }

        public void request_repaint () throws Error {
            check_status (Raw.map_request_repaint (require_live ()));
        }

        public void request_still_image () throws Error {
            check_status (Raw.map_request_still_image (require_live ()));
        }

        public void set_style_url (string url) throws Error {
            check_status (Raw.map_set_style_url (require_live (), url));
        }

        public void set_style_json (string json) throws Error {
            check_status (Raw.map_set_style_json (require_live (), json));
        }

        public void set_debug_options (uint32 options) throws Error {
            check_status (Raw.map_set_debug_options (require_live (), options));
        }

        public uint32 get_debug_options () throws Error {
            uint32 options;
            check_status (Raw.map_get_debug_options (require_live (), out options));
            return options;
        }

        public void set_rendering_stats_view_enabled (bool enabled) throws Error {
            check_status (Raw.map_set_rendering_stats_view_enabled (require_live (), enabled));
        }

        public bool get_rendering_stats_view_enabled () throws Error {
            bool enabled;
            check_status (Raw.map_get_rendering_stats_view_enabled (require_live (), out enabled));
            return enabled;
        }

        public bool is_fully_loaded () throws Error {
            bool loaded;
            check_status (Raw.map_is_fully_loaded (require_live (), out loaded));
            return loaded;
        }

        public void dump_debug_logs () throws Error {
            check_status (Raw.map_dump_debug_logs (require_live ()));
        }

        public CameraOptions get_camera () throws Error {
            Raw.CameraOptions native_camera = {};
            native_camera.size = (uint32) sizeof (Raw.CameraOptions);
            check_status (Raw.map_get_camera (require_live (), &native_camera));
            return CameraOptions.from_native (native_camera);
        }

        public void jump_to (CameraOptions camera) throws Error {
            var native_camera = camera.to_native ();
            check_status (Raw.map_jump_to (require_live (), &native_camera));
        }

        public MapProjectionHandle create_projection () throws Error {
            Raw.MapProjection projection;
            check_status (Raw.map_projection_create (require_live (), out projection));
            return new MapProjectionHandle ((owned) projection);
        }

        public RenderSessionHandle attach_metal_owned_texture (MetalOwnedTextureDescriptor descriptor) throws Error {
            var native_descriptor = descriptor.to_native ();
            Raw.RenderSession session;
            check_status (Raw.metal_owned_texture_attach (require_live (), &native_descriptor, out session));
            return new RenderSessionHandle (this, (owned) session);
        }

        public void add_geojson_source_url (string source_id, string url) throws Error {
            check_status (Raw.map_add_geojson_source_url (require_live (), string_view (source_id), string_view (url)));
        }

        public void set_geojson_source_url (string source_id, string url) throws Error {
            check_status (Raw.map_set_geojson_source_url (require_live (), string_view (source_id), string_view (url)));
        }

        public bool remove_style_source (string source_id) throws Error {
            bool removed;
            check_status (Raw.map_remove_style_source (require_live (), string_view (source_id), out removed));
            return removed;
        }

        public bool style_source_exists (string source_id) throws Error {
            bool exists;
            check_status (Raw.map_style_source_exists (require_live (), string_view (source_id), out exists));
            return exists;
        }

        public StyleSourceType get_style_source_type (string source_id) throws Error {
            uint32 source_type;
            bool found;
            check_status (Raw.map_get_style_source_type (require_live (), string_view (source_id), out source_type, out found));
            return found ? style_source_type_from_raw (source_type) : StyleSourceType.UNKNOWN;
        }

        public StyleSourceInfo? get_style_source_info (string source_id) throws Error {
            Raw.StyleSourceInfo info = {};
            info.size = (uint32) sizeof (Raw.StyleSourceInfo);
            bool found;
            check_status (Raw.map_get_style_source_info (require_live (), string_view (source_id), &info, out found));
            return found ? new StyleSourceInfo (info) : null;
        }

        public StringList list_style_source_ids () throws Error {
            Raw.StyleIdList list;
            check_status (Raw.map_list_style_source_ids (require_live (), out list));
            return copy_style_id_list ((owned) list);
        }

        public void add_location_indicator_layer (string layer_id, string before_layer_id = "") throws Error {
            check_status (Raw.map_add_location_indicator_layer (require_live (), string_view (layer_id), string_view (before_layer_id)));
        }

        public bool remove_style_layer (string layer_id) throws Error {
            bool removed;
            check_status (Raw.map_remove_style_layer (require_live (), string_view (layer_id), out removed));
            return removed;
        }

        public bool style_layer_exists (string layer_id) throws Error {
            bool exists;
            check_status (Raw.map_style_layer_exists (require_live (), string_view (layer_id), out exists));
            return exists;
        }

        public StringList list_style_layer_ids () throws Error {
            Raw.StyleIdList list;
            check_status (Raw.map_list_style_layer_ids (require_live (), out list));
            return copy_style_id_list ((owned) list);
        }

        public void set_style_image (string image_id, PremultipliedRgba8Image image, StyleImageOptions? options = null) throws Error {
            var native_image = image.to_native ();
            var native_options = (options ?? new StyleImageOptions ()).to_native ();
            check_status (Raw.map_set_style_image (require_live (), string_view (image_id), &native_image, &native_options));
        }

        public bool remove_style_image (string image_id) throws Error {
            bool removed;
            check_status (Raw.map_remove_style_image (require_live (), string_view (image_id), out removed));
            return removed;
        }

        public bool style_image_exists (string image_id) throws Error {
            bool exists;
            check_status (Raw.map_style_image_exists (require_live (), string_view (image_id), out exists));
            return exists;
        }

        public StyleImageInfo? get_style_image_info (string image_id) throws Error {
            Raw.StyleImageInfo info = {};
            info.size = (uint32) sizeof (Raw.StyleImageInfo);
            bool found;
            check_status (Raw.map_get_style_image_info (require_live (), string_view (image_id), &info, out found));
            return found ? new StyleImageInfo (info) : null;
        }

        public uint8[]? copy_style_image_premultiplied_rgba8 (string image_id) throws Error {
            var info = get_style_image_info (image_id);
            if (info == null) {
                return null;
            }
            uint8[] pixels = new uint8[info.byte_length];
            size_t byte_length;
            bool found;
            check_status (Raw.map_copy_style_image_premultiplied_rgba8 (require_live (), string_view (image_id), pixels, pixels.length, out byte_length, out found));
            return found ? pixels : null;
        }
    }

    private void check_status (Raw.Status status) throws Error {
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

    private string copy_c_string (char* value) {
        if (value == null) {
            return "";
        }
        return (string) value;
    }

    private Raw.StringView string_view (string value) throws Error {
        return Raw.StringView () { data = (char*) value, size = value.length };
    }

    private string copy_string_view (Raw.StringView view) throws Error {
        if (view.data == null && view.size == 0) {
            return "";
        }
        if (view.data == null) {
            throw new Error.INVALID_ARGUMENT ("string view data is null");
        }
        return ((string) view.data).substring (0, (long) view.size);
    }

    private StringList copy_style_id_list (owned Raw.StyleIdList list) throws Error {
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

    private NetworkStatus network_status_from_raw (uint32 raw_status) {
        switch (raw_status) {
            case 1:
                return NetworkStatus.ONLINE;
            case 2:
                return NetworkStatus.OFFLINE;
            default:
                return NetworkStatus.UNKNOWN;
        }
    }

    private RuntimeEventType runtime_event_type_from_raw (uint32 raw_type) {
        if (raw_type >= 1 && raw_type <= 22) {
            return (RuntimeEventType) raw_type;
        }
        return RuntimeEventType.UNKNOWN;
    }

    private RuntimeEventSourceType runtime_event_source_type_from_raw (uint32 raw_type) {
        if (raw_type <= 1) {
            return (RuntimeEventSourceType) raw_type;
        }
        return RuntimeEventSourceType.UNKNOWN;
    }

    private LogSeverity log_severity_from_raw (uint32 raw_severity) {
        if (raw_severity >= 1 && raw_severity <= 3) {
            return (LogSeverity) raw_severity;
        }
        return LogSeverity.UNKNOWN;
    }

    private StyleSourceType style_source_type_from_raw (uint32 raw_type) {
        if (raw_type <= 8) {
            return (StyleSourceType) raw_type;
        }
        return StyleSourceType.UNKNOWN;
    }

    private ResourceKind resource_kind_from_raw (uint32 raw_kind) {
        if (raw_kind <= 7) {
            return (ResourceKind) raw_kind;
        }
        return ResourceKind.UNKNOWN;
    }

    private LogEvent log_event_from_raw (uint32 raw_event) {
        if (raw_event <= 16) {
            return (LogEvent) raw_event;
        }
        return LogEvent.UNKNOWN;
    }
}
