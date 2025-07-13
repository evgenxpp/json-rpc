use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use serde_json::Value;
use std::fmt::Write;
use std::{
    any::type_name,
    borrow::Cow,
    fmt::{self, Display},
    result::Result as StdResult,
};

use crate::json;

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

impl Serialize for ErrorCode {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.as_i64())
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let code = i64::deserialize(deserializer)?;
        Self::try_from(code)
            .map_err(|err| DeError::custom(err.data().unwrap_or_else(|| err.message())))
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    code: ErrorCode,
    message: Cow<'static, str>,
    data: Option<Cow<'static, str>>,
}

impl Error {
    const FIELD_CODE: &str = "code";
    const FIELD_MESSAGE: &str = "message";
    const FIELD_DATA: &str = "data";

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

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(type_name::<Self>(), 3)?;
        state.serialize_field(Self::FIELD_CODE, &self.code)?;
        state.serialize_field(Self::FIELD_MESSAGE, &self.message)?;

        if let Some(data) = &self.data {
            state.serialize_field(Self::FIELD_DATA, data)?;
        }

        state.end()
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ErrorVisitor)
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

struct ErrorVisitor;

impl<'de> Visitor<'de> for ErrorVisitor {
    type Value = Error;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(r#"`{{"code":"ErrorCode","message":"string","data":"null|string"}}`"#)
    }

    fn visit_map<A>(self, mut map: A) -> StdResult<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut code = None;
        let mut message = None;
        let mut data = None;

        while let Some((key, value)) = map.next_entry::<String, Value>()? {
            match key.as_str() {
                Self::Value::FIELD_CODE => {
                    let parsed_code = ErrorCode::deserialize(value)
                        .map_err(|err| json::make_field_error(Self::Value::FIELD_CODE, err))?;
                    code = Some(parsed_code);
                }
                Self::Value::FIELD_MESSAGE => {
                    message =
                        json::deserialize_string(Self::Value::FIELD_MESSAGE, value).map(Some)?;
                }
                Self::Value::FIELD_DATA => {
                    data = json::deserialize_nullable_string(Self::Value::FIELD_DATA, value)?;
                }
                unknown => {
                    return Err(DeError::unknown_field(
                        unknown,
                        &[
                            Self::Value::FIELD_CODE,
                            Self::Value::FIELD_MESSAGE,
                            Self::Value::FIELD_DATA,
                        ],
                    ));
                }
            }
        }

        let error = Error::new(
            code.ok_or_else(|| DeError::missing_field(Self::Value::FIELD_CODE))?,
            message.ok_or_else(|| DeError::missing_field(Self::Value::FIELD_MESSAGE))?,
        );

