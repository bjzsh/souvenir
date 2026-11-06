use crate::interop::souvenir_error;
use souvenir::Error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const ERROR_TYPES: &str = r#"
/** Error thrown for invalid identifiers or inputs. `code` is stable. */
export type SouvenirError = Error & {
    name: "SouvenirError";
    code:
        | "InvalidPrefix"
        | "MissingSeparator"
        | "InvalidLength"
        | "InvalidCharacter"
        | "NonCanonical"
        | "ZeroPrefix"
        | "ReservedPrefix"
        | "PrefixMismatch"
        | "TimestampOutOfRange"
        | "RandomOutOfRange"
        | "NotUuidV7"
        | "RandomSourceFailure"
        | "ClockFailure"
        | "InvalidType";
};

export type SouvenirErrorCode = SouvenirError["code"];
"#;

pub(crate) fn invalid_type(expected: &str) -> JsValue {
    souvenir_error("InvalidType", &format!("Expected {expected}"))
}

pub(crate) fn convert(err: Error) -> JsValue {
    souvenir_error(err.code(), err.message())
}
