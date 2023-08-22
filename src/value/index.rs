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

use crate::{Array, JsonValue, Object};

/// Static NULL, which is returned if the searched key-value pair does not exist.
static NULL: JsonValue = JsonValue::Null;

/// This trait can be used to get an index based on the subscript of an internal member of JsonValue.
pub trait Index: private::IndexSealed {
    /// Gets a common reference to the value with the specified subscript (or key) from a JsonValue.
    fn index_into<'a>(&self, value: &'a JsonValue) -> &'a JsonValue;

    /// Gets a mutable reference to the value of the specified subscript (or key) from a JsonValue.
    fn index_into_mut<'a>(&self, value: &'a mut JsonValue) -> &'a mut JsonValue;

    /// Removes the member with the specified subscript (or key) from a JsonValue.
    fn index_remove(&self, value: &mut JsonValue) -> Option<JsonValue>;
}

impl Index for usize {
    /// Uses the array subscript to visit the Array type of JsonValue
    /// and get a common reference to the corresponding JsonValue.
    /// A null type will be returned in the following two cases:
    ///
    /// 1.Use a subscript to visit non-array types.
    ///
    /// 2.The subscript exceeds the current length of the Array type.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Array};
    ///
    /// // Non-array types
    /// assert_eq!(JsonValue::Number(0.0.into())[0], JsonValue::Null);
    ///
    /// // Array type
    /// let mut array = Array::new();
    /// array.push(JsonValue::Null);
    /// array.push(JsonValue::Boolean(true));
    /// array.push(JsonValue::Number(0.0.into()));
    ///
    /// let value = JsonValue::Array(array);
    ///
    /// // When subscript < length
    /// assert_eq!(value[0], JsonValue::Null);
    /// assert_eq!(value[1], JsonValue::Boolean(true));
    /// assert_eq!(value[2], JsonValue::Number(0.0.into()));
    /// // When subscript >= length
    /// assert_eq!(value[3], JsonValue::Null);
    /// ```
    fn index_into<'a>(&self, value: &'a JsonValue) -> &'a JsonValue {
        if let JsonValue::Array(ref array) = value {
            if *self < array.len() {
                return array.get(*self).unwrap();
            }
        }
        &NULL
    }

    /// Uses the array subscript to visit the Array type of JsonValue
    /// and get a mutable reference to the corresponding JsonValue.
    ///
    /// If the visited JsonValue is not Array type, the JsonValue will be
    /// replaced with an empty Array type and visits again with that subscript.
    ///
    /// If the visited JsonValue is Array type, but the subscript exceeds the length of the array,
    /// then adds a Null type JsonValue at the end of the array and return a mutable reference of it.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Array};
    ///
    /// // Non-array types
    /// let mut value = JsonValue::Null;
    /// value[0] = JsonValue::Null;
    ///
    /// let mut array = Array::new();
    /// array.push(JsonValue::Null);
    /// assert_eq!(value, JsonValue::Array(array));
    ///
    /// // Array type
    /// let mut array = Array::new();
    /// array.push(JsonValue::Null);
    /// let mut value = JsonValue::Array(array);
    ///
    /// // Contains the subscript
    /// value[0] = JsonValue::Number(0.0.into());
    /// assert_eq!(value[0], JsonValue::Number(0.0.into()));
    /// assert_eq!(value.try_as_array().unwrap().len(), 1);
    ///
    /// // Does not contain the subscript
    /// value[1] = JsonValue::Boolean(true);
    /// assert_eq!(value[1], JsonValue::Boolean(true));
    /// assert_eq!(value.try_as_array().unwrap().len(), 2);
    /// ```
    fn index_into_mut<'a>(&self, value: &'a mut JsonValue) -> &'a mut JsonValue {
        if let JsonValue::Array(ref mut array) = value {
            return if *self < array.len() {
                array.get_mut(*self).unwrap()
            } else {
                array.push(JsonValue::Null);
                array.last_mut().unwrap()
            };
        }
        *value = JsonValue::new_array(Array::new());
        self.index_into_mut(value)
    }

    /// Removes the element at the specified location of Array type JsonValue and returns that content.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Array};
    ///
    /// let mut array = Array::new();
    /// array.push(1i32.into());
    ///
    /// let mut value: JsonValue = JsonValue::Array(array);
    /// assert_eq!(value[0], 1i32.into());
    ///
    /// let ret = value.remove(0);
    /// assert_eq!(value[0], JsonValue::Null);
    /// assert_eq!(ret.unwrap(), 1i32.into());
    /// ```
    fn index_remove(&self, value: &mut JsonValue) -> Option<JsonValue> {
        if let JsonValue::Array(ref mut array) = value {
            if *self < array.len() {
                return array.remove(*self);
            }
        }
        None
    }
}

