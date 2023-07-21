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

use crate::Error;
use core::fmt::{Display, Formatter};
use std::fmt::Debug;

/// Numerical type
///
/// # Examples
/// ```
/// use ylong_json::Number;
///
/// let number: Number = 0.0.into();
/// assert_eq!(number.is_float(), true);
/// ```
#[derive(Clone)]
pub enum Number {
    /// Unsigned integer
    Unsigned(u64),
    /// Signed integer
    Signed(i64),
    /// Floating point number
    Float(f64),
}

impl Number {
    /// Determines whether the number is an unsigned integer.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Number;
    ///
    /// let number: Number = 1u8.into();
    /// assert_eq!(number.is_unsigned(), true);
    ///
    /// let number: Number = 1i8.into();
    /// assert_eq!(number.is_unsigned(), false);
    /// ```
    pub fn is_unsigned(&self) -> bool {
        matches!(*self, Self::Unsigned(_))
    }

    /// Determines whether the number is a signed integer.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Number;
    ///
    /// let number: Number = 1i8.into();
    /// assert_eq!(number.is_signed(), true);
    ///
    /// let number: Number = 1u8.into();
    /// assert_eq!(number.is_signed(), false);
    /// ```
    pub fn is_signed(&self) -> bool {
        matches!(*self, Self::Signed(_))
    }

    /// Determines whether the number is a floating point number.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Number;
    ///
    /// let number: Number = 0.0.into();
    /// assert_eq!(number.is_float(), true);
    ///
    /// let number: Number = 1i8.into();
    /// assert_eq!(number.is_float(), false);
    /// ```
    pub fn is_float(&self) -> bool {
        matches!(*self, Self::Float(_))
    }

    /// Trys converting the number to u64. If conversion fails, returns Error.
    ///
    /// Only Unsigned case means success, other cases return Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Number;
    ///
    /// let number: Number = 1u8.into();
    /// assert_eq!(number.try_as_u64().unwrap(), 1u64);
    ///
    /// let number: Number = 1i8.into();
    /// assert_eq!(number.try_as_u64().is_err(), true);
    /// ```
    pub fn try_as_u64(&self) -> Result<u64, Error> {
        match self {
            Self::Unsigned(u) => Ok(*u),
            _ => Err(Error::TypeTransform),
        }
    }

    /// Trys converting the number to i64. If conversion fails, returns Error.
    ///
    /// Only in 164 range unsigned numbers can be converted. Signed numbers can be converted.
    /// Otherwise, returns Error.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Number;
    ///
    /// let number: Number = 1i8.into();
    /// assert_eq!(number.try_as_i64().unwrap(), 1i64);
    ///
    /// let number: Number = u64::MAX.into();
    /// assert_eq!(number.try_as_i64().is_err(), true);
    /// ```
    pub fn try_as_i64(&self) -> Result<i64, Error> {
        match self {
            Self::Unsigned(u) => {
                if *u <= i64::MAX as u64 {
                    Ok(*u as i64)
                } else {
                    Err(Error::TypeTransform)
                }
            }
            Self::Signed(i) => Ok(*i),
            Self::Float(_) => Err(Error::TypeTransform),
        }
    }

    /// Trys converting the number to f64. If conversion fails, returns Error.
    ///
    /// All types can be converted to f64.
    ///
    /// # Examples
    /// ```
    /// use ylong_json::Number;
    ///
    /// let number: Number = 0.0.into();
    /// assert_eq!(number.try_as_f64().unwrap(), 0.0f64);
    /// ```
    pub fn try_as_f64(&self) -> Result<f64, Error> {
        match self {
            Self::Unsigned(u) => Ok(*u as f64),
            Self::Signed(i) => Ok(*i as f64),
            Self::Float(f) => Ok(*f),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        let a = self.try_as_f64().unwrap();
        let b = other.try_as_f64().unwrap();
        a == b
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Number::Unsigned(x) => write!(f, "{x}"),
            Number::Signed(x) => write!(f, "{x}"),
            Number::Float(x) => write!(f, "{x:?}"),
        }
    }
}

impl Debug for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

macro_rules! number_from_unsigned {
    ($($u: tt),* $(,)?) => {
        $(
            impl From<$u> for Number {
                fn from(u: $u) -> Self {
                    Self::Unsigned(u as u64)
                }
            }
        )*
    }
}

