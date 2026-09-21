use std::ops::RangeInclusive;

use crate::{
    encoding::{ALPHABET, ALPHABET_INV, Q, R},
    error::{Error, Result},
    prefix::Prefix,
};

const SUFFIX_LENGTH: usize = 22;

pub fn suffix_interval(prefix: Prefix) -> RangeInclusive<u128> {
    let max = if (prefix.to_u32() as u128) < R {
        Q
    } else {
        Q - 1
    };

    0..=max
}

/// Encode 22 base-32 digits, rejecting values that would be truncated.
/// Prefix-specific bounds are checked by Id::from_parts.
pub fn encode_suffix(suffix: u128) -> Result<String> {
    if suffix >= 1u128 << 110 {
        return Err(Error::InvalidData);
    }

    Ok(encode_suffix_digits(suffix))
}

/// Format a suffix already known to fit in 22 digits. Only call after checking
/// the input range or obtaining the suffix from a valid Id.
pub fn encode_suffix_digits(suffix: u128) -> String {
    let mut raw = suffix;
    let mut buf = [0; 22];

    for b in buf.iter_mut().rev() {
        *b = ALPHABET[(raw & 0x1f) as usize];
        raw >>= 5;
    }

    // UNSAFE: All bytes are guaranteed to be in ASCII range.
    unsafe { String::from_utf8_unchecked(buf.to_vec()) }
}

pub fn decode_suffix(suffix: &str) -> Result<u128> {
    if suffix.len() != SUFFIX_LENGTH {
        return Err(Error::InvalidLength {
            expected: SUFFIX_LENGTH,
            found: suffix.len(),
        });
    }

    suffix.chars().try_fold(0u128, |acc, ch| {
        if !ch.is_ascii() {
            return Err(Error::InvalidChar { found: ch });
        }

        let value = ALPHABET_INV[ch as usize];

        if value == 0xff {
            return Err(Error::InvalidChar { found: ch });
        }

        Ok((acc << 5) | value as u128)
    })
}

#[cfg(test)]
mod test {
    use crate::encoding::{decode_suffix, encode_suffix};

    #[test]
    fn unicode_errors_report_characters_and_byte_lengths() {
        for ch in ['é', '€', '🦀'] {
            let suffix = format!("0{ch}{}", "0".repeat(21 - ch.len_utf8()));
            assert_eq!(
                decode_suffix(&suffix),
                Err(crate::error::Error::InvalidChar { found: ch })
            );
        }
        assert_eq!(
            decode_suffix("é"),
            Err(crate::error::Error::InvalidLength {
                expected: 22,
                found: 2
            })
        );
    }

    #[test]
    fn decode_smoke() {
        assert_eq!(
            Ok(0x0110c8531d0952d8d73e1194e95),
            decode_suffix("0123456789abcdefghjkmn")
        );

        assert_eq!(
            Ok(0x000000000000000000000000000),
            decode_suffix("0000000000000000000000")
        );

        assert_eq!(
            Ok(0xfffffffffffffffffffffffffff),
            decode_suffix("7zzzzzzzzzzzzzzzzzzzzz")
        );
    }

    #[test]
    fn round_trip() {
        for i in 0..100_000u128 {
            let value = i * (((1u128 << 110) - 1) / 99_999);

            let result = encode_suffix(value).unwrap();
            let parsed = decode_suffix(&result).unwrap();

            assert_eq!(value, parsed);
        }
    }
}
