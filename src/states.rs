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

use crate::reader::Cacheable;
use crate::{
    consts::*, deserializer::Deserializer, Array, Error, JsonValue, Number, Object, ParseError,
};
use core::convert::TryFrom;
#[cfg(feature = "c_adapter")]
pub use std::ffi::CString;

macro_rules! unexpected_character {
    ($deserializer: expr) => {{
        let position = $deserializer.reader.position();
        match read_error_char($deserializer) {
            Ok(Some(ch)) => {
                Err(ParseError::UnexpectedCharacter(position.line(), position.column(), ch).into())
            }
            Ok(None) => Err(ParseError::InvalidUtf8Bytes(position.line()).into()),
            Err(e) => Err(e),
        }
    }};
}

macro_rules! unexpected_eoj {
    ($deserializer: expr) => {
        Err(ParseError::UnexpectedEndOfJson($deserializer.reader.position().line()).into())
    };
}

macro_rules! eat_whitespace_until_not {
    ($deserializer: expr) => {{
        loop {
            match $deserializer.reader.peek().map_err(Error::new_reader)? {
                Some(ch) if WHITE_SPACE_SET.contains(&ch) => $deserializer.reader.discard(),
                x => break x,
            }
        }
    }};
}

macro_rules! eat_digits_until_not {
    ($deserializer: expr) => {{
        loop {
            match $deserializer.reader.peek().map_err(Error::new_reader)? {
                Some(ch) if (ZERO..=NINE).contains(&ch) => $deserializer.reader.discard(),
                x => break x,
            }
        }
    }};
}

macro_rules! match_str {
    ($deserializer: expr, $str: expr) => {{
        for item in $str {
            match $deserializer.reader.peek().map_err(Error::new_reader)? {
                Some(ch) if ch == *item => $deserializer.reader.discard(),
                Some(_) => return unexpected_character!($deserializer),
                None => return unexpected_eoj!($deserializer),
            }
        }
    }};
}

pub(crate) fn check_recursion<R: Cacheable>(
    deserializer: &mut Deserializer<R>,
) -> Result<(), Error> {
    if deserializer.recursion_depth > RECURSION_LIMIT {
        Err(Error::ExceedRecursionLimit)
    } else {
        Ok(())
    }
}

#[inline]
pub(crate) fn start_parsing<R: Cacheable>(
    deserializer: &mut Deserializer<R>,
) -> Result<JsonValue, Error> {
    let value = parse_value(deserializer)?;

    // If the text is not finished, return TrailingBytes Error.
    if eat_whitespace_until_not!(deserializer).is_some() {
        return Err(ParseError::TrailingBytes(deserializer.reader.position().line()).into());
    }
    Ok(value)
}

// Parses value.
fn parse_value<R: Cacheable>(deserializer: &mut Deserializer<R>) -> Result<JsonValue, Error> {
    match eat_whitespace_until_not!(deserializer) {
        Some(ZERO..=NINE | MINUS) => Ok(JsonValue::Number(parse_number(deserializer)?)),
        Some(LEFT_CURLY_BRACKET) => {
            deserializer.reader.discard();
            parse_object(deserializer)
        }
        Some(LEFT_SQUARE_BRACKET) => {
            deserializer.reader.discard();
            parse_array(deserializer)
        }
        Some(QUOTATION_MARK) => {
            deserializer.reader.discard();
            Ok(JsonValue::String(parse_string(deserializer)?))
        }
        Some(T_LOWER) => {
            deserializer.reader.discard();
            match_str!(deserializer, TRUE_LEFT_STR);
            Ok(JsonValue::Boolean(true))
        }
        Some(F_LOWER) => {
            deserializer.reader.discard();
            match_str!(deserializer, FALSE_LEFT_STR);
            Ok(JsonValue::Boolean(false))
        }
        Some(N_LOWER) => {
            deserializer.reader.discard();
            match_str!(deserializer, NULL_LEFT_STR);
            Ok(JsonValue::Null)
        }
        Some(_) => unexpected_character!(deserializer),
        None => unexpected_eoj!(deserializer),
    }
}

