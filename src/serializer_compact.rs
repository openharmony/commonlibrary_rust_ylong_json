// Copyright (c) 2023 Huawei Device Co., Ltd.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::encoder::encode_string_inner;
use crate::{Error, Error::*};
use serde::{ser, ser::SerializeSeq, Serialize};

/// A data format that can serialize any data structure supported by Serde.
struct Serializer<W>
where
    W: std::io::Write,
{
    writer: W,
    element_num: Vec<usize>, // Used to record the number of traveled elements in the sequence.
}

/// An auxiliary struct which implements Write trait used in 'to_string' function.
struct AuxiliaryWriter {
    output: Vec<u8>,
}

impl std::io::Write for AuxiliaryWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// ylong_json::serializer_compact supports two functions to produce as output: 'to_string' and 'to_writer'.
///
/// The to_string function serialize an instance which implements the Serialize Trait to a string and return.
pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: Serialize,
{
    let mut writer = AuxiliaryWriter { output: Vec::new() };
    to_writer(value, &mut writer)?;
    Ok(unsafe { String::from_utf8_unchecked(writer.output) })
}

/// The to_writer function serialize an instance which implements the Serialize Trait and
/// writes result into the writer passed in by the user, which needs to implement the std::io::Write.
pub fn to_writer<T, W>(value: &T, writer: &mut W) -> Result<(), Error>
where
    T: Serialize,
    W: std::io::Write,
{
    let mut serializer = Serializer {
        writer,
        element_num: Vec::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(())
}

impl<'a, W: std::io::Write> ser::Serializer for &'a mut Serializer<W> {
    // Using `Ok` to propagate the data structure around simplifies Serializers
    // which build in-memory data structures. Set 'ok=()' and write the serialization
    // result to the buffer contained by the instance.
    type Ok = ();

    // The error type when serializing an instance may occur.
    type Error = Error;

    // Associative type composite data structures, such as sequences and maps,
    // used to track other states at serialization time. In this case, no state
    // is required other than what is already stored in the Serializer struct.
    type SerializeSeq = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    // The following 12 methods take one of the base types of the data model
    // and map it to JSON by appending it to the output string.
    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.writer.write_fmt(format_args!("{v}"))?;
        Ok(())
    }

    // JSON does not distinguish between integers of different sizes,
    // so all signed integers and all unsigned integers will be serialized in the same way.
    // Therefore, all integers are converted to 64-bit integers and serialized here.
    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.writer.write_fmt(format_args!("{v}"))?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        self.writer.write_fmt(format_args!("{v}"))?;
        Ok(())
    }

    // Same way for floating-point types.
    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        self.writer.write_fmt(format_args!("{v:?}"))?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.serialize_str(v.encode_utf8(&mut [0; 4]))
    }

    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.writer.write_all(b"\"")?;
        encode_string_inner(&mut self.writer, v)?;
        self.writer.write_all(b"\"")?;
        Ok(())
    }

    // Serialize a byte array into an array of bytes.
    // Binary formats will typically represent byte arrays more compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    // JSON `null` represent an absent optional.
    fn serialize_none(self) -> Result<(), Error> {
        self.serialize_unit()
    }

    // Represent an optional instance as the contained value.
    fn serialize_some<T>(self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit represents an anonymous value that does not contain any data.
    // Map this to JSON as "null".
    fn serialize_unit(self) -> Result<(), Error> {
        self.writer.write_all(b"null")?;
        Ok(())
    }

    // Unit struct represents a named value that does not contain any data. Map it to JSON as "null".
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to track it by index or by name. Binary formats
    // usually use index while human-readable formats usually use name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str, // The name of the variant.
    ) -> Result<(), Error> {
        self.serialize_str(variant)
    }

    // Treat newtype structs as insignificant wrappers for the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // The newtype variant (and all other variant serialization methods)
    // specifically refers to the enumerated representation of the "externally tagged".
    //
    // Serialize it to JSON with an outer tag of the form '{NAME: VALUE}'.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str, // The name of the variant.
        value: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.writer.write_all(b"{")?;
        variant.serialize(&mut *self)?;
        self.writer.write_all(b":")?;
        value.serialize(&mut *self)?;
        self.writer.write_all(b"}")?;
        Ok(())
    }

    // The following is the serialization of compound types.
    // The start of the sequence, each value, and the end use three separate method calls.
    // The serializer can be able to support sequences for which the length is unknown up front.
    //
    // This one used to serialize the start of the sequence, which in JSON is `[`.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        self.element_num.push(0); // Now there is no element in this sequence.
        self.writer.write_all(b"[")?;
        Ok(self)
    }

    // Tuples and sequences are represented similarly in JSON.
    // Some formats can represent tuples more efficiently by omitting the length.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(Some(len))
    }

    // Tuple and sequences are represented similarly in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize, // The number of data fields that will be serialized.
    ) -> Result<Self::SerializeTupleStruct, Error> {
        self.serialize_seq(Some(len))
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`.
    // This method used only for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str, // The name of the variant.
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        self.element_num.push(0);
        self.writer.write_all(b"{")?;
        variant.serialize(&mut *self)?;
        self.writer.write_all(b":[")?;
        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        self.element_num.push(0);
        self.writer.write_all(b"{")?;
        Ok(self)
    }

    // Structs and maps are represented similarly in JSON.
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize, // The number of data fields that will be serialized.
    ) -> Result<Self::SerializeStruct, Error> {
        self.serialize_map(Some(len))
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This used for the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str, // The name of the variant.
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        self.element_num.push(0);
        self.writer.write_all(b"{")?;
        variant.serialize(&mut *self)?;
        self.writer.write_all(b":{")?;
        Ok(self)
    }
}

impl<W> Serializer<W>
where
    W: std::io::Write,
{
    // Serialize a single member of sequence or map.
    fn whether_to_add_comma(&mut self) -> Result<(), Error> {
        match self.element_num.last_mut() {
            None => return Err(IncorrectSerdeUsage),
            Some(x) => {
                // This is not the first element, add a comma before the element.
                if *x > 0 {
                    self.writer.write_all(b",")?
                };
                *x += 1;
            }
        }
        Ok(())
    }

    // Add another half of the parentheses at the end.
    fn add_another_half(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.writer.write_all(buf)?;
        Ok(())
    }
}

// The following 7 impls used to serialize compound types like sequences and maps.
// At the beginning of serializing such types, one call of Serializer method.
// Next, zero or more calls which serializes a single element of the compound type.
// Finally, one call to end the serializing of the compound type.
//
// This impl is SerializeSeq so these methods are called after calling 'serialize_seq' on Serializer.
impl<'a, W: std::io::Write> ser::SerializeSeq for &'a mut Serializer<W> {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element in the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.whether_to_add_comma()?;
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<(), Error> {
        self.element_num.pop();
        self.add_another_half(b"]")
    }
}

// Same thing but for tuples.
impl<'a, W: std::io::Write> ser::SerializeTuple for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.whether_to_add_comma()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        self.element_num.pop();
        self.add_another_half(b"]")
    }
}

// Same thing but for tuple structs.
impl<'a, W: std::io::Write> ser::SerializeTupleStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.whether_to_add_comma()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        self.element_num.pop();
        self.add_another_half(b"]")
    }
}

// The tuple variants are slightly different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.writer.write_all(b"{");
//    variant.serialize(&mut *self)?;
//    self.writer.write_all(b":[");
//
// So the `end` method in this impl is responsible for closing
// both the `]` and the `}`.
impl<'a, W: std::io::Write> ser::SerializeTupleVariant for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.whether_to_add_comma()?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        self.element_num.pop();
        self.add_another_half(b"]}")
    }
}

// Some "Serialize" types cannot hold both keys and values
// in memory, so the "SerializeMap" implementation needs to
// support "serialize_key" and "serialize_value" respectively.
//
// There is a third optional method on the `SerializeMap` trait.
// The 'serialize_entry' method allows the serializer to be optimized
// for cases where both keys and values are available. In JSON, it doesn't
// make a difference so the default behavior for `serialize_entry` is fine.
impl<'a, W: std::io::Write> ser::SerializeMap for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys, so if the key is serialized to something other than a string,
    // the following implementation will produce invalid JSON.
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.whether_to_add_comma()?;
        key.serialize(&mut **self)
    }

    // It makes no difference whether the colon is printed at
    // the end of 'serialize_key' or at the beginning of 'serialize_value'.
    // Here we choose to print at the beginning of 'serialize_value'.
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.writer.write_all(b":")?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        self.add_another_half(b"}")
    }
}

// A structure is like a map where the keys are restricted to a compile-time constant string.
impl<'a, W: std::io::Write> ser::SerializeStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.whether_to_add_comma()?;
        key.serialize(&mut **self)?;
        self.writer.write_all(b":")?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        self.element_num.pop();
        self.add_another_half(b"}")
    }
}

// Similar to 'SerializeTupleVariant', the 'end' method here is responsible for
// closing two curly braces opened by 'serialize_struct_variant'.
impl<'a, W: std::io::Write> ser::SerializeStructVariant for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.whether_to_add_comma()?;
        key.serialize(&mut **self)?;
        self.writer.write_all(b":")?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        self.element_num.pop();
        self.add_another_half(b"}}")
    }
}

#[cfg(test)]
mod ut_serializer {
    use super::*;
    use std::collections::HashMap;

    /// UT test to serialize simple types.
    ///
    /// # Title
    /// ut_serialize_simple
    ///
    /// # Brief
    /// 1.Uses Serializer::to_string method to serialize simple types.
    /// 2.Checks if the test results are correct.
    #[test]
    fn ut_serialize_simple() {
        let value: Option<u32> = None;
        let expected = "null";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: Option<u32> = Some(123);
        let expected = "123";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value = true;
        let expected = "true";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: i8 = 123;
        let expected = "123";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: i16 = 123;
        let expected = "123";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: i32 = 123;
        let expected = "123";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: i64 = 123;
        let expected = "123";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: u8 = 123;
        let expected = "123";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: u16 = 123;
        let expected = "123";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: u32 = 123;
        let expected = "123";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: u64 = 123;
        let expected = "123";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: f32 = 1.0;
        let expected = "1.0";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value: f64 = 1.0;
        let expected = "1.0";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value = "abc";
        let expected = "\"abc\"";
        assert_eq!(to_string(&value).unwrap(), expected);

        let value = b"abc";
        let expected = "[97,98,99]";
        assert_eq!(to_string(&value).unwrap(), expected);
    }

    /// UT test to serialize struct
    ///
    /// # Title
    /// ut_serialize_struct
    ///
    /// # Brief
    /// 1.Uses Serializer::to_string method to serialize struct.
    /// 2.Checks if the test results are correct.
    #[test]
    fn ut_serialize_struct() {
        #[derive(Serialize)]
        struct TestUnit;
        let test = TestUnit;
        let expected = "null";
        assert_eq!(to_string(&test).unwrap(), expected);

        #[derive(Serialize)]
        struct TestNewtype(u32);
        let test = TestNewtype(123);
        let expected = "123";
        assert_eq!(to_string(&test).unwrap(), expected);

        #[derive(Serialize)]
        struct TestTuple(u32, u32, bool);
        let test = TestTuple(123, 321, true);
        let expected = "[123,321,true]";
        assert_eq!(to_string(&test).unwrap(), expected);

        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
            tup: (i32, i32, i32),
        }

        let test = Test {
            int: 1,
            seq: vec!["a", "b"],
            tup: (1, 2, 3),
        };
        let expected = r#"{"int":1,"seq":["a","b"],"tup":[1,2,3]}"#;
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    /// UT test to serialize enum
    ///
    /// # Title
    /// ut_serialize_enum
    ///
    /// # Brief
    /// 1.Uses Serializer::to_string method to serialize enum.
    /// 2.Checks if the test results are correct.
    #[test]
    fn ut_serialize_enum() {
        #[derive(Serialize)]
        enum E {
            Newtype(i32),
            Unit,
            Struct { a: u32 },
            Tuple(u32, u32),
        }

        let n = E::Newtype(-1);
        let expected = r#"{"Newtype":-1}"#;
        assert_eq!(to_string(&n).unwrap(), expected);

        let u = E::Unit;
        let expected = r#""Unit""#;
        assert_eq!(to_string(&u).unwrap(), expected);

        let s = E::Struct { a: 10 };
        let expected = r#"{"Struct":{"a":10}}"#;
        assert_eq!(to_string(&s).unwrap(), expected);

        let t = E::Tuple(100, 200);
        let expected = r#"{"Tuple":[100,200]}"#;
        assert_eq!(to_string(&t).unwrap(), expected);
    }

    /// UT test to serialize string
    ///
    /// # Title
    /// ut_serialize_string
    ///
    /// # Brief
    /// 1.Uses Serializer::to_string method to serialize string.
    /// 2.Checks if the test results are correct.
    #[test]
    fn ut_serialize_string() {
        let ch = 't';
        let expected = r#""t""#;
        assert_eq!(to_string(&ch).unwrap(), expected);

        let value = String::from("test string.");
        let expected = r#""test string.""#;
        assert_eq!(to_string(&value).unwrap(), expected);

        #[cfg(not(feature = "ascii_only"))]
        {
            let ch = '中';
            let expected = r#""\u4e2d""#;
            assert_eq!(to_string(&ch).unwrap(), expected);

            let value = String::from("中文测试字符串");
            let expected = r#""\u4e2d\u6587\u6d4b\u8bd5\u5b57\u7b26\u4e32""#;
            assert_eq!(to_string(&value).unwrap(), expected);
        }
    }

    /// UT test to serializer object
    ///
    /// # Title
    /// ut_serialize_object
    ///
    /// # Brief
    /// 1.Uses Serializer::to_string method to serialize object.
    /// 2.Checks if the test results are correct.
    #[test]
    fn ut_serialize_object() {
        let mut hash = HashMap::new();
        hash.insert("apple", 1);
        let expected = r#"{"apple":1}"#;
        assert_eq!(to_string(&hash).unwrap(), expected);
        hash.insert("banana", 2);
        assert!(to_string(&hash).is_ok());
    }
}
