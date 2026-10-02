use crate::id::Id;
use crate::prefix::Prefix;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            self.to_bytes().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            <String as Deserialize<'de>>::deserialize(deserializer)
                .map(|str| Self::parse(&str))?
                .map_err(Error::custom)
        } else {
            <[u8; 16] as Deserialize<'de>>::deserialize(deserializer).map(Self::new)
        }
    }
}

impl Serialize for Prefix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            self.to_u32().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Prefix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            <String as Deserialize<'de>>::deserialize(deserializer)
                .map(|str| Self::parse(&str))?
                .map_err(Error::custom)
        } else {
            <u32 as Deserialize<'de>>::deserialize(deserializer)
                .map(Self::new)?
                .map_err(Error::custom)
        }
    }
}
