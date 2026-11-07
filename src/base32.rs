use crate::error::{Error, Result};
use crate::spec::{ALPHABET, BODY_BITS, SUFFIX_LEN};

/// Marks a byte outside the alphabet in [`DECODE`].
const INVALID: u8 = 0xFF;

/// Maps an ASCII byte to its 5-bit suffix value, or [`INVALID`].
const DECODE: [u8; 256] = {
    let mut table = [INVALID; 256];
    let mut i = 0;
    while i < ALPHABET.len() {
        table[ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    table
};

/// Encode a 122-bit body as the 25-character suffix (SPEC.md section 5), most
/// significant first, into the first 25 bytes of `out`.
pub(crate) const fn encode(body: u128, out: &mut [u8]) {
    let mut i = 0;
    while i < SUFFIX_LEN {
        let shift = 5 * (SUFFIX_LEN - 1 - i);
        out[i] = ALPHABET[((body >> shift) & 31) as usize];
        i += 1;
    }
}

/// Decode a 25-character suffix into a 122-bit body (SPEC.md section 5).
pub(crate) const fn decode(suffix: &[u8]) -> Result<u128> {
    if suffix.len() != SUFFIX_LEN {
        return Err(Error::InvalidLength);
    }
    let mut value = 0u128;
    let mut i = 0;
    while i < SUFFIX_LEN {
        let digit = DECODE[suffix[i] as usize];
        if digit == INVALID {
            return Err(Error::InvalidCharacter);
        }
        value = (value << 5) | digit as u128;
        i += 1;
    }
    if value >> BODY_BITS != 0 {
        return Err(Error::NonCanonical);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::BODY_MASK;

    #[test]
    fn body_codec_edges() {
        let mut out = [0u8; SUFFIX_LEN];
        encode(BODY_MASK, &mut out);
        assert_eq!(&out, b"3zzzzzzzzzzzzzzzzzzzzzzzz");
        assert_eq!(decode(&out), Ok(BODY_MASK));
        assert_eq!(
            decode(b"4000000000000000000000000"),
            Err(Error::NonCanonical)
        );
    }
}
