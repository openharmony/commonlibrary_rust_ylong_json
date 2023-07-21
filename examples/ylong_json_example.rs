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

//! cargo build --example ylong_json_example
//! Simple use examples of serialization and deserialization of JsonValue.

use std::io::stdout;
use ylong_json::JsonValue;

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

fn main() {
    let value = JsonValue::from_text(JSON_TEXT).unwrap();
    let mut console = stdout();
    value.formatted_encode(&mut console).unwrap();
    value.compact_encode(&mut console).unwrap();
}
