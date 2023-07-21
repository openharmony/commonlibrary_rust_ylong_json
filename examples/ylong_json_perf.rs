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

//! A performance testing suite for ylong_json.
//!
//! This performance testing suite compares the speed of the `ylong_json` crate
//! with `serde_json` for parsing JSON text and converting JSON objects into
//! strings. The test is run multiple times as defined by `LOOPS_NUM`.
//!
//! Example JSON used in this test represents an image object with various properties.

use serde_json::Value;
use std::str::FromStr;
use std::time::Instant;
use ylong_json::JsonValue;

const LOOPS_NUM: usize = 10000;
const JSON_TEXT: &str = r#"
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

fn main() {
    let value = JsonValue::from_str(JSON_TEXT).unwrap();
    println!("{}", value.to_compact_string().unwrap());

    let st = Instant::now();
    for _ in 0..LOOPS_NUM {
        let value = JsonValue::from_str(JSON_TEXT).unwrap();
        let _ = value.to_compact_string();
    }
    let ed = Instant::now();
    println!(
        "ylong_json: {}ms",
        ed.duration_since(st).as_secs_f64() * 1000f64
    );

    let value: Value = serde_json::from_str(JSON_TEXT).unwrap();
    println!("{value}");

    let st = Instant::now();
    for _ in 0..LOOPS_NUM {
        let value: Value = serde_json::from_str(JSON_TEXT).unwrap();
        format!("{value}");
    }
    let ed = Instant::now();
    println!(
        "serde_json: {}ms",
        ed.duration_since(st).as_secs_f64() * 1000f64
    );
}
