use serde_json::{Map, Value};
use std::fmt::{self, Display};

use crate::err::{Error, Result};

#[derive(Debug, PartialEq)]
pub enum Id {
    Null,
    Num(i64),
    Str(String),
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Id::Null => write!(f, "null"),
            Id::Num(id) => write!(f, "{}", id),
            Id::Str(id) => write!(f, "{}", id),
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

#[derive(Debug)]
pub enum RequestParams {
    Array(Vec<Value>),
    Object(Map<String, Value>),
}

impl From<Vec<Value>> for RequestParams {
    fn from(value: Vec<Value>) -> Self {
        RequestParams::Array(value)
    }
}

impl From<Map<String, Value>> for RequestParams {
    fn from(value: Map<String, Value>) -> Self {
        RequestParams::Object(value)
    }
}

#[derive(Debug)]
pub struct Request {
    id: Option<Id>,
    method: String,
    params: Option<RequestParams>,
}

impl Request {
    pub fn new_request(id: Id, method: String, params: Option<RequestParams>) -> Self {
        Self::new(Some(id), method, params)
    }

    pub fn new_notification(method: String, params: Option<RequestParams>) -> Self {
        Self::new(None, method, params)
    }

    pub fn id(&self) -> Option<&Id> {
        self.id.as_ref()
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn params(&self) -> Option<&RequestParams> {
        self.params.as_ref()
    }

    pub fn is_request(&self) -> bool {
        self.id().is_some()
    }

    pub fn is_notification(&self) -> bool {
        self.id().is_none()
    }

    fn new(id: Option<Id>, method: String, params: Option<RequestParams>) -> Self {
        Self { id, method, params }
    }
}

#[derive(Debug)]
pub struct Response {
    id: Id,
    result: Result<Value>,
}

impl Response {
    pub fn new_success(id: Id, result: Value) -> Self {
        Self {
            id,
            result: Ok(result),
        }
    }

    pub fn new_error(id: Id, error: Error) -> Self {
        Self {
            id,
            result: Err(error),
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn result(&self) -> std::result::Result<&Value, &Error> {
        self.result.as_ref()
    }

    pub fn is_success(&self) -> bool {
        self.result().is_ok()
    }

    pub fn is_error(&self) -> bool {
        self.result().is_err()
    }
}

#[derive(Debug)]
pub enum Message {
    Request(Request),
    Response(Response),
}

impl From<Request> for Message {
    fn from(value: Request) -> Self {
        Message::Request(value)
    }
}

impl From<Response> for Message {
    fn from(value: Response) -> Self {
        Message::Response(value)
    }
}

#[derive(Debug)]
pub struct Batch(Vec<Message>);

impl Batch {
    pub fn new(requests: Vec<Message>) -> Self {
        Self(requests)
    }

    pub fn messages(&self) -> &Vec<Message> {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.messages().is_empty()
    }
}

impl From<Vec<Message>> for Batch {
    fn from(value: Vec<Message>) -> Self {
        Batch::new(value)
    }
}

impl From<Batch> for Vec<Message> {
    fn from(value: Batch) -> Self {
        value.0
    }
}
