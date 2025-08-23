use crate::error::{Error, Result};

pub const PREFIX: &[u8; 32] = b"\xffabcdefghijklmnopqrstuvwxyz\xff\xff\xff\xff\xff";
const PREFIX_INV: &[u8; 256] = &{
    let mut output = [255; 256];

    let mut i = 0;
    while i < 32 {
        output[PREFIX[i as usize] as usize] = i;
        i += 1;
    }

    output[PREFIX[0] as usize] = 255;
    output
};

pub fn encode_prefix(mut raw: u32) -> Result<String> {
    let mut buf = [0u8; 4];
    let mut size = 0;

    for b in buf.iter_mut().rev() {
        let char = raw & 0x1f;
        raw >>= 5;

        if char == 0 && size == 0 {
            continue;
        }

        *b = PREFIX[char as usize];
        size += 1;
    }

    if size < 1 {
        return Err(Error::InvalidData);
    }

    String::from_utf8(buf[..size].to_vec()).map_err(|_| Error::InvalidData)
}

pub fn decode_prefix(prefix: &str) -> Result<u32> {
    let size = prefix.len();

    if !(1..=4).contains(&size) {
        return Err(Error::InvalidPrefix);
    }

    prefix
        .as_bytes()
        .iter()
        .try_fold(0u32, |acc, &ch| {
            let value = PREFIX_INV[ch as usize];

            if value != 0xff {
                Ok((acc << 5) | value as u32)
            } else {
                Err(Error::InvalidPrefix)
            }
        })
        .map(|result| result << ((4 - size) * 5))
}

#[allow(clippy::overly_complex_bool_expr)]
pub fn valid_prefix(prefix: u32) -> bool {
    let a = prefix >> 15;
    let b = (prefix >> 10) & 0x1f;
    let c = (prefix >> 5) & 0x1f;
    let d = prefix & 0x1f;

    if !((a != 0 && b != 0 && c != 0 && d != 0)
        || (a != 0 && b != 0 && c != 0 && d == 0)
        || (a != 0 && b != 0 && c == 0 && d == 0)
        || (a != 0 && b == 0 && c == 0 && d == 0))
    {
        return false;
    }

    if a > 26 || b > 26 || c > 26 || d > 26 {
        return false;
    }

    true
}

#[cfg(test)]
mod test {
    use crate::{
        encoding::{PREFIX, decode_prefix, encode_prefix, valid_prefix},
        error::Error,
    };

    #[test]
    fn encode_smoke() {
        assert_eq!("user", encode_prefix(0b10101_10011_00101_10010).unwrap());
        assert_eq!("use", encode_prefix(0b10101_10011_00101_00000).unwrap());
        assert_eq!("us", encode_prefix(0b10101_10011_00000_00000).unwrap());
        assert_eq!("u", encode_prefix(0b10101_00000_00000_00000).unwrap());
    }

    #[test]
    fn encode_iter() {
        fn format_one(char: u32) -> String {
            if char == 0 {
                return String::new();
            }

            let char = [PREFIX[char as usize]];
            String::from_utf8(char.to_vec()).unwrap()
        }

        for i in 0..(1 << 20) {
            if !valid_prefix(i) {
                assert_eq!(Err(Error::InvalidData), encode_prefix(i));
                continue;
            }

            let a = i >> 15;
            let b = (i >> 10) & 0x1f;
            let c = (i >> 5) & 0x1f;
            let d = i & 0x1f;

            assert_eq!(
                Ok(format!(
                    "{}{}{}{}",
                    format_one(a),
                    format_one(b),
                    format_one(c),
                    format_one(d),
                )),
                encode_prefix(i),
            );
        }
    }

    #[test]
    fn decode_smoke() {
        assert_eq!(Ok(0b10101_10011_00101_10010), decode_prefix("user"));
        assert_eq!(Ok(0b10101_10011_00101_00000), decode_prefix("use"));
        assert_eq!(Ok(0b10101_10011_00000_00000), decode_prefix("us"));
        assert_eq!(Ok(0b10101_00000_00000_00000), decode_prefix("u"));
    }

    #[test]
    fn decode_invalid() {
        assert_eq!(Err(Error::InvalidPrefix), decode_prefix(""));
        assert_eq!(Err(Error::InvalidPrefix), decode_prefix("aaaaa"));
        assert_eq!(Err(Error::InvalidPrefix), decode_prefix("\0"));
        assert_eq!(Err(Error::InvalidPrefix), decode_prefix("A"));
        assert_eq!(Err(Error::InvalidPrefix), decode_prefix("!"));
        assert_eq!(Err(Error::InvalidPrefix), decode_prefix(" "));
    }

    #[test]
    fn decode_iter() {
        for a in 'a'..='z' {
            let a_val = a as u32 - 'a' as u32 + 1;
            assert_eq!(Ok(a_val << 15), decode_prefix(&format!("{a}")));

            for b in 'a'..='z' {
                let b_val = b as u32 - 'a' as u32 + 1;
                assert_eq!(
                    Ok(a_val << 15 | b_val << 10),
                    decode_prefix(&format!("{a}{b}"))
                );

                for c in 'a'..='z' {
                    let c_val = c as u32 - 'a' as u32 + 1;
                    assert_eq!(
                        Ok(a_val << 15 | b_val << 10 | c_val << 5),
                        decode_prefix(&format!("{a}{b}{c}"))
                    );

                    for d in 'a'..='z' {
                        let d_val = d as u32 - 'a' as u32 + 1;
                        assert_eq!(
                            Ok(a_val << 15 | b_val << 10 | c_val << 5 | d_val),
                            decode_prefix(&format!("{a}{b}{c}{d}"))
                        );
                    }
                }
            }
        }
    }
}
