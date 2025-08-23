use crate::{
    encoding::{decode_prefix, decode_suffix, encode_prefix, encode_suffix, valid_prefix},
    error::Error,
};

pub fn encode_id(bytes: [u8; 16]) -> Result<String, Error> {
    let raw = u128::from_be_bytes(bytes);

    let prefix = encode_prefix((raw >> 108) as u32)?;
    let suffix = encode_suffix(raw & ((1 << 108) - 1))?;

    Ok(format!("{}_{}", prefix, suffix))
}

pub fn decode_id(id: &str) -> Result<[u8; 16], Error> {
    let (prefix, suffix) = id.rsplit_once('_').ok_or(Error::InvalidFormat)?;

    let left = decode_prefix(prefix).map(|x| (x as u128) << 108)?;
    let right = decode_suffix(suffix)?;

    Ok((left | right).to_be_bytes())
}

pub fn valid_id(bytes: [u8; 16]) -> bool {
    valid_prefix((u128::from_be_bytes(bytes) >> 108) as u32)
}

#[cfg(test)]
mod test {
    use crate::{
        encoding::{decode_id, encode_id},
        error::Error,
    };

    #[test]
    fn encode_one() {
        let bytes: [u8; 16] = [
            0xac, 0xcb, 0x20, 0x2d, 0x95, 0x0c, 0x2a, 0x86, 0xff, 0x0c, 0x13, 0x29, 0x81, 0x7c,
            0x6a, 0x04,
        ];

        assert_eq!("user_02v58c5a3fy30k560qrtg4", encode_id(bytes).unwrap())
    }

    #[test]
    fn decode_random() {
        assert_eq!(
            [
                0x08, 0x86, 0x00, 0x11, 0x0c, 0x85, 0x31, 0xd0, 0x95, 0x2d, 0x8d, 0x73, 0xe1, 0x19,
                0x4e, 0x95,
            ],
            decode_id("abc_0123456789abcdefghjkmn").unwrap()
        );
    }

    #[test]
    fn decode_min() {
        assert_eq!(
            [
                0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ],
            decode_id("a_0000000000000000000000").unwrap()
        );
    }

    #[test]
    fn decode_max() {
        assert_eq!(
            [
                0xd6, 0xb5, 0xaf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff,
            ],
            decode_id("zzzz_7zzzzzzzzzzzzzzzzzzzzz").unwrap()
        );
    }

    #[test]
    fn decode_invalid() {
        assert_eq!(
            Err(Error::InvalidFormat),
            decode_id("80000000000000000000000000")
        )
    }
}
