macro_rules! name {
    ( $name:literal ) => {
        pub const NAME: &str = $name;
    };
}

macro_rules! expected {
    ( $expected_schema:literal ) => {
        pub const EXPECTED_SCHEMA: &str = $expected_schema;
    };
}

macro_rules! fields {
    (
        $( $const_name:ident => $field_name:literal ),* $(,)?
    ) => {
        pub mod fields {
            $(
                pub const $const_name: &str = $field_name;
            )*
        }

        pub const FIELD_NAMES: &[&str] = &[
            $(
                fields::$const_name,
            )*
        ];
    };
}

pub const VERSION: &str = "2.0";

pub mod id {
    expected!("string|number:i64");
}

pub mod params {
    expected!("array:any|object:any");
}

pub mod error {
    name!("Error");
    expected!(r#"{"code":"number:i64","message":"string","data":"any"}"#);

    fields!(
        CODE => "code",
        MESSAGE => "message",
        DATA => "data",
    );
}

pub mod request {
    name!("Request");
    expected!(
        r#"{"jsonrpc":"2.0","id":"null|string|number:i64","method":"string","params":"null|array:any|object:any"}"#
    );

    fields!(
        JSONRPC => "jsonrpc",
        ID => "id",
        METHOD => "method",
        PARAMS => "params",
    );
}

pub mod response {
    name!("Response");
    expected!(
        r#"{"jsonrpc":"2.0","id":"null|string|number:i64","result":"any"}|{"jsonrpc":"2.0","id":"null|string|number:i64","error":"object:Error"}"#
    );

    fields!(
        JSONRPC => "jsonrpc",
        ID => "id",
        RESULT => "result",
        ERROR => "error",
    );
}

pub mod batch_request {
    expected!("array:Request");
}

pub mod batch_response {
    expected!("array:Response");
}

pub mod message {
    expected!("array:Request|array:Response|object:Request|object:Response");
}