impl Index for str {
    /// Uses key to visit Object type JsonValue, and returns a common reference to corresponding JsonValue.
    /// A null type will be returned in the following two cases:
    ///
    /// 1.Uses key to visit non-object types.
    ///
    /// 2.The searched Object type does not contain the key.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object};
    ///
    /// // Non-object types
    /// assert_eq!(JsonValue::Number(0.0.into())["key"], JsonValue::Null);
    ///
    /// // Object type
    /// let mut object = Object::new();
    /// object.insert(String::from("key"), JsonValue::Number(0.0.into()));
    ///
    /// let value = JsonValue::Object(object);
    ///
    /// // The key exists.
    /// assert_eq!(value["key"], JsonValue::Number(0.0.into()));
    ///
    /// // The key does not exist.
    /// assert_eq!(value["not exist"], JsonValue::Null);
    /// ```
    fn index_into<'a>(&self, value: &'a JsonValue) -> &'a JsonValue {
        if let JsonValue::Object(ref object) = value {
            return object.get(self).unwrap_or(&NULL);
        }
        &NULL
    }

    /// Uses key to visit Object type JsonValue, and returns a mutable reference to corresponding JsonValue.
    ///
    /// If the visited JsonValue is not Object type, the JsonValue will be
    /// replaced with an empty Object type and visits again with that key.
    ///
    /// If the visited JsonValue is of object type but does not contain the key, a key-value pair of
    /// the key and a null type will be inserted and returns a mutable reference to the JsonValue.
    ///
    ///
    /// # Examples
    /// ```
    /// use ylong_json::{JsonValue, Object};
    ///
    /// // Non-object types
    /// let mut value = JsonValue::Null;
    /// let mut object = Object::new();
    /// object.insert(String::from("key"), JsonValue::Number(0.0.into()));
    ///
    /// value["key"] = JsonValue::Number(0.0.into());
    /// assert_eq!(value, JsonValue::Object(object));
    ///
    /// // Object type
    /// let mut object = Object::new();
    /// object.insert(String::from("key"), JsonValue::Number(0.0.into()));
    /// let mut value = JsonValue::Object(object);
    ///
    /// // Contains the key.
    /// value["key"] = JsonValue::Boolean(true);
    /// assert_eq!(value["key"], JsonValue::Boolean(true));
    /// assert_eq!(value.try_as_mut_object().unwrap().len(), 1);
    ///
    /// // Dose not contain the key.
    /// value["not exist"] = JsonValue::Number(1.1.into());
    /// assert_eq!(value["not exist"], JsonValue::Number(1.1.into()));
    /// assert_eq!(value.try_as_mut_object().unwrap().len(), 2);
    /// ```
    fn index_into_mut<'a>(&self, value: &'a mut JsonValue) -> &'a mut JsonValue {
        if let JsonValue::Object(ref mut object) = value {
            #[cfg(feature = "list_object")]
            {
                return object.get_key_mut_maybe_insert(self);
            }
            #[cfg(feature = "vec_object")]
            {
                if let Some(pos) = object.iter().position(|(k, _)| k == self) {
                    return object.get_mut_by_position(pos).unwrap();
                }
                object.insert(String::from(self), JsonValue::Null);
                return object.last_mut().unwrap();
            }
            #[cfg(feature = "btree_object")]
            {
                if !object.contains_key(self) {
                    object.insert(String::from(self), JsonValue::Null);
                }
                return object.get_mut(self).unwrap();
            }
        }
        *value = JsonValue::Object(Object::new());
        self.index_into_mut(value)
    }

    /// Removes the element at the specified location of Object type JsonValue and returns that content.
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
    /// let ret = value.remove("key");
    /// assert_eq!(value["key"], JsonValue::Null);
    /// assert_eq!(ret.unwrap(), "value".into());
    /// ```
    fn index_remove(&self, value: &mut JsonValue) -> Option<JsonValue> {
        if let JsonValue::Object(ref mut object) = value {
            return object.remove(self);
        }
        None
    }
}

impl Index for String {
    /// Same as 'Index for str'.
    fn index_into<'a>(&self, value: &'a JsonValue) -> &'a JsonValue {
        self.as_str().index_into(value)
    }

    /// Same as 'Index for str'.
    fn index_into_mut<'a>(&self, value: &'a mut JsonValue) -> &'a mut JsonValue {
        self.as_str().index_into_mut(value)
    }

    /// Same as 'Index for str'.
    fn index_remove(&self, value: &mut JsonValue) -> Option<JsonValue> {
        self.as_str().index_remove(value)
    }
}

impl<'a, T> Index for &'a T
where
    T: ?Sized + Index,
{
    /// Implements Index for the relevant reference type.
    fn index_into<'v>(&self, value: &'v JsonValue) -> &'v JsonValue {
        (**self).index_into(value)
    }

    /// Implements Index for the relevant reference type.
    fn index_into_mut<'v>(&self, value: &'v mut JsonValue) -> &'v mut JsonValue {
        (**self).index_into_mut(value)
    }

    /// Implements Index for the relevant reference type.
    fn index_remove(&self, value: &mut JsonValue) -> Option<JsonValue> {
        (**self).index_remove(value)
    }
}

// To prevent the Index by external implementation.
mod private {
    pub trait IndexSealed {}

