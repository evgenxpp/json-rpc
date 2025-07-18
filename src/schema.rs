macro_rules! fields {
    (
        $( $const_name:ident : $field_name:literal ),* $(,)?
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
    pub const DSL_SCHEMA: &str = "null|string|i64";
}

pub mod parameters {
    pub const DSL_SCHEMA: &str = "[]|{}";
}

pub mod notification {
    pub const DSL_SCHEMA: &str = "{jsonrpc: \"2.0\", method: string, params?: []|{}}";

    fields!(
        JSONRPC: "jsonrpc",
        METHOD: "method",
        PARAMS: "params",
    );
}

pub mod request {
    pub const DSL_SCHEMA: &str =
        "{jsonrpc: \"2.0\", id: null|string|i64, method: string, params?: []|{}}";

    fields!(
        JSONRPC: "jsonrpc",
        ID: "id",
        METHOD: "method",
        PARAMS: "params",
    );
}

pub mod error {
    pub const DSL_SCHEMA: &str = "{code: i64, message: string, data?: any}";

    fields!(
        CODE: "code",
        MESSAGE: "message",
        DATA: "data",
    );
}

pub mod response {
    pub const DSL_SCHEMA: &str = "{jsonrpc: \"2.0\", id: null|string|i64, result: any }|{jsonrpc: \"2.0\", id: null|string|i64, error: {code: i64, message: string, data?: any}}";

    fields!(
        JSONRPC: "jsonrpc",
        ID: "id",
        RESULT: "result",
        ERROR: "error",
    );
}
