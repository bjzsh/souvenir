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

pub fn parse_base32(id: &str) -> Result<[u8; 8], Error> {
    let mut id: [u8; 13] = id.as_bytes().try_into().map_err(|_| Error::InvalidData)?;
    let mut max = 0;

    for b in &mut id {
        *b = CROCKFORD_INV[*b as usize];
        max |= *b;
    }

    if max > 32 || id[0] > 15 {
        return Err(Error::InvalidData);
    }

    let mut out = 0u64;
    for b in id {
        out <<= 5;
        out |= b as u64;
    }

    Ok(out.to_be_bytes())
}

pub fn stringify_base32(id: [u8; 8]) -> Result<String, Error> {
    let mut buf = [0; 13];
    let mut data = u64::from_be_bytes(id);

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
    fn encode_decode() {
        for _ in 0..100000 {
            let value: [u8; 8] = random();
            let result = stringify_base32(value).unwrap();
            let parsed = parse_base32(&*result).unwrap();
            assert_eq!(value, parsed);
        }
    }

    #[test]
    fn decode() {
        assert_eq!(
            [0x00, 0x88, 0x64, 0x29, 0x8e, 0x84, 0xa9, 0x6c],
            parse_base32("0123456789abc").unwrap()
        );

        assert_eq!(
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            parse_base32("fzzzzzzzzzzzz").unwrap()
        );

        assert_eq!(
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            parse_base32("0000000000000").unwrap()
        );

        parse_base32("g000000000000").expect_err("should have failed");
    }
}
