# ylong_json User Guide

The `ylong_json` module provides serialization of text or string in JSON syntax format, and deserialization of corresponding generated instances. In addition, the `ylong_json` module also adapts to the third-party library `serde`. Structures that implement specific traits in the `serde` library can be serialized and deserialized.

If you need to see the detailed interface description, please check the docs of the corresponding interface. You can use `cargo doc --open` to generate and view the docs.

### Function 1: Generates a JSON instance
`ylong_json` provides the ability to generate an instance of `JsonValue` from JSON text or string. You need to use a series of instance creation methods for the "JsonValue" to use this feature.

(1) You can create a `JsonValue` instance by:
```rust
use std::fs::File;
use std::str::FromStr;
use std::io::Read;
use ylong_json::JsonValue;
fn create_json_value_instance() {
    let str: &str = "";
    // You can use `from_str` to try to generate a `JsonValue` instance from 
    // the &str type.
    // If the passed &str does not conform to JSON syntax, the corresponding 
    // Error will be returned.
    let json_value = JsonValue::from_str(str);
    
    let text: String = String::from("");
    // You can use `from_text` to generate a `JsonValue` instance from 
    // a series of types that implement AsRef<[u8]>.
    // If the passed text content does not conform to JSON syntax, the 
    // corresponding Error will be returned.
    let json_value = JsonValue::from_text(text);
    
    let path: &str = "";
    // You can use `from_file` to read a file from corresponding path and 
    // try to generate a `JsonValue` instance.
    // If the passed path is not valid or the text content does not conform 
    // to JSON syntax, the corresponding Error will be returned.
    let json_value = JsonValue::from_file(path);
    
    let mut reader: Box<dyn Read> = Box::new(File::open("").unwrap());
    // You can use `from_reader` interface to read text from an instance 
    // that implements io::Read and try to generate a `JsonValue` instance.
    // If the read fails or if the content from the reader does not conform
    // to JSON syntax, the corresponding Error will be returned.
    let json_value = JsonValue::from_reader(&mut reader);
}
```
Once the `JsonValue` instance has been successfully created, you can attempt to read and modify the corresponding contents.

(2) If the type in the JSON text implements the third-party library `serde::Deserialize` trait, you can directly deserialize the text content to an instance of that type.
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

    // You can use `from_str` to try to generate an instance from String.
    // If the passed String does not conform to JSON syntax, the corresponding
    // Error will be returned.
    let str = r#"{"int":1,"seq":["abcd","efgh"],"tup":[1,2,3]}"#;
    let example = from_str::<Example>(str).unwrap();

    // You can use `from_slice` to try to generate an instance from &u8.
    // If the passed &u8 does not conform to JSON syntax, the corresponding
    // Error will be returned.
    let slice = str.as_bytes();
    let example = from_slice::<Example>(slice).unwrap();

    // You can use `from_reader` to try to generate an instance from 
    // locations, files, io streams, and so on that implement io::Write.
    // If the passed text content does not conform to JSON syntax, 
    // the corresponding Error will be returned.
    let mut file: File = File::open("./example.txt").unwrap();
    let example = from_reader::<Example>(file).unwrap();
}
```

### Function 2: Reads and modifies a key-value pair
After a `JsonValue` instance is successfully generated, you can use a subscript to find the corresponding key-value pair (to obtain a common reference to the corresponding `JsonValue`).

A subscript of type &str or String can be used to find a key-value pair in Object; 
A Subscript of type usize can be used to find a key-value pair in an Array.
```rust
use std::str::FromStr;
use ylong_json::JsonValue;

// JSON string for the example
const JSON_TEXT: &str = r#"
{
    "key": "value",
    "array": [1, 2, 3]
}
"#;

fn find_key_value_pair() {
    // Creates a JsonValue instance from the example string, the syntax is 
    // correct so the parse must succeed here, so uses unwrap.
    let json_value = JsonValue::from_str(JSON_TEXT).unwrap();

    // Since json is itself a table, you can use the &str type to obtain
    // a common reference to the internal value.
    let value: &JsonValue = &json_value["key"];

    // You can use the &str type to obtain a common reference to the "array" member, and
    // then use the usize type to obtain a common reference to the corresponding element.
    let array_item: &JsonValue = &json_value["array"][0];

    // If you try to find a key that does not exist in a table, 
    // `&JsonValue::Null` will be returned.
    let no_such_key: &JsonValue = &json_value["no_such_key"];

    // When searching for the Array type, if the subscript exceeds the Array length, 
    // `&JsonValue::Null` will also be returned.
    let no_such_index: &JsonValue = &json_value["array"][100];

    // If you use a subscript to visit `JsonValue` types other than Object and Array, 
    // `&JsonValue::Null` will also be returned.
    let invalid_index: &JsonValue = &json_value["key"]["invalid"];
    let invalid_index: &JsonValue = &json_value["key"][0];
}
```
You can also use the same method to obtain a mutable reference to `JsonValue`. 
After obtaining the mutable reference, you can modify it, but you need to make sure that it conforms to JSON syntax.
```rust
use ylong_json::JsonValue;

