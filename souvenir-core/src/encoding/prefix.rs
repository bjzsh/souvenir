use crate::{
    encoding::SUBTREE_SIZES,
    error::{Error, Result},
    prefix::Prefix,
};

pub fn encode_prefix(rank: Prefix) -> String {
    let mut rank = rank.to_u32();
    let mut prefix = String::with_capacity(4);

    for size in SUBTREE_SIZES {
        prefix.push((b'a' + (rank / size) as u8) as char);

        let remainder = rank % size;
        if remainder == 0 {
            break;
        }

        rank = remainder - 1;
    }

    prefix
}

pub const fn decode_prefix(prefix: &str) -> Result<Prefix> {
    if prefix.is_empty() || prefix.len() > 4 {
        return Err(Error::InvalidPrefix);
    }

    let mut rank = 0;
    let mut depth = 0;

    while depth < prefix.len() {
        let letter = prefix.as_bytes()[depth];

        if letter < b'a' || letter > b'z' {
            return Err(Error::InvalidPrefix);
        }

        if depth != 0 {
            rank += 1;
        }

        rank += (letter - b'a') as u32 * SUBTREE_SIZES[depth];
        depth += 1;
    }

    Ok(unsafe { Prefix::new_unchecked(rank) })
}

#[cfg(test)]
mod test {
    use super::{decode_prefix, encode_prefix};
    use crate::error::Error;

    fn all_prefixes() -> Vec<String> {
        let mut prefixes = Vec::new();

        for a in 'a'..='z' {
            prefixes.push(format!("{a}"));

            for b in 'a'..='z' {
                prefixes.push(format!("{a}{b}"));

                for c in 'a'..='z' {
                    prefixes.push(format!("{a}{b}{c}"));

                    for d in 'a'..='z' {
                        prefixes.push(format!("{a}{b}{c}{d}"));
                    }
                }
            }
        }

        prefixes.sort();
        prefixes
    }

    #[test]
    fn ordering() {
        let prefixes = all_prefixes();

        for (index, prefix) in prefixes.iter().enumerate() {
            let decoded = decode_prefix(prefix).unwrap().to_u32();
            assert_eq!(index as u32, decoded);
        }
    }

    #[test]
    fn round_trip() {
        let prefixes = all_prefixes();

        for prefix in prefixes {
            let decoded = decode_prefix(&prefix).unwrap();
            let encoded = encode_prefix(decoded);

            assert_eq!(prefix, encoded);
        }
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
}
