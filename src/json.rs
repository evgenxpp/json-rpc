use std::fmt::Display;

use serde::de::{Deserialize, Error as DeError};
use serde_json::Value;

pub(crate) fn make_field_error<R, E>(field: &str, reason: R) -> E
where
    R: Display,
    E: DeError,
{
    E::custom(format!("field `{}` contains an {}", field, reason))
}

pub(crate) fn deserialize_i64<E>(field: &str, value: Value) -> Result<i64, E>
where
    E: DeError,
{
    i64::deserialize(value).map_err(|err| make_field_error(field, err))
}

pub(crate) fn deserialize_string<E>(field: &str, value: Value) -> Result<String, E>
where
    E: DeError,
{
    String::deserialize(value).map_err(|err| make_field_error(field, err))
}

pub(crate) fn deserialize_nullable_string<E>(field: &str, value: Value) -> Result<Option<String>, E>
where
    E: DeError,
{
    match value {
        Value::Null => Ok(None),
        _ => deserialize_string(field, value).map(Some),
    }
}
