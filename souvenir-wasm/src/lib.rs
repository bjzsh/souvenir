#![forbid(unsafe_code)]
// Library code never panics at run time: no explicit panics, and no
// `unwrap`/`expect`. (Panics in `const` evaluation, such as an invalid
// `prefix!` literal, are compile errors and are fine.)
#![cfg_attr(
    not(test),
    deny(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]
#![warn(missing_docs)]

//! JavaScript bindings for Souvenir IDs.
//!
//! IDs cross the boundary as their canonical strings, so there are no object
//! handles to free. The hand-written TypeScript wrapper in `ts/` adds
//! branded types and `typed()`.

mod error;
mod generators;
mod interop;
mod marshal;
mod uuid_text;

use error::{convert, invalid_type};
use generators::{MONOTONIC, random_generator};
use interop::{
    BigIntInput, BytesInput, NumberInput, StringInput, UnknownInput, UuidInput, checked_bytes,
    checked_random, checked_timestamp, is_uint8_array,
};
use marshal::{JsResult, id, id_string, prefix, string};
use souvenir::uuid::Uuid;
use souvenir::{Error, Generator, Prefix, Souvenir};
use std::sync::PoisonError;
use uuid_text::parse_hyphenated;
use wasm_bindgen::prelude::*;

/// Generate a new ID with the given prefix.
#[wasm_bindgen(js_name = newId)]
pub fn new_id(prefix_text: &StringInput) -> JsResult<String> {
    id_string(random_generator().generate_untyped(prefix(prefix_text)?))
}

/// Generate a new ID from the module-wide monotonic generator: each result
/// is greater than the previous one, even within one millisecond.
#[wasm_bindgen(js_name = newMonotonicId)]
pub fn new_monotonic_id(prefix_text: &StringInput) -> JsResult<String> {
    let prefix = prefix(prefix_text)?;
    // A panic can't leave the generator half-updated, so a poisoned lock is
    // still safe to use.
    let mut generator = MONOTONIC.lock().unwrap_or_else(PoisonError::into_inner);
    id_string(generator.generate_untyped(prefix))
}

/// Validate an ID string and return it.
#[wasm_bindgen]
pub fn parse(text: &StringInput) -> JsResult<String> {
    id(text).map(|id| id.to_string())
}

/// Validate an ID string with the given prefix and return it. An invalid
/// `prefix` fails with `InvalidPrefix` before `text` is looked at.
#[wasm_bindgen(js_name = parseExpecting)]
pub fn parse_expecting(prefix_text: &StringInput, text: &StringInput) -> JsResult<String> {
    let prefix = prefix(prefix_text)?;
    let text = string(text)?;
    id_string(Souvenir::parse_expecting(prefix, &text))
}

/// Check that `prefix` is 1-8 characters of a-z, throwing `InvalidPrefix`
/// otherwise.
#[wasm_bindgen(js_name = validatePrefix)]
pub fn validate_prefix(prefix_text: &StringInput) -> JsResult<()> {
    prefix(prefix_text).map(|_| ())
}

/// Whether `value` is a valid ID string (with the given prefix, if any).
/// Never throws.
#[wasm_bindgen(js_name = isValid)]
pub fn is_valid(value: &UnknownInput, prefix: Option<String>) -> bool {
    let Some(text) = value.as_string() else {
        return false;
    };
    match prefix {
        Some(prefix) => Prefix::parse(&prefix)
            .and_then(|prefix| Souvenir::parse_expecting(prefix, &text))
            .is_ok(),
        None => Souvenir::is_valid(&text),
    }
}

/// Build an ID from a prefix, a timestamp in Unix milliseconds (a safe
/// integer) and a 74-bit random bigint.
#[wasm_bindgen(js_name = fromParts)]
pub fn from_parts(
    prefix_text: &StringInput,
    timestamp_ms: &NumberInput,
    random: &BigIntInput,
) -> JsResult<String> {
    let prefix = prefix(prefix_text)?;
    let timestamp_ms = checked_timestamp(timestamp_ms)? as u64;
    let random = checked_random(random)?
        .parse::<u128>()
        .map_err(|_| convert(Error::RandomOutOfRange))?;
    id_string(Souvenir::from_parts(prefix, timestamp_ms, random))
}

/// The prefix of an ID string.
#[wasm_bindgen(js_name = prefixOf)]
pub fn prefix_of(id_text: &StringInput) -> JsResult<String> {
    id(id_text).map(|id| id.prefix().to_string())
}

/// The creation time of an ID, in Unix milliseconds.
#[wasm_bindgen(js_name = timestampMs)]
pub fn timestamp_ms(id_text: &StringInput) -> JsResult<f64> {
    id(id_text).map(|id| id.timestamp_ms() as f64)
}

/// The 74-bit random field of an ID.
#[wasm_bindgen(js_name = randomOf)]
pub fn random_of(id_text: &StringInput) -> JsResult<u128> {
    id(id_text).map(|id| id.random())
}

/// The 20-byte binary form of an ID.
#[wasm_bindgen(js_name = toBytes)]
pub fn to_bytes(id_text: &StringInput) -> JsResult<Box<[u8]>> {
    id(id_text).map(|id| Box::new(id.to_bytes()) as Box<[u8]>)
}

/// Decode the 20-byte binary form. All-zero bytes throw `ZeroPrefix`.
#[wasm_bindgen(js_name = fromBytes)]
pub fn from_bytes(bytes: &BytesInput) -> JsResult<String> {
    let bytes = checked_bytes(bytes)?;
    id_string(Souvenir::from_slice(&bytes))
}

/// The UUIDv7 form of an ID, as lowercase hyphenated text. The prefix is
/// dropped; pass it back to `fromUuid`.
#[wasm_bindgen(js_name = toUuid)]
pub fn to_uuid(id_text: &StringInput) -> JsResult<String> {
    id(id_text).map(|id| id.to_uuid().to_string())
}

/// Rebuild an ID from a prefix and a UUIDv7 (hyphenated text or 16 bytes).
#[wasm_bindgen(js_name = fromUuid)]
pub fn from_uuid(prefix_text: &StringInput, value: &UuidInput) -> JsResult<String> {
    let prefix = prefix(prefix_text)?;
    let uuid = if let Some(text) = value.as_string() {
        parse_hyphenated(&text).map_err(convert)?
    } else if is_uint8_array(value) {
        Uuid::from_slice(&checked_bytes(value)?).map_err(|_| convert(Error::InvalidLength))?
    } else {
        return Err(invalid_type("a string or Uint8Array"));
    };
    id_string(Souvenir::from_uuid(prefix, uuid))
}
