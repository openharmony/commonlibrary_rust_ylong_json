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

use super::{BytesReader, Cacheable, NBytesReadable, Position, RemainderCountable};
use std::convert::Infallible;

/// Reader for reading slices. This reader implements `BytesReader` trait,
/// `Cacheable` trait and `Countable` trait.
///
/// let slice = "Hello World";
/// let mut reader = SliceReader::new(slice.as_bytes());
/// assert_eq!(reader.next(), Ok(Some(b'H')));
/// assert_eq!(reader.peek(), Ok(Some(b'e')));
/// assert_eq!(reader.peek(), Ok(Some(b'e')));
pub(crate) struct SliceReader<'a> {
    slice: &'a [u8],
    index: usize,         // A cursor to the next character to be read.
    cache: Option<Cache>, // A cache for storing accumulated characters.
}

// A simple cache implementation for `SliceReader`. We just need to save a
// starting position.
struct Cache(usize);

impl<'a> SliceReader<'a> {
    /// Create a new `SliceReader` from the given slice.
    ///
    /// let slice = "Hello World";
    /// let reader = SliceReader::new(slice.as_bytes());
    #[inline]
    pub(crate) fn new(slice: &'a [u8]) -> Self {
        Self {
            slice,
            index: 0,
            cache: None,
        }
    }
}

impl<'a> BytesReader for SliceReader<'a> {
    type Error = Infallible; // Use Infallible because no error will be returned in SliceReader.

    #[inline]
    fn next(&mut self) -> Result<Option<u8>, Self::Error> {
        if self.index >= self.slice.len() {
            return Ok(None);
        }
        let ch = self.slice[self.index];
        self.index += 1;
        Ok(Some(ch))
    }

    #[inline]
    fn peek(&mut self) -> Result<Option<u8>, Self::Error> {
        if self.index >= self.slice.len() {
            return Ok(None);
        }
        Ok(Some(self.slice[self.index]))
    }

    #[inline]
    fn discard(&mut self) {
        if self.index < self.slice.len() {
            self.index += 1;
        }
    }

    #[inline]
    fn index(&self) -> usize {
        self.index
    }

    fn position(&self) -> Position {
        // The traversal method is used to calculate the `Position`, which
        // is expensive, and it is not recommended to call it frequently.
        let index = core::cmp::min(self.index, self.slice.len());

        let mut position = Position { line: 1, column: 1 };
        for i in 0..index {
            match self.slice[i] {
                b'\n' => {
                    position.line += 1;
                    position.column = 1;
                }
                _ => {
                    position.column += 1;
                }
            }
        }
        position
    }
}

impl<'a> Cacheable for SliceReader<'a> {
    #[inline]
    fn start_caching(&mut self) {
        self.cache = Some(Cache(self.index));
    }

    #[inline]
    fn cached_len(&mut self) -> Option<usize> {
        self.cache.as_ref().map(|c| self.index - c.0)
    }

    #[inline]
    fn cached_slice(&mut self) -> Option<&[u8]> {
        self.cache.as_ref().map(|c| &self.slice[c.0..self.index])
    }

    #[inline]
    fn cached_data(&mut self) -> Option<Vec<u8>> {
        self.cache
            .as_ref()
            .map(|c| self.slice[c.0..self.index].to_vec())
    }

    #[inline]
    fn end_caching(&mut self) {
        self.cache = None;
    }

    #[inline]
    fn take_cached_data(&mut self) -> Option<Vec<u8>> {
        self.cache
            .take()
            .map(|c| self.slice[c.0..self.index].to_vec())
    }
}

impl<'a> RemainderCountable for SliceReader<'a> {
    #[inline]
    fn remainder_len(&self) -> usize {
        self.slice.len() - self.index
    }

    #[inline]
    fn remainder_slice(&self) -> &[u8] {
        &self.slice[self.index..]
    }

    #[inline]
    fn remainder_data(&self) -> Vec<u8> {
        self.remainder_slice().to_vec()
    }
}

impl<'a> NBytesReadable for SliceReader<'a> {
    fn next_n(&mut self, n: usize) -> Result<Option<&[u8]>, Self::Error> {
        if self.index + n > self.slice.len() {
            return Ok(None);
        }
        let result = &self.slice[self.index..self.index + n];
        self.index += n;
        Ok(Some(result))
    }

    fn peek_n(&mut self, n: usize) -> Result<Option<&[u8]>, Self::Error> {
        if self.index + n > self.slice.len() {
            return Ok(None);
        }
        Ok(Some(&self.slice[self.index..self.index + n]))
    }

    fn discard_n(&mut self, n: usize) {
        if self.index + n > self.slice.len() {
            return;
        }
        self.index += n;
    }
}

