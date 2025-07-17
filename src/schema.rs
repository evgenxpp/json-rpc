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
    expected!(
        r#"{"anyOf": [{ "type": "string" }, {"type": "integer", "minimum": 0, "maximum": 18446744073709551615}]}"#
    );
}

pub mod parameters {
    expected!(
        r#"{"anyOf": [{ "type": "array" }, { "type": "object", "additionalProperties": true }]}"#
    );
}

pub mod error {
    name!("Error");
    expected!(
        r#"{"type": "object", "required": ["code", "message"], "properties": {"code": {"oneOf": [{ "enum": [-32700, -32600, -32601, -32602, -32603] }, {"type": "integer", "minimum": -32099, "maximum": -32000}]}, "message": {"type": "string"}, "data": {"type": ["object", "array", "string", "number", "boolean", "null"]}}, "additionalProperties": false}"#
    );

    fields!(
        CODE => "code",
        MESSAGE => "message",
        DATA => "data",
    );
}

pub mod notification {
    name!("Request");
    expected!(
        r#"{"jsonrpc":"2.0","id":"null|string|number:i64","method":"string","params":"array:any|object:any"}"#
    );

    fields!(
        JSONRPC => "jsonrpc",
        METHOD => "method",
        PARAMS => "params",
    );
}

pub mod request {
    name!("Request");
    expected!(
        r#"{"jsonrpc":"2.0","id":"null|string|number:i64","method":"string","params":"array:any|object:any"}"#
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

pub mod batch {
    expected!("array:[Request|Response]");
}

pub mod message {
    expected!("array:Request|array:Response|object:Request|object:Response");
}