// Parses object
fn parse_object<R: Cacheable>(deserializer: &mut Deserializer<R>) -> Result<JsonValue, Error> {
    // Uses an internal state machine to determine the flow.
    enum InnerState {
        Start,      // State at the start of the match.
        AfterComma, // Comma already exists.
        NoComma,    // Comma didn't exist before
    }

    deserializer.recursion_depth += 1;
    check_recursion(deserializer)?;

    // Creates an Object to store key-value pairs.
    let mut object = Object::new();
    // The initial status is Start.
    let mut state = InnerState::Start;

    loop {
        match (state, eat_whitespace_until_not!(deserializer)) {
            // If "}" is encountered in the initial or NoComma state, object is null.
            (InnerState::Start | InnerState::NoComma, Some(RIGHT_CURLY_BRACKET)) => {
                deserializer.reader.discard();
                deserializer.recursion_depth -= 1;
                break;
            }
            // If "\" is encountered in the initial state or
            // if "," is already present, matches key-value pairs.
            (InnerState::Start | InnerState::AfterComma, Some(QUOTATION_MARK)) => {
                deserializer.reader.discard();
                let k = parse_key(deserializer)?;

                // Matches ':'
                match eat_whitespace_until_not!(deserializer) {
                    Some(COLON) => deserializer.reader.discard(),
                    Some(_) => return unexpected_character!(deserializer),
                    None => return unexpected_eoj!(deserializer),
                };

                // Inserts into object.
                object.insert(k, parse_value(deserializer)?);

                // Sets the state to NoComma.
                state = InnerState::NoComma;
            }
            // In the initial state, it is illegal to encounter any other character.
            (InnerState::Start, Some(_)) => return unexpected_character!(deserializer),
            // In the NoComma state, when "," is encountered, converts state to HaveComma.
            (InnerState::NoComma, Some(COMMA)) => {
                deserializer.reader.discard();
                state = InnerState::AfterComma;
            }
            // In the NoComma state, it's illegal to encounter any other character.
            (InnerState::NoComma, Some(_)) => return unexpected_character!(deserializer),
            // In the HaveComma state, it's illegal to encounter any other character.
            (InnerState::AfterComma, Some(_)) => return unexpected_character!(deserializer),
            // In all cases, None is illegal.
            (_, None) => return unexpected_eoj!(deserializer),
        }
    }
    Ok(JsonValue::Object(object))
}

// Parses string
#[cfg(not(feature = "c_adapter"))]
pub(crate) fn parse_string<R: Cacheable>(
    deserializer: &mut Deserializer<R>,
) -> Result<String, Error> {
    let vec = parse_string_inner(deserializer)?;
    // Since the vec contents are all checked upon matching, the unchecked method is used directly here.
    Ok(unsafe { String::from_utf8_unchecked(vec) })
}

#[cfg(feature = "c_adapter")]
pub(crate) fn parse_string<R: Cacheable>(
    deserializer: &mut Deserializer<R>,
) -> Result<CString, Error> {
    let vec = parse_string_inner(deserializer)?;
    // Since the vec contents are all checked upon matching, the unchecked method is used directly here.
    Ok(unsafe { CString::from_vec_unchecked(vec) })
}

// Parses key
#[inline]
fn parse_key<R: Cacheable>(deserializer: &mut Deserializer<R>) -> Result<String, Error> {
    let vec = parse_string_inner(deserializer)?;
    // Since the vec contents are all checked upon matching, the unchecked method is used directly here.
    Ok(unsafe { String::from_utf8_unchecked(vec) })
}

pub(crate) fn parse_string_inner<R: Cacheable>(
    deserializer: &mut Deserializer<R>,
) -> Result<Vec<u8>, Error> {
    // Used to store strings.
    let mut vec = Vec::new();

    // Sets the starting position of the string.
    deserializer.reader.start_caching();

    loop {
        match deserializer.reader.peek().map_err(Error::new_reader)? {
            Some(ch) => {
                // Improves character recognition speed (reduce the number of comparisons) by looking up tables.
                // If it is an ordinary character, skips it.
                if !ESCAPE[ch as usize] {
                    deserializer.reader.discard();
                    continue;
                }
                match ch {
                    // When '"' is encountered, the string is added to vec.
                    QUOTATION_MARK => {
                        vec.extend_from_slice(deserializer.reader.cached_slice().unwrap());
                        deserializer.reader.end_caching();
                        deserializer.reader.discard();
                        break;
                    }
                    // When '\\' is encountered, matches escape character.
                    REVERSE_SOLIDUS => {
                        vec.extend_from_slice(deserializer.reader.cached_slice().unwrap());
                        deserializer.reader.discard();
                        parse_escape_character(deserializer, &mut vec)?;
                        deserializer.reader.start_caching();
                    }

                    _ => {
                        // Other control characters are not output.
                        return unexpected_character!(deserializer);
                    }
                }
            }
            None => return unexpected_eoj!(deserializer),
        }
    }
    Ok(vec)
}

