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

#![allow(clippy::missing_safety_doc)]

use crate::*;
use core::ptr::{null_mut, slice_from_raw_parts};
use core::str::from_utf8_unchecked;
use libc::{c_char, c_double, c_int, c_longlong, c_void, strlen};
use std::ffi::CString;

/// Boolean value mapping.
const FALSE: c_int = 0;

/// Operation success or failure.
const SUCCESS: c_int = 1;
const FAILURE: c_int = 0;

/// Empty pointer of YlongJson*
const NULL_MUT_YLONG_JSON: *mut YlongJson = null_mut::<YlongJson>();
/// Empty pointer of char*
const NULL_MUT_CSTR: *mut c_char = null_mut::<c_char>();

/// A void* pointer is passed to C for use.
pub type YlongJson = c_void;

/// Parses a JSON text string.
/// Returns a JSON object on success and null on failure.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_parse(
    value: *mut c_char,
    err_msg: *mut *mut c_char,
) -> *mut YlongJson {
    // Using `ptr::slice_from_raw_parts` here dramatically
    // reduces the cost of converting between char* and &[u8].
    let len = strlen(value);
    let slice = &*slice_from_raw_parts(value as *mut u8, len);

    let value = match JsonValue::from_text(slice) {
        Ok(v) => v,
        Err(e) => {
            // If an error occurs, writes error messages into err_msg.
            *err_msg = CString::from_vec_unchecked(e.to_string().into_bytes()).into_raw();
            return NULL_MUT_YLONG_JSON;
        }
    };

    Box::into_raw(Box::from(value)) as *mut YlongJson
}

/// Frees a C string.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_free_string(string: *mut c_char) {
    if string.is_null() {
        return;
    }

    let _ = Box::from_raw(string);
}

/// Outputs a JSON object to a string in plain format.
/// Returns a C string on success and null on failure.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_print_unformatted(item: *const YlongJson) -> *mut c_char {
    if item.is_null() {
        return NULL_MUT_CSTR;
    }

    let value = &mut *(item as *mut JsonValue);

    // Requests 256 bytes of memory in advance to improve output efficiency.
    // Here we use `Vec::with_capacity(256)`, which performs better when `12 < string.len() < 256`.
    // If here we use `Vec::new()`, it performs better when `string.len() < 32 ||  256 > string.len()`
    // In most cases, `12 < the average string.len() < 256`, so we use `Vec::with_capacity()`.
    let mut vec = Vec::with_capacity(256);
    if value.compact_encode(&mut vec).is_err() {
        return NULL_MUT_CSTR;
    }

    CString::from_vec_unchecked(vec).into_raw()
}

/// Deletes a JSON object.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_delete(item: *mut YlongJson) {
    if item.is_null() {
        return;
    }

    let _ = Box::from_raw(item as *mut JsonValue);
}

/// Duplicates a JSON object.
/// Return a new JSON object on success and null on failure.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_duplicate(
    item: *const YlongJson,
    recurse: c_int,
) -> *mut YlongJson {
    if item.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    if recurse == FALSE {
        let value = &*(item as *mut JsonValue);
        let value_clone = match value {
            JsonValue::Array(_) => JsonValue::Array(Array::new()),
            JsonValue::Object(_) => JsonValue::Object(Object::new()),
            x => x.clone(),
        };
        return Box::into_raw(Box::from(value_clone)) as *mut YlongJson;
    }

    let value = &*(item as *mut JsonValue);
    Box::into_raw(Box::from(value.clone())) as *mut YlongJson
}

/// Creates a JSON null object and returns a new JSON null object.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_create_null() -> *mut YlongJson {
    Box::into_raw(Box::from(JsonValue::Null)) as *mut YlongJson
}

/// Checks whether a JSON object is null.
/// Returns a boolean value indicating whether the object is null.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_is_null(item: *mut YlongJson) -> c_int {
    if item.is_null() {
        return FALSE;
    }

    let item = &*(item as *mut JsonValue);

    item.is_null() as c_int
}

/// Creates a JSON boolean object.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_create_bool(boolean: c_int) -> *mut YlongJson {
    // If it is equal to 0, the result is false. Otherwise it is true.
    Box::into_raw(Box::from(JsonValue::Boolean(boolean != FALSE))) as *mut YlongJson
}

/// Checks whether a JSON object is a boolean.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_is_bool(item: *const YlongJson) -> c_int {
    if item.is_null() {
        return FALSE;
    }

    let item = &*(item as *mut JsonValue);

    item.is_boolean() as c_int
}

/// Gets the boolean value of a JSON boolean object.
/// Returns a boolean value on success and an error code on failure.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_value_from_bool(
    boolean: *const YlongJson,
    value: *mut c_int,
) -> c_int {
    if boolean.is_null() {
        return FAILURE;
    }

    let boolean = &*(boolean as *mut JsonValue);
    let boolean = match boolean.try_as_boolean() {
        Ok(b) => b,
        Err(_) => return FAILURE,
    };
    // For c_int value, true maps to 1, while false maps to 0.
    *value = *boolean as c_int;
    SUCCESS
}

/// Sets the boolean value of a JSON boolean object.
/// Returns a `c_int` indicating whether the operation was successful (SUCCESS) or failed (FAILURE).
#[no_mangle]
pub unsafe extern "C" fn ylong_json_set_value_to_bool(
    boolean: *mut YlongJson,
    value: c_int,
) -> c_int {
    if boolean.is_null() {
        return FAILURE;
    }

    let boolean = &mut *(boolean as *mut JsonValue);
    let boolean = match boolean.try_as_mut_boolean() {
        Ok(b) => b,
        Err(_) => return FAILURE,
    };
    // The *boolean is false if value is 0, and true if value is not 1.
    *boolean = value != FALSE;
    SUCCESS
}

/// Creates a JSON double number object.
/// Returns a pointer to the newly created JSON number.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_create_double_number(number: c_double) -> *mut YlongJson {
    Box::into_raw(Box::from(JsonValue::Number(Number::Float(number)))) as *mut YlongJson
}

/// Creates a JSON integer number object.
/// Returns a pointer to the newly created JSON number.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_create_int_number(number: c_longlong) -> *mut YlongJson {
    Box::into_raw(Box::from(JsonValue::Number(Number::Signed(number)))) as *mut YlongJson
}

/// Checks whether a JSON object is a number.
/// Returns a `c_int` where TRUE indicates that the item is a number, and FALSE indicates otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_is_number(item: *const YlongJson) -> c_int {
    if item.is_null() {
        return FALSE;
    }

    let item = &*(item as *mut JsonValue);
    item.is_number() as c_int
}

/// Checks whether a JSON object is a double number.
/// Returns a `c_int` where TRUE indicates that the number is a double, and FALSE indicates otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_is_double_number(item: *const YlongJson) -> c_int {
    if item.is_null() {
        return FALSE;
    }

    let item = &*(item as *mut JsonValue);
    match item.try_as_number() {
        Ok(n) => matches!(n, Number::Float(_)) as c_int,
        Err(_) => FALSE,
    }
}

/// Checks whether a JSON object is an integer number.
/// Returns a `c_int` where TRUE indicates that the number is an integer, and FALSE indicates otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_is_int_number(item: *const YlongJson) -> c_int {
    if item.is_null() {
        return FALSE;
    }

    let item = &*(item as *mut JsonValue);
    match item.try_as_number() {
        Ok(n) => matches!(n, Number::Signed(_) | Number::Unsigned(_)) as c_int,
        Err(_) => FALSE,
    }
}

/// Gets the double value of a JSON number object.
/// Returns a `c_int` indicating whether the operation was successful (SUCCESS) or failed (FAILURE).
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_double_value_from_number(
    number: *const YlongJson,
    value: *mut c_double,
) -> c_int {
    if number.is_null() {
        return FAILURE;
    }

    let number = &*(number as *mut JsonValue);
    let number = match number.try_as_number() {
        Ok(n) => n,
        Err(_) => return FAILURE,
    };
    // Coercing u64 or i64 to f64 may result in a loss of data accuracy.
    match number {
        Number::Float(f) => *value = *f as c_double,
        Number::Unsigned(u) => *value = *u as c_double,
        Number::Signed(i) => *value = *i as c_double,
    }
    SUCCESS
}

/// Gets the integer value of a JSON number object.
/// Returns a `c_int` indicating whether the operation was successful (SUCCESS) or failed (FAILURE).
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_int_value_from_number(
    number: *const YlongJson,
    value: *mut c_longlong,
) -> c_int {
    if number.is_null() {
        return FAILURE;
    }

    let number = &*(number as *mut JsonValue);
    let number = match number.try_as_number() {
        Ok(n) => n,
        Err(_) => return FAILURE,
    };
    // Coercing u64 or i64 or f64 to i64 may result in a loss of data accuracy.
    match number {
        Number::Float(f) => *value = *f as c_longlong,
        Number::Unsigned(u) => *value = *u as c_longlong,
        Number::Signed(i) => *value = *i as c_longlong,
    }
    SUCCESS
}

/// Sets the double value of a JSON number object.
/// Returns a `c_int` indicating whether the operation was successful (SUCCESS) or failed (FAILURE).
#[no_mangle]
pub unsafe extern "C" fn ylong_json_set_double_value_to_number(
    number: *mut YlongJson,
    value: c_double,
) -> c_int {
    if number.is_null() {
        return FAILURE;
    }

    let number = &mut *(number as *mut JsonValue);
    let number = match number.try_as_mut_number() {
        Ok(n) => n,
        Err(_) => return FAILURE,
    };
    *number = Number::Float(value);
    SUCCESS
}

/// Sets the integer value of a JSON number object.
/// Returns a `c_int` indicating whether the operation was successful (SUCCESS) or failed (FAILURE).
#[no_mangle]
pub unsafe extern "C" fn ylong_json_set_int_value_to_number(
    number: *mut YlongJson,
    value: c_longlong,
) -> c_int {
    if number.is_null() {
        return FAILURE;
    }

    let number = &mut *(number as *mut JsonValue);
    let number = match number.try_as_mut_number() {
        Ok(n) => n,
        Err(_) => return FAILURE,
    };
    *number = Number::Signed(value);
    SUCCESS
}

/// Creates a `YlongJson` string from a given C-style string.
/// If the input string is null, it returns a null `YlongJson`.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_create_string(string: *const c_char) -> *mut YlongJson {
    if string.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let len = strlen(string);
    let slice = &*slice_from_raw_parts(string as *mut u8, len);
    let string = CString::from_vec_unchecked(slice.to_vec());
    Box::into_raw(Box::from(JsonValue::String(string))) as *mut YlongJson
}

/// Checks if the `YlongJson` item is a string.
/// Returns `FALSE` if the item is null or not a string, and `TRUE` otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_is_string(item: *const YlongJson) -> c_int {
    if item.is_null() {
        return FALSE;
    }

    let item = &*(item as *mut JsonValue);
    item.is_string() as c_int
}

/// The char* returned by this function differs from the original data,
/// meaning that any changes to this char* will not be reflected in the original data.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_value_from_string(
    string: *const YlongJson,
    value: *mut *mut c_char,
) -> c_int {
    if string.is_null() {
        return FAILURE;
    }

    let string = &*(string as *mut JsonValue);
    let string = match string.try_as_string() {
        Ok(s) => s,
        Err(_) => return FAILURE,
    };
    // If `c_adapter` feature is on, the pointer of the inner char arrays can be obtained directly,
    // because the string pointer actually points to a `CString`
    *value = string.as_ptr() as *mut c_char;
    SUCCESS
}

/// Sets a `YlongJson` string to a given C-style string.
/// If the `YlongJson` string or the input string is null, it returns `FAILURE`.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_set_value_to_string(
    string: *mut YlongJson,
    value: *const c_char,
) -> c_int {
    if string.is_null() || value.is_null() {
        return FAILURE;
    }

    let string = &mut *(string as *mut JsonValue);
    let string = match string.try_as_mut_string() {
        Ok(s) => s,
        Err(_) => return FAILURE,
    };
    let len = strlen(value);
    let slice = &*slice_from_raw_parts(value as *mut u8, len);
    *string = CString::from_vec_unchecked(slice.to_vec());
    SUCCESS
}

