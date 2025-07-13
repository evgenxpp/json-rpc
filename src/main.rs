use crate::{error::Error, id::Id, response::Response};

mod error;
mod id;
mod json;
mod response;

fn main() {
    let resp = Response::new_success(i64::MAX.into(), "test".into());
    let json = serde_json::to_value(resp).unwrap();
    let resp = serde_json::from_value::<Response>(json).unwrap();
    println!("{:?}", resp);
}