// Parses escape characters.
fn parse_escape_character<R: Cacheable>(
    deserializer: &mut Deserializer<R>,
    vec: &mut Vec<u8>,
) -> Result<(), Error> {
    vec.push(
        match deserializer.reader.peek().map_err(Error::new_reader)? {
            Some(QUOTATION_MARK) => QUOTATION_MARK,
            Some(REVERSE_SOLIDUS) => REVERSE_SOLIDUS,
            Some(SOLIDUS) => SOLIDUS,
            Some(BS) => BS_UNICODE as u8,
            Some(FF) => FF_UNICODE as u8,
            Some(LF) => LF_UNICODE as u8,
            Some(CR) => CR_UNICODE as u8,
            Some(HT) => HT_UNICODE as u8,
            Some(UNICODE) => {
                deserializer.reader.discard();
                return parse_unicode(deserializer, vec);
            }
            Some(_) => return unexpected_character!(deserializer),
            None => return unexpected_eoj!(deserializer),
        },
    );
    deserializer.reader.discard();
    Ok(())
}

// Parses unicode
fn parse_unicode<R: Cacheable>(
    deserializer: &mut Deserializer<R>,
    vec: &mut Vec<u8>,
) -> Result<(), Error> {
    // Reads a hexadecimal number.
    #[inline]
    fn get_next_digit<R: Cacheable>(deserializer: &mut Deserializer<R>) -> Result<u16, Error> {
        if let Some(ch) = deserializer.reader.peek().map_err(Error::new_reader)? {
            let result = match ch {
                ZERO..=NINE => ch as u16 - ZERO as u16,
                A_LOWER..=F_LOWER => ch as u16 - A_LOWER as u16 + 10,
                A_UPPER..=F_UPPER => ch as u16 - A_UPPER as u16 + 10,
                _ => return unexpected_character!(deserializer),
            };
            deserializer.reader.discard();
            return Ok(result);
        }
        unexpected_eoj!(deserializer)
    }

    // Reads four hexadecimal digits consecutively.
    #[inline]
    fn get_next_four_digits<R: Cacheable>(
        deserializer: &mut Deserializer<R>,
    ) -> Result<u16, Error> {
        Ok(get_next_digit(deserializer)? << 12
            | get_next_digit(deserializer)? << 8
            | get_next_digit(deserializer)? << 4
            | get_next_digit(deserializer)?)
    }

    // Unicode character logic: \uXXXX or \uXXXX\uXXXX
    let unicode1 = get_next_four_digits(deserializer)?;
    let unicode = match char::try_from(unicode1 as u32) {
        Ok(code) => code,
        Err(_) => {
            match_str!(deserializer, UNICODE_START_STR);

            match core::char::decode_utf16(
                [unicode1, get_next_four_digits(deserializer)?]
                    .iter()
                    .copied(),
            )
            .next()
            {
                Some(Ok(code)) => code,
                _ => return Err(Error::Utf8Transform),
            }
        }
    };
    vec.extend_from_slice(unicode.encode_utf8(&mut [0; 4]).as_bytes());
    Ok(())
}

