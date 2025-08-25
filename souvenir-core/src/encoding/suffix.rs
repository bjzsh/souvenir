use crate::{
    encoding::{ALPHABET, ALPHABET_INV},
    error::{Error, Result},
    suffix::Suffix,
};

pub fn encode_suffix(suffix: Suffix) -> String {
    let mut raw = suffix.to_u128();
    let mut buf = [0; 22];

    for b in buf.iter_mut().rev() {
        *b = ALPHABET[(raw & 0x1f) as usize];
        raw >>= 5;
    }

    // UNSAFE: All bytes are guaranteed to be in ASCII range.
    unsafe { String::from_utf8_unchecked(buf.to_vec()) }
}

pub fn decode_suffix(suffix: &str) -> Result<Suffix> {
    if suffix.len() != 22 {
        return Err(Error::InvalidLength {
            expected: 22,
            found: suffix.len(),
        });
    }

    suffix
        .as_bytes()
        .iter()
        .enumerate()
        .try_fold(0u128, |acc, (i, &ch)| {
            let value = ALPHABET_INV[ch as usize];

            if value == 0xff || (i == 0 && value > 7) {
                return Err(Error::InvalidChar { found: ch as char });
            }

            Ok((acc << 5) | value as u128)
        })
        .map(Suffix::new)
}

#[cfg(test)]
mod test {
    use crate::{
        encoding::{decode_suffix, encode_suffix},
        suffix::Suffix,
    };
    use rand::random;

    #[test]
    fn decode_smoke() {
        assert_eq!(
            Ok(0x0110c8531d0952d8d73e1194e95),
            decode_suffix("0123456789abcdefghjkmn").map(Suffix::to_u128)
        );

        assert_eq!(
            Ok(0x000000000000000000000000000),
            decode_suffix("0000000000000000000000").map(Suffix::to_u128)
        );

        assert_eq!(
            Ok(0xfffffffffffffffffffffffffff),
            decode_suffix("7zzzzzzzzzzzzzzzzzzzzz").map(Suffix::to_u128)
        );
    }

    #[test]
    fn round_trip() {
        for _ in 0..100000 {
            let value = random::<u128>() & ((1 << 108) - 1);

            let result = encode_suffix(value.into());
            let parsed = decode_suffix(&result).unwrap();

            assert_eq!(value, parsed.into());
        }
    }
}
