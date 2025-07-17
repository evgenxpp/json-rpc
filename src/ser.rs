use std::any::type_name;

use serde::{Serialize, Serializer, ser::SerializeStruct};

use crate::{
    err::{Error, ErrorCode, ErrorData},
    msg::{Id, Message, Notification, Parameters, Request, Response},
    schema,
};

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Id::Null => serializer.serialize_unit(),
            Id::U64(id) => serializer.serialize_u64(*id),
            Id::Str(id) => serializer.serialize_str(id),
        }
    }
}

impl Serialize for Parameters {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Parameters::Array(params) => params.serialize(serializer),
            Parameters::Object(params) => params.serialize(serializer),
        }
    }
}

impl Serialize for Notification {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(type_name::<Notification>(), 3)?;

        state.serialize_field(schema::request::fields::JSONRPC, schema::VERSION)?;
        state.serialize_field(schema::request::fields::METHOD, &self.method)?;

        if let Some(params) = &self.params {
            state.serialize_field(schema::request::fields::PARAMS, params)?;
        }

        state.end()
    }
}

impl Serialize for Request {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(type_name::<Request>(), 4)?;

        state.serialize_field(schema::request::fields::JSONRPC, schema::VERSION)?;
        state.serialize_field(schema::request::fields::ID, &self.id)?;
        state.serialize_field(schema::request::fields::METHOD, &self.method)?;

        if let Some(params) = &self.params {
            state.serialize_field(schema::request::fields::PARAMS, params)?;
        }

        state.end()
    }
}

impl Serialize for ErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.as_i64())
    }
}

impl Serialize for ErrorData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(type_name::<Error>(), 3)?;

        state.serialize_field(schema::error::fields::CODE, &self.code)?;
        state.serialize_field(schema::error::fields::MESSAGE, &self.message)?;

        if let Some(data) = &self.data {
            state.serialize_field(schema::error::fields::DATA, data)?;
        }

        state.end()
    }
}

impl Serialize for Response {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(type_name::<Response>(), 3)?;

        state.serialize_field(schema::response::fields::JSONRPC, schema::VERSION)?;
        state.serialize_field(schema::response::fields::ID, &self.id)?;

        match &self.result {
            Ok(result) => state.serialize_field(schema::response::fields::RESULT, result)?,
            Err(error) => state.serialize_field(schema::response::fields::ERROR, error)?,
        }

        state.end()
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Message::Notification(notification) => notification.serialize(serializer),
            Message::Request(request) => request.serialize(serializer),
            Message::Response(response) => response.serialize(serializer),
        }
    }
}
