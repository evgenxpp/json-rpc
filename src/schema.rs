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

pub mod id {
    expected!("string|number:i64");
}

pub mod params {
    expected!("array:any|object:any");
}

pub mod error {
    name!("Error");
    expected!(r#"{"code":"number:i64","message":"string","data":"null|string"}"#);

    fields!(
        CODE => "code",
        MESSAGE => "message",
        DATA => "data",
    );
}

pub mod request {
    name!("Request");
    expected!(
        r#"{"id":"null|string|number:i64","method":"string","params":"null|array:any|object:any"}"#
    );

    fields!(
        ID => "id",
        METHOD => "method",
        PARAMS => "params",
    );
}

pub mod response {
    name!("Response");
    expected!(
        r#"{"id":"string|number:i64","result":"any"}|{"id":"null|string|number:i64","error":"object:Error"}"#
    );

    fields!(
        ID => "id",
        RESULT => "result",
        ERROR => "error",
    );
}