#[cfg(test)]
mod ut_slice_reader {
    use super::{BytesReader, Cacheable, NBytesReadable, RemainderCountable, SliceReader};

    /// UT test case for `SliceReader::new`.
    ///
    /// # Title
    /// ut_slice_reader_new
    ///
    /// # Brief
    /// 1. Call `SliceReader::new`.
    /// 2. Check that parts of the return value are default values.
    #[test]
    fn ut_slice_reader_new() {
        let slice = "A";
        let slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.slice, slice.as_bytes());
        assert_eq!(slice_reader.index, 0);
        assert!(slice_reader.cache.is_none());
    }

    /// UT test case for `SliceReader::next`.
    ///
    /// # Title
    /// ut_slice_reader_next
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::next`.
    /// 3. Check the return value against the following conditions:
    ///     - If the end is not read, it returns `Ok(Some(..))`, and the index
    ///     is moved backward; if the end is read, it returns `Ok(None)`, and
    ///     the index is not moved.
    #[test]
    fn ut_slice_reader_next() {
        let slice = "A";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.next(), Ok(Some(b'A')));
        assert_eq!(slice_reader.next(), Ok(None));
    }

    /// UT test case for `SliceReader::peek`.
    ///
    /// # Title
    /// ut_slice_reader_peek
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::peek`.
    /// 3. Check the return value against the following conditions:
    ///     - If the end is not read, it returns `Ok(Some(..))`; if the end is
    ///     read, it returns `Ok(None)`.
    #[test]
    fn ut_slice_reader_peek() {
        let slice = "A";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.peek(), Ok(Some(b'A')));
        assert_eq!(slice_reader.peek(), Ok(Some(b'A')));
        assert_eq!(slice_reader.next(), Ok(Some(b'A')));
        assert_eq!(slice_reader.peek(), Ok(None));
    }

    /// UT test case for `SliceReader::discard`.
    ///
    /// # Title
    /// ut_slice_reader_discard
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::discard`.
    /// 3. Check `index` against the following conditions:
    ///     - If the end is not read, the index is moved backward; if the end is
    ///     read, the index is not moved.
    #[test]
    fn ut_slice_reader_discard() {
        let slice = "A";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.index, 0);
        slice_reader.discard();
        assert_eq!(slice_reader.index, 1);
        slice_reader.discard();
        assert_eq!(slice_reader.index, 1);
    }

    /// UT test case for `SliceReader::index`.
    ///
    /// # Title
    /// ut_slice_reader_index
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::index`.
    /// 3. Check if the `index` is correct.
    #[test]
    fn ut_slice_reader_index() {
        let slice = "A";
        let slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.index(), 0);
    }

    /// UT test case for `SliceReader::position`.
    ///
    /// # Title
    /// ut_slice_reader_position
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::position`.
    /// 3. Check the return value against the following conditions:
    ///     - If `'\n'` is read, the line number will increase and the column
    ///     number will return to 1; if other characters are read, the line
    ///     number will remain unchanged and the column number will increase.
    #[test]
    fn ut_slice_reader_position() {
        let slice = "A\nB";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        let position = slice_reader.position();
        assert_eq!(position.line(), 1);
        assert_eq!(position.column(), 1);
        assert_eq!(slice_reader.next(), Ok(Some(b'A')));

        let position = slice_reader.position();
        assert_eq!(position.line(), 1);
        assert_eq!(position.column(), 2);
        assert_eq!(slice_reader.next(), Ok(Some(b'\n')));

        let position = slice_reader.position();
        assert_eq!(position.line(), 2);
        assert_eq!(position.column(), 1);
        assert_eq!(slice_reader.next(), Ok(Some(b'B')));

        let position = slice_reader.position();
        assert_eq!(position.line(), 2);
        assert_eq!(position.column(), 2);

        assert_eq!(slice_reader.next(), Ok(None));
        let position = slice_reader.position();
        assert_eq!(position.line(), 2);
        assert_eq!(position.column(), 2);
    }

    /// UT test case for `SliceReader::start_caching`.
    ///
    /// # Title
    /// ut_slice_reader_start_caching
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::start_caching`.
    /// 3. Check if `cache` is correct.
    #[test]
    fn ut_slice_reader_start_caching() {
        let slice = "A";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        assert!(slice_reader.cache.is_none());
        slice_reader.start_caching();
        assert_eq!(slice_reader.cache.as_ref().unwrap().0, 0);
    }

    /// UT test case for `SliceReader::cached_len`.
    ///
    /// # Title
    /// ut_slice_reader_cached_len
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::cached_len`.
    /// 3. Check the return value against the following conditions:
    ///     - Returns `None` if caching is not enabled, otherwise returns
    ///     `Some(..)`.
    #[test]
    fn ut_slice_reader_cached_len() {
        let slice = "A";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.cached_len(), None);
        slice_reader.start_caching();
        assert_eq!(slice_reader.cached_len(), Some(0));
    }

    /// UT test case for `SliceReader::cached_slice`.
    ///
    /// # Title
    /// ut_slice_reader_cached_slice
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::cached_slice`.
    /// 3. Check the return value against the following conditions:
    ///     - Returns `None` if caching is not enabled, otherwise returns
    ///     `Some(..)`.
    #[test]
    fn ut_slice_reader_cached_slice() {
        let slice = "A";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.cached_slice(), None);
        slice_reader.start_caching();
        assert_eq!(slice_reader.cached_slice(), Some([].as_slice()));
    }

    /// UT test case for `SliceReader::cached_data`.
    ///
    /// # Title
    /// ut_slice_reader_cached_slice
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::cached_data`.
    /// 3. Check the return value against the following conditions:
    ///     - Returns `None` if caching is not enabled, otherwise returns
    ///     `Some(..)`.
    #[test]
    fn ut_slice_reader_cached_data() {
        let slice = "A";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.cached_data(), None);
        slice_reader.start_caching();
        assert_eq!(slice_reader.cached_data(), Some(Vec::new()));
    }

    /// UT test case for `SliceReader::end_caching`.
    ///
    /// # Title
    /// ut_slice_reader_end_caching
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::end_caching`.
    /// 3. Check if `cache` is correct.
    #[test]
    fn ut_slice_reader_end_caching() {
        let slice = "A";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        slice_reader.start_caching();
        assert!(slice_reader.cache.is_some());
        slice_reader.end_caching();
        assert!(slice_reader.cache.is_none());
    }

    /// UT test case for `SliceReader::take_cached_data`.
    ///
    /// # Title
    /// ut_slice_reader_take_cached_data
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::take_cached_data`.
    /// 3. Check if the return value is correct.
    #[test]
    fn ut_slice_reader_take_cached_data() {
        let slice = "A";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        slice_reader.start_caching();
        assert!(slice_reader.cache.is_some());
        assert_eq!(slice_reader.take_cached_data(), Some(Vec::new()));
        assert!(slice_reader.cache.is_none());
    }

    /// UT test case for `SliceReader::remainder_len`.
    ///
    /// # Title
    /// ut_slice_reader_remainder_len
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::remainder_len`.
    /// 3. Check if the return value is correct.
    #[test]
    fn ut_slice_reader_remainder_len() {
        let slice = "A";
        let slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.remainder_len(), 1);
    }

    /// UT test case for `SliceReader::remainder_slice`.
    ///
    /// # Title
    /// ut_slice_reader_remainder_slice
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::remainder_slice`.
    /// 3. Check if the return value is correct.
    #[test]
    fn ut_slice_reader_remainder_slice() {
        let slice = "A";
        let slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.remainder_slice(), slice.as_bytes());
    }

    /// UT test case for `SliceReader::remainder_data`.
    ///
    /// # Title
    /// ut_slice_reader_remainder_slice
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::remainder_slice`.
    /// 3. Check if the return value is correct.
    #[test]
    fn ut_slice_reader_remainder_data() {
        let slice = "A";
        let slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.remainder_data(), slice.as_bytes().to_vec());
    }

    /// UT test case for `SliceReader::next_n`.
    ///
    /// # Title
    /// ut_slice_reader_next_n
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::next_n`.
    /// 3. Check if the return value is correct.
    #[test]
    fn ut_slice_reader_next_n() {
        let slice = "ABC";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.next_n(2).unwrap(), Some("AB".as_bytes()));
        assert_eq!(slice_reader.next_n(2).unwrap(), None);
    }

    /// UT test case for `SliceReader::peek_n`.
    ///
    /// # Title
    /// ut_slice_reader_peek_n
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::peek_n`.
    /// 3. Check if the return value is correct.
    #[test]
    fn ut_slice_reader_peek_n() {
        let slice = "ABC";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        assert_eq!(slice_reader.peek_n(2).unwrap(), Some("AB".as_bytes()));
        assert_eq!(slice_reader.peek_n(4).unwrap(), None);
    }

    /// UT test case for `SliceReader::discard_n`.
    ///
    /// # Title
    /// ut_slice_reader_discard_n
    ///
    /// # Brief
    /// 1. Create a `SliceReader`.
    /// 2. Call `SliceReader::discard_n`.
    /// 3. Check if the return value is correct.
    #[test]
    fn ut_slice_reader_discard_n() {
        let slice = "ABC";
        let mut slice_reader = SliceReader::new(slice.as_bytes());
        slice_reader.discard_n(2);
        assert_eq!(slice_reader.next().unwrap(), Some(b'C'));
    }
}
