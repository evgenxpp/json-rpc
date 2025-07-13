use std::{any::type_name, fmt};

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as DeError, MapAccess, Visitor},
    ser::SerializeStruct,
};
use serde_json::Value;

use crate::{
    error::{Error, Result},
    id::Id,
    json,
};

use std::result::Result as StdResult;

#[derive(Debug)]
pub struct Response {
    id: Option<Id>,
    result: Result<Value>,
}

impl Response {
    const FIELD_ID: &str = "id";
    const FIELD_RESULT: &str = "result";
    const FIELD_ERROR: &str = "error";

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
}

impl Serialize for Response {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(type_name::<Self>(), 2)?;
        state.serialize_field(Self::FIELD_ID, &self.id)?;

        match self.result.as_ref() {
            Ok(value) => {
                state.serialize_field(Self::FIELD_RESULT, &value)?;
            }
            Err(error) => {
                state.serialize_field(Self::FIELD_ERROR, &error)?;
            }
        }

        state.end()
    }
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ResponseVisitor)
    }
}

struct ResponseVisitor;

impl<'de> Visitor<'de> for ResponseVisitor {
    type Value = Response;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(r#"`{{"id":"Id","result":"any"}}|{{"id":"null|Id","error":"Error"}}`"#)
    }

    fn visit_map<A>(self, mut map: A) -> StdResult<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut id = None;
        let mut result = None;
        let mut error = None;

        while let Some((key, value)) = map.next_entry::<String, Value>()? {
            match key.as_str() {
                Self::Value::FIELD_ID => {
                    id = match value {
                        Value::Null => None,
                        _ => {
                            let parsed_id = Id::deserialize(value).map_err(|err| {
                                json::make_field_error(Self::Value::FIELD_ID, err)
                            })?;

                            Some(parsed_id)
                        }
                    };
                }
                Self::Value::FIELD_RESULT => {
                    result = Some(value);
                }
                Self::Value::FIELD_ERROR => {
                    let parsed_error = Error::deserialize(value)
                        .map_err(|err| json::make_field_error(Self::Value::FIELD_ERROR, err))?;

                    error = Some(parsed_error);
                }
                unknown => {
                    return Err(DeError::unknown_field(
                        unknown,
                        &[
                            Self::Value::FIELD_ID,
                            Self::Value::FIELD_RESULT,
                            Self::Value::FIELD_ERROR,
                        ],
                    ));
                }
            }
        }

        match (result, error) {
            (Some(_), Some(_)) => Err(DeError::custom(
                "`result` and `error` cannot both be present in the same response",
            )),
            (Some(result), None) => {
                let id = id.ok_or_else(|| {
                    DeError::custom("`id` is required in a successful response with `result`")
                })?;
                Ok(Response::new_success(id, result))
            }
            (None, Some(error)) => Ok(Response::new_error(id, error)),
            (None, None) => Err(DeError::custom(
                "response must contain either `result` or `error`",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ErrorCode;

    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialization() {
        let assert_json = |response: Response| {
            let mut json = json!({
                "id": response.id(),
            });

            match response.result() {
                Ok(value) => {
                    json["result"] = value.clone();
                }
                Err(error) => {
                    json["error"] = serde_json::to_value(error).unwrap();
                }
            }

            assert_eq!(serde_json::to_value(&response).unwrap(), json);
        };

        assert_json(Response::new_success(i64::MAX.into(), json!(null)));

        assert_json(Response::new_success(
            i64::MAX.into(),
            json!({ "foo": "bar" }),
        ));

        assert_json(Response::new_success(
            String::from("uid").into(),
            json!({ "foo": "bar" }),
        ));

        assert_json(Response::new_error(
            None,
            Error::new_default(ErrorCode::MethodNotFound),
        ));

        assert_json(Response::new_error(
            Some(i64::MAX.into()),
            Error::new_default(ErrorCode::MethodNotFound),
        ));

        assert_json(Response::new_error(
            Some(String::from("uid").into()),
            Error::new_default(ErrorCode::MethodNotFound),
        ));
    }

    #[test]
    fn test_deserialization() {
        let json = json!([]);
        let response = serde_json::from_value::<Response>(json);
        assert!(response.unwrap_err().to_string().contains("invalid type"));

        let json = json!({});
        let response = serde_json::from_value::<Response>(json);
        assert!(
            response
                .unwrap_err()
                .to_string()
                .contains("response must contain either `result` or `error`")
        );

        let error = Error::new_default(ErrorCode::InternalError);
        let json = json!({
            "result": null,
            "error": serde_json::to_value(error).unwrap(),
        });
        let response = serde_json::from_value::<Response>(json);
        assert!(
            response
                .unwrap_err()
                .to_string()
                .contains("`result` and `error` cannot both be present in the same response")
        );

        let json = json!({ "result": null });
        let response = serde_json::from_value::<Response>(json);
        assert!(
            response
                .unwrap_err()
                .to_string()
                .contains("`id` is required in a successful response with `result`")
        );

        let json = json!({
            "id": null,
            "result": null,
        });
        let response = serde_json::from_value::<Response>(json);
        assert!(
            response
                .unwrap_err()
                .to_string()
                .contains("`id` is required in a successful response with `result`")
        );

        let json = json!({
            "id": null,
            "error": null,
        });
        let response = serde_json::from_value::<Response>(json);
        assert!(
            response
                .unwrap_err()
                .to_string()
                .contains("field `error` contains an invalid type: null")
        );
    }
}