        match data {
            Some(data) => Ok(error.with_data(data)),
            _ => Ok(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64;

    use serde_json::json;

    use super::*;

    const CODE_PARSE_ERROR: i64 = -32700;
    const CODE_INVALID_REQUEST: i64 = -32600;
    const CODE_METHOD_NOT_FOUND: i64 = -32601;
    const CODE_INVALID_PARAMS: i64 = -32602;
    const CODE_INTERNAL_ERROR: i64 = -32603;
    const CODE_SERVER_ERROR_MIN: i64 = -32099;
    const CODE_SERVER_ERROR_MAX: i64 = -32000;

    const MSG_PARSE_ERROR: &str = "Parse error";
    const MSG_INVALID_REQUEST: &str = "Invalid Request";
    const MSG_METHOD_NOT_FOUND: &str = "Method not found";
    const MSG_INVALID_PARAMS: &str = "Invalid params";
    const MSG_INTERNAL_ERROR: &str = "Internal error";
    const MSG_SERVER_ERROR: &str = "Server error";

    #[test]
    fn test_error_codes() {
        assert_eq!(ErrorCode::ParseError.as_i64(), CODE_PARSE_ERROR);
        assert_eq!(ErrorCode::InvalidRequest.as_i64(), CODE_INVALID_REQUEST);
        assert_eq!(ErrorCode::MethodNotFound.as_i64(), CODE_METHOD_NOT_FOUND);
        assert_eq!(ErrorCode::InvalidParams.as_i64(), CODE_INVALID_PARAMS);
        assert_eq!(ErrorCode::InternalError.as_i64(), CODE_INTERNAL_ERROR);
        assert_eq!(ErrorCode::ParseError.as_i64(), CODE_PARSE_ERROR);
        assert_eq!(ErrorCode::ParseError.as_i64(), CODE_PARSE_ERROR);
        assert_eq!(
            ErrorCode::ServerError(CODE_SERVER_ERROR_MIN).as_i64(),
            CODE_SERVER_ERROR_MIN
        );
    }

    #[test]
    fn test_error_code_create_predefined() {
        let invalid_code = ErrorCode::create(0);
        assert!(matches!(
            invalid_code.unwrap_err().code,
            ErrorCode::InvalidRequest
        ));

        let parse_error = ErrorCode::create(CODE_PARSE_ERROR);
        assert!(matches!(parse_error.unwrap(), ErrorCode::ParseError));

        let invalid_request = ErrorCode::create(CODE_INVALID_REQUEST);
        assert!(matches!(
            invalid_request.unwrap(),
            ErrorCode::InvalidRequest
        ));

        let method_not_found = ErrorCode::create(CODE_METHOD_NOT_FOUND);
        assert!(matches!(
            method_not_found.unwrap(),
            ErrorCode::MethodNotFound
        ));

        let invalid_params = ErrorCode::create(CODE_INVALID_PARAMS);
        assert!(matches!(invalid_params.unwrap(), ErrorCode::InvalidParams));

        let internal_error = ErrorCode::create(CODE_INTERNAL_ERROR);
        assert!(matches!(internal_error.unwrap(), ErrorCode::InternalError));
    }

    #[test]
    fn test_error_code_create_server_error_range() {
        let lower_bound = ErrorCode::create(CODE_SERVER_ERROR_MIN);
        assert!(
            matches!(lower_bound.unwrap(), ErrorCode::ServerError(code) if code == CODE_SERVER_ERROR_MIN)
        );

        let upper_bound = ErrorCode::create(CODE_SERVER_ERROR_MAX);
        assert!(
            matches!(upper_bound.unwrap(), ErrorCode::ServerError(code) if code == CODE_SERVER_ERROR_MAX)
        );

        let out_of_lower_bound = ErrorCode::create(CODE_SERVER_ERROR_MIN - 1);
        assert!(matches!(
            out_of_lower_bound.unwrap_err().code,
            ErrorCode::InvalidRequest
        ));

        let out_of_upper_bound = ErrorCode::create(CODE_SERVER_ERROR_MAX + 1);
        assert!(matches!(
            out_of_upper_bound.unwrap_err().code,
            ErrorCode::InvalidRequest
        ));
    }

    #[test]
    fn test_error_messages() {
        let parse_error = Error::new_default(ErrorCode::ParseError);
        assert_eq!(parse_error.message, MSG_PARSE_ERROR);

        let invalid_request = Error::new_default(ErrorCode::InvalidRequest);
        assert_eq!(invalid_request.message, MSG_INVALID_REQUEST);

        let method_not_found = Error::new_default(ErrorCode::MethodNotFound);
        assert_eq!(method_not_found.message, MSG_METHOD_NOT_FOUND);

        let invalid_params = Error::new_default(ErrorCode::InvalidParams);
        assert_eq!(invalid_params.message, MSG_INVALID_PARAMS);

        let internal_error = Error::new_default(ErrorCode::InternalError);
        assert_eq!(internal_error.message, MSG_INTERNAL_ERROR);

        let server_error = Error::new_default(ErrorCode::ServerError(CODE_SERVER_ERROR_MIN));
        assert_eq!(server_error.message, MSG_SERVER_ERROR);
    }

    #[test]
    fn test_error_serialization() {
        let reason = "invalid foo bar";
        let test_data_set = [
            (CODE_PARSE_ERROR, MSG_PARSE_ERROR),
            (CODE_INVALID_REQUEST, MSG_INVALID_REQUEST),
            (CODE_METHOD_NOT_FOUND, MSG_METHOD_NOT_FOUND),
            (CODE_INVALID_PARAMS, MSG_INVALID_PARAMS),
            (CODE_INTERNAL_ERROR, MSG_INTERNAL_ERROR),
            (CODE_SERVER_ERROR_MIN, MSG_SERVER_ERROR),
        ];

        let assertion = |error: &Error,
                         expected_code: i64,
                         expected_message: &str,
                         expected_reason: Option<&str>| {
            let actual_json = serde_json::to_value(error).unwrap();

            let mut expected_json = json!({
                "code": expected_code,
                "message": expected_message,
            });

            if let Some(reason) = expected_reason {
                expected_json["data"] = reason.into();
            }

            assert_eq!(actual_json, expected_json);
        };

        for (expected_code, expected_message) in test_data_set.iter() {
            let code = ErrorCode::create(*expected_code).unwrap();
            let error = Error::new_default(code);

            assertion(&error, *expected_code, expected_message, None);
            assertion(
                &error.with_data(reason),
                *expected_code,
                expected_message,
                Some(reason),
            );
        }
    }

    #[test]
    fn test_error_deserialization() {
        let reason = "invalid foo bar";
        let assert_ok = |json: Value, code: i64, message: &str, data: Option<&str>| {
            let result: StdResult<Error, _> = serde_json::from_value(json);
            let error = result.unwrap();

            assert_eq!(error.code.as_i64(), code);
            assert_eq!(error.message(), message);
            assert_eq!(error.data(), data);
        };
        let assert_error = |json: Value, msg: &str| {
            let result: StdResult<Error, _> = serde_json::from_value(json);

            assert!(result.unwrap_err().to_string().contains(msg))
        };

        assert_ok(
            json!({
                "code": CODE_INVALID_REQUEST,
                "message": MSG_INVALID_REQUEST,
            }),
            CODE_INVALID_REQUEST,
            MSG_INVALID_REQUEST,
            None,
        );

        assert_ok(
            json!({
                "code": CODE_INVALID_REQUEST,
                "message": MSG_INVALID_REQUEST,
                "data": null,
            }),
            CODE_INVALID_REQUEST,
            MSG_INVALID_REQUEST,
            None,
        );

        assert_ok(
            json!({
                "code": CODE_INVALID_REQUEST,
                "message": MSG_INVALID_REQUEST,
                "data": reason,
            }),
            CODE_INVALID_REQUEST,
            MSG_INVALID_REQUEST,
            Some(reason),
        );

        assert_error(json!([]), "invalid type");

        assert_error(json!({}), "missing field `code`");

        assert_error(
            json!({ "code": null }),
            "field `code` contains an invalid type: null",
        );

        assert_error(
            json!({ "code": true }),
            "field `code` contains an invalid type: boolean",
        );

        assert_error(
            json!({ "code": f64::consts::PI }),
            "field `code` contains an invalid type: floating point",
        );

        assert_error(
            json!({ "code": 0 }),
            "field `code` contains an invalid error code",
        );

        assert_error(
            json!({ "code": CODE_INVALID_REQUEST }),
            "missing field `message`",
        );

        assert_error(
            json!({
                "code": CODE_INVALID_REQUEST,
                "message": null,
            }),
            "field `message` contains an invalid type: null",
        );

        assert_error(
            json!({
               "code": CODE_INVALID_REQUEST,
               "message": true,
            }),
            "field `message` contains an invalid type: boolean",
        );

        assert_error(
            json!({
                "code": CODE_INVALID_REQUEST,
                "message": MSG_INVALID_REQUEST,
                "data": true,
            }),
            "field `data` contains an invalid type: boolean",
        );

        assert_error(
            json!({ "unknown": null }),
            "unknown field `unknown`, expected one of `code`, `message`, `data`",
        );
    }
}
