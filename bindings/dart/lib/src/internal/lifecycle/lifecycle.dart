import 'dart:ffi';
import 'dart:isolate';

import 'package:ffi/ffi.dart';

import '../loader/native_library.dart';
import '../status/status.dart';

final DynamicLibrary _library = openMaplibreNativeCLibrary();

final NativeFinalizer _leakReporter = NativeFinalizer(
  _library.lookup<NativeFinalizerFunction>('mln_dart_handle_leak_report'),
);

final Pointer<Void> Function(Pointer<Char>, Pointer<Void>) _createLeakToken =
    _library.lookupFunction<
      Pointer<Void> Function(Pointer<Char>, Pointer<Void>),
      Pointer<Void> Function(Pointer<Char>, Pointer<Void>)
    >('mln_dart_handle_leak_token_create');

final void Function(Pointer<Void>) _destroyLeakToken = _library
    .lookupFunction<Void Function(Pointer<Void>), void Function(Pointer<Void>)>(
      'mln_dart_handle_leak_token_destroy',
    );

/// Close-once state for an owned native handle pointer.
final class NativeHandleState<T extends NativeType> implements Finalizable {
  /// Creates state for a live native handle pointer.
  NativeHandleState(
    this._pointer,
    this.typeName, {
    int? ownerIsolateHash,
    bool leakReporting = true,
  }) : _ownerIsolateHash = ownerIsolateHash ?? Isolate.current.hashCode {
    if (_pointer == nullptr) {
      throwInvalidArgument('$typeName pointer must not be null');
    }
    if (leakReporting) {
      _attachLeakReporter(_pointer!.cast<Void>());
    }
  }

  Pointer<T>? _pointer;
  final int _ownerIsolateHash;
  final Object _finalizerDetachToken = Object();
  Pointer<Void>? _leakToken;

  /// Native handle type name used in diagnostics.
  final String typeName;

  /// Whether this binding object has released its native handle.
  bool get isClosed => _pointer == null;

  /// Returns the live pointer, or throws when the handle is closed.
  Pointer<T> get pointer {
    _checkOwnerIsolate();
    final pointer = _pointer;
    if (pointer == null) {
      throwInvalidArgument('$typeName is closed');
    }
    return pointer;
  }

  /// Releases the native handle with [destroy] exactly once after success.
  void close(int Function(Pointer<T>) destroy, String Function() diagnostic) {
    _checkOwnerIsolate();
    final pointer = _pointer;
    if (pointer == null) {
      return;
    }

    final status = destroy(pointer);
    checkNativeStatus(status, diagnostic);
    _pointer = null;
    _detachLeakReporter();
  }

  void _checkOwnerIsolate() {
    if (Isolate.current.hashCode != _ownerIsolateHash) {
      throwWrongThread('$typeName belongs to a different Dart isolate');
    }
  }

  void _attachLeakReporter(Pointer<Void> pointer) {
    final nativeTypeName = typeName.toNativeUtf8().cast<Char>();
    try {
      final token = _createLeakToken(nativeTypeName, pointer);
      if (token == nullptr) {
        return;
      }
      _leakToken = token;
      _leakReporter.attach(this, token, detach: _finalizerDetachToken);
    } finally {
      calloc.free(nativeTypeName);
    }
  }

  void _detachLeakReporter() {
    final token = _leakToken;
    if (token == null) {
      return;
    }
    _leakReporter.detach(_finalizerDetachToken);
    _destroyLeakToken(token);
    _leakToken = null;
  }
}
