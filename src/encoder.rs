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

use crate::{consts::*, Array, Error, JsonValue, Number, Object};
#[cfg(feature = "c_adapter")]
use std::ffi::CString;
use std::io::Write;

// todo: Considers extracting Encoder traits.

/// JSON encoder with additional formats, used to output JsonValue instances in JSON format to the specified location.
///
/// This encoder will add additional formatting control whitespace characters during encoding.
pub(crate) struct FormattedEncoder<'a, W: Write> {
    output: &'a mut W,
    /// The current number of nested layers
    tab: usize,
}

impl<'a, W: Write> FormattedEncoder<'a, W> {
    /// Creates
    pub(crate) fn new(output: &'a mut W) -> Self {
        Self { output, tab: 0 }
    }

    /// Encodes
    pub(crate) fn encode(&mut self, value: &JsonValue) -> Result<(), Error> {
        self.encode_value(value)?;
        self.output.write_all(LINE_FEED_STR)?;
        Ok(())
    }

    /// Encodes JsonValue
    fn encode_value(&mut self, value: &JsonValue) -> Result<(), Error> {
        match value {
            JsonValue::Null => self.encode_null(),
            JsonValue::Boolean(boolean) => self.encode_boolean(boolean),
            JsonValue::Number(number) => self.encode_number(number),
            JsonValue::String(string) => self.encode_string(string),
            JsonValue::Array(array) => self.encode_array(array),
            JsonValue::Object(object) => self.encode_object(object),
        }
    }

    /// Add tabs to improve readability.
    fn add_tab(&mut self) -> Result<(), Error> {
        for _ in 0..self.tab {
            self.output.write_all(FOUR_SPACES_STR)?;
        }
        Ok(())
    }

    /// Encodes Null
    fn encode_null(&mut self) -> Result<(), Error> {
        encode_null(self.output)
    }

    /// Encodes Boolean
    fn encode_boolean(&mut self, boolean: &bool) -> Result<(), Error> {
        encode_boolean(self.output, *boolean)
    }

    /// Encodes Number
    fn encode_number(&mut self, number: &Number) -> Result<(), Error> {
        encode_number(self.output, number)
    }

    /// Encodes Key
    fn encode_key(&mut self, key: &str) -> Result<(), Error> {
        encode_string(self.output, key)
    }

    /// Encodes String
    #[cfg(feature = "c_adapter")]
    fn encode_string(&mut self, string: &CString) -> Result<(), Error> {
        encode_string(self.output, unsafe {
            core::str::from_utf8_unchecked(string.as_bytes())
        })
    }

    /// Encodes String
    #[cfg(not(feature = "c_adapter"))]
    fn encode_string(&mut self, string: &str) -> Result<(), Error> {
        encode_string(self.output, string)
    }

    /// Encodes Array
    fn encode_array(&mut self, array: &Array) -> Result<(), Error> {
        // Check whether multiple lines are required. If array or object
        // exists in the array value, multiple lines are required.
        let mut multiple_line = false;
        for v in array.iter() {
            if v.is_array() | v.is_object() {
                multiple_line = true;
                break;
            }
        }

        self.output.write_all(LEFT_SQUARE_BRACKET_STR)?;
        if multiple_line {
            self.output.write_all(LINE_FEED_STR)?;
            self.tab += 1;
            self.add_tab()?;
            for (n, v) in array.iter().enumerate() {
                if n != 0 {
                    self.output.write_all(COMMA_STR)?;
                    self.output.write_all(LINE_FEED_STR)?;
                    self.add_tab()?;
                }
                self.encode_value(v)?;
            }
            self.output.write_all(LINE_FEED_STR)?;
            self.tab -= 1;
            self.add_tab()?;
        } else {
            for (n, v) in array.iter().enumerate() {
                if n != 0 {
                    self.output.write_all(COMMA_STR)?;
                    self.output.write_all(SPACE_STR)?;
                }
                self.encode_value(v)?;
            }
        }
        self.output.write_all(RIGHT_SQUARE_BRACKET_STR)?;
        Ok(())
    }

