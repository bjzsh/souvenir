//! JavaScript boundary helpers for errors and checked BigInt conversion.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function souvenirError(code, message, expected, found) {
    const error = new Error(message);
    error.name = 'SouvenirError';
    error.code = code;
    if (expected !== undefined) error.expected = expected;
    if (found !== undefined) error.found = found;
    return error;
}

const typedArrayTag = Object.getOwnPropertyDescriptor(
    Object.getPrototypeOf(Uint8Array.prototype), Symbol.toStringTag
).get;

export function checkedBytes(value) {
    if (!ArrayBuffer.isView(value) || typedArrayTag.call(value) !== 'Uint8Array') {
        throw souvenirError('InvalidType', 'Expected a Uint8Array');
    }
    return value;
}

export function bigintDecimal(value) {
    if (typeof value !== 'bigint') {
        throw souvenirError('InvalidType', 'Expected a bigint');
    }
    if (value < 0n || value > (1n << 128n) - 1n) {
        throw souvenirError('InvalidRange', 'Expected an unsigned 128-bit bigint');
    }
    return value.toString();
}
")]
extern "C" {
    #[wasm_bindgen(typescript_type = "string")]
    pub type StringInput;

    #[wasm_bindgen(typescript_type = "Uint8Array")]
    pub type BytesInput;

    #[wasm_bindgen(typescript_type = "unknown")]
    pub type UnknownInput;

    #[wasm_bindgen(catch, js_name = checkedBytes)]
    pub(crate) fn checked_bytes(value: &BytesInput) -> Result<Vec<u8>, JsValue>;

    #[wasm_bindgen(typescript_type = "bigint")]
    pub type BigIntInput;

    #[wasm_bindgen(js_name = souvenirError)]
    pub(crate) fn souvenir_error(
        code: &str,
        message: &str,
        expected: JsValue,
        found: JsValue,
    ) -> JsValue;

    #[wasm_bindgen(catch, js_name = bigintDecimal)]
    pub(crate) fn bigint_decimal(value: &BigIntInput) -> Result<String, JsValue>;
}
