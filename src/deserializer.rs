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

use crate::reader::{BytesReader, Cacheable, IoReader, SliceReader};
use serde::de;
use serde::de::{
    DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use serde::Deserialize;
use std::io::Read;

use crate::{consts::*, error::*, states::*, Number, ParseError::*};

#[cfg(feature = "c_adapter")]
type JsonString = CString;
#[cfg(not(feature = "c_adapter"))]
type JsonString = String;

impl Number {
    fn visit<'de, V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Number::Unsigned(x) => visitor.visit_u64(x),
            Number::Signed(x) => visitor.visit_i64(x),
            Number::Float(x) => visitor.visit_f64(x),
        }
    }
}

/// A struct that can deserialize JSON into Rust values of user's types.
pub(crate) struct Deserializer<R>
where
    R: BytesReader + Cacheable,
{
    pub(crate) reader: R,
    pub(crate) recursion_depth: u32,
}

impl<R> Deserializer<R>
where
    R: BytesReader + Cacheable,
{
    /// Creates a new instance of Deserializer.
    /// This method is usually used in the following methods:
    /// - Deserializer::new_from_reader
    /// - Deserializer::new_from_slice
    pub fn new(reader: R) -> Self {
        Deserializer {
            reader,
            recursion_depth: 0,
        }
    }
}

/// Creates an instance of Deserializer from reader.
impl<R: Read> Deserializer<IoReader<R>> {
    pub fn new_from_io(reader: R) -> Self {
        Deserializer::new(IoReader::new(reader))
    }
}

/// Creates an instance of Deserializer from slice.
impl<'a> Deserializer<SliceReader<'a>> {
    pub fn new_from_slice(slice: &'a [u8]) -> Self {
        Deserializer::new(SliceReader::new(slice))
    }
}