// Matches number.
pub(crate) fn parse_number<R: Cacheable>(
    deserializer: &mut Deserializer<R>,
) -> Result<Number, Error> {
    // Sets the starting position of the string.
    deserializer.reader.start_caching();

    // `neg\dot\exp` determines which of `u64\i64\f64` will be used to represent the final number.
    let mut neg = false;
    let mut dot = false;
    let mut exp = false;

    // Matches '-', JSON syntax does not match '+'
    if let Some(MINUS) = deserializer.reader.peek().map_err(Error::new_reader)? {
        deserializer.reader.discard();
        neg = true;
    }
    // `next_ch` temporarily saves unmatched characters after peek.
    // Used to reduce the number of repeated peeks.
    let mut next_ch = match deserializer.reader.peek().map_err(Error::new_reader)? {
        // The integer part cannot have a leading 0, so if it encounters a 0 here,
        // it enters the value 0 state directly.
        Some(ZERO) => {
            deserializer.reader.discard();
            // The reason to peek here is to compare with
            // Some(ONE... =NINE) branches keep the same return value.
            deserializer.reader.peek().map_err(Error::new_reader)?
        }
        Some(ONE..=NINE) => {
            // Matches one digit character first. Ensure that there is at least one digit character.
            deserializer.reader.discard();
            // Matches as many numeric characters as possible.
            eat_digits_until_not!(deserializer)
        }
        Some(_) => return unexpected_character!(deserializer),
        None => return unexpected_eoj!(deserializer),
    };

    // If there is a decimal point, matches fractional part.
    if let Some(DECIMAL_POINT) = next_ch {
        deserializer.reader.discard();
        dot = true;

        // Matches a numeric character.
        match deserializer.reader.peek().map_err(Error::new_reader)? {
            Some(ZERO..=NINE) => deserializer.reader.discard(),
            Some(_) => return unexpected_character!(deserializer),
            None => return unexpected_eoj!(deserializer),
        };
        //Saves the extra characters for the next match.
        next_ch = eat_digits_until_not!(deserializer)
    }

    // If e is present, matches exponential part.
    if let Some(E_LOWER | E_UPPER) = next_ch {
        deserializer.reader.discard();
        exp = true;
        // Try to match the sign of the exponential part, which can be without the sign.
        match deserializer.reader.peek().map_err(Error::new_reader)? {
            Some(PLUS | MINUS) => deserializer.reader.discard(),
            Some(_) => {}
            None => return unexpected_eoj!(deserializer),
        }
        // Matches a numeric character.
        match deserializer.reader.peek().map_err(Error::new_reader)? {
            Some(ZERO..=NINE) => deserializer.reader.discard(),
            Some(_) => return unexpected_character!(deserializer),
            None => return unexpected_eoj!(deserializer),
        };
        // Matches the remaining numeric characters.
        eat_digits_until_not!(deserializer);
    }

    // The contents of u8 have been checked, so the unchecked method can be used here.
    let str =
        unsafe { core::str::from_utf8_unchecked(deserializer.reader.cached_slice().unwrap()) };
    let number = match (neg, dot, exp) {
        (false, false, false) => {
            Number::Unsigned(str.parse::<u64>().map_err(|_| Error::ParseNumber)?)
        }
        (true, false, false) => Number::Signed(str.parse::<i64>().map_err(|_| Error::ParseNumber)?),
        (_, _, _) => Number::Float(str.parse::<f64>().map_err(|_| Error::ParseNumber)?),
    };

    deserializer.reader.end_caching();
    Ok(number)
}

fn parse_array<R: Cacheable>(deserializer: &mut Deserializer<R>) -> Result<JsonValue, Error> {
    enum InnerState {
        Start,
        AfterComma,
        NoComma,
    }

    deserializer.recursion_depth += 1;
    check_recursion(deserializer)?;

    // Creates an Array to store value.
    let mut array = Array::new();
    // The initial status is Start.
    let mut state = InnerState::Start;

    loop {
        match (state, eat_whitespace_until_not!(deserializer)) {
            // In the initial state, if "]" is encountered, meaning the array is empty.
            (InnerState::Start, Some(RIGHT_SQUARE_BRACKET)) => break,
            // If in the initial state or "," has appeared,
            // matches key-value pairs when any character is encountered.
            (InnerState::Start | InnerState::AfterComma, _) => {
                array.push(parse_value(deserializer)?);

                // Here sets the state to NoComma.
                state = InnerState::NoComma;
            }
            // In NoComma state, the array ends when "]" is encountered.
            (InnerState::NoComma, Some(RIGHT_SQUARE_BRACKET)) => break,
            // In the NoComma state, when "," is encountered, converts to the HaveComma state.
            (InnerState::NoComma, Some(COMMA)) => {
                deserializer.reader.discard();
                state = InnerState::AfterComma;
            }
            // In the NoComma state, it is illegal to encounter any other character.
            (InnerState::NoComma, Some(_)) => return unexpected_character!(deserializer),
            // In all cases, None is illegal.
            (_, None) => return unexpected_eoj!(deserializer),
        }
    }
    deserializer.reader.discard();
    deserializer.recursion_depth -= 1;
    Ok(JsonValue::Array(array))
}

pub(crate) fn read_error_char<R: Cacheable>(
    deserializer: &mut Deserializer<R>,
) -> Result<Option<char>, Error> {
    const CONT_MASK: u8 = 0b0011_1111;

    #[inline]
    fn utf8_first_byte(byte: u8, width: u32) -> u32 {
        (byte & (0x7F >> width)) as u32
    }

    #[inline]
    fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
        (ch << 6) | (byte & CONT_MASK) as u32
    }

    let x = match deserializer.reader.next().map_err(Error::new_reader)? {
        Some(x) => x,
        None => return Ok(None),
    };

    let ch = if x < 128 {
        x as u32
    } else {
        let init = utf8_first_byte(x, 2);

        let y = match deserializer.reader.next().map_err(Error::new_reader)? {
            Some(y) => y,
            None => return Ok(None),
        };

        let mut ch = utf8_acc_cont_byte(init, y);

        if x >= 0xE0 {
            let z = match deserializer.reader.next().map_err(Error::new_reader)? {
                Some(z) => z,
                None => return Ok(None),
            };

            let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
            ch = init << 12 | y_z;

            if x >= 0xF0 {
                let w = match deserializer.reader.next().map_err(Error::new_reader)? {
                    Some(w) => w,
                    None => return Ok(None),
                };
                ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
            }
        }
        ch
    };
    unsafe { Ok(Some(char::from_u32_unchecked(ch))) }
}

