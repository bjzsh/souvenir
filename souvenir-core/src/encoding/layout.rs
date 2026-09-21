// Prefix
pub const PREFIX_BASE: u32 = 26;

pub const PREFIX_COUNT: u32 =
    PREFIX_BASE.pow(1) + PREFIX_BASE.pow(2) + PREFIX_BASE.pow(3) + PREFIX_BASE.pow(4);

pub const SUBTREE_SIZES: [u32; 4] = [
    PREFIX_BASE.pow(0) + PREFIX_BASE.pow(1) + PREFIX_BASE.pow(2) + PREFIX_BASE.pow(3),
    PREFIX_BASE.pow(0) + PREFIX_BASE.pow(1) + PREFIX_BASE.pow(2),
    PREFIX_BASE.pow(0) + PREFIX_BASE.pow(1),
    PREFIX_BASE.pow(0),
];

// Suffix
pub const N: u128 = PREFIX_COUNT as u128;
pub const Q: u128 = u128::MAX / N;
pub const R: u128 = u128::MAX % N + 1;

// Id
pub const SPLIT: u128 = R * (Q + 1);
