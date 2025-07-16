use std::{
    borrow::Cow,
    fmt::{self, Display},
};

use serde::{
    Deserialize, Deserializer,
    de::{
        MapAccess, SeqAccess, Visitor,
        value::{MapAccessDeserializer, SeqAccessDeserializer},
    },
};
use serde_json::Value;

use crate::{
    err::{Error, ErrorCode, ErrorData},
    msg::{Batch, Id, Message, Request, RequestParams, Response},
    schema,
};

struct BatchVisitor;

impl<'de> Visitor<'de> for BatchVisitor {
    type Value = Batch;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(schema::batch::EXPECTED_SCHEMA)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut messages = Vec::with_capacity(seq.size_hint().unwrap_or_default());

        while let Some(message) = seq.next_element::<Message>()? {
            messages.push(message);
        }

        Ok(messages.into())
    }
}

impl<'de> Deserialize<'de> for Batch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(BatchVisitor)
    }
}

struct ErrorVisitor;

impl ErrorVisitor {
    fn visit_field_code<E: serde::de::Error>(value: Value) -> std::result::Result<ErrorCode, E> {
        let code = deserialize_i64(schema::error::fields::CODE, value)?;
        ErrorCode::try_from(code).map_err(|err| map_field_error(schema::error::fields::CODE, err))
    }

    fn visit_field_message<E: serde::de::Error>(value: Value) -> std::result::Result<String, E> {
        deserialize_string(schema::error::fields::MESSAGE, value)
    }

    fn visit_field_data<E: serde::de::Error>(value: Value) -> std::result::Result<ErrorData, E> {
        ErrorData::deserialize(value)
            .map_err(|err| make_field_error(schema::error::fields::DATA, err))
    }

    fn visit_unknown<E: serde::de::Error>(field: &str) -> E {
        serde::de::Error::unknown_field(field, schema::error::FIELD_NAMES)
    }
}

impl<'de> Visitor<'de> for ErrorVisitor {
    type Value = Error;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(schema::error::EXPECTED_SCHEMA)
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut code = None;
        let mut message = None;
        let mut data = None;

        while let Some((key, value)) = map.next_entry::<Cow<str>, Value>()? {
            match &*key {
                schema::error::fields::CODE => code = Self::visit_field_code(value).map(Some)?,
                schema::error::fields::MESSAGE => {
                    message = Self::visit_field_message(value).map(Some)?
                }
                schema::error::fields::DATA => data = Self::visit_field_data(value).map(Some)?,
                unknown => return Err(Self::visit_unknown(unknown)),
            }
        }

        let mut error = Error::new(
            unwrap_or_missing_error(schema::error::fields::CODE, code)?,
            unwrap_or_missing_error(schema::error::fields::MESSAGE, message)?,
        );

        if let Some(data) = data {
            error = error.with_data(data);
        }

        Ok(error)
    }
}

impl<'de> Deserialize<'de> for ErrorData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(ErrorData::new(value))
    }
}

struct IdVisitor;

impl IdVisitor {
    const MSG_NUMBER_TOO_LARGE: &str = "number too large: expected a signed 64-bit integer";
}

impl<'de> Visitor<'de> for IdVisitor {
    type Value = Id;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(schema::id::EXPECTED_SCHEMA)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Id::Null)
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(v.to_owned())
    }

    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::Str(v))
    }

    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::Num(v))
    }

    fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v <= i64::MAX as u64 {
            Ok(Self::Value::Num(v as i64))
        } else {
            Err(serde::de::Error::custom(Self::MSG_NUMBER_TOO_LARGE))
        }
    }
}

struct MessageVisitor;

impl<'de> Visitor<'de> for MessageVisitor {
    type Value = Message;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(schema::message::EXPECTED_SCHEMA)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let raw_value = Value::deserialize(MapAccessDeserializer::new(map))?;

        if let Ok(request) = Request::deserialize(&raw_value) {
            return Ok(request.into());
        }

        if let Ok(response) = Response::deserialize(&raw_value) {
            return Ok(response.into());
        }

        Err(serde::de::Error::custom(
            "object is neither a Request nor a Response",
        ))
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(MessageVisitor)
    }
}

struct RequestParamsVisitor;

impl<'de> Visitor<'de> for RequestParamsVisitor {
    type Value = RequestParams;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(schema::params::EXPECTED_SCHEMA)
    }

    fn visit_seq<A>(self, seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let array = Deserialize::deserialize(SeqAccessDeserializer::new(seq))?;
        Ok(RequestParams::Array(array))
    }

    fn visit_map<A>(self, map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let object = Deserialize::deserialize(MapAccessDeserializer::new(map))?;
        Ok(RequestParams::Object(object))
    }
}

struct RequestVisitor;

impl RequestVisitor {
    fn visit_field_id<E: serde::de::Error>(value: Value) -> std::result::Result<Id, E> {
        Id::deserialize(value).map_err(|err| make_field_error(schema::request::fields::ID, err))
    }

    fn visit_field_method<E: serde::de::Error>(value: Value) -> std::result::Result<String, E> {
        deserialize_string(schema::request::fields::METHOD, value)
    }

    fn visit_field_params<E: serde::de::Error>(
        value: Value,
    ) -> std::result::Result<RequestParams, E> {
        RequestParams::deserialize(value)
            .map_err(|err| make_field_error(schema::request::fields::PARAMS, err))
    }

    fn visit_unknown<E: serde::de::Error>(field: &str) -> E {
        serde::de::Error::unknown_field(field, schema::request::FIELD_NAMES)
    }
}

