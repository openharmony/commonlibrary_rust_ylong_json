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

mod io_reader;
pub(crate) use io_reader::IoReader;

mod slice_reader;
pub(crate) use slice_reader::SliceReader;

/// `BytesReader` provides the basic byte read interface, such as `next`,
/// `peek`, `index`. Users can obtain the next byte or the current read
/// position according to these interfaces.
pub(crate) trait BytesReader {
    /// Errors that may occur during reading, usually in the I\O process.
    type Error: Into<Box<dyn std::error::Error>>;

    /// Get the next character and move the cursor to the next place.
    fn next(&mut self) -> Result<Option<u8>, Self::Error>;

    /// Get the next character, but don't move the cursor. So the next read
    /// will get the same character.
    fn peek(&mut self) -> Result<Option<u8>, Self::Error>;

    /// Discard the next character and move the cursor to the next place.
    fn discard(&mut self);

    /// Get the current cursor position and return it as usize.
    fn index(&self) -> usize;

    /// Get the current cursor position and return it as `Position`.
    fn position(&self) -> Position;
}

/// `Cacheable` provides some byte cache interfaces for caching a portion of
/// contiguous bytes in a byte stream.
pub(crate) trait Cacheable: BytesReader {
    /// Start the cache operation. This interface needs to be used with
    /// `end_caching` or `take_cached_data`.
    fn start_caching(&mut self);

    /// Get the length of the cached bytes. Since the logic of caching
    /// operations is implementation-dependent, we provide an interface that
    /// uses mutable references here.
    fn cached_len(&mut self) -> Option<usize>;

    /// Get a slice of the cached bytes. Since the logic of caching operations
    /// is implementation-dependent, we provide an interface that uses mutable
    /// references here.
    fn cached_slice(&mut self) -> Option<&[u8]>;

    /// Get a `Vec` of the cached bytes. Since the logic of caching operations
    /// is implementation-dependent, we provide an interface that uses mutable
    /// references here.
    fn cached_data(&mut self) -> Option<Vec<u8>>;

    /// End the cache operation. This interface needs to be used with
    /// `start_caching`.
    fn end_caching(&mut self);

    /// End the cache operation and return the cached bytes. This interface
    /// needs to be used with `start_caching`.
    fn take_cached_data(&mut self) -> Option<Vec<u8>>;
}

/// `RemainderCountable` provides the interface related to the remainder.
pub(crate) trait RemainderCountable: BytesReader {
    /// Get the length of the remainder.
    fn remainder_len(&self) -> usize;

    /// Get a slice of the remainder.
    fn remainder_slice(&self) -> &[u8];

    /// Get a `Vec<u8>` of the remainder.
    fn remainder_data(&self) -> Vec<u8>;
}

/// `NBytesReadable` provides interfaces to read 'n' bytes at one time.
pub(crate) trait NBytesReadable: BytesReader {
    /// Read the next 'n' bytes and move the cursor to the next nth position.
    /// If there are not enough bytes remaining to satisfy 'n', return `None`
    /// and do nothing.
    fn next_n(&mut self, n: usize) -> Result<Option<&[u8]>, Self::Error>;

    /// Get the next 'n' bytes and do not move the cursor. If there are not
    /// enough bytes remaining to satisfy 'n', return `None` and do nothing.
    fn peek_n(&mut self, n: usize) -> Result<Option<&[u8]>, Self::Error>;

    /// Discard the next 'n' bytes and move the cursor to the next nth position.
    /// If there are not enough bytes remaining to satisfy 'n', do nothing.
    fn discard_n(&mut self, n: usize);
}

/// Position information which expressed in row and column.
#[derive(Clone)]
pub(crate) struct Position {
    line: usize,
    column: usize,
}

impl Position {
    /// Create a `Position` from the given line and column.
    #[inline]
    pub(crate) fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Get line.
    #[inline]
    pub(crate) fn line(&self) -> usize {
        self.line
    }

    /// Get column.
    #[inline]
    pub(crate) fn column(&self) -> usize {
        self.column
    }
}

#[cfg(test)]
mod ut_position {
    use super::Position;

    /// UT test for `Position::new`.
    ///
    /// # Title
    /// ut_position_new
    ///
    /// # Brief
    /// 1. Call `Position::new` to create a `Position`.
    /// 2. Check if the results are correct.
    #[test]
    fn ut_position_new() {
        let position = Position::new(1, 1);
        assert_eq!(position.line, 1);
        assert_eq!(position.column, 1);
    }

    /// UT test for `Position::line`.
    ///
    /// # Title
    /// ut_position_line
    ///
    /// # Brief
    /// 1. Create a `Position`.
    /// 2. Call `Position::line` to get the line number of `Position`.
    /// 3. Check if the results are correct.
    #[test]
    fn ut_position_line() {
        let position = Position::new(1, 1);
        assert_eq!(position.line(), 1);
    }

    /// UT test for `Position::column`.
    ///
    /// # Title
    /// ut_position_column
    ///
    /// # Brief
    /// 1. Create a `Position`.
    /// 2. Call `Position::column` to get the column number of `Position`.
    /// 3. Check if the results are correct.
    #[test]
    fn ut_position_column() {
        let position = Position::new(1, 1);
        assert_eq!(position.column(), 1);
    }

    /// UT test case for `Position::clone`.
    ///
    /// # Title
    /// ut_position_clone
    ///
    /// # Brief
    /// 1. Create a `Position`.
    /// 2. Call `Position::clone` to get a copy of `Position`.
    /// 3. Check if the results are correct.
    #[allow(clippy::redundant_clone)]
    #[test]
    fn ut_position_clone() {
        let position = Position::new(1, 1);
        let position = position.clone();
        assert_eq!(position.line, 1);
        assert_eq!(position.column, 1);
    }
}
