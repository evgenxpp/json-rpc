use std::{
    borrow::Cow,
    fmt::{self, Display},
    result::Result as StdResult,
};

use log::error;
use serde_json::Value;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug, Clone, PartialEq)]
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
                error!(
                    "Cannot construct ErrorCode from value `{}`. Reason: `{}`",
                    code,
                    ErrorCode::InvalidRequest
                );

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

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorData {
    pub value: Value,
}

impl ErrorData {
    pub fn new<T: Into<Value>>(value: T) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl Display for ErrorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value.to_string())
    }
}

impl<T: Into<Value>> From<T> for ErrorData {
    fn from(value: T) -> Self {
        ErrorData::new(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub code: ErrorCode,
    pub message: Cow<'static, str>,
    pub data: Option<ErrorData>,
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

    pub fn with_data<T: Into<ErrorData>>(mut self, data: T) -> Self {
        self.data = Some(data.into());
        self
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code() {
        fn assert_valid_error_code_with(num: i64, expected_code: ErrorCode) {
            let actual = ErrorCode::create(num);
            assert!(
                actual.is_ok(),
                "Error code {} is rejected, but should be accepted",
                num
            );

            let actual = actual.unwrap();
            assert_eq!(
                actual, expected_code,
                "Error code {} is accepted, but incorrect enum is produced: got {:?}, expected {:?}",
                num, actual, expected_code
            );

            assert_eq!(
                actual.as_i64(),
                num,
                "Enum variant {:?} returns unexpected value",
                actual
            );

            assert_eq!(
                actual.to_string(),
                num.to_string(),
                "Enum variant {:?} returns unexpected string representation",
                actual
            );
        }

        fn assert_invalid_error_code_with(num: i64) {
            let result = ErrorCode::create(num);
            assert!(
                result.is_err(),
                "Error code {} is accepted, but should be rejected",
                num
            );
        }

        // valid system error codes
        assert_valid_error_code_with(-32700, ErrorCode::ParseError);
        assert_valid_error_code_with(-32600, ErrorCode::InvalidRequest);
        assert_valid_error_code_with(-32601, ErrorCode::MethodNotFound);
        assert_valid_error_code_with(-32602, ErrorCode::InvalidParams);
        assert_valid_error_code_with(-32603, ErrorCode::InternalError);

        // valid server error range
        assert_valid_error_code_with(-32099, ErrorCode::ServerError(-32099));
        assert_valid_error_code_with(-32000, ErrorCode::ServerError(-32000));

        // invalid codes
        assert_invalid_error_code_with(0);
        assert_invalid_error_code_with(-32100);
        assert_invalid_error_code_with(-31999);
    }

    #[test]
    fn test_error() {
        fn assert_error_default_message(code: ErrorCode, msg: &str) {
            let code = Error::new_default(code);
            assert_eq!(
                code.message, msg,
                "Default message for {:?} is incorrect. Expected: {}",
                code, msg
            )
        }

        assert_error_default_message(ErrorCode::ParseError, "Parse error");
        assert_error_default_message(ErrorCode::InvalidRequest, "Invalid Request");
        assert_error_default_message(ErrorCode::MethodNotFound, "Method not found");
        assert_error_default_message(ErrorCode::InvalidParams, "Invalid params");
        assert_error_default_message(ErrorCode::InternalError, "Internal error");
        assert_error_default_message(ErrorCode::ServerError(0), "Server error");
    }
}
