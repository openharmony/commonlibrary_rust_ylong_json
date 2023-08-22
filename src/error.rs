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

use core::fmt::{Debug, Display, Formatter, Result};
use std::ffi::IntoStringError;
use std::string::FromUtf8Error;

/// Errors during parsing.
pub enum Error {
    /// Parsing error.
    Parsing(ParseError),

    /// Io error.
    Io(std::io::Error),

    /// Parse number error.
    ParseNumber,

    /// Utf8 transform error.
    Utf8Transform,

    /// Type transform error.
    TypeTransform,

    /// Reader error.
    Reader(Box<dyn std::error::Error>),

    /// Incorrect serde usage error.
    IncorrectSerdeUsage,

    /// Used to convert serde-related errors.
    Custom(String),

    /// Exceeds the recursion limit.
    ExceedRecursionLimit,
}

/// The specific location and character of the error during parsing.
pub enum ParseError {
    /// Undesired character (line number, character number, current character)
    UnexpectedCharacter(usize, usize, char),

    /// Illegal UTF-8 character (line number)
    InvalidUtf8Bytes(usize),

    /// Undesired end-of-file character (line number)
    UnexpectedEndOfJson(usize),

    /// Expected Eof but not received (line number)
    TrailingBytes(usize),

    /// The input sequence has not yet been parsed.
    ParsingUnfinished,

    /// There is an extra comma after the last value in an array or map (line number, character number)
    TrailingComma(usize, usize),

    /// A colon is missing (line number, character number)
    MissingColon(usize, usize),

    /// A comma is missing (line number, character number)
    MissingComma(usize, usize),
}

impl Error {
    pub(crate) fn new_reader<E: Into<Box<dyn std::error::Error>>>(e: E) -> Self {
        Error::Reader(e.into())
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedCharacter(line, pos, unexpected) => {
                write!(
                    f,
                    "[Line]: {line}, [Pos]: {pos}, [Error]: Unexpected character: "
                )?;
                let mut str = match *unexpected {
                    '\u{8}' => Some("\'\\b\'"),
                    '\u{b}' => Some("\'\\v\'"),
                    '\u{c}' => Some("\'\\f\'"),
                    _ => None,
                };
                if let Some(s) = str.take() {
                    write!(f, "{s}.")
                } else {
                    write!(f, "{unexpected:?}.")
                }
            }
            Self::InvalidUtf8Bytes(line) => {
                write!(f, "[line]: {line}, [Error]: Invalid UTF-8 byte.")
            }
            Self::UnexpectedEndOfJson(line) => {
                write!(f, "[Line]: {line}, [Error]: Unexpected end of json.")
            }
            Self::TrailingBytes(line) => {
                write!(f, "[Line]: {line}, [Error]: Expected end of json but not.")
            }
            Self::ParsingUnfinished => {
                write!(f, "[Error]: Value has not been fully deserialized.")
            }
            Self::TrailingComma(line, pos) => {
                write!(
                    f,
                    "[Line]: {line}, [Pos]: {pos}, [Error]: Has a comma after the last value in an array or map."
                )
            }
            Self::MissingColon(line, pos) => {
                write!(f, "[Line]: {line}, [Pos]: {pos}, [Error]: A colon is missing between key and value.")
            }
            Self::MissingComma(line, pos) => {
                write!(
                    f,
                    "[Line]: {line}, [Pos]: {pos}, [Error]: A comma is missing before next value."
                )
            }
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Parsing(e) => write!(f, "Parse Error: {e:?}"),
            Self::Io(e) => write!(f, "Io Error: {e:?}"),
            Self::ParseNumber => write!(f, "Parse Number Error"),
            Self::TypeTransform => write!(f, "Type Transform Error"),
            Self::Utf8Transform => write!(f, "Utf8 Transform Error"),
            Self::IncorrectSerdeUsage => write!(f, "Incorrect Serde Usage Error"),
            Self::Custom(s) => write!(f, "{s}"),
            Self::Reader(e) => write!(f, "Reader Error:{e:?}"),
            Self::ExceedRecursionLimit => write!(f, "Exceed the recursion limit"),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(self, f)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parsing(e)
    }
}