    /// Encodes Object
    fn encode_object(&mut self, object: &Object) -> Result<(), Error> {
        self.output.write_all(LEFT_CURLY_BRACKET_STR)?;
        self.tab += 1;
        for (u, (k, v)) in object.iter().enumerate() {
            if u != 0 {
                self.output.write_all(COMMA_STR)?;
            }
            self.output.write_all(LINE_FEED_STR)?;
            self.add_tab()?;
            self.encode_key(k)?;
            self.output.write_all(COLON_STR)?;
            self.output.write_all(SPACE_STR)?;
            self.encode_value(v)?;
        }
        self.tab -= 1;
        // Non-empty objects require additional newlines and tabs.
        if !object.is_empty() {
            self.output.write_all(LINE_FEED_STR)?;
            self.add_tab()?;
        }
        self.output.write_all(RIGHT_CURLY_BRACKET_STR)?;
        Ok(())
    }
}

/// JSON encoder that outputs no extra whitespace characters ,
/// used to output a JsonValue instance in JSON format to a specified location.
pub(crate) struct CompactEncoder<'a, W: Write> {
    output: &'a mut W,
}

impl<'a, W: Write> CompactEncoder<'a, W> {
    /// Creates
    pub(crate) fn new(output: &'a mut W) -> Self {
        Self { output }
    }

    /// Encodes
    pub(crate) fn encode(&mut self, value: &JsonValue) -> Result<(), Error> {
        self.encode_value(value)
    }

    /// Encodes JsonValue
    fn encode_value(&mut self, value: &JsonValue) -> Result<(), Error> {
        match value {
            JsonValue::Null => self.encode_null(),
            JsonValue::Boolean(boolean) => self.encode_boolean(boolean),
            JsonValue::Number(number) => self.encode_number(number),
            JsonValue::String(string) => self.encode_string(string),
            JsonValue::Array(array) => self.encode_array(array),
            JsonValue::Object(object) => self.encode_object(object),
        }
    }

    /// Encodes Null
    fn encode_null(&mut self) -> Result<(), Error> {
        encode_null(self.output)
    }

    /// Encodes Boolean
    fn encode_boolean(&mut self, boolean: &bool) -> Result<(), Error> {
        encode_boolean(self.output, *boolean)
    }

    /// Encodes Number
    fn encode_number(&mut self, number: &Number) -> Result<(), Error> {
        encode_number(self.output, number)
    }

    /// Encodes Key
    fn encode_key(&mut self, key: &str) -> Result<(), Error> {
        encode_string(self.output, key)
    }

    /// Encodes String
    #[cfg(feature = "c_adapter")]
    fn encode_string(&mut self, string: &CString) -> Result<(), Error> {
        encode_string(self.output, unsafe {
            std::str::from_utf8_unchecked(string.as_bytes())
        })
    }

    /// Encodes String
    #[cfg(not(feature = "c_adapter"))]
    fn encode_string(&mut self, string: &str) -> Result<(), Error> {
        encode_string(self.output, string)
    }

    /// Encodes Array
    fn encode_array(&mut self, array: &Array) -> Result<(), Error> {
        self.output.write_all(LEFT_SQUARE_BRACKET_STR)?;
        for (n, v) in array.iter().enumerate() {
            if n != 0 {
                self.output.write_all(COMMA_STR)?;
            }
            self.encode_value(v)?;
        }
        self.output.write_all(RIGHT_SQUARE_BRACKET_STR)?;
        Ok(())
    }

    /// Encodes Object
    fn encode_object(&mut self, object: &Object) -> Result<(), Error> {
        self.output.write_all(LEFT_CURLY_BRACKET_STR)?;
        for (u, (k, v)) in object.iter().enumerate() {
            if u != 0 {
                self.output.write_all(COMMA_STR)?;
            }
            self.encode_key(k)?;
            self.output.write_all(COLON_STR)?;
            self.encode_value(v)?;
        }
        self.output.write_all(RIGHT_CURLY_BRACKET_STR)?;
        Ok(())
    }
}

