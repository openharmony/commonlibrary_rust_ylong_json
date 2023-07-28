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

use serde::Deserialize;
use std::borrow::Cow;
use ylong_json::from_str;

#[derive(Deserialize, PartialEq, Debug)]
struct ExampleOne {
    #[serde(rename(deserialize = "Image"))]
    image: InnerImage,
}

#[derive(Deserialize, PartialEq, Debug)]
struct InnerImage {
    #[serde(rename(deserialize = "Width"))]
    width: u32,
    #[serde(rename(deserialize = "Height"))]
    height: u32,
    #[serde(rename(deserialize = "Title"))]
    title: String,
    #[serde(rename(deserialize = "Thumbnail"))]
    thumbnail: InnerThumbnail,
    #[serde(rename(deserialize = "Animated"))]
    animated: bool,
    #[serde(rename(deserialize = "IDs"))]
    ids: Vec<u32>,
}

#[derive(Deserialize, PartialEq, Debug)]
struct InnerThumbnail {
    #[serde(rename(deserialize = "Url"))]
    url: String,
    #[serde(rename(deserialize = "Height"))]
    height: u32,
    #[serde(rename(deserialize = "Width"))]
    width: u32,
}

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

#[test]
fn sdv_adapt_serde_example_one() {
    let de_res = from_str::<ExampleOne>(RFC7159_EXAMPLE1);
    let expected = ExampleOne {
        image: InnerImage {
            width: 800,
            height: 600,
            thumbnail: InnerThumbnail {
                url: String::from("http://www.example.com/image/481989943"),
                height: 125,
                width: 100,
            },
            animated: false,
            ids: vec![116, 943, 234, 38793],
            title: String::from("View from 15th Floor"),
        },
    };
    assert_eq!(expected, de_res.unwrap());
}

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

#[derive(Deserialize, PartialEq, Debug)]
struct ExampleTwo<'a> {
    precision: String,
    #[serde(rename(deserialize = "Latitude"))]
    latitude: f32,
    #[serde(rename(deserialize = "Longitude"))]
    longitude: f32,
    #[serde(rename(deserialize = "Address"))]
    address: String,
    #[serde(rename(deserialize = "City"))]
    city: String,
    #[serde(rename(deserialize = "State"))]
    state: String,
    #[serde(rename(deserialize = "Zip"))]
    zip: Cow<'a, str>,
    #[serde(rename(deserialize = "Country"))]
    country: Cow<'a, str>,
}

#[test]
fn sdv_adapt_serde_example_two() {
    let de_res = from_str::<Vec<ExampleTwo>>(RFC7159_EXAMPLE2).unwrap();
    let expected_0 = ExampleTwo {
        precision: String::from("zip"),
        latitude: 37.7668,
        longitude: -122.3959,
        address: String::from(""),
        city: String::from("SAN FRANCISCO"),
        state: String::from("CA"),
        zip: Cow::from("94107"),
        country: Cow::from("US"),
    };
    let expected_1 = ExampleTwo {
        precision: String::from("zip"),
        latitude: 37.371_991,
        longitude: -122.026_02,
        address: String::from(""),
        city: String::from("SUNNYVALE"),
        state: String::from("CA"),
        zip: Cow::from("94085"),
        country: Cow::from("US"),
    };
    assert_eq!(expected_0, de_res[0]);
    assert_eq!(expected_1, de_res[1]);
}

