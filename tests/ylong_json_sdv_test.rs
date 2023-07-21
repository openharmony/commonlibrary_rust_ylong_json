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

use ylong_json::{Array, JsonValue, Object};

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

const RFC7159_EXAMPLE2: &str = r#"
[
    {
       "precision": "zip",
       "Latitude":  37.7668,
       "Longitude": -122.3959,
       "Address":   "",
       "City":      "SAN FRANCISCO",
       "State":     "CA",
       "Zip":       "94107",
       "Country":   "US"
    },
    {
       "precision": "zip",
       "Latitude":  37.371991,
       "Longitude": -122.026020,
       "Address":   "",
       "City":      "SUNNYVALE",
       "State":     "CA",
       "Zip":       "94085",
       "Country":   "US"
    }
]
"#;

const JSON_PARSE_TEST: &str = r#"
[
    {
        "null1": null
    },
    {
        "boolean1": true,
        "boolean2": false
    },
    {
        "number1": 0,
        "number2": -0,
        "number3": 123,
        "number4": -123,
        "number5": 123.456,
        "number6": -123.456,
        "number7": 123.456e+7,
        "number8": 123.456e-7,
        "number9": 123.456E+7,
        "number10": 123.456E-7,
        "number11": -123.456e+7,
        "number12": -123.456e-7,
        "number13": -123.456E+7,
        "number14": -123.456E-7,
        "number15": 0.0,
        "number16": -0.0e+7,
        "number17": 3e2
    },
    {
        "string1": "",
        "string2": "Hello World",
        "string3": "abcdefghijklmnopqrstuvwxyz",
        "string4": "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        "string5": "0123456789",
        "string6": " \b\f\n\r\t",
        "string7": "\"\\\/",
        "string8": "`1~!@#$%^&*()_+-={':[,]}|;.</>?",
        "string9": "\u0123\u4567\u89AB\uCDEF\uabcd\uef4A"
    },
    {
        "array1": [],
        "array2": [
                                     ],
        "array3": [null,true,0.0,"string",[],{}],
        "array4": [
                null                    ,               true,           0.0           ,
        "string",                            []
                             ,        {}   ],
        "array5": [[[[[[["nest"]]]]]]]
    },
    {
        "object1": {},
        "object2": {
                                                   },
        "object3": {"key1":null,"key2":true,"key3":0.0,"key4":"string","key5":[],"key6":{}},
        "object4": {
                "key1"                :                 null   ,       "key2"
                           :                  true     ,            "key3"         :
         0.0      ,      "key4":"string"            ,
                       "key5":                [],          "key6":            {
                                       }
        },
        "object5": {"nest1": {"nest2": {"nest3": {"nest4": {}}}}}
    },
    {
        "": "key1",
        "\/\\\"\uCAFE\uBABE\uAB98\uFCDE\ubcda\uef4A\b\f\n\r\t`1~!@#$%^&*()_+-=[]{}|;:',./<>?" : "key2"
    },
    {
                                    "key_value1"
                            :                        "value"
                ,         "key_value2"     :           [
                      ]             ,                     "key_value3"                  :
                      {}
    }
]
"#;

macro_rules! rfc7159_example1_check {
    ($json: expr) => {
        assert_eq!($json["Image"]["Width"], 800.into());
        assert_eq!($json["Image"]["Height"], 600.into());
        assert_eq!($json["Image"]["Title"], "View from 15th Floor".into());
        assert_eq!(
            $json["Image"]["Thumbnail"]["Url"],
            "http://www.example.com/image/481989943".into()
        );
        assert_eq!($json["Image"]["Thumbnail"]["Height"], 125.into());
        assert_eq!($json["Image"]["Thumbnail"]["Width"], 100.into());
        assert_eq!($json["Image"]["Animated"], false.into());
        assert_eq!($json["Image"]["IDs"][0], 116.into());
        assert_eq!($json["Image"]["IDs"][1], 943.into());
        assert_eq!($json["Image"]["IDs"][2], 234.into());
        assert_eq!($json["Image"]["IDs"][3], 38793.into());
    };
}

macro_rules! rfc7159_example2_check {
    ($json: expr) => {
        assert_eq!($json[0]["precision"], "zip".into());
        assert_eq!($json[0]["Latitude"], 37.7668.into());
        assert_eq!($json[0]["Longitude"], (-122.3959).into());
        assert_eq!($json[0]["Address"], "".into());
        assert_eq!($json[0]["City"], "SAN FRANCISCO".into());
        assert_eq!($json[0]["State"], "CA".into());
        assert_eq!($json[0]["Zip"], "94107".into());
        assert_eq!($json[0]["Country"], "US".into());
        assert_eq!($json[1]["precision"], "zip".into());
        assert_eq!($json[1]["Latitude"], 37.371991.into());
        assert_eq!($json[1]["Longitude"], (-122.026020).into());
        assert_eq!($json[1]["Address"], "".into());
        assert_eq!($json[1]["City"], "SUNNYVALE".into());
        assert_eq!($json[1]["State"], "CA".into());
        assert_eq!($json[1]["Zip"], "94085".into());
        assert_eq!($json[1]["Country"], "US".into());
    };
}