#[inline]
fn encode_null(writer: &mut dyn Write) -> Result<(), Error> {
    writer.write_all(NULL_STR)?;
    Ok(())
}

#[inline]
fn encode_boolean(writer: &mut dyn Write, boolean: bool) -> Result<(), Error> {
    if boolean {
        writer.write_all(TRUE_STR)?;
    } else {
        writer.write_all(FALSE_STR)?;
    }
    Ok(())
}

#[inline]
pub(crate) fn encode_number(writer: &mut dyn Write, number: &Number) -> Result<(), Error> {
    write!(writer, "{number}")?;
    Ok(())
}

#[inline]
fn encode_string(writer: &mut dyn Write, string: &str) -> Result<(), Error> {
    writer.write_all(QUOTATION_MARK_STR)?;
    encode_string_inner(writer, string)?;
    writer.write_all(QUOTATION_MARK_STR)?;
    Ok(())
}

#[cfg(feature = "ascii_only")]
pub(crate) fn encode_string_inner(writer: &mut dyn Write, string: &str) -> Result<(), Error> {
    let bytes = string.as_bytes();
    let len = bytes.len();
    let mut start = 0usize;

    for i in 0..len {
        let ch = &bytes[i];
        if ESCAPE[(*ch) as usize] {
            writer.write_all(&bytes[start..i])?;
            start = i + 1;

            match *ch {
                REVERSE_SOLIDUS => writer.write_all(JSON_REVERSE_SOLIDUS)?,
                QUOTATION_MARK => writer.write_all(JSON_QUOTATION_MARK)?,
                BS_UNICODE_U8 => writer.write_all(JSON_BS)?,
                FF_UNICODE_U8 => writer.write_all(JSON_FF)?,
                LF_UNICODE_U8 => writer.write_all(JSON_LF)?,
                CR_UNICODE_U8 => writer.write_all(JSON_CR)?,
                HT_UNICODE_U8 => writer.write_all(JSON_HT)?,
                x => write!(writer, "\\u{number:0>width$x}", number = x, width = 4)?,
            }
        }
    }
    if start != len {
        writer.write_all(&bytes[start..len])?;
    }

    Ok(())
}

