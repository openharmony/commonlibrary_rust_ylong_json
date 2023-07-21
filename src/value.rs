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

mod array;
mod index;
mod number;
mod object;

pub use array::Array;
pub use index::Index;
pub use number::Number;
pub use object::Object;

use crate::{start_parsing, CompactEncoder, Error, FormattedEncoder};
use core::fmt::{Debug, Display, Formatter};
use core::str::FromStr;
#[cfg(feature = "c_adapter")]
use std::ffi::CString;
use std::io::{Read, Write};

#[cfg(feature = "c_adapter")]
pub type JsonString = CString;
#[cfg(not(feature = "c_adapter"))]
pub type JsonString = String;

use crate::deserializer::Deserializer;
#[cfg(not(feature = "c_adapter"))]
use std::fs::File;
#[cfg(not(feature = "c_adapter"))]
use std::path::Path;

/// There are 6 types of values that appear in Json text:
///
/// 1.Null: Null tupe
///
/// 2.Boolean: Boolean type
///
/// 3.Number: Numerical type
///
/// 4.String: String type
///
/// 5.Array: Array type
///
/// 6.Object: Object type
///
/// RFC 7159 3. Values say
/// “A Json value must be an object, an array, a number, a string, or a text string of false, null, or true.”
///
/// # features
/// Uses"c_adatper" feature:
/// In order to adapt the C encapsulation layer interface, the structure changes the
/// underlying implementation of String, and uses CString to get the char* pointer easily.
// TODO: Enhance the encapsulation of JsonValue, makes users can't use enum directly.
#[derive(Clone)]
pub enum JsonValue {
    /// Null type
    Null,

    /// Boolean type
    Boolean(bool),

    /// Numerical type
    Number(Number),

    /// String type
    String(JsonString),

    /// Array type
    Array(Array),

    /// Object type
    Object(Object),
}

/// JsonValue print method 1, prints the content directly (without extra double quotes).
///
/// # Examples
/// ```
/// use ylong_json::JsonValue;
///
/// let value: JsonValue = "hello".into();
/// let string = format!("{value}");
/// assert_eq!(string, "\"hello\"");
/// ```
impl Display for JsonValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Number(n) => Display::fmt(n, f),
            // The Debug output is the same for CString and String.
            Self::String(s) => write!(f, "{s:?}"),
            Self::Array(a) => Display::fmt(a, f),
            Self::Object(o) => Display::fmt(o, f),
        }
    }
}

impl Debug for JsonValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl JsonValue {
    /// Creates an instance of JsonValue for Null type.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let value = JsonValue::new_null();
    /// assert_eq!(value, JsonValue::Null);
    /// ```
    pub fn new_null() -> Self {
        Self::Null
    }

    /// Creates an instance of JsonValue for Boolean type.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let value = JsonValue::new_boolean(true);
    /// assert_eq!(value, JsonValue::Boolean(true));
    /// ```
    pub fn new_boolean(boolean: bool) -> Self {
        Self::Boolean(boolean)
    }

    /// Creates an instance of JsonValue for Numerical type.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let value = JsonValue::new_number(0.0.into());
    /// assert_eq!(value, JsonValue::Number(0.0.into()));
    /// ```
    pub fn new_number(number: Number) -> Self {
        Self::Number(number)
    }

    /// Creates an instance of JsonValue for String type.
    ///
    /// # Examples
    /// ```not run
    /// use ylong_json::JsonValue;
    ///
    /// let value = JsonValue::new_string("Hello World");
    /// // When open "c_adapter" feature, the internal value is String.
    /// assert_eq!(value, JsonValue::String(String::from("Hello World")));
    ///
    /// // When opening "c_adapter" feature, the internal value is CString.
    /// // assert_eq!(value, JsonValue::String(CString::new("Hello World")));
    /// ```
    pub fn new_string(str: &str) -> Self {
        // The underlying implementation of String here is CString.
        #[cfg(feature = "c_adapter")]
        let result = Self::String(JsonString::new(str).unwrap());

        #[cfg(not(feature = "c_adapter"))]
        let result = Self::String(JsonString::from(str));

        result
    }

    /// Creates an instance of JsonValue for Array type.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Array};
    ///
    /// let value = JsonValue::new_array(Array::new());
    /// assert_eq!(value, JsonValue::Array(Array::new()));
    /// ```
    pub fn new_array(array: Array) -> Self {
        Self::Array(array)
    }

    /// Creates an instance of JsonValue for Object type.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object};
    ///
    /// let value = JsonValue::new_object(Object::new());
    /// assert_eq!(value, JsonValue::Object(Object::new()));
    /// ```
    pub fn new_object(object: Object) -> Self {
        Self::Object(object)
    }

    /// Determines whether JsonValue is Null type. Returns true if yes, false if no.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let null_value = JsonValue::new_null();
    /// assert_eq!(null_value.is_null(), true);
    ///
    /// let other_value = JsonValue::new_number(0.0.into());
    /// assert_eq!(other_value.is_null(), false);
    /// ```
    pub fn is_null(&self) -> bool {
        matches!(*self, Self::Null)
    }

    /// Determines whether JsonValue is Boolean type. Returns true if yes, false if no.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let boolean_value = JsonValue::new_boolean(true);
    /// assert_eq!(boolean_value.is_boolean(), true);
    ///
    /// let other_value = JsonValue::new_null();
    /// assert_eq!(other_value.is_boolean(), false);
    /// ```
    pub fn is_boolean(&self) -> bool {
        matches!(*self, Self::Boolean(_))
    }

    /// Determines whether JsonValue is true. Returns true if yes, false if no.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let true_value = JsonValue::new_boolean(true);
    /// assert_eq!(true_value.is_true(), true);
    ///
    /// let other_value = JsonValue::new_boolean(false);
    /// assert_eq!(other_value.is_true(), false);
    /// ```
    pub fn is_true(&self) -> bool {
        match *self {
            Self::Boolean(x) => x,
            _ => false,
        }
    }

    /// Determines whether JsonValue is false. Returns true if yes, false if no.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let false_value = JsonValue::new_boolean(false);
    /// assert_eq!(false_value.is_false(), true);
    ///
    /// let other_value = JsonValue::new_boolean(true);
    /// assert_eq!(other_value.is_false(), false);
    /// ```
    pub fn is_false(&self) -> bool {
        match *self {
            Self::Boolean(x) => !x,
            _ => false,
        }
    }