macro_rules! json_parse_test_check {
    ($json: expr) => {
        assert_eq!($json[0]["null1"], JsonValue::new_null());
        assert_eq!($json[1]["boolean1"], true.into());
        assert_eq!($json[1]["boolean2"], false.into());
        assert_eq!($json[2]["number1"], 0.into());
        assert_eq!($json[2]["number2"], 0.into());
        assert_eq!($json[2]["number3"], 123.into());
        assert_eq!($json[2]["number4"], (-123).into());
        assert_eq!($json[2]["number5"], 123.456.into());
        assert_eq!($json[2]["number6"], (-123.456).into());
        assert_eq!($json[2]["number7"], 1234560000.into());
        assert_eq!($json[2]["number8"], 0.0000123456.into());
        assert_eq!($json[2]["number9"], 1234560000.into());
        assert_eq!($json[2]["number10"], 0.0000123456.into());
        assert_eq!($json[2]["number11"], (-1234560000).into());
        assert_eq!($json[2]["number12"], (-0.0000123456).into());
        assert_eq!($json[2]["number13"], (-1234560000).into());
        assert_eq!($json[2]["number14"], (-0.0000123456).into());
        assert_eq!($json[2]["number15"], 0.into());
        assert_eq!($json[2]["number16"], 0.into());
        assert_eq!($json[2]["number17"], 300.into());
        assert_eq!($json[3]["string1"], "".into());
        assert_eq!($json[3]["string2"], "Hello World".into());
        assert_eq!($json[3]["string3"], "abcdefghijklmnopqrstuvwxyz".into());
        assert_eq!($json[3]["string4"], "ABCDEFGHIJKLMNOPQRSTUVWXYZ".into());
        assert_eq!($json[3]["string5"], "0123456789".into());
        assert_eq!($json[3]["string6"], " \u{0008}\u{000c}\n\r\t".into());
        assert_eq!($json[3]["string7"], "\"\\/".into());
        assert_eq!($json[3]["string8"], "`1~!@#$%^&*()_+-={':[,]}|;.</>?".into());
        assert_eq!($json[3]["string9"], "\u{0123}\u{4567}\u{89AB}\u{CDEF}\u{abcd}\u{ef4A}".into());
        assert_eq!($json[4]["array1"], Array::new().into());
        assert_eq!($json[4]["array2"], Array::new().into());
        assert_eq!($json[4]["array3"][0], JsonValue::new_null());
        assert_eq!($json[4]["array3"][1], true.into());
        assert_eq!($json[4]["array3"][2], 0.into());
        assert_eq!($json[4]["array3"][3], "string".into());
        assert_eq!($json[4]["array3"][4], Array::new().into());
        assert_eq!($json[4]["array3"][5], Object::new().into());
        assert_eq!($json[4]["array4"][0], JsonValue::new_null());
        assert_eq!($json[4]["array4"][1], true.into());
        assert_eq!($json[4]["array4"][2], 0.into());
        assert_eq!($json[4]["array4"][3], "string".into());
        assert_eq!($json[4]["array4"][4], Array::new().into());
        assert_eq!($json[4]["array4"][5], Object::new().into());
        assert_eq!($json[4]["array5"][0][0][0][0][0][0][0], "nest".into());
        assert_eq!($json[5]["object1"], Object::new().into());
        assert_eq!($json[5]["object2"], Object::new().into());
        assert_eq!($json[5]["object3"]["key1"], JsonValue::new_null());
        assert_eq!($json[5]["object3"]["key2"], true.into());
        assert_eq!($json[5]["object3"]["key3"], 0.into());
        assert_eq!($json[5]["object3"]["key4"], "string".into());
        assert_eq!($json[5]["object3"]["key5"], Array::new().into());
        assert_eq!($json[5]["object3"]["key6"], Object::new().into());
        assert_eq!($json[5]["object4"]["key1"], JsonValue::new_null());
        assert_eq!($json[5]["object4"]["key2"], true.into());
        assert_eq!($json[5]["object4"]["key3"], 0.into());
        assert_eq!($json[5]["object4"]["key4"], "string".into());
        assert_eq!($json[5]["object4"]["key5"], Array::new().into());
        assert_eq!($json[5]["object4"]["key6"], Object::new().into());
        assert_eq!($json[5]["object5"]["nest1"]["nest2"]["nest3"]["nest4"], Object::new().into());
        assert_eq!($json[6][""], "key1".into());
        assert_eq!(
            $json[6]["/\\\"\u{CAFE}\u{BABE}\u{AB98}\u{FCDE}\u{bcda}\u{ef4A}\u{0008}\u{000c}\n\r\t`1~!@#$%^&*()_+-=[]{}|;:',./<>?"],
            "key2".into()
        );
        assert_eq!($json[7]["key_value1"], "value".into());
        assert_eq!($json[7]["key_value2"], Array::new().into());
        assert_eq!($json[7]["key_value3"], Object::new().into());
    }
}

