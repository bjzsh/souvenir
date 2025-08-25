use crate::{
    encoding::{decode_prefix, decode_suffix, encode_prefix, encode_suffix, validate_prefix},
    error::{Error, Result},
    id::Id,
    suffix::Suffix,
};

pub fn encode_id(id: Id) -> String {
    let prefix = encode_prefix(id.prefix());
    let suffix = encode_suffix(id.suffix());

    format!("{}_{}", prefix, suffix)
}

pub fn decode_id(id: &str) -> Result<Id> {
    let (prefix, suffix) = id.rsplit_once('_').ok_or(Error::InvalidFormat)?;

    let prefix = decode_prefix(prefix)?;
    let suffix = decode_suffix(suffix)?;

    Ok(Id::new(prefix, suffix))
}

pub fn validate_id(bytes: [u8; 16]) -> Result<Id> {
    let suffix = u128::from_be_bytes(bytes);
    let prefix = (suffix >> 108) as u32;

    validate_prefix(prefix).map(|prefix| Id::new(prefix, Suffix::new(suffix)))
}

#[cfg(test)]
mod test {
    use crate::{error::Error, id::Id};

    fn encode_id(bytes: [u8; 16]) -> String {
        crate::encoding::encode_id(Id::from_bytes(bytes).unwrap())
    }

    fn decode_id(id: &str) -> Result<[u8; 16], Error> {
        crate::encoding::decode_id(id).map(Id::to_bytes)
    }

    #[test]
    fn encode_one() {
        let bytes: [u8; 16] = [
            0xac, 0xcb, 0x20, 0x2d, 0x95, 0x0c, 0x2a, 0x86, 0xff, 0x0c, 0x13, 0x29, 0x81, 0x7c,
            0x6a, 0x04,
        ];

        assert_eq!("user_02v58c5a3fy30k560qrtg4", encode_id(bytes))
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
