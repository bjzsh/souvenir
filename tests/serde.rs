//! serde integration: text in human-readable formats, 20 bytes otherwise.
#![cfg(feature = "serde")]

use serde::{Deserialize, Serialize};
use souvenir::{Error, Id, Prefix, Souvenir, tagged};

tagged!(User = "user");
tagged!(Org = "org");

const TEXT: &str = "user_00t35pg1sg9ech32tymrkdq6v";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Record {
    id: Id<User>,
    any: Souvenir,
    parent: Option<Souvenir>,
}

fn record() -> Record {
    Record {
        id: TEXT.parse().unwrap(),
        any: "org_00t35pg1sgajz7jf1pz84mv4e".parse().unwrap(),
        parent: None,
    }
}

#[test]
fn json_uses_text() {
    let json = serde_json::to_string(&record()).unwrap();
    assert_eq!(
        json,
        format!(r#"{{"id":"{TEXT}","any":"org_00t35pg1sgajz7jf1pz84mv4e","parent":null}}"#)
    );
    assert_eq!(serde_json::from_str::<Record>(&json).unwrap(), record());
}

#[test]
fn json_errors() {
    let wrong_prefix = serde_json::from_str::<Id<Org>>(&format!("\"{TEXT}\""));
    assert!(
        wrong_prefix
            .unwrap_err()
            .to_string()
            .contains(Error::PrefixMismatch.message())
    );
    let bad = serde_json::from_str::<Souvenir>("\"user_123\"");
    assert!(
        bad.unwrap_err()
            .to_string()
            .contains(Error::InvalidLength.message())
    );
    assert!(serde_json::from_str::<Souvenir>("42").is_err());
}

#[test]
fn binary_uses_20_bytes() {
    let id = Souvenir::parse(TEXT).unwrap();
    let bytes = postcard::to_stdvec(&id).unwrap();
    // postcard: varint length (20) + the bytes.
    assert_eq!(bytes.len(), 21);
    assert_eq!(&bytes[1..], id.as_bytes());
    assert_eq!(postcard::from_bytes::<Souvenir>(&bytes).unwrap(), id);

    let record = record();
    let encoded = postcard::to_stdvec(&record).unwrap();
    assert_eq!(postcard::from_bytes::<Record>(&encoded).unwrap(), record);
}

#[test]
fn binary_rejects_zero_bytes() {
    let mut zeros = vec![20u8];
    zeros.extend([0u8; 20]);
    let err = postcard::from_bytes::<Souvenir>(&zeros).unwrap_err();
    assert!(matches!(err, postcard::Error::SerdeDeCustom));
}

#[test]
fn json_accepts_byte_arrays() {
    // A 20-element JSON array goes through visit_seq.
    let id = Souvenir::parse(TEXT).unwrap();
    let json = serde_json::to_string(id.as_bytes()).unwrap();
    let mut de = serde_json::Deserializer::from_str(&json);
    let parsed = serde::Deserializer::deserialize_seq(&mut de, SeqProbe).unwrap();
    assert_eq!(parsed, id);
}

/// Drives `Souvenir`'s visitor through a sequence, as non-self-describing
/// formats that encode bytes as tuples would.
struct SeqProbe;

impl<'de> serde::de::Visitor<'de> for SeqProbe {
    type Value = Souvenir;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("seq")
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, seq: A) -> Result<Souvenir, A::Error> {
        Souvenir::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))
    }
}

#[test]
fn prefixes_serialize_as_strings() {
    let prefix: Prefix = "user".parse().unwrap();
    assert_eq!(serde_json::to_string(&prefix).unwrap(), "\"user\"");
    assert_eq!(serde_json::from_str::<Prefix>("\"user\"").unwrap(), prefix);
    let err = serde_json::from_str::<Prefix>("\"User\"").unwrap_err();
    assert!(
        err.to_string().contains(Error::InvalidPrefix.message()),
        "{err}"
    );
    let bytes = postcard::to_stdvec(&prefix).unwrap();
    assert_eq!(postcard::from_bytes::<Prefix>(&bytes).unwrap(), prefix);
}
