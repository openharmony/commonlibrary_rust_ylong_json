# ylong_json

## 简介

`ylong_json` 模块提供了 JSON 语法格式文本或字符串的序列化功能，以及对应生成实例的反序列化功能。

`ylong_json` 包含以下核心功能：

### 功能一：生成 JSON 实例
`ylong_json` 提供了从 JSON 文本或字符串生成一个 `JsonValue` 实例的功能。

（1）可以通过以下方法创建 `JsonValue` 实例：
```rust
use std::fs::File;
use std::str::FromStr;
use std::io::Read;
use ylong_json::JsonValue;

fn create_json_value_instance() {
    let str: &str = "";
    // 可以使用 from_str 接口，从 &str 类型尝试生成 JsonValue 实例。
    // 如果传入的 &str 不符合 JSON 语法，会返回对应的 Error。
    let json_value = JsonValue::from_str(str);
    
    let text: String = String::from("");
    // 可以使用 from_text 接口，从一系列实现 AsRef<[u8]> 的类型生成 JsonValue 实例。
    // 如果传入的文本内容不符合 JSON 语法，会返回对应的 Error。
    let json_value = JsonValue::from_text(text);
    
    let path: &str = "";
    // 可以使用 from_file 接口，从对应路径的文件读取内容，并尝试生成 JsonValue 实例。
    // 如果传入的 path 不合法或者文本内容不符合 JSON 语法，会返回对应的 Error。
    let json_value = JsonValue::from_file(path);
    
    let mut reader: Box<dyn Read> = Box::new(File::open("").unwrap());
    // 可以使用 from_reader 接口，从实现了 io::Read 的实例中读取文本，并尝试生成 JsonValue 实例。
    // 如果读取失败或者从 reader 中读取的内容不符合 JSON 语法，会返回对应的 Error。
    let json_value = JsonValue::from_reader(&mut reader);
}
```
当 `JsonValue` 实例创建成功后，就可以尝试读取和修改对应的内容了。

（2）如果 JSON 文本中的类型实现了第三方库 `serde::Deserialize` trait，则可以直接将文本内容反序列化为该类型的实例。
```rust
use std::fs::File;
use serde::Deserialize;
use ylong_json::deserializer::{from_reader, from_slice, from_st};
fn deserialize_json_to_instance() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Example {
        int: u32,
        seq: Vec<String>,
        tup: (i32, i32, i32),
    }

    // 可以使用 from_str 接口，从 &str 类型生成实例。
    // 如果传入的 &str 不符合 JSON 语法，会返回对应的 Error。
    let tr = r#"{"int":1,"seq":["abcd","efgh"],"tup":[1,2,3]}"#;
    let example = from_str::<Example>(str).unwrap();

    // 可以使用 from_slice 接口，从 &u8 类型生成实例。
    // 如果传入的 &u8 不符合 JSON 语法，会返回对应的 Error。
    let slice = str.as_bytes();
    let example = from_slice::<Example>(slice).unwrap();

    
    // 可以使用 from_reader 接口，从实现了 io::Write 的位置、文件、io流等生成实例。
    // 如果传入的文本内容不符合 JSON 语法，会返回对应的 Error。
    let mut file: File = File::open("./example.txt").unwrap();
    let example = from_reader::<Example>(file).unwrap();
}
```

### 功能二：读取、修改键值对
`JsonValue` 实例生成成功后，可以通过各种下标来查找对应的键值对（获取到对应 `JsonValue` 的普通引用）。

