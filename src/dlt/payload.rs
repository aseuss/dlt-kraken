use std::mem;
use paste::paste;
use std::str;

pub enum ByteConverter {
    FromBigEndian,
    FromLittleEndian,
}

macro_rules! impl_from_bytes {
    ($($type:ident)+) => ($(
        paste! {
            impl ByteConverter {
                fn [< $type _from_bytes >](&self, bytes: [u8; mem::size_of::<$type>()]) -> $type {
                    match self {
                        ByteConverter::FromBigEndian => $type::from_be_bytes(bytes).try_into().unwrap(),
                        ByteConverter::FromLittleEndian => $type::from_le_bytes(bytes).try_into().unwrap(),
                    }
                }
            }
        }
    )+)
}

impl_from_bytes! { u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 }

enum TypeLength {
    Bits8,
    Bits16,
    Bits32,
    Bits64,
    Bits128,
    Undefined,
}

impl From<u8> for TypeLength {
    fn from(value: u8) -> Self {
        match value {
            0x1 => TypeLength::Bits8,
            0x2 => TypeLength::Bits16,
            0x3 => TypeLength::Bits32,
            0x4 => TypeLength::Bits64,
            0x5 => TypeLength::Bits128,
            _ => TypeLength::Undefined,
        }
    }
}

enum TypeInfoStringEncoding {
    Ascii,
    Utf8,
    Reserved,
}

impl From<u32> for TypeInfoStringEncoding {
    fn from(value: u32) -> Self {
        match (value & TYPE_INFO_STRING_CODING_BIT_MASK) >> 15 {
            0x0 => TypeInfoStringEncoding::Ascii,
            0x1 => TypeInfoStringEncoding::Utf8,
            _ => TypeInfoStringEncoding::Reserved,
        }
    }
}

struct TypeInfo {
    length: TypeLength,
    var_info: bool,
    fixed_point: bool,
    string_coding: TypeInfoStringEncoding,
}

impl TypeInfo {
    fn new(length: TypeLength, var_info: bool, fixed_point: bool, string_coding: TypeInfoStringEncoding) -> TypeInfo {
        TypeInfo { length, var_info, fixed_point, string_coding}
    }
}

enum Type {
    Bool(TypeInfo),
    Signed(TypeInfo),
    Unsigned(TypeInfo),
    Float(TypeInfo),
    Array(TypeInfo),
    String(TypeInfo),
    Raw(TypeInfo),
    // VariableInfo,
    // FixedPoint,
    TraceInfo(TypeInfo),
    Struct(TypeInfo),
    // StringCoding,
    Reserved,
}

/*
      Bit 0 - 3 Type Length (TYLE)
      Bit 4 Type Bool (BOOL)
      Bit 5 Type Signed (SINT)
      Bit 6 Type Unsigned (UINT)
      Bit 7 Type Float (FLOA)
      Bit 8 Type Array (ARAY)
      Bit 9 Type String (STRG)
      Bit 10 Type Raw (RAWD)
      Bit 11 Variable Info (VARI)
      Bit 12 Fixed Point (FIXP)
      Bit 13 Trace Info (TRAI)
      Bit 14 Type Struct (STRU)
      Bit 15 – 17 String Coding (SCOD)
      Bit 18 – 31 reserved for future us
*/

const TYPE_INFO_LENGTH_BIT_MASK: u32 = 0x000F;
const TYPE_INFO_BOOL_BIT_MASK: u32 = 0x0010;
const TYPE_INFO_INT_BIT_MASK: u32 = 0x0020;
const TYPE_INFO_UINT_BIT_MASK: u32 = 0x0040;
const TYPE_INFO_FLOAT_BIT_MASK: u32 = 0x0080;
const TYPE_INFO_ARRAY_BIT_MASK: u32 = 0x0100;
const TYPE_INFO_STRING_BIT_MASK: u32 = 0x0200;
const TYPE_INFO_RAW_BIT_MASK: u32 = 0x0400;
const TYPE_INFO_VARIABLE_INFO_BIT_MASK: u32 = 0x0800;
const TYPE_INFO_FIXED_POINT_BIT_MASK: u32 = 0x1000;
const TYPE_INFO_TRACE_INFO_BIT_MASK: u32 = 0x2000;
const TYPE_INFO_STRUCT_BIT_MASK: u32 = 0x4000;
const TYPE_INFO_STRING_CODING_BIT_MASK: u32 = 0x38000;

