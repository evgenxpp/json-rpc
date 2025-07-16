pub mod err;
pub mod msg;

mod de;
mod schema;
mod ser;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::msg::Response;

    #[test]
    fn main() {
        let json = json!({
            "jsonrpc": "2.0",
            "id": "2",
            "error": {
                "code": -32601,
                "message": "tmes",
                "data": {
                    "a": 1,
                    "b": true,
                    "c": "ssss",
                }
            }
        });

        let msg = serde_json::from_value::<Response>(json).unwrap();

        println!("{:#?}", msg);

        let data = msg.result().unwrap_err().data().unwrap().value();
        println!("{}", data);
    }
}