const JSON_PARSE_EXAMPLE3: &str = r#"
{
    "null_test" : {
        "null1": null
    },
    "bool_test" : {
        "boolean1": true,
        "boolean2": false
    },
    "number_test" : {
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
    "string_test" : { 
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
    "array_test" : {
        "array1": [  ],
        "array2": [ true,  false
                                     ],
        "array3": [0 , 1 , -1 , 100 , -100],
        "array4": [
                0.0                    ,               -0.0,           1.0           ,
        -1.0,                            12.34
                             ,        -12.34   ],
        "array5": [[[[[[["nest"]]]]]]]
    },
    "object_test" : {
        "object1": null, 
        "object2":   null   ,
        "object3": {"key1":"Null","key2": {"Bool":true},
        "key3":{"Number":0.0},"key4":{"StringVal":"string"},
        "key5":{"Array":[]},"key6":{"Object":null}},
        "object4": {
                "key1"                :                 "Null"   ,       "key2"
                   :         {   "Bool"   :   true   }     ,            "key3"         :
            {   "Number"   :   0.0   }      ,      "key4":{"StringVal":"string"}     ,
                       "key5":                {"Array":[]},          "key6":        {
                    "Object":null                  }
        },
        "object5": {"nest1": {"nest2": {"nest3": {"nest4": null}}}},
        "": {
            "": "key1",
            "\/\\\"\uCAFE\uBABE\uAB98\uFCDE\ubcda\uef4A\b\f\n\r\t`1~!@#$%^&*()_+-=[]{}|;:',./<>?" : "key2"
        }
    }
}
"#;

#[derive(Deserialize, PartialEq, Debug)]
struct ParseTest {
    null_test: NullTest,
    bool_test: BoolTest,
    number_test: NumberTest,
    string_test: StringTest,
    array_test: ArrayTest,
    object_test: ObjectTest,
}

#[derive(Deserialize, PartialEq, Debug)]
struct StructNull;

#[derive(Deserialize, PartialEq, Debug)]
struct NullTest {
    null1: StructNull,
}

#[derive(Deserialize, PartialEq, Debug)]
struct BoolTest {
    boolean1: bool,
    boolean2: bool,
}

#[derive(Deserialize, PartialEq, Debug)]
struct NumberTest {
    number1: u8,
    number2: i8,
    number3: u16,
    number4: i16,
    number5: f32,
    number6: f32,
    number7: f64,
    number8: f64,
    number9: f64,
    number10: f64,
    number11: f64,
    number12: f64,
    number13: f64,
    number14: f64,
    number15: f64,
    number16: f64,
    number17: f64,
}

#[derive(Deserialize, PartialEq, Debug)]
struct StringTest {
    string1: String,
    string2: String,
    string3: String,
    string4: String,
    string5: String,
    string6: String,
    string7: String,
    string8: String,
    string9: String,
}

#[derive(Deserialize, PartialEq, Debug)]
struct ArrayTest {
    array1: Vec<StructNull>,
    array2: Vec<bool>,
    array3: Vec<i64>,
    array4: Vec<f64>,
    #[warn(clippy::type_complexity)]
    array5: Vec<Vec<Vec<Vec<Vec<Vec<Vec<String>>>>>>>,
}

#[derive(Deserialize, PartialEq, Debug)]
struct ObjectTest {
    object1: StructNull,
    object2: StructNull,
    object3: ObjectThreeFour,
    object4: ObjectThreeFour,
    object5: ObjectFive,
    #[serde(rename(deserialize = ""))]
    object6: ObjectSix,
}

#[derive(Deserialize, PartialEq, Debug)]
enum InnerValue {
    Null,
    Bool(bool),
    Number(f64),
    StringVal(String),
    Array(Vec<InnerValue>),
    Object(StructNull),
}

#[derive(Deserialize, PartialEq, Debug)]
struct ObjectThreeFour {
    key1: InnerValue,
    key2: InnerValue,
    key3: InnerValue,
    key4: InnerValue,
    key5: InnerValue,
    key6: InnerValue,
}

#[derive(Deserialize, PartialEq, Debug)]
struct ObjectFive {
    nest1: NestOne,
}

#[derive(Deserialize, PartialEq, Debug)]
struct NestOne {
    nest2: NestTwo,
}

#[derive(Deserialize, PartialEq, Debug)]
struct NestTwo {
    nest3: NestThree,
}

#[derive(Deserialize, PartialEq, Debug)]
struct NestThree {
    nest4: NestFour,
}

#[derive(Deserialize, PartialEq, Debug)]
struct NestFour;

#[derive(Deserialize, PartialEq, Debug)]
struct ObjectSix {
    #[serde(rename(deserialize = ""))]
    mem1: String,
    #[serde(rename(
        deserialize = "/\\\"\u{CAFE}\u{BABE}\u{AB98}\u{FCDE}\u{bcda}\u{ef4A}\u{0008}\u{000c}\n\r\t`1~!@#$%^&*()_+-=[]{}|;:',./<>?"
    ))]
    mem2: String,
}

#[test]
fn sdv_adapt_serde_example_three() {
    let de_res = from_str::<ParseTest>(JSON_PARSE_EXAMPLE3);
    let expected = ParseTest {
        null_test: NullTest { null1: StructNull },
        bool_test: BoolTest {
            boolean1: true,
            boolean2: false,
        },
        number_test: NumberTest {
            number1: 0,
            number2: 0,
            number3: 123,
            number4: -123,
            number5: 123.456,
            number6: -123.456,
            number7: 1234560000.0,
            number8: 1.23456e-5,
            number9: 1234560000.0,
            number10: 1.23456e-5,
            number11: -1234560000.0,
            number12: -1.23456e-5,
            number13: -1234560000.0,
            number14: -1.23456e-5,
            number15: 0.0,
            number16: -0.0,
            number17: 300.0,
        },
        string_test: StringTest {
            string1: String::from(""),
            string2: String::from("Hello World"),
            string3: String::from("abcdefghijklmnopqrstuvwxyz"),
            string4: String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),

            string5: String::from("0123456789"),
            string6: String::from(" \u{8}\u{c}\n\r\t"),
            string7: String::from("\"\\/"),
            string8: String::from("`1~!@#$%^&*()_+-={':[,]}|;.</>?"),
            string9: String::from("ģ䕧覫췯ꯍ\u{ef4a}"),
        },
        array_test: ArrayTest {
            array1: vec![],
            array2: vec![true, false],
            array3: vec![0, 1, -1, 100, -100],
            array4: vec![0.0, -0.0, 1.0, -1.0, 12.34, -12.34],
            array5: vec![vec![vec![vec![vec![vec![vec![String::from("nest")]]]]]]],
        },
        object_test: ObjectTest {
            object1: StructNull,
            object2: StructNull,
            object3: ObjectThreeFour {
                key1: InnerValue::Null,
                key2: InnerValue::Bool(true),
                key3: InnerValue::Number(0.0),
                key4: InnerValue::StringVal(String::from("string")),
                key5: InnerValue::Array(vec![]),
                key6: InnerValue::Object(StructNull),
            },
            object4: ObjectThreeFour {
                key1: InnerValue::Null,
                key2: InnerValue::Bool(true),
                key3: InnerValue::Number(0.0),
                key4: InnerValue::StringVal(String::from("string")),
                key5: InnerValue::Array(vec![]),
                key6: InnerValue::Object(StructNull),
            },
            object5: ObjectFive {
                nest1: NestOne {
                    nest2: NestTwo {
                        nest3: NestThree { nest4: NestFour },
                    },
                },
            },
            object6: ObjectSix {
                mem1: String::from("key1"),
                mem2: String::from("key2"),
            },
        },
    };
    assert_eq!(expected, de_res.unwrap());
}
