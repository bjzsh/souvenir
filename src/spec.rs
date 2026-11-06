// The numbers `SPEC.md` defines. `Souvenir` re-exports the ones users need
// as associated constants.

pub(crate) const MAX_PREFIX_LEN: usize = 8;

pub(crate) const PREFIX_BITS: u32 = 38;

pub(crate) const TIMESTAMP_BITS: u32 = 48;

pub(crate) const RANDOM_BITS: u32 = 74;

/// The timestamp and random fields together.
pub(crate) const BODY_BITS: u32 = TIMESTAMP_BITS + RANDOM_BITS;

pub(crate) const BYTE_LEN: usize = 20;

/// Length of the text after the `_`.
pub(crate) const SUFFIX_LEN: usize = 25;

pub(crate) const MAX_TEXT_LEN: usize = MAX_PREFIX_LEN + 1 + SUFFIX_LEN;

/// Lowercase Crockford Base32, in ascending ASCII order.
pub(crate) const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

pub(crate) const TIMESTAMP_MAX: u64 = (1 << TIMESTAMP_BITS) - 1;

pub(crate) const RANDOM_MAX: u128 = (1 << RANDOM_BITS) - 1;

/// `PREFIX_COUNTS[m]` is the number of strings of length `0..=m` over `a`-`z`
/// (the sum of `26^k` for `k` in `0..=m`), for the prefix rank (SPEC.md section 4).
pub(crate) const PREFIX_COUNTS: [u64; MAX_PREFIX_LEN + 1] = [
    1,
    27,
    703,
    18_279,
    475_255,
    12_356_631,
    321_272_407,
    8_353_082_583,
    217_180_147_159,
];

/// One past the rank of `"zzzzzzzz"`. Ranks from here up to 2^38 - 1 are
/// reserved for future formats.
pub(crate) const PREFIX_RANK_LIMIT: u64 = PREFIX_COUNTS[MAX_PREFIX_LEN];

pub(crate) const BODY_MASK: u128 = (1 << BODY_BITS) - 1;

// Compile-time checks that the hard-coded tables are what the spec defines.
const _: () = {
    let mut sum = 0u64;
    let mut power = 1u64;
    let mut m = 0;

    while m <= MAX_PREFIX_LEN {
        sum += power;
        assert!(PREFIX_COUNTS[m] == sum);
        power *= 26;
        m += 1;
    }

    assert!(PREFIX_RANK_LIMIT <= 1 << PREFIX_BITS);
    assert!(PREFIX_BITS + BODY_BITS == 8 * BYTE_LEN as u32);
    assert!(5 * SUFFIX_LEN as u32 >= BODY_BITS && 5 * (SUFFIX_LEN as u32 - 1) < BODY_BITS);

    let mut i = 1;
    while i < ALPHABET.len() {
        assert!(ALPHABET[i - 1] < ALPHABET[i]);
        i += 1;
    }
};