    /// Determines whether JsonValue is Numerical type. Returns true if yes, false if no.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let number_value = JsonValue::new_number(0.0.into());
    /// assert_eq!(number_value.is_number(), true);
    ///
    /// let other_value = JsonValue::new_null();
    /// assert_eq!(other_value.is_number(), false);
    /// ```
    pub fn is_number(&self) -> bool {
        matches!(*self, Self::Number(_))
    }

    /// Determines whether JsonValue is String type. Returns true if yes, false if no.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let string_value = JsonValue::new_string("Hello World");
    /// assert_eq!(string_value.is_string(), true);
    ///
    /// let other_value = JsonValue::new_null();
    /// assert_eq!(other_value.is_string(), false);
    /// ```
    pub fn is_string(&self) -> bool {
        matches!(*self, Self::String(_))
    }

    /// Determines whether JsonValue is Array type. Returns true if yes, false if no.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Array};
    ///
    /// let array_value = JsonValue::new_array(Array::new());
    /// assert_eq!(array_value.is_array(), true);
    ///
    /// let other_value = JsonValue::new_null();
    /// assert_eq!(other_value.is_array(), false);
    /// ```
    pub fn is_array(&self) -> bool {
        matches!(*self, Self::Array(_))
    }

    /// Determines whether JsonValue is Object type. Returns true if yes, false if no.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object};
    ///
    /// let object_value = JsonValue::new_object(Object::new());
    /// assert_eq!(object_value.is_object(), true);
    ///
    /// let other_value = JsonValue::new_null();
    /// assert_eq!(other_value.is_object(), false);
    /// ```
    pub fn is_object(&self) -> bool {
        matches!(*self, Self::Object(_))
    }

