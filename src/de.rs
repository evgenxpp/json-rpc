use std::{any::type_name, fmt};

use serde::{
    Deserialize, Deserializer,
    de::{
        self, MapAccess, SeqAccess, Visitor,
        value::{MapAccessDeserializer, SeqAccessDeserializer},
    },
};
use serde_json::Value;

use crate::{
    err::{Error, ErrorCode, ErrorData},
    msg::{Id, Message, Notification, Parameters, Request, Response},
    schema,
};

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use schema::id::DSL_SCHEMA;

        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write_dsl_schema(formatter, DSL_SCHEMA)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Id::default())
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Id::U64(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Id::Str(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_string(v.to_owned())
            }
        }

        deserializer.deserialize_any(IdVisitor)
    }
}

impl<'de> Deserialize<'de> for Parameters {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use schema::parameters::DSL_SCHEMA;

        struct ParametersVisitor;

        impl<'de> Visitor<'de> for ParametersVisitor {
            type Value = Parameters;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write_dsl_schema(formatter, DSL_SCHEMA)
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let array = Deserialize::deserialize(SeqAccessDeserializer::new(seq))?;
                Ok(Parameters::Array(array))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let object = Deserialize::deserialize(MapAccessDeserializer::new(map))?;
                Ok(Parameters::Object(object))
            }
        }

        deserializer.deserialize_any(ParametersVisitor)
    }
}

impl<'de> Deserialize<'de> for Notification {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use schema::notification::{DSL_SCHEMA, FIELD_NAMES, fields};

        struct NotificationVisitor;

        impl<'de> Visitor<'de> for NotificationVisitor {
            type Value = Notification;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write_dsl_schema(formatter, DSL_SCHEMA)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut jsonrpc: Option<String> = None;
                let mut method: Option<String> = None;
                let mut params: Option<Parameters> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        fields::JSONRPC => {
                            jsonrpc = de_to_value(&mut map, fields::JSONRPC, jsonrpc)?;
                        }
                        fields::METHOD => {
                            method = de_to_value(&mut map, fields::METHOD, method)?;
                        }
                        fields::PARAMS => {
                            params = de_to_value(&mut map, fields::PARAMS, params)?;
                        }
                        unknown => {
                            return Err(make_unknown_field_error(unknown, FIELD_NAMES));
                        }
                    }
                }

                validate_jsonrpc_version(fields::JSONRPC, jsonrpc)?;

                let method = unwrap_or_missing_error(fields::METHOD, method)?;

                Ok(Notification::new(method, params))
            }
        }

        deserializer.deserialize_struct(
            type_name::<Notification>(),
            FIELD_NAMES,
            NotificationVisitor,
        )
    }
}

impl<'de> Deserialize<'de> for Request {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use schema::request::{DSL_SCHEMA, FIELD_NAMES, fields};

        struct RequestVisitor;

