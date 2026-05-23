/// Resource requests, responses, transforms, providers, and request handles.
library;

/// Resource kind requested by MapLibre Native.
final class ResourceKind {
  const ResourceKind._(this.rawValue, this.name);

  /// Unknown resource kind.
  static const unknown = ResourceKind._(0, 'unknown');

  /// Style document.
  static const style = ResourceKind._(1, 'style');

  /// Source metadata.
  static const source = ResourceKind._(2, 'source');

  /// Tile payload.
  static const tile = ResourceKind._(3, 'tile');

  /// Glyph payload.
  static const glyphs = ResourceKind._(4, 'glyphs');

  /// Sprite image.
  static const spriteImage = ResourceKind._(5, 'spriteImage');

  /// Sprite JSON.
  static const spriteJson = ResourceKind._(6, 'spriteJson');

  /// Image payload.
  static const image = ResourceKind._(7, 'image');

  /// Creates a resource kind from a raw native value.
  factory ResourceKind.fromRawValue(int rawValue) => switch (rawValue) {
    0 => unknown,
    1 => style,
    2 => source,
    3 => tile,
    4 => glyphs,
    5 => spriteImage,
    6 => spriteJson,
    7 => image,
    _ => ResourceKind._(rawValue, 'unknown($rawValue)'),
  };

  /// Raw native value.
  final int rawValue;

  /// Human-readable name.
  final String name;
}

/// Resource loading method.
final class ResourceLoadingMethod {
  const ResourceLoadingMethod._(this.rawValue, this.name);

  /// All loading methods.
  static const all = ResourceLoadingMethod._(0, 'all');

  /// Cache-only loading.
  static const cacheOnly = ResourceLoadingMethod._(1, 'cacheOnly');

  /// Network-only loading.
  static const networkOnly = ResourceLoadingMethod._(2, 'networkOnly');

  /// Raw native value.
  final int rawValue;

  /// Human-readable name.
  final String name;
}

/// Resource priority.
final class ResourcePriority {
  const ResourcePriority._(this.rawValue, this.name);

  /// Regular priority.
  static const regular = ResourcePriority._(0, 'regular');

  /// Low priority.
  static const low = ResourcePriority._(1, 'low');

  /// Raw native value.
  final int rawValue;

  /// Human-readable name.
  final String name;
}

/// Resource usage.
final class ResourceUsage {
  const ResourceUsage._(this.rawValue, this.name);

  /// Online usage.
  static const online = ResourceUsage._(0, 'online');

  /// Offline usage.
  static const offline = ResourceUsage._(1, 'offline');

  /// Raw native value.
  final int rawValue;

  /// Human-readable name.
  final String name;
}

/// Resource storage policy.
final class ResourceStoragePolicy {
  const ResourceStoragePolicy._(this.rawValue, this.name);

  /// Permanent storage.
  static const permanent = ResourceStoragePolicy._(0, 'permanent');

  /// Volatile storage.
  static const volatile = ResourceStoragePolicy._(1, 'volatile');

  /// Raw native value.
  final int rawValue;

  /// Human-readable name.
  final String name;
}

/// Resource response status.
final class ResourceResponseStatus {
  const ResourceResponseStatus._(this.rawValue, this.name);

  /// Successful response.
  static const ok = ResourceResponseStatus._(0, 'ok');

  /// Error response.
  static const error = ResourceResponseStatus._(1, 'error');

  /// No-content response.
  static const noContent = ResourceResponseStatus._(2, 'noContent');

  /// Not-modified response.
  static const notModified = ResourceResponseStatus._(3, 'notModified');

  /// Raw native value.
  final int rawValue;

  /// Human-readable name.
  final String name;
}

/// Decision returned by a resource provider callback.
final class ResourceProviderDecision {
  const ResourceProviderDecision._(this.rawValue, this.name);

  /// Let native networking handle the request.
  static const passThrough = ResourceProviderDecision._(0, 'passThrough');

  /// The binding will complete or release the request handle.
  static const handle = ResourceProviderDecision._(1, 'handle');

  /// Raw native value.
  final int rawValue;

  /// Human-readable name.
  final String name;
}