impl From<core::str::Utf8Error> for Error {
    fn from(_: core::str::Utf8Error) -> Self {
        Error::Utf8Transform
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl std::error::Error for Error {}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Self {
        Error::Utf8Transform
    }
}

impl From<IntoStringError> for Error {
    fn from(_: IntoStringError) -> Self {
        Error::TypeTransform
    }
}

#[cfg(test)]
mod ut_error {
    use crate::{Error, ParseError};
    use std::ffi::CString;
    use std::io::ErrorKind;

    /// UT test for `Error::fmt`.
    ///
    /// # Title
    /// ut_error_fmt
    ///
    /// # Brief
    /// 1. Creates a `Error`.
    /// 2. Calls `Error::fmt` on this error.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_error_fmt() {
        assert_eq!(
            format!(
                "{:?}",
                Error::Parsing(ParseError::UnexpectedCharacter(1, 1, 'a'))
            ),
            "Parse Error: [Line]: 1, [Pos]: 1, [Error]: Unexpected character: 'a'.",
        );

        assert_eq!(
            format!(
                "{:?}",
                Error::Parsing(ParseError::UnexpectedCharacter(1, 1, '\u{8}'))
            ),
            "Parse Error: [Line]: 1, [Pos]: 1, [Error]: Unexpected character: '\\b'.",
        );

        assert_eq!(
            format!(
                "{:?}",
                Error::Parsing(ParseError::UnexpectedCharacter(1, 1, '\u{b}'))
            ),
            "Parse Error: [Line]: 1, [Pos]: 1, [Error]: Unexpected character: '\\v'.",
        );

        assert_eq!(
            format!(
                "{:?}",
                Error::Parsing(ParseError::UnexpectedCharacter(1, 1, '\u{c}'))
            ),
            "Parse Error: [Line]: 1, [Pos]: 1, [Error]: Unexpected character: '\\f'.",
        );

        assert_eq!(
            format!("{:?}", Error::Parsing(ParseError::InvalidUtf8Bytes(1))),
            "Parse Error: [line]: 1, [Error]: Invalid UTF-8 byte.",
        );

        assert_eq!(
            format!("{:?}", Error::Parsing(ParseError::UnexpectedEndOfJson(1))),
            "Parse Error: [Line]: 1, [Error]: Unexpected end of json.",
        );

        assert_eq!(
            format!("{:?}", Error::Parsing(ParseError::TrailingBytes(1))),
            "Parse Error: [Line]: 1, [Error]: Expected end of json but not.",
        );

        assert_eq!(
            format!("{:?}", Error::Parsing(ParseError::ParsingUnfinished)),
            "Parse Error: [Error]: Value has not been fully deserialized.",
        );

        assert_eq!(
            format!(
                "{:?}",
                Error::Io(std::io::Error::from(ErrorKind::AddrInUse))
            ),
            "Io Error: Kind(AddrInUse)",
        );

        assert_eq!(format!("{:?}", Error::ParseNumber), "Parse Number Error",);

        assert_eq!(
            format!("{:?}", Error::Utf8Transform),
            "Utf8 Transform Error",
        );

        assert_eq!(
            format!("{:?}", Error::TypeTransform),
            "Type Transform Error",
        );

        assert_eq!(
            format!("{:?}", Error::IncorrectSerdeUsage),
            "Incorrect Serde Usage Error",
        );

        assert_eq!(
            format!("{:?}", Error::Custom(String::from("Custom Error"))),
            "Custom Error",
        );

        assert_eq!(
            format!("{:?}", Error::ExceedRecursionLimit),
            "Exceed the recursion limit",
        );

