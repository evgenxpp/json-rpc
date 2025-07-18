use serde_json::{Map, Value};
use std::fmt::{self, Display};

use crate::err::Error;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum Id {
    #[default]
    Null,
    U64(u64),
    Str(String),
}

impl From<u64> for Id {
    fn from(value: u64) -> Self {
        Id::U64(value)
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id::Str(value)
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Id::Str(value.to_owned())
    }
}

impl Id {
    const NULL_STR: &str = "null";

    pub fn is_i64(&self) -> bool {
        matches!(self, Id::U64(_))
    }

    pub fn is_str(&self) -> bool {
        matches!(self, Id::Str(_))
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Id::U64(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Id::Str(id) => Some(id),
            _ => None,
        }
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Id::Null => write!(f, "{}", Self::NULL_STR),
            Id::U64(id) => write!(f, "{}", id),
            Id::Str(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Parameters {
    Array(Vec<Value>),
    Object(Map<String, Value>),
}

impl From<Vec<Value>> for Parameters {
    fn from(value: Vec<Value>) -> Self {
        Parameters::Array(value)
    }
}

impl From<Map<String, Value>> for Parameters {
    fn from(value: Map<String, Value>) -> Self {
        Parameters::Object(value)
    }
}

impl Parameters {
    pub fn is_array(&self) -> bool {
        matches!(self, Parameters::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Parameters::Object(_))
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Parameters::Array(array) => Some(array),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        match self {
            Parameters::Object(object) => Some(object),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub method: String,
    pub params: Option<Parameters>,
}

impl Notification {
    pub fn new<M>(method: M, params: Option<Parameters>) -> Self
    where
        M: Into<String>,
    {
        Self {
            params,
            method: method.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub id: Id,
    pub method: String,
    pub params: Option<Parameters>,
}

impl Request {
    pub fn new<I, M>(id: I, method: M, params: Option<Parameters>) -> Self
    where
        I: Into<Id>,
        M: Into<String>,
    {
        Self {
            params,
            id: id.into(),
            method: method.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub id: Id,
    pub result: Result<Value, Error>,
}

impl Response {
    pub fn new<I>(id: I, result: Result<Value, Error>) -> Self
    where
        I: Into<Id>,
    {
        Self {
            result,
            id: id.into(),
        }
    }

    pub fn new_success<I, R>(id: I, result: R) -> Self
    where
        I: Into<Id>,
        R: Into<Value>,
    {
        Self::new(id, Ok(result.into()))
    }

    pub fn new_error<I>(id: I, error: Error) -> Self
    where
        I: Into<Id>,
    {
        Self::new(id, Err(error))
    }

    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    pub fn is_error(&self) -> bool {
        self.result.is_err()
    }

    pub fn as_success(&self) -> Option<&Value> {
        self.result.as_ref().ok()
    }

    pub fn as_error(&self) -> Option<&Error> {
        self.result.as_ref().err()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Notification(Notification),
    Request(Request),
    Response(Response),
}

impl From<Notification> for Message {
    fn from(value: Notification) -> Self {
        Message::Notification(value)
    }
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

impl Message {
    pub fn is_notification(&self) -> bool {
        matches!(self, Message::Notification(_))
    }

    pub fn is_request(&self) -> bool {
        matches!(self, Message::Request(_))
    }

    pub fn is_response(&self) -> bool {
        matches!(self, Message::Response(_))
    }

    pub fn as_notification(&self) -> Option<&Notification> {
        match self {
            Message::Notification(notification) => Some(notification),
            _ => None,
        }
    }

    pub fn as_request(&self) -> Option<&Request> {
        match self {
            Message::Request(request) => Some(request),
            _ => None,
        }
    }

    pub fn as_response(&self) -> Option<&Response> {
        match self {
            Message::Response(response) => Some(response),
            _ => None,
        }
    }
}
