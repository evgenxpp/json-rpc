use std::fmt::{self, Display};

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as DeError, Visitor},
};
use std::result::Result as StdResult;

#[derive(Debug, PartialEq)]
pub enum Id {
    Num(i64),
    Str(String),
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Id::Num(num_id) => write!(f, "{num_id}"),
            Id::Str(str_id) => write!(f, "{str_id}"),
        }
    }
}

impl From<i64> for Id {
    fn from(value: i64) -> Self {
        Id::Num(value)
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id::Str(value)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Num(num_id) => serializer.serialize_i64(*num_id),
            Self::Str(str_id) => serializer.serialize_str(str_id),
        }
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IdVisitor)
    }
}

struct IdVisitor;

impl<'de> Visitor<'de> for IdVisitor {
    type Value = Id;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string or a 64-bit signed integer")
    }

    fn visit_str<E>(self, v: &str) -> StdResult<Self::Value, E>
    where
        E: DeError,
    {
        self.visit_string(v.to_owned())
    }

    fn visit_string<E>(self, v: String) -> StdResult<Self::Value, E>
    where
        E: DeError,
    {
        Ok(Self::Value::Str(v))
    }

    fn visit_i64<E>(self, v: i64) -> StdResult<Self::Value, E>
    where
        E: DeError,
    {
        Ok(Self::Value::Num(v))
    }

    fn visit_u64<E>(self, v: u64) -> StdResult<Self::Value, E>
    where
        E: DeError,
    {
        if v <= i64::MAX as u64 {
            Ok(Self::Value::Num(v as i64))
        } else {
            Err(DeError::custom(
                "number too large: expected a signed 64-bit integer ",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialization() {
        let expected_id = i64::MAX;
        let num_id = Id::Num(i64::MAX);
        let json = serde_json::to_value(num_id).unwrap();
        assert_eq!(json, json!(expected_id));

        let expected_id = "foo bar";
        let str_id = Id::Str(expected_id.to_owned());
        let json = serde_json::to_value(str_id).unwrap();
        assert_eq!(json, json!(expected_id));
    }

    #[test]
    fn test_deserialization() {
        assert!(
            serde_json::from_value::<Id>(json!(null))
                .unwrap_err()
                .to_string()
                .contains("invalid type: null, expected a string or a 64-bit signed integer")
        );

        assert!(
            serde_json::from_value::<Id>(json!(true))
                .unwrap_err()
                .to_string()
                .contains(
                    "invalid type: boolean `true`, expected a string or a 64-bit signed integer"
                )
        );

        assert!(
            serde_json::from_value::<Id>(json!(u64::MAX))
                .unwrap_err()
                .to_string()
                .contains("number too large: expected a signed 64-bit integer ")
        );

        assert_eq!(
            serde_json::from_value::<Id>(json!(i64::MAX)).unwrap(),
            Id::Num(i64::MAX)
        );
    }
}
