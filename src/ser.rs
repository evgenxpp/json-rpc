use serde::{Serialize, ser::SerializeStruct};

use crate::{
    err::Error,
    msg::{Id, Request, RequestParams, Response},
    schema,
};

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(schema::error::NAME, 3)?;
        state.serialize_field(schema::error::fields::CODE, &self.code().as_i64())?;
        state.serialize_field(schema::error::fields::MESSAGE, self.message())?;

        if let Some(data) = self.data() {
            state.serialize_field(schema::error::fields::DATA, data)?;
        }

        state.end()
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Num(value) => serializer.serialize_i64(*value),
            Self::Str(value) => serializer.serialize_str(value),
        }
    }
}

impl Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(schema::request::NAME, 3)?;

        state.serialize_field(schema::request::fields::ID, &self.id())?;
        state.serialize_field(schema::request::fields::METHOD, self.method())?;

        if let Some(params) = self.params() {
            state.serialize_field(schema::request::fields::PARAMS, params)?;
        }

        state.end()
    }
}

impl Serialize for RequestParams {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Array(array) => serializer.collect_seq(array),
            Self::Object(object) => serializer.collect_map(object),
        }
    }
}

impl Serialize for Response {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(schema::response::NAME, 2)?;

        state.serialize_field(schema::response::fields::ID, &self.id())?;

        match self.result() {
            Ok(result) => state.serialize_field(schema::response::fields::RESULT, result)?,
            Err(error) => state.serialize_field(schema::response::fields::ERROR, error)?,
        }

        state.end()
    }
}