impl<'de> Visitor<'de> for RequestVisitor {
    type Value = Request;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(schema::request::EXPECTED_SCHEMA)
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut jsonrpc = None;
        let mut id = None;
        let mut method = None;
        let mut params = None;

        while let Some((key, value)) = map.next_entry::<Cow<str>, Value>()? {
            match &*key {
                schema::request::fields::JSONRPC => {
                    jsonrpc =
                        deserialize_string(schema::request::fields::JSONRPC, value).map(Some)?;
                }
                schema::request::fields::ID => id = Self::visit_field_id(value).map(Some)?,
                schema::request::fields::METHOD => {
                    method = Self::visit_field_method(value).map(Some)?
                }
                schema::request::fields::PARAMS => {
                    params = Self::visit_field_params(value).map(Some)?
                }
                unknown => return Err(Self::visit_unknown(unknown)),
            }
        }

        ensure_valid_jsonrpc_version(
            schema::request::fields::JSONRPC,
            unwrap_or_missing_error(schema::request::fields::JSONRPC, jsonrpc)?,
        )?;

        let method = unwrap_or_missing_error(schema::request::fields::METHOD, method)?;

        Ok(match id {
            Some(id) => Request::new_request(id, method, params),
            _ => Request::new_notification(method, params),
        })
    }
}

struct ResponseVisitor;

impl ResponseVisitor {
    const MSG_PAYLOAD_AMBIGUITY: &str =
        "`result` and `error` cannot both be present in the same response";
    const MSG_MISSING_PAYLOAD: &str = "response must contain either `result` or `error`";

    fn visit_field_id<E: serde::de::Error>(value: Value) -> std::result::Result<Id, E> {
        Id::deserialize(value).map_err(|err| make_field_error(schema::response::fields::ID, err))
    }

    fn visit_field_error<E: serde::de::Error>(value: Value) -> std::result::Result<Error, E> {
        Error::deserialize(value)
            .map_err(|err| make_field_error(schema::response::fields::ERROR, err))
    }

    fn visit_unknown<E: serde::de::Error>(field: &str) -> E {
        serde::de::Error::unknown_field(field, schema::response::FIELD_NAMES)
    }
}

impl<'de> Visitor<'de> for ResponseVisitor {
    type Value = Response;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(schema::response::EXPECTED_SCHEMA)
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut jsonrpc = None;
        let mut id = None;
        let mut result = None;
        let mut error = None;

        while let Some((key, value)) = map.next_entry::<Cow<str>, Value>()? {
            match &*key {
                schema::response::fields::JSONRPC => {
                    jsonrpc =
                        deserialize_string(schema::response::fields::JSONRPC, value).map(Some)?;
                }
                schema::response::fields::ID => id = Self::visit_field_id(value).map(Some)?,
                schema::response::fields::RESULT => result = Some(value),
                schema::response::fields::ERROR => {
                    error = Self::visit_field_error(value).map(Some)?
                }
                unknown => return Err(Self::visit_unknown(unknown)),
            }
        }

        ensure_valid_jsonrpc_version(
            schema::response::fields::JSONRPC,
            unwrap_or_missing_error(schema::response::fields::JSONRPC, jsonrpc)?,
        )?;

        let id = unwrap_or_missing_error(schema::response::fields::ID, id)?;

        match (result, error) {
            (Some(result), None) => Ok(Response::new_success(id, result)),
            (None, Some(error)) => Ok(Response::new_error(id, error)),
            (None, None) => Err(serde::de::Error::custom(Self::MSG_MISSING_PAYLOAD)),
            (Some(_), Some(_)) => Err(serde::de::Error::custom(Self::MSG_PAYLOAD_AMBIGUITY)),
        }
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ErrorVisitor)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(IdVisitor)
    }
}

impl<'de> Deserialize<'de> for Request {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(RequestVisitor)
    }
}

impl<'de> Deserialize<'de> for RequestParams {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RequestParamsVisitor)
    }
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ResponseVisitor)
    }
}

fn make_field_error<R, E>(field: &'static str, reason: R) -> E
where
    R: Display,
    E: serde::de::Error,
{
    E::custom(format!("field `{}` contains an {}", field, reason))
}

fn map_field_error<E: serde::de::Error>(field: &'static str, error: Error) -> E {
    match error.data() {
        Some(data) => make_field_error(field, data),
        _ => make_field_error(field, error.message()),
    }
}

fn unwrap_or_missing_error<T, E: serde::de::Error>(
    field: &'static str,
    value: Option<T>,
) -> std::result::Result<T, E> {
    value.ok_or_else(|| serde::de::Error::missing_field(field))
}

fn deserialize_i64<E>(field: &'static str, value: Value) -> Result<i64, E>
where
    E: serde::de::Error,
{
    i64::deserialize(value).map_err(|err| make_field_error(field, err))
}

fn deserialize_string<E>(field: &'static str, value: Value) -> Result<String, E>
where
    E: serde::de::Error,
{
    String::deserialize(value).map_err(|err| make_field_error(field, err))
}

fn ensure_valid_jsonrpc_version<E: serde::de::Error>(
    field: &str,
    jsonrpc: String,
) -> std::result::Result<(), E> {
    if jsonrpc == schema::VERSION {
        Ok(())
    } else {
        Err(serde::de::Error::custom(format!(
            "invalid value for field `{}`: expected version \"{}\", got \"{}\"",
            field,
            schema::VERSION,
            jsonrpc
        )))
    }
}
