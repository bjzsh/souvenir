mod id;
mod prefix;
mod suffix;

pub use id::*;
pub use prefix::*;
pub use suffix::*;

pub const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
pub const ALPHABET_INV: &[u8; 256] = &{
    let mut output = [255; 256];

    let mut i = 0;
    while i < 32 {
        output[ALPHABET[i as usize] as usize] = i;
        i += 1;
    }

    output
};
