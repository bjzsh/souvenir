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

fn random_hex(case: &Value) -> u128 {
    u128::from_str_radix(case["random_hex"].as_str().unwrap(), 16).unwrap()
}

/// Check a result against a case's `error` code, or else its `text`.
fn check(result: Result<Souvenir, Error>, case: &Value) {
    match case.get("error") {
        Some(code) => assert_eq!(
            result.map_err(error_code),
            Err(code.as_str().unwrap()),
            "{}",
            case["why"]
        ),
        None => assert_eq!(
            result.unwrap().to_string(),
            case["text"].as_str().unwrap(),
            "{}",
            case["why"]
        ),
    }
}

#[test]
fn from_parts() {
    let v = vectors();
    for case in v["from_parts"].as_array().unwrap() {
        let ts = case["timestamp_ms"].as_u64().unwrap();
        let result = Prefix::parse(case["prefix"].as_str().unwrap())
            .and_then(|p| Souvenir::from_parts(p, ts, random_hex(case)));
        check(result, case);
    }
}

#[cfg(feature = "uuid")]
#[test]
fn from_uuid() {
    let v = vectors();
    for case in v["from_uuid"].as_array().unwrap() {
        let uuid = uuid::Uuid::parse_str(case["uuid"].as_str().unwrap()).unwrap();
        let result = Prefix::parse(case["prefix"].as_str().unwrap())
            .and_then(|p| Souvenir::from_uuid(p, uuid));
        if let Ok(id) = result {
            assert_eq!(id.to_uuid(), uuid, "{}", case["why"]);
        }
        check(result, case);
    }
}

#[test]
fn monotonic() {
    use souvenir::{Generator, MonotonicGenerator};
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    let v = vectors();
    for sequence in v["monotonic"].as_array().unwrap() {
        let now = Cell::new(0u64);
        let draws = RefCell::new(VecDeque::<u128>::new());
        // `csprng_bits(n)` returns the scripted value: the generator's
        // big-endian read of the bytes it asks for gives it back.
        let random = |buf: &mut [u8]| {
            let value = draws.borrow_mut().pop_front().expect("an unscripted draw");
            let bytes = value.to_be_bytes();
            buf.copy_from_slice(&bytes[bytes.len() - buf.len()..]);
            Ok(())
        };
        let mut generator = MonotonicGenerator::with_sources(random, || Ok(now.get()));

        let mut last_body = None;
        for case in sequence["steps"].as_array().unwrap() {
            now.set(case["now_ms"].as_u64().unwrap());
            draws.borrow_mut().extend(
                case["draws_hex"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|d| u128::from_str_radix(d.as_str().unwrap(), 16).unwrap()),
            );
            let prefix = Prefix::parse(case["prefix"].as_str().unwrap()).unwrap();
            let result = generator.generate_untyped(prefix);
            assert!(draws.borrow().is_empty(), "unused draw: {}", case["why"]);
            if let Ok(id) = result {
                assert_eq!(id.timestamp_ms(), case["timestamp_ms"].as_u64().unwrap());
                assert_eq!(id.random(), random_hex(case));
                let body = (id.timestamp_ms(), id.random());
                assert!(last_body < Some(body), "{}", case["why"]);
                last_body = Some(body);
            }
            check(result, case);
        }
    }
}
