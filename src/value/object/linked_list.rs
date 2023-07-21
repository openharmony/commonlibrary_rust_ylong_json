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

use crate::{Cursor, CursorMut, Iter, IterMut, JsonValue, LinkedList, Node};
use core::fmt::{Debug, Display, Formatter};
use core::ptr::null;

/// Object type, implemented using LinkedList.
///
/// # Situation
/// * When the average number of items under Object is less than 15 (estimated value).
///
/// * When the average number of items under Object exceeds 15 (estimated), but do not or rarely made the query operation.
///
/// # Attention
/// * Only opening the 'list_object' feature, this Object type can be used , and it conflicts with other objects.
///
/// * This Object ** does not provide the ** de-duplicate function.
/// * Users are required to ensure that there are no duplicate entries.
///
/// * The output order of this Object is the same as the insertion order.
///
/// # Examples
/// ```
/// use ylong_json::Object;
///
/// let object = Object::new();
/// ```
#[derive(Default, Clone, PartialEq)]
pub struct Object {
    inner: LinkedList<(String, JsonValue)>,
}

impl Object {
    /// Creates an empty Object。
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
            inner: LinkedList::new(),
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
        self.get_cursor(key).is_some()
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
        self.inner.push_back((key, value))
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
        self.get_cursor_mut(key)?.remove_current().map(|(_, v)| v)
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
        self.get_cursor(key)?.current().map(|(_, v)| v)
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
        // Using get_cursor_mut causes a problem referencing temporary variables.
        self.get_node_mut(key).map(|n| &mut n.get_element_mut().1)
    }

    /// Gets a common reference to the node in Object with the specified key.
    /// If there is an element with the same name, returns the one with the smallest subscript.
    ///
    /// After getting a common reference to a node, the node cannot be released. Otherwise, undefined behavior occurs.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object};
    ///
    /// let mut object = Object::new();
    /// assert_eq!(object.get_node("no_such_key").is_none(), true);
    ///
    /// object.insert(String::from("null"), JsonValue::Null);
    /// assert_eq!(object.get_node("null").is_some(), true);
    /// ```
    pub fn get_node(&self, key: &str) -> Option<&Node<(String, JsonValue)>> {
        self.get_cursor(key)?.current_node()
    }

    /// Gets a mutable reference to the node in Object with the specified key.
    /// If there is an element with the same name, returns the one with the smallest subscript.
    ///
    /// After getting a mutable reference to a node, the node cannot be released. Otherwise, undefined behavior occurs.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object};
    ///
    /// let mut object = Object::new();
    /// assert_eq!(object.get_node_mut("no_such_key").is_none(), true);
    ///
    /// object.insert(String::from("null"), JsonValue::Null);
    /// assert_eq!(object.get_node_mut("null").is_some(), true);
    /// ```
    pub fn get_node_mut(&mut self, key: &str) -> Option<&mut Node<(String, JsonValue)>> {
        self.get_cursor_mut(key)?.current_node()
    }

    /// Gets the last node.
    #[cfg(feature = "c_adapter")]
    pub(crate) fn last_node_mut(&mut self) -> Option<&mut Node<(String, JsonValue)>> {
        let mut cursor = self.inner.cursor_back_mut();
        let _ = cursor.index()?;
        cursor.current_node()
    }

    /// Needs using this method to avoid the life cycle check, which involves unsafe operations.
    pub(crate) fn get_key_mut_maybe_insert(&mut self, key: &str) -> &mut JsonValue {
        let mut cursor = self.inner.cursor_front();
        let mut ptr = null();
        while cursor.index().is_some() {
            let current = cursor.current().unwrap();
            if current.0 == key {
                ptr = cursor.current_node_ptr();
                break;
            }
            cursor.move_next();
        }

        if ptr.is_null() {
            self.insert(String::from(key), JsonValue::Null);
            &mut self.inner.back_mut().unwrap().1
        } else {
            unsafe {
                &mut (*(ptr as *mut Node<(String, JsonValue)>))
                    .get_element_mut()
                    .1
            }
        }
    }

    /// Gets the common cursor of the node corresponding to the specified key.
    fn get_cursor(&self, key: &str) -> Option<Cursor<'_, (String, JsonValue)>> {
        let mut cursor = self.inner.cursor_front();
        while cursor.index().is_some() {
            let (k, _) = cursor.current().unwrap();
            if key == k {
                return Some(cursor);
            }
            cursor.move_next();
        }
        None
    }

    /// Gets the mutable cursor of the node corresponding to the specified key.
    fn get_cursor_mut(&mut self, key: &str) -> Option<CursorMut<'_, (String, JsonValue)>> {
        let mut cursor = self.inner.cursor_front_mut();
        while cursor.index().is_some() {
            let (k, _) = cursor.current().unwrap();
            if key == k {
                return Some(cursor);
            }
            cursor.move_next();
        }
        None
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
mod ut_linked_list {
    use crate::{JsonValue, Object};

    /// UT test for `Object::contains_key`.
    ///
    /// # Title
    /// ut_object_contains_key
    ///
    /// # Brief
    /// 1. Creates a `Object`.
    /// 2. Calls `Object::contains_key` on it.
    /// 3. Checks  if the test results are correct.
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
    /// 3. Checks  if the test results are correct.
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
    /// 3. Checks  if the test results are correct.
    #[test]
    fn ut_object_get_mut() {
        let mut object = object!("key1" => "value1");
        assert_eq!(
            object.get_mut("key1"),
            Some(&mut JsonValue::new_string("value1"))
        );
        assert_eq!(object.get_mut("key2"), None);
    }

    /// UT test for `Object::get_node`.
    ///
    /// # Title
    /// ut_object_get_node
    ///
    /// # Brief
    /// 1. Creates a `Object`.
    /// 2. Calls `Object::get_node` on it.
    /// 3. Checks  if the test results are correct.
    #[test]
    fn ut_object_get_node() {
        let object = object!("key1" => "value1");
        assert!(object.get_node("key1").is_some());
        assert!(object.get_node("key2").is_none());
    }

    /// UT test for `Object::get_node_mut`.
    ///
    /// # Title
    /// ut_object_get_node_mut
    ///
    /// # Brief
    /// 1. Creates a `Object`.
    /// 2. Calls `Object::get_node_mut` on it.
    /// 3. Checks  if the test results are correct.
    #[test]
    fn ut_object_get_node_mut() {
        let mut object = object!("key1" => "value1");
        assert!(object.get_node_mut("key1").is_some());
        assert!(object.get_node_mut("key2").is_none());
    }

    /// UT test for `Object::fmt`.
    ///
    /// # Title
    /// ut_object_fmt
    ///
    /// # Brief
    /// 1. Creates a `Object`.
    /// 2. Calls `Object::fmt` on it.
    /// 3. Checks  if the test results are correct.
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
}
