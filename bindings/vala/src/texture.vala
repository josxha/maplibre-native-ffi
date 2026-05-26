namespace MaplibreNative {
    internal Raw.RenderTargetExtent render_target_extent (uint32 width, uint32 height, double scale_factor) {
        Raw.RenderTargetExtent extent = {};
        extent.size = (uint32) sizeof (Raw.RenderTargetExtent);
        extent.width = width;
        extent.height = height;
        extent.scale_factor = scale_factor;
        return extent;
    }

    internal Raw.MetalContextDescriptor metal_context_descriptor (NativePointer device) throws Error {
        Raw.MetalContextDescriptor context = {};
        context.size = (uint32) sizeof (Raw.MetalContextDescriptor);
        context.device = device.to_native ();
        return context;
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

    public class MetalOwnedTextureDescriptor {
        public uint32 width { get; set; default = 64; }
        public uint32 height { get; set; default = 64; }
        public double scale_factor { get; set; default = 1.0; }
        public NativePointer device { get; set; }

        public MetalOwnedTextureDescriptor (NativePointer device) {
            this.device = device;
        }

        internal Raw.MetalOwnedTextureDescriptor to_native () throws Error {
            Raw.MetalOwnedTextureDescriptor descriptor = Raw.metal_owned_texture_descriptor_default ();
            descriptor.extent = render_target_extent (width, height, scale_factor);
            descriptor.context = metal_context_descriptor (device);
            return descriptor;
        }
    }

    public class MetalBorrowedTextureDescriptor {
        public uint32 width { get; set; default = 64; }
        public uint32 height { get; set; default = 64; }
        public double scale_factor { get; set; default = 1.0; }
        public NativePointer texture { get; set; }

        public MetalBorrowedTextureDescriptor (NativePointer texture) {
            this.texture = texture;
        }

        internal Raw.MetalBorrowedTextureDescriptor to_native () throws Error {
            Raw.MetalBorrowedTextureDescriptor descriptor = Raw.metal_borrowed_texture_descriptor_default ();
            descriptor.extent = render_target_extent (width, height, scale_factor);
            descriptor.texture = texture.to_native ();
            return descriptor;
        }
    }

    public class VulkanContextDescriptor {
        public NativePointer instance { get; set; }
        public NativePointer physical_device { get; set; }
        public NativePointer device { get; set; }
        public NativePointer graphics_queue { get; set; }
        public uint32 graphics_queue_family_index { get; set; }

        public VulkanContextDescriptor (NativePointer instance, NativePointer physical_device, NativePointer device, NativePointer graphics_queue, uint32 graphics_queue_family_index) {
            this.instance = instance;
            this.physical_device = physical_device;
            this.device = device;
            this.graphics_queue = graphics_queue;
            this.graphics_queue_family_index = graphics_queue_family_index;
        }

        internal Raw.VulkanContextDescriptor to_native () throws Error {
            Raw.VulkanContextDescriptor context = {};
            context.size = (uint32) sizeof (Raw.VulkanContextDescriptor);
            context.instance = instance.to_native ();
            context.physical_device = physical_device.to_native ();
            context.device = device.to_native ();
            context.graphics_queue = graphics_queue.to_native ();
            context.graphics_queue_family_index = graphics_queue_family_index;
            return context;
        }
    }

    public class VulkanOwnedTextureDescriptor {
        public uint32 width { get; set; default = 64; }
        public uint32 height { get; set; default = 64; }
        public double scale_factor { get; set; default = 1.0; }
        public VulkanContextDescriptor context { get; set; }

        public VulkanOwnedTextureDescriptor (VulkanContextDescriptor context) {
            this.context = context;
        }

        internal Raw.VulkanOwnedTextureDescriptor to_native () throws Error {
            Raw.VulkanOwnedTextureDescriptor descriptor = Raw.vulkan_owned_texture_descriptor_default ();
            descriptor.extent = render_target_extent (width, height, scale_factor);
            descriptor.context = context.to_native ();
            return descriptor;
        }
    }

    public class VulkanBorrowedTextureDescriptor {
        public uint32 width { get; set; default = 64; }
        public uint32 height { get; set; default = 64; }
        public double scale_factor { get; set; default = 1.0; }
        public VulkanContextDescriptor context { get; set; }
        public NativePointer image { get; set; }
        public NativePointer image_view { get; set; }
        public uint32 format { get; set; }
        public uint32 initial_layout { get; set; }
        public uint32 final_layout { get; set; }

        public VulkanBorrowedTextureDescriptor (VulkanContextDescriptor context, NativePointer image, NativePointer image_view) {
            this.context = context;
            this.image = image;
            this.image_view = image_view;
        }

        internal Raw.VulkanBorrowedTextureDescriptor to_native () throws Error {
            Raw.VulkanBorrowedTextureDescriptor descriptor = Raw.vulkan_borrowed_texture_descriptor_default ();
            descriptor.extent = render_target_extent (width, height, scale_factor);
            descriptor.context = context.to_native ();
            descriptor.image = image.to_native ();
            descriptor.image_view = image_view.to_native ();
            descriptor.format = format;
            descriptor.initial_layout = initial_layout;
            descriptor.final_layout = final_layout;
            return descriptor;
        }
    }

    public class VulkanOwnedTextureFrameHandle {
        private RenderSessionHandle session;
        private Raw.VulkanOwnedTextureFrame frame;
        private bool closed;

        internal VulkanOwnedTextureFrameHandle (RenderSessionHandle session, Raw.VulkanOwnedTextureFrame frame) {
            this.session = session;
            this.frame = frame;
        }

        ~VulkanOwnedTextureFrameHandle () {
            if (!closed) {
                warning ("VulkanOwnedTextureFrameHandle finalized while live; call close() on the owner thread");
            }
        }

        private void require_live () throws Error {
            if (closed) {
                throw new Error.INVALID_STATE ("vulkan texture frame is closed");
            }
        }

        public void close () throws Error {
            if (closed) {
                return;
            }
            check_status (Raw.vulkan_owned_texture_release_frame (session.require_live (), &frame));
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

        public NativePointer get_image () throws Error {
            require_live ();
            return NativePointer ((size_t) frame.image);
        }

        public NativePointer get_image_view () throws Error {
            require_live ();
            return NativePointer ((size_t) frame.image_view);
        }

        public NativePointer get_device () throws Error {
            require_live ();
            return NativePointer ((size_t) frame.device);
        }

        public uint32 get_format () throws Error {
            require_live ();
            return frame.format;
        }

        public uint32 get_layout () throws Error {
            require_live ();
            return frame.layout;
        }
    }
}