&str 和 String 类型的下标可以用于查找 Object 内的键值对；usize 类型的下标可以用于查找 Array 内的键值对。
```rust
use std::str::FromStr;
use ylong_json::JsonValue;

// 示例的 JSON 字符串
const JSON_TEXT: &str = r#"
{
    "key": "value",
    "array": [1, 2, 3]
}
"#;

fn find_key_value_pair() {
    // 根据示例字符串创建 JsonValue 实例，语法正确所以此处解析必定成功，使用 unwrap。
    let json_value = JsonValue::from_str(JSON_TEXT).unwrap();

    // 由于 json 本身也是一个表，所以可以使用 &str 类型获取内部值的普通引用。
    let value: &JsonValue = &json_value["key"];

    // 可以通过 &str 类型先获取到 “array” 成员的普通引用，再根据 usize 类型获取对应元素的普通引用。
    let array_item: &JsonValue = &json_value["array"][0];

    // 如果尝试查找一个不存在表中的键，会返回 &JsonValue::Null。
    let no_such_key: &JsonValue = &json_value["no_such_key"];

    // 对 Array 类型查找时，若下标超过 Array 长度，也会返回 &JsonValue::Null。
    let no_such_index: &JsonValue = &json_value["array"][100];

    // 对一个 Object 和 Array 类型以外的 JsonValue 类型使用下标访问也会返回 &JsonValue::Null。
    let invalid_index: &JsonValue = &json_value["key"]["invalid"];
    let invalid_index: &JsonValue = &json_value["key"][0];
}
```
也可以通过相同方法获取到对应 `JsonValue` 的可变引用，获取到可变引用后可以对其进行修改，修改时需要注意符合 JSON 语法。
```rust
use ylong_json::JsonValue;

// 示例的 JSON 字符串
const JSON_TEXT: &str = r#"
{
    "key": "value",
    "array": [1, 2, 3]
}
"#;

fn modify_key_value_pair() {
    // 根据示例字符串创建 JsonValue 实例，语法正确所以此处解析必定成功，使用 unwrap。
    // 此处由于需要获取可变引用，JSON 实例需要可变。
    let mut json_value = JsonValue::from_str(JSON_TEXT).unwrap();
    
    // 通过 “key” 获取到对应成员的可变引用，并将其设置为数值 123。
    // 库中给许多基本类型实现了从自身到 JsonValue 的转换，所以可以通过 into() 方法转换为 JsonValue。
    // 执行此句代码后，表中内容如下：
    // {
    //      "key": 123,
    //      "array": [1, 2, 3]
    // }
    json_value["key"] = 123_i32.into();
    
    // 通过 “array” 和下标 0 获取到对应成员的可变引用，并将其设置为数值 123。
    // 执行此句代码后，表中内容如下：
    // {
    //      "key": 123,
    //      "array": [123, 2, 3]
    // }
    json_value["array"][0] = 123_i32.into();
    
    // 如果尝试获取一个不存在表中的键的可变引用，会在表中插入该键且对应值为 JsonValue::Null，并在此基础上进行修改。
    // 执行此行代码后，json_value 中会增加一个成员 “no_such_key”，且值为数值 123。
    // 表中内容如下：
    // {
    //      "key": 123,
    //      "array": [123, 2, 3],
    //      "no_such_key": 123
    // }
    json_value["no_such_key"] = 123_i32.into();
    
    // 对 Array 类型的成员尝试获取可变引用时，若下标超过 Array 长度，
    // 会在 Array 末尾插入一个 JsonValue::Null，并返回该位置的可变引用。
    // 执行此行代码后，json_value 的 “array” 成员的长度变为 4。
    // 表中内容如下：
    // {
    //      "key": 123,
    //      "array": [123, 2, 3, 123],
    //      "no_such_key": 123
    // }
    json_value["array"][100] = 123_i32.into();
    
    // 对一个非 Object 类型使用 &str 类型或 String 下标获取可变引用时，
    // 会将该值替换为一个空 Object，然后再用此下标对其进行访问。
    // 执行此代码后，json_value 的 array 成员变成 Object 类型，且含有一个键值对：“key” => 123。
    // 表中内容如下：
    // {
    //      "key": 123,
    //      "array": {
    //          "key": 123
    //      },
    //      "no_such_key": 123
    // }
    json_value["array"]["key"] = 123_i32.into();
    
    // 对一个非 Array 类型使用 usize 类型下标获取可变引用时，
    // 会将该值替换成一个空 Array，然后再用此下标对其进行访问。
    // 执行此代码后，json_value 的 key 成员变成 Array 类型，且含有一个成员： key[0] => 123
    // 表中内容如下：
    // {
    //      "key": [123],
    //      "array": {
    //          "key": 123
    //      },
    //      "no_such_key": 123
    // }
    json_value["key"][0] = 123_i32.into();
}
```

### 功能三：输出 JSON 文本
（1）当拥有一个 `JsonValue` 实例时，可以将该 `JsonValue` 实例转化成文本并输出到指定位置：字符串、文件、网络等。
```rust
use std::fs::File;
use ylong_json::JsonValue;

fn output_json_text(json_value: JsonValue) {
    // 使用 to_compact_string() 接口将 json_value 输出成一个字符串。
    let string = json_value.to_compact_string().unwrap();

    // 使用 compact_encode() 接口将 JSON 文本输出到指定实现了 io::Write 的位置，文件、io流等。
    let mut file: File = File::open("").unwrap();
    let _ = json_value.compact_encode(&mut file);
}
```
由于 JSON 内部元素没有较强的顺序要求，所以成员的输出顺序会有一定随机性，但是不影响 JSON 文本的语义。