/// Creates a `YlongJson` array.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_create_array() -> *mut YlongJson {
    Box::into_raw(Box::from(JsonValue::Array(Array::new()))) as *mut YlongJson
}

/// Checks if the `YlongJson` item is an array.
/// Returns `FALSE` if the item is null or not an array, and `TRUE` otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_is_array(item: *const YlongJson) -> c_int {
    if item.is_null() {
        return FALSE;
    }

    let item = &*(item as *mut JsonValue);
    item.is_array() as c_int
}

/// Gets the size of a `YlongJson` array.
/// If the `YlongJson` array or the size is null, it returns `FAILURE`.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_array_size(
    array: *const YlongJson,
    size: *mut c_int,
) -> c_int {
    if array.is_null() || size.is_null() {
        return FAILURE;
    }

    let array = &*(array as *mut JsonValue);
    let array = match array.try_as_array() {
        Ok(a) => a,
        Err(_) => return FAILURE,
    };

    *size = array.len() as c_int;
    SUCCESS
}

/// Gets a `YlongJson` item from an array by index.
/// Returns null `YlongJson` if the array is null, the item doesn't exist, or any error occurs.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_array_item(
    array: *const YlongJson,
    index: c_int,
) -> *mut YlongJson {
    if array.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let array_ref = &mut *(array as *mut JsonValue);
    let array_ref = match array_ref.try_as_mut_array() {
        Ok(a) => a,
        Err(_) => return NULL_MUT_YLONG_JSON,
    };

    if index as usize >= array_ref.len() {
        return NULL_MUT_YLONG_JSON;
    }

    array_ref.get_mut(index as usize).unwrap() as *mut JsonValue as *mut YlongJson
}

/// Adds a `YlongJson` item to an array.
/// Returns `FAILURE` if the array or the item is null, and `SUCCESS` otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_add_item_to_array(
    array: *mut YlongJson,
    item: *mut YlongJson,
) -> c_int {
    if array.is_null() || item.is_null() {
        return FAILURE;
    }

    let array_ref = &mut *(array as *mut JsonValue);
    let array_ref = match array_ref.try_as_mut_array() {
        Ok(a) => a,
        Err(_) => return FAILURE,
    };

    let value = Box::from_raw(item as *mut JsonValue);
    array_ref.push(*value);

    SUCCESS
}

/// Replaces a `YlongJson` item in an array by index with a new item.
/// Returns `FAILURE` if the array or the new item is null, the index is out of bounds, or any error occurs, and `SUCCESS` otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_replace_array_item_by_index(
    array: *mut YlongJson,
    index: c_int,
    new_item: *mut YlongJson,
) -> c_int {
    if array.is_null() || new_item.is_null() {
        return FAILURE;
    }

    let array_ref = &mut *(array as *mut JsonValue);
    let array_ref = match array_ref.try_as_mut_array() {
        Ok(o) => o,
        Err(_) => return FAILURE,
    };

    if let Some(value) = array_ref.get_mut(index as usize) {
        let new_value = Box::from_raw(new_item as *mut JsonValue);

        *value = *new_value;

        return SUCCESS;
    }
    FAILURE
}

/// Removes a `YlongJson` item from an array by index.
/// Returns null `YlongJson` if the array is null, the item doesn't exist, or any error occurs.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_remove_array_item_by_index(
    array: *mut YlongJson,
    index: c_int,
) -> *mut YlongJson {
    if array.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let array = &mut *(array as *mut JsonValue);
    let array = match array.try_as_mut_array() {
        Ok(a) => a,
        Err(_) => return NULL_MUT_YLONG_JSON,
    };

    // Uses method 'remove' of Array, but not the method 'remove' of underlying data structure.
    if let Some(v) = Array::remove(array, index as usize) {
        return Box::into_raw(Box::new(v)) as *mut YlongJson;
    }
    NULL_MUT_YLONG_JSON
}

/// Deletes a `YlongJson` item from an array by index.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_delete_array_item_by_index(
    array: *mut YlongJson,
    index: c_int,
) {
    if array.is_null() {
        return;
    }

    let array = &mut *(array as *mut JsonValue);
    let array = match array.try_as_mut_array() {
        Ok(a) => a,
        Err(_) => return,
    };
    array.remove(index as usize);
}

/// In list_array mode, it is more efficient to get a node through this method and then delete it.
#[cfg(feature = "list_array")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_array_node(
    array: *mut YlongJson,
    index: c_int,
) -> *mut YlongJson {
    if array.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let array_ref = &mut *(array as *mut JsonValue);
    let array_ref = match array_ref.try_as_mut_array() {
        Ok(a) => a,
        Err(_) => return NULL_MUT_YLONG_JSON,
    };

    if index as usize >= array_ref.len() {
        return NULL_MUT_YLONG_JSON;
    }

    let node = array_ref.get_node_mut(index as usize).unwrap();
    node as *mut Node<JsonValue> as *mut YlongJson
}

/// Retrieves a `YlongJson` item from an array node.
#[cfg(feature = "list_array")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_item_from_array_node(
    array_node: *mut YlongJson,
) -> *mut YlongJson {
    if array_node.is_null() {
        return NULL_MUT_YLONG_JSON;
    }
    let node = &mut *(array_node as *mut Node<JsonValue>);
    node.get_element_mut() as *mut JsonValue as *mut YlongJson
}

/// Adds a `YlongJson` item to an array, then returns the node.
/// Returns null `YlongJson` if the array or the item is null.
#[cfg(feature = "list_array")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_add_item_to_array_then_get_node(
    array: *mut YlongJson,
    item: *mut YlongJson,
) -> *mut YlongJson {
    if array.is_null() || item.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let array_ref = &mut *(array as *mut JsonValue);
    let array_ref = match array_ref.try_as_mut_array() {
        Ok(a) => a,
        Err(_) => return NULL_MUT_YLONG_JSON,
    };

    let value = Box::from_raw(item as *mut JsonValue);
    array_ref.push(*value);

    array_ref.last_node_mut().unwrap() as *mut Node<JsonValue> as *mut YlongJson
}

/// Replaces an item of an array node with a new item.
/// Returns `FAILURE` if the array node or the new item is null, and `SUCCESS` otherwise.
#[cfg(feature = "list_array")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_replace_item_of_array_node(
    array_node: *mut YlongJson,
    new_item: *mut YlongJson,
) -> c_int {
    if array_node.is_null() || new_item.is_null() {
        return FAILURE;
    }

    let node = &mut *(array_node as *mut Node<JsonValue>);
    let value = node.get_element_mut();

    let new_value = Box::from_raw(new_item as *mut JsonValue);
    *value = *new_value;
    SUCCESS
}

/// Removes an array node.
#[cfg(feature = "list_array")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_remove_array_node(
    array_node: *mut YlongJson,
) -> *mut YlongJson {
    if array_node.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let node = &mut *(array_node as *mut Node<JsonValue>);
    Box::into_raw(Box::new(node.remove_self().unwrap())) as *mut YlongJson
}

/// Deletes an array node.
#[cfg(feature = "list_array")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_delete_array_node(array_node: *mut YlongJson) {
    if array_node.is_null() {
        return;
    }

    let node = &mut *(array_node as *mut Node<JsonValue>);
    let _ = node.remove_self();
}

/// Creates a `YlongJson` object.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_create_object() -> *mut YlongJson {
    Box::into_raw(Box::from(JsonValue::Object(Object::new()))) as *mut YlongJson
}

/// Checks if the `YlongJson` item is an object.
/// Returns `FALSE` if the item is null or not an object, and `TRUE` otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_is_object(item: *const YlongJson) -> c_int {
    if item.is_null() {
        return FALSE;
    }

    let item = &*(item as *mut JsonValue);
    item.is_object() as c_int
}

/// Gets the size of a `YlongJson` object.
/// If the `YlongJson` object or the size is null, it returns `FAILURE`.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_object_size(
    object: *mut YlongJson,
    size: *mut c_int,
) -> c_int {
    if object.is_null() || size.is_null() {
        return FAILURE;
    }

    let object = &mut *(object as *mut JsonValue);
    let object = match object.try_as_mut_object() {
        Ok(o) => o,
        Err(_) => return FAILURE,
    };

    *size = object.len() as c_int;
    SUCCESS
}

/// Checks if a JSON object has a specific item.
/// Returns a `c_int` indicating whether the item exists (TRUE) or not (FALSE).
#[no_mangle]
pub unsafe extern "C" fn ylong_json_has_object_item(
    object: *mut YlongJson,
    string: *const c_char,
) -> c_int {
    if object.is_null() || string.is_null() {
        return FALSE;
    }

    let object = &*(object as *mut JsonValue);
    let object = match object.try_as_object() {
        Ok(o) => o,
        Err(_) => return FALSE,
    };

    // Using `ptr::slice_from_raw_parts` here dramatically
    // reduces the cost of converting between char* and &[u8].
    // Then use `from_utf8_unchecked` to further reduce cost.
    let len = strlen(string);
    let slice = &*slice_from_raw_parts(string as *mut u8, len);
    let str = from_utf8_unchecked(slice);

    object.contains_key(str) as c_int
}

/// Retrieves an item from a JSON object by key.
/// Returns a mutable pointer to the retrieved JSON item.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_object_item(
    object: *const YlongJson,
    string: *const c_char,
) -> *mut YlongJson {
    // If object is empty, the search fails.
    if object.is_null() || string.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let object_ref = &mut *(object as *mut JsonValue);

    // If the type is not object, return err.
    let object_ref = match object_ref.try_as_mut_object() {
        Ok(o) => o,
        Err(_) => return NULL_MUT_YLONG_JSON,
    };

    // Using `ptr::slice_from_raw_parts` here dramatically
    // reduces the cost of converting between char* and &[u8].
    // Then use `from_utf8_unchecked` to further reduce cost.
    let len = strlen(string);
    let slice = &*slice_from_raw_parts(string as *mut u8, len);
    let index = from_utf8_unchecked(slice);

    let target = match object_ref.get_mut(index) {
        Some(v) => v,
        None => return NULL_MUT_YLONG_JSON,
    };
    target as *mut JsonValue as *mut YlongJson
}

/// Adds an item to a JSON object.
/// Returns a `c_int` indicating whether the operation was successful (SUCCESS) or failed (FAILURE).
#[no_mangle]
pub unsafe extern "C" fn ylong_json_add_item_to_object(
    object: *mut YlongJson,
    string: *const c_char,
    item: *mut YlongJson,
) -> c_int {
    // If object or string or item is empty, returns FAILED.
    if object.is_null() || string.is_null() || item.is_null() {
        return FAILURE;
    }

    let object_ref = &mut *(object as *mut JsonValue);
    let object_ref = match object_ref.try_as_mut_object() {
        Ok(o) => o,
        Err(_) => return FAILURE,
    };

    // Using `ptr::slice_from_raw_parts` here dramatically
    // reduces the cost of converting between char* and &[u8].
    // Then use `from_utf8_unchecked` to further reduce cost.
    let len = strlen(string);
    let slice = &*slice_from_raw_parts(string as *mut u8, len);
    let index = from_utf8_unchecked(slice);

    let value = Box::from_raw(item as *mut JsonValue);

    object_ref.insert(String::from(index), *value);

    SUCCESS
}

/// Replaces an item in a JSON object by key.
/// Returns a `c_int` indicating whether the operation was successful (SUCCESS) or failed (FAILURE).
#[no_mangle]
pub unsafe extern "C" fn ylong_json_replace_object_item_by_index(
    object: *mut YlongJson,
    index: *const c_char,
    new_item: *mut YlongJson,
) -> c_int {
    if object.is_null() || index.is_null() || new_item.is_null() {
        return FAILURE;
    }

    let object_ref = &mut *(object as *mut JsonValue);
    let object_ref = match object_ref.try_as_mut_object() {
        Ok(o) => o,
        Err(_) => return FAILURE,
    };

    // Using `ptr::slice_from_raw_parts` here dramatically
    // reduces the cost of converting between char* and &[u8].
    // Then use `from_utf8_unchecked` to further reduce cost.
    let len = strlen(index);
    let slice = &*slice_from_raw_parts(index as *mut u8, len);
    let index = from_utf8_unchecked(slice);

    if let Some(value) = object_ref.get_mut(index) {
        let new_value = Box::from_raw(new_item as *mut JsonValue);

        *value = *new_value;

        return SUCCESS;
    }

    FAILURE
}

