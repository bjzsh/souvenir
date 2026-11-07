use crate::{Error, Id, Prefix, Result, Souvenir, Tagged};
use uuid::Uuid;

impl Souvenir {
    /// The timestamp and random bits as a UUIDv7. The prefix is dropped;
    /// use [`from_uuid`](Self::from_uuid) with the same prefix to restore it.
    #[must_use]
    pub const fn to_uuid(self) -> Uuid {
        let random = self.random();
        let u = ((self.timestamp_ms() as u128) << 80)
            | (0x7 << 76)
            | ((random >> 62) << 64)
            | (0b10 << 62)
            | (random & Self::RAND_B_MASK);
        Uuid::from_u128(u)
    }

    /// Rebuild an ID from a UUIDv7 and a prefix.
    ///
    /// Fails with [`Error::NotUuidV7`] unless the UUID has version 7 and
    /// variant `0b10`.
    pub const fn from_uuid(prefix: Prefix, uuid: Uuid) -> Result<Self> {
        let u = uuid.as_u128();
        if (u >> 76) & 0xF != 0x7 || (u >> 62) & 0b11 != 0b10 {
            return Err(Error::NotUuidV7);
        }
        let random = (((u >> 64) & 0xFFF) << 62) | (u & Self::RAND_B_MASK);
        Self::from_parts(prefix, (u >> 80) as u64, random)
    }
}

// Crate-internal: the UUIDv7 bit mapping (SPEC.md section 8). The timestamp fills
// `unix_ts_ms`; the random bits fill `rand_a` (12) and `rand_b` (62).
impl Souvenir {
    const RAND_B_MASK: u128 = (1 << 62) - 1;
}

impl<T: Tagged> Id<T> {
    /// Rebuild an ID from a UUIDv7; the type supplies the prefix.
    pub const fn from_uuid(uuid: Uuid) -> Result<Self> {
        Ok(Self::from_untyped_unchecked(tri!(Souvenir::from_uuid(
            T::PREFIX,
            uuid
        ))))
    }

    /// The timestamp and random bits as a UUIDv7, for storage in a column
    /// whose type already implies the prefix.
    #[must_use]
    pub const fn to_uuid(self) -> Uuid {
        self.to_untyped().to_uuid()
    }
}

// Only typed IDs convert with `From`/`TryFrom`: their type restores the
// prefix, which the UUID form drops.
impl<T: Tagged> From<Id<T>> for Uuid {
    fn from(id: Id<T>) -> Self {
        id.to_uuid()
    }
}

impl<T: Tagged> TryFrom<Uuid> for Id<T> {
    type Error = Error;

    /// Fails with [`Error::NotUuidV7`] unless the UUID is a UUIDv7.
    fn try_from(uuid: Uuid) -> Result<Self> {
        Self::from_uuid(uuid)
    }
}
