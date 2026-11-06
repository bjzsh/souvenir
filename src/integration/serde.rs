use crate::spec::{BYTE_LEN, MAX_TEXT_LEN};
use crate::{Id, Prefix, Souvenir, Tagged};
use core::fmt;
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Souvenir {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.encode(&mut [0; MAX_TEXT_LEN]))
        } else {
            serializer.serialize_bytes(self.as_bytes())
        }
    }
}

impl<'de> Deserialize<'de> for Souvenir {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(SouvenirVisitor)
        } else {
            deserializer.deserialize_bytes(SouvenirVisitor)
        }
    }
}

impl<T: Tagged> Serialize for Id<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_untyped().serialize(serializer)
    }
}

impl<'de, T: Tagged> Deserialize<'de> for Id<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(Souvenir::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// A prefix serializes as its string in every format.
impl Serialize for Prefix {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Prefix {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(PrefixVisitor)
    }
}

// Visitors

struct SouvenirVisitor;

impl<'de> Visitor<'de> for SouvenirVisitor {
    type Value = Souvenir;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a Souvenir ID string or 20 bytes")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Souvenir, E> {
        Souvenir::parse(v).map_err(E::custom)
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Souvenir, E> {
        Souvenir::from_slice(v).map_err(E::custom)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Souvenir, A::Error> {
        let mut bytes = [0u8; BYTE_LEN];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(i, &self))?;
        }
        if seq.next_element::<u8>()?.is_some() {
            return Err(de::Error::invalid_length(BYTE_LEN + 1, &self));
        }
        Souvenir::from_bytes(bytes).map_err(de::Error::custom)
    }
}

struct PrefixVisitor;

impl Visitor<'_> for PrefixVisitor {
    type Value = Prefix;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a Souvenir prefix: 1-8 characters of a-z")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Prefix, E> {
        Prefix::parse(v).map_err(E::custom)
    }
}
