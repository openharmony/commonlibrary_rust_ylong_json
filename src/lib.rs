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

//! ylong_json is a library for parsing and serializing JSON in Rust.
//! This library provides a convenient macro-based API for creating
//! JSON values, and utilities for reading and writing JSON data
//! from various sources.

// TODO: 1) Isolates ylong_json no_std.
// TODO: 2) Handles illegal Utf-8 bytes.
// TODO: 3) Refactors ylong_json.
// TODO: 4) JsonValue provides 'Contains' methods。

/// Creates an array with at least one but any number of elements.
#[macro_export]
macro_rules! array {
    () => ({
        Array::new()
    });
    ($($x:expr),+ $(,)?) => ({
        let mut array = Array::new();
        $(
            array.push($x.into());
        )*
        array
    });
}

/// Creates an object with at least one but any number of key-value pairs.
#[macro_export]
macro_rules! object {
    () => ({
        Object::new()
    });
    ($($k: expr => $v: expr);+ $(;)?) => ({
        let mut object = Object::new();
        $(
           object.insert(String::from($k), $v.into());
        )*
        object
    });
}

mod consts;
mod encoder;
mod error;
mod reader;
#[macro_use]
mod states;
mod value;

pub use error::{Error, ParseError};
pub use value::{Array, Index, JsonValue, Number, Object};

pub(crate) use encoder::{CompactEncoder, FormattedEncoder};
pub(crate) use states::start_parsing;

#[cfg(feature = "c_adapter")]
mod adapter;
#[cfg(feature = "c_adapter")]
pub use adapter::*;

mod deserializer;
#[cfg(any(feature = "list_array", feature = "list_object"))]
mod linked_list;
mod serializer_compact;

#[cfg(any(feature = "list_array", feature = "list_object"))]
pub(crate) use linked_list::{Cursor, CursorMut, LinkedList};
#[cfg(any(feature = "list_array", feature = "list_object"))]
pub use linked_list::{Iter, IterMut, Node};

pub use deserializer::{from_reader, from_slice, from_str};
pub use serializer_compact::{to_string, to_writer};