impl From<u32> for Type {
    fn from(value: u32) -> Self {
        let has_var_info = value & TYPE_INFO_VARIABLE_INFO_BIT_MASK == TYPE_INFO_VARIABLE_INFO_BIT_MASK;
        let has_fixed_point = value & TYPE_INFO_FIXED_POINT_BIT_MASK == TYPE_INFO_FIXED_POINT_BIT_MASK;
        let type_len= TypeLength::from((value & TYPE_INFO_LENGTH_BIT_MASK) as u8);
        let string_coding =  TypeInfoStringEncoding::from(value);
        let type_info = TypeInfo::new(type_len, has_var_info, has_fixed_point, string_coding);

        match value {
            value if value & TYPE_INFO_BOOL_BIT_MASK == TYPE_INFO_BOOL_BIT_MASK => {
                Type::Bool(type_info)
            },
            value if value & TYPE_INFO_INT_BIT_MASK == TYPE_INFO_INT_BIT_MASK => {
                Type::Signed(type_info)
            },
            value if value & TYPE_INFO_UINT_BIT_MASK == TYPE_INFO_UINT_BIT_MASK => {
                Type::Unsigned(type_info)
            },
            value if value & TYPE_INFO_FLOAT_BIT_MASK == TYPE_INFO_FLOAT_BIT_MASK => {
                Type::Float(type_info)
            },
            value if value & TYPE_INFO_ARRAY_BIT_MASK == TYPE_INFO_ARRAY_BIT_MASK => {
                Type::Array(type_info)
            },
            value if value & TYPE_INFO_STRING_BIT_MASK == TYPE_INFO_STRING_BIT_MASK => {
                Type::String(type_info)
            },
            value if value & TYPE_INFO_RAW_BIT_MASK == TYPE_INFO_RAW_BIT_MASK => {
                Type::Raw(type_info)
            },
            value if value & TYPE_INFO_TRACE_INFO_BIT_MASK == TYPE_INFO_TRACE_INFO_BIT_MASK => {
                Type::TraceInfo(type_info)
            },
            value if value & TYPE_INFO_STRUCT_BIT_MASK == TYPE_INFO_STRUCT_BIT_MASK => {
                Type::Struct(type_info)
            },
            _ => Type::Reserved,
        }
    }
}

#[derive(Debug)]
pub enum Value<'a> {
    Bool(bool),
    SInt8(i8),
    SInt16(i16),
    SInt32(i32),
    SInt64(i64),
    SInt128(i128),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    UInt128(u128),
    Float32(f32),
    Float64(f64),
    String(&'a str),
    TraceData(&'a str),
}

pub struct Payload<'a> {
    data : &'a [u8],
    index: usize,
    count: usize,
    is_big_endian : bool,
}

impl Payload<'_> {

    pub fn new<'a>(data: &'a [u8], index: usize, is_big_endian: bool, count: usize) -> Payload<'a> {
        Payload{data, index, count, is_big_endian }
    }

    pub fn iter<'a>(&'a self) -> PayloadIter<'a> {
        PayloadIter {
            data : self.data,
            index : self.index,
            count : self.count,
            converter : if self.is_big_endian { ByteConverter::FromBigEndian } else { ByteConverter::FromLittleEndian }
        }
    }
}

pub struct PayloadIter<'a> {
    data: &'a [u8],
    index: usize,
    count: usize,
    converter: ByteConverter,
}

impl<'a> Iterator for PayloadIter<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;
            self.read_argument()
        } else {
            return None
        }
    }
}

