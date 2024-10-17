use crate::Error;

const CROCKFORD: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
const CROCKFORD_INV: &[u8; 256] = &{
    let mut output = [255; 256];

    let mut i = 0;
    while i < 32 {
        output[CROCKFORD[i as usize] as usize] = i;
        i += 1;
    }

    output
};

pub fn parse_base32(id: &str) -> Result<[u8; 16], Error> {
    let mut id: [u8; 26] = id.as_bytes().try_into().map_err(|_| Error::InvalidData)?;
    let mut max = 0;

    for b in &mut id {
        *b = CROCKFORD_INV[*b as usize];
        max |= *b;
    }

    if max > 32 || id[0] > 7 {
        return Err(Error::InvalidData);
    }

    let mut out = 0u128;
    for b in id {
        out <<= 5;
        out |= b as u128;
    }

    Ok(out.to_be_bytes())
}

pub fn stringify_base32(id: [u8; 16]) -> Result<String, Error> {
    let mut buf = [0; 26];
    let mut data = u128::from_be_bytes(id);

    for b in buf.iter_mut().rev() {
        *b = CROCKFORD[((data as u8) & 0x1f) as usize];
        debug_assert!(b.is_ascii());
        data >>= 5;
    }

    Ok(String::from_utf8(buf.to_vec()).expect("only ascii bytes should be in the buffer"))
}

#[cfg(test)]
mod test {
    use crate::encoding::{parse_base32, stringify_base32};
    use rand::random;

    #[test]
    fn decode_random() {
        assert_eq!(
            [
                0x64, 0x29, 0x8e, 0x84, 0xa9, 0x6c, 0x00, 0x00, 0x00, 0x88, 0x64, 0x29, 0x8e, 0x84,
                0xa9, 0x6c
            ],
            parse_base32("3456789abc0000123456789abc").unwrap()
        );
    }

    #[test]
    fn decode_min() {
        assert_eq!(
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ],
            parse_base32("00000000000000000000000000").unwrap()
        );
    }

    #[test]
    fn decode_max() {
        assert_eq!(
            [
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff
            ],
            parse_base32("7zzzzzzzzzzzzzzzzzzzzzzzzz").unwrap()
        );
    }

    #[test]
    fn decode_invalid() {
        parse_base32("80000000000000000000000000").expect_err("should have failed");
    }

    #[test]
    fn round_trip_random() {
        for _ in 0..100000 {
            let value: [u8; 16] = random();
            let result = stringify_base32(value).unwrap();
            let parsed = parse_base32(&*result).unwrap();
            assert_eq!(value, parsed);
        }
    }
}