// JSON string for the example
const JSON_TEXT: &str = r#"
{
    "key": "value",
    "array": [1, 2, 3]
}
"#;

fn modify_key_value_pair() {
    // Creates a JsonValue instance from the example string, the syntax is
    // correct so the parse must succeed here, so uses unwrap.
    // Here the JSON instance needs to be mutable because you need to obtain a mutable reference.
    let mut json_value = JsonValue::from_str(JSON_TEXT).unwrap();
    
    // Obtains a mutable reference to the member by "key" and set it to the number 123.
    // In the libraty, many primitive types implement conversion from themselves to JsonValue, 
    // so they can be converted to `JsonValue` by using `into()` method.
    // After executing this code, the contents of the table are as follows:
    // {
    //      "key": 123,
    //      "array": [1, 2, 3]
    // }
    json_value["key"] = 123_i32.into();
    
    // Obtains a mutable reference to the member by using "array" and the subscript 0,
    // and set it to the number 123.
    // After executing this code, the contents of the table are as follows:
    // {
    //      "key": 123,
    //      "array": [123, 2, 3]
    // }
    json_value["array"][0] = 123_i32.into();
   
    // If you try to obtain a mutable reference to a key that does not exist in the table, 
    // then the key will be inserted in the table with the corresponding value JsonValue::Null,
    // and changes the value baesd on that.
    // After executing this code, the json_value member "no_such_key" has been added, 
    // and the value is 123.
    // The contents of the table are as follows:
    // {
    //      "key": 123,
    //      "array": [123, 2, 3],
    //      "no_such_key": 123
    // }
    json_value["no_such_key"] = 123_i32.into();
    
    // When trying to obtain a mutable reference to a member of the Array type, if the 
    // subscript exceeds the Array length, then a `JsonValue::Null` will be added at 
    // the end of the Array and will return a mutable reference to that position.
    // After executing this code, the length of the array member of `json_value` becomes 4, 
    // and the value of the last member is 123.
    // The contents of the table are as follows:
    // {
    //      "key": 123,
    //      "array": [123, 2, 3, 123],
    //      "no_such_key": 123
    // }
    json_value["array"][100] = 123_i32.into();
    
    // When using a subscript of &str type or String type to obtain a mutable reference to
    // a non-Object type, will replace the value with an empty Object and then visit it with
    // that subscript.
    // After executing this code, the array member of `json_value` becomes of type Object 
    // and contains a key-value pair: "key" => 123.
    // The contents of the table are as follows:
    // {
    //      "key": 123,
    //      "array": {
    //          "key": 123
    //      },
    //      "no_such_key": 123
    // }
    json_value["array"]["key"] = 123_i32.into();
    
    // When using a subscript of usize type to obtain a mutable reference to a non-Array
    // type, will replace the value with an empty Array and then visit it with that subscript.
    // After executing this code, the key member of `json_value` becomes of type Array, 
    // and contains a member: key[0] => 123.
    // The contents of the table are as follows:
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

### Function 3: Outputs JSON text
(1) When you have a JsonValue instance, you can convert it to text and output it to a specified location: string, file, network, etc.
```rust
use std::fs::File;
use ylong_json::JsonValue;

fn output_json_text(json_value: JsonValue) {
    // Uses `to_compact_string()` to output the `json_value` as a string.
    let string = json_value.to_compact_string().unwrap();

    // Uses `compact_encode()` to output JSON text to a specified location,  
    // file, io stream, etc., which implements io::Write.
    let mut file: File = File::open("").unwrap();
    let _ = json_value.compact_encode(&mut file);
}
```
Because there is no strong order requirement for JSON internal elements, 
the output order of members will have a certain randomness, 
but it does not affect the semantics of JSON text.

(2) You can also serialize an instance of a type that implements the `serde::Serialize` trait to JSON text.
```rust
use std::fs::File;
use serde::Serialize;
use ylong_json::serializer_compact::{to_string, to_writer};

fn<V: Serialize> output_json_text(value: V) {
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

    // Uses `to_string()` to output the value as a string.
    let string = to_string(&example).unwrap();

    // Uses `to_writer()` to output JSON text to a specified location,
    // file, io stream, etc., which implements io::Write.
    let mut file: File = File::open("./example.txt").unwrap();
    let _ = to_writer(&example, &mut file);
}
```