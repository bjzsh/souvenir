use crate::{Error, Id, Result, Storage, Tagged};
use uuid::Uuid;

/// An ID's binary database form: the 20 bytes for `Storage::Bytes`, a
/// UUIDv7 for `Storage::Uuid`.
#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
pub(crate) enum StoredBinary {
    Bytes([u8; crate::spec::BYTE_LEN]),
    Uuid(Uuid),
}

#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
impl StoredBinary {
    pub(crate) fn of<T: Tagged>(id: &Id<T>) -> Self {
        match T::STORAGE {
            Storage::Uuid => Self::Uuid(id.to_uuid()),
            _ => Self::Bytes(id.to_bytes()),
        }
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        match self {
            Self::Bytes(bytes) => bytes,
            Self::Uuid(uuid) => uuid.as_bytes(),
        }
    }
}

/// Decode `T`'s stored binary form.
pub(crate) fn decode_stored_binary<T: Tagged>(bytes: &[u8]) -> Result<Id<T>> {
    match T::STORAGE {
        Storage::Uuid => {
            let uuid = Uuid::from_slice(bytes).map_err(|_| Error::InvalidLength)?;
            Id::from_uuid(uuid)
        }
        _ => Id::from_slice(bytes),
    }
}
