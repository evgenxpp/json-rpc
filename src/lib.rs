pub mod err;
pub mod msg;

mod de;
mod schema;
mod ser;

#[cfg(test)]
mod tests {
    use serde::de::{MapAccess, value::MapAccessDeserializer};
    use serde_json::{Map, json};

    use crate::{
        err::{Error, ErrorCode},
        msg::{Id, Message, Notification, Request, Response},
    };

    #[test]
    fn main() {
        let mut msgs: Vec<Message> = Vec::new();

        msgs.push(Request::new(Id::Null, "do0", None).into());
        msgs.push(Request::new(1, "do1", None).into());
        msgs.push(
            Request::new(
                "bc0caa41-22f3-4075-873e-240670c1bf17",
                "do2",
                Some(vec![1.into(), "test".into(), true.into()].into()),
            )
            .into(),
        );
        msgs.push(Notification::new("notify1", None).into());
        msgs.push(
            Notification::new(
                "notify2",
                Some(vec![2.into(), "test2".into(), false.into()].into()),
            )
            .into(),
        );

        msgs.push(
            Response::new_error(Id::Null, Error::new_default(ErrorCode::InternalError)).into(),
        );

        msgs.push(
            Response::new_error(
                1,
                Error::new_default(ErrorCode::InternalError).with_data("test"),
            )
            .into(),
        );

        msgs.push(Response::new_success(Id::Null, "test").into());
        msgs.push(Response::new_success(15, vec![1, 2, 3]).into());

        for msg in msgs.iter() {
            let json = serde_json::to_string(msg).unwrap();
            println!("{}", json);

            let msg = serde_json::from_str::<Message>(&json).unwrap();
            println!("{:?}", msg)
        }
    }
}
