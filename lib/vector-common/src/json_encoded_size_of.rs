use serde::{ser, Serialize};
use value::Value;

const NULL_SIZE: usize = 4;
const TRUE_SIZE: usize = 4;
const FALSE_SIZE: usize = 5;

const BRACKET_SIZE: usize = 1;
const BRACES_SIZE: usize = 2;
const BRACE_SIZE: usize = 1;

const QUOTES_SIZE: usize = 2;
const COMMA_SIZE: usize = 1;
const COLON_SIZE: usize = 1;

const EPOCH_RFC3339: &'static str = "1970-01-01T00:00:00.000Z";

pub struct JsonEncodedValue<'a>(pub &'a Value);

impl<'a> Serialize for JsonEncodedValue<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match &self.0 {
            // The `Value` type serializes `Value::Bytes` using `serialize_str`, but this requires
            // an extra allocation.
            //
            // Since we never serialize bytes as an array of integers in the JSON representation of
            // `Value`, we can use `serialize_bytes` instead to count the actual number of bytes in
            // the byte array, and add two extra bytes for the surrounding quotes. This avoids the
            // extra allocations, while still allowing `Value` itself to allocate and serialize
            // `Value::Bytes` to a string.
            Value::Bytes(b) => serializer.serialize_bytes(b),

            // We approximate the size of a timestamp by using milliseconds precision.
            //
            // This can be off, if a different timezone is used (but our `Value` type's serialie
            // implementation always uses UTC offset), or if the precision is more or less than
            // milliseconds precision (which can happen, because our `Value` type does automatic
            // inference of the required amount of precision, from nanoseconds to seconds).
            //
            // This is done to avoid having to allocate the timestamp to a string, to calculate the
            // exact byte size. A future improvement should calculate the required precision, and
            // addopt the proper timestamp length accordingly.
            Value::Timestamp(_) => serializer.serialize_str(EPOCH_RFC3339),

            // Collection types have their inner `Value`'s wrapped in `JsonEncodedValue`.
            Value::Object(m) => serializer.collect_map(m.iter().map(|(k, v)| (k, Self(v)))),
            Value::Array(a) => serializer.collect_seq(a.iter().map(|v| Self(v))),

            // All other `Value` variants are serialized according to the default serialization
            // implementation of that type.
            v => v.serialize(serializer),
        }
    }
}

pub trait JsonEncodedSizeOf {
    fn json_encoded_size_of(&self) -> usize;
}

impl<T> JsonEncodedSizeOf for T
where
    T: serde::Serialize,
{
    #[inline]
    fn json_encoded_size_of(&self) -> usize {
        size_of(self).unwrap()
    }
}

#[derive(Debug)]
pub struct Error;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error")
    }
}

impl std::error::Error for Error {}
impl ser::Error for Error {
    fn custom<T: std::fmt::Display>(_msg: T) -> Self {
        Self
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Serializer {
    bytes: usize,
    start_collection: bool,
}

/// Return the size of `T` as represented by a JSON-encoded string.
///
/// # Errors
///
/// Returns an error if `T` cannot be serialized.
pub fn size_of<T>(value: &T) -> Result<usize>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        bytes: 0,
        start_collection: false,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.bytes)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.bytes += if v { TRUE_SIZE } else { FALSE_SIZE };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        // -128 ..= 127
        if v < -99 {
            self.bytes += 4;
        } else if v < -9 {
            self.bytes += 3;
        } else if v < 0 {
            self.bytes += 2;
        } else if v < 10 {
            self.bytes += 1;
        } else if v < 100 {
            self.bytes += 2;
        } else {
            self.bytes += 3;
        }

        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        // -32_768 ..= 32_767
        if v < -9_999 {
            self.bytes += 6;
        } else if v < -999 {
            self.bytes += 5;
        } else if v < -99 {
            self.bytes += 4;
        } else if v < -9 {
            self.bytes += 3;
        } else if v < 0 {
            self.bytes += 2;
        } else if v < 10 {
            self.bytes += 1;
        } else if v < 100 {
            self.bytes += 2;
        } else if v < 1_000 {
            self.bytes += 3;
        } else if v < 10_000 {
            self.bytes += 4;
        } else {
            self.bytes += 5;
        }

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        // -2_147_483_648 ..= 2_147_483_647
        if v < -999_999_999 {
            self.bytes += 11;
        } else if v < -99_999_999 {
            self.bytes += 10;
        } else if v < -9_999_999 {
            self.bytes += 9;
        } else if v < -999_999 {
            self.bytes += 8;
        } else if v < -99_999 {
            self.bytes += 7;
        } else if v < -9_999 {
            self.bytes += 6;
        } else if v < -999 {
            self.bytes += 5;
        } else if v < -99 {
            self.bytes += 4;
        } else if v < -9 {
            self.bytes += 3;
        } else if v < 0 {
            self.bytes += 2;
        } else if v < 10 {
            self.bytes += 1;
        } else if v < 100 {
            self.bytes += 2;
        } else if v < 1_000 {
            self.bytes += 3;
        } else if v < 10_000 {
            self.bytes += 4;
        } else if v < 100_000 {
            self.bytes += 5;
        } else if v < 1_000_000 {
            self.bytes += 6;
        } else if v < 10_000_000 {
            self.bytes += 7;
        } else if v < 100_000_000 {
            self.bytes += 8;
        } else if v < 1_000_000_000 {
            self.bytes += 9;
        } else {
            self.bytes += 10;
        }