        assert_eq!(
            format!(
                "{:?}",
                Error::Reader(std::io::Error::from(ErrorKind::AddrInUse).into())
            ),
            "Reader Error:Kind(AddrInUse)",
        );

        assert_eq!(
            format!(
                "{}",
                Error::Parsing(ParseError::UnexpectedCharacter(1, 1, 'a'))
            ),
            "Parse Error: [Line]: 1, [Pos]: 1, [Error]: Unexpected character: 'a'.",
        );

        assert_eq!(
            format!(
                "{}",
                Error::Parsing(ParseError::UnexpectedCharacter(1, 1, '\u{8}'))
            ),
            "Parse Error: [Line]: 1, [Pos]: 1, [Error]: Unexpected character: '\\b'.",
        );

        assert_eq!(
            format!(
                "{}",
                Error::Parsing(ParseError::UnexpectedCharacter(1, 1, '\u{b}'))
            ),
            "Parse Error: [Line]: 1, [Pos]: 1, [Error]: Unexpected character: '\\v'.",
        );

        assert_eq!(
            format!(
                "{}",
                Error::Parsing(ParseError::UnexpectedCharacter(1, 1, '\u{c}'))
            ),
            "Parse Error: [Line]: 1, [Pos]: 1, [Error]: Unexpected character: '\\f'.",
        );

        assert_eq!(
            format!("{}", Error::Parsing(ParseError::InvalidUtf8Bytes(1))),
            "Parse Error: [line]: 1, [Error]: Invalid UTF-8 byte.",
        );

        assert_eq!(
            format!("{}", Error::Parsing(ParseError::UnexpectedEndOfJson(1))),
            "Parse Error: [Line]: 1, [Error]: Unexpected end of json.",
        );

        assert_eq!(
            format!("{}", Error::Parsing(ParseError::TrailingBytes(1))),
            "Parse Error: [Line]: 1, [Error]: Expected end of json but not.",
        );

        assert_eq!(
            format!("{}", Error::Io(std::io::Error::from(ErrorKind::AddrInUse))),
            "Io Error: Kind(AddrInUse)",
        );

        assert_eq!(format!("{}", Error::ParseNumber), "Parse Number Error",);

        assert_eq!(format!("{}", Error::Utf8Transform), "Utf8 Transform Error",);

        assert_eq!(format!("{}", Error::TypeTransform), "Type Transform Error",);

        assert_eq!(
            format!(
                "{}",
                Error::Reader(std::io::Error::from(ErrorKind::AddrInUse).into())
            ),
            "Reader Error:Kind(AddrInUse)",
        );
    }

    /// UT test for `Error::from`.
    ///
    /// # Title
    /// ut_error_from
    ///
    /// # Brief
    /// 1. Creates some other errors.
    /// 2. Calls `Error::from` on those error.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_error_from() {
        assert_eq!(
            format!("{}", Error::from(ParseError::TrailingBytes(1))),
            "Parse Error: [Line]: 1, [Error]: Expected end of json but not.",
        );

        assert_eq!(
            format!(
                "{}",
                Error::from(std::io::Error::from(ErrorKind::AddrInUse))
            ),
            "Io Error: Kind(AddrInUse)",
        );

        let str_vec = [0b10000000u8; 1];
        assert_eq!(
            format!(
                "{}",
                Error::from(std::str::from_utf8(&str_vec).err().unwrap())
            ),
            "Utf8 Transform Error",
        );

        assert_eq!(
            format!(
                "{}",
                Error::from(String::from_utf8(vec![129, 129, 129]).err().unwrap())
            ),
            "Utf8 Transform Error",
        );

        assert_eq!(
            format!(
                "{}",
                Error::from(
                    CString::new(vec![129, 129, 129])
                        .expect("CString::new failed")
                        .into_string()
                        .err()
                        .unwrap()
                )
            ),
            "Type Transform Error",
        );
    }
}