（2）可以将一个实现了第三方库 `serde::Serialize` trait 的类型实例序列化为 JSON 文本。 
```rust
use std::fs::File;
use serde::Serialize;
use ylong_json::serializer_compact::{to_string, to_writer};

fn output_json_text() {
    #[derive(Serialize)]
    struct Exmaple {
        int: u32,
        seq: Vec<&'static str>,
        tup: (i32, i32, i32),
    }

    let example = Example {
        int: 1,
        seq: vec!["a", "b"],
        tup: (1, 2, 3),
    };

    // 使用 to_string() 接口将 value 输出成一个字符串。
    let string = to_string(&example).unwrap();

    // 使用 to_writer() 接口将 JSON 文本输出到指定实现了 io::Write 的位置，文件、io流等。
    let mut file: File = File::open("./example.txt").unwrap();
    let _ = to_writer(&example, &mut file);
}
```

## 编译构建
若使用 GN 编译工具链, 在 BUILD.GN 的 deps 段下添加依赖。添加后使用 GN 进行编译和构建：

```gn 
deps += ["//example_path/ylong_json:lib"]
```

若使用 Cargo 编译工具链, 在 ```Cargo.toml``` 下添加依赖。添加后使用 ```cargo``` 进行编译和构建：

```toml
[dependencies]
ylong_json = { path = "/example_path/ylong_json" } # 请使用路径依赖
```

## 性能测试
```
1.测试环境
操作系统：Linux
架构：x86_64
字节序：小端
CPU 型号：Intel(R) Xeon(R) Gold 6278C CPU @ 2.60GHz
CPU 核心数：8
内存：16G

2.测试结果
| 序列化   | ylong_json      | serde_json      |
-----------------------------------------------
| null     | 150 ns/iter     | 175 ns/iter    |
| boolean  | 155 ns/iter     | 178 ns/iter    |
| number   | 309 ns/iter     | 291 ns/iter    |
| string   | 513 ns/iter     | 413 ns/iter    |
| array    | 998 ns/iter     | 1,075 ns/iter  |
| object   | 1,333 ns/iter   | 1,348 ns/iter  |
| example1 | 12,537 ns/iter  | 12,288 ns/iter |
| example2 | 23,754 ns/iter  | 21,936 ns/iter |
| example3 | 103,061 ns/iter | 97,247 ns/iter |
| example4 | 15,234 ns/iter  | 17,895 ns/iter |

| 反序列化  | ylong_json      | serde_json     |
-----------------------------------------------
| null     | 257 ns/iter     | 399 ns/iter    |
| boolean  | 260 ns/iter     | 400 ns/iter    |
| number   | 1,507 ns/iter   | 989 ns/iter    |
| string   | 414 ns/iter     | 610 ns/iter    |
| array    | 2,258 ns/iter   | 2,148 ns/iter  |
| object   | 810 ns/iter     | 1,386 ns/iter  |
| example1 | 10,191 ns/iter  | 10,227 ns/iter |
| example2 | 15,753 ns/iter  | 18,022 ns/iter |
| example3 | 55,910 ns/iter  | 59,717 ns/iter |
| example4 | 18,461 ns/iter  | 12,471 ns/iter |
```

## 目录

```
ylong_json
├─ examples                               # ylong_json 代码示例
├─ include                                # ylong_json.h
├─ src
│  ├─ value                               # Array, Object 类型定义和相关方法实现
│  ├─ adapter.rs                          # 适配 C 的接口实现
│  ├─ consts.rs                           # 一些常数与表格的定义
│  ├─ deserializer.rs                     # 适配 serde 的反序列化实现
│  ├─ encoder.rs                          # 为 JsonValue 类型序列化实现
│  ├─ error.rs                            # 错误类型定义，便于定位
│  ├─ link_list.rs                        # LinkedList 类型定义和相关方法实现
│  ├─ serializer_compact.rs               # 适配 serde 的序列化实现
│  ├─ states.rs                           # 为 JsonValue 类型反序列化实现
│  └─ value.rs                            # JsonValue 类型定义和相关方法实现
└─ tests                                  # 测试目录
```