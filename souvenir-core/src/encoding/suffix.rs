use crate::{
    encoding::{ALPHABET, ALPHABET_INV},
    error::{Error, Result},
};

pub fn encode_suffix(mut raw: u128) -> Result<String> {
    let mut buf = [0; 22];

    for b in buf.iter_mut().rev() {
        *b = ALPHABET[(raw & 0x1f) as usize];
        raw >>= 5;
    }

    String::from_utf8(buf.to_vec()).map_err(|_| Error::InvalidData)
}

pub fn decode_suffix(suffix: &str) -> Result<u128> {
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
}

#[cfg(test)]
mod test {
    use crate::encoding::{decode_suffix, encode_suffix};
    use rand::random;

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
        for _ in 0..100000 {
            let value = random::<u128>() & ((1 << 108) - 1);

            let result = encode_suffix(value).unwrap();
            let parsed = decode_suffix(&result).unwrap();

            assert_eq!(value, parsed);
        }
    }
}