#[cfg(test)]
mod ut_states {
    use crate::reader::BytesReader;
    use crate::states::*;
    use std::io::{ErrorKind, Read};

    struct ErrorIo;

    impl Read for ErrorIo {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(ErrorKind::AddrInUse.into())
        }
    }

    /// UT test for macro `eat_whitespace_until_not`.
    ///
    /// # Title
    /// ut_macro_eat_whitespace_until_not
    ///
    /// # Brief
    /// 1. Constructs various inputs.
    /// 2. Uses macro `eat_whitespace_until_not`.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_macro_eat_whitespace_until_not() {
        fn test_func<R: BytesReader + Cacheable>(
            deserializer: &mut Deserializer<R>,
        ) -> Result<Option<u8>, Error> {
            Ok(eat_whitespace_until_not!(deserializer))
        }

        let mut deserializer = Deserializer::new_from_slice(b"      n");
        assert_eq!(test_func(&mut deserializer).unwrap(), Some(b'n'));

        let mut deserializer = Deserializer::new_from_slice(b"      ");
        assert_eq!(test_func(&mut deserializer).unwrap(), None);

        let mut deserializer = Deserializer::new_from_io(ErrorIo);
        assert!(test_func(&mut deserializer).is_err());
    }

    /// UT test for macro `eat_digits_until_not`.
    ///
    /// # Title
    /// ut_macro_eat_digits_until_not
    ///
    /// # Brief
    /// 1. Constructs various inputs.
    /// 2. Uses macro `eat_digits_until_not`.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_macro_eat_digits_until_not() {
        fn test_func<R: BytesReader + Cacheable>(
            deserializer: &mut Deserializer<R>,
        ) -> Result<Option<u8>, Error> {
            Ok(eat_digits_until_not!(deserializer))
        }

        let mut deserializer = Deserializer::new_from_slice(b"1234n");
        assert_eq!(test_func(&mut deserializer).unwrap(), Some(b'n'));

        let mut deserializer = Deserializer::new_from_slice(b"1234");
        assert_eq!(test_func(&mut deserializer).unwrap(), None);

        let mut deserializer = Deserializer::new_from_io(ErrorIo);
        assert!(test_func(&mut deserializer).is_err());
    }

    /// UT test for macro `match_str`.
    ///
    /// # Title
    /// ut_macro_match_str
    ///
    /// # Brief
    /// 1. Constructs various inputs.
    /// 2. Uses macro `match_str`.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_macro_match_str() {
        #[allow(clippy::unit_arg)]
        fn test_func<R: Cacheable>(
            deserializer: &mut Deserializer<R>,
            target: &[u8],
        ) -> Result<(), Error> {
            Ok(match_str!(deserializer, target))
        }

        let mut deserializer = Deserializer::new_from_slice(b"1234");
        assert!(test_func(&mut deserializer, b"1234").is_ok());

        let mut deserializer = Deserializer::new_from_io(ErrorIo);
        assert!(test_func(&mut deserializer, b"1234").is_err());
    }

    /// UT test for `start_parsing`.
    ///
    /// # Title
    /// ut_start_parsing
    ///
    /// # Brief
    /// 1. Constructs various inputs.
    /// 2. Calls `start_parsing`.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_start_parsing() {
        let mut deserializer = Deserializer::new_from_slice(b"null");
        assert_eq!(start_parsing(&mut deserializer).unwrap(), JsonValue::Null);

        let mut deserializer = Deserializer::new_from_slice(b"null      invalid");
        assert!(start_parsing(&mut deserializer).is_err());
    }

    /// UT test for `read_error_char`.
    ///
    /// # Title
    /// ut_read_error_char
    ///
    /// # Brief
    /// 1. Constructs various inputs.
    /// 2. Calls `read_error_char`.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_read_error_char() {
        let mut deserializer = Deserializer::new_from_slice("𤭢".as_bytes());
        assert_eq!(read_error_char(&mut deserializer).unwrap(), Some('𤭢'));

        let mut deserializer = Deserializer::new_from_slice(&[]);
        assert_eq!(read_error_char(&mut deserializer).unwrap(), None);

        let mut deserializer = Deserializer::new_from_slice(&[0xf0]);
        assert_eq!(read_error_char(&mut deserializer).unwrap(), None);

        let mut deserializer = Deserializer::new_from_slice(&[0xf0, 0xa4]);
        assert_eq!(read_error_char(&mut deserializer).unwrap(), None);

        let mut deserializer = Deserializer::new_from_slice(&[0xf0, 0xa4, 0xad]);
        assert_eq!(read_error_char(&mut deserializer).unwrap(), None);
    }

    /// UT test for `parse_value`.
    ///
    /// # Title
    /// ut_parse_value
    ///
    /// # Brief
    /// 1. Creates an instance of json.
    /// 2. Calls the parsing function of State.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_parse_value() {
        let mut deserializer = Deserializer::new_from_slice(b"null");
        assert_eq!(parse_value(&mut deserializer).unwrap(), JsonValue::Null);

        let mut deserializer = Deserializer::new_from_slice(b"true");
        assert_eq!(
            parse_value(&mut deserializer).unwrap(),
            JsonValue::Boolean(true)
        );

        let mut deserializer = Deserializer::new_from_slice(b"false");
        assert_eq!(
            parse_value(&mut deserializer).unwrap(),
            JsonValue::Boolean(false)
        );

        let mut deserializer = Deserializer::new_from_slice(b"123");
        assert!(parse_value(&mut deserializer).is_ok());

        let mut deserializer = Deserializer::new_from_slice(b"\"abc\"");
        assert!(parse_value(&mut deserializer).is_ok());

        let mut deserializer = Deserializer::new_from_slice(b"[1, 2, 3]");
        assert!(parse_value(&mut deserializer).is_ok());

        let mut deserializer = Deserializer::new_from_slice(b"{\"key\":\"value\"}");
        assert!(parse_value(&mut deserializer).is_ok());

        let mut deserializer = Deserializer::new_from_slice(b"\"abc\"");
        assert!(parse_value(&mut deserializer).is_ok());
    }

    /// UT test for `parse_string`.
    ///
    /// # Title
    /// ut_parse_string
    ///
    /// # Brief
    /// 1. Creates an instance of Reader.
    /// 2. Calls the parsing function of State.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_parse_string() {
        // 1.Enter a valid key (or String) and return a string.
        // 2.Enter an invalid key (or string) and return an Error message.

        #[cfg(feature = "c_adapter")]
        use std::ffi::CString;

        // Ensure that the previous '"' has been read before entering parse_string.
        // Empty string
        let str = "\"";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        #[cfg(not(feature = "c_adapter"))]
        assert_eq!(parse_string(&mut deserializer).unwrap(), String::from(""));
        #[cfg(feature = "c_adapter")]
        assert_eq!(
            parse_string(&mut deserializer).unwrap(),
            CString::new("").unwrap()
        );

        // General character
        let str = "abcdefghijklmnopqrstuvwxyz1234567890-=~!@#$%^&*()_+[]{}|<>?:;'\"";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        #[cfg(not(feature = "c_adapter"))]
        assert_eq!(
            parse_string(&mut deserializer).unwrap(),
            String::from("abcdefghijklmnopqrstuvwxyz1234567890-=~!@#$%^&*()_+[]{}|<>?:;'"),
        );
        #[cfg(feature = "c_adapter")]
        assert_eq!(
            parse_string(&mut deserializer).unwrap(),
            CString::new("abcdefghijklmnopqrstuvwxyz1234567890-=~!@#$%^&*()_+[]{}|<>?:;'").unwrap(),
        );

        // Escape character
        let str = r#"\/\\\"\uCAFE\uBABE\uAB98\uFCDE\ubcda\uef4A\b\f\n\r\t""#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        #[cfg(not(feature = "c_adapter"))]
        assert_eq!(
            parse_string(&mut deserializer).unwrap(),
            String::from(
                "/\\\"\u{CAFE}\u{BABE}\u{AB98}\u{FCDE}\u{bcda}\u{ef4A}\u{0008}\u{000c}\n\r\t"
            ),
        );
        #[cfg(feature = "c_adapter")]
        assert_eq!(
            parse_string(&mut deserializer).unwrap(),
            CString::new(
                "/\\\"\u{CAFE}\u{BABE}\u{AB98}\u{FCDE}\u{bcda}\u{ef4A}\u{0008}\u{000c}\n\r\t"
            )
            .unwrap(),
        );

        let str = r#"\uD852\uDF62""#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        #[cfg(not(feature = "c_adapter"))]
        assert_eq!(parse_string(&mut deserializer).unwrap(), String::from("𤭢"),);
        #[cfg(feature = "c_adapter")]
        assert_eq!(
            parse_string(&mut deserializer).unwrap(),
            CString::new("𤭢").unwrap(),
        );

        // Error scenes
        // 1.There are no trailing quotes to end a match (or encounter a terminator).
        let str = "abc";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_string(&mut deserializer).is_err());

        // 2.Illegal escape character.
        let str = r#"\g""#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_string(&mut deserializer).is_err());

        // 3.A backslash is followed by a terminator.
        let str = r#"\"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_string(&mut deserializer).is_err());

        // 4.Illegal unicode characters.
        let str = r#"\uBEEF"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_string(&mut deserializer).is_err());

        let str = r#"\uZ000"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_string(&mut deserializer).is_err());

        let str = r#"\u"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_string(&mut deserializer).is_err());

        let str = r#"\uD852\uDB00""#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_string(&mut deserializer).is_err());

        // 5.Control character.
        let str = "\u{0}";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_string(&mut deserializer).is_err());
    }

    /// UT test for `parse_number`.
    ///
    /// # Title
    /// ut_parse_number
    ///
    /// # Brief
    /// 1. Creates an instance of Reader.
    /// 2. Calls the parsing function of State.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_parse_number() {
        // 1.Enters a value (legal) and return a numeric value.
        // 2.Enters a value (illegal) and return the corresponding Error.
        // 3.Enters a value (text terminated prematurely, illegal) and return the corresponding Error.

        let str = r#"0"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert_eq!(parse_number(&mut deserializer).unwrap(), 0.into());

        let str = r#"-0"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert_eq!(parse_number(&mut deserializer).unwrap(), 0.into());

        let str = r#"0.123e+4"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert_eq!(parse_number(&mut deserializer).unwrap(), 1230.into());

        // Error scenes.
        // 1.No number exists.
        let str = r#""#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_number(&mut deserializer).is_err());

        // 2.Non-numeric characters exist.
        let str = r#"a123"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_number(&mut deserializer).is_err());

        // 3.There is no integer part.
        let str = r#".123"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_number(&mut deserializer).is_err());

        // 4.Positive numbers appear with a plus sign.
        let str = r#"+1234"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_number(&mut deserializer).is_err());

        // 5.Integer part in front of a number of 0.
        let str = r#"00001234"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        // In this case, only 0 will be read.
        // The subsequent matching will cause an error when encounter a number.
        assert_eq!(parse_number(&mut deserializer).unwrap(), 0.into());

        // 6.The integer part contains other characters.
        let str = r#"12a34"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        // In this case, only 12 will be read.
        // The subsequent matching will cause an error when encounter 'a'.
        assert_eq!(parse_number(&mut deserializer).unwrap(), 12.into());

        // 7.The decimal part contains other characters.
        let str = r#"12.a34"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_number(&mut deserializer).is_err());

        let str = r#"12."#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_number(&mut deserializer).is_err());

        let str = r#"12.3a4"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        // In this case, only 12.3 will be read.
        // The subsequent matching will cause an error when encounter 'a'.
        assert_eq!(parse_number(&mut deserializer).unwrap(), (12.3).into());

        // 8.The exponential part contains other characters.
        let str = r#"12.34e+2a3"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        // In this case, only 12.34e+2 will be read.
        // The subsequent matching will cause an error when encounter 'a'.
        assert_eq!(parse_number(&mut deserializer).unwrap(), (1234).into());

        let str = r#"12.34e"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_number(&mut deserializer).is_err());

        let str = r#"12.34ea"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_number(&mut deserializer).is_err());

        let str = r#"12.34e+"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_number(&mut deserializer).is_err());
    }

    /// UT test for `ut_parse_array`.
    ///
    /// # Title
    /// ut_parse_array
    ///
    /// # Brief
    /// 1. Creates an instance of Reader.
    /// 2. Calls the parsing function of State.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_parse_array() {
        // 1.Enters a value (legal) and return a numeric value.
        // 2.Enters a value (illegal) and return the corresponding Error.
        // 3.Enters a value (text terminated prematurely, illegal) and return the corresponding Error.

        // Before entering the parse_array function, needs to match '['.
        let str = r#"]"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert_eq!(parse_array(&mut deserializer).unwrap(), Array::new().into());

        let str = r#"              ]"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert_eq!(parse_array(&mut deserializer).unwrap(), Array::new().into());

        let str = r#"1, 2, 3]"#;
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        let array = array!(1u8, 2u8, 3u8);
        assert_eq!(parse_array(&mut deserializer).unwrap(), array.into());

        let str = "\
            1,\
            2,\
            3\
        ]";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        let array = array!(1u8, 2u8, 3u8);
        assert_eq!(parse_array(&mut deserializer).unwrap(), array.into());

        // Error scenes.
        // 1.Encounter terminator too early.
        let str = "";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_array(&mut deserializer).is_err());

        let str = "1  ";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_array(&mut deserializer).is_err());

        // 2.',' is not used between values.
        let str = "1 2";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_array(&mut deserializer).is_err());

        // 3.The extra ',' at the end.
        let str = "1, 2,]";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_array(&mut deserializer).is_err());
    }

    /// UT test for `parse_object`.
    ///
    /// # Title
    /// parse_object
    ///
    /// # Brief
    /// 1. Creates an instance of Reader.
    /// 2. Calls the parsing function of State.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_parse_object() {
        // 1.Enters a value (legal) and return a numeric value.
        // 2.Enters a value (illegal) and return the corresponding Error.
        // 3.Enters a value (text terminated prematurely, illegal) and return the corresponding Error.

        // Before entering parse_object, needs to match '{'.
        let str = "}";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert_eq!(
            parse_object(&mut deserializer).unwrap(),
            Object::new().into()
        );

        let str = "\"key1\": \"value\", \"key2\": \"value\"}";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        let object = object!("key1" => "value"; "key2" => "value");
        assert_eq!(parse_object(&mut deserializer).unwrap(), object.into());

        let str = "\
            \"key1\": \"value\",\
            \"key2\": \"value\"\
        }";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        let object = object!("key1" => "value"; "key2" => "value");
        assert_eq!(parse_object(&mut deserializer).unwrap(), object.into());

        // Error scenes.
        // 1.Encounter terminator too early.
        let str = "";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_object(&mut deserializer).is_err());

        // 2.Encounter ',' too early.
        let str = ",";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_object(&mut deserializer).is_err());

        // 3.The extra ',' at the end.
        let str = "\"key1\": \"value\", \"key2\": \"value\",}";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_object(&mut deserializer).is_err());

        // 4.There is no ':'.
        let str = "\"key1\"t";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_object(&mut deserializer).is_err());

        let str = "\"key1\"";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_object(&mut deserializer).is_err());

        // 5.Extra character.
        let str = "\"key1\": 1      t";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_object(&mut deserializer).is_err());

        let str = "\"key1\": 1, t";
        let mut deserializer = Deserializer::new_from_slice(str.as_bytes());
        assert!(parse_object(&mut deserializer).is_err());
    }
    /// UT test for recursion limit.
    ///
    /// # Title
    /// ut_recursion_limit
    ///
    /// # Brief
    /// 1. Creates an instance exceeds recursion limit.
    /// 2. Calls the parsing function of State.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_recursion_limit() {
        // Examples of array.
        // This example has 128 layers of recursion(The upper recursion limit).
        let text = r#"
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        "#;
        let mut deserializer = Deserializer::new_from_slice(text.as_ref());
        assert!(start_parsing(&mut deserializer).is_ok());

        // This example has 129 layers of recursion(The upper recursion limit is 128).
        let text = r#"
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        "#;
        let mut deserializer = Deserializer::new_from_slice(text.as_ref());
        assert!(start_parsing(&mut deserializer).is_err());

        // Examples of object.
        let mut str = String::from(r#"{"key":"value"}"#);
        // 128 layers
        for _i in 0..RECURSION_LIMIT - 1 {
            str = str.replace(r#""value""#, r#"{"key":"value"}"#);
        }
        let text = str.as_bytes();
        let mut deserializer = Deserializer::new_from_slice(text);
        assert!(start_parsing(&mut deserializer).is_ok());

        let mut str = String::from(r#"{"key":"value"}"#);
        // 129 layers
        for _i in 0..RECURSION_LIMIT {
            str = str.replace(r#""value""#, r#"{"key":"value"}"#);
        }
        let text = str.as_bytes();
        let mut deserializer = Deserializer::new_from_slice(text);
        assert!(start_parsing(&mut deserializer).is_err());
    }
}
