#![deny(trivial_numeric_casts)]
#![allow(
    clippy::derive_partial_eq_without_eq,
    clippy::enum_variant_names,
    clippy::redundant_field_names,
    clippy::too_many_lines
)]

use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum AdjacentlyTagged<T> {
    Unit,
    Newtype(T),
    Tuple(u8, u8),
    Struct { f: u8 },
}

#[test]
fn unit() {
    // unit with no content
    assert_tokens(
        &AdjacentlyTagged::Unit::<u8>,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 1,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
            Token::StructEnd,
        ],
    );

    // unit with no content and incorrect hint for number of elements
    assert_de_tokens(
        &AdjacentlyTagged::Unit::<u8>,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
            Token::StructEnd,
        ],
    );

    // unit with tag first
    assert_de_tokens(
        &AdjacentlyTagged::Unit::<u8>,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
            Token::Str("c"),
            Token::Unit,
            Token::StructEnd,
        ],
    );

    // unit with content first
    assert_de_tokens(
        &AdjacentlyTagged::Unit::<u8>,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::Unit,
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
            Token::StructEnd,
        ],
    );

    // unit with excess content (f, g, h)
    assert_de_tokens(
        &AdjacentlyTagged::Unit::<u8>,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("f"),
            Token::Unit,
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
            Token::Str("g"),
            Token::Unit,
            Token::Str("c"),
            Token::Unit,
            Token::Str("h"),
            Token::Unit,
            Token::StructEnd,
        ],
    );
}

