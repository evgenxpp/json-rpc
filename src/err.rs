use std::{
    borrow::Cow,
    fmt::{self, Display},
    result::Result as StdResult,
};

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug, Clone)]
pub enum ErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    ServerError(i64),
}

impl ErrorCode {
    const CODE_PARSE_ERROR: i64 = -32700;
    const CODE_INVALID_REQUEST: i64 = -32600;
    const CODE_METHOD_NOT_FOUND: i64 = -32601;
    const CODE_INVALID_PARAMS: i64 = -32602;
    const CODE_INTERNAL_ERROR: i64 = -32603;
    const CODE_SERVER_ERROR_MIN: i64 = -32099;
    const CODE_SERVER_ERROR_MAX: i64 = -32000;

    const ERR_INVALID_CODE: &str =
        "invalid error code: must be predefined or in range -32099 to -32000";

    pub fn create(code: i64) -> Result<Self> {
        let error_code = match code {
            Self::CODE_PARSE_ERROR => Self::ParseError,
            Self::CODE_INVALID_REQUEST => Self::InvalidRequest,
            Self::CODE_METHOD_NOT_FOUND => Self::MethodNotFound,
            Self::CODE_INVALID_PARAMS => Self::InvalidParams,
            Self::CODE_INTERNAL_ERROR => Self::InternalError,
            Self::CODE_SERVER_ERROR_MIN..=Self::CODE_SERVER_ERROR_MAX => Self::ServerError(code),
            _ => {
                return Error::new_default(ErrorCode::InvalidRequest)
                    .with_data(Self::ERR_INVALID_CODE)
                    .into();
            }
        };

        Ok(error_code)
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            ErrorCode::ParseError => ErrorCode::CODE_PARSE_ERROR,
            ErrorCode::InvalidRequest => ErrorCode::CODE_INVALID_REQUEST,
            ErrorCode::MethodNotFound => ErrorCode::CODE_METHOD_NOT_FOUND,
            ErrorCode::InvalidParams => ErrorCode::CODE_INVALID_PARAMS,
            ErrorCode::InternalError => ErrorCode::CODE_INTERNAL_ERROR,
            ErrorCode::ServerError(code) => *code,
        }
    }
}

impl From<ErrorCode> for i64 {
    fn from(value: ErrorCode) -> Self {
        value.as_i64()
    }
}

impl TryFrom<i64> for ErrorCode {
    type Error = Error;

    fn try_from(value: i64) -> StdResult<Self, Self::Error> {
        Self::create(value)
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_i64())
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    code: ErrorCode,
    message: Cow<'static, str>,
    data: Option<Cow<'static, str>>,
}

impl Error {
    const MSG_PARSE_ERROR: &str = "Parse error";
    const MSG_INVALID_REQUEST: &str = "Invalid Request";
    const MSG_METHOD_NOT_FOUND: &str = "Method not found";
    const MSG_INVALID_PARAMS: &str = "Invalid params";
    const MSG_INTERNAL_ERROR: &str = "Internal error";
    const MSG_SERVER_ERROR: &str = "Server error";

    pub fn new<T>(code: ErrorCode, message: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn new_default(code: ErrorCode) -> Self {
        let message = match code {
            ErrorCode::ParseError => Self::MSG_PARSE_ERROR,
            ErrorCode::InvalidRequest => Self::MSG_INVALID_REQUEST,
            ErrorCode::MethodNotFound => Self::MSG_METHOD_NOT_FOUND,
            ErrorCode::InvalidParams => Self::MSG_INVALID_PARAMS,
            ErrorCode::InternalError => Self::MSG_INTERNAL_ERROR,
            ErrorCode::ServerError(_) => Self::MSG_SERVER_ERROR,
        };

        Self {
            code,
            data: None,
            message: Cow::Borrowed(message),
        }
    }

    pub fn with_data<T>(mut self, data: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.data = Some(data.into());
        self
    }

    pub fn code(&self) -> &ErrorCode {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn data(&self) -> Option<&str> {
        self.data.as_deref()
    }
}

impl<T> From<Error> for Result<T> {
    fn from(value: Error) -> Self {
        Self::Err(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON-RPC Error (code {}): {}", self.code, self.message)?;

        if let Some(data) = &self.data {
            write!(f, ". Data: `{}`", data)?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {}
