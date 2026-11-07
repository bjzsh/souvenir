//! Conformance against `tests/fixtures/souvenir_v2_vectors.json`, the
//! vectors `SPEC.md` declares normative. The JS tests use the same file, and
//! the constants it lists are checked by a unit test in `src/spec.rs`.

use serde_json::Value;
use souvenir::{Error, Prefix, Souvenir};

fn vectors() -> Value {
    serde_json::from_str(include_str!("fixtures/souvenir_v2_vectors.json")).unwrap()
}

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn to_hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}

fn rank(prefix: &str) -> Result<u64, Error> {
    Prefix::parse(prefix).map(Prefix::rank)
}

fn error_code(e: Error) -> &'static str {
    e.code()
}

#[test]
fn prefix_ranks() {
    let v = vectors();
    for (prefix, r) in v["prefix_ranks"].as_object().unwrap() {
        let r = r.as_u64().unwrap();
        assert_eq!(rank(prefix), Ok(r), "rank({prefix})");
        assert_eq!(
            Prefix::from_rank(r).unwrap(),
            prefix.as_str(),
            "unrank({r})"
        );
    }
}

#[test]
fn valid() {
    let v = vectors();
    for case in v["valid"].as_array().unwrap() {
        let prefix = case["prefix"].as_str().unwrap();
        let ts = case["timestamp_ms"].as_u64().unwrap();
        let random = u128::from_str_radix(case["random_hex"].as_str().unwrap(), 16).unwrap();
        let bytes_hex = case["bytes_hex"].as_str().unwrap();
        let text = case["text"].as_str().unwrap();
        let uuid_text = case["uuid"].as_str().unwrap();

        let prefix_value = Prefix::parse(prefix).unwrap();
        let id = Souvenir::from_parts(prefix_value, ts, random).unwrap();
        assert_eq!(to_hex(&id.to_bytes()), bytes_hex, "{text}");
        assert_eq!(id.to_string(), text);
        #[cfg(feature = "uuid")]
        assert_eq!(id.to_uuid().to_string(), uuid_text);
        #[cfg(not(feature = "uuid"))]
        let _ = uuid_text;

        for parsed in [
            Souvenir::parse(text).unwrap(),
            Souvenir::from_slice(&hex(bytes_hex)).unwrap(),
            #[cfg(feature = "uuid")]
            Souvenir::from_uuid(prefix_value, uuid::Uuid::parse_str(uuid_text).unwrap()).unwrap(),
        ] {
            assert_eq!(parsed, id);
            assert_eq!(parsed.prefix(), prefix);
            assert_eq!(
                rank(parsed.prefix().as_str()).unwrap(),
                case["prefix_rank"].as_u64().unwrap()
            );
            assert_eq!(parsed.timestamp_ms(), ts);
            assert_eq!(parsed.random(), random);
        }
    }
}

#[test]
fn sorted_text_matches_byte_order() {
    let v = vectors();
    let expected: Vec<&str> = v["sorted_text"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    let mut ids: Vec<Souvenir> = expected
        .iter()
        .rev()
        .map(|t| Souvenir::parse(t).unwrap())
        .collect();
    ids.sort();
    let sorted: Vec<String> = ids.iter().map(Souvenir::to_string).collect();
    assert_eq!(sorted, expected);
    let mut by_bytes = ids.clone();
    by_bytes.sort_by_key(|id| id.to_bytes());
    assert_eq!(by_bytes, ids);
}

#[test]
fn invalid() {
    let v = vectors();
    for case in v["invalid"].as_array().unwrap() {
        let input = case["input"].as_str().unwrap();
        let expected = case["error"].as_str().unwrap();
        let result = match case["kind"].as_str().unwrap() {
            "text" => Souvenir::parse(input),
            "bytes" => Souvenir::from_slice(&hex(input)),
            kind => panic!("unknown kind {kind}"),
        };
        assert_eq!(
            result.map_err(error_code),
            Err(expected),
            "{}: {input:?}",
            case["why"]
        );
    }
}

#[test]
fn parse_expecting() {
    let v = vectors();
    for case in v["parse_expecting"].as_array().unwrap() {
        let expected_prefix = case["expected"].as_str().unwrap();
        let input = case["input"].as_str().unwrap();
        // The expected prefix is checked first: an invalid one never reaches
        // parse_expecting.
        let result =
            Prefix::parse(expected_prefix).and_then(|p| Souvenir::parse_expecting(p, input));
        match case.get("error") {
            Some(code) => assert_eq!(
                result.map_err(error_code),
                Err(code.as_str().unwrap()),
                "{}",
                case["why"]
            ),
            None => assert_eq!(result.unwrap().to_string(), input),
        }
    }
}
