/// JSON value types used by style, query, feature state, and descriptors.
library;

/// Ordered JSON object member. Duplicate keys are preserved.
final class JsonMember {
  /// Creates a JSON object member.
  const JsonMember(this.key, this.value);

  /// Member key.
  final String key;

  /// Member value.
  final JsonValue value;
}

/// Owned JSON-like value tree used by style, GeoJSON, and copied native values.
sealed class JsonValue {
  const JsonValue();
}

/// JSON null.
final class JsonNull extends JsonValue {
  /// Creates JSON null.
  const JsonNull();
}

/// JSON boolean.
final class JsonBool extends JsonValue {
  /// Creates a JSON boolean.
  const JsonBool(this.value);

  /// Boolean value.
  final bool value;
}

/// JSON unsigned integer.
final class JsonUInt extends JsonValue {
  /// Creates a JSON unsigned integer in the supported unsigned 63-bit subset.
  const JsonUInt(this.value);

  /// Integer value in the supported unsigned 63-bit subset.
  final int value;
}

/// JSON signed integer.
final class JsonInt extends JsonValue {
  /// Creates a JSON signed integer.
  const JsonInt(this.value);

  /// Integer value.
  final int value;
}

/// JSON double.
final class JsonDouble extends JsonValue {
  /// Creates a JSON double.
  const JsonDouble(this.value);

  /// Double value.
  final double value;
}

/// JSON string.
final class JsonString extends JsonValue {
  /// Creates a JSON string.
  const JsonString(this.value);

  /// String value.
  final String value;
}

/// JSON array.
final class JsonArray extends JsonValue {
  /// Creates a JSON array.
  const JsonArray(this.values);

  /// Array values.
  final List<JsonValue> values;
}

/// JSON object.
final class JsonObject extends JsonValue {
  /// Creates a JSON object.
  const JsonObject(this.members);

  /// Ordered object members.
  final List<JsonMember> members;
}
