use std::ops::RangeInclusive;

use super::suffix::encode_suffix_digits;

use crate::{
    encoding::{Q, R, SPLIT, decode_prefix, decode_suffix, encode_prefix, suffix_interval},
    error::{Error, Result},
    id::Id,
    prefix::Prefix,
};

pub fn id_interval(prefix: Prefix) -> RangeInclusive<u128> {
    let rank = prefix.to_u32() as u128;
    let extra = rank.min(R);
    let start = rank * Q + extra;

    start..=start + *suffix_interval(prefix).end()
}

pub const fn split_id(value: u128) -> (u32, u128) {
    let (rank, suffix) = if value < SPLIT {
        (value / (Q + 1), value % (Q + 1))
    } else {
        let offset = value - SPLIT;
        (R + offset / Q, offset % Q)
    };

    (rank as u32, suffix)
}

pub fn encode_id(id: Id) -> String {
    let prefix = encode_prefix(id.prefix());
    let suffix = encode_suffix_digits(id.suffix());

    format!("{}_{}", prefix, suffix)
}

pub fn decode_id(id: &str) -> Result<Id> {
    let (prefix, suffix) = id.rsplit_once('_').ok_or(Error::InvalidFormat)?;

    let prefix = decode_prefix(prefix)?;
    let suffix = decode_suffix(suffix)?;

    Id::from_parts(prefix, suffix)
}
