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

/// Array type, implemented using LinkedList.
///
/// # Situation
/// * When the average number of Array entries does not exceed 15 (estimated value).
///
/// * When the average number of Array entries exceeds 15 (estimated value), but do not or rarely made query operation.
///
/// # Attention
/// * Only open `list_array` feature can be used, and conflicts with other array-related features.
///
/// # Examples
/// ```
/// use ylong_json::Array;
///
/// let array = Array::new();
/// ```
#[derive(Default, Clone, PartialEq)]
pub struct Array {
    inner: LinkedList<JsonValue>,
}

impl Array {
    /// Creates an empty Array instance.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Array;
    ///
    /// let array = Array::new();
    /// ```
    pub fn new() -> Self {
        Self {
            inner: LinkedList::new(),
        }
    }

    /// Gets length of Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// assert_eq!(array.len(), 0);
    ///
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Determines whether Array is empty.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// assert_eq!(array.is_empty(), true);
    ///
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Insert a new JsonValue at the end of Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// assert_eq!(array.len(), 0);
    ///
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.len(), 1);
    /// ```
    pub fn push(&mut self, value: JsonValue) {
        self.inner.push_back(value);
    }

    /// Pops the element at the end of Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// assert_eq!(array.pop(), None);
    ///
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.pop(), Some(JsonValue::Null));
    /// ```
    pub fn pop(&mut self) -> Option<JsonValue> {
        self.inner.pop_back()
    }

    /// Gets a common iterator of Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Array;
    ///
    /// let array = Array::new();
    /// let iter = array.iter();
    /// ```
    pub fn iter(&self) -> Iter<'_, JsonValue> {
        self.inner.iter()
    }

    /// Gets a mutable iterator of Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Array;
    ///
    /// let mut array = Array::new();
    /// let iter_mut = array.iter_mut();
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, JsonValue> {
        self.inner.iter_mut()
    }

    /// Returns a common reference to the specified index ** member ** in Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.get(0), Some(&JsonValue::Null));
    /// assert_eq!(array.get(1), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&JsonValue> {
        self.get_cursor(index)?.current()
    }

    /// Returns a mutable reference to the specified index ** member ** in Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.get_mut(0), Some(&mut JsonValue::Null));
    /// assert_eq!(array.get_mut(1), None);
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut JsonValue> {
        // Using 'get_cursor_mut' causes a problem referencing temporary variables.
        self.get_node_mut(index).map(|n| n.get_element_mut())
    }

    /// Returns a common reference to the trailing ** member ** in Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// assert_eq!(array.last(), None);
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.last(), Some(&JsonValue::Null));
    /// ```
    pub fn last(&self) -> Option<&JsonValue> {
        self.inner.back()
    }

    /// Returns a mutable reference to the trailing ** member ** in Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// assert_eq!(array.last_mut(), None);
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.last_mut(), Some(&mut JsonValue::Null));
    /// ```
    pub fn last_mut(&mut self) -> Option<&mut JsonValue> {
        self.inner.back_mut()
    }

    /// Removes the node in Array with the specified index.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// array.push(JsonValue::Null);
    /// array.push(JsonValue::Boolean(true));
    /// array.push(JsonValue::Null);
    ///
    /// assert_eq!(array.len(), 3);
    /// let second = array.remove(1);
    /// assert_eq!(second, Some(JsonValue::Boolean(true)));
    /// assert_eq!(array.len(), 2);
    /// ```
    pub fn remove(&mut self, index: usize) -> Option<JsonValue> {
        self.get_cursor_mut(index)?.remove_current()
    }

    /// Returns a common reference to the specified index ** node ** in Array.
    ///
    /// After getting a common reference to a node, the corresponding node cannot be released.
    /// Otherwise undefined behavior will occur.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.get_node(0).is_some(), true);
    /// assert_eq!(array.get_node(1).is_none(), true);
    /// ```
    pub fn get_node(&self, index: usize) -> Option<&Node<JsonValue>> {
        self.get_cursor(index)?.current_node()
    }

    /// Returns a mutable reference to the specified index ** node ** in Array.
    ///
    /// After getting a mutable reference to a node, the corresponding node cannot be released.
    /// Otherwise undefined behavior will occur.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// let value = JsonValue::Null;
    /// array.push(value);
    /// assert_eq!(array.get_node_mut(0).is_some(), true);
    /// assert_eq!(array.get_node_mut(1).is_none(), true);
    /// ```
    pub fn get_node_mut(&mut self, index: usize) -> Option<&mut Node<JsonValue>> {
        self.get_cursor_mut(index)?.current_node()
    }

    /// Returns a common reference to the trailing ** node ** in Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// assert_eq!(array.last_node().is_none(), true);
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.last_node().is_some(), true);
    /// ```
    pub fn last_node(&self) -> Option<&Node<JsonValue>> {
        self.inner.back_node()
    }

    /// Returns a mutable reference to the trailing ** node ** in Array.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let mut array = Array::new();
    /// assert_eq!(array.last_node_mut().is_none(), true);
    /// array.push(JsonValue::Null);
    /// assert_eq!(array.last_node_mut().is_some(), true);
    /// ```
    pub fn last_node_mut(&mut self) -> Option<&mut Node<JsonValue>> {
        self.inner.back_node_mut()
    }

    /// Gets the common cursor of the specified index node.
    fn get_cursor(&self, index: usize) -> Option<Cursor<'_, JsonValue>> {
        let len = self.len();
        // If index is greater than the array length, returns.
        // If index is less than half the array length, searches from front to back;
        // If index is greater than half the array length, searches from the back to the front.
        return if index >= len {
            None
        } else if index >= (len - 1) / 2 {
            let mut steps = len - 1 - index;
            let mut cursor = self.inner.cursor_back();
            while steps != 0 {
                let _ = cursor.index()?;
                cursor.move_prev();
                steps -= 1;
            }
            Some(cursor)
        } else {
            let mut steps = index;
            let mut cursor = self.inner.cursor_front();
            while steps != 0 {
                let _ = cursor.index()?;
                cursor.move_next();
                steps -= 1;
            }
            Some(cursor)
        };
    }

    /// Gets the mutable cursor of the specified index node.
    fn get_cursor_mut(&mut self, index: usize) -> Option<CursorMut<'_, JsonValue>> {
        let len = self.len();
        // If index is greater than the array length, returns.
        // If index is less than half the array length, searches from front to back;
        // If index is greater than half the array length, searches from the back to the front.
        return if index >= len {
            None
        } else if index >= (len - 1) / 2 {
            let mut steps = len - 1 - index;
            let mut cursor = self.inner.cursor_back_mut();
            while steps != 0 {
                let _ = cursor.index()?;
                cursor.move_prev();
                steps -= 1;
            }
            Some(cursor)
        } else {
            let mut steps = index;
            let mut cursor = self.inner.cursor_front_mut();
            while steps != 0 {
                let _ = cursor.index()?;
                cursor.move_next();
                steps -= 1;
            }
            Some(cursor)
        };
    }
}

impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;
        for (n, item) in self.inner.iter().enumerate() {
            if n != 0 {
                write!(f, ",")?;
            }
            write!(f, "{item}")?;
        }
        write!(f, "]")
    }
}

impl Debug for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod ut_linked_list {
    use crate::{Array, JsonValue};

    /// UT test for `Array::is_empty`.
    ///
    /// # Title
    /// ut_array_is_empty
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::is_empty`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_is_empty() {
        assert!(Array::new().is_empty());
        assert!(!array!(1).is_empty());
    }

    /// UT test for `Array::pop`.
    ///
    /// # Title
    /// ut_array_pop
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::pop`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_pop() {
        let mut array = array!(1);
        assert_eq!(array.pop(), Some(JsonValue::new_number(1.into())));
        assert_eq!(array.pop(), None);
    }

    /// UT test for `Array::iter_mut`.
    ///
    /// # Title
    /// ut_array_iter_mut
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::iter_mut`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_iter_mut() {
        let mut array = array!(1);
        let mut iter = array.iter_mut();
        assert_eq!(iter.next(), Some(&mut JsonValue::new_number(1.into())));
        assert_eq!(iter.next(), None);
    }

    /// UT test for `Array::last`.
    ///
    /// # Title
    /// ut_array_last
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::last`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_last() {
        let array = array!(1);
        assert_eq!(array.last(), Some(&JsonValue::new_number(1.into())));

        let array = Array::new();
        assert_eq!(array.last(), None);
    }

    /// UT test for `Array::get_node`.
    ///
    /// # Title
    /// ut_array_get_node
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::get_node`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_get_node() {
        let array = array!(1, 2, 3, 4, 5, 6);
        assert!(array.get_node(0).is_some());
        assert!(array.get_node(1).is_some());
        assert!(array.get_node(2).is_some());
        assert!(array.get_node(3).is_some());
        assert!(array.get_node(4).is_some());
        assert!(array.get_node(5).is_some());
        assert!(array.get_node(6).is_none());
    }

    /// UT test for `Array::get_node_mut`.
    ///
    /// # Title
    /// ut_array_get_node_mut
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::get_node_mut`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_get_node_mut() {
        let mut array = array!(1, 2, 3, 4, 5, 6);
        assert!(array.get_node_mut(0).is_some());
        assert!(array.get_node_mut(1).is_some());
        assert!(array.get_node_mut(2).is_some());
        assert!(array.get_node_mut(3).is_some());
        assert!(array.get_node_mut(4).is_some());
        assert!(array.get_node_mut(5).is_some());
        assert!(array.get_node_mut(6).is_none());
    }

    /// UT test for `Array::last_node`.
    ///
    /// # Title
    /// ut_array_last_node
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::last_node`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_last_node() {
        let array = array!(1);
        assert!(array.last_node().is_some());

        let array = Array::new();
        assert!(array.last_node().is_none());
    }

    /// UT test for `Array::last_node_mut`.
    ///
    /// # Title
    /// ut_array_last_node_mut
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::last_node_mut`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_last_node_mut() {
        let mut array = array!(1);
        assert!(array.last_node_mut().is_some());

        let mut array = Array::new();
        assert!(array.last_node_mut().is_none());
    }

    /// UT test for `Array::fmt`.
    ///
    /// # Title
    /// ut_array_fmt
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::fmt`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_fmt() {
        let array = array!(1, 2);
        assert_eq!(format!("{array}"), "[1,2]");
        assert_eq!(format!("{array:?}"), "[1,2]");
    }
}
