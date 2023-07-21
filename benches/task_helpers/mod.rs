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

pub const LOOPS_NUM: usize = 10;

pub const NULL_EXAMPLE: &str = "null";
pub const BOOLEAN_EXAMPLE: &str = "false";
pub const NUMBER_EXAMPLE: &str = "12.34";
pub const STRING_EXAMPLE: &str = "\"Hello\"";
pub const ARRAY_EXAMPLE: &str = "[false,null,12.34]";
pub const OBJECT_EXAMPLE: &str = r#"{"key":"value"}"#;

pub const RFC7159_EXAMPLE1: &str = r#"
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

pub const RFC7159_EXAMPLE2: &str = r#"
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

pub const JSON_PARSE_TEST: &str = r#"
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

pub const LONG_KEY_VALUE: &str = r#"
{
    "long_key_value_object":{
    "-----LONG KEY-----uoTVt77ryiZ5GnfVXf6kEBJQS8hBMY2BMsyLyckIPrNEvknjp82jz9yatYV0S77uLb99nPR6WqSDPtrWzc1XHJVPLoIlxaDGKm4xB7KaFl95wdnYRvuyCEmrzdoZS1KtXyf31vYLD4r9BnFm6wBuefKvONcLNGi5bsZqq100MWmFXjQUYhd6nZDJWVTAtpF195PiyvoJiJxSkiwpallQCqTbcoZTMf5SJ7KH1umstVVPW6NvgRO5PwwHc2N7QytBvw":
    "-----LONG VALUE-----by4iUNvpmeZ5ypvznYm7DSiY6gEgRy64yFGHB6pSgMGVRvElAnrSXpaSC8Exa9aMbx4hGkStSKMSbsk2t8JVxDqBKQVo7NdJiSwQf2p5YxFIU5aS2y4gazdDHcwuo7pqrp47AuXfxC799qUDD4q6VWD9u49Nuy7DXLjrdgLz17cC3uCaMwSZK3wc6Lu0Mri6Di4M9NEe36WGBN1xcmcHvm8GH7XXGikuuZ432HG76DEek1s99jHTzQZEILiDQAB",

    "-----LONG KEY-----by4iUNvpmeZ5ypvznYm7DSiY6gEgRy64yFGHB6pSgMGVRvElAnrSXpaSC8Exa9aMbx4hGkStSKMSbsk2t8JVxDqBKQVo7NdJiSwQf2p5YxFIU5aS2y4gazdDHcwuo7pqrp47AuXfxC799qUDD4q6VWD9u49Nuy7DXLjrdgLz17cC3uCaMwSZK3wc6Lu0Mri6Di4M9NEe36WGBN1xcmcHvm8GH7XXGikuuZ432HG76DEek1s99jHTzQZEILiDQAB":
    "-----LONG VALUE-----uoTVt77ryiZ5GnfVXf6kEBJQS8hBMY2BMsyLyckIPrNEvknjp82jz9yatYV0S77uLb99nPR6WqSDPtrWzc1XHJVPLoIlxaDGKm4xB7KaFl95wdnYRvuyCEmrzdoZS1KtXyf31vYLD4r9BnFm6wBuefKvONcLNGi5bsZqq100MWmFXjQUYhd6nZDJWVTAtpF195PiyvoJiJxSkiwpallQCqTbcoZTMf5SJ7KH1umstVVPW6NvgRO5PwwHc2N7QytBvw"
    }
}
"#;
