use crate::id::Id;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <String as Deserialize<'de>>::deserialize(deserializer)
            .map(|str| Self::parse(&str))?
            .map_err(Error::custom)
    }
}
