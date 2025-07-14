pub mod err;
pub mod msg;

mod de;
mod schema;
mod ser;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::msg::{Message, Response};

    #[test]
    fn main() {
        let json = json!([{
            "jsonrpc": "2.0",
            "id": "2",
            "method": "test1",
            "params": [1,2,3]
        }, {
            "jsonrpc": "2.0",
            "id": "3",
            "method": "test2",
        }]);

        let msg = serde_json::from_value::<Message>(json);

        println!("{:#?}", msg)
    }
}