#[test]
fn newtype() {
    let value = AdjacentlyTagged::Newtype::<u8>(1);

    // newtype with tag first
    assert_tokens(
        &value,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Newtype",
            },
            Token::Str("c"),
            Token::U8(1),
            Token::StructEnd,
        ],
    );

    // newtype with content first
    assert_de_tokens(
        &value,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::U8(1),
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Newtype",
            },
            Token::StructEnd,
        ],
    );

    // optional newtype with no content field
    assert_de_tokens(
        &AdjacentlyTagged::Newtype::<Option<u8>>(None),
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 1,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Newtype",
            },
            Token::StructEnd,
        ],
    );

    // integer field keys
    assert_de_tokens(
        &value,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::U64(1), // content field
            Token::U8(1),
            Token::U64(0), // tag field
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Newtype",
            },
            Token::StructEnd,
        ],
    );

    // byte-array field keys
    assert_de_tokens(
        &value,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Bytes(b"c"),
            Token::U8(1),
            Token::Bytes(b"t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Newtype",
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn tuple() {
    let value = AdjacentlyTagged::Tuple::<u8>(1, 1);

    // tuple with tag first
    assert_tokens(
        &value,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Tuple",
            },
            Token::Str("c"),
            Token::Tuple { len: 2 },
            Token::U8(1),
            Token::U8(1),
            Token::TupleEnd,
            Token::StructEnd,
        ],
    );

    // tuple with content first
    assert_de_tokens(
        &value,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::Tuple { len: 2 },
            Token::U8(1),
            Token::U8(1),
            Token::TupleEnd,
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Tuple",
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn struct_() {
    let value = AdjacentlyTagged::Struct::<u8> { f: 1 };

    // struct with tag first
    assert_tokens(
        &value,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Struct",
            },
            Token::Str("c"),
            Token::Struct {
                name: "Struct",
                len: 1,
            },
            Token::Str("f"),
            Token::U8(1),
            Token::StructEnd,
            Token::StructEnd,
        ],
    );

    // struct with content first
    assert_de_tokens(
        &value,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::Struct {
                name: "Struct",
                len: 1,
            },
            Token::Str("f"),
            Token::U8(1),
            Token::StructEnd,
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Struct",
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn bytes() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(tag = "t", content = "c")]
    enum Data {
        A { a: i32 },
    }

    let data = Data::A { a: 0 };

    assert_tokens(
        &data,
        &[
            Token::Struct {
                name: "Data",
                len: 2,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "Data",
                variant: "A",
            },
            Token::Str("c"),
            Token::Struct { name: "A", len: 1 },
            Token::Str("a"),
            Token::I32(0),
            Token::StructEnd,
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &data,
        &[
            Token::Struct {
                name: "Data",
                len: 2,
            },
            Token::Bytes(b"t"),
            Token::UnitVariant {
                name: "Data",
                variant: "A",
            },
            Token::Bytes(b"c"),
            Token::Struct { name: "A", len: 1 },
            Token::Str("a"),
            Token::I32(0),
            Token::StructEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn struct_with_flatten() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(tag = "t", content = "c")]
    enum Data {
        A {
            a: i32,
            #[serde(flatten)]
            flat: Flat,
        },
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Flat {
        b: i32,
    }

    let data = Data::A {
        a: 0,
        flat: Flat { b: 0 },
    };

    assert_tokens(
        &data,
        &[
            Token::Struct {
                name: "Data",
                len: 2,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "Data",
                variant: "A",
            },
            Token::Str("c"),
            Token::Map { len: None },
            Token::Str("a"),
            Token::I32(0),
            Token::Str("b"),
            Token::I32(0),
            Token::MapEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn expecting_message() {
    #[derive(Deserialize)]
    #[serde(tag = "tag", content = "content")]
    #[serde(expecting = "something strange...")]
    enum Enum {
        AdjacentlyTagged,
    }

    assert_de_tokens_error::<Enum>(
        &[Token::Str("AdjacentlyTagged")],
        r#"invalid type: string "AdjacentlyTagged", expected something strange..."#,
    );

    assert_de_tokens_error::<Enum>(
        &[Token::Map { len: None }, Token::Unit],
        r#"invalid type: unit value, expected "tag", "content", or other ignored fields"#,
    );

    // Check that #[serde(expecting = "...")] doesn't affect variant identifier error message
    assert_de_tokens_error::<Enum>(
        &[Token::Map { len: None }, Token::Str("tag"), Token::Unit],
        "invalid type: unit value, expected variant of enum Enum",
    );
}

#[test]
fn partially_untagged() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    #[serde(tag = "t", content = "c")]
    enum Data {
        A(u32),
        B,
        #[serde(untagged)]
        Var(u32),
    }

    let data = Data::A(7);

    assert_de_tokens(
        &data,
        &[
            Token::Map { len: None },
            Token::Str("t"),
            Token::Str("A"),
            Token::Str("c"),
            Token::U32(7),
            Token::MapEnd,
        ],
    );

    let data = Data::Var(42);

    assert_de_tokens(&data, &[Token::U32(42)]);

    // TODO test error output
}

#[test]
fn newtype_with_newtype() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct NewtypeStruct(u32);

    assert_de_tokens(
        &AdjacentlyTagged::Newtype(NewtypeStruct(5)),
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::NewtypeStruct {
                name: "NewtypeStruct",
            },
            Token::U32(5),
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Newtype",
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn deny_unknown_fields() {
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(tag = "t", content = "c", deny_unknown_fields)]
    enum AdjacentlyTagged {
        Unit,
    }

    assert_de_tokens(
        &AdjacentlyTagged::Unit,
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
            Token::Str("c"),
            Token::Unit,
            Token::StructEnd,
        ],
    );

    assert_de_tokens_error::<AdjacentlyTagged>(
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("t"),
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
            Token::Str("c"),
            Token::Unit,
            Token::Str("h"),
        ],
        r#"invalid value: string "h", expected "t" or "c""#,
    );

    assert_de_tokens_error::<AdjacentlyTagged>(
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("h"),
        ],
        r#"invalid value: string "h", expected "t" or "c""#,
    );

    assert_de_tokens_error::<AdjacentlyTagged>(
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Str("c"),
            Token::Unit,
            Token::Str("h"),
        ],
        r#"invalid value: string "h", expected "t" or "c""#,
    );

    assert_de_tokens_error::<AdjacentlyTagged>(
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::U64(0), // tag field
            Token::UnitVariant {
                name: "AdjacentlyTagged",
                variant: "Unit",
            },
            Token::U64(3),
        ],
        r#"invalid value: integer `3`, expected "t" or "c""#,
    );

    assert_de_tokens_error::<AdjacentlyTagged>(
        &[
            Token::Struct {
                name: "AdjacentlyTagged",
                len: 2,
            },
            Token::Bytes(b"c"),
            Token::Unit,
            Token::Bytes(b"h"),
        ],
        r#"invalid value: byte array, expected "t" or "c""#,
    );
}