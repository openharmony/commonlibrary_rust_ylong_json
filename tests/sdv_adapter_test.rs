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

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int};
use std::ptr::*;
use ylong_json::*;

const RFC7159_EXAMPLE1: &str = r#"
{
    "Image": {
        "Width":  800,
        "Height": 600,
        "Title":  "View from 15th Floor",
        "Thumbnail": {
            "Url":    "http://www.example.com/image/481989943",
            "Height": 125,
            "Width":  100
        },
        "Animated" : false,
        "IDs": [116, 943, 234, 38793]
    }
}
"#;

macro_rules! test_json {
    ($json: expr) => {{
        let target = str_to_c_char("Image");
        let image = ylong_json_get_object_item($json, target);
        let _ = CString::from_raw(target);

        assert_eq!(ylong_json_is_object(image), 1);

        let target = str_to_c_char("Width");
        let width = ylong_json_get_object_item(image, target);
        let _ = CString::from_raw(target);
        let mut ptr: c_double = 0f64;
        assert_eq!(
            ylong_json_get_double_value_from_number(width, &mut ptr as *mut c_double),
            1
        );
        assert_eq!(ptr, 800f64);

        let target = str_to_c_char("Height");
        let height = ylong_json_get_object_item(image, target);
        let _ = CString::from_raw(target);
        let mut ptr: c_double = 0f64;
        assert_eq!(
            ylong_json_get_double_value_from_number(height, &mut ptr as *mut c_double),
            1
        );
        assert_eq!(ptr, 600f64);

        let target = str_to_c_char("Title");
        let title = ylong_json_get_object_item(image, target);
        let _ = CString::from_raw(target);
        let mut ptr: *mut c_char = null_mut::<c_char>();
        assert_eq!(
            ylong_json_get_value_from_string(title, &mut ptr as *mut *mut c_char),
            1
        );
        assert_eq!(
            CStr::from_ptr(ptr).to_str().unwrap(),
            "View from 15th Floor"
        );

        let target = str_to_c_char("Thumbnail");
        let thumbnail = ylong_json_get_object_item(image, target);
        let _ = CString::from_raw(target);
        assert_eq!(ylong_json_is_object(thumbnail), 1);

        let target = str_to_c_char("Url");
        let url = ylong_json_get_object_item(thumbnail, target);
        let _ = CString::from_raw(target);
        let mut ptr: *mut c_char = null_mut::<c_char>();
        assert_eq!(
            ylong_json_get_value_from_string(url, &mut ptr as *mut *mut c_char),
            1
        );
        assert_eq!(
            CStr::from_ptr(ptr).to_str().unwrap(),
            "http://www.example.com/image/481989943"
        );

        let target = str_to_c_char("Height");
        let height = ylong_json_get_object_item(thumbnail, target);
        let _ = CString::from_raw(target);
        let mut ptr: c_double = 0f64;
        assert_eq!(
            ylong_json_get_double_value_from_number(height, &mut ptr as *mut c_double),
            1
        );
        assert_eq!(ptr, 125f64);

        let target = str_to_c_char("Width");
        let width = ylong_json_get_object_item(thumbnail, target);
        let _ = CString::from_raw(target);
        let mut ptr: c_double = 0f64;
        assert_eq!(
            ylong_json_get_double_value_from_number(width, &mut ptr as *mut c_double),
            1
        );
        assert_eq!(ptr, 100f64);

        let target = str_to_c_char("Animated");
        let animated = ylong_json_get_object_item(image, target);
        let _ = CString::from_raw(target);
        let mut ptr: c_int = 0;
        assert_eq!(
            ylong_json_get_value_from_bool(animated, &mut ptr as *mut c_int),
            1
        );
        assert_eq!(ptr, 0);

        let target = str_to_c_char("IDs");
        let ids = ylong_json_get_object_item(image, target);
        let _ = CString::from_raw(target);

        assert_eq!(ylong_json_is_array(ids), 1);

        let item = ylong_json_get_array_item(ids, 0);
        let mut ptr = 0f64;
        assert_eq!(
            ylong_json_get_double_value_from_number(item, &mut ptr as *mut c_double),
            1
        );
        assert_eq!(ptr, 116f64);

        let item = ylong_json_get_array_item(ids, 1);
        let mut ptr = 0f64;
        assert_eq!(
            ylong_json_get_double_value_from_number(item, &mut ptr as *mut c_double),
            1
        );
        assert_eq!(ptr, 943f64);

        let item = ylong_json_get_array_item(ids, 2);
        let mut ptr = 0f64;
        assert_eq!(
            ylong_json_get_double_value_from_number(item, &mut ptr as *mut c_double),
            1
        );
        assert_eq!(ptr, 234f64);

        let item = ylong_json_get_array_item(ids, 3);
        let mut ptr = 0f64;
        assert_eq!(
            ylong_json_get_double_value_from_number(item, &mut ptr as *mut c_double),
            1
        );
        assert_eq!(ptr, 38793f64);
    }};
}

#[test]
fn sdv_adapter_test() {
    unsafe {
        sdv_adapter_parse_and_print();
        sdv_adapter_parse_memory_check();
    }
}

unsafe fn str_to_c_char(str: &str) -> *mut c_char {
    CString::from_vec_unchecked(str.as_bytes().to_vec()).into_raw()
}

unsafe fn sdv_adapter_parse_and_print() {
    let text = str_to_c_char(RFC7159_EXAMPLE1);

    let msg = null_mut::<c_char>();
    let mut json = Some(ylong_json_parse(
        text,
        &msg as *const *mut c_char as *mut *mut c_char,
    ));

    for _ in 0..1000 {
        let curr = json.take().unwrap();

        let curr_str = ylong_json_print_unformatted(curr);

        let msg = null_mut::<c_char>();
        let new = ylong_json_parse(curr_str, &msg as *const *mut c_char as *mut *mut c_char);

        let _ = CString::from_raw(curr_str);

        test_json!(new);

        json = Some(new);
        ylong_json_delete(curr);
    }

    ylong_json_delete(json.take().unwrap());

    // 析构 text
    let _ = CString::from_raw(text);
}

unsafe fn sdv_adapter_parse_memory_check() {
    const TEXT: &str = r#"
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

    let text = str_to_c_char(TEXT);
    let msg = null_mut::<c_char>();
    let json = ylong_json_parse(text, &msg as *const *mut c_char as *mut *mut c_char);
    let _ = Box::from_raw(text);

    assert!(msg.is_null());
    let result = ylong_json_print_unformatted(json);
    ylong_json_free_string(result);

    let duplicate = ylong_json_duplicate(json, 1);
    ylong_json_delete(duplicate);

    let null = ylong_json_create_null();
    ylong_json_delete(null);

    let index = str_to_c_char("string");
    let string = ylong_json_get_object_item(json, index);
    let content = null_mut::<c_char>();
    ylong_json_get_value_from_string(string, &content as *const *mut c_char as *mut *mut c_char);
    let _ = Box::from_raw(index);

    let null = ylong_json_create_null();
    let index = str_to_c_char("123");
    ylong_json_add_item_to_object(json, index, null);

    let content = str_to_c_char("aaaa");
    let string = ylong_json_create_string(content);
    ylong_json_replace_object_item_by_index(json, index, string);
    let _ = Box::from_raw(content);

    let removed = ylong_json_remove_object_item_by_index(json, index);
    ylong_json_delete(removed);
    let _ = Box::from_raw(index);

    extern "C" fn func(_value: *mut YlongJson) {}
    ylong_json_for_each_object_item(json, func);

    ylong_json_delete(json);
}
