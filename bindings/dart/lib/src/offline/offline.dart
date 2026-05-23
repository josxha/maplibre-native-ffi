/// Offline region definitions, status values, metadata, and operation handles.
library;

/// Ambient cache maintenance operation.
final class AmbientCacheOperation {
  const AmbientCacheOperation._(this.rawValue, this.name);

  /// Reset the ambient cache database.
  static const resetDatabase = AmbientCacheOperation._(1, 'resetDatabase');

  /// Pack the ambient cache database.
  static const packDatabase = AmbientCacheOperation._(2, 'packDatabase');

  /// Invalidate cached ambient resources.
  static const invalidate = AmbientCacheOperation._(3, 'invalidate');

  /// Clear cached ambient resources.
  static const clear = AmbientCacheOperation._(4, 'clear');

  /// Raw native value.
  final int rawValue;

  /// Human-readable name.
  final String name;
}
