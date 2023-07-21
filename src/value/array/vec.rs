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

/// Array type, implemented using Vec.
///
/// # Situation
/// * When the average number of Array entries exceeds 15 (estimated value), and query operation exists.
///
/// # Attention
/// * 只有开启 `vec_array` feature 时才可以使用，且与其他的 array 相关 feature 冲突。（默认开启）
/// Only open `vec_array` feature can be used, and conflicts with other array-related features. (Enabled by default)
///
/// # Examples
/// ```
/// use ylong_json::Array;
///
/// let array = Array::new();
/// assert_eq!(array.is_empty(), true);
/// ```
#[derive(Default, Clone)]
pub struct Array {
    inner: Vec<JsonValue>,
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
        Self { inner: Vec::new() }
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
        self.inner.len() == 0
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
        self.inner.push(value)
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
        self.inner.pop()
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
        self.inner.get(index)
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
        self.inner.get_mut(index)
    }

    /// Returns a common reference to the trailing ** member ** in Array.。
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
        self.inner.last()
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
        self.inner.last_mut()
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
        if index >= self.inner.len() {
            return None;
        }
        Some(self.inner.remove(index))
    }
}

impl PartialEq for Array {
    /// Determines whether two arrays are equal.
    ///
    /// Two Arrays are equal: They have the same length and the elements in each position are equal.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{Array, JsonValue};
    ///
    /// let array1 = Array::new();
    /// let array2 = Array::new();
    /// let mut array3 = Array::new();
    /// array3.push(JsonValue::Null);
    ///
    /// assert_eq!(array1, array2);
    /// assert_ne!(array1, array3);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (a, b) in self.iter().zip(other.iter()) {
            if a != b {
                return false;
            }
        }
        true
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
mod ut_vec {
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

    /// UT test for `Array::remove`.
    ///
    /// # Title
    /// ut_array_remove
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::remove`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_remove() {
        let mut array = array!(1);
        assert_eq!(array.remove(3), None);
        assert_eq!(array.remove(0), Some(JsonValue::new_number(1.into())));
    }

    /// UT test for `Array::eq`.
    ///
    /// # Title
    /// ut_array_eq
    ///
    /// # Brief
    /// 1. Creates some `Array`s.
    /// 2. Calls `Array::eq`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_array_eq() {
        let array1 = array!(1);
        let array2 = array!(1, 2);
        let array3 = array!(1, 3);

        assert_eq!(array1, array1);
        assert_ne!(array1, array2);
        assert_ne!(array2, array3);
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
