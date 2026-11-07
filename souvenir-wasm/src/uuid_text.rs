// Strict and with Souvenir's error codes, unlike `Uuid::parse_str`.

use souvenir::uuid::Uuid;
use souvenir::{Error, Result};

const UUID_LEN: usize = 16;

/// Length of the hyphenated UUID text form.
const UUID_TEXT_LEN: usize = 36;

const HYPHENS: [usize; 4] = [8, 13, 18, 23];

/// Parse `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`, in either case.
pub(crate) fn parse_hyphenated(text: &str) -> Result<Uuid> {
    let text = text.as_bytes();
    if text.len() != UUID_TEXT_LEN {
        return Err(Error::InvalidLength);
    }
    let mut out = [0u8; UUID_LEN];
    let mut pos = 0;
    for byte in &mut out {
        if HYPHENS.contains(&pos) {
            if text[pos] != b'-' {
                return Err(Error::InvalidCharacter);
            }
            pos += 1;
        }
        *byte = (hex_digit(text[pos])? << 4) | hex_digit(text[pos + 1])?;
        pos += 2;
    }
    Ok(Uuid::from_bytes(out))
}

/// Parse a hex digit (either case).
fn hex_digit(c: u8) -> Result<u8> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(Error::InvalidCharacter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let uuid = Uuid::from_bytes([
            0x01, 0xa1, 0x96, 0xd0, 0x0e, 0x60, 0x7a, 0x97, 0xb3, 0xc9, 0xe1, 0xb7, 0xd0, 0x4a,
            0x6c, 0x8e,
        ]);
        assert_eq!(
            parse_hyphenated("01a196d0-0e60-7a97-b3c9-e1b7d04a6c8e"),
            Ok(uuid)
        );
        assert_eq!(
            parse_hyphenated("01A196D0-0E60-7A97-B3C9-E1B7D04A6C8E"),
            Ok(uuid)
        );
        assert_eq!(parse_hyphenated("01a196d0"), Err(Error::InvalidLength));
        assert_eq!(
            parse_hyphenated("01a196d0x0e60-7a97-b3c9-e1b7d04a6c8e"),
            Err(Error::InvalidCharacter)
        );
        assert_eq!(
            parse_hyphenated("01a196d0-0e60-7a97-b3c9-e1b7d04a6c8g"),
            Err(Error::InvalidCharacter)
        );
    }
}
