use wasm_bindgen::prelude::*;

use souvenir::Generic;

/// A 128-bit prefixed identifier
#[wasm_bindgen]
#[derive(Clone)]
pub struct Id(Generic);

#[wasm_bindgen]
impl Id {
    /// Create an `Id` from an array of bytes
    #[wasm_bindgen(constructor)]
    pub fn new(prefix: String, value: &[u8]) -> Result<Self, JsError> {
        Ok(Self(Generic::new(
            prefix,
            value
                .try_into()
                .map_err(|_| JsError::new("Invalid value for Id!"))?,
        )))
    }

    /// Convert this `Id` to an array of bytes
    #[wasm_bindgen(js_name = toBytes)]
    pub fn to_bytes(&self) -> Box<[u8]> {
        Box::new(self.0.as_bytes().to_owned())
    }

    /// Convert this `Id` to a 128-bit BigInt
    #[wasm_bindgen(js_name = toBigInt)]
    pub fn to_u128(&self) -> u128 {
        self.0.to_u128()
    }

    /// Convert this `Id` to its string representation
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Get the prefix of this `Id`
    pub fn prefix(&self) -> String {
        self.0.prefix().to_owned()
    }
}

#[wasm_bindgen]
impl Id {
    /// Generate a random `Id` with the given prefix
    pub fn random(prefix: String) -> Result<Self, JsError> {
        let mut buf = [0u8; 16];
        getrandom::fill(&mut buf).map_err(|_| JsError::new("Could not generate random bytes"))?;
        Ok(Self(Generic::new(prefix, buf)))
    }

    /// Parse an `Id` from its string representation
    pub fn parse(value: &str) -> Result<Self, JsError> {
        Ok(Self(Generic::parse(value).map_err(|_| {
            JsError::new("Could not parse provided Id")
        })?))
    }

    /// Check if an `Id` string is well-formatted
    pub fn test(value: &str) -> bool {
        Generic::test(value)
    }
}
