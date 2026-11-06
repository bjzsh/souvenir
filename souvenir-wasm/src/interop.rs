use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function souvenirError(code, message) {
    const error = new Error(message);
    error.name = 'SouvenirError';
    error.code = code;
    return error;
}

const typedArrayTag = Object.getOwnPropertyDescriptor(
    Object.getPrototypeOf(Uint8Array.prototype), Symbol.toStringTag
).get;

export function isUint8Array(value) {
    return ArrayBuffer.isView(value) && typedArrayTag.call(value) === 'Uint8Array';
}

export function checkedBytes(value) {
    if (!isUint8Array(value)) {
        throw souvenirError('InvalidType', 'Expected a Uint8Array');
    }
    return value;
}

export function checkedTimestamp(value) {
    if (typeof value !== 'number') {
        throw souvenirError('InvalidType', 'Expected a number');
    }
    if (!Number.isSafeInteger(value) || value < 0 || value > 2 ** 48 - 1) {
        throw souvenirError('TimestampOutOfRange', 'Timestamp must be an integer in [0, 2^48)');
    }
    return value;
}

export function checkedRandom(value) {
    if (typeof value !== 'bigint') {
        throw souvenirError('InvalidType', 'Expected a bigint');
    }
    if (value < 0n || value > (1n << 74n) - 1n) {
        throw souvenirError('RandomOutOfRange', 'Random value must be in [0, 2^74)');
    }
    return value.toString();
}

export function now() {
    return Date.now();
}
")]
extern "C" {
    #[wasm_bindgen(typescript_type = "string")]
    pub type StringInput;

    #[wasm_bindgen(typescript_type = "Uint8Array")]
    pub type BytesInput;

    #[wasm_bindgen(typescript_type = "string | Uint8Array")]
    pub type UuidInput;

    #[wasm_bindgen(typescript_type = "number")]
    pub type NumberInput;

    #[wasm_bindgen(typescript_type = "bigint")]
    pub type BigIntInput;

    #[wasm_bindgen(typescript_type = "unknown")]
    pub type UnknownInput;

    #[wasm_bindgen(js_name = souvenirError)]
    pub(crate) fn souvenir_error(code: &str, message: &str) -> JsValue;

    #[wasm_bindgen(js_name = isUint8Array)]
    pub(crate) fn is_uint8_array(value: &JsValue) -> bool;

    #[wasm_bindgen(catch, js_name = checkedBytes)]
    pub(crate) fn checked_bytes(value: &JsValue) -> Result<Vec<u8>, JsValue>;

    #[wasm_bindgen(catch, js_name = checkedTimestamp)]
    pub(crate) fn checked_timestamp(value: &NumberInput) -> Result<f64, JsValue>;

    #[wasm_bindgen(catch, js_name = checkedRandom)]
    pub(crate) fn checked_random(value: &BigIntInput) -> Result<String, JsValue>;

    pub(crate) fn now() -> f64;
}
