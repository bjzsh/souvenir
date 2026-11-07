use crate::error::{convert, invalid_type};
use crate::interop::StringInput;
use souvenir::{Prefix, Souvenir};
use wasm_bindgen::JsValue;

/// The result of an exported function: errors are thrown `SouvenirError`s.
pub(crate) type JsResult<T> = Result<T, JsValue>;

/// A string argument, or an `InvalidType` error.
pub(crate) fn string(value: &JsValue) -> JsResult<String> {
    value.as_string().ok_or_else(|| invalid_type("a string"))
}

/// A prefix argument.
pub(crate) fn prefix(value: &StringInput) -> JsResult<Prefix> {
    Prefix::parse(&string(value)?).map_err(convert)
}

/// An ID argument.
pub(crate) fn id(value: &StringInput) -> JsResult<Souvenir> {
    Souvenir::parse(&string(value)?).map_err(convert)
}

/// An ID result as a JavaScript string, or a thrown `SouvenirError`.
pub(crate) fn id_string(result: souvenir::Result<Souvenir>) -> JsResult<String> {
    result.map(|id| id.to_string()).map_err(convert)
}