/// Deserializes an instance of type `T` from an IO stream of JSON.
/// # Example
/// ```not run
/// use serde::Deserialize;
/// use std::fs::File;
/// use ylong_json::from_reader;
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct Test {
///     int: u32,
///     seq: Vec<String>,
///     tup: (i32, i32, i32),
/// }
///
/// let expected = Test {
///     int: 1,
///     seq: vec![String::from("abcd"), String::from("efgh")],
///     tup: (1, 2, 3),
/// };
/// let file = File::open("./test.txt").unwrap();
/// assert_eq!(expected, from_reader(file).unwrap());
/// ```
pub fn from_reader<R, T>(reader: R) -> Result<T, Error>
where
    R: Read,
    T: DeserializeOwned,
{
    let mut deserializer = Deserializer::new_from_io(reader);
    let t = T::deserialize(&mut deserializer)?;
    match eat_whitespace_until_not!(deserializer) {
        None => Ok(t),
        _ => Err(Error::Parsing(ParsingUnfinished)),
    }
}

/// Deserializes an instance of type `T` from bytes.
/// # Example
/// ```
/// use serde::Deserialize;
/// use ylong_json::from_slice;
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct Test {
///     int: u32,
///     seq: Vec<String>,
///     tup: (i32, i32, i32),
/// }
///
/// let slice = r#"{"int":1,"seq":["abcd","efgh"],"tup":[1,2,3]}"#.as_bytes();
/// let expected = Test {
///     int: 1,
///     seq: vec![String::from("abcd"), String::from("efgh")],
///     tup: (1, 2, 3),
/// };
/// assert_eq!(expected, from_slice(slice).unwrap())
/// ```
pub fn from_slice<'a, T>(slice: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::new_from_slice(slice);
    let t = T::deserialize(&mut deserializer)?;
    match eat_whitespace_until_not!(deserializer) {
        None => Ok(t),
        _ => {
            unexpected_character!(&mut deserializer)
        }
    }
}

/// Deserializes an instance of type `T` from str.
/// # Example
/// ```
/// use serde::Deserialize;
/// use ylong_json::from_str;
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct Test {
///     int: u32,
///     seq: Vec<String>,
///     tup: (i32, i32, i32),
/// }
///
/// let str = r#"{"int":1,"seq":["abcd","efgh"],"tup":[1,2,3]}"#;
/// let expected = Test {
///     int: 1,
///     seq: vec![String::from("abcd"), String::from("efgh")],
///     tup: (1, 2, 3),
/// };
/// assert_eq!(expected, from_str(str).unwrap())
/// ```
pub fn from_str<'a, T>(str: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    from_slice(str.as_bytes())
}

impl<R> Deserializer<R>
where
    R: BytesReader + Cacheable,
{
    // Look at the next character without moving cursor.
    fn peek_char(&mut self) -> Result<Option<u8>, Error> {
        self.reader.peek().map_err(Error::new_reader)
    }

    // Get the next character and move the cursor to the next place.
    fn next_char(&mut self) -> Result<Option<u8>, Error> {
        self.reader.next().map_err(Error::new_reader)
    }

    // Discard the next character and move the cursor to the next place.
    fn discard_char(&mut self) {
        self.reader.discard();
    }

    // Parse value of bool, `true` or `false`.
    fn parse_bool(&mut self) -> Result<bool, Error> {
        let peek_ch = match eat_whitespace_until_not!(self) {
            Some(ch) => ch,
            None => {
                return Err(Error::Parsing(ParsingUnfinished));
            }
        };

        match peek_ch {
            b't' => {
                self.reader.discard();
                match_str!(self, b"rue");
                Ok(true)
            }
            b'f' => {
                self.reader.discard();
                match_str!(self, b"alse");
                Ok(false)
            }
            _ => {
                unexpected_character!(self)
            }
        }
    }

    fn de_parse_number<'de, V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let peek_ch = match eat_whitespace_until_not!(self) {
            Some(ch) => ch,
            None => {
                return Err(Error::Parsing(ParsingUnfinished));
            }
        };

        match peek_ch {
            b'-' => parse_number(self)?.visit(visitor),
            b'0'..=b'9' => parse_number(self)?.visit(visitor),
            _ => unexpected_character!(self),
        }
    }

    fn de_parse_string(&mut self) -> Result<JsonString, Error> {
        match self.peek_char()? {
            Some(b'"') => self.discard_char(),
            _ => return unexpected_character!(self),
        }
        parse_string(self)
    }
}

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<R>
where
    R: BytesReader + Cacheable,
{
    type Error = Error;

    // Choose a parsing method to parse value based on the input data.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let peek_ch = match eat_whitespace_until_not!(self) {
            Some(ch) => ch,
            None => {
                return Err(Error::Parsing(ParsingUnfinished));
            }
        };

        match peek_ch {
            b'n' => self.deserialize_unit(visitor),
            b't' | b'f' => self.deserialize_bool(visitor),
            b'"' => self.deserialize_str(visitor),
            b'0'..=b'9' => self.deserialize_u64(visitor),
            b'-' => self.deserialize_i64(visitor),
            b'[' => self.deserialize_seq(visitor),
            b'{' => self.deserialize_map(visitor),
            _ => unexpected_character!(self),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de_parse_number(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de_parse_number(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de_parse_number(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de_parse_number(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de_parse_number(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de_parse_number(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de_parse_number(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de_parse_number(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de_parse_number(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de_parse_number(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let str = self.de_parse_string()?;
        #[cfg(feature = "c_adapter")]
        return visitor.visit_str(str.to_str()?);

        #[cfg(not(feature = "c_adapter"))]
        visitor.visit_str(str.as_str())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let peek_ch = match eat_whitespace_until_not!(self) {
            Some(ch) => ch,
            None => {
                return Err(Error::Parsing(ParsingUnfinished));
            }
        };

        match peek_ch {
            b'"' => {
                let v = parse_string_inner(self)?;
                visitor.visit_bytes(&v)
            }
            b'[' => self.deserialize_seq(visitor),
            _ => unexpected_character!(self),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let peek_ch = match eat_whitespace_until_not!(self) {
            Some(ch) => ch,
            None => {
                return Err(Error::Parsing(ParsingUnfinished));
            }
        };

        match peek_ch {
            b'n' => {
                self.discard_char();
                match_str!(self, b"ull");
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let peek_ch = match eat_whitespace_until_not!(self) {
            Some(ch) => ch,
            None => {
                return Err(Error::Parsing(ParsingUnfinished));
            }
        };

        match peek_ch {
            b'n' => {
                self.discard_char();
                match_str!(self, b"ull");
                visitor.visit_unit()
            }
            _ => unexpected_character!(self),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let peek_ch = match eat_whitespace_until_not!(self) {
            Some(ch) => ch,
            None => {
                return Err(Error::Parsing(ParsingUnfinished));
            }
        };

        match peek_ch {
            b'[' => {
                self.discard_char();
                let value = visitor.visit_seq(SeqAssistant::new(self))?;

                let peek_ch_inner = match eat_whitespace_until_not!(self) {
                    Some(ch) => ch,
                    None => {
                        return Err(Error::Parsing(ParsingUnfinished));
                    }
                };

                match peek_ch_inner {
                    b']' => {
                        self.discard_char();
                        Ok(value)
                    }
                    _ => unexpected_character!(self),
                }
            }
            _ => unexpected_character!(self),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let peek_ch = match eat_whitespace_until_not!(self) {
            Some(ch) => ch,
            None => {
                return Err(Error::Parsing(ParsingUnfinished));
            }
        };

        match peek_ch {
            b'{' => {
                self.discard_char();
                let value = visitor.visit_map(SeqAssistant::new(self))?;

                let peek_ch_inner = match eat_whitespace_until_not!(self) {
                    Some(ch) => ch,
                    None => {
                        return Err(Error::Parsing(ParsingUnfinished));
                    }
                };

                match peek_ch_inner {
                    b'}' => {
                        self.discard_char();
                        Ok(value)
                    }
                    _ => unexpected_character!(self),
                }
            }
            _ => unexpected_character!(self),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let peek_ch = match eat_whitespace_until_not!(self) {
            Some(ch) => ch,
            None => {
                return Err(Error::Parsing(ParsingUnfinished));
            }
        };

        match peek_ch {
            b'"' => {
                #[cfg(feature = "c_adapter")]
                return visitor
                    .visit_enum(self.de_parse_string()?.into_string()?.into_deserializer());

                #[cfg(not(feature = "c_adapter"))]
                visitor.visit_enum(self.de_parse_string()?.into_deserializer())
            }
            _ => {
                if self.next_char()? == Some(b'{') {
                    eat_whitespace_until_not!(self);
                    let value = visitor.visit_enum(EnumAssistant::new(self))?;
                    eat_whitespace_until_not!(self);

                    if self.next_char()? == Some(b'}') {
                        Ok(value)
                    } else {
                        unexpected_character!(self)
                    }
                } else {
                    unexpected_character!(self)
                }
            }
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct SeqAssistant<'a, R: 'a>
where
    R: BytesReader + Cacheable,
{
    deserializer: &'a mut Deserializer<R>,
    is_first: bool,
}

impl<'a, R: 'a> SeqAssistant<'a, R>
where
    R: BytesReader + Cacheable,
{
    fn new(deserializer: &'a mut Deserializer<R>) -> Self {
        SeqAssistant {
            deserializer,
            is_first: true,
        }
    }
}

impl<'de, 'a, R> SeqAccess<'de> for SeqAssistant<'a, R>
where
    R: BytesReader + Cacheable,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        let peek_ch = match eat_whitespace_until_not!(self.deserializer) {
            Some(b']') => return Ok(None),
            Some(b',') if !self.is_first => {
                self.deserializer.discard_char();
                eat_whitespace_until_not!(self.deserializer)
            }
            Some(ch) => {
                if self.is_first {
                    self.is_first = false;
                    Some(ch)
                } else {
                    let position = self.deserializer.reader.position();
                    return Err(Error::Parsing(MissingComma(
                        position.line(),
                        position.column(),
                    )));
                }
            }
            None => return Err(Error::Parsing(ParsingUnfinished)),
        };

        match peek_ch {
            Some(b']') => {
                let position = self.deserializer.reader.position();
                Err(Error::Parsing(TrailingComma(
                    position.line(),
                    position.column(),
                )))
            }
            Some(_) => Ok(Some(seed.deserialize(&mut *self.deserializer)?)),
            None => Err(Error::Parsing(ParsingUnfinished)),
        }
    }
}

impl<'de, 'a, R> MapAccess<'de> for SeqAssistant<'a, R>
where
    R: BytesReader + Cacheable,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        let peek_ch = match eat_whitespace_until_not!(self.deserializer) {
            Some(b'}') => return Ok(None),
            Some(b',') if !self.is_first => {
                self.deserializer.discard_char();
                eat_whitespace_until_not!(self.deserializer)
            }
            Some(ch) => {
                if self.is_first {
                    self.is_first = false;
                    Some(ch)
                } else {
                    let position = self.deserializer.reader.position();
                    return Err(Error::Parsing(MissingComma(
                        position.line(),
                        position.column(),
                    )));
                }
            }
            None => {
                return Err(Error::Parsing(ParsingUnfinished));
            }
        };

        match peek_ch {
            Some(b'"') => Ok(Some(seed.deserialize(&mut *self.deserializer)?)),
            Some(b'}') => {
                let position = self.deserializer.reader.position();
                Err(Error::Parsing(TrailingComma(
                    position.line(),
                    position.column(),
                )))
            }
            // Object key must be String.
            _ => unexpected_character!(self.deserializer),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: DeserializeSeed<'de>,
    {
        match eat_whitespace_until_not!(self.deserializer) {
            Some(b':') => {
                self.deserializer.discard_char();
                eat_whitespace_until_not!(self.deserializer);
                seed.deserialize(&mut *self.deserializer)
            }
            Some(_ch) => {
                let position = self.deserializer.reader.position();
                Err(Error::Parsing(MissingColon(
                    position.line(),
                    position.column(),
                )))
            }
            None => Err(Error::Parsing(ParsingUnfinished)),
        }
    }
}

struct EnumAssistant<'a, R: 'a>
where
    R: BytesReader + Cacheable,
{
    deserializer: &'a mut Deserializer<R>,
}

impl<'a, R: 'a> EnumAssistant<'a, R>
where
    R: BytesReader + Cacheable,
{
    fn new(deserializer: &'a mut Deserializer<R>) -> Self {
        EnumAssistant { deserializer }
    }
}

impl<'de, 'a, R: 'a> EnumAccess<'de> for EnumAssistant<'a, R>
where
    R: BytesReader + Cacheable,
{
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.deserializer)?;

        match eat_whitespace_until_not!(self.deserializer) {
            Some(b':') => {
                self.deserializer.discard_char();
                Ok((value, self))
            }
            _ => unexpected_character!(self.deserializer),
        }
    }
}

impl<'de, 'a, R: 'a> VariantAccess<'de> for EnumAssistant<'a, R>
where
    R: BytesReader + Cacheable,
{
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        serde::de::Deserialize::deserialize(self.deserializer)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.deserializer)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_seq(self.deserializer, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_map(self.deserializer, visitor)
    }
}

#[cfg(test)]
mod ut_test_for_deserializer {
    use crate::deserializer::{from_slice, from_str};
    use serde::Deserialize;
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::option::Option;

    /// UT test to deserialize simple types
    ///
    /// # Title
    /// ut_deserialize_simple
    ///
    /// # Brief
    /// 1.Uses deserializer::from_slice method to deserialize simple types.
    /// 2.Checks if the test results are correct.
    #[test]
    fn ut_deserialize_simple() {
        let slice_null = b"null";
        let expected: Option<i32> = None;
        assert_eq!(expected, from_slice(slice_null).unwrap());

        let slice_bool = b"true";
        let expected = true;
        assert_eq!(expected, from_slice(slice_bool).unwrap());

        let slice_bool = b"false";
        let expected = false;
        assert_eq!(expected, from_slice(slice_bool).unwrap());

        let slice_num = b"123";
        let expected: u8 = 123;
        assert_eq!(expected, from_slice(slice_num).unwrap());

        let slice_num = b"123";
        let expected: u16 = 123;
        assert_eq!(expected, from_slice(slice_num).unwrap());

        let slice_num = b"123";
        let expected: u32 = 123;
        assert_eq!(expected, from_slice(slice_num).unwrap());

        let slice_num = b"-12";
        let expected: i8 = -12;
        assert_eq!(expected, from_slice(slice_num).unwrap());

        let slice_num = b"-12";
        let expected: i16 = -12;
        assert_eq!(expected, from_slice(slice_num).unwrap());

        let slice_num = b"-321";
        let expected: i32 = -321;
        assert_eq!(expected, from_slice(slice_num).unwrap());

        let slice_num = b"-321.123";
        let expected: f32 = -321.123;
        assert_eq!(expected, from_slice(slice_num).unwrap());

        let slice_char = b"\"c\"";
        let expected = 'c';
        assert_eq!(expected, from_slice::<char>(slice_char).unwrap());

        let slice_str = b"\"string\"";
        let expected = String::from("string");
        assert_eq!(expected, from_slice::<String>(slice_str).unwrap());

        let slice_option = b"true";
        let expected = Some(true);
        assert_eq!(expected, from_slice::<Option<bool>>(slice_option).unwrap());

        let slice_option = b"null";
        let expected = None;
        assert_eq!(expected, from_slice::<Option<bool>>(slice_option).unwrap());

        let slice_seq = b"[1, 2, 3]";
        let expected: Vec<u8> = vec![1, 2, 3];
        assert_eq!(expected, from_slice::<Vec<u8>>(slice_seq).unwrap());

        let slice_map = r#"{ "apple" : 3 }"#.as_bytes();
        let mut expected = HashMap::new();
        expected.insert("appple", 3);
        assert_eq!(
            expected,
            crate::from_slice::<HashMap<&str, i32>>(slice_map).unwrap()
        );
    }

    #[test]
    fn de_map() {
        use std::collections::HashMap;
        let slice_map = r#"{ "apple" : 3 }"#.as_bytes();
        let mut expected = HashMap::new();
        expected.insert(String::from("apple"), 3);
        assert_eq!(
            expected,
            from_slice::<HashMap<String, i32>>(slice_map).unwrap()
        );
    }

    /// UT test to deserialize simple types with abnormal input of JSON.
    ///
    /// # Title
    /// ut_deserialize_simple_error
    ///
    /// # Brief
    /// 1.Uses deserializer::from_slice method to deserialize simple types with abnormal input of JSON.
    /// 2.Checks if the test results are correct.
    #[test]
    fn ut_deserialize_simple_error() {
        // The following is the test for abnormal input of JSON.
        let incorrect_input = b"nul";
        let res = from_slice::<i32>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"  ";
        let res = from_slice::<bool>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"tru";
        let res = from_slice::<bool>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"fals";
        let res = from_slice::<bool>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"ruet";
        let res = from_slice::<bool>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"  ";
        let res = from_slice::<u64>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"12x";
        let res = from_slice::<u64>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"-12x";
        let res = from_slice::<i64>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"-12.21x";
        let res = from_slice::<f64>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"  ";
        let res = from_slice::<String>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"string";
        let res = from_slice::<String>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"\"string";
        let res = from_slice::<String>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"  ";
        let res = from_slice::<Option<bool>>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"  ";
        let res = from_slice::<Vec<u8>>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"  [";
        let res = from_slice::<Vec<u8>>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = b"  [ 1";
        let res = from_slice::<Vec<u8>>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = r#"  "#.as_bytes();
        let res = from_slice::<HashMap<bool, i32>>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = r#"{ true x: 1 }"#.as_bytes();
        let res = from_slice::<HashMap<bool, i32>>(incorrect_input);
        assert!(res.is_err());

        let incorrect_input = r#"{ : 1 }"#.as_bytes();
        let res = from_slice::<HashMap<bool, i32>>(incorrect_input);
        assert!(res.is_err());
    }

    /// UT test to deserialize struct
    ///
    /// # Title
    /// ut_deserialize_struct
    ///
    /// # Brief
    /// 1.Uses deserializer::from_str method to deserialize struct.
    /// 2.Checks if the test results are correct.
    #[test]
    fn ut_deserialize_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct TestUnit;
        let str = "null";
        let expected = TestUnit;
        assert_eq!(expected, from_str(str).unwrap());

        #[derive(Deserialize, PartialEq, Debug)]
        struct TestNewtype(u32);
        let str = "123";
        let expected = TestNewtype(123);
        assert_eq!(expected, from_str(str).unwrap());

        #[derive(Deserialize, PartialEq, Debug)]
        struct TestTuple(u32, u32, bool);
        let str = "[123,321,true]";
        let expected = TestTuple(123, 321, true);
        assert_eq!(expected, from_str(str).unwrap());

        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: Vec<String>,
            tup: (i32, i32, i32),
        }

        let str = r#"{"int":1,"seq":["abcd","efgh"],"tup":[1,2,3]}"#;
        let expected = Test {
            int: 1,
            seq: vec![String::from("abcd"), String::from("efgh")],
            tup: (1, 2, 3),
        };
        assert_eq!(expected, from_str(str).unwrap());

        let str = r#"{
            "int" : 1 ,
            "seq" : ["abcd" , "efgh" ], 
            "tup" : [1, 2 ,3 ]
        }"#;
        let expected = Test {
            int: 1,
            seq: vec![String::from("abcd"), String::from("efgh")],
            tup: (1, 2, 3),
        };
        assert_eq!(expected, from_str(str).unwrap());

        // The following is the test for abnormal input of JSON.
        // Missing '}' at the end.
        let str_abnormal = r#"{"int":1,"seq":["abcd","efgh"],"tup":[1,2,3]"#;
        let res = from_str::<Test>(str_abnormal);
        assert!(res.is_err());

        // Missing ','.
        let str_abnormal = r#"{"int":1 "seq":["abcd","efgh"],"tup":[1,2,3]"#;
        let res = from_str::<Test>(str_abnormal);
        assert!(res.is_err());

        // Trailing ','.
        let str_abnormal = r#"{"int":1, "seq":["abcd","efgh",],"tup":[1,2,3]"#;
        let res = from_str::<Test>(str_abnormal);
        assert!(res.is_err());

        // Missing ':'.
        let str_abnormal = r#"{"int":1, "seq":["abcd","efgh"],"tup" [1,2,3]"#;
        let res = from_str::<Test>(str_abnormal);
        assert!(res.is_err());

        // Incorrect field name.
        let str_abnormal = r#"{"it":1, "sq" : ["abcd","efgh"],"tp": [1,2,3]"#;
        let res = from_str::<Test>(str_abnormal);
        assert!(res.is_err());
    }

    /// UT test to deserialize enum
    ///
    /// # Title
    /// ut_deserialize_enum
    ///
    /// # Brief
    /// 1.Uses deserializer::from_slice method to deserialize enum.
    /// 2.Checks if the test results are correct.
    #[test]
    fn ut_deserialize_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E<'a> {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
            Reference(Cow<'a, str>),
        }

        let u = r#""Unit""#.as_bytes();
        let expected = E::Unit;
        assert_eq!(expected, from_slice(u).unwrap());

        let n = r#"{"Newtype":1}"#.as_bytes();
        let expected = E::Newtype(1);
        assert_eq!(expected, from_slice(n).unwrap());

        let t = r#"{"Tuple":[1,2]}"#.as_bytes();
        let expected = E::Tuple(1, 2);
        assert_eq!(expected, from_slice(t).unwrap());

        let s = r#"{"Struct":{"a":1}}"#.as_bytes();
        let expected = E::Struct { a: 1 };
        assert_eq!(expected, from_slice(s).unwrap());

        let s = r#"{"Reference":"reference"}"#.as_bytes();
        let expected = E::Reference(Cow::from("reference"));
        assert_eq!(expected, from_slice(s).unwrap());

        // The following is the test for abnormal input of JSON.
        let slice_abnormal = r#"   "#.as_bytes();
        let res = from_slice::<E>(slice_abnormal);
        assert!(res.is_err());

        let slice_abnormal = r#"x"#.as_bytes();
        let res = from_slice::<E>(slice_abnormal);
        assert!(res.is_err());

        let slice_abnormal = r#""Unit"#.as_bytes();
        let res = from_slice::<E>(slice_abnormal);
        assert!(res.is_err());

        let slice_abnormal = r#"{"Newtype" 1"#.as_bytes();
        let res = from_slice::<E>(slice_abnormal);
        assert!(res.is_err());

        let slice_abnormal = r#"{"Newtype":1"#.as_bytes();
        let res = from_slice::<E>(slice_abnormal);
        assert!(res.is_err());

        let slice_abnormal = r#"{"Tuple":[1,2}"#.as_bytes();
        let res = from_slice::<E>(slice_abnormal);
        assert!(res.is_err());
    }
}
