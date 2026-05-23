use napi::{Error as NapiError, Status};

pub(crate) fn from_core(error: maplibre_native_core::Error) -> NapiError {
    let kind = match error.kind() {
        maplibre_native_core::ErrorKind::InvalidArgument => "InvalidArgument",
        maplibre_native_core::ErrorKind::InvalidState => "InvalidState",
        maplibre_native_core::ErrorKind::WrongThread => "WrongThread",
        maplibre_native_core::ErrorKind::Unsupported => "Unsupported",
        maplibre_native_core::ErrorKind::NativeError => "NativeError",
        maplibre_native_core::ErrorKind::AbiVersionMismatch => "AbiVersionMismatch",
        maplibre_native_core::ErrorKind::UnknownStatus => "UnknownStatus",
        _ => "UnknownStatus",
    };
    let raw_status = error
        .raw_status()
        .map_or_else(|| "none".to_owned(), |status| status.to_string());
    NapiError::new(
        Status::GenericFailure,
        format!("{kind} ({raw_status}): {}", error.diagnostic()),
    )
}

pub(crate) fn invalid_argument(message: impl Into<String>) -> NapiError {
    NapiError::new(Status::InvalidArg, message.into())
}
