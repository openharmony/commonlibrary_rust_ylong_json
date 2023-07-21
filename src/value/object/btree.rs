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

use crate::JsonValue;
use core::fmt::{Debug, Display, Formatter};
use std::collections::btree_map::{BTreeMap, Iter, IterMut};

/// Object type, implemented using the standard library Btree.
///
/// # Situation
/// * When the average number of objects exceeds 1024 (estimated value) but does not exceed 5000 (estimated value),
/// and the creation and query ratio is greater than 600.(Number of queries for 1 Object creation).
///
/// * When the average number of objects exceeds 5000 (estimated value).
///
/// # Attention
/// Only opening ` btree_object ` feature can be used, and associated with the object of other feature conflict. (Enabled by default)
///
/// # Examples
/// ```
/// use ylong_json::Object;
///
/// let object = Object::new();
/// assert_eq!(object.is_empty(), true);
/// ```
#[derive(Default, Clone, PartialEq)]
pub struct Object {
    inner: BTreeMap<String, JsonValue>,
}

impl Object {
    /// Creates an empty Object.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Object;
    ///
    /// let object = Object::new();
    /// assert_eq!(object.is_empty(), true);
    /// ```
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    /// Gets the length of Object.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object};
    ///
    /// let mut object = Object::new();
    /// assert_eq!(object.len(), 0);
    /// object.insert(String::from("null"), JsonValue::Null);
    /// assert_eq!(object.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Determines whether the Object is empty.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object};
    ///
    /// let mut object = Object::new();
    /// assert_eq!(object.is_empty(), true);
    /// object.insert(String::from("null"), JsonValue::Null);
    /// assert_eq!(object.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Checks whether the specified key exists in the Object.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object, Number};
    ///
    /// let mut object = Object::new();
    /// object.insert(String::from("null"), JsonValue::Null);
    ///
    /// assert_eq!(object.contains_key("null"), true);
    /// assert_eq!(object.contains_key("no_such_key"), false);
    /// ```
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Inserts the specified key and value into the Object, and replaces the value if the key already exists in the Object.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object};
    ///
    /// let mut object = Object::new();
    /// assert_eq!(object.len(), 0);
    /// object.insert(String::from("null"), JsonValue::Null);
    /// assert_eq!(object.len(), 1);
    /// ```
    pub fn insert(&mut self, key: String, value: JsonValue) {
        self.inner.insert(key, value);
    }

    /// Removes the element under the specified Key from Object.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object, Number};
    ///
    /// let mut object = Object::new();
    /// object.insert(String::from("null"), JsonValue::Null);
    /// assert_eq!(object.len(), 1);
    /// assert_eq!(object.remove("null"), Some(JsonValue::Null));
    /// assert_eq!(object.len(), 0);
    /// ```
    pub fn remove(&mut self, key: &str) -> Option<JsonValue> {
        self.inner.remove(key)
    }

    /// Gets a common iterator of Object.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Object;
    ///
    /// let object = Object::new();
    /// let iter = object.iter();
    /// ```
    pub fn iter(&self) -> Iter<'_, String, JsonValue> {
        self.inner.iter()
    }

    /// Gets a mutable iterator of Object.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Object;
    ///
    /// let mut object = Object::new();
    /// let iter_mut = object.iter_mut();
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, String, JsonValue> {
        self.inner.iter_mut()
    }

    /// Gets a common reference to the element in Object with the specified key.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object, Number};
    ///
    /// let mut object = Object::new();
    /// object.insert(String::from("test"), JsonValue::Number(Number::from(123)));
    ///
    /// assert_eq!(object.get("test"), Some(&JsonValue::Number(Number::from(123))));
    /// assert_eq!(object.get("no_such_key"), None);
    /// ```
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.inner.get(key)
    }

    /// Gets a mutable reference to the element in Object with the specified key.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object};
    ///
    /// let mut object = Object::new();
    /// object.insert(String::from("null"), JsonValue::Null);
    ///
    /// assert_eq!(object.get_mut("null"), Some(&mut JsonValue::Null));
    /// assert_eq!(object.get_mut("no_such_key"), None);
    /// ```
    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
        self.inner.get_mut(key)
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{{")?;
        for (n, (key, value)) in self.inner.iter().enumerate() {
            if n != 0 {
                write!(f, ",")?;
            }
            write!(f, "\"{key}\":{value}")?;
        }
        write!(f, "}}")
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod ut_btree {
    use crate::{JsonValue, Object};

    /// UT test for `Object::iter_mut`.
    ///
    /// # Title
    /// ut_object_iter_mut
    ///
    /// # Brief
    /// 1. Creates some `Object`s.
    /// 2. Calls `Object::iter_mut`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_object_iter_mut() {
        let mut object = object!("key1" => "value1");
        let mut iter = object.iter_mut();
        assert_eq!(
            iter.next(),
            Some((&String::from("key1"), &mut JsonValue::new_string("value1")))
        );
        assert_eq!(iter.next(), None);
    }

    /// UT test for `Object::fmt`.
    ///
    /// # Title
    /// ut_object_fmt
    ///
    /// # Brief
    /// 1. Creates a `Object`.
    /// 2. Calls `Object::fmt` on it.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_object_fmt() {
        let object = object!("key1" => "value1"; "key2" => "value2");
        assert_eq!(
            format!("{object}"),
            "{\"key1\":\"value1\",\"key2\":\"value2\"}"
        );
        assert_eq!(
            format!("{object:?}"),
            "{\"key1\":\"value1\",\"key2\":\"value2\"}"
        );
    }

    /// UT test for `Object::eq`.
    ///
    /// # Title
    /// ut_object_fmt
    ///
    /// # Brief
    /// 1. Creates a `Object`.
    /// 2. Calls `Object::eq` on it.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_object_eq() {
        let object1 = object!("key1" => "value1");
        let object2 = object!("key1" => "value1"; "key2" => "value2");
        let object3 = object!("key1" => "value1"; "key3" => "value3");

        assert_eq!(object1, object1);
        assert_ne!(object1, object2);
        assert_ne!(object2, object3);
    }
}
