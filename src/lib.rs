pub mod err;
pub mod msg;

mod de;
mod schema;
mod ser;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::msg::{Batch, Message, Request, Response};

    #[test]
    fn main() {
        let json = json!([ 1, "abc", true ]);

        let req = serde_json::from_value::<Batch>(json).unwrap();

        println!("{:#?}", req);

        let json = serde_json::to_string(&req).unwrap();
        println!("{}", json);
    }
}
