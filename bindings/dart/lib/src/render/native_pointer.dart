/// Opaque borrowed native address value used for backend interop handles.
///
/// The value does not own, retain, dereference, or validate the pointed-to
/// object. Passing it to MapLibre Native transfers no ownership and grants the
/// Dart binding no memory access.
final class NativePointer {
  /// Creates an opaque borrowed pointer value from an address bit pattern.
  const NativePointer(this.address);

  /// Null native pointer value.
  static const nullPointer = NativePointer(0);

  /// Address bit pattern for this borrowed native pointer.
  final int address;

  /// Returns true when this pointer represents a null backend handle.
  bool get isNull => address == 0;

  @override
  String toString() => 'NativePointer[address=0x${address.toRadixString(16)}]';
}
