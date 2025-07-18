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

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value, json};

    use super::*;

    #[test]
    fn test_serialize_id() {
        let id = Id::Null;
        let json = serde_json::to_value(&id);

        assert!(json.is_ok());
        assert_eq!(json.unwrap(), Value::Null);

        let raw = u64::MIN;
        let id = Id::U64(raw);
        let json = serde_json::to_value(&id);

        assert!(json.is_ok());
        assert_eq!(json.unwrap(), Value::from(raw));

        let raw = u64::MAX;
        let id = Id::U64(raw);
        let json = serde_json::to_value(&id);

        assert!(json.is_ok());
        assert_eq!(json.unwrap(), Value::from(raw));

        let raw = "".to_owned();
        let id = Id::Str(raw.clone());
        let json = serde_json::to_value(&id);

        assert!(json.is_ok());
        assert_eq!(json.unwrap(), Value::from(raw));

        let raw = "123".to_owned();
        let id = Id::Str(raw.clone());
        let json = serde_json::to_value(&id);

        assert!(json.is_ok());
        assert_eq!(json.unwrap(), Value::from(raw));
    }

    #[test]
    fn test_serialize_parameters() {
        let foo = [1, 2, 3];
        let bar = {
            let mut map = Map::new();
            map.insert("val1".into(), 1.into());
            map.insert("val2".into(), false.into());
            map.insert("val3".into(), "smth".into());
            map
        };

        let raw: Vec<Value> = vec![
            Value::Null,
            true.into(),
            1.into(),
            "str".into(),
            foo.into(),
            bar.clone().into(),
        ];
        let params = Parameters::Array(raw.clone());
        let json = serde_json::to_value(&params);

        assert!(json.is_ok());
        assert_eq!(json.unwrap(), Value::from(raw));

        let params = Parameters::Object(bar.clone());
        let json = serde_json::to_value(&params);

        assert!(json.is_ok());
        assert_eq!(json.unwrap(), Value::from(bar.clone()));
    }

    #[test]
    fn test_serialize_notification() {
        let method = "".to_owned();
        let params = None;
        let notification = Notification::new(method.clone(), params);
        let json = serde_json::to_value(notification);

        assert!(json.is_ok());
        assert_eq!(
            json.unwrap(),
            json!({
                "jsonrpc": "2.0",
                "method": method,
            })
        );

        let method = "do".to_owned();
        let params = Parameters::Array(vec![1.into(), true.into()]);
        let notification = Notification::new(method.clone(), Some(params.clone()));
        let json = serde_json::to_value(notification);

        assert!(json.is_ok());
        assert_eq!(
            json.unwrap(),
            json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
            })
        );
    }

    #[test]
    fn test_serialize_request() {
        let id = Id::Null;
        let method = "".to_owned();
        let params = None;
        let request = Request::new(id.clone(), method.clone(), params);
        let json = serde_json::to_value(request);

        assert!(json.is_ok());
        assert_eq!(
            json.unwrap(),
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
            })
        );

        let id = Id::U64(u64::MIN);
        let method = "do".to_owned();
        let params = Parameters::Array(vec![1.into(), true.into()]);
        let request = Request::new(id.clone(), method.clone(), Some(params.clone()));
        let json = serde_json::to_value(request);

        assert!(json.is_ok());
        assert_eq!(
            json.unwrap(),
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
                "params": serde_json::to_value(params).unwrap(),
            })
        );
    }

    #[test]
    fn test_serialize_error_code() {
        fn assert_error_code_with(code: ErrorCode, num: i64) {
            let json = serde_json::to_value(code);

            assert!(json.is_ok());
            assert_eq!(json.unwrap(), Value::from(num));
        }

        assert_error_code_with(ErrorCode::ParseError, -32700);
        assert_error_code_with(ErrorCode::InvalidRequest, -32600);
        assert_error_code_with(ErrorCode::MethodNotFound, -32601);
        assert_error_code_with(ErrorCode::InvalidParams, -32602);
        assert_error_code_with(ErrorCode::InternalError, -32603);
        assert_error_code_with(ErrorCode::ServerError(-32099), -32099);
    }

    #[test]
    fn test_serialize_error_data() {
        fn assert_data_with<T: Into<Value>>(value: T) {
            let value: Value = value.into();
            let data = ErrorData::new(value.clone());
            let json = serde_json::to_value(data);

            assert!(json.is_ok());
            assert_eq!(json.unwrap(), value);
        }

        assert_data_with(Value::Null);
        assert_data_with(true);
        assert_data_with(123);
        assert_data_with("smth");
        assert_data_with(vec![1, 2, 3]);
        assert_data_with({
            let mut map = Map::new();
            map.insert("val1".into(), 1.into());
            map.insert("val2".into(), false.into());
            map.insert("val3".into(), "smth".into());
            map
        });
    }

    #[test]
    fn test_serialize_error() {
        let code = ErrorCode::InternalError;
        let message = "";
        let error = Error::new(ErrorCode::InternalError, message.to_owned());
        let json = serde_json::to_value(error);

        assert!(json.is_ok());
        assert_eq!(
            json.unwrap(),
            json!({
                "code": code,
                "message": message,
            })
        );

        let code = ErrorCode::InternalError;
        let message = String::from("oops");
        let data = vec![1, 2, 3];
        let error = Error::new(ErrorCode::InternalError, message.clone()).with_data(data.clone());
        let json = serde_json::to_value(error);

        assert!(json.is_ok());
        assert_eq!(
            json.unwrap(),
            json!({
                "code": code,
                "message": message,
                "data": data,
            })
        );
    }

    #[test]
    fn test_serialize_response() {
        fn assert_success_response_with<I: Into<Id>, R: Into<Value>>(id: I, result: R) {
            let id: Id = id.into();
            let result: Value = result.into();
            let response = Response::new_success(id.clone(), result.clone());
            let json = serde_json::to_value(response);

            assert!(json.is_ok());
            assert_eq!(
                json.unwrap(),
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result,
                })
            );
        }

        fn assert_error_response_with<I: Into<Id>>(id: I, error: Error) {
            let id: Id = id.into();
            let response = Response::new_error(id.clone(), error.clone());
            let json = serde_json::to_value(response);

            assert!(json.is_ok());
            assert_eq!(
                json.unwrap(),
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": error,
                })
            );
        }

        assert_success_response_with(Id::Null, "smth");
        assert_success_response_with("id", u64::MAX);
        assert_success_response_with(u64::MIN, true);
        assert_success_response_with(u64::MAX, vec![1, 2, 3]);
        assert_success_response_with(
            u64::MAX,
            json!({
                "foo": 1,
                "bar": true,
            }),
        );

        assert_error_response_with(Id::Null, Error::new_default(ErrorCode::ServerError(-32099)));
        assert_error_response_with(u64::MAX, Error::new_default(ErrorCode::InternalError));
        assert_error_response_with(
            "id",
            Error::new_default(ErrorCode::InvalidParams).with_data("smth"),
        );
    }

    #[test]
    fn test_serialize_message() {
        fn assert_message_with<M>(message: M)
        where
            M: Into<Message> + Serialize + Clone,
        {
            let expected = serde_json::to_value(message.clone()).unwrap();
            let message = message.into();
            let json = serde_json::to_value(message);

            assert!(json.is_ok());
            assert_eq!(json.unwrap(), expected);
        }

        let arr_params: Parameters = vec![1.into(), 2.into(), 3.into()].into();
        let arr_params_value = serde_json::to_value(arr_params.clone()).unwrap();
        let obj_params: Parameters = {
            let mut map = Map::new();
            map.insert("val1".to_owned(), 1.into());
            map.insert("val2".to_owned(), true.into());
            map.into()
        };
        let obj_params_value = serde_json::to_value(obj_params.clone()).unwrap();

        assert_message_with(Notification::new("", None));
        assert_message_with(Notification::new("do1", Some(arr_params.clone())));
        assert_message_with(Notification::new("do2", Some(obj_params.clone())));
        assert_message_with(Request::new(Id::Null, "", None));
        assert_message_with(Request::new("", "do1", Some(arr_params.clone())));
        assert_message_with(Request::new(u64::MAX, "do2", Some(obj_params.clone())));
        assert_message_with(Response::new_success(Id::Null, "smth"));
        assert_message_with(Response::new_success("", arr_params_value.clone()));
        assert_message_with(Response::new_success(u64::MAX, obj_params_value.clone()));
        assert_message_with(Response::new_error(
            Id::Null,
            Error::new_default(ErrorCode::InternalError),
        ));
        assert_message_with(Response::new_error(
            "",
            Error::new_default(ErrorCode::InvalidParams).with_data(arr_params_value.clone()),
        ));
        assert_message_with(Response::new_error(
            u64::MAX,
            Error::new_default(ErrorCode::InvalidParams).with_data(obj_params_value.clone()),
        ));
    }
}