/// Removes an item in a JSON object by index.
/// Returns a new JSON object without the item if successful, NULL_MUT_YLONG_JSON otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_remove_object_item_by_index(
    object: *mut YlongJson,
    index: *const c_char,
) -> *mut YlongJson {
    if object.is_null() || index.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let object = &mut *(object as *mut JsonValue);

    // Using `ptr::slice_from_raw_parts` here dramatically
    // reduces the cost of converting between char* and &[u8].
    // Then use `from_utf8_unchecked` to further reduce cost.
    let len = strlen(index);
    let slice = &*slice_from_raw_parts(index as *mut u8, len);
    let index = from_utf8_unchecked(slice);

    if let Some(v) = object.remove(index) {
        return Box::into_raw(Box::new(v)) as *mut YlongJson;
    }
    NULL_MUT_YLONG_JSON
}

/// Deletes an item in a JSON object by index.
/// Does not return a value.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_delete_object_item_by_index(
    object: *mut YlongJson,
    index: *const c_char,
) {
    if object.is_null() || index.is_null() {
        return;
    }

    let object = &mut *(object as *mut JsonValue);

    // Using ptr::slice_from_raw_parts here dramatically
    // reduces the cost of converting between char* and &[u8].
    // Then use from_utf8_unchecked to further reduce cost.
    let len = strlen(index);
    let slice = &*slice_from_raw_parts(index as *mut u8, len);
    let index = from_utf8_unchecked(slice);

    object.remove(index);
}

/// Gets all items from a JSON object.
/// Returns SUCCESS if the operation is successful, FAILURE otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_all_object_items(
    object: *mut YlongJson,
    key: *mut *mut c_char,
    value: *mut *mut YlongJson,
    len: *mut c_int,
) -> c_int {
    if object.is_null() || key.is_null() || value.is_null() || len.is_null() {
        return FAILURE;
    }

    let object = &mut *(object as *mut JsonValue);
    let object = match object.try_as_mut_object() {
        Ok(o) => o,
        Err(_) => return FAILURE,
    };

    for (n, (k, v)) in object.iter_mut().enumerate() {
        // k.clone ().into_bytes() is more efficient than k.as_bytes ().to_vec().
        let k = CString::from_vec_unchecked(k.clone().into_bytes()).into_raw();
        let v = v as *mut JsonValue as *mut YlongJson;
        *(key.add(n)) = k;
        *(value.add(n)) = v;
    }
    *len = object.len() as c_int;
    SUCCESS
}

/// Applies a function to each item in a JSON object.
/// Returns SUCCESS if the operation is successful, FAILURE otherwise.
#[no_mangle]
pub unsafe extern "C" fn ylong_json_for_each_object_item(
    object: *mut YlongJson,
    func: unsafe extern "C" fn(*mut YlongJson),
) -> c_int {
    if object.is_null() {
        return FAILURE;
    }

    let object = &mut *(object as *mut JsonValue);
    let object = match object.try_as_mut_object() {
        Ok(o) => o,
        Err(_) => return FAILURE,
    };

    object.iter_mut().for_each(|(_k, v)| {
        let value = v as *mut JsonValue as *mut YlongJson;
        func(value);
    });
    SUCCESS
}

/// Gets an object node from a JSON object by key.
/// Returns a pointer to the object node if successful, NULL_MUT_YLONG_JSON otherwise.
#[cfg(feature = "list_object")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_object_node(
    object: *const YlongJson,
    string: *const c_char,
) -> *mut YlongJson {
    // If object is empty, the search fails.
    if object.is_null() || string.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let object_ref = &mut *(object as *mut JsonValue);

    // If the type is not object, returns err.
    let object_ref = match object_ref.try_as_mut_object() {
        Ok(o) => o,
        Err(_) => return NULL_MUT_YLONG_JSON,
    };

    // Using `ptr::slice_from_raw_parts` here dramatically
    // reduces the cost of converting between char* and &[u8].
    // Then use `from_utf8_unchecked` to further reduce cost.
    let len = strlen(string);
    let slice = &*slice_from_raw_parts(string as *mut u8, len);
    let index = from_utf8_unchecked(slice);

    // When using list to get a node, the return value points to the memory is CursorMut<JsonValue>.
    let target = match object_ref.get_node_mut(index) {
        Some(v) => v,
        None => return NULL_MUT_YLONG_JSON,
    };
    target as *mut Node<(String, JsonValue)> as *mut YlongJson
}

/// Gets an item from an object node.
/// Returns a pointer to the item if successful, NULL_MUT_YLONG_JSON otherwise.
#[cfg(feature = "list_object")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_get_item_from_object_node(
    object_node: *mut YlongJson,
) -> *mut YlongJson {
    if object_node.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let node = &mut *(object_node as *mut Node<(String, JsonValue)>);
    (&mut node.get_element_mut().1) as *mut JsonValue as *mut YlongJson
}

/// Adds an item to a JSON object, then returns a pointer to the object node.
/// Returns a pointer to the object node if successful, NULL_MUT_YLONG_JSON otherwise.
#[cfg(feature = "list_object")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_add_item_to_object_then_get_node(
    object: *mut YlongJson,
    string: *const c_char,
    item: *mut YlongJson,
) -> *mut YlongJson {
    // If object or item is empty, returns NULL_MUT_YLONG_JSON.
    if object.is_null() || string.is_null() || item.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let object_ref = &mut *(object as *mut JsonValue);
    let object_ref = match object_ref.try_as_mut_object() {
        Ok(v) => v,
        Err(_) => return NULL_MUT_YLONG_JSON,
    };

    // Using `ptr::slice_from_raw_parts` here dramatically
    // reduces the cost of converting between char* and &[u8].
    // Then use `from_utf8_unchecked` to further reduce cost.
    let len = strlen(string);
    let slice = &*slice_from_raw_parts(string as *mut u8, len);
    let string = from_utf8_unchecked(slice);

    let value = Box::from_raw(item as *mut JsonValue);
    object_ref.insert(String::from(string), *value);

    let target = object_ref.last_node_mut().unwrap();
    target as *mut Node<(String, JsonValue)> as *mut YlongJson
}

/// Replaces an item in an object node.
/// Returns SUCCESS if the operation is successful, FAILURE otherwise.
#[cfg(feature = "list_object")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_replace_item_of_object_node(
    object_node: *mut YlongJson,
    new_item: *mut YlongJson,
) -> c_int {
    if object_node.is_null() || new_item.is_null() {
        return FAILURE;
    }

    let node = &mut *(object_node as *mut Node<(String, JsonValue)>);
    let (_, value) = node.get_element_mut();
    let new_value = Box::from_raw(new_item as *mut JsonValue);
    *value = *new_value;

    SUCCESS
}

/// Removes an object node.
/// Returns a pointer to the removed item if successful, NULL_MUT_YLONG_JSON otherwise.
#[cfg(feature = "list_object")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_remove_object_node(
    object_node: *mut YlongJson,
) -> *mut YlongJson {
    if object_node.is_null() {
        return NULL_MUT_YLONG_JSON;
    }

    let node = &mut *(object_node as *mut Node<(String, JsonValue)>);
    Box::into_raw(Box::new(node.remove_self().unwrap().1)) as *mut YlongJson
}

/// Deletes a node from a JSON object.
#[cfg(feature = "list_object")]
#[no_mangle]
pub unsafe extern "C" fn ylong_json_delete_object_node(object_node: *mut YlongJson) {
    if object_node.is_null() {
        return;
    }

    let node = &mut *(object_node as *mut Node<(String, JsonValue)>);
    let _ = node.remove_self();
}

#[cfg(test)]
mod ut_adapter {
    use crate::*;
    use libc::*;
    use std::ffi::{CStr, CString};
    use std::mem::size_of;
    use std::ptr::{null, null_mut};

    const JSON_TEXT: &str = r#"
{
    "null": null,
    "true": true,
    "false": false,
    "number": 3.14,
    "string": "Hello World!",
    "array": [1, 2, 3],
    "object": {
        "key1": 1,
        "key2": 2,
        "key3": 3
    }
}
"#;

    unsafe fn str_to_c_char(str: &str) -> *mut c_char {
        CString::from_vec_unchecked(str.as_bytes().to_vec()).into_raw()
    }

