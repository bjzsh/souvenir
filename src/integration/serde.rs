use crate::{Id, Type};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl<T: Type> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de, T: Type> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <String as Deserialize<'de>>::deserialize(deserializer)
            .map(|str| Self::parse(&str))?
            .map_err(|e| Error::custom(e))
    }
}
