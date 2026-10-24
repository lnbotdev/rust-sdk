//! Decoding of Server-Sent Events payloads.

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::errors::LnBotError;

/// Decodes the JSON of one SSE `data:` line into an SDK type.
///
/// Event payloads use snake_case keys on the wire (`max_fee`, `created_at`),
/// while the SDK types use the camelCase names of the REST API. Keys are
/// converted at every depth before decoding, which covers the `data` object
/// nested in wallet stream events. Keys already in camelCase are unchanged.
pub(crate) fn decode<T: DeserializeOwned>(raw: &str) -> Result<T, LnBotError> {
    let value: Value = serde_json::from_str(raw)?;
    Ok(serde_json::from_value(camel_case_keys(value))?)
}

fn camel_case_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| (camel_case(key), camel_case_keys(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(camel_case_keys).collect()),
        other => other,
    }
}

/// Converts `snake_case` to `camelCase`: an underscore followed by a lowercase
/// letter or digit is dropped and that character uppercased. Leading
/// underscores and any other characters are kept.
fn camel_case(key: String) -> String {
    if !key.contains('_') {
        return key;
    }
    let mut out = String::with_capacity(key.len());
    let mut chars = key.chars().peekable();
    while let Some(c) = chars.next() {
        match chars.peek() {
            Some(&next)
                if c == '_'
                    && !out.is_empty()
                    && (next.is_ascii_lowercase() || next.is_ascii_digit()) =>
            {
                out.push(next.to_ascii_uppercase());
                chars.next();
            }
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn camel_case_converts_snake_case() {
        assert_eq!(camel_case("max_fee".into()), "maxFee");
        assert_eq!(camel_case("failure_reason".into()), "failureReason");
        assert_eq!(camel_case("tx_number".into()), "txNumber");
        assert_eq!(camel_case("a_b_c".into()), "aBC");
    }

    #[test]
    fn camel_case_keeps_other_keys() {
        assert_eq!(camel_case("maxFee".into()), "maxFee");
        assert_eq!(camel_case("bolt11".into()), "bolt11");
        assert_eq!(camel_case("_id".into()), "_id");
        assert_eq!(camel_case("trailing_".into()), "trailing_");
        assert_eq!(camel_case("upper_Case".into()), "upper_Case");
    }

    #[test]
    fn camel_case_keys_converts_nested_objects_and_arrays() {
        let value = json!({
            "created_at": "t",
            "data": { "max_fee": 5, "items": [{ "tx_number": 1 }] },
            "alreadyCamel": { "service_fee": 1 }
        });
        assert_eq!(
            camel_case_keys(value),
            json!({
                "createdAt": "t",
                "data": { "maxFee": 5, "items": [{ "txNumber": 1 }] },
                "alreadyCamel": { "serviceFee": 1 }
            }),
        );
    }

    #[test]
    fn camel_case_keys_leaves_values_alone() {
        let value = json!({ "status": "snake_case_value", "memo": "a_b" });
        assert_eq!(camel_case_keys(value.clone()), value);
    }
}
