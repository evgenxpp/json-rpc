pub mod err;
pub mod msg;

mod de;
mod schema;
mod ser;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::msg::{Request, Response};

    #[test]
    fn main() {
        let json = json!({
            "jsonrpc": "2.0",
            "method": "test"
        });

        let req = serde_json::from_value::<Request>(json).unwrap();

        println!("{:#?}", req);

        let json = serde_json::to_string(&req).unwrap();
        println!("{}", json);
    }
}