    /// UT test for `ylong_json_parse`.
    ///
    /// # Title
    /// ut_ylong_json_parse
    ///
    /// # Brief
    /// 1. Calls `ylong_json_parse` to generate a JsonValue as YlongJson*.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_parse() {
        unsafe {
            // Passes in the correct syntax text string.
            let str = str_to_c_char(JSON_TEXT);
            let err = null_mut::<c_char>();
            let json = ylong_json_parse(str, &err as *const *mut c_char as *mut *mut c_char);
            // No error message.
            assert!(err.is_null());
            // The data structure is correct.
            assert!(!json.is_null());

            // Destruction
            let _ = Box::from_raw(str);
            ylong_json_delete(json);

            // Passes in the incorrect syntax text string.
            let str = str_to_c_char("{");
            let err = null_mut::<c_char>();
            let json = ylong_json_parse(str, &err as *const *mut c_char as *mut *mut c_char);
            // Here is an error message.
            assert!(!err.is_null());
            // No correct syntax structure.
            assert!(json.is_null());

            // Destruction
            ylong_json_free_string(err);
            let _ = Box::from_raw(str);
            ylong_json_delete(json);
        }
    }

    //noinspection SpellCheckingInspection
    //noinspection ALL
    /// UT test for `ylong_json_free_string`.
    ///
    /// # Title
    /// ut_ylong_json_free_string
    ///
    /// # Brief
    /// 1. Calls `ylong_json_free_string` to free a YlongJson*(`C` string).
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_free_string() {
        unsafe {
            // Null ptr scene, if the process does not exit abnormally, it is successful.
            let string = null_mut();
            ylong_json_free_string(string);

            let str = str_to_c_char(JSON_TEXT);
            let err = null_mut::<c_char>();
            let json = ylong_json_parse(str, &err as *const *mut c_char as *mut *mut c_char);
            assert!(err.is_null());
            assert!(!json.is_null());

            // The char* generated by `ylong_json_print_unformatted` needs
            // to be destructed by calling `ylong_json_free_string`.
            let result = ylong_json_print_unformatted(json);
            ylong_json_free_string(result);

            // Destruction
            let _ = Box::from_raw(str);
            ylong_json_delete(json);
        }
    }

    /// UT test for `ylong_json_print_unformatted`.
    ///
    /// # Title
    /// ut_ylong_json_print_unformatted
    ///
    /// # Brief
    /// 1. Calls `ylong_json_print_unformatted` to print the value as `C` string.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_print_unformatted() {
        unsafe {
            // Null ptr
            let json = null_mut();
            assert!(ylong_json_print_unformatted(json).is_null());

            // Correct scene
            let str = str_to_c_char("{\"array\":[1,2,3]}");
            let err = null_mut::<c_char>();
            let json = ylong_json_parse(str, &err as *const *mut c_char as *mut *mut c_char);
            assert!(err.is_null());
            assert!(!json.is_null());

            let result = ylong_json_print_unformatted(json);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{\"array\":[1,2,3]}");

            // Destruction
            let _ = Box::from_raw(str);
            ylong_json_delete(json);
        }
    }

    /// UT test for `ylong_json_delete`.
    ///
    /// # Title
    /// ut_ylong_json_delete
    ///
    /// # Brief
    /// 1. Calls `ylong_json_delete` to delete the value.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_delete() {
        unsafe {
            // Null ptr scene, if the process does not exit abnormally, it is successful.
            let json = null_mut();
            ylong_json_delete(json);

            // The YlongJson* generated by `ylong_json_parse` needs
            // to be destructed by calling `ylong_json_delete`.
            let str = str_to_c_char(JSON_TEXT);
            let err = null_mut::<c_char>();
            let json = ylong_json_parse(str, &err as *const *mut c_char as *mut *mut c_char);
            assert!(err.is_null());
            assert!(!json.is_null());
            let _ = Box::from_raw(str);
            ylong_json_delete(json);

            // If the YlongJson* generated by the function starting with
            // `ylong_json_create` is not inserted into another YlongJson*,
            // YlongJson* needs to be destructed by calling `ylong_json_delete`.
            let null = ylong_json_create_null();
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_duplicate`.
    ///
    /// # Title
    /// ut_ylong_json_duplicate
    ///
    /// # Brief
    /// 1. Calls `ylong_json_duplicate` to clone the value.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_duplicate() {
        unsafe {
            // Null ptr
            let json = null_mut();
            assert!(ylong_json_duplicate(json, 0).is_null());

            // Null ptr
            let json = null_mut();
            assert!(ylong_json_duplicate(json, 1).is_null());

            let str = str_to_c_char(JSON_TEXT);
            let err = null_mut::<c_char>();
            let json = ylong_json_parse(str, &err as *const *mut c_char as *mut *mut c_char);
            assert!(err.is_null());
            assert!(!json.is_null());

            // If recurse is 0, does not clone recursively.
            let duplicate = ylong_json_duplicate(json, 0);
            let result = ylong_json_print_unformatted(duplicate);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{}");
            // Destruction
            let _ = Box::from_raw(str);
            ylong_json_delete(duplicate);

            // If recurse is not 0, do recursive cloning.
            let duplicate = ylong_json_duplicate(json, 1);
            let result = ylong_json_print_unformatted(duplicate);
            let result = CString::from_raw(result).into_string().unwrap();
            let origin = ylong_json_print_unformatted(json);
            let origin = CString::from_raw(origin).into_string().unwrap();
            // The json address value is not equal to duplicate,
            // which means it is a different instance.
            assert_ne!(duplicate, json);
            // But the output is the same.
            assert_eq!(result, origin);
            // Destruction
            ylong_json_delete(duplicate);
            ylong_json_delete(json);
        }
    }

    /// UT test for `ylong_json_create_null`.
    ///
    /// # Title
    /// ut_ylong_json_create_null
    ///
    /// # Brief
    /// 1. Calls `ylong_json_create_null` to create a null.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_create_null() {
        unsafe {
            let null = ylong_json_create_null();
            assert_eq!(ylong_json_is_null(null), 1);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_is_null`.
    ///
    /// # Title
    /// ut_ylong_json_is_null
    ///
    /// # Brief
    /// 1. Calls `ylong_json_is_null` to determine whether the underlying structure is null.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_is_null() {
        unsafe {
            // Null ptr
            let null = null_mut();
            assert_eq!(ylong_json_is_null(null), 0);

            // If the underlying structure is Null, returns true.
            let null = ylong_json_create_null();
            assert_eq!(ylong_json_is_null(null), 1);
            ylong_json_delete(null);

            // Else returns false.
            let bool = ylong_json_create_bool(0xffff);
            assert_eq!(ylong_json_is_null(bool), 0);
            ylong_json_delete(bool);
        }
    }

    /// UT test for `ylong_json_create_bool`.
    ///
    /// # Title
    /// ut_ylong_json_create_bool
    ///
    /// # Brief
    /// 1. Calls `ylong_json_create_bool` to create a bool.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_create_bool() {
        unsafe {
            // Creates true.
            let bool = ylong_json_create_bool(0xffff);
            let mut val = 0;
            ylong_json_get_value_from_bool(bool, &mut val as *mut c_int);
            assert_eq!(val, 1);
            ylong_json_delete(bool);

            // Creates false.
            let bool = ylong_json_create_bool(0);
            let mut val = 1;
            ylong_json_get_value_from_bool(bool, &mut val as *mut c_int);
            assert_eq!(val, 0);
            ylong_json_delete(bool);
        }
    }

    /// UT test for `ylong_json_is_bool`.
    ///
    /// # Title
    /// ut_ylong_json_is_bool
    ///
    /// # Brief
    /// 1. Calls `ylong_json_is_bool` to determine whether the underlying structure is bool.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_is_bool() {
        unsafe {
            // Null ptr
            let bool = null_mut();
            assert_eq!(ylong_json_is_bool(bool), 0);

            // True
            let bool = ylong_json_create_bool(0xffff);
            assert_eq!(ylong_json_is_bool(bool), 1);
            ylong_json_delete(bool);

            // False
            let bool = ylong_json_create_bool(0);
            assert_eq!(ylong_json_is_bool(bool), 1);
            ylong_json_delete(bool);

            // Non-bool case
            let null = ylong_json_create_null();
            assert_eq!(ylong_json_is_bool(null), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_value_from_bool`.
    ///
    /// # Title
    /// ut_ylong_json_get_value_from_bool
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_value_from_bool` to get value from bool.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_get_value_from_bool() {
        unsafe {
            // Null ptr
            let bool = null_mut();
            let mut val = 0i32;
            assert_eq!(
                ylong_json_get_value_from_bool(bool, &mut val as *mut c_int),
                0
            );

            let bool = ylong_json_create_bool(0xffff);
            let mut val = 0;
            assert_eq!(
                ylong_json_get_value_from_bool(bool, &mut val as *mut c_int),
                1
            );
            assert_eq!(val, 1);
            ylong_json_delete(bool);

            let null = ylong_json_create_null();
            let mut val = 0;
            assert_eq!(
                ylong_json_get_value_from_bool(null, &mut val as *mut c_int),
                0
            );
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_set_value_to_bool`.
    ///
    /// # Title
    /// ut_ylong_json_set_value_to_bool
    ///
    /// # Brief
    /// 1. Calls `ylong_json_set_value_to_bool` to set value to bool.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_set_value_to_bool() {
        unsafe {
            // Null ptr
            let bool = null_mut();
            assert_eq!(ylong_json_set_value_to_bool(bool, 1), 0);

            let bool = ylong_json_create_bool(0xffff);
            let mut val = 0;
            assert_eq!(
                ylong_json_get_value_from_bool(bool, &mut val as *mut c_int),
                1
            );
            assert_eq!(val, 1);

            assert_eq!(ylong_json_set_value_to_bool(bool, 0), 1);
            assert_eq!(
                ylong_json_get_value_from_bool(bool, &mut val as *mut c_int),
                1
            );
            assert_eq!(val, 0);
            ylong_json_delete(bool);

            let null = ylong_json_create_null();
            assert_eq!(ylong_json_set_value_to_bool(null, 0), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_create_double_number`.
    ///
    /// # Title
    /// ut_ylong_json_create_double_number
    ///
    /// # Brief
    /// 1. Calls `ylong_json_create_double_number` to create a double number.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_create_double_number() {
        unsafe {
            let double = ylong_json_create_double_number(3.24);
            let mut number = 0f64;
            ylong_json_get_double_value_from_number(double, &mut number as *mut c_double);
            assert_eq!(number, 3.24);
            ylong_json_delete(double);
        }
    }

    /// UT test for `ylong_json_create_int_number`.
    ///
    /// # Title
    /// ut_ylong_json_create_int_number
    ///
    /// # Brief
    /// 1. Calls `ylong_json_create_int_number` to create a int number.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_create_int_number() {
        unsafe {
            let int = ylong_json_create_int_number(0xffff);
            let mut number = 0i64;
            ylong_json_get_int_value_from_number(int, &mut number as *mut c_longlong);
            assert_eq!(number, 0xffff);
            ylong_json_delete(int);
        }
    }

    /// UT test for `ylong_json_is_number`.
    ///
    /// # Title
    /// ut_ylong_json_is_number
    ///
    /// # Brief
    /// 1. Calls `ylong_json_is_number` to determine whether the value is number.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_is_number() {
        unsafe {
            // Null ptr
            let number = null_mut();
            assert_eq!(ylong_json_is_number(number), 0);

            let int = ylong_json_create_int_number(1i64);
            assert_eq!(ylong_json_is_number(int), 1);
            ylong_json_delete(int);

            let double = ylong_json_create_double_number(3.24);
            assert_eq!(ylong_json_is_number(double), 1);
            ylong_json_delete(double);

            let null = ylong_json_create_null();
            assert_eq!(ylong_json_is_number(null), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_is_double_number`.
    ///
    /// # Title
    /// ut_ylong_json_is_double_number
    ///
    /// # Brief
    /// 1. Calls `ylong_json_is_double_number` to determine whether the value is double number.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_is_double_number() {
        unsafe {
            // Null ptr
            let double = null_mut();
            assert_eq!(ylong_json_is_double_number(double), 0);

            let int = ylong_json_create_int_number(1i64);
            assert_eq!(ylong_json_is_double_number(int), 0);
            ylong_json_delete(int);

            let double = ylong_json_create_double_number(3.24);
            assert_eq!(ylong_json_is_double_number(double), 1);
            ylong_json_delete(double);

            let null = ylong_json_create_null();
            assert_eq!(ylong_json_is_double_number(null), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_is_int_number`.
    ///
    /// # Title
    /// ut_ylong_json_is_int_number
    ///
    /// # Brief
    /// 1. Calls `ylong_json_is_int_number` to determine whether the value is int number.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_is_int_number() {
        unsafe {
            // Null ptr
            let int = null_mut();
            assert_eq!(ylong_json_is_int_number(int), 0);

            let int = ylong_json_create_int_number(1i64);
            assert_eq!(ylong_json_is_int_number(int), 1);
            ylong_json_delete(int);

            let double = ylong_json_create_double_number(3.24);
            assert_eq!(ylong_json_is_int_number(double), 0);
            ylong_json_delete(double);

            let null = ylong_json_create_null();
            assert_eq!(ylong_json_is_int_number(null), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_double_value_from_number`.
    ///
    /// # Title
    /// ut_ylong_json_get_double_value_from_number
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_double_value_from_number` to get double value from number.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_get_double_value_from_number() {
        unsafe {
            // Null ptr
            let double = null_mut();
            let mut number = 0f64;
            assert_eq!(
                ylong_json_get_double_value_from_number(double, &mut number as *mut c_double),
                0
            );

            let int = ylong_json_create_int_number(1i64);
            let mut number = 0f64;
            assert_eq!(
                ylong_json_get_double_value_from_number(int, &mut number as *mut c_double),
                1
            );
            assert_eq!(number, 1.0);
            ylong_json_delete(int);

            let double = ylong_json_create_double_number(3.24);
            let mut number = 0f64;
            assert_eq!(
                ylong_json_get_double_value_from_number(double, &mut number as *mut c_double),
                1
            );
            assert_eq!(number, 3.24);
            ylong_json_delete(double);

            let null = ylong_json_create_null();
            let mut number = 0f64;
            assert_eq!(
                ylong_json_get_double_value_from_number(null, &mut number as *mut c_double),
                0
            );
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_int_value_from_number`.
    ///
    /// # Title
    /// ut_ylong_json_get_int_value_from_number
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_int_value_from_number` to get int value from number.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_get_int_value_from_number() {
        unsafe {
            // Null ptr
            let int = null_mut();
            let mut number = 0i64;
            assert_eq!(
                ylong_json_get_int_value_from_number(int, &mut number as *mut c_longlong),
                0
            );

            let int = ylong_json_create_int_number(1i64);
            let mut number = 0i64;
            assert_eq!(
                ylong_json_get_int_value_from_number(int, &mut number as *mut c_longlong),
                1
            );
            assert_eq!(number, 1i64);
            ylong_json_delete(int);

            let double = ylong_json_create_double_number(3.24);
            let mut number = 0i64;
            assert_eq!(
                ylong_json_get_int_value_from_number(double, &mut number as *mut c_longlong),
                1
            );
            assert_eq!(number, 3i64);
            ylong_json_delete(double);

            let null = ylong_json_create_null();
            let mut number = 0i64;
            assert_eq!(
                ylong_json_get_int_value_from_number(null, &mut number as *mut c_longlong),
                0
            );
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_set_double_value_to_number`.
    ///
    /// # Title
    /// ut_ylong_json_set_double_value_to_number
    ///
    /// # Brief
    /// 1. Calls `ylong_json_set_double_value_to_number` to set double value to number.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_set_double_value_to_number() {
        unsafe {
            // Null ptr
            let number = null_mut();
            assert_eq!(ylong_json_set_double_value_to_number(number, 3.24), 0);

            let double = ylong_json_create_double_number(3.24);
            let mut number = 0f64;
            ylong_json_get_double_value_from_number(double, &mut number as *mut c_double);
            assert_eq!(number, 3.24);
            assert_eq!(ylong_json_set_double_value_to_number(double, 1.23), 1);
            ylong_json_get_double_value_from_number(double, &mut number as *mut c_double);
            assert_eq!(number, 1.23);
            ylong_json_delete(double);

            let int = ylong_json_create_int_number(1i64);
            let mut number = 0i64;
            ylong_json_get_int_value_from_number(int, &mut number as *mut c_longlong);
            assert_eq!(number, 1i64);
            assert_eq!(ylong_json_set_double_value_to_number(int, 3.24), 1);
            ylong_json_get_int_value_from_number(int, &mut number as *mut c_longlong);
            assert_eq!(number, 3i64);
            ylong_json_delete(int);

            let null = ylong_json_create_null();
            assert_eq!(ylong_json_set_double_value_to_number(null, 3.24), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_set_int_value_to_number`.
    ///
    /// # Title
    /// ut_ylong_json_set_int_value_to_number
    ///
    /// # Brief
    /// 1. Calls `ylong_json_set_int_value_to_number` to set int value to number.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_set_int_value_to_number() {
        unsafe {
            // Null ptr
            let number = null_mut();
            assert_eq!(ylong_json_set_int_value_to_number(number, 1), 0);

            let int = ylong_json_create_int_number(1i64);
            let mut number = 0i64;
            ylong_json_get_int_value_from_number(int, &mut number as *mut c_longlong);
            assert_eq!(number, 1i64);
            assert_eq!(ylong_json_set_int_value_to_number(int, 3i64), 1);
            ylong_json_get_int_value_from_number(int, &mut number as *mut c_longlong);
            assert_eq!(number, 3i64);
            ylong_json_delete(int);

            let double = ylong_json_create_double_number(3.24);
            let mut number = 0f64;
            ylong_json_get_double_value_from_number(double, &mut number as *mut c_double);
            assert_eq!(number, 3.24);
            assert_eq!(ylong_json_set_int_value_to_number(double, 1), 1);
            ylong_json_get_double_value_from_number(double, &mut number as *mut c_double);
            assert_eq!(number, 1.0);
            ylong_json_delete(double);

            let null = ylong_json_create_null();
            assert_eq!(ylong_json_set_int_value_to_number(null, 1), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_create_string`.
    ///
    /// # Title
    /// ut_ylong_json_create_string
    ///
    /// # Brief
    /// 1. Calls `ylong_json_create_string` to create a string from *mut c_char.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_create_string() {
        unsafe {
            // Null ptr
            let str = null();
            assert!(ylong_json_create_string(str).is_null());

            let str = str_to_c_char("Hello World");
            let string = ylong_json_create_string(str);
            let mut content = null_mut();
            ylong_json_get_value_from_string(string, &mut content as *mut *mut c_char);
            let result = String::from_utf8_unchecked(CStr::from_ptr(content).to_bytes().to_vec());
            assert_eq!(result, "Hello World");
            // Destruction
            let _ = Box::from_raw(str);
            ylong_json_delete(string);
        }
    }

    /// UT test for `ylong_json_is_string`.
    ///
    /// # Title
    /// ut_ylong_json_is_string
    ///
    /// # Brief
    /// 1. Calls `ylong_json_is_string` to determine whether the value is string.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_is_string() {
        unsafe {
            // Null ptr
            let string = null_mut();
            assert_eq!(ylong_json_is_string(string), 0);

            let str = str_to_c_char("Hello World");
            let string = ylong_json_create_string(str);
            assert_eq!(ylong_json_is_string(string), 1);
            let _ = Box::from_raw(str);
            ylong_json_delete(string);

            let null = ylong_json_create_null();
            assert_eq!(ylong_json_is_string(null), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_value_from_string`.
    ///
    /// # Title
    /// ut_ylong_json_get_value_from_string
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_value_from_string` to get value from string.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_get_value_from_string() {
        unsafe {
            // Null ptr
            let string = null_mut();
            let mut str = null_mut();
            assert_eq!(
                ylong_json_get_value_from_string(string, &mut str as *mut *mut c_char),
                0
            );

            let str = str_to_c_char("Hello World");
            let string = ylong_json_create_string(str);
            let mut content = null_mut();
            assert_eq!(
                ylong_json_get_value_from_string(string, &mut content as *mut *mut c_char),
                1
            );
            let result = String::from_utf8_unchecked(CStr::from_ptr(content).to_bytes().to_vec());
            assert_eq!(result, "Hello World");
            let _ = Box::from_raw(str);
            ylong_json_delete(string);

            let null = ylong_json_create_null();
            let mut content = null_mut();
            assert_eq!(
                ylong_json_get_value_from_string(null, &mut content as *mut *mut c_char),
                0
            );
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_set_value_to_string`.
    ///
    /// # Title
    /// ut_ylong_json_set_value_to_string
    ///
    /// # Brief
    /// 1. Calls `ylong_json_set_value_to_string` to set value to string.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_set_value_to_string() {
        unsafe {
            // Null ptr
            let string = null_mut();
            let str = str_to_c_char("Hello World");
            assert_eq!(ylong_json_set_value_to_string(string, str), 0);
            let _ = Box::from_raw(str);

            // Null ptr
            let str = str_to_c_char("Hello World");
            let string = ylong_json_create_string(str);
            let _ = Box::from_raw(str);
            let str = null();
            assert_eq!(ylong_json_set_value_to_string(string, str), 0);
            ylong_json_delete(string);

            let str = str_to_c_char("Hello World");
            let string = ylong_json_create_string(str);
            // Check if the original value is "Hello World".
            let mut content = null_mut();
            ylong_json_get_value_from_string(string, &mut content as *mut *mut c_char);
            let result = String::from_utf8_unchecked(CStr::from_ptr(content).to_bytes().to_vec());
            assert_eq!(result, "Hello World");
            let _ = Box::from_raw(str);
            // Use the function to set the content to "New String".
            let str = str_to_c_char("New String");
            assert_eq!(ylong_json_set_value_to_string(string, str), 1);
            // Check whether the Settings are successful.
            ylong_json_get_value_from_string(string, &mut content as *mut *mut c_char);
            let result = String::from_utf8_unchecked(CStr::from_ptr(content).to_bytes().to_vec());
            assert_eq!(result, "New String");
            let _ = Box::from_raw(str);
            ylong_json_delete(string);

            let null = ylong_json_create_null();
            let str = str_to_c_char("New String");
            assert_eq!(ylong_json_set_value_to_string(null, str), 0);
            let _ = Box::from_raw(str);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_create_array`.
    ///
    /// # Title
    /// ut_ylong_json_create_array
    ///
    /// # Brief
    /// 1. Calls `ylong_json_create_array` to create an array.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_create_array() {
        unsafe {
            let array = ylong_json_create_array();
            let result = ylong_json_print_unformatted(array);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "[]");
            ylong_json_delete(array);
        }
    }

    /// UT test for `ut_ylong_json_is_array`.
    ///
    /// # Title
    /// ut_ylong_json_is_array
    ///
    /// # Brief
    /// 1. Calls `long_json_is_array` to determine whether the value is an array.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_is_array() {
        unsafe {
            // Null ptr
            let array = null_mut();
            assert_eq!(ylong_json_is_array(array), 0);

            let array = ylong_json_create_array();
            assert_eq!(ylong_json_is_array(array), 1);
            ylong_json_delete(array);

            let null = ylong_json_create_null();
            assert_eq!(ylong_json_is_array(null), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_array_size`.
    ///
    /// # Title
    /// ut_ylong_json_get_array_size
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_array_size` to get size of the array.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_get_array_size() {
        unsafe {
            // Null ptr
            let array = null_mut();
            let mut len = 0i32;
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 0);

            // Null ptr
            let array = ylong_json_create_array();
            let len = null_mut();
            assert_eq!(ylong_json_get_array_size(array, len), 0);
            ylong_json_delete(array);

            let array = ylong_json_create_array();
            let mut len = 1i32;
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 0);
            ylong_json_delete(array);

            let null = ylong_json_create_null();
            let mut len = 1i32;
            assert_eq!(ylong_json_get_array_size(null, &mut len as *mut c_int), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_array_item`.
    ///
    /// # Title
    /// ut_ylong_json_get_array_item
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_array_item` to get an item of the array.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_get_array_item() {
        unsafe {
            // Null ptr
            let array = null_mut();
            assert!(ylong_json_get_array_item(array, 0).is_null());

            const TEXT: &str = "[null, 1.0, true, \"Test\"]";
            let str = str_to_c_char(TEXT);
            let mut msg = null_mut();
            let array = ylong_json_parse(str, &mut msg as *mut *mut c_char);
            let _ = Box::from_raw(str);
            assert_eq!(ylong_json_is_array(array), 1);

            let item0 = ylong_json_get_array_item(array, 0);
            assert_eq!(ylong_json_is_null(item0), 1);

            let item1 = ylong_json_get_array_item(array, 1);
            assert_eq!(ylong_json_is_double_number(item1), 1);
            let mut number = 0f64;
            assert_eq!(
                ylong_json_get_double_value_from_number(item1, &mut number as *mut c_double),
                1
            );
            assert_eq!(number, 1.0);

            let item2 = ylong_json_get_array_item(array, 2);
            assert_eq!(ylong_json_is_bool(item2), 1);
            let mut bool = 0i32;
            assert_eq!(
                ylong_json_get_value_from_bool(item2, &mut bool as *mut c_int),
                1
            );
            assert_eq!(bool, 1i32);

            let item3 = ylong_json_get_array_item(array, 3);
            assert_eq!(ylong_json_is_string(item3), 1);
            let mut content = null_mut();
            assert_eq!(
                ylong_json_get_value_from_string(item3, &mut content as *mut *mut c_char),
                1
            );
            let result = String::from_utf8_unchecked(CStr::from_ptr(content).to_bytes().to_vec());
            assert_eq!(result, "Test");

            assert!(ylong_json_get_array_item(array, 4).is_null());

            ylong_json_delete(array);

            let null = ylong_json_create_null();
            assert!(ylong_json_get_array_item(null, 0).is_null());
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_add_item_to_array`.
    ///
    /// # Title
    /// ut_ylong_json_add_item_to_array
    ///
    /// # Brief
    /// 1. Calls `ylong_json_add_item_to_array` to add an item to the array.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_add_item_to_array() {
        unsafe {
            // Null ptr
            let array = null_mut();
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_array(array, item), 0);
            ylong_json_delete(item);

            let array = ylong_json_create_array();
            let item = null_mut();
            assert_eq!(ylong_json_add_item_to_array(array, item), 0);
            ylong_json_delete(array);

            let array = ylong_json_create_array();
            let null = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_array(array, null), 1);
            let result = ylong_json_print_unformatted(array);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "[null]");
            ylong_json_delete(array);

            let null = ylong_json_create_null();
            let null2 = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_array(null, null2), 0);
            ylong_json_delete(null);
            ylong_json_delete(null2);
        }
    }

    /// UT test for `ylong_json_replace_item_in_array`.
    ///
    /// # Title
    /// ut_ylong_json_replace_item_in_array
    ///
    /// # Brief
    /// 1. Calls `ylong_json_replace_item_in_array` to replace an item in the array.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_replace_item_in_array() {
        unsafe {
            // Null ptr
            let array = null_mut();
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_replace_array_item_by_index(array, 0, item), 0);
            ylong_json_delete(item);

            // Null ptr
            let array = ylong_json_create_array();
            let item = null_mut();
            assert_eq!(ylong_json_replace_array_item_by_index(array, 0, item), 0);
            ylong_json_delete(array);

            let array = ylong_json_create_array();
            let null = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_array(array, null), 1);
            let result = ylong_json_print_unformatted(array);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "[null]");
            let replace = ylong_json_create_bool(1);
            assert_eq!(ylong_json_replace_array_item_by_index(array, 0, replace), 1);
            let result = ylong_json_print_unformatted(array);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "[true]");
            ylong_json_delete(array);

            let null = ylong_json_create_null();
            let null2 = ylong_json_create_null();
            assert_eq!(ylong_json_replace_array_item_by_index(null, 0, null2), 0);
            ylong_json_delete(null);
            ylong_json_delete(null2);
        }
    }

    /// UT test for `ylong_json_remove_array_item_by_index`.
    ///
    /// # Title
    /// ut_ylong_json_remove_array_item_by_index
    ///
    /// # Brief
    /// 1. Calls `ylong_json_remove_array_item_by_index` to remove an item in the array by index.
    /// (Uses the method 'remove' of Array.)
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_remove_array_item_by_index() {
        unsafe {
            // Null ptr
            let array = null_mut();
            assert!(ylong_json_remove_array_item_by_index(array, 0).is_null());

            const TEXT: &str = "[null, 1.0, true, \"Test\"]";
            let str = str_to_c_char(TEXT);
            let mut msg = null_mut();
            let array = ylong_json_parse(str, &mut msg as *mut *mut c_char);
            let _ = Box::from_raw(str);
            let mut len = 0i32;
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 4);

            let item0 = ylong_json_remove_array_item_by_index(array, 0);
            assert_eq!(ylong_json_is_null(item0), 1);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 3);
            ylong_json_delete(item0);

            let item3 = ylong_json_remove_array_item_by_index(array, 2);
            assert_eq!(ylong_json_is_string(item3), 1);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 2);
            ylong_json_delete(item3);

            let item1 = ylong_json_remove_array_item_by_index(array, 0);
            assert_eq!(ylong_json_is_number(item1), 1);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 1);
            ylong_json_delete(item1);

            let item2 = ylong_json_remove_array_item_by_index(array, 0);
            assert_eq!(ylong_json_is_bool(item2), 1);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 0);
            ylong_json_delete(item2);
            ylong_json_delete(array);

            let null = ylong_json_create_null();
            let item = ylong_json_remove_array_item_by_index(null, 0);
            assert!(item.is_null());
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_delete_array_item_by_index`.
    ///
    /// # Title
    /// ut_ylong_json_delete_array_item_by_index
    ///
    /// # Brief
    /// 1. Calls `ylong_json_delete_array_item_by_index` to delete an item in the array by index.
    /// (Uses the method 'remove' of underlying data structure.)
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_delete_array_item_by_index() {
        unsafe {
            // Null ptr scene, if the process does not exit abnormally, it is successful.
            let array = null_mut();
            ylong_json_delete_array_item_by_index(array, 0);

            const TEXT: &str = "[null, 1.0, true, \"Test\"]";
            let str = str_to_c_char(TEXT);
            let mut msg = null_mut();
            let array = ylong_json_parse(str, &mut msg as *mut *mut c_char);
            let mut len = 0i32;
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 4);

            ylong_json_delete_array_item_by_index(array, 0);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 3);

            ylong_json_delete_array_item_by_index(array, 0);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 2);

            ylong_json_delete_array_item_by_index(array, 0);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 1);

            ylong_json_delete_array_item_by_index(array, 0);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 0);

            let _ = Box::from_raw(str);
            ylong_json_delete(array);

            let null = ylong_json_create_null();
            ylong_json_delete_array_item_by_index(null, 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_array_node`.
    ///
    /// # Title
    /// ut_ylong_json_get_array_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_array_node` to get an array node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_ylong_json_get_array_node() {
        unsafe {
            // Null ptr
            let array = null_mut();
            assert!(ylong_json_get_array_node(array, 0).is_null());

            const TEXT: &str = "[null, 1.0, true, \"Test\"]";
            let str = str_to_c_char(TEXT);
            let mut msg = null_mut();
            let array = ylong_json_parse(str, &mut msg as *mut *mut c_char);
            let _ = Box::from_raw(str);
            assert_eq!(ylong_json_is_array(array), 1);

            let node0 = ylong_json_get_array_node(array, 0);
            assert!(!node0.is_null());

            let node1 = ylong_json_get_array_node(array, 1);
            assert!(!node1.is_null());

            let node2 = ylong_json_get_array_node(array, 2);
            assert!(!node2.is_null());

            let node3 = ylong_json_get_array_node(array, 3);
            assert!(!node3.is_null());

            let node4 = ylong_json_get_array_node(array, 4);
            assert!(node4.is_null());

            ylong_json_delete(array);

            let null = ylong_json_create_null();
            assert!(ylong_json_get_array_node(null, 0).is_null());
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_item_from_array_node`.
    ///
    /// # Title
    /// ut_ylong_json_get_item_from_array_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_item_from_array_node` to get the item of an array node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_ylong_json_get_item_from_array_node() {
        unsafe {
            // Null ptr
            let node = null_mut();
            assert!(ylong_json_get_array_node(node, 0).is_null());

            const TEXT: &str = "[null, 1.0, true, \"Test\"]";
            let str = str_to_c_char(TEXT);
            let mut msg = null_mut();
            let array = ylong_json_parse(str, &mut msg as *mut *mut c_char);
            let _ = Box::from_raw(str);
            assert_eq!(ylong_json_is_array(array), 1);

            let node0 = ylong_json_get_array_node(array, 0);
            assert!(!node0.is_null());
            let item0 = ylong_json_get_item_from_array_node(node0);
            assert_eq!(ylong_json_is_null(item0), 1);

            let node1 = ylong_json_get_array_node(array, 1);
            assert!(!node1.is_null());
            let item1 = ylong_json_get_item_from_array_node(node1);
            let mut number = 0f64;
            assert_eq!(
                ylong_json_get_double_value_from_number(item1, &mut number as *mut c_double),
                1
            );
            assert_eq!(number, 1.0);

            let node2 = ylong_json_get_array_node(array, 2);
            assert!(!node2.is_null());
            let item2 = ylong_json_get_item_from_array_node(node2);
            let mut bool = 0i32;
            assert_eq!(
                ylong_json_get_value_from_bool(item2, &mut bool as *mut c_int),
                1
            );
            assert_eq!(bool, 1i32);

            let node3 = ylong_json_get_array_node(array, 3);
            assert!(!node3.is_null());
            let item3 = ylong_json_get_item_from_array_node(node3);
            let mut content = null_mut();
            assert_eq!(
                ylong_json_get_value_from_string(item3, &mut content as *mut *mut c_char),
                1
            );
            let result = String::from_utf8_unchecked(CStr::from_ptr(content).to_bytes().to_vec());
            assert_eq!(result, "Test");

            ylong_json_delete(array);
        }
    }

    /// UT test for `ylong_json_add_item_to_array_then_get_node`.
    ///
    /// # Title
    /// ut_ylong_json_add_item_to_array_then_get_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_add_item_to_array_then_get_node` to add an item to array and get the corresponding node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_ylong_json_add_item_to_array_then_get_node() {
        unsafe {
            // Null ptr
            let array = null_mut();
            let item = ylong_json_create_null();
            assert!(ylong_json_add_item_to_array_then_get_node(array, item).is_null());
            ylong_json_delete(item);

            // Null ptr
            let array = ylong_json_create_array();
            let item = null_mut();
            assert!(ylong_json_add_item_to_array_then_get_node(array, item).is_null());
            ylong_json_delete(array);

            let array = ylong_json_create_array();
            let null = ylong_json_create_null();
            let node0 = ylong_json_add_item_to_array_then_get_node(array, null);
            assert!(!node0.is_null());
            let mut len = 0i32;
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 1);
            let item0 = ylong_json_get_item_from_array_node(node0);
            assert!(!item0.is_null());
            assert_eq!(ylong_json_is_null(item0), 1);
            let item0 = ylong_json_remove_array_node(node0);
            ylong_json_delete(item0);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 0);
            ylong_json_delete(array);
        }
    }

    /// UT test for `ylong_json_replace_item_of_array_node`.
    ///
    /// # Title
    /// ut_ylong_json_replace_item_of_array_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_replace_item_of_array_node` to replace the item of an array node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_ylong_json_replace_item_of_array_node() {
        unsafe {
            // Null ptr
            let node = null_mut();
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_replace_item_of_array_node(node, item), 0);
            ylong_json_delete(item);

            // Null ptr scene, if the process does not exit abnormally, it is successful.
            let array = ylong_json_create_array();
            let null = ylong_json_create_null();
            let node = ylong_json_add_item_to_array_then_get_node(array, null);
            let item = null_mut();
            assert_eq!(ylong_json_replace_item_of_array_node(node, item), 0);
            ylong_json_delete(array);

            let array = ylong_json_create_array();
            let null = ylong_json_create_null();
            let node = ylong_json_add_item_to_array_then_get_node(array, null);
            let result = ylong_json_print_unformatted(array);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "[null]");

            let bool = ylong_json_create_bool(1);
            assert_eq!(ylong_json_replace_item_of_array_node(node, bool), 1);
            let result = ylong_json_print_unformatted(array);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "[true]");

            ylong_json_delete(array);
        }
    }

    /// UT test for `ylong_json_remove_array_node`.
    ///
    /// # Title
    /// ut_ylong_json_remove_array_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_remove_array_node` to remove an array node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_ylong_json_remove_array_node() {
        unsafe {
            // Null ptr
            let node = null_mut();
            assert!(ylong_json_remove_array_node(node).is_null());

            const TEXT: &str = "[null, 1.0, true, \"Test\"]";
            let str = str_to_c_char(TEXT);
            let mut msg = null_mut();
            let array = ylong_json_parse(str, &mut msg as *mut *mut c_char);
            let _ = Box::from_raw(str);
            assert_eq!(ylong_json_is_array(array), 1);
            let mut len = 0i32;
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 4);

            let node0 = ylong_json_get_array_node(array, 0);
            assert!(!node0.is_null());
            let item0 = ylong_json_remove_array_node(node0);
            assert!(!item0.is_null());
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 3);
            assert_eq!(ylong_json_is_null(item0), 1);
            ylong_json_delete(item0);

            let node0 = ylong_json_get_array_node(array, 0);
            assert!(!node0.is_null());
            let item0 = ylong_json_remove_array_node(node0);
            assert!(!item0.is_null());
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 2);
            let mut number = 0f64;
            assert_eq!(
                ylong_json_get_double_value_from_number(item0, &mut number as *mut c_double),
                1
            );
            assert_eq!(number, 1.0);
            ylong_json_delete(item0);

            let node0 = ylong_json_get_array_node(array, 0);
            assert!(!node0.is_null());
            let item0 = ylong_json_remove_array_node(node0);
            assert!(!item0.is_null());
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 1);
            let mut bool = 0i32;
            assert_eq!(
                ylong_json_get_value_from_bool(item0, &mut bool as *mut c_int),
                1
            );
            assert_eq!(bool, 1i32);
            ylong_json_delete(item0);

            let node0 = ylong_json_get_array_node(array, 0);
            assert!(!node0.is_null());
            let item0 = ylong_json_remove_array_node(node0);
            assert!(!item0.is_null());
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 0);
            let mut content = null_mut();
            assert_eq!(
                ylong_json_get_value_from_string(item0, &mut content as *mut *mut c_char),
                1
            );
            let result = String::from_utf8_unchecked(CStr::from_ptr(content).to_bytes().to_vec());
            assert_eq!(result, "Test");
            ylong_json_delete(item0);

            ylong_json_delete(array);
        }
    }

    /// UT test for `ylong_json_delete_array_node`.
    ///
    /// # Title
    /// ut_ylong_json_delete_array_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_delete_array_node` to delete an array node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_ylong_json_delete_array_node() {
        unsafe {
            // Null ptr scene, if the process does not exit abnormally, it is successful.
            let node = null_mut();
            ylong_json_delete_array_node(node);

            const TEXT: &str = "[null, 1.0, true, \"Test\"]";
            let str = str_to_c_char(TEXT);
            let mut msg = null_mut();
            let array = ylong_json_parse(str, &mut msg as *mut *mut c_char);
            let _ = Box::from_raw(str);
            let mut len = 0i32;
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 4);

            let node0 = ylong_json_get_array_node(array, 0);
            ylong_json_delete_array_node(node0);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 3);

            let node1 = ylong_json_get_array_node(array, 0);
            ylong_json_delete_array_node(node1);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 2);

            let node2 = ylong_json_get_array_node(array, 0);
            ylong_json_delete_array_node(node2);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 1);

            let node3 = ylong_json_get_array_node(array, 0);
            ylong_json_delete_array_node(node3);
            assert_eq!(ylong_json_get_array_size(array, &mut len as *mut c_int), 1);
            assert_eq!(len, 0);

            ylong_json_delete(array);
        }
    }

    /// UT test for `ylong_json_create_object`.
    ///
    /// # Title
    /// ut_ylong_json_create_object
    ///
    /// # Brief
    /// 1. Calls `ylong_json_create_object` to create an object.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_create_object() {
        unsafe {
            let object = ylong_json_create_object();
            assert_eq!(ylong_json_is_object(object), 1);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{}");
            ylong_json_delete(object);
        }
    }

    /// UT test for `ylong_json_is_object`.
    ///
    /// # Title
    /// ut_ylong_json_is_object
    ///
    /// # Brief
    /// 1. Calls `ylong_json_is_object` to determine whether the value is object.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_is_object() {
        unsafe {
            // Null ptr
            let object = null_mut();
            assert_eq!(ylong_json_is_object(object), 0);

            let object = ylong_json_create_object();
            assert_eq!(ylong_json_is_object(object), 1);
            ylong_json_delete(object);

            let null = ylong_json_create_null();
            assert_eq!(ylong_json_is_object(null), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_object_size`.
    ///
    /// # Title
    /// ut_ylong_json_get_object_size
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_object_size` to get size of an object.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_get_object_size() {
        unsafe {
            // Null ptr
            let object = null_mut();
            let mut len = 0i32;
            assert_eq!(
                ylong_json_get_object_size(object, &mut len as *mut c_int),
                0
            );

            // Null ptr
            let object = ylong_json_create_object();
            let len = null_mut();
            assert_eq!(ylong_json_get_object_size(object, len), 0);
            ylong_json_delete(object);

            let object = ylong_json_create_object();
            let mut len = 1i32;
            assert_eq!(
                ylong_json_get_object_size(object, &mut len as *mut c_int),
                1
            );
            assert_eq!(len, 0);
            ylong_json_delete(object);

            let null = ylong_json_create_null();
            let mut len = 0i32;
            assert_eq!(ylong_json_get_object_size(null, &mut len as *mut c_int), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_has_object_item`.
    ///
    /// # Title
    /// ut_ylong_json_has_object_item
    ///
    /// # Brief
    /// 1. Calls `ylong_json_has_object_item` to determine whether the item exists in the object.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_has_object_item() {
        unsafe {
            // Null ptr
            let object = null_mut();
            let str = str_to_c_char("Hello World");
            assert_eq!(ylong_json_has_object_item(object, str), 0);
            let _ = Box::from_raw(str);

            // Null ptr
            let object = ylong_json_create_object();
            let str = null();
            assert_eq!(ylong_json_has_object_item(object, str), 0);
            ylong_json_delete(object);

            const TEXT: &str = "{\"null\":null}";
            let str = str_to_c_char(TEXT);
            let mut msg = null_mut();
            let object = ylong_json_parse(str, &mut msg as *mut *mut c_char);
            let _ = Box::from_raw(str);

            let str = str_to_c_char("null");
            assert_eq!(ylong_json_has_object_item(object, str), 1);
            let _ = Box::from_raw(str);

            let str = str_to_c_char("no_such_key");
            assert_eq!(ylong_json_has_object_item(object, str), 0);
            let _ = Box::from_raw(str);

            ylong_json_delete(object);

            let null = ylong_json_create_null();
            let str = str_to_c_char("Invalid");
            assert_eq!(ylong_json_has_object_item(null, str), 0);
            let _ = Box::from_raw(str);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_object_item`.
    ///
    /// # Title
    /// ut_ylong_json_get_object_item
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_object_item` to get an item in the object.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_get_object_item() {
        unsafe {
            // Null ptr
            let object = null_mut();
            let str = str_to_c_char("Hello World");
            assert!(ylong_json_get_object_item(object, str).is_null());
            let _ = Box::from_raw(str);

            // Null ptr
            let object = ylong_json_create_object();
            let str = null();
            assert!(ylong_json_get_object_item(object, str).is_null());
            ylong_json_delete(object);

            const TEXT: &str = "{\"null\":null}";
            let str = str_to_c_char(TEXT);
            let mut msg = null_mut();
            let object = ylong_json_parse(str, &mut msg as *mut *mut c_char);
            let _ = Box::from_raw(str);

            let str = str_to_c_char("null");
            let item = ylong_json_get_object_item(object, str);
            assert_eq!(ylong_json_is_null(item), 1);
            let _ = Box::from_raw(str);

            let str = str_to_c_char("no_such_key");
            let item = ylong_json_get_object_item(object, str);
            assert!(item.is_null());
            let _ = Box::from_raw(str);

            ylong_json_delete(object);

            let null = ylong_json_create_null();
            let str = str_to_c_char("Invalid");
            assert!(ylong_json_get_object_item(null, str).is_null());
            let _ = Box::from_raw(str);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_add_item_to_object`.
    ///
    /// # Title
    /// ut_ylong_json_add_item_to_object
    ///
    /// # Brief
    /// 1. Calls `ylong_json_add_item_to_object` to add an item to the object.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_add_item_to_object() {
        unsafe {
            // Null ptr
            let object = null_mut();
            let str = str_to_c_char("Hello World");
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_object(object, str, item), 0);
            let _ = Box::from_raw(str);
            ylong_json_delete(item);

            // Null ptr
            let object = ylong_json_create_object();
            let str = null();
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_object(object, str, item), 0);
            ylong_json_delete(object);
            ylong_json_delete(item);

            // Null ptr
            let object = ylong_json_create_object();
            let str = str_to_c_char("Hello World");
            let item = null_mut();
            assert_eq!(ylong_json_add_item_to_object(object, str, item), 0);
            ylong_json_delete(object);
            let _ = Box::from_raw(str);

            let object = ylong_json_create_object();
            let mut len = 0i32;
            assert_eq!(
                ylong_json_get_object_size(object, &mut len as *mut c_int),
                1
            );
            assert_eq!(len, 0);
            let str = str_to_c_char("Hello World");
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_object(object, str, item), 1);
            let _ = Box::from_raw(str);
            assert_eq!(
                ylong_json_get_object_size(object, &mut len as *mut c_int),
                1
            );
            assert_eq!(len, 1);
            ylong_json_delete(object);

            let null = ylong_json_create_null();
            let str = str_to_c_char("Hello World");
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_object(null, str, item), 0);
            ylong_json_delete(null);
            let _ = Box::from_raw(str);
            ylong_json_delete(item);
        }
    }

    /// UT test for `ylong_json_replace_object_item_by_index`.
    ///
    /// # Title
    /// ut_ylong_json_replace_object_item_by_index
    ///
    /// # Brief
    /// 1. Calls `ylong_json_replace_object_item_by_index` to replace an item in the object by index.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_replace_object_item_by_index() {
        unsafe {
            // Null ptr
            let object = null_mut();
            let str = str_to_c_char("Hello World");
            let item = ylong_json_create_null();
            assert_eq!(
                ylong_json_replace_object_item_by_index(object, str, item),
                0
            );
            let _ = Box::from_raw(str);
            ylong_json_delete(item);

            // Null ptr
            let object = ylong_json_create_object();
            let str = null();
            let item = ylong_json_create_null();
            assert_eq!(
                ylong_json_replace_object_item_by_index(object, str, item),
                0
            );
            ylong_json_delete(object);
            ylong_json_delete(item);

            // Null ptr
            let object = ylong_json_create_object();
            let str = str_to_c_char("Hello World");
            let item = null_mut();
            assert_eq!(
                ylong_json_replace_object_item_by_index(object, str, item),
                0
            );
            ylong_json_delete(object);
            let _ = Box::from_raw(str);

            let object = ylong_json_create_object();
            let str = str_to_c_char("Init");
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_object(object, str, item), 1);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{\"Init\":null}");
            let item = ylong_json_create_bool(1);
            assert_eq!(
                ylong_json_replace_object_item_by_index(object, str, item),
                1
            );
            let _ = Box::from_raw(str);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{\"Init\":true}");
            ylong_json_delete(object);

            let null = ylong_json_create_null();
            let str = str_to_c_char("Hello World");
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_replace_object_item_by_index(null, str, item), 0);
            ylong_json_delete(null);
            let _ = Box::from_raw(str);
            ylong_json_delete(item);
        }
    }

    /// UT test for `ylong_json_remove_object_item_by_index`.
    ///
    /// # Title
    /// ut_ylong_json_remove_object_item_by_index
    ///
    /// # Brief
    /// 1. Calls `ylong_json_remove_object_item_by_index` to remove an item in the object by index.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_remove_object_item_by_index() {
        unsafe {
            // Null ptr
            let object = null_mut();
            let str = str_to_c_char("Hello World");
            assert!(ylong_json_remove_object_item_by_index(object, str).is_null());
            let _ = Box::from_raw(str);

            // Null ptr
            let object = ylong_json_create_object();
            let str = null();
            assert!(ylong_json_remove_object_item_by_index(object, str).is_null());
            ylong_json_delete(object);

            let object = ylong_json_create_object();
            let str = str_to_c_char("Init");
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_object(object, str, item), 1);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{\"Init\":null}");
            let item = ylong_json_remove_object_item_by_index(object, str);
            assert!(!item.is_null());
            assert_eq!(ylong_json_is_null(item), 1);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{}");
            ylong_json_delete(object);
            let _ = Box::from_raw(str);
            ylong_json_delete(item);

            let null = ylong_json_create_null();
            let str = str_to_c_char("Hello World");
            assert!(ylong_json_remove_object_item_by_index(null, str).is_null());
            ylong_json_delete(null);
            let _ = Box::from_raw(str);
        }
    }

    /// UT test for `ylong_json_delete_object_by_index`.
    ///
    /// # Title
    /// ut_ylong_json_delete_object_by_index
    ///
    /// # Brief
    /// 1. Calls `ylong_json_delete_object_by_index` to delete an item in the object by index.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_delete_object_by_index() {
        unsafe {
            // Null ptr
            let object = null_mut();
            let str = str_to_c_char("Hello World");
            ylong_json_delete_object_item_by_index(object, str);
            let _ = Box::from_raw(str);

            // Null ptr
            let object = ylong_json_create_object();
            let str = null();
            ylong_json_delete_object_item_by_index(object, str);
            ylong_json_delete(object);

            let object = ylong_json_create_object();
            let str = str_to_c_char("Init");
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_add_item_to_object(object, str, item), 1);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{\"Init\":null}");
            ylong_json_delete_object_item_by_index(object, str);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{}");
            ylong_json_delete(object);
            let _ = Box::from_raw(str);

            let null = ylong_json_create_null();
            let str = str_to_c_char("Hello World");
            ylong_json_delete_object_item_by_index(null, str);
            ylong_json_delete(null);
            let _ = Box::from_raw(str);
        }
    }

    /// UT test for `ylong_json_get_all_object_items`.
    ///
    /// # Title
    /// ut_ylong_json_get_all_object_items
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_all_object_items` to get all items in the object.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_get_all_object_items() {
        unsafe {
            // Null ptr
            let object = null_mut();
            let mut len = 1i32;
            let keys = malloc(size_of::<*mut c_char>() * (len as usize)) as *mut *mut c_char;
            let values =
                malloc(size_of::<*mut YlongJson>() * (len as usize)) as *mut *mut YlongJson;
            assert_eq!(
                ylong_json_get_all_object_items(object, keys, values, &mut len as *mut c_int),
                0
            );
            free(keys as *mut c_void);
            free(values as *mut c_void);

            // Null ptr
            let object = ylong_json_create_object();
            let mut len = 1i32;
            let keys = null_mut();
            let values =
                malloc(size_of::<*mut YlongJson>() * (len as usize)) as *mut *mut YlongJson;
            assert_eq!(
                ylong_json_get_all_object_items(object, keys, values, &mut len as *mut c_int),
                0
            );
            ylong_json_delete(object);
            free(values as *mut c_void);

            // Null ptr
            let object = ylong_json_create_object();
            let mut len = 1i32;
            let keys = malloc(size_of::<*mut c_char>() * (len as usize)) as *mut *mut c_char;
            let values = null_mut();
            assert_eq!(
                ylong_json_get_all_object_items(object, keys, values, &mut len as *mut c_int),
                0
            );
            ylong_json_delete(object);
            free(keys as *mut c_void);

            // Null ptr
            let object = ylong_json_create_object();
            let len = 1i32;
            let keys = malloc(size_of::<*mut c_char>() * (len as usize)) as *mut *mut c_char;
            let values =
                malloc(size_of::<*mut YlongJson>() * (len as usize)) as *mut *mut YlongJson;
            let len = null_mut();
            assert_eq!(
                ylong_json_get_all_object_items(object, keys, values, len),
                0
            );
            ylong_json_delete(object);
            free(keys as *mut c_void);
            free(values as *mut c_void);

            const TEXT: &str = r#"{"A":null,"B":1.0,"C":true,"D":"Test"}"#;
            let text = str_to_c_char(TEXT);
            let mut err_msg = null_mut();
            let object = ylong_json_parse(text, &mut err_msg as *mut *mut c_char);
            let _ = Box::from_raw(text);
            let mut len = 0i32;
            assert_eq!(
                ylong_json_get_object_size(object, &mut len as *mut c_int),
                1
            );
            assert_eq!(len, 4);
            let keys = malloc(size_of::<*mut c_char>() * (len as usize)) as *mut *mut c_char;
            let values =
                malloc(size_of::<*mut YlongJson>() * (len as usize)) as *mut *mut YlongJson;
            assert_eq!(
                ylong_json_get_all_object_items(object, keys, values, &mut len as *mut c_int),
                1
            );
            let mut cnt = 0;
            let key_result = ["A", "B", "C", "D"];
            let value_result = ["null", "1.0", "true", "\"Test\""];
            while cnt != len {
                let key = *(keys.offset(cnt as isize));
                let key_str = CStr::from_ptr(key).to_str().unwrap();
                assert_eq!(key_str, key_result[cnt as usize]);
                ylong_json_free_string(key);

                let item = *(values.offset(cnt as isize));
                let value = ylong_json_print_unformatted(item);
                let value_str = CString::from_raw(value).into_string().unwrap();
                assert_eq!(value_str, value_result[cnt as usize]);
                cnt += 1;
            }
            free(keys as *mut c_void);
            free(values as *mut c_void);
            ylong_json_delete(object);

            // 非 object
            let null = ylong_json_create_null();
            let mut len = 1i32;
            let keys = malloc(size_of::<*mut c_char>() * (len as usize)) as *mut *mut c_char;
            let values =
                malloc(size_of::<*mut YlongJson>() * (len as usize)) as *mut *mut YlongJson;
            assert_eq!(
                ylong_json_get_all_object_items(null, keys, values, &mut len as *mut c_int),
                0
            );
            ylong_json_delete(null);
            free(keys as *mut c_void);
            free(values as *mut c_void);
        }
    }

    /// UT test for `ylong_json_for_each_object_item`.
    ///
    /// # Title
    /// ut_ylong_json_for_each_object_item
    ///
    /// # Brief
    /// 1. Calls `ylong_json_for_each_object_item` to do `func` for each item in the object.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_ylong_json_for_each_object_item() {
        unsafe {
            unsafe extern "C" fn func(target: *mut YlongJson) {
                ylong_json_set_int_value_to_number(target, 1000);
            }

            // Null ptr
            let object = null_mut();
            assert_eq!(ylong_json_for_each_object_item(object, func), 0);

            const TEXT: &str = r#"{"A":1,"B":2,"C":3,"D":null,"E":1.0}"#;
            let text = str_to_c_char(TEXT);
            let err_msg = null_mut();
            let object = ylong_json_parse(text, err_msg);
            let _ = Box::from_raw(text);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{\"A\":1,\"B\":2,\"C\":3,\"D\":null,\"E\":1.0}");
            assert_eq!(ylong_json_for_each_object_item(object, func), 1);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(
                result,
                "{\"A\":1000,\"B\":1000,\"C\":1000,\"D\":null,\"E\":1000}"
            );
            ylong_json_delete(object);

            let null = ylong_json_create_null();
            assert_eq!(ylong_json_for_each_object_item(null, func), 0);
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_object_node`.
    ///
    /// # Title
    /// ut_ylong_json_get_object_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_object_node` to get an object node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_object")]
    #[test]
    fn ut_ylong_json_get_object_node() {
        unsafe {
            // Null ptr
            let object = null_mut();
            let str = str_to_c_char("Hello World");
            let node = ylong_json_get_object_node(object, str);
            assert!(node.is_null());
            let _ = Box::from_raw(str);

            // Null ptr
            let object = ylong_json_create_object();
            let str = null_mut();
            let node = ylong_json_get_object_node(object, str);
            assert!(node.is_null());
            ylong_json_delete(object);

            const TEXT: &str = r#"{"null":null}"#;
            let text = str_to_c_char(TEXT);
            let err_msg = null_mut();
            let object = ylong_json_parse(text, err_msg);
            let _ = Box::from_raw(text);
            let str = str_to_c_char("null");
            let node = ylong_json_get_object_node(object, str);
            let _ = Box::from_raw(str);
            assert!(!node.is_null());
            let item = ylong_json_get_item_from_object_node(node);
            assert_eq!(ylong_json_is_null(item), 1);
            ylong_json_delete(object);

            // Non-object
            let null = ylong_json_create_null();
            let str = str_to_c_char("Hello World");
            let node = ylong_json_get_object_node(null, str);
            let _ = Box::from_raw(str);
            assert!(node.is_null());
            ylong_json_delete(null);
        }
    }

    /// UT test for `ylong_json_get_item_from_object_node`.
    ///
    /// # Title
    /// ut_ylong_json_get_item_from_object_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_get_item_from_object_node` to get the item of an object node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_object")]
    #[test]
    fn ut_ylong_json_get_item_from_object_node() {
        unsafe {
            // Null ptr
            let node = null_mut();
            let item = ylong_json_get_item_from_object_node(node);
            assert!(item.is_null());

            const TEXT: &str = r#"{"null":null}"#;
            let text = str_to_c_char(TEXT);
            let err_msg = null_mut();
            let object = ylong_json_parse(text, err_msg);
            let str = str_to_c_char("null");
            let node = ylong_json_get_object_node(object, str);
            assert!(!node.is_null());
            let item = ylong_json_get_item_from_object_node(node);
            assert_eq!(ylong_json_is_null(item), 1);
            let _ = Box::from_raw(text);
            let _ = Box::from_raw(str);
            ylong_json_delete(object);
        }
    }

    /// UT test for `ylong_json_add_item_to_object_then_get_node`.
    ///
    /// # Title
    /// ut_ylong_json_add_item_to_object_then_get_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_add_item_to_object_then_get_node` to add an item to the object and get the corresponding node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_object")]
    #[test]
    fn ut_ylong_json_add_item_to_object_then_get_node() {
        unsafe {
            // Null ptr
            let object = null_mut();
            let str = str_to_c_char("null");
            let item = ylong_json_create_null();
            let node = ylong_json_add_item_to_object_then_get_node(object, str, item);
            assert!(node.is_null());
            let _ = Box::from_raw(str);
            ylong_json_delete(item);

            // Null ptr
            let object = ylong_json_create_object();
            let str = null_mut();
            let item = ylong_json_create_null();
            let node = ylong_json_add_item_to_object_then_get_node(object, str, item);
            assert!(node.is_null());
            ylong_json_delete(object);
            ylong_json_delete(item);

            // Null ptr
            let object = ylong_json_create_object();
            let str = str_to_c_char("null");
            let item = null_mut();
            let node = ylong_json_add_item_to_object_then_get_node(object, str, item);
            assert!(node.is_null());
            ylong_json_delete(object);
            let _ = Box::from_raw(str);

            let object = ylong_json_create_object();
            let str = str_to_c_char("null");
            let item = ylong_json_create_null();
            let node = ylong_json_add_item_to_object_then_get_node(object, str, item);
            assert!(!node.is_null());
            let item = ylong_json_get_item_from_object_node(node);
            assert_eq!(ylong_json_is_null(item), 1);
            ylong_json_delete(object);
            let _ = Box::from_raw(str);

            let null = ylong_json_create_null();
            let str = str_to_c_char("null");
            let item = ylong_json_create_null();
            let node = ylong_json_add_item_to_object_then_get_node(null, str, item);
            assert!(node.is_null());
            ylong_json_delete(null);
            let _ = Box::from_raw(str);
            ylong_json_delete(item);
        }
    }

    /// UT test for `ylong_json_replace_item_of_object_node`.
    ///
    /// # Title
    /// ut_ylong_json_replace_item_of_object_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_replace_item_of_object_node` to replace the item of an object node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_object")]
    #[test]
    fn ut_ylong_json_replace_item_of_object_node() {
        unsafe {
            // Null ptr
            let node = null_mut();
            let item = ylong_json_create_null();
            assert_eq!(ylong_json_replace_item_of_object_node(node, item), 0);
            ylong_json_delete(item);

            // Null ptr
            let object = ylong_json_create_object();
            let str = str_to_c_char("null");
            let item = ylong_json_create_null();
            let node = ylong_json_add_item_to_object_then_get_node(object, str, item);
            let item = null_mut();
            assert_eq!(ylong_json_replace_item_of_object_node(node, item), 0);
            ylong_json_delete(object);
            let _ = Box::from_raw(str);

            let object = ylong_json_create_object();
            let str = str_to_c_char("null");
            let item = ylong_json_create_null();
            let node = ylong_json_add_item_to_object_then_get_node(object, str, item);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{\"null\":null}");
            let item = ylong_json_create_bool(1);
            assert_eq!(ylong_json_replace_item_of_object_node(node, item), 1);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{\"null\":true}");
            ylong_json_delete(object);
            let _ = Box::from_raw(str);
        }
    }

    /// UT test for `ylong_json_remove_object_node`.
    ///
    /// # Title
    /// ut_ylong_json_remove_object_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_remove_object_node` to remove an object node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_object")]
    #[test]
    fn ut_ylong_json_remove_object_node() {
        unsafe {
            // Null ptr
            let node = null_mut();
            let item = ylong_json_remove_object_node(node);
            assert!(item.is_null());

            let object = ylong_json_create_object();
            let str = str_to_c_char("null");
            let item = ylong_json_create_null();
            let node = ylong_json_add_item_to_object_then_get_node(object, str, item);
            let _ = Box::from_raw(str);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{\"null\":null}");
            let item = ylong_json_remove_object_node(node);
            assert_eq!(ylong_json_is_null(item), 1);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{}");
            ylong_json_delete(item);
            ylong_json_delete(object);
        }
    }

    /// UT test for `ylong_json_delete_object_node`.
    ///
    /// # Title
    /// ut_ylong_json_delete_object_node
    ///
    /// # Brief
    /// 1. Calls `ylong_json_delete_object_node` to delete an object node.
    /// 2. Checks if the test results are correct.
    #[cfg(feature = "list_object")]
    #[test]
    fn ut_ylong_json_delete_object_node() {
        unsafe {
            // Null ptr scene, the process is correct if it exits without exception.
            let node = null_mut();
            ylong_json_delete_object_node(node);

            let object = ylong_json_create_object();
            let str = str_to_c_char("null");
            let item = ylong_json_create_null();
            let node = ylong_json_add_item_to_object_then_get_node(object, str, item);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{\"null\":null}");
            ylong_json_delete_object_node(node);
            let result = ylong_json_print_unformatted(object);
            let result = CString::from_raw(result).into_string().unwrap();
            assert_eq!(result, "{}");
            let _ = Box::from_raw(str);
            ylong_json_delete(object);
        }
    }
}
