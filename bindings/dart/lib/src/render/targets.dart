import 'native_pointer.dart';

/// Logical render target extent in UI pixels.
final class RenderTargetExtent {
  /// Creates a render target extent.
  const RenderTargetExtent({
    required this.width,
    required this.height,
    this.scaleFactor = 1,
  });

  /// Logical map width in UI pixels.
  final int width;

  /// Logical map height in UI pixels.
  final int height;

  /// UI-to-device pixel scale.
  final double scaleFactor;
}

/// Metal backend context fields shared by Metal render targets.
final class MetalContextDescriptor {
  /// Creates a Metal context descriptor.
  const MetalContextDescriptor({required this.device});

  /// Borrowed or retained native Metal device pointer, depending on target kind.
  final NativePointer device;
}

/// Vulkan backend context fields shared by Vulkan render targets.
final class VulkanContextDescriptor {
  /// Creates a Vulkan context descriptor.
  const VulkanContextDescriptor({
    required this.instance,
    required this.physicalDevice,
    required this.device,
    required this.graphicsQueue,
    required this.graphicsQueueFamilyIndex,
  });

  /// Borrowed VkInstance.
  final NativePointer instance;

  /// Borrowed VkPhysicalDevice.
  final NativePointer physicalDevice;

  /// Borrowed VkDevice.
  final NativePointer device;

  /// Borrowed graphics VkQueue.
  final NativePointer graphicsQueue;

  /// Queue family index for [graphicsQueue].
  final int graphicsQueueFamilyIndex;
}

/// Metal native surface session attachment options.
final class MetalSurfaceDescriptor {
  /// Creates a Metal surface descriptor.
  const MetalSurfaceDescriptor({
    required this.extent,
    required this.context,
    required this.layer,
  });

  /// Logical surface extent.
  final RenderTargetExtent extent;

  /// Metal backend context.
  final MetalContextDescriptor context;

  /// Borrowed `CAMetalLayer*` / `CA::MetalLayer*`.
  final NativePointer layer;
}

/// Vulkan native surface session attachment options.
final class VulkanSurfaceDescriptor {
  /// Creates a Vulkan surface descriptor.
  const VulkanSurfaceDescriptor({
    required this.extent,
    required this.context,
    required this.surface,
  });

  /// Logical surface extent.
  final RenderTargetExtent extent;

  /// Borrowed Vulkan context.
  final VulkanContextDescriptor context;

  /// Borrowed `VkSurfaceKHR`.
  final NativePointer surface;
}

/// Metal texture session attachment options for a session-owned target.
final class MetalOwnedTextureDescriptor {
  /// Creates a Metal owned-texture descriptor.
  const MetalOwnedTextureDescriptor({
    required this.extent,
    required this.context,
  });

  /// Logical texture extent.
  final RenderTargetExtent extent;

  /// Metal backend context.
  final MetalContextDescriptor context;
}

/// Metal caller-owned texture session attachment options.
final class MetalBorrowedTextureDescriptor {
  /// Creates a Metal borrowed-texture descriptor.
  const MetalBorrowedTextureDescriptor({
    required this.extent,
    required this.texture,
  });

  /// Logical texture extent.
  final RenderTargetExtent extent;

  /// Borrowed `id<MTLTexture>` / `MTL::Texture*`.
  final NativePointer texture;
}

/// Vulkan texture session attachment options for a session-owned target.
final class VulkanOwnedTextureDescriptor {
  /// Creates a Vulkan owned-texture descriptor.
  const VulkanOwnedTextureDescriptor({
    required this.extent,
    required this.context,
  });

  /// Logical texture extent.
  final RenderTargetExtent extent;

  /// Borrowed Vulkan context.
  final VulkanContextDescriptor context;
}

/// Vulkan caller-owned texture session attachment options.
final class VulkanBorrowedTextureDescriptor {
  /// Creates a Vulkan borrowed-texture descriptor.
  const VulkanBorrowedTextureDescriptor({
    required this.extent,
    required this.context,
    required this.image,
    required this.imageView,
    required this.format,
    required this.initialLayout,
    required this.finalLayout,
  });

  /// Logical texture extent.
  final RenderTargetExtent extent;

  /// Borrowed Vulkan context.
  final VulkanContextDescriptor context;

  /// Borrowed VkImage.
  final NativePointer image;

  /// Borrowed VkImageView.
  final NativePointer imageView;

  /// Backend-native VkFormat value.
  final int format;

  /// Backend-native VkImageLayout at render-pass begin.
  final int initialLayout;

  /// Backend-native VkImageLayout left after rendering succeeds.
  final int finalLayout;
}
