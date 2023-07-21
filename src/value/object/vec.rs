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
use core::slice::{Iter, IterMut};

/// Object type, implemented using Vec.
///
/// # Situation
/// 1. When the average number of entries x under Object is about 15 <= x <= 100.
///
/// 2. When the average number of Object entries x is about 101 <= x <= 1024, and the creation to
/// query ratio (the average number of queries created once) < 600.
///
/// 3. When the average number of objects x is about 1025 <= x <= 10000, and the creation to
/// query ratio (the average number of queries created once) < 500.
///
/// # Attention
/// * Only opening the 'vec_object' feature, this Object type can be used , and it conflicts with other Objects.
///
/// * This Object ** does not provide the ** de-duplicate function.
/// * Users are required to ensure that there are no duplicate entries.
///
/// * The output order of this Object is the same as the insertion order.
/// # Examples
/// ```
/// use ylong_json::Object;
///
/// let object = Object::new();
/// assert_eq!(object.is_empty(), true);
/// ```
#[derive(Default, Clone)]
pub struct Object {
    inner: Vec<(String, JsonValue)>,
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
        Self { inner: Vec::new() }
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
        self.inner.iter().any(|(k, _)| k == key)
    }

    /// Inserts the specified key and value into an Object, appending them to the end without deduplication.
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
        self.inner.push((key, value))
    }

    /// Removes the element under the specified key from the Object.If there is an element with
    /// the same name in the Object, deletes the one with the smallest subscript.
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
        let pos = self.inner.iter().position(|(k, _)| k == key)?;
        Some(self.inner.remove(pos).1)
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
    pub fn iter(&self) -> Iter<'_, (String, JsonValue)> {
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
    pub fn iter_mut(&mut self) -> IterMut<'_, (String, JsonValue)> {
        self.inner.iter_mut()
    }

    /// Gets a common reference to the element in Object with the specified key.
    /// If there is an element with the same name, returns the one with the smallest subscript.
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
        self.inner.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Gets a mutable reference to the element in Object with the specified key.
    /// If there is an element with the same name, returns the one with the smallest subscript.
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
        self.inner
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// Gets a mutable reference to the last element.
    pub(crate) fn last_mut(&mut self) -> Option<&mut JsonValue> {
        self.inner.last_mut().map(|(_, v)| v)
    }

    pub(crate) fn get_mut_by_position(&mut self, index: usize) -> Option<&mut JsonValue> {
        self.inner.get_mut(index).map(|(_, v)| v)
    }
}

impl PartialEq for Object {
    /// Determines whether two objects are equal.
    ///
    /// The condition for two objects to be equal is that the two objects are of equal length
    /// and the key-value pair can be one-to-one and exactly equal.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Object, JsonValue};
    ///
    /// let object1 = Object::new();
    /// let object2 = Object::new();
    /// let mut object3 = Object::new();
    /// object3.insert("test".to_string(), JsonValue::Null);
    ///
    /// assert_eq!(object1, object2);
    /// assert_ne!(object1, object3);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (k, v) in self.iter() {
            if other.get(k) != Some(v) {
                return false;
            }
        }
        true
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
mod ut_vec {
    use crate::{JsonValue, Object};

    /// UT test for `Object::contains_key`.
    ///
    /// # Title
    /// ut_object_contains_key
    ///
    /// # Brief
    /// 1. Creates a `Object`.
    /// 2. Calls `Object::contains_key` on it.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_object_contains_key() {
        let object = object!("key1" => "value1");
        assert!(object.contains_key("key1"));
        assert!(!object.contains_key("key2"));
    }

    /// UT test for `Object::iter_mut`.
    ///
    /// # Title
    /// ut_object_iter_mut
    ///
    /// # Brief
    /// 1. Creates a `Object`.
    /// 2. Calls `Object::iter_mut` on it.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_object_iter_mut() {
        let mut object = object!("key1" => "value1");
        let mut iter_mut = object.iter_mut();
        assert_eq!(
            iter_mut.next(),
            Some(&mut (String::from("key1"), JsonValue::new_string("value1")))
        );
        assert_eq!(iter_mut.next(), None);
    }

    /// UT test for `Object::get_mut`.
    ///
    /// # Title
    /// ut_object_get_mut
    ///
    /// # Brief
    /// 1. Creates a `Object`.
    /// 2. Calls `Object::get_mut` on it.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_object_get_mut() {
        let mut object = object!("key1" => "value1");
        assert_eq!(
            object.get_mut("key1"),
            Some(&mut JsonValue::new_string("value1"))
        );
        assert_eq!(object.get_mut("key2"), None);
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
