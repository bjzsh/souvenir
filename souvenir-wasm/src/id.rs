//! Immutable JavaScript identifier API.

use souvenir::{Error, Prefix, Suffix};
use wasm_bindgen::prelude::*;

use crate::error::{convert_error, error};
use crate::interop::{
    BigIntInput, BytesInput, StringInput, UnknownInput, bigint_decimal, checked_bytes,
};

type Value = souvenir::Id;

/// An immutable 128-bit identifier with a prefix and a 108-bit suffix.
#[wasm_bindgen]
#[derive(Clone)]
pub struct Id(Value);

#[wasm_bindgen]
impl Id {
    /// Create an identifier from exactly 16 bytes in big-endian order.
    #[wasm_bindgen(constructor)]
    pub fn new(value: &BytesInput) -> Result<Self, JsValue> {
        Self::from_bytes(value)
    }

    /// Create an identifier from exactly 16 bytes in big-endian order.
    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(value: &BytesInput) -> Result<Self, JsValue> {
        let value = checked_bytes(value)?;
        value
            .as_slice()
            .try_into()
            .map_err(|_| Error::InvalidLength {
                expected: 16,
                found: value.len(),
            })
            .and_then(Value::from_bytes)
            .map(Self)
            .map_err(convert_error)
    }

    /// Create an identifier from an unsigned 128-bit bigint, validating its prefix.
    /// Negative and out-of-range inputs are rejected rather than truncated.
    #[wasm_bindgen(js_name = fromBigInt)]
    pub fn from_big_int(value: &BigIntInput) -> Result<Self, JsValue> {
        let value = bigint_decimal(value)?
            .parse::<u128>()
            .map_err(|_| error("InvalidRange", "Expected an unsigned 128-bit bigint"))?;
        Value::try_from(value).map(Self).map_err(convert_error)
    }

    /// Convert this identifier to 16 bytes in big-endian order.
    #[wasm_bindgen(js_name = toBytes)]
    pub fn to_bytes(&self) -> Box<[u8]> {
        Box::new(self.0.as_bytes().to_owned())
    }

    /// Convert this identifier to an unsigned 128-bit bigint.
    #[wasm_bindgen(js_name = toBigInt)]
    pub fn to_u128(&self) -> u128 {
        self.0.to_u128()
    }

    /// Return the canonical identifier string.
    #[wasm_bindgen(js_name = toString)]
    pub fn wasm_to_string(&self) -> String {
        self.0.to_string()
    }

    /// Serialize as the canonical identifier string in JSON.stringify().
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> String {
        self.0.to_string()
    }

    /// Compare identifier values, including both prefix and suffix.
    pub fn equals(&self, other: &Id) -> bool {
        self.0 == other.0
    }

    /// The identifier's prefix.
    #[wasm_bindgen(getter)]
    pub fn prefix(&self) -> String {
        self.0.prefix().to_string()
    }

    /// The identifier's encoded suffix.
    #[wasm_bindgen(getter)]
    pub fn suffix(&self) -> String {
        self.0.suffix().to_string()
    }

    /// Return a new identifier with a different prefix and the same suffix.
    pub fn cast(&self, prefix: &StringInput) -> Result<Self, JsValue> {
        Prefix::parse(&string_input(prefix)?)
            .map(|prefix| self.0.cast(prefix))
            .map(Self)
            .map_err(convert_error)
    }

    /// Generate a random identifier with the given prefix.
    pub fn random(prefix: &StringInput) -> Result<Self, JsValue> {
        let prefix = Prefix::parse(&string_input(prefix)?).map_err(convert_error)?;
        let mut buf = [0u8; 16];
        getrandom::fill(&mut buf)
            .map_err(|_| error("RandomnessUnavailable", "Could not generate random bytes"))?;
        Ok(Self(Value::new(
            prefix,
            Suffix::new(u128::from_be_bytes(buf)),
        )))
    }

    /// Parse an identifier string, throwing a SouvenirError for invalid input.
    pub fn parse(value: &StringInput) -> Result<Self, JsValue> {
        Value::parse(&string_input(value)?)
            .map(Self)
            .map_err(convert_error)
    }

    /// Parse an unknown value, returning undefined for non-strings or invalid identifiers.
    #[wasm_bindgen(js_name = tryParse)]
    pub fn try_parse(value: &UnknownInput) -> Option<Self> {
        Value::parse(&value.as_string()?).ok().map(Self)
    }

    /// Check whether an unknown value is a valid identifier string.
    pub fn test(value: &UnknownInput) -> bool {
        value.as_string().is_some_and(|value| Value::test(&value))
    }
}

fn string_input(value: &StringInput) -> Result<String, JsValue> {
    value
        .as_string()
        .ok_or_else(|| error("InvalidType", "Expected a string"))
}