impl<'a> IntoIterator for &'a Payload<'a> {
    type Item = Value<'a>;
    type IntoIter = PayloadIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> PayloadIter<'a> {

    fn read_argument(&mut self) -> Option<Value <'a>> {
        let read_to = self.index + mem::size_of::<u32>();
        let type_info = self.converter.u32_from_bytes(self.data[self.index .. read_to].try_into().unwrap());
        self.index = read_to;
        let arg_type = Type::from(type_info);

        match arg_type {
            Type::Bool(type_info) => self.read_bool(&type_info),
            Type::Unsigned(type_info) => self.read_unsigned(&type_info),
            Type::Signed(type_info) => self.read_signed(&type_info),
            Type::Float(type_info) => self.read_float(&type_info),
            Type::String(type_info) => self.read_string(&type_info),
            Type::Raw(type_info) => self.read_rawdata(&type_info),
            Type::TraceInfo(type_info) => self.read_trace_info(&type_info),
            Type::Array(type_info) => self.read_array(&type_info),
            Type::Struct(type_info) => self.read_struct(&type_info),
            Type::Reserved => None,
        }
    }

    fn read_bool(&mut self, type_info: &TypeInfo) -> Option<Value<'a>> {
        match type_info.length {
            TypeLength::Bits8 => {
                let read_to = self.index + mem::size_of::<u8>();
                let boolean = self.converter.u8_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::Bool(boolean == 0x1))
            },
            _ => None,
        }
    }

    fn read_signed(&mut self, type_info: &TypeInfo) -> Option<Value<'a>> {
        match type_info.length {
            TypeLength::Bits8 => {
                let read_to = self.index + mem::size_of::<i8>();
                let signed_int = self.converter.i8_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::SInt8(signed_int))
            },
            TypeLength::Bits16 => {
                let read_to = self.index + mem::size_of::<i16>();
                let signed_int = self.converter.i16_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::SInt16(signed_int))
            },
            TypeLength::Bits32 => {
                let read_to = self.index + mem::size_of::<i32>();
                let signed_int = self.converter.i32_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::SInt32(signed_int))
            },
            TypeLength::Bits64 => {
                let read_to = self.index + mem::size_of::<i64>();
                let signed_int = self.converter.i64_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::SInt64(signed_int))
            },
            TypeLength::Bits128 => {
                let read_to = self.index + mem::size_of::<i128>();
                let signed_int = self.converter.i128_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::SInt128(signed_int))
            },
            TypeLength::Undefined => None,
        }
    }

    fn read_unsigned(&mut self, type_info: &TypeInfo) -> Option<Value<'a>> {
        match type_info.length {
            TypeLength::Bits8 => {
                let read_to = self.index + mem::size_of::<u8>();
                let unsigned_int = self.converter.u8_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::UInt8(unsigned_int))
            },
            TypeLength::Bits16 => {
                let read_to = self.index + mem::size_of::<u16>();
                let unsigned_int = self.converter.u16_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::UInt16(unsigned_int))
            },
            TypeLength::Bits32 => {
                let read_to = self.index + mem::size_of::<u32>();
                let unsigned_int = self.converter.u32_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::UInt32(unsigned_int))
            },
            TypeLength::Bits64 => {
                let read_to = self.index + mem::size_of::<u64>();
                let unsigned_int = self.converter.u64_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::UInt64(unsigned_int))
            },
            TypeLength::Bits128 => {
                let read_to = self.index + mem::size_of::<u128>();
                let unsigned_int = self.converter.u128_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap());
                self.index = read_to;
                Some(Value::UInt128(unsigned_int))
            },
            TypeLength::Undefined => None,
        }
    }

    fn read_float(&self, type_info: &TypeInfo) -> Option<Value<'a>> {
        None
    }

    fn read_array(&self, _type_info: &TypeInfo) -> Option<Value<'a>> {
        None
    }

    fn read_string(&mut self, type_info: &TypeInfo) -> Option<Value<'a>> {
        let mut read_to = self.index + mem::size_of::<u16>();
        let str_len = self.converter.u16_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap()) as usize;
        self.index = read_to;

        read_to = read_to + str_len;
        let string: &'a str = str::from_utf8(&self.data[self.index .. read_to]).unwrap().trim_matches(char::from(0));
        self.index = read_to;

        Some(Value::String(string))
    }

    fn read_rawdata(&mut self, type_info: &TypeInfo) -> Option<Value<'a>> {
        None
    }

    fn read_trace_info(&mut self, type_info: &TypeInfo) -> Option<Value<'a>> {
        let mut read_to = self.index + mem::size_of::<u16>();
        let str_len = self.converter.u16_from_bytes(*&self.data[self.index .. read_to].try_into().unwrap()) as usize;
        self.index = read_to;

        read_to = read_to + str_len;
        let trace_data: &'a str = str::from_utf8(&self.data[self.index .. read_to]).unwrap().trim_matches(char::from(0));
        self.index = read_to;

        Some(Value::TraceData(trace_data))
    }

    fn read_struct(&mut self, _type_info: &TypeInfo) -> Option<Value<'a>> {
        None
    }
}
