//! Public error declarations and conversion from Rust errors.

use souvenir::Error;
use wasm_bindgen::prelude::*;

use crate::interop::souvenir_error;

#[wasm_bindgen(typescript_custom_section)]
const ERROR_TYPES: &str = r#"
/** Error thrown for invalid identifiers, conversion inputs, or unavailable randomness. */
export type SouvenirError = Error & { name: "SouvenirError" } & (
    | { code: "InvalidLength"; expected: number; found: number }
    | { code: "InvalidChar"; found: string }
    | { code: "InvalidData" }
    | { code: "InvalidPrefix" }
    | { code: "InvalidFormat" }
    | { code: "InvalidType" }
    | { code: "InvalidRange" }
    | { code: "RandomnessUnavailable" }
);

export type SouvenirErrorCode = SouvenirError["code"];
"#;

pub(crate) fn error(code: &str, message: &str) -> JsValue {
    souvenir_error(code, message, JsValue::UNDEFINED, JsValue::UNDEFINED)
}

pub(crate) fn convert_error(err: Error) -> JsValue {
    let (code, expected, found) = match &err {
        Error::InvalidData => ("InvalidData", JsValue::UNDEFINED, JsValue::UNDEFINED),
        Error::InvalidPrefix => ("InvalidPrefix", JsValue::UNDEFINED, JsValue::UNDEFINED),
        Error::InvalidFormat => ("InvalidFormat", JsValue::UNDEFINED, JsValue::UNDEFINED),
        Error::InvalidChar { found } => (
            "InvalidChar",
            JsValue::UNDEFINED,
            JsValue::from_str(&found.to_string()),
        ),
        Error::InvalidLength { expected, found } => (
            "InvalidLength",
            JsValue::from_f64(*expected as f64),
            JsValue::from_f64(*found as f64),
        ),
    };
    souvenir_error(code, &err.message(), expected, found)
}