        Ok(())
    }

    #[rustfmt::skip]
    fn serialize_i64(self, v: i64) -> Result<()> {
        // -9_223_372_036_854_775_808 ..= 9_223_372_036_854_775_807
        if v <         -999_999_999_999_999_999 { self.bytes += 20;
        } else if v <   -99_999_999_999_999_999 { self.bytes += 19;
        } else if v <    -9_999_999_999_999_999 { self.bytes += 18;
        } else if v <      -999_999_999_999_999 { self.bytes += 17;
        } else if v <       -99_999_999_999_999 { self.bytes += 16;
        } else if v <        -9_999_999_999_999 { self.bytes += 15;
        } else if v <          -999_999_999_999 { self.bytes += 14;
        } else if v <           -99_999_999_999 { self.bytes += 13;
        } else if v <            -9_999_999_999 { self.bytes += 12;
        } else if v <              -999_999_999 { self.bytes += 11;
        } else if v <               -99_999_999 { self.bytes += 10;
        } else if v <                -9_999_999 { self.bytes += 9;
        } else if v <                  -999_999 { self.bytes += 8;
        } else if v <                   -99_999 { self.bytes += 7;
        } else if v <                    -9_999 { self.bytes += 6;
        } else if v <                      -999 { self.bytes += 5;
        } else if v <                       -99 { self.bytes += 4;
        } else if v <                        -9 { self.bytes += 3;
        } else if v <                         0 { self.bytes += 2;
        } else if v <                        10 { self.bytes += 1;
        } else if v <                       100 { self.bytes += 2;
        } else if v <                     1_000 { self.bytes += 3;
        } else if v <                    10_000 { self.bytes += 4;
        } else if v <                   100_000 { self.bytes += 5;
        } else if v <                 1_000_000 { self.bytes += 6;
        } else if v <                10_000_000 { self.bytes += 7;
        } else if v <               100_000_000 { self.bytes += 8;
        } else if v <             1_000_000_000 { self.bytes += 9;
        } else if v <            10_000_000_000 { self.bytes += 10;
        } else if v <           100_000_000_000 { self.bytes += 11;
        } else if v <         1_000_000_000_000 { self.bytes += 12;
        } else if v <        10_000_000_000_000 { self.bytes += 13;
        } else if v <       100_000_000_000_000 { self.bytes += 14;
        } else if v <     1_000_000_000_000_000 { self.bytes += 15;
        } else if v <    10_000_000_000_000_000 { self.bytes += 16;
        } else if v <   100_000_000_000_000_000 { self.bytes += 17;
        } else if v < 1_000_000_000_000_000_000 { self.bytes += 18;
        } else                                  { self.bytes += 20; }

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        // 0 ..= 255
        if v < 10 {
            self.bytes += 1;
        } else if v < 100 {
            self.bytes += 2;
        } else {
            self.bytes += 3;
        }

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        // 0 ..= 65_535
        if v < 10 {
            self.bytes += 1;
        } else if v < 100 {
            self.bytes += 2;
        } else if v < 1000 {
            self.bytes += 3;
        } else if v < 10000 {
            self.bytes += 4;
        } else {
            self.bytes += 5;
        }

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        // 0 ..= 4_294_967_295
        if v < 10 {
            self.bytes += 1;
        } else if v < 100 {
            self.bytes += 2;
        } else if v < 1_000 {
            self.bytes += 3;
        } else if v < 10_000 {
            self.bytes += 4;
        } else if v < 100_000 {
            self.bytes += 5;
        } else if v < 1_000_000 {
            self.bytes += 6;
        } else if v < 10_000_000 {
            self.bytes += 7;
        } else if v < 100_000_000 {
            self.bytes += 8;
        } else if v < 1_000_000_000 {
            self.bytes += 9;
        } else {
            self.bytes += 10;
        }

        Ok(())
    }

    #[rustfmt::skip]
    fn serialize_u64(self, v: u64) -> Result<()> {
        // 0 ..= 18_446_744_073_709_551_615
        if        v < 10 { self.bytes += 1;
        } else if v < 100 { self.bytes += 2;
        } else if v < 1_000 { self.bytes += 3;
        } else if v < 10_000 { self.bytes += 4;
        } else if v < 100_000 { self.bytes += 5;
        } else if v < 1_000_000 { self.bytes += 6;
        } else if v < 10_000_000 { self.bytes += 7;
        } else if v < 100_000_000 { self.bytes += 8;
        } else if v < 1_000_000_000 { self.bytes += 9;
        } else if v < 10_000_000_000 { self.bytes += 10;
        } else if v < 100_000_000_000 { self.bytes += 11;
        } else if v < 1_000_000_000_000 { self.bytes += 12;
        } else if v < 10_000_000_000_000 { self.bytes += 13;
        } else if v < 100_000_000_000_000 { self.bytes += 14;
        } else if v < 1_000_000_000_000_000 { self.bytes += 15;
        } else if v < 10_000_000_000_000_000 { self.bytes += 16;
        } else if v < 100_000_000_000_000_000 { self.bytes += 17;
        } else if v < 1_000_000_000_000_000_000 { self.bytes += 18;
        } else if v < 10_000_000_000_000_000_000 { self.bytes += 19;
        } else                                    { self.bytes += 20; }

        Ok(())
    }

    /// This method assumes the float isn't NaN or infinite, which holds true for our `Value` type,
    /// but might not hold true in other cases.
    ///
    /// If the float _is_ of one of those types, this method won't panic, but the reported byte size
    /// won't be accurate.
    fn serialize_f32(self, v: f32) -> Result<()> {
        let mut buffer = ryu::Buffer::new();
        self.bytes += buffer.format_finite(v).len();

        Ok(())
    }

    /// This method assumes the float isn't NaN or infinite, which holds true for our `Value` type,
    /// but might not hold true in other cases.
    ///
    /// If the float _is_ of one of those types, this method won't panic, but the reported byte size
    /// won't be accurate.
    fn serialize_f64(self, v: f64) -> Result<()> {
        let mut buffer = ryu::Buffer::new();
        self.bytes += buffer.format_finite(v).len();

        Ok(())
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<()> {
        self.bytes += QUOTES_SIZE + v.len_utf8();
        Ok(())
    }

    // TODO: handle escaping.
    fn serialize_str(self, v: &str) -> Result<()> {
        self.bytes += QUOTES_SIZE + v.len();
        Ok(())
    }

    // Consider `bytes` as being a valid `str`.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.bytes += QUOTES_SIZE + v.len();
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        self.bytes += NULL_SIZE;
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.bytes += BRACES_SIZE + COLON_SIZE;
        self.serialize_str(variant)?;
        value.serialize(self)?;

        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.start_collection = true;
        self.bytes += BRACKET_SIZE;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.bytes += BRACE_SIZE + COLON_SIZE + BRACKET_SIZE;
        self.serialize_str(variant)?;
        self.start_collection = true;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.start_collection = true;
        self.bytes += BRACE_SIZE;
        Ok(self)
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        // { "variant": { ...
        self.bytes += BRACE_SIZE + COLON_SIZE + BRACE_SIZE;
        self.serialize_str(variant)?;
        self.start_collection = true;
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.start_collection {
            self.bytes += COMMA_SIZE;
        }
        self.start_collection = false;

        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.bytes += BRACKET_SIZE;
        self.start_collection = false;
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.start_collection {
            self.bytes += COMMA_SIZE;
        }
        self.start_collection = false;

        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.bytes += BRACKET_SIZE;
        self.start_collection = false;
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.start_collection {
            self.bytes += COMMA_SIZE;
        }
        self.start_collection = false;

        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.bytes += BRACKET_SIZE;
        self.start_collection = false;
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.start_collection {
            self.bytes += COMMA_SIZE;
        }
        self.start_collection = false;

        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.bytes += BRACKET_SIZE + BRACE_SIZE;
        self.start_collection = false;
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    // TODO:
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.start_collection {
            self.bytes += COMMA_SIZE;
        }
        self.start_collection = false;

        key.serialize(&mut **self)
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.bytes += COLON_SIZE;
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.start_collection = false;
        self.bytes += BRACE_SIZE;
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.start_collection {
            self.bytes += COMMA_SIZE;
        }
        self.start_collection = false;

        key.serialize(&mut **self)?;
        self.bytes += COLON_SIZE;
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.start_collection = false;
        self.bytes += BRACE_SIZE;
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.start_collection {
            self.bytes += COMMA_SIZE;
        }
        self.start_collection = false;

        key.serialize(&mut **self)?;
        self.bytes += COLON_SIZE;
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.start_collection = false;
        self.bytes += BRACE_SIZE + BRACE_SIZE;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_struct() {
    #[derive(Serialize)]
    struct Test {
        int: u32,
        seq: Vec<&'static str>,
    }

    let test = Test {
        int: 1,
        seq: vec!["a", "b"],
    };
    let expected = r#"{"int":1,"seq":["a","b"]}"#;
    assert_eq!(size_of(&test).unwrap(), expected.len());
}

#[test]
fn test_enum() {
    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let u = E::Unit;
    let expected = r#""Unit""#;
    assert_eq!(size_of(&u).unwrap(), expected.len());

    let n = E::Newtype(1);
    let expected = r#"{"Newtype":1}"#;
    assert_eq!(size_of(&n).unwrap(), expected.len());

    let t = E::Tuple(1, 2);
    let expected = r#"{"Tuple":[1,2]}"#;
    assert_eq!(size_of(&t).unwrap(), expected.len());

    let s = E::Struct { a: 1 };
    let expected = r#"{"Struct":{"a":1}}"#;
    assert_eq!(size_of(&s).unwrap(), expected.len());
}