#[cfg(not(feature = "ascii_only"))]
pub(crate) fn encode_string_inner(writer: &mut dyn Write, string: &str) -> Result<(), Error> {
    fn split_pattern(
        writer: &mut dyn Write,
        pattern: &mut &str,
        split_pos: &mut usize,
        ch: char,
    ) -> Result<(), Error> {
        let (l, r) = (*pattern).split_at(*split_pos);
        writer.write_all(l.as_bytes())?;
        *pattern = r;

        let (_, r) = (*pattern).split_at(ch.len_utf8());
        *pattern = r;
        *split_pos = 0;
        Ok(())
    }

    let mut pattern = string;
    let mut split_pos = 0usize;
    for ch in string.chars() {
        if ch.is_ascii() {
            match PRINT_MAP[ch as usize] {
                PrintMapItem::Other => {
                    split_pos += 1;
                    continue;
                }
                PrintMapItem::Special(x) => {
                    split_pattern(writer, &mut pattern, &mut split_pos, ch)?;
                    writer.write_all(x)?;
                }
                PrintMapItem::Control => {
                    split_pattern(writer, &mut pattern, &mut split_pos, ch)?;
                    let bytes = ch as u32;
                    write!(writer, "\\u{number:0>width$x}", number = bytes, width = 4)?;
                }
            }
            continue;
        }
        split_pattern(writer, &mut pattern, &mut split_pos, ch)?;
        let bytes = ch as u32;
        write!(writer, "\\u{number:0>width$x}", number = bytes, width = 4)?;
    }
    if split_pos != 0 {
        writer.write_all(pattern.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod ut_encoder {
    use crate::{CompactEncoder, FormattedEncoder, JsonValue};
    use std::io::Write;

    struct StringWriter {
        string: String,
    }

    impl StringWriter {
        fn new() -> Self {
            Self {
                string: String::new(),
            }
        }
    }

    impl Write for StringWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.string
                .push_str(unsafe { std::str::from_utf8_unchecked(buf) });
            self.flush()?;
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    macro_rules! encoder_test_case {
        ($encoder: ident, $input: expr, $output: expr $(,)?) => {
            let value = JsonValue::from_text($input).unwrap();
            let mut writer = StringWriter::new();
            let mut encoder = $encoder::new(&mut writer);
            assert!(encoder.encode(&value).is_ok());
            assert_eq!(writer.string, $output);
        };
    }

    /// UT test for `FormattedEncoder`.
    ///
    /// # Title
    /// ut_formatted_encoder
    ///
    /// # Brief
    /// 1. Creates a `JsonValue` called `json_value`.
    /// 2. Creates a `FormattedEncoder` called `encoder`.
    /// 3. Uses `encoder` to encode `json_value`.
    /// 4. Checks if the results are correct.
    #[test]
    fn ut_formatted_encoder() {
        encoder_test_case!(
            FormattedEncoder,
            "{\"null\":null}",
            "{\n    \"null\": null\n}\n",
        );

        encoder_test_case!(
            FormattedEncoder,
            "{\"true\":true}",
            "{\n    \"true\": true\n}\n",
        );

        encoder_test_case!(
            FormattedEncoder,
            "{\"false\":false}",
            "{\n    \"false\": false\n}\n",
        );

        encoder_test_case!(
            FormattedEncoder,
            "{\"number\":3.14}",
            "{\n    \"number\": 3.14\n}\n",
        );

        encoder_test_case!(
            FormattedEncoder,
            "{\"string\":\"HelloWorld\"}",
            "{\n    \"string\": \"HelloWorld\"\n}\n",
        );

        encoder_test_case!(
            FormattedEncoder,
            "{\"array\":[1, 2, 3]}",
            "{\n    \"array\": [1, 2, 3]\n}\n",
        );

        encoder_test_case!(
            FormattedEncoder,
            "{\"object\":{\"key1\":1}}",
            "{\n    \"object\": {\n        \"key1\": 1\n    }\n}\n",
        );
    }

    /// UT test for `CompactEncoder`.
    ///
    /// # Title
    /// ut_compact_encoder
    ///
    /// # Brief
    /// 1. Creates a `JsonValue` called `json_value`.
    /// 2. Creates a `Compact` called `encoder`.
    /// 3. Uses `encoder` to encode `json_value`.
    /// 4. Checks if the results are correct.
    #[test]
    fn ut_compact_encoder() {
        encoder_test_case!(CompactEncoder, "{\"null\":null}", "{\"null\":null}",);

        encoder_test_case!(CompactEncoder, "{\"true\":true}", "{\"true\":true}",);

        encoder_test_case!(CompactEncoder, "{\"false\":false}", "{\"false\":false}",);

        encoder_test_case!(CompactEncoder, "{\"number\":3.14}", "{\"number\":3.14}",);

        encoder_test_case!(
            CompactEncoder,
            "{\"string\":\"HelloWorld\"}",
            "{\"string\":\"HelloWorld\"}",
        );

        #[cfg(not(feature = "ascii_only"))]
        encoder_test_case!(
            CompactEncoder,
            "{\"string\":\"\\b\\t\\f\\n\\u0000\\u2764\"}",
            "{\"string\":\"\\b\\t\\f\\n\\u0000\\u2764\"}",
        );

        #[cfg(feature = "ascii_only")]
        encoder_test_case!(
            CompactEncoder,
            "{\"string\":\"\\b\\t\\f\\n\\u0000\\u2764\"}",
            "{\"string\":\"\\b\\t\\f\\n\\u0000\u{2764}\"}",
        );

        encoder_test_case!(CompactEncoder, "{\"array\":[1,2,3]}", "{\"array\":[1,2,3]}",);

        encoder_test_case!(
            CompactEncoder,
            "{\"object\":{\"key1\":1,\"key2\":2}}",
            "{\"object\":{\"key1\":1,\"key2\":2}}",
        );
    }
}