        impl<'de> Visitor<'de> for RequestVisitor {
            type Value = Request;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write_dsl_schema(formatter, DSL_SCHEMA)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut jsonrpc: Option<String> = None;
                let mut id: Option<Id> = None;
                let mut method: Option<String> = None;
                let mut params: Option<Parameters> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        fields::JSONRPC => {
                            jsonrpc = de_to_value(&mut map, fields::JSONRPC, jsonrpc)?;
                        }
                        fields::ID => {
                            id = de_to_value(&mut map, fields::ID, id)?;
                        }
                        fields::METHOD => {
                            method = de_to_value(&mut map, fields::METHOD, method)?;
                        }
                        fields::PARAMS => {
                            params = de_to_value(&mut map, fields::PARAMS, params)?;
                        }
                        unknown => {
                            return Err(make_unknown_field_error(unknown, FIELD_NAMES));
                        }
                    }
                }

                validate_jsonrpc_version(fields::JSONRPC, jsonrpc)?;

                let id = unwrap_or_missing_error(fields::ID, id)?;
                let method = unwrap_or_missing_error(fields::METHOD, method)?;

                Ok(Request::new(id, method, params))
            }
        }

        deserializer.deserialize_struct(type_name::<Request>(), FIELD_NAMES, RequestVisitor)
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let code = i64::deserialize(deserializer)?;

        ErrorCode::create(code).map_err(|err| {
            let msg = err
                .data
                .map(|data| data.to_string())
                .unwrap_or(err.message.into());

            de::Error::custom(msg)
        })
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

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use schema::error::{DSL_SCHEMA, FIELD_NAMES, fields};

        struct ErrorVisitor;

        impl<'de> Visitor<'de> for ErrorVisitor {
            type Value = Error;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write_dsl_schema(formatter, DSL_SCHEMA)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut code: Option<ErrorCode> = None;
                let mut message: Option<String> = None;
                let mut data: Option<ErrorData> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        fields::CODE => {
                            code = de_to_value(&mut map, fields::CODE, code)?;
                        }
                        fields::MESSAGE => {
                            message = de_to_value(&mut map, fields::MESSAGE, message)?;
                        }
                        fields::DATA => {
                            data = de_to_value(&mut map, fields::DATA, data)?;
                        }
                        unknown => {
                            return Err(make_unknown_field_error(unknown, FIELD_NAMES));
                        }
                    }
                }

                let mut error = Error::new(
                    unwrap_or_missing_error(fields::CODE, code)?,
                    unwrap_or_missing_error(fields::MESSAGE, message)?,
                );

                if let Some(data) = data {
                    error = error.with_data(data);
                }

                Ok(error)
            }
        }

        deserializer.deserialize_struct(type_name::<Error>(), FIELD_NAMES, ErrorVisitor)
    }
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use schema::response::{DSL_SCHEMA, FIELD_NAMES, fields};

        const MSG_MISSING_PAYLOAD: &str = "response must contain either `result` or `error`";
        const MSG_PAYLOAD_AMBIGUITY: &str =
            "`result` and `error` cannot both be present in the same response";

        struct ResponseVisitor;

        impl<'de> Visitor<'de> for ResponseVisitor {
            type Value = Response;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write_dsl_schema(formatter, DSL_SCHEMA)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut jsonrpc: Option<String> = None;
                let mut id: Option<Id> = None;
                let mut result: Option<Value> = None;
                let mut error: Option<Error> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        fields::JSONRPC => {
                            jsonrpc = de_to_value(&mut map, fields::JSONRPC, jsonrpc)?;
                        }
                        fields::ID => {
                            id = de_to_value(&mut map, fields::ID, id)?;
                        }
                        fields::RESULT => {
                            result = de_to_value(&mut map, fields::RESULT, result)?;
                        }
                        fields::ERROR => {
                            error = de_to_value(&mut map, fields::ERROR, error)?;
                        }
                        unknown => {
                            return Err(make_unknown_field_error(unknown, FIELD_NAMES));
                        }
                    }
                }

                validate_jsonrpc_version(fields::JSONRPC, jsonrpc)?;

                let id = unwrap_or_missing_error(fields::ID, id)?;

                match (result, error) {
                    (Some(result), None) => Ok(Response::new_success(id, result)),
                    (None, Some(error)) => Ok(Response::new_error(id, error)),
                    (None, None) => Err(de::Error::custom(MSG_MISSING_PAYLOAD)),
                    (Some(_), Some(_)) => Err(de::Error::custom(MSG_PAYLOAD_AMBIGUITY)),
                }
            }
        }

        deserializer.deserialize_struct(type_name::<Response>(), FIELD_NAMES, ResponseVisitor)
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw: Value = Value::deserialize(deserializer)?;

        Request::deserialize(&raw)
            .map(Message::Request)
            .or_else(|_| Notification::deserialize(&raw).map(Message::Notification))
            .or_else(|_| Response::deserialize(&raw).map(Message::Response))
            .map_err(de::Error::custom)
    }
}

fn de_to_value<'de, A, T, E>(
    map: &mut A,
    field: &'static str,
    value: Option<T>,
) -> Result<Option<T>, E>
where
    A: MapAccess<'de>,
    T: Deserialize<'de>,
    E: de::Error,
{
    if value.is_some() {
        return Err(de::Error::duplicate_field(field));
    }

    map.next_value::<T>()
        .map_err(|err| E::custom(format!("field `{}` contains an {}", field, err)))
        .map(Some)
}

fn make_unknown_field_error<E>(unknown: &str, fields: &'static [&str]) -> E
where
    E: de::Error,
{
    de::Error::unknown_field(unknown, fields)
}

fn unwrap_or_missing_error<T, E: de::Error>(field: &'static str, value: Option<T>) -> Result<T, E> {
    value.ok_or_else(|| de::Error::missing_field(field))
}

fn validate_jsonrpc_version<E: de::Error>(
    field: &'static str,
    jsonrpc: Option<String>,
) -> Result<(), E> {
    let jsonrpc = unwrap_or_missing_error(field, jsonrpc)?;

    if jsonrpc == schema::VERSION {
        return Ok(());
    }

    Err(de::Error::custom(format!(
        "invalid value for field `{}`: expected version \"{}\", got \"{}\"",
        field,
        schema::VERSION,
        jsonrpc
    )))
}

fn write_dsl_schema(formatter: &mut fmt::Formatter, dsl_schema: &'static str) -> fmt::Result {
    write!(formatter, "`DSL: {}`", dsl_schema)
}
