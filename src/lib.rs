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
            "id": "123",
            "result": null,
            "error": {
                "code": -32000,
                "message": "aaaaa",
                "data": "bbbbb"
            },
        });

        let msg = serde_json::from_value::<Response>(json);

        println!("{:#?}", msg)
    }
}