/*
 * @title  ylong_json sdv 测试用例
 * @design 使用路径覆盖
 * @precon 无
 * @brief  1. 准备一个 json 文本
 *         2. 根据该文本创建一个 Json 实例
 *         3. 修改该实例中的值
 *         4. 以字符串形式输出到指定位置
 *         5. 校验输出结果
 * @expect 1. 得到预期输出的字符串。
 * @auto   是
 */
#[test]
fn sdv_ylong_json() {
    sdv_json_parse();
    sdv_json_modify();
    sdv_json_output();
}

fn sdv_json_parse() {
    // 测试 RFC7159 13. Examples 里的两个 json 文本。
    let json = JsonValue::from_text(RFC7159_EXAMPLE1).unwrap();
    rfc7159_example1_check!(json);

    let json = JsonValue::from_text(RFC7159_EXAMPLE2).unwrap();
    rfc7159_example2_check!(json);

    let json = JsonValue::from_text(JSON_PARSE_TEST).unwrap();
    json_parse_test_check!(json);
}

fn sdv_json_modify() {
    let json_text = "{}";
    let mut json = JsonValue::from_text(json_text).unwrap();
    // 初始时 json 为空。
    assert!(json.try_as_object().unwrap().is_empty());

    json["null"] = JsonValue::new_null();
    json["boolean"] = true.into();
    json["number"] = 123.into();
    json["string"] = "Hello World".into();
    json["array"] = Array::new().into();
    json["object"] = Object::new().into();

    assert!(json["null"].is_null());
    assert_eq!(json["boolean"], true.into());
    assert_eq!(json["number"], 123.into());
    assert_eq!(json["string"], "Hello World".into());
    assert_eq!(json["array"], Array::new().into());
    assert_eq!(json["object"], Object::new().into());
    assert_eq!(json.try_as_object().unwrap().len(), 6);

    json["array"][0] = 123.into();
    json["array"][1] = "string".into();
    json["array"][2] = JsonValue::new_null();

    assert_eq!(json["array"][0], 123.into());
    assert_eq!(json["array"][1], "string".into());
    assert_eq!(json["array"][2], JsonValue::new_null());
    assert_eq!(json["array"].try_as_array().unwrap().len(), 3);

    json["array"] = Array::new().into();
    assert_eq!(json["array"].try_as_array().unwrap().len(), 0);

    json["object"]["number"] = 123.into();
    json["object"]["string"] = "string".into();
    json["object"]["null"] = JsonValue::new_null();

    assert_eq!(json["object"]["number"], 123.into());
    assert_eq!(json["object"]["string"], "string".into());
    assert_eq!(json["object"]["null"], JsonValue::new_null());
    assert_eq!(json["object"].try_as_object().unwrap().len(), 3);

    json["object"] = Object::new().into();
    assert_eq!(json["object"].try_as_object().unwrap().len(), 0);
}

#[allow(unused_assignments)]
fn sdv_json_output() {
    const LOOPS_NUM: usize = 1000;

    let mut json = JsonValue::from_text(RFC7159_EXAMPLE1).unwrap();
    let mut vec = Vec::new();

    for _ in 0..LOOPS_NUM {
        // 将 json 内容写入 vec 中
        vec.clear();
        assert!(json.formatted_encode(&mut vec).is_ok());

        // 通过 vec 重新生成一个 json 实例
        let temp = JsonValue::from_text(&vec).unwrap();

        // 比较内容是否发生变化
        rfc7159_example1_check!(temp);

        json = temp;
    }

    let mut json = JsonValue::from_text(RFC7159_EXAMPLE2).unwrap();
    let mut vec = Vec::new();

    for _ in 0..LOOPS_NUM {
        // 将 json 内容写入 vec 中
        vec.clear();
        assert!(json.formatted_encode(&mut vec).is_ok());

        // 通过 vec 重新生成一个 json 实例
        let temp = JsonValue::from_text(&vec).unwrap();

        // 比较内容是否发生变化
        rfc7159_example2_check!(temp);

        json = temp;
    }

    let mut json = JsonValue::from_text(JSON_PARSE_TEST).unwrap();
    let mut vec = Vec::new();

    for _ in 0..LOOPS_NUM {
        // 将 json 内容写入 vec 中
        vec.clear();
        assert!(json.formatted_encode(&mut vec).is_ok());

        // 通过 vec 重新生成一个 json 实例
        let temp = JsonValue::from_text(&vec).unwrap();

        // 比较内容是否发生变化
        json_parse_test_check!(temp);

        json = temp;
    }
}
