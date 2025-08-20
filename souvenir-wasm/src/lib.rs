use souvenir::Error;
use wasm_bindgen::prelude::*;

type Value = souvenir::Id;

fn convert_error(err: Error) -> JsError {
    let message = match err {
        Error::InvalidData => "InvalidData: The provided data is not a valid Id",
        Error::InvalidPrefix => "InvalidPrefix: The provided prefix is not valid",
        Error::InvalidChar { found: _ } => "InvalidChar: The input contains and invalid character",
        Error::InvalidFormat => "InvalidFormat: The input is not in the correct format",
        Error::InvalidLength {
            expected: _,
            found: _,
        } => "InvalidLength: The input has the incorrect length",
    };

    JsError::new(message)
}

/// A 128-bit prefixed identifier
#[wasm_bindgen]
#[derive(Clone)]
pub struct Id(Value);

#[wasm_bindgen]
impl Id {
    /// Create an `Id` from an array of bytes
    #[wasm_bindgen(constructor)]
    pub fn new(value: &[u8]) -> Result<Self, JsError> {
        value
            .try_into()
            .map_err(|_| Error::InvalidLength {
                expected: 16,
                found: value.len(),
            })
            .map(Value::new)
            .flatten()
            .map(|inner| Self(inner))
            .map_err(convert_error)
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

    /// Get the suffix of this `Id`
    pub fn suffix(&self) -> String {
        self.0.suffix().to_owned()
    }

    /// Cast this `Id` to a new prefix
    pub fn cast(&self, prefix: &str) -> Result<Self, JsError> {
        self.0
            .cast(prefix)
            .map(|inner| Self(inner))
            .map_err(convert_error)
    }
}

#[wasm_bindgen]
impl Id {
    /// Generate a random `Id` with the given prefix
    pub fn random(prefix: String) -> Result<Self, JsError> {
        let mut buf = [0u8; 16];
        getrandom::fill(&mut buf)
            .map_err(|_| JsError::new("Error: Could not generate random bytes"))?;
        Value::from_parts(&prefix, buf)
            .map(|inner| Self(inner))
            .map_err(convert_error)
    }

    /// Parse an `Id` from its string representation
    pub fn parse(value: &str) -> Result<Self, JsError> {
        Value::parse(value)
            .map(|inner| Self(inner))
            .map_err(convert_error)
    }

    /// Check if an `Id` string is well-formatted
    pub fn test(value: &str) -> bool {
        Value::test(value)
    }
}