macro_rules! number_from_signed {
    ($($i: tt),* $(,)?) => {
        $(
            impl From<$i> for Number {
                fn from(i: $i) -> Self {
                    Self::Signed(i as i64)
                }
            }
        )*
    }
}

macro_rules! number_from_float {
    ($($f: tt),* $(,)?) => {
        $(
            impl From<$f> for Number {
                fn from(f: $f) -> Self {
                    Self::Float(f as f64)
                }
            }
        )*
    }
}

number_from_unsigned!(u8, u16, u32, u64, usize);
number_from_signed!(i8, i16, i32, i64, isize);
number_from_float!(f32, f64);

#[cfg(test)]
mod ut_number {
    use crate::Number;

    /// UT test for `Number::fmt`.
    ///
    /// # Title
    /// ut_number_fmt
    ///
    /// # Brief
    /// 1. Creates some `Number`s.
    /// 2. Calls `Number::fmt`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_number_fmt() {
        assert_eq!(format!("{}", Number::Unsigned(1)), "1");
        assert_eq!(format!("{:?}", Number::Unsigned(1)), "1");

        assert_eq!(format!("{}", Number::Signed(1)), "1");
        assert_eq!(format!("{:?}", Number::Signed(1)), "1");

        assert_eq!(format!("{}", Number::Float(1.0)), "1.0");
        assert_eq!(format!("{:?}", Number::Float(1.0)), "1.0");
    }

    /// UT test for `Number::clone`.
    ///
    /// # Title
    /// ut_number_clone
    ///
    /// # Brief
    /// 1. Creates some `Number`s.
    /// 2. Calls `Number::clone`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_number_clone() {
        let number1 = Number::Unsigned(1);
        assert_eq!(number1, number1.clone());

        let number1 = Number::Signed(1);
        assert_eq!(number1, number1.clone());

        let number1 = Number::Float(1.0);
        assert_eq!(number1, number1.clone());
    }

    /// UT test for `Number::is_unsigned`.
    ///
    /// # Title
    /// ut_number_is_unsigned
    ///
    /// # Brief
    /// 1. Creates some `Number`s.
    /// 2. Calls `Number::is_unsigned`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_number_is_unsigned() {
        assert!(Number::Unsigned(1).is_unsigned());
        assert!(!Number::Signed(1).is_unsigned());
        assert!(!Number::Float(1.0).is_unsigned());
    }

    /// UT test for `Number::is_signed`.
    ///
    /// # Title
    /// ut_number_is_signed
    ///
    /// # Brief
    /// 1. Creates some `Number`s.
    /// 2. Calls `Number::is_signed`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_number_is_signed() {
        assert!(!Number::Unsigned(1).is_signed());
        assert!(Number::Signed(1).is_signed());
        assert!(!Number::Float(1.0).is_signed());
    }

    /// UT test for `Number::is_float`.
    ///
    /// # Title
    /// ut_number_is_float
    ///
    /// # Brief
    /// 1. Creates some `Number`s.
    /// 2. Calls `Number::is_float`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_number_is_float() {
        assert!(!Number::Unsigned(1).is_float());
        assert!(!Number::Signed(1).is_float());
        assert!(Number::Float(1.0).is_float());
    }

    /// UT test for `Number::try_as_u64`.
    ///
    /// # Title
    /// ut_number_try_as_u64
    ///
    /// # Brief
    /// 1. Creates some `Number`s.
    /// 2. Calls `Number::try_as_u64`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_number_try_as_u64() {
        assert!(Number::Unsigned(1).try_as_u64().is_ok());
        assert!(Number::Signed(1).try_as_u64().is_err());
        assert!(Number::Float(1.0).try_as_u64().is_err());
    }

    /// UT test for `Number::try_as_i64`.
    ///
    /// # Title
    /// ut_number_try_as_i64
    ///
    /// # Brief
    /// 1. Creates some `Number`s.
    /// 2. Calls `Number::try_as_i64`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_number_try_as_i64() {
        assert!(Number::Unsigned(1).try_as_i64().is_ok());
        assert!(Number::Unsigned(u64::MAX).try_as_i64().is_err());
        assert!(Number::Signed(1).try_as_i64().is_ok());
        assert!(Number::Float(1.0).try_as_i64().is_err());
    }
}
