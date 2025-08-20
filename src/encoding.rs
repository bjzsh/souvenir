use crate::Error;

const PREFIX: &[u8; 27] = b"\0abcdefghijklmnopqrstuvwxyz";
const PREFIX_INV: &[u8; 256] = &{
    let mut output = [255; 256];

    let mut i = 0;
    while i < 27 {
        output[PREFIX[i as usize] as usize] = i;
        i += 1;
    }

    output[PREFIX[0] as usize] = 255;
    output
};

const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
const ALPHABET_INV: &[u8; 256] = &{
    let mut output = [255; 256];

    let mut i = 0;
    while i < 32 {
        output[ALPHABET[i as usize] as usize] = i;
        i += 1;
    }

    output
};

pub fn decode_id(id: &str) -> Result<[u8; 16], Error> {
    let (prefix, suffix) = id.rsplit_once('_').ok_or(Error::InvalidFormat)?;
    let left = decode_prefix(prefix)?;
    let right = decode_suffix(suffix)?;
    Ok((left | right).to_be_bytes())
}

pub fn decode_prefix(prefix: &str) -> Result<u128, Error> {
    if prefix.is_empty() || prefix.len() > 4 {
        return Err(Error::InvalidPrefix);
    }

    let mut prefix = prefix.as_bytes().to_vec();
    prefix.resize(4, 0);

    let mut max = 0;

    for b in &mut prefix {
        if *b == 0 {
            continue;
        }

        *b = PREFIX_INV[*b as usize];
        max |= *b;
    }

    if max == 0xff {
        return Err(Error::InvalidPrefix);
    }

    let mut out = 0u128;
    for b in prefix {
        out <<= 5;
        out |= b as u128;
    }

    Ok(out << 108)
}

pub fn decode_suffix(str: &str) -> Result<u128, Error> {
    let mut suffix: [u8; 22] = str
        .as_bytes()
        .try_into()
        .map_err(|_| Error::InvalidLength {
            expected: 22,
            found: str.len(),
        })?;

    for b in &mut suffix {
        let index = ALPHABET_INV[*b as usize];

        if index == 0xff {
            return Err(Error::InvalidChar { found: *b as char });
        }

        *b = index;
    }

    if suffix[0] > 7 {
        return Err(Error::InvalidChar {
            found: str.chars().next().unwrap(),
        });
    }

    let mut out = 0u128;
    for b in suffix {
        out <<= 5;
        out |= b as u128;
    }

    Ok(out)
}

pub fn encode_id(bytes: [u8; 16]) -> Result<String, Error> {
    let raw = u128::from_be_bytes(bytes);
    let prefix = encode_prefix(raw)?;
    let suffix = encode_suffix(raw)?;
    Ok(format!("{}_{}", prefix, suffix))
}

pub fn encode_prefix(raw: u128) -> Result<String, Error> {
    let mut data = raw >> 108;
    let mut buf = [0; 4];

    let mut chars = 0;
    let mut can_skip = true;

    for b in buf.iter_mut().rev() {
        let part = data & 0x1f;
        data >>= 5;

        if part == 0 && can_skip {
            continue;
        }

        if part == 0 || part > 26 {
            return Err(Error::InvalidData);
        }

        *b = PREFIX[part as usize];
        debug_assert!(b.is_ascii());

        can_skip = false;
        chars += 1;
    }

    if can_skip {
        return Err(Error::InvalidData);
    }

    String::from_utf8(buf[..chars].to_vec()).map_err(|_| Error::InvalidData)
}

pub fn encode_suffix(raw: u128) -> Result<String, Error> {
    let mut data = raw & ((1 << 108) - 1);
    let mut buf = [0; 22];

    for b in buf.iter_mut().rev() {
        *b = ALPHABET[((data as u8) & 0x1f) as usize];
        debug_assert!(b.is_ascii());
        data >>= 5;
    }

    String::from_utf8(buf.to_vec()).map_err(|_| Error::InvalidData)
}

#[cfg(test)]
mod test {
    use crate::encoding::{
        decode_id, decode_prefix, decode_suffix, encode_id, encode_prefix, encode_suffix,
    };
    use rand::random;

    #[test]
    fn prefix_decode() {
        assert_eq!(
            0b10101_10011_00101_10010,
            decode_prefix("user").unwrap() >> 108
        );

        assert_eq!(
            0b10101_10011_00101_00000,
            decode_prefix("use").unwrap() >> 108
        );

        assert_eq!(
            0b10101_10011_00000_00000,
            decode_prefix("us").unwrap() >> 108
        );

        assert_eq!(
            0b10101_00000_00000_00000,
            decode_prefix("u").unwrap() >> 108
        );
    }

    #[test]
    fn prefix_encode() {
        assert_eq!(
            "user",
            encode_prefix(0b10101_10011_00101_10010 << 108).unwrap(),
        );

        assert_eq!(
            "use",
            encode_prefix(0b10101_10011_00101_00000 << 108).unwrap(),
        );

        assert_eq!(
            "us",
            encode_prefix(0b10101_10011_00000_00000 << 108).unwrap(),
        );

        assert_eq!(
            "u",
            encode_prefix(0b10101_00000_00000_00000 << 108).unwrap(),
        );
    }

    #[test]
    fn suffix_roundtrip() {
        for _ in 0..100000 {
            let value: u128 = random();

            let result = encode_suffix(value).unwrap();
            let parsed = decode_suffix(&result).unwrap();

            assert_eq!(value & ((1 << 108) - 1), parsed);
        }
    }

    #[test]
    fn encode_one() {
        let bytes: [u8; 16] = [
            172, 203, 32, 45, 149, 12, 42, 134, 255, 12, 19, 41, 129, 124, 106, 4,
        ];

        assert_eq!("user_02v58c5a3fy30k560qrtg4", encode_id(bytes).unwrap())
    }

    #[test]
    fn decode_random() {
        assert_eq!(
            [
                0x08, 0x86, 0x00, 0x11, 0x0c, 0x85, 0x31, 0xd0, 0x95, 0x2d, 0x8d, 0x73, 0xe1, 0x19,
                0x4e, 0x95
            ],
            decode_id("abc_0123456789abcdefghjkmn").unwrap()
        );
    }

    #[test]
    fn decode_min() {
        assert_eq!(
            [
                0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ],
            decode_id("a_0000000000000000000000").unwrap()
        );
    }

    #[test]
    fn decode_max() {
        assert_eq!(
            [
                0xd6, 0xb5, 0xaf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff
            ],
            decode_id("zzzz_7zzzzzzzzzzzzzzzzzzzzz").unwrap()
        );
    }

    #[test]
    fn decode_invalid() {
        decode_id("80000000000000000000000000").expect_err("should have failed");
    }

    #[test]
    fn round_trip_random() {
        for _ in 0..100000 {
            let value = (random::<u128>() & ((1 << 108) - 1)) | (0x08000 << 108);
            let value = value.to_be_bytes();

            let result = encode_id(value).unwrap();
            let parsed = decode_id(&result).unwrap();

            assert_eq!(value, parsed);
        }
    }
}
