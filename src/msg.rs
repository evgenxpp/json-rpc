use serde_json::{Map, Value};
use std::fmt::{self, Display};
use std::result::Result as StdResult;

use crate::err::{Error, Result};

#[derive(Debug, PartialEq)]
pub enum Id {
    Num(i64),
    Str(String),
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Id::Num(id) => write!(f, "{id}"),
            Id::Str(id) => write!(f, "{id}"),
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
    id: Option<Id>,
    result: Result<Value>,
}

impl Response {
    pub fn new_success(id: Id, result: Value) -> Self {
        Self {
            id: Some(id),
            result: Ok(result),
        }
    }

    pub fn new_error(id: Option<Id>, error: Error) -> Self {
        Self {
            id,
            result: Err(error),
        }
    }

    pub fn id(&self) -> Option<&Id> {
        self.id.as_ref()
    }

    pub fn result(&self) -> StdResult<&Value, &Error> {
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
pub struct BatchRequest(Vec<Request>);

impl BatchRequest {
    pub fn new(requests: Vec<Request>) -> Self {
        Self(requests)
    }

    pub fn requests(&self) -> &Vec<Request> {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.requests().is_empty()
    }
}

impl From<Vec<Request>> for BatchRequest {
    fn from(value: Vec<Request>) -> Self {
        BatchRequest::new(value)
    }
}

impl From<BatchRequest> for Vec<Request> {
    fn from(value: BatchRequest) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub struct BatchResponse(Vec<Response>);

impl BatchResponse {
    pub fn new(responses: Vec<Response>) -> Self {
        Self(responses)
    }

    pub fn responses(&self) -> &Vec<Response> {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.responses().is_empty()
    }
}

impl From<Vec<Response>> for BatchResponse {
    fn from(value: Vec<Response>) -> Self {
        BatchResponse::new(value)
    }
}

impl From<BatchResponse> for Vec<Response> {
    fn from(value: BatchResponse) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub enum Message {
    Request(Request),
    Response(Response),
    BatchRequest(BatchRequest),
    BatchResponse(BatchResponse),
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

impl From<BatchRequest> for Message {
    fn from(value: BatchRequest) -> Self {
        Message::BatchRequest(value)
    }
}

impl From<BatchResponse> for Message {
    fn from(value: BatchResponse) -> Self {
        Message::BatchResponse(value)
    }
}