    impl IndexSealed for usize {}

    impl IndexSealed for str {}

    impl IndexSealed for String {}

    impl<'a, T> IndexSealed for &'a T where T: ?Sized + IndexSealed {}
}

#[cfg(test)]
mod ut_index {
    use crate::{Array, Index, JsonValue, Object};

    /// UT test for `usize::index_into`.
    ///
    /// # Title
    /// ut_usize_index_into
    ///
    /// # Brief
    /// 1. Creates some `usize`s and some `JsonValue`s.
    /// 2. Calls `Index::index_into`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_usize_index_into() {
        let value = JsonValue::new_boolean(true);
        assert!(1usize.index_into(&value).is_null());
    }

    /// UT test for `usize::index_into_mut`.
    ///
    /// # Title
    /// ut_usize_index_into_mut
    ///
    /// # Brief
    /// 1. Creates some `usize`s and some `JsonValue`s.
    /// 2. Calls `Index::index_into_mut`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_usize_index_into_mut() {
        let mut value = JsonValue::new_array(array!(1));
        assert!(0usize.index_into_mut(&mut value).is_number());
        assert!(1usize.index_into_mut(&mut value).is_null());

        let mut value = JsonValue::new_null();
        assert!(0usize.index_into_mut(&mut value).is_null());
        assert!(value.is_array())
    }

    /// UT test for `usize::index_remove`.
    ///
    /// # Title
    /// ut_usize_index_remove
    ///
    /// # Brief
    /// 1. Creates some `usize`s and some `JsonValue`s.
    /// 2. Calls `Index::index_remove`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_usize_index_remove() {
        let mut value = JsonValue::new_array(array!(1));
        assert_eq!(
            0usize.index_remove(&mut value),
            Some(JsonValue::new_number(1.into()))
        );
        assert!(0usize.index_remove(&mut value).is_none());
    }

    /// UT test for `str::index_into`.
    ///
    /// # Title
    /// ut_str_index_into
    ///
    /// # Brief
    /// 1. Creates some `str`s and some `JsonValue`s.
    /// 2. Calls `Index::index_into`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_str_index_into() {
        let value = JsonValue::new_boolean(true);
        assert!("key".index_into(&value).is_null());
    }

    /// UT test for `str::index_into_mut`.
    ///
    /// # Title
    /// ut_str_index_into_mut
    ///
    /// # Brief
    /// 1. Creates some `str`s and some `JsonValue`s.
    /// 2. Calls `Index::index_into_mut`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_str_index_into_mut() {
        let mut value = JsonValue::new_object(object!("key1" => "value1"));
        assert!("key1".index_into_mut(&mut value).is_string());
        assert!("key2".index_into_mut(&mut value).is_null());

        let mut value = JsonValue::new_null();
        assert!("key1".index_into_mut(&mut value).is_null());
        assert!(value.is_object())
    }

    /// UT test for `str::index_remove`.
    ///
    /// # Title
    /// ut_str_index_remove
    ///
    /// # Brief
    /// 1. Creates some `str`s and some `JsonValue`s.
    /// 2. Calls `Index::index_remove`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_str_index_remove() {
        let mut value = JsonValue::new_object(object!("key1" => "value1"));
        assert_eq!(
            "key1".index_remove(&mut value),
            Some(JsonValue::new_string("value1"))
        );

        let mut value = JsonValue::new_null();
        assert!("key1".index_remove(&mut value).is_none());
    }

    /// UT test for `String::index_into`.
    ///
    /// # Title
    /// ut_string_index_into
    ///
    /// # Brief
    /// 1. Creates some `String`s and some `JsonValue`s.
    /// 2. Calls `Index::index_into`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_string_index_into() {
        let value = JsonValue::new_boolean(true);
        assert!(String::from("key").index_into(&value).is_null());
    }

    /// UT test for `String::index_into_mut`.
    ///
    /// # Title
    /// ut_string_index_into_mut
    ///
    /// # Brief
    /// 1. Creates some `String`s and some `JsonValue`s.
    /// 2. Calls `Index::index_into_mut`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_string_index_into_mut() {
        let mut value = JsonValue::new_object(object!("key1" => "value1"));
        assert!(String::from("key1").index_into_mut(&mut value).is_string());
        assert!(String::from("key2").index_into_mut(&mut value).is_null());

        let mut value = JsonValue::new_null();
        assert!(String::from("key1").index_into_mut(&mut value).is_null());
        assert!(value.is_object())
    }

    /// UT test for `String::index_remove`.
    ///
    /// # Title
    /// ut_string_index_remove
    ///
    /// # Brief
    /// 1. Creates some `String`s and some `JsonValue`s.
    /// 2. Calls `Index::index_remove`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_string_index_remove() {
        let mut value = JsonValue::new_object(object!("key1" => "value1"));
        assert_eq!(
            String::from("key1").index_remove(&mut value),
            Some(JsonValue::new_string("value1"))
        );
        assert!(String::from("key1").index_remove(&mut value).is_none());
    }
}
