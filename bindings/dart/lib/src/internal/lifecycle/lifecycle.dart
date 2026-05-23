import 'dart:ffi';

import '../status/status.dart';

/// Close-once state for an owned native handle pointer.
final class NativeHandleState<T extends NativeType> {
  /// Creates state for a live native handle pointer.
  NativeHandleState(this._pointer, this.typeName) {
    if (_pointer == nullptr) {
      throwInvalidArgument('$typeName pointer must not be null');
    }
  }

  Pointer<T>? _pointer;

  /// Native handle type name used in diagnostics.
  final String typeName;

  /// Whether this binding object has released its native handle.
  bool get isClosed => _pointer == null;

  /// Returns the live pointer, or throws when the handle is closed.
  Pointer<T> get pointer {
    final pointer = _pointer;
    if (pointer == null) {
      throwInvalidArgument('$typeName is closed');
    }
    return pointer;
  }

  /// Releases the native handle with [destroy] exactly once after success.
  void close(int Function(Pointer<T>) destroy, String Function() diagnostic) {
    final pointer = _pointer;
    if (pointer == null) {
      return;
    }

    final status = destroy(pointer);
    checkNativeStatus(status, diagnostic);
    _pointer = null;
  }
}
