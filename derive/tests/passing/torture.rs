use easy_config::EasyConfig;
use easy_config::schema::*;
use std::collections::{BTreeMap, HashMap};

#[derive(EasyConfig)]
struct UnitStruct;

#[derive(EasyConfig)]
struct NewTypeStruct(u8);

#[derive(EasyConfig)]
struct TupleStruct(u8, i32);

#[derive(EasyConfig)]
struct InnerStruct {
    x: f32,
    y: f32,
}

#[derive(EasyConfig)]
struct MiddleStruct {
    inner: InnerStruct,
    label: String,
}

#[derive(EasyConfig)]
struct DeepStruct {
    middle: MiddleStruct,
    flag: bool,
}

#[derive(EasyConfig)]
enum ComplexEnum {
    Unit,
    NewType(u8),
    Tuple(u8, i32),
    Struct { a: u8, b: String },
}

#[derive(EasyConfig)]
struct Torture {
    // primitives
    a_bool: bool,
    a_i8: i8,
    a_i16: i16,
    a_i32: i32,
    a_i64: i64,
    a_u8: u8,
    a_u16: u16,
    a_u32: u32,
    a_u64: u64,
    a_f32: f32,
    a_f64: f64,
    a_char: char,
    a_string: String,
    // collections
    a_vec: Vec<u8>,
    a_map: HashMap<String, i32>,
    a_btree: BTreeMap<String, i32>,
    // optionals
    a_option: Option<u8>,
    a_nested_option: Option<Option<u8>>,
    // structs
    a_unit_struct: UnitStruct,
    a_newtype_struct: NewTypeStruct,
    a_tuple_struct: TupleStruct,
    a_inner_struct: InnerStruct,
    // double nest
    a_middle_struct: MiddleStruct,
    // triple nest
    a_deep_struct: DeepStruct,
    // enum
    a_enum: ComplexEnum,
}

fn main() {
    assert_eq!(UnitStruct::SCHEMA, Schema::Struct(&StructSchema {
        name: "UnitStruct",
        fields: Fields::Unit,
    }));

    assert_eq!(NewTypeStruct::SCHEMA, Schema::Struct(&StructSchema {
        name: "NewTypeStruct",
        fields: Fields::NewType(&Schema::U8),
    }));

    assert_eq!(TupleStruct::SCHEMA, Schema::Struct(&StructSchema {
        name: "TupleStruct",
        fields: Fields::Tuple(&[Schema::U8, Schema::I32]),
    }));

    assert_eq!(InnerStruct::SCHEMA, Schema::Struct(&StructSchema {
        name: "InnerStruct",
        fields: Fields::Named(&[
            NamedField { name: "x", schema: Schema::F32 },
            NamedField { name: "y", schema: Schema::F32 },
        ]),
    }));

    assert_eq!(MiddleStruct::SCHEMA, Schema::Struct(&StructSchema {
        name: "MiddleStruct",
        fields: Fields::Named(&[
            NamedField { name: "inner", schema: InnerStruct::SCHEMA },
            NamedField { name: "label", schema: Schema::String },
        ]),
    }));

    assert_eq!(DeepStruct::SCHEMA, Schema::Struct(&StructSchema {
        name: "DeepStruct",
        fields: Fields::Named(&[
            NamedField { name: "middle", schema: MiddleStruct::SCHEMA },
            NamedField { name: "flag", schema: Schema::Bool },
        ]),
    }));

    assert_eq!(ComplexEnum::SCHEMA, Schema::Enum(&EnumSchema {
        name: "ComplexEnum",
        variants: &[
            NamedBagOfFields { name: "Unit", fields: Fields::Unit },
            NamedBagOfFields { name: "NewType", fields: Fields::NewType(&Schema::U8) },
            NamedBagOfFields { name: "Tuple", fields: Fields::Tuple(&[Schema::U8, Schema::I32]) },
            NamedBagOfFields { name: "Struct", fields: Fields::Named(&[
                NamedField { name: "a", schema: Schema::U8 },
                NamedField { name: "b", schema: Schema::String },
            ])},
        ],
    }));

    assert_eq!(Torture::SCHEMA, Schema::Struct(&StructSchema {
        name: "Torture",
        fields: Fields::Named(&[
            NamedField { name: "a_bool",           schema: Schema::Bool },
            NamedField { name: "a_i8",             schema: Schema::I8 },
            NamedField { name: "a_i16",            schema: Schema::I16 },
            NamedField { name: "a_i32",            schema: Schema::I32 },
            NamedField { name: "a_i64",            schema: Schema::I64 },
            NamedField { name: "a_u8",             schema: Schema::U8 },
            NamedField { name: "a_u16",            schema: Schema::U16 },
            NamedField { name: "a_u32",            schema: Schema::U32 },
            NamedField { name: "a_u64",            schema: Schema::U64 },
            NamedField { name: "a_f32",            schema: Schema::F32 },
            NamedField { name: "a_f64",            schema: Schema::F64 },
            NamedField { name: "a_char",           schema: Schema::Char },
            NamedField { name: "a_string",         schema: Schema::String },
            NamedField { name: "a_vec",            schema: Schema::Sequence(&Schema::U8) },
            NamedField { name: "a_map",            schema: Schema::Map(&Schema::String, &Schema::I32) },
            NamedField { name: "a_btree",          schema: Schema::Map(&Schema::String, &Schema::I32) },
            NamedField { name: "a_option",         schema: Schema::Option(&Schema::U8) },
            NamedField { name: "a_nested_option",  schema: Schema::Option(&Schema::Option(&Schema::U8)) },
            NamedField { name: "a_unit_struct",    schema: UnitStruct::SCHEMA },
            NamedField { name: "a_newtype_struct", schema: NewTypeStruct::SCHEMA },
            NamedField { name: "a_tuple_struct",   schema: TupleStruct::SCHEMA },
            NamedField { name: "a_inner_struct",   schema: InnerStruct::SCHEMA },
            NamedField { name: "a_middle_struct",  schema: MiddleStruct::SCHEMA },
            NamedField { name: "a_deep_struct",    schema: DeepStruct::SCHEMA },
            NamedField { name: "a_enum",           schema: ComplexEnum::SCHEMA },
        ]),
    }));
}