    /// Trys to convert JsonValue to a common reference of Boolean type. Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Error};
    ///
    /// let boolean_value = JsonValue::new_boolean(true);
    /// assert_eq!(boolean_value.try_as_boolean().unwrap(), &true);
    ///
    /// let other_value = JsonValue::new_null();
    /// assert!(other_value.try_as_boolean().is_err());
    /// ```
    pub fn try_as_boolean(&self) -> Result<&bool, Error> {
        match self {
            Self::Boolean(boolean) => Ok(boolean),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to a common reference of Numerical type. Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Number, Error};
    ///
    /// let number_value = JsonValue::new_number(0.0.into());
    /// assert_eq!(number_value.try_as_number().unwrap(), &Number::from(0.0));
    ///
    /// let other_value = JsonValue::new_null();
    /// assert!(other_value.try_as_number().is_err());
    /// ```
    pub fn try_as_number(&self) -> Result<&Number, Error> {
        match self {
            Self::Number(number) => Ok(number),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to a common reference of String type. Conversion failure will return Error.
    ///
    /// # Examples
    /// ```no_run
    /// #[cfg(feature = "c_adapter")]
    /// use std::ffi::CString;
    /// use ylong_json::{JsonValue, Error};
    ///
    /// let string_value = JsonValue::new_string("Hello World");
    /// #[cfg(feature = "c_adapter")]
    /// assert_eq!(string_value.try_as_string().unwrap(), &CString::new("Hello World").unwrap());
    /// #[cfg(not(feature = "c_adapter"))]
    /// assert_eq!(string_value.try_as_string().unwrap(), &String::from("Hello World"));
    /// // When opening "c_adapter" feature, the underlying implementation is CString.
    /// //assert_eq!(string_value.try_as_string().unwrap(), &CString::new("Hello World").unwrap());
    ///
    /// let other_value = JsonValue::new_null();
    /// assert!(other_value.try_as_string().is_err());
    /// ```
    pub fn try_as_string(&self) -> Result<&JsonString, Error> {
        match self {
            Self::String(string) => Ok(string),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to a common reference of Array type. Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Array, Error};
    ///
    /// let array_value = JsonValue::new_array(Array::new());
    /// assert_eq!(array_value.try_as_array().unwrap(), &Array::new());
    ///
    /// let other_value = JsonValue::new_null();
    /// assert!(other_value.try_as_array().is_err());
    /// ```
    pub fn try_as_array(&self) -> Result<&Array, Error> {
        match self {
            Self::Array(array) => Ok(array),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to a common reference of Object type. Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object, Error};
    ///
    /// let array_value = JsonValue::new_object(Object::new());
    /// assert_eq!(array_value.try_as_object().unwrap(), &Object::new());
    ///
    /// let other_value = JsonValue::new_null();
    /// assert!(other_value.try_as_object().is_err());
    /// ```
    pub fn try_as_object(&self) -> Result<&Object, Error> {
        match self {
            Self::Object(object) => Ok(object),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to a mutable reference of Boolean type. Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Error};
    ///
    /// let mut boolean_value = JsonValue::new_boolean(true);
    /// assert_eq!(boolean_value.try_as_mut_boolean().unwrap(), &mut true);
    ///
    /// let mut other_value = JsonValue::new_null();
    /// assert!(other_value.try_as_mut_boolean().is_err());
    /// ```
    pub fn try_as_mut_boolean(&mut self) -> Result<&mut bool, Error> {
        match self {
            Self::Boolean(boolean) => Ok(boolean),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to a mutable reference of Numerical type. Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Number, Error};
    ///
    /// let mut number_value = JsonValue::new_number(0.0.into());
    /// assert_eq!(number_value.try_as_mut_number().unwrap(), &mut Number::from(0.0));
    ///
    /// let mut other_value = JsonValue::new_null();
    /// assert!(other_value.try_as_mut_number().is_err());
    /// ```
    pub fn try_as_mut_number(&mut self) -> Result<&mut Number, Error> {
        match self {
            Self::Number(number) => Ok(number),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to a mutable reference of String type. Conversion failure will return Error.
    ///
    /// # Examples
    /// ```no_run
    /// #[cfg(feature = "c_adapter")]
    /// use std::ffi::CString;
    /// use ylong_json::{JsonValue, Error};
    ///
    /// let mut string_value = JsonValue::new_string("Hello World");
    /// #[cfg(feature = "c_adapter")]
    /// assert_eq!(string_value.try_as_mut_string().unwrap(), &mut CString::new("Hello World").unwrap());
    /// #[cfg(not(feature = "c_adapter"))]
    /// assert_eq!(string_value.try_as_mut_string().unwrap(), &mut String::from("Hello World"));
    /// // When opening "c_adapter" feature, the underlying implementation is CString.
    /// //assert_eq!(string_value.try_as_mut_string().unwrap(), &mut CString::new("Hello World").unwrap());
    ///
    /// let mut other_value = JsonValue::new_null();
    /// assert!(other_value.try_as_mut_string().is_err());
    /// ```
    pub fn try_as_mut_string(&mut self) -> Result<&mut JsonString, Error> {
        match self {
            Self::String(string) => Ok(string),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to a mutable reference of Array type. Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Error, Array};
    ///
    /// let mut array_value = JsonValue::new_array(Array::new());
    /// assert_eq!(array_value.try_as_mut_array().unwrap(), &mut Array::new());
    ///
    /// let mut other_value = JsonValue::new_null();
    /// assert!(other_value.try_as_mut_array().is_err());
    /// ```
    pub fn try_as_mut_array(&mut self) -> Result<&mut Array, Error> {
        match self {
            Self::Array(array) => Ok(array),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to a mutable reference of Object type. Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Error, Object};
    ///
    /// let mut object_value = JsonValue::new_object(Object::new());
    /// assert_eq!(object_value.try_as_mut_object().unwrap(), &mut Object::new());
    ///
    /// let mut other_value = JsonValue::new_null();
    /// assert!(other_value.try_as_mut_object().is_err());
    /// ```
    pub fn try_as_mut_object(&mut self) -> Result<&mut Object, Error> {
        match self {
            Self::Object(object) => Ok(object),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to Boolean type. This method transfers ownership.
    /// Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Error};
    ///
    /// let boolean_value = JsonValue::new_boolean(true);
    /// assert_eq!(boolean_value.try_into_boolean().unwrap(), true);
    ///
    /// let other_value = JsonValue::new_null();
    /// assert!(other_value.try_into_boolean().is_err());
    /// ```
    pub fn try_into_boolean(self) -> Result<bool, Error> {
        match self {
            Self::Boolean(boolean) => Ok(boolean),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to Numerical type. This method transfers ownership.
    /// Conversion failure will return Error.
    ///
    /// The value will be output as f64. If you want to output other types, use 'as' to convert.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Number, Error};
    ///
    /// let number_value = JsonValue::new_number(0.0.into());
    /// assert_eq!(number_value.try_into_number().unwrap(), Number::from(0.0));
    ///
    /// let other_value = JsonValue::new_null();
    /// assert!(other_value.try_into_number().is_err());
    /// ```
    pub fn try_into_number(self) -> Result<Number, Error> {
        match self {
            Self::Number(number) => Ok(number),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to String type. This method transfers ownership.
    /// Conversion failure will return Error.
    ///
    /// # Examples
    /// ```no_run
    /// #[cfg(feature = "c_adapter")]
    /// use std::ffi::CString;
    /// use ylong_json::{JsonValue, Error};
    ///
    /// let string_value = JsonValue::new_string("Hello World");
    /// #[cfg(feature = "c_adapter")]
    /// assert_eq!(string_value.try_into_string().unwrap(), CString::new("Hello World").unwrap());
    /// #[cfg(not(feature = "c_adapter"))]
    /// assert_eq!(string_value.try_into_string().unwrap(), String::from("Hello World"));
    /// // When opening "c_adapter" feature, the underlying implementation is CString.
    /// //assert_eq!(string_value.try_into_string().unwrap(), CString::new("Hello World").unwrap());
    ///
    /// let other_value = JsonValue::new_null();
    /// assert!(other_value.try_into_string().is_err());
    /// ```
    pub fn try_into_string(self) -> Result<JsonString, Error> {
        match self {
            Self::String(string) => Ok(string),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to Array type. This method transfers ownership.
    /// Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Error, Array};
    ///
    /// let array_value = JsonValue::new_array(Array::new());
    /// assert_eq!(array_value.try_into_array().unwrap(), Array::new());
    ///
    /// let other_value = JsonValue::new_null();
    /// assert!(other_value.try_into_array().is_err());
    /// ```
    pub fn try_into_array(self) -> Result<Array, Error> {
        match self {
            Self::Array(array) => Ok(array),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to convert JsonValue to Object type. This method transfers ownership.
    /// Conversion failure will return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Error, Object};
    ///
    /// let object_value = JsonValue::new_object(Object::new());
    /// assert_eq!(object_value.try_into_object().unwrap(), Object::new());
    ///
    /// let other_value = JsonValue::new_null();
    /// assert!(other_value.try_into_object().is_err());
    /// ```
    pub fn try_into_object(self) -> Result<Object, Error> {
        match self {
            Self::Object(object) => Ok(object),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys to remove a member form an Object or Array. If the member is found by Index,
    /// gets value of the member from the Object or Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Object, JsonValue};
    ///
    /// let mut object = Object::new();
    /// object.insert(String::from("key"), "value".into());
    ///
    /// let mut value: JsonValue = object.into();
    /// assert_eq!(value["key"], "value".into());
    ///
    /// value.remove("key");
    /// assert_eq!(value["key"], JsonValue::Null);
    /// ```
    pub fn remove<I: index::Index>(&mut self, index: I) -> Option<JsonValue> {
        index.index_remove(self)
    }

    /// Reads the contents from the file and Trys to deserialize to a JsonValue instance.
    ///
    /// # Examples
    /// ```not run
    /// use ylong_json::JsonValue;
    ///
    /// let value = JsonValue::from_file("./json.txt").unwrap();
    /// ```
    #[cfg(not(feature = "c_adapter"))]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut file = File::open(path.as_ref())?;
        Self::from_reader(&mut file)
    }

    /// Gets the text from an object that implements the Read trait provided
    /// by the standard library and Trys to deserialize it into a JsonValue instance.
    ///
    /// # Examples
    /// ```not run
    /// use ylong_json::JsonValue;
    /// use std::fs::File;
    ///
    /// let mut file = File::open("./json.txt").unwrap();
    /// let value = JsonValue::from_reader(&mut file).unwrap();
    /// ```
    pub fn from_reader<R: Read>(input: R) -> Result<Self, Error> {
        let mut deserializer = Deserializer::new_from_io(input);
        start_parsing(&mut deserializer)
    }

    /// Reads the text from a type that can be converted to [u8] and Trys to deserialize it to a Json instance.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let text = r#"
    /// {
    ///     "key": "value"
    /// }
    /// "#;
    /// let value = JsonValue::from_text(text.as_bytes()).unwrap();
    ///
    /// assert_eq!(value["key"], "value".into());
    /// ```
    pub fn from_text<T: AsRef<[u8]>>(text: T) -> Result<Self, Error> {
        let mut deserializer = Deserializer::new_from_slice(text.as_ref());
        start_parsing(&mut deserializer)
    }

    /// Serializes the JsonValue instance to a formatted string with additional whitespace characters.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let text = r#"{
    ///     "key": "value"
    /// }
    /// "#;
    /// let value = JsonValue::from_text(text.as_bytes()).unwrap();
    /// let string = value.to_formatted_string().unwrap();
    /// assert_eq!(string, text);
    /// ```
    pub fn to_formatted_string(&self) -> Result<std::string::String, Error> {
        let mut vec = Vec::new();
        self.formatted_encode(&mut vec)?;
        Ok(unsafe { std::string::String::from_utf8_unchecked(vec) })
    }

    /// Serializes the JsonValue instance to a one-line string with no additional whitespace.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let text = r#"{"key":"value"}"#;
    /// let value = JsonValue::from_text(text.as_bytes()).unwrap();
    /// let string = value.to_compact_string().unwrap();
    /// assert_eq!(string, text);
    /// ```
    pub fn to_compact_string(&self) -> Result<std::string::String, Error> {
        let mut vec = Vec::new();
        self.compact_encode(&mut vec)?;
        Ok(unsafe { std::string::String::from_utf8_unchecked(vec) })
    }

    /// Serializes the JsonValue instance to a formatted string with additional whitespace characters.
    /// And outputs to the specified location as a stream of bytes.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let text = r#"{
    ///     "key": "value"
    /// }
    /// "#;
    /// let value = JsonValue::from_text(text.as_bytes()).unwrap();
    /// let mut vec = Vec::new();
    /// value.formatted_encode(&mut vec).unwrap();
    /// assert_eq!(vec, text.as_bytes());
    /// ```
    pub fn formatted_encode<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        let mut encoder = FormattedEncoder::new(output);
        encoder.encode(self)
    }

    /// Serializes the JsonValue instance to a one-line string with no additional whitespace.
    /// And outputs to the specified location as a stream of bytes.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let text = r#"{"key":"value"}"#;
    /// let value = JsonValue::from_text(text.as_bytes()).unwrap();
    /// let mut vec = Vec::new();
    /// value.compact_encode(&mut vec).unwrap();
    /// assert_eq!(vec, text.as_bytes());
    /// ```
    pub fn compact_encode<W: Write>(&self, output: &mut W) -> Result<(), Error> {
        let mut encoder = CompactEncoder::new(output);
        encoder.encode(self)
    }
}

impl FromStr for JsonValue {
    type Err = Error;

    /// Generates an instance of JsonValue from &str.
    ///
    /// # Examples
    /// ```
    /// use core::str::FromStr;
    /// use ylong_json::JsonValue;
    ///
    /// let text = r#"
    /// {
    ///     "key": "value"
    /// }
    /// "#;
    /// let value = JsonValue::from_str(text).unwrap();
    ///
    /// assert_eq!(value["key"], "value".into());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_text(s)
    }
}

impl PartialEq for JsonValue {
    /// Determines whether two Jsonvalues are equal.
    /// Only the same type can be compared; different types simply return false.
    ///
    /// If types are the same, false is returned only if the internal variables are exactly equal.
    /// For example, when comparing arrays, two Arrays are considered equal
    /// only if all their JsonValues have the same position and value.
    /// When comparing objects, two Objects are considered equal
    /// only if all of their key/value pairs match one another.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Array, Object};
    ///
    /// assert_eq!(JsonValue::new_null(), JsonValue::new_null());
    /// assert_eq!(JsonValue::new_boolean(true), JsonValue::new_boolean(true));
    /// assert_eq!(JsonValue::new_number(0.0.into()), JsonValue::new_number(0.0.into()));
    /// assert_eq!(JsonValue::new_string(""), JsonValue::new_string(""));
    /// assert_eq!(JsonValue::new_array(Array::new()), JsonValue::new_array(Array::new()));
    /// assert_eq!(JsonValue::new_object(Object::new()), JsonValue::new_object(Object::new()));
    ///
    /// assert_ne!(JsonValue::new_null(), JsonValue::new_number(0.0.into()));
    /// assert_ne!(JsonValue::new_boolean(true), JsonValue::new_boolean(false));
    ///
    /// let mut array1 = Array::new();
    /// array1.push(JsonValue::new_null());
    ///
    /// let mut array2 = Array::new();
    /// array2.push(JsonValue::new_number(0.0.into()));
    /// assert_ne!(JsonValue::new_array(array1), JsonValue::new_array(array2));
    ///
    /// let mut object1 = Object::new();
    /// object1.insert(String::from("Key"), JsonValue::new_null());
    ///
    /// let mut object2 = Object::new();
    /// object2.insert(String::from("Key"), JsonValue::new_number(0.0.into()));
    /// assert_ne!(JsonValue::new_object(object1), JsonValue::new_object(object2));
    /// ```
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsonValue::Null, JsonValue::Null) => true,
            (JsonValue::Boolean(a), JsonValue::Boolean(b)) => a == b,
            (JsonValue::Number(a), JsonValue::Number(b)) => a == b,
            (JsonValue::String(a), JsonValue::String(b)) => a == b,
            (JsonValue::Array(a), JsonValue::Array(b)) => a == b,
            (JsonValue::Object(a), JsonValue::Object(b)) => a == b,
            _ => false,
        }
    }
}

impl<I: index::Index> core::ops::Index<I> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: I) -> &Self::Output {
        index.index_into(self)
    }
}

impl<I: index::Index> core::ops::IndexMut<I> for JsonValue {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_into_mut(self)
    }
}

impl From<&str> for JsonValue {
    /// Converts from &str to JsonValue.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::JsonValue;
    ///
    /// let value: JsonValue = "Hello World".into();
    /// ```
    fn from(t: &str) -> Self {
        #[cfg(feature = "c_adapter")]
        let result = Self::String(JsonString::new(t).unwrap());

        #[cfg(not(feature = "c_adapter"))]
        let result = Self::String(String::from(t));
        result
    }
}

impl From<JsonString> for JsonValue {
    /// Converts from String to JsonValue.
    ///
    /// # Examples
    /// ```not run
    /// use ylong_json::JsonValue;
    ///
    /// // Unused "c_adapter" feature.
    /// let value: JsonValue = String::from("Hello World").into();
    /// // Uses "c_adapter" feature.
    /// // let value: JsonValue = CString::new("Hello World").into();
    /// ```
    fn from(t: JsonString) -> Self {
        Self::String(t)
    }
}

macro_rules! json_value_from_type {
    ($type: tt, $func: expr) => {
        impl From<$type> for JsonValue {
            #[doc = concat!("从 ", stringify!($type), " 转换为 JsonValue。")]
            ///
            /// # Examples
            /// ```
            /// use ylong_json::*;
            ///
            #[doc = concat!("let value: JsonValue = ", stringify!($type), "::default().into();")]
            /// ```
            fn from(t: $type) -> Self {
                $func(t.into())
            }
        }

        impl From<&$type> for JsonValue {
            #[doc = concat!("从 &", stringify!($type), " 转换为 JsonValue。")]
            ///
            /// # Examples
            /// ```
            /// use ylong_json::*;
            ///
            #[doc = concat!("let value: JsonValue = ", stringify!($type), "::default().into();")]
            /// ```
            fn from(t: &$type) -> Self {
                $func(t.clone().into())
            }
        }

        impl From<&mut $type> for JsonValue {
            #[doc = concat!("从 &mut", stringify!($type), " 转换为 JsonValue。")]
            ///
            /// # Examples
            /// ```
            /// use ylong_json::*;
            ///
            #[doc = concat!("let value: JsonValue = ", stringify!($type), "::default().into();")]
            /// ```
            fn from(t: &mut $type) -> Self {
                $func(t.clone().into())
            }
        }
    };
}

macro_rules! number_value_from_type {
    ($($type: tt),* $(,)?) => {
        $(
            impl From<$type> for JsonValue {
                #[doc = concat!("从 ", stringify!($type), " 转换为 JsonValue。")]
                ///
                /// # Examples
                /// ```
                /// use ylong_json::JsonValue;
                ///
                /// // Due to the conversion to f64, there may be a loss of accuracy.
                #[doc = concat!("let value: JsonValue = ", stringify!($type), "::MAX.into();")]
                /// ```
                fn from(t: $type) -> Self {
                    Self::Number(Number::from(t))
                }
            }
        )*
    }
}

json_value_from_type!(bool, JsonValue::new_boolean);
json_value_from_type!(Array, JsonValue::new_array);
json_value_from_type!(Object, JsonValue::new_object);

number_value_from_type!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

#[cfg(test)]
mod ut_json_value {
    use super::{array::Array, object::Object, JsonValue};
    use std::io::{ErrorKind, Read, Result};
    use std::str::FromStr;

    /// UT test for `JsonValue::fmt`.
    ///
    /// # Title
    /// ut_json_value_fmt
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `Display::fmt` and `Debug::fmt`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_fmt() {
        let value = JsonValue::new_null();
        assert_eq!(format!("{value}"), "null");
        assert_eq!(format!("{value:?}"), "null");

        let value = JsonValue::new_boolean(false);
        assert_eq!(format!("{value}"), "false");
        assert_eq!(format!("{value:?}"), "false");

        let value = JsonValue::new_number(12.34.into());
        assert_eq!(format!("{value}"), "12.34");
        assert_eq!(format!("{value:?}"), "12.34");

        let value = JsonValue::new_string("Hello");
        assert_eq!(format!("{value}"), "\"Hello\"");
        assert_eq!(format!("{value:?}"), "\"Hello\"");

        let value = JsonValue::new_array(array!(false, JsonValue::Null, 12.34));
        assert_eq!(format!("{value}"), "[false,null,12.34]");
        assert_eq!(format!("{value}"), "[false,null,12.34]");

        let object = object!("null" => JsonValue::Null);
        let value = JsonValue::new_object(object);
        assert_eq!(format!("{value}"), "{\"null\":null}");
        assert_eq!(format!("{value}"), "{\"null\":null}");
    }

    /// UT test for `JsonValue::clone`.
    ///
    /// # Title
    /// ut_json_value_fmt
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::clone`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_clone() {
        let value1 = JsonValue::new_null();
        assert_eq!(value1, value1.clone());
    }

    /// UT test for `JsonValue::is_null`.
    ///
    /// # Title
    /// ut_json_value_is_null
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::is_null`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_is_null() {
        assert!(JsonValue::new_null().is_null());
        assert!(!JsonValue::new_boolean(true).is_null());
        assert!(!JsonValue::new_boolean(false).is_null());
        assert!(!JsonValue::new_number(12.34.into()).is_null());
        assert!(!JsonValue::new_string("hello").is_null());
        assert!(!JsonValue::new_array(Array::new()).is_null());
        assert!(!JsonValue::new_object(Object::new()).is_null());
    }

    /// UT test for `JsonValue::is_true`.
    ///
    /// # Title
    /// ut_json_value_is_true
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::is_true`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_is_true() {
        assert!(!JsonValue::new_null().is_true());
        assert!(JsonValue::new_boolean(true).is_true());
        assert!(!JsonValue::new_boolean(false).is_true());
        assert!(!JsonValue::new_number(12.34.into()).is_true());
        assert!(!JsonValue::new_string("hello").is_true());
        assert!(!JsonValue::new_array(Array::new()).is_true());
        assert!(!JsonValue::new_object(Object::new()).is_true());
    }

    /// UT test for `JsonValue::is_false`.
    ///
    /// # Title
    /// ut_json_value_is_false
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::is_false`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_is_false() {
        assert!(!JsonValue::new_null().is_false());
        assert!(!JsonValue::new_boolean(true).is_false());
        assert!(JsonValue::new_boolean(false).is_false());
        assert!(!JsonValue::new_number(12.34.into()).is_false());
        assert!(!JsonValue::new_string("hello").is_false());
        assert!(!JsonValue::new_array(Array::new()).is_false());
        assert!(!JsonValue::new_object(Object::new()).is_false());
    }

    /// UT test for `JsonValue::is_boolean`.
    ///
    /// # Title
    /// ut_json_value_is_false
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::is_boolean`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_is_boolean() {
        assert!(!JsonValue::new_null().is_boolean());
        assert!(JsonValue::new_boolean(true).is_boolean());
        assert!(JsonValue::new_boolean(false).is_boolean());
        assert!(!JsonValue::new_number(12.34.into()).is_boolean());
        assert!(!JsonValue::new_string("hello").is_boolean());
        assert!(!JsonValue::new_array(Array::new()).is_boolean());
        assert!(!JsonValue::new_object(Object::new()).is_boolean());
    }

    /// UT test for `JsonValue::is_number`.
    ///
    /// # Title
    /// ut_json_value_is_number
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::is_number`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_is_number() {
        assert!(!JsonValue::new_null().is_number());
        assert!(!JsonValue::new_boolean(true).is_number());
        assert!(!JsonValue::new_boolean(false).is_number());
        assert!(JsonValue::new_number(12.34.into()).is_number());
        assert!(!JsonValue::new_string("hello").is_number());
        assert!(!JsonValue::new_array(Array::new()).is_number());
        assert!(!JsonValue::new_object(Object::new()).is_number());
    }

    /// UT test for `JsonValue::is_string`.
    ///
    /// # Title
    /// ut_json_value_is_string
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::is_string`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_is_string() {
        assert!(!JsonValue::new_null().is_string());
        assert!(!JsonValue::new_boolean(true).is_string());
        assert!(!JsonValue::new_boolean(false).is_string());
        assert!(!JsonValue::new_number(12.34.into()).is_string());
        assert!(JsonValue::new_string("hello").is_string());
        assert!(!JsonValue::new_array(Array::new()).is_string());
        assert!(!JsonValue::new_object(Object::new()).is_string());
    }

    /// UT test for `JsonValue::is_array`.
    ///
    /// # Title
    /// ut_json_value_is_array
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::is_array`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_is_array() {
        assert!(!JsonValue::new_null().is_array());
        assert!(!JsonValue::new_boolean(true).is_array());
        assert!(!JsonValue::new_boolean(false).is_array());
        assert!(!JsonValue::new_number(12.34.into()).is_array());
        assert!(!JsonValue::new_string("hello").is_array());
        assert!(JsonValue::new_array(Array::new()).is_array());
        assert!(!JsonValue::new_object(Object::new()).is_array());
    }

    /// UT test for `JsonValue::is_object`.
    ///
    /// # Title
    /// ut_json_value_is_object
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::is_object`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_is_object() {
        assert!(!JsonValue::new_null().is_object());
        assert!(!JsonValue::new_boolean(true).is_object());
        assert!(!JsonValue::new_boolean(false).is_object());
        assert!(!JsonValue::new_number(12.34.into()).is_object());
        assert!(!JsonValue::new_string("hello").is_object());
        assert!(!JsonValue::new_array(Array::new()).is_object());
        assert!(JsonValue::new_object(Object::new()).is_object());
    }

    /// UT test for `JsonValue::try_as_boolean`.
    ///
    /// # Title
    /// ut_json_value_try_as_boolean
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_as_boolean`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_as_boolean() {
        assert!(JsonValue::new_null().try_as_boolean().is_err());
        assert!(JsonValue::new_boolean(true).try_as_boolean().is_ok());
        assert!(JsonValue::new_boolean(false).try_as_boolean().is_ok());
        assert!(JsonValue::new_number(12.34.into())
            .try_as_boolean()
            .is_err());
        assert!(JsonValue::new_string("hello").try_as_boolean().is_err());
        assert!(JsonValue::new_array(Array::new()).try_as_boolean().is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_as_boolean()
            .is_err());
    }

    /// UT test for `JsonValue::try_as_number`.
    ///
    /// # Title
    /// ut_json_value_try_as_number
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_as_number`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_as_number() {
        assert!(JsonValue::new_null().try_as_number().is_err());
        assert!(JsonValue::new_boolean(true).try_as_number().is_err());
        assert!(JsonValue::new_boolean(false).try_as_number().is_err());
        assert!(JsonValue::new_number(12.34.into()).try_as_number().is_ok());
        assert!(JsonValue::new_string("hello").try_as_number().is_err());
        assert!(JsonValue::new_array(Array::new()).try_as_number().is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_as_number()
            .is_err());
    }

    /// UT test for `JsonValue::try_as_string`.
    ///
    /// # Title
    /// ut_json_value_try_as_string
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_as_string`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_as_string() {
        assert!(JsonValue::new_null().try_as_string().is_err());
        assert!(JsonValue::new_boolean(true).try_as_string().is_err());
        assert!(JsonValue::new_boolean(false).try_as_string().is_err());
        assert!(JsonValue::new_number(12.34.into()).try_as_string().is_err());
        assert!(JsonValue::new_string("hello").try_as_string().is_ok());
        assert!(JsonValue::new_array(Array::new()).try_as_string().is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_as_string()
            .is_err());
    }

    /// UT test for `JsonValue::try_as_array`.
    ///
    /// # Title
    /// ut_json_value_try_as_array
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_as_array`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_as_array() {
        assert!(JsonValue::new_null().try_as_array().is_err());
        assert!(JsonValue::new_boolean(true).try_as_array().is_err());
        assert!(JsonValue::new_boolean(false).try_as_array().is_err());
        assert!(JsonValue::new_number(12.34.into()).try_as_array().is_err());
        assert!(JsonValue::new_string("hello").try_as_array().is_err());
        assert!(JsonValue::new_array(Array::new()).try_as_array().is_ok());
        assert!(JsonValue::new_object(Object::new()).try_as_array().is_err());
    }

    /// UT test for `JsonValue::try_as_object`.
    ///
    /// # Title
    /// ut_json_value_try_as_object
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_as_object`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_as_object() {
        assert!(JsonValue::new_null().try_as_object().is_err());
        assert!(JsonValue::new_boolean(true).try_as_object().is_err());
        assert!(JsonValue::new_boolean(false).try_as_object().is_err());
        assert!(JsonValue::new_number(12.34.into()).try_as_object().is_err());
        assert!(JsonValue::new_string("hello").try_as_object().is_err());
        assert!(JsonValue::new_array(Array::new()).try_as_object().is_err());
        assert!(JsonValue::new_object(Object::new()).try_as_object().is_ok());
    }

    /// UT test for `JsonValue::try_as_mut_boolean`.
    ///
    /// # Title
    /// ut_json_value_try_as_mut_boolean
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_as_mut_boolean`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_as_mut_boolean() {
        assert!(JsonValue::new_null().try_as_mut_boolean().is_err());
        assert!(JsonValue::new_boolean(true).try_as_mut_boolean().is_ok());
        assert!(JsonValue::new_boolean(false).try_as_mut_boolean().is_ok());
        assert!(JsonValue::new_number(12.34.into())
            .try_as_mut_boolean()
            .is_err());
        assert!(JsonValue::new_string("hello").try_as_mut_boolean().is_err());
        assert!(JsonValue::new_array(Array::new())
            .try_as_mut_boolean()
            .is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_as_mut_boolean()
            .is_err());
    }

    /// UT test for `JsonValue::try_as_mut_number`.
    ///
    /// # Title
    /// ut_json_value_try_as_mut_number
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_as_mut_number`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_as_mut_number() {
        assert!(JsonValue::new_null().try_as_mut_number().is_err());
        assert!(JsonValue::new_boolean(true).try_as_mut_number().is_err());
        assert!(JsonValue::new_boolean(false).try_as_mut_number().is_err());
        assert!(JsonValue::new_number(12.34.into())
            .try_as_mut_number()
            .is_ok());
        assert!(JsonValue::new_string("hello").try_as_mut_number().is_err());
        assert!(JsonValue::new_array(Array::new())
            .try_as_mut_number()
            .is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_as_mut_number()
            .is_err());
    }

    /// UT test for `JsonValue::try_as_mut_string`.
    ///
    /// # Title
    /// ut_json_value_try_as_mut_string
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_as_mut_string`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_as_mut_string() {
        assert!(JsonValue::new_null().try_as_mut_string().is_err());
        assert!(JsonValue::new_boolean(true).try_as_mut_string().is_err());
        assert!(JsonValue::new_boolean(false).try_as_mut_string().is_err());
        assert!(JsonValue::new_number(12.34.into())
            .try_as_mut_string()
            .is_err());
        assert!(JsonValue::new_string("hello").try_as_mut_string().is_ok());
        assert!(JsonValue::new_array(Array::new())
            .try_as_mut_string()
            .is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_as_mut_string()
            .is_err());
    }

    /// UT test for `JsonValue::try_as_mut_array`.
    ///
    /// # Title
    /// ut_json_value_try_as_mut_array
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_as_mut_array`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_as_mut_array() {
        assert!(JsonValue::new_null().try_as_mut_array().is_err());
        assert!(JsonValue::new_boolean(true).try_as_mut_array().is_err());
        assert!(JsonValue::new_boolean(false).try_as_mut_array().is_err());
        assert!(JsonValue::new_number(12.34.into())
            .try_as_mut_array()
            .is_err());
        assert!(JsonValue::new_string("hello").try_as_mut_array().is_err());
        assert!(JsonValue::new_array(Array::new())
            .try_as_mut_array()
            .is_ok());
        assert!(JsonValue::new_object(Object::new())
            .try_as_mut_array()
            .is_err());
    }

    /// UT test for `JsonValue::try_as_mut_object`.
    ///
    /// # Title
    /// ut_json_value_try_as_mut_object
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_as_mut_object`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_as_mut_object() {
        assert!(JsonValue::new_null().try_as_mut_object().is_err());
        assert!(JsonValue::new_boolean(true).try_as_mut_object().is_err());
        assert!(JsonValue::new_boolean(false).try_as_mut_object().is_err());
        assert!(JsonValue::new_number(12.34.into())
            .try_as_mut_object()
            .is_err());
        assert!(JsonValue::new_string("hello").try_as_mut_object().is_err());
        assert!(JsonValue::new_array(Array::new())
            .try_as_mut_object()
            .is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_as_mut_object()
            .is_ok());
    }

    /// UT test for `JsonValue::try_into_boolean`.
    ///
    /// # Title
    /// ut_json_value_try_into_boolean
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_into_boolean`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_into_boolean() {
        assert!(JsonValue::new_null().try_into_boolean().is_err());
        assert!(JsonValue::new_boolean(true).try_into_boolean().is_ok());
        assert!(JsonValue::new_boolean(false).try_into_boolean().is_ok());
        assert!(JsonValue::new_number(12.34.into())
            .try_into_boolean()
            .is_err());
        assert!(JsonValue::new_string("hello").try_into_boolean().is_err());
        assert!(JsonValue::new_array(Array::new())
            .try_into_boolean()
            .is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_into_boolean()
            .is_err());
    }

    /// UT test for `JsonValue::try_into_number`.
    ///
    /// # Title
    /// ut_json_value_try_into_number
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_into_number`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_into_number() {
        assert!(JsonValue::new_null().try_into_number().is_err());
        assert!(JsonValue::new_boolean(true).try_into_number().is_err());
        assert!(JsonValue::new_boolean(false).try_into_number().is_err());
        assert!(JsonValue::new_number(12.34.into())
            .try_into_number()
            .is_ok());
        assert!(JsonValue::new_string("hello").try_into_number().is_err());
        assert!(JsonValue::new_array(Array::new())
            .try_into_number()
            .is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_into_number()
            .is_err());
    }

    /// UT test for `JsonValue::try_into_string`.
    ///
    /// # Title
    /// ut_json_value_try_into_string
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_into_string`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_into_string() {
        assert!(JsonValue::new_null().try_into_string().is_err());
        assert!(JsonValue::new_boolean(true).try_into_string().is_err());
        assert!(JsonValue::new_boolean(false).try_into_string().is_err());
        assert!(JsonValue::new_number(12.34.into())
            .try_into_string()
            .is_err());
        assert!(JsonValue::new_string("hello").try_into_string().is_ok());
        assert!(JsonValue::new_array(Array::new())
            .try_into_string()
            .is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_into_string()
            .is_err());
    }

    /// UT test for `JsonValue::try_into_array`.
    ///
    /// # Title
    /// ut_json_value_try_into_array
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_into_array`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_into_array() {
        assert!(JsonValue::new_null().try_into_array().is_err());
        assert!(JsonValue::new_boolean(true).try_into_array().is_err());
        assert!(JsonValue::new_boolean(false).try_into_array().is_err());
        assert!(JsonValue::new_number(12.34.into())
            .try_into_array()
            .is_err());
        assert!(JsonValue::new_string("hello").try_into_array().is_err());
        assert!(JsonValue::new_array(Array::new()).try_into_array().is_ok());
        assert!(JsonValue::new_object(Object::new())
            .try_into_array()
            .is_err());
    }

    /// UT test for `JsonValue::try_into_object`.
    ///
    /// # Title
    /// ut_json_value_try_into_object
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::try_into_object`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_try_into_object() {
        assert!(JsonValue::new_null().try_into_object().is_err());
        assert!(JsonValue::new_boolean(true).try_into_object().is_err());
        assert!(JsonValue::new_boolean(false).try_into_object().is_err());
        assert!(JsonValue::new_number(12.34.into())
            .try_into_object()
            .is_err());
        assert!(JsonValue::new_string("hello").try_into_object().is_err());
        assert!(JsonValue::new_array(Array::new())
            .try_into_object()
            .is_err());
        assert!(JsonValue::new_object(Object::new())
            .try_into_object()
            .is_ok());
    }

    /// UT test for `JsonValue::to_formatted_string`.
    ///
    /// # Title
    /// ut_json_value_to_formatted_string
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::to_formatted_string`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_to_formatted_string() {
        assert_eq!(
            JsonValue::new_null().to_formatted_string().unwrap(),
            "null\n"
        );
    }

    /// UT test for `JsonValue::to_compact_string`.
    ///
    /// # Title
    /// ut_json_value_to_compact_string
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::to_compact_string`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_to_compact_string() {
        assert_eq!(JsonValue::new_null().to_compact_string().unwrap(), "null");
    }

    /// UT test for `JsonValue::from_str`.
    ///
    /// # Title
    /// ut_json_value_from_str
    ///
    /// # Brief
    /// 1. Calls `JsonValue::from_str` to create a `JsonValue`.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_json_value_from_str() {
        assert_eq!(JsonValue::from_str("null").unwrap(), JsonValue::new_null());
    }

    /// UT test for `JsonValue::eq`.
    ///
    /// # Title
    /// ut_json_value_eq
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`s.
    /// 2. Calls `JsonValue::eq`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_eq() {
        assert_eq!(JsonValue::new_null(), JsonValue::new_null());
        assert_eq!(JsonValue::new_boolean(true), JsonValue::new_boolean(true));
        assert_eq!(
            JsonValue::new_number(1.into()),
            JsonValue::new_number(1.into())
        );
        assert_eq!(
            JsonValue::new_string("string"),
            JsonValue::new_string("string")
        );
        assert_eq!(
            JsonValue::new_array(Array::new()),
            JsonValue::new_array(Array::new())
        );
        assert_eq!(
            JsonValue::new_object(Object::new()),
            JsonValue::new_object(Object::new())
        );
        assert_ne!(JsonValue::new_null(), JsonValue::new_boolean(true));
    }

    /// UT test for `JsonValue::from`.
    ///
    /// # Title
    /// ut_json_value_from
    ///
    /// # Brief
    /// 1. Calls `JsonValue::from` to create `JsonValue`s.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_json_value_from() {
        assert_eq!(JsonValue::from(true), JsonValue::new_boolean(true));
        assert_eq!(JsonValue::from(false), JsonValue::new_boolean(false));
        assert_eq!(
            JsonValue::from(Array::new()),
            JsonValue::new_array(Array::new())
        );
        assert_eq!(
            JsonValue::from(Object::new()),
            JsonValue::new_object(Object::new())
        );

        assert_eq!(JsonValue::from(&true), JsonValue::new_boolean(true));
        assert_eq!(JsonValue::from(&false), JsonValue::new_boolean(false));
        assert_eq!(
            JsonValue::from(&Array::new()),
            JsonValue::new_array(Array::new())
        );
        assert_eq!(
            JsonValue::from(&Object::new()),
            JsonValue::new_object(Object::new())
        );

        assert_eq!(JsonValue::from(&mut true), JsonValue::new_boolean(true));
        assert_eq!(JsonValue::from(&mut false), JsonValue::new_boolean(false));
        assert_eq!(
            JsonValue::from(&mut Array::new()),
            JsonValue::new_array(Array::new())
        );
        assert_eq!(
            JsonValue::from(&mut Object::new()),
            JsonValue::new_object(Object::new())
        );

        #[cfg(not(feature = "c_adapter"))]
        assert_eq!(JsonValue::from(String::new()), JsonValue::new_string(""));

        #[cfg(feature = "c_adapter")]
        {
            use std::ffi::CString;
            assert_eq!(
                JsonValue::from(CString::new("").unwrap()),
                JsonValue::new_string("")
            );
        }
    }

    /// UT test for `JsonValue::remove`.
    ///
    /// # Title
    /// ut_json_value_remove
    ///
    /// # Brief
    /// 1. Creates some `JsonValue`.
    /// 2. Calls `JsonValue::remove` on them.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_json_value_remove() {
        let mut object = JsonValue::new_object(
            object!("key1" => "value1"; "key2" => "value2"; "key3" => "value3"),
        );
        assert_eq!(object["key1"], JsonValue::new_string("value1"));
        assert_eq!(object["key2"], JsonValue::new_string("value2"));
        assert_eq!(object["key3"], JsonValue::new_string("value3"));

        object.remove("key2");
        assert_eq!(object["key1"], JsonValue::new_string("value1"));
        assert_eq!(object["key2"], JsonValue::new_null());
        assert_eq!(object["key3"], JsonValue::new_string("value3"));

        let mut array = JsonValue::new_array(array!(false, JsonValue::new_null(), 12.34));
        assert_eq!(array[0], JsonValue::new_boolean(false));
        assert_eq!(array[1], JsonValue::new_null());
        assert_eq!(array[2], JsonValue::new_number(12.34.into()));

        array.remove(1);
        assert_eq!(array[0], JsonValue::new_boolean(false));
        assert_eq!(array[1], JsonValue::new_number(12.34.into()));
        assert_eq!(array[2], JsonValue::new_null());
    }

    /// UT test for `JsonValue::from_reader`.
    ///
    /// # Title
    /// ut_json_value_from_reader
    ///
    /// # Brief
    /// 1. Calls `JsonValue::from_reader` to create some `JsonValue`.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_json_value_from_reader() {
        struct TestErrorIo;

        impl Read for TestErrorIo {
            fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
                Err(ErrorKind::AddrInUse.into())
            }
        }

        assert!(JsonValue::from_reader(TestErrorIo).is_err());
    }
}
