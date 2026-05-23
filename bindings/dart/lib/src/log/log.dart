/// Log severity mask for asynchronous native log dispatch.
final class LogSeverityMask {
  /// Creates a log severity mask from raw C bits.
  const LogSeverityMask(this.bits);

  /// Info severity bit.
  static const info = LogSeverityMask(1 << 1);

  /// Warning severity bit.
  static const warning = LogSeverityMask(1 << 2);

  /// Error severity bit.
  static const error = LogSeverityMask(1 << 3);

  /// Native default mask: info and warning may be asynchronous.
  static const defaultMask = LogSeverityMask((1 << 1) | (1 << 2));

  /// All known severity bits.
  static const all = LogSeverityMask((1 << 1) | (1 << 2) | (1 << 3));

  /// Raw native mask bits.
  final int bits;

  /// Returns true when all [severity] bits are present in this mask.
  bool contains(LogSeverityMask severity) =>
      (bits & severity.bits) == severity.bits;
}
