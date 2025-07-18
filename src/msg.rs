use serde_json::{Map, Value};
use std::fmt::{self, Display};

use crate::err::Error;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum Id {
    #[default]
    Null,
    I64(i64),
    Str(String),
}

impl From<i64> for Id {
    fn from(value: i64) -> Self {
        Id::I64(value)
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

    pub fn is_null(&self) -> bool {
        matches!(self, Id::Null)
    }

    pub fn is_i64(&self) -> bool {
        matches!(self, Id::I64(_))
    }

    pub fn is_str(&self) -> bool {
        matches!(self, Id::Str(_))
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Id::I64(id) => Some(*id),
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
            Id::I64(id) => write!(f, "{}", id),
            Id::Str(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id() {
        // Null case
        assert!(
            Id::Null.is_null() && !Id::Null.is_i64() && !Id::Null.is_str(),
            "Id::Null is not correctly recognized as null"
        );
        assert_eq!(
            Id::Null.to_string(),
            "null",
            "Id::Null stringifies incorrectly: got {:?}, expected \"null\"",
            Id::Null.to_string()
        );

        // Integer case
        let expected = i64::MAX;
        let id = Id::from(expected);
        assert!(
            id.is_i64() && !id.is_null() && !id.is_str(),
            "Id from i64 is not correctly recognized as is_i64()"
        );
        assert_eq!(
            id.as_i64(),
            Some(expected),
            "Id::as_i64() returned {:?}, expected Some({})",
            id.as_i64(),
            expected
        );
        assert_eq!(
            id.to_string(),
            expected.to_string(),
            "Id::to_string() returned {:?}, expected {:?}",
            id.to_string(),
            expected.to_string()
        );

        // String case
        let expected = "smth";
        let id = Id::from(expected.to_owned());
        assert!(
            id.is_str() && !id.is_null() && !id.is_i64(),
            "Id from String is not correctly recognized as is_str()"
        );
        assert_eq!(
            id.as_str(),
            Some(expected),
            "Id::as_str() returned {:?}, expected Some(\"{}\")",
            id.as_str(),
            expected
        );
        assert_eq!(
            id.to_string(),
            expected,
            "Id::to_string() returned {:?}, expected {:?}",
            id.to_string(),
            expected
        );
    }

    #[test]
    fn test_parameters() {
        // Array case
        let expected: Vec<Value> = vec![1.into(), 2.into(), 3.into()];
        let params = Parameters::from(expected.clone());
        assert!(
            params.is_array() && !params.is_object(),
            "Parameters from array are not correctly recognized as is_array()"
        );
        assert_eq!(
            params.as_array(),
            Some(expected.as_slice()),
            "Parameters::as_array() returned {:?}, expected {:?}",
            params.as_array(),
            expected
        );

        // Object case
        let mut map = Map::new();
        map.insert("val1".to_owned(), 1.into());
        map.insert("val2".to_owned(), true.into());
        let expected = map;

        let params = Parameters::from(expected.clone());
        assert!(
            params.is_object() && !params.is_array(),
            "Parameters from object are not correctly recognized as is_object()"
        );
        assert_eq!(
            params.as_object(),
            Some(&expected),
            "Parameters::as_object() returned {:?}, expected {:?}",
            params.as_object(),
            expected
        );
    }

    #[test]
    fn test_message() {
        // Notificatiob case
        let expected = Notification::new("notify", None);
        let message = Message::from(expected.clone());
        assert!(
            message.is_notification() && !message.is_request() && !message.is_response(),
            "Message from Notification is not correctly recognized as is_notification()"
        );
        assert_eq!(
            message.as_notification(),
            Some(&expected),
            "Message::as_notification() returned {:?}, expected {:?}",
            message.as_notification(),
            expected
        );

        // Request case
        let expected = Request::new(Id::Null, "notify", None);
        let message = Message::from(expected.clone());
        assert!(
            message.is_request() && !message.is_notification() && !message.is_response(),
            "Message from Request is not correctly recognized as is_request()"
        );
        assert_eq!(
            message.as_request(),
            Some(&expected),
            "Message::as_request() returned {:?}, expected {:?}",
            message.as_request(),
            expected
        );

        // Response case
        let expected = Response::new_success(Id::Null, "smth");
        let message = Message::from(expected.clone());
        assert!(
            message.is_response() && !message.is_notification() && !message.is_request(),
            "Message from Response is not correctly recognized as is_response()"
        );
        assert_eq!(
            message.as_response(),
            Some(&expected),
            "Message::as_response() returned {:?}, expected {:?}",
            message.as_response(),
            expected
        );
    }
}
