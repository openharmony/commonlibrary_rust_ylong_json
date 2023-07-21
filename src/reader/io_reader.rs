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

use super::{BytesReader, Cacheable, Position};
use std::io::{Error, ErrorKind, Read, Result};

/// Reader for reading I\O. This reader implements `BytesReader` trait and
/// `Cacheable` trait.
///
/// # Examples
/// ```not run
/// use std::fs::File;
/// use ylong_bytes_reader::{IoReader, BytesReader};
///
/// let file = File::open("./test.txt").unwrap();
/// let mut io_reader = IoReader::new(file);
/// let char = io_reader.next();
/// let char = io_reader.peek();
/// ```
pub(crate) struct IoReader<R: Read> {
    io: R,
    buf: Vec<u8>, // Buffer for storing read bytes.
    cur: usize,   // The position of the cursor in the current buf.
    idx: usize,   // A counter of all bytes that have been read.
    pos: Position,
    cache: Option<Cache>,
}

// A simple cache implementation for `IoReader`.
struct Cache {
    cache: Vec<u8>,
    pre: usize, // Last cached location.
}

impl Cache {
    /// Create a new `Cache`.
    fn new() -> Self {
        Self {
            cache: Vec::new(),
            pre: 0,
        }
    }
}

impl<R: Read> IoReader<R> {
    /// Create a new `IoReader` from the given I\O.
    pub(crate) fn new(io: R) -> Self {
        Self {
            io,
            buf: Vec::with_capacity(1024), // Default size is 1024.
            cur: 0,
            idx: 0,
            pos: Position::new(1, 1),
            cache: None,
        }
    }

    // Try to read some bytes from io to fill buf.
    fn read_bytes(&mut self) -> Result<bool> {
        unsafe {
            self.buf.set_len(1024);
        }
        loop {
            return match self.io.read(self.buf.as_mut_slice()) {
                Ok(0) => unsafe {
                    self.buf.set_len(0);
                    Ok(false)
                },
                Ok(n) => unsafe {
                    self.buf.set_len(n);
                    Ok(true)
                },
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(e) => Err(e),
            };
        }
    }

    // If there is not enough bytes in buf, try to read some bytes from io and
    // reset some parameters inside.
    fn load(&mut self) -> Result<bool> {
        if let Some(ref mut cacher) = self.cache {
            cacher.cache.extend_from_slice(&self.buf[cacher.pre..]);
            cacher.pre = 0;
        }
        let result = self.read_bytes();
        if let Ok(true) = result {
            self.cur = 0;
        }
        result
    }

    // Every time a user calls a cache-related interface, the cache content
    // needs to be updated in time.
    fn update_cache(&mut self) {
        if let Some(ref mut cacher) = self.cache {
            if self.cur > cacher.pre {
                cacher
                    .cache
                    .extend_from_slice(&self.buf[cacher.pre..self.cur]);
            }
            cacher.pre = self.cur;
        }
    }
}

impl<R: Read> BytesReader for IoReader<R> {
    type Error = Error;

    fn next(&mut self) -> Result<Option<u8>> {
        if self.cur == self.buf.len() {
            match self.load() {
                Ok(true) => {}
                Ok(false) => return Ok(None),
                Err(e) => return Err(e),
            }
        }

        let ch = self.buf[self.cur];
        self.cur += 1;
        self.idx += 1;

        if ch == b'\n' {
            self.pos.line += 1;
            self.pos.column = 1;
        } else {
            self.pos.column += 1;
        }

        Ok(Some(ch))
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        if self.cur == self.buf.len() {
            match self.load() {
                Ok(true) => {}
                Ok(false) => return Ok(None),
                Err(e) => return Err(e),
            }
        }

        Ok(Some(self.buf[self.cur]))
    }

    fn discard(&mut self) {
        if self.cur == self.buf.len() {
            match self.load() {
                Ok(true) => {}
                Ok(false) => return,
                Err(_) => return,
            }
        }

        let ch = self.buf[self.cur];
        self.cur += 1;
        self.idx += 1;

        if ch == b'\n' {
            self.pos.line += 1;
            self.pos.column = 1;
        } else {
            self.pos.column += 1;
        }
    }

    #[inline]
    fn index(&self) -> usize {
        self.idx
    }

    #[inline]
    fn position(&self) -> Position {
        self.pos.clone()
    }
}

impl<R: Read> Cacheable for IoReader<R> {
    fn start_caching(&mut self) {
        if let Some(ref mut cacher) = self.cache {
            cacher.cache.clear();
            cacher.pre = self.cur;
        } else {
            let mut cache = Cache::new();
            cache.pre = self.cur;
            self.cache = Some(cache);
        }
    }

    fn cached_len(&mut self) -> Option<usize> {
        self.update_cache();
        self.cache.as_ref().map(|c| c.cache.len())
    }

    fn cached_slice(&mut self) -> Option<&[u8]> {
        self.update_cache();
        self.cache.as_ref().map(|c| c.cache.as_slice())
    }

    fn cached_data(&mut self) -> Option<Vec<u8>> {
        self.update_cache();
        self.cache.as_ref().map(|c| c.cache.clone())
    }

    fn end_caching(&mut self) {
        self.cache = None;
    }

    fn take_cached_data(&mut self) -> Option<Vec<u8>> {
        self.update_cache();
        self.cache.take().map(|c| c.cache)
    }
}

#[cfg(test)]
mod ut_io_reader {
    use super::{BytesReader, Cacheable, IoReader};
    use std::cmp;
    use std::io::{ErrorKind, Read};

    struct TestIo {
        vec: Vec<u8>,
        idx: usize,
    }

    impl TestIo {
        fn new(vec: Vec<u8>) -> Self {
            Self { vec, idx: 0 }
        }
    }

    impl Read for TestIo {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.idx == self.vec.len() {
                return Ok(0);
            }
            let last = cmp::min(self.idx + buf.len(), self.vec.len());
            let len = last - self.idx;
            buf[..len].copy_from_slice(&self.vec[self.idx..last]);
            self.idx = last;
            Ok(len)
        }
    }

    struct TestWouldBlockIo {
        cnt: usize,
    }

    impl TestWouldBlockIo {
        fn new() -> Self {
            Self { cnt: 0 }
        }

        fn is_finished(&self) -> bool {
            self.cnt == 10
        }
    }

    impl Read for TestWouldBlockIo {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            if self.cnt < 10 {
                self.cnt += 1;
                return Err(ErrorKind::WouldBlock.into());
            }
            Ok(0)
        }
    }

    struct TestErrIo;

    impl TestErrIo {
        fn new() -> Self {
            Self
        }
    }

    impl Read for TestErrIo {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(ErrorKind::AddrInUse.into())
        }
    }

    /// UT test case for `IoReader::new`.
    ///
    /// # Title
    /// ut_io_reader_new
    ///
    /// # Brief
    /// 1. Call `IoReader::new`.
    /// 2. Check that parts of the return value are default values.
    #[test]
    fn ut_io_reader_new() {
        let io = TestIo::new(Vec::new());
        let io_reader = IoReader::new(io);

        assert_eq!(io_reader.cur, 0);
        assert_eq!(io_reader.idx, 0);
        assert_eq!(io_reader.pos.line, 1);
        assert_eq!(io_reader.pos.column, 1);
        assert_eq!(io_reader.buf.capacity(), 1024);
        assert!(io_reader.buf.is_empty());
        assert!(io_reader.cache.is_none());
    }

    /// UT test case for `IoReader::next`.
    ///
    /// # Title
    /// ut_test_case_io_reader_next
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::next`.
    /// 3. Check the return value against the following conditions:
    ///     - If the end is not read, it returns `Ok(Some(..))`, and the index
    ///     is moved backward; if the end is read, it returns `Ok(None)`, and
    ///     the index is not moved.
    #[test]
    fn ut_io_reader_next() {
        // Use TestIo.
        let io = TestIo::new(vec![1u8; 1025]);
        let mut io_reader = IoReader::new(io);
        assert_eq!(io_reader.next().unwrap(), Some(1));
        for _ in 0..1023 {
            let _ = io_reader.next().unwrap();
        }
        assert_eq!(io_reader.next().unwrap(), Some(1));
        assert_eq!(io_reader.next().unwrap(), None);

        // Use TestWouldBlockIo.
        let io = TestWouldBlockIo::new();
        let mut io_reader = IoReader::new(io);
        assert_eq!(io_reader.next().unwrap(), None);
        assert!(io_reader.io.is_finished());

        // Use TestErrIo
        let io = TestErrIo::new();
        let mut io_reader = IoReader::new(io);
        assert!(io_reader.next().is_err());
    }

    /// UT test case for `IoReader::peek`.
    ///
    /// # Title
    /// ut_io_reader_peek
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::peek`.
    /// 3. Check the return value against the following conditions:
    ///     - If the end is not read, it returns `Ok(Some(..))`; if the end is
    ///     read, it returns `Ok(None)`.
    #[test]
    fn ut_io_reader_peek() {
        // Use TestIo.
        let io = TestIo::new(vec![1u8; 1]);
        let mut io_reader = IoReader::new(io);
        assert_eq!(io_reader.peek().unwrap(), Some(1));
        assert_eq!(io_reader.peek().unwrap(), Some(1));
        assert_eq!(io_reader.next().unwrap(), Some(1));
        assert_eq!(io_reader.peek().unwrap(), None);

        // Use TestWouldBlockIo.
        let io = TestWouldBlockIo::new();
        let mut io_reader = IoReader::new(io);
        assert_eq!(io_reader.peek().unwrap(), None);
        assert!(io_reader.io.is_finished());

        // Use TestErrorIo.
        let io = TestErrIo::new();
        let mut io_reader = IoReader::new(io);
        assert!(io_reader.peek().is_err());
    }

    /// UT test case for `IoReader::discard`.
    ///
    /// # Title
    /// ut_io_reader_discard
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::discard`.
    /// 3. Check `index` against the following conditions:
    ///     - If the end is not read, the index is moved backward; if the end is
    ///     read, the index is not moved.
    #[test]
    fn ut_io_reader_discard() {
        let io = TestIo::new(vec![1u8; 1]);
        let mut io_reader = IoReader::new(io);
        assert_eq!(io_reader.index(), 0);
        io_reader.discard();
        assert_eq!(io_reader.index(), 1);
        io_reader.discard();
        assert_eq!(io_reader.index(), 1);
    }

    /// UT test case for `IoReader::index`.
    ///
    /// # Title
    /// ut_io_reader_index
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::index`.
    /// 3. Check if the `index` is correct.
    #[test]
    fn ut_io_reader_index() {
        let io = TestIo::new(vec![1u8; 1]);
        let io_reader = IoReader::new(io);
        assert_eq!(io_reader.index(), 0);
    }

    /// UT test case for `IoReader::position`.
    ///
    /// # Title
    /// ut_io_reader_position
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::position`.
    /// 3. Check the return value against the following conditions:
    ///     - If `'\n'` is read, the line number will increase and the column
    ///     number will return to 1; if other characters are read, the line
    ///     number will remain unchanged and the column number will increase.
    #[test]
    fn ut_io_reader_position() {
        let io = TestIo::new(vec![1u8, b'\n', 2, b'\n', 3]);
        let mut io_reader = IoReader::new(io);
        let position = io_reader.position();
        assert_eq!(position.line(), 1);
        assert_eq!(position.column(), 1);
        assert_eq!(io_reader.next().unwrap(), Some(1));

        // Use `next()`.
        let position = io_reader.position();
        assert_eq!(position.line(), 1);
        assert_eq!(position.column(), 2);
        assert_eq!(io_reader.next().unwrap(), Some(b'\n'));

        let position = io_reader.position();
        assert_eq!(position.line(), 2);
        assert_eq!(position.column(), 1);
        assert_eq!(io_reader.next().unwrap(), Some(2));

        // Use `peek()` and `discard()`.
        let position = io_reader.position();
        assert_eq!(position.line(), 2);
        assert_eq!(position.column(), 2);
        assert_eq!(io_reader.peek().unwrap(), Some(b'\n'));
        io_reader.discard();

        let position = io_reader.position();
        assert_eq!(position.line(), 3);
        assert_eq!(position.column(), 1);
        assert_eq!(io_reader.peek().unwrap(), Some(3));
        io_reader.discard();

        let position = io_reader.position();
        assert_eq!(position.line(), 3);
        assert_eq!(position.column(), 2);
        assert_eq!(io_reader.peek().unwrap(), None);
    }

    /// UT test case for `IoReader::start_caching`.
    ///
    /// # Title
    /// ut_io_reader_start_caching
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::start_caching`.
    /// 3. Check if `cache` is correct.
    #[test]
    fn ut_io_reader_start_caching() {
        let io = TestIo::new(vec![1]);
        let mut io_reader = IoReader::new(io);
        assert!(io_reader.cache.is_none());
        io_reader.start_caching();
        assert!(io_reader.cache.is_some());
        assert_eq!(io_reader.cached_len(), Some(0));

        assert_eq!(io_reader.next().unwrap(), Some(1));
        assert_eq!(io_reader.cached_len(), Some(1));
        io_reader.start_caching();
        assert_eq!(io_reader.cached_len(), Some(0));
    }

    /// UT test case for `IoReader::cached_len`.
    ///
    /// # Title
    /// ut_io_reader_cached_len
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::cached_len`.
    /// 3. Check the return value against the following conditions:
    ///     - Returns `None` if caching is not enabled, otherwise returns
    ///     `Some(..)`.
    #[test]
    fn ut_io_reader_cached_len() {
        let io = TestIo::new(Vec::new());
        let mut io_reader = IoReader::new(io);
        assert_eq!(io_reader.cached_len(), None);
        io_reader.start_caching();
        assert_eq!(io_reader.cached_len(), Some(0));
    }

    /// UT test case for `IoReader::cached_slice`.
    ///
    /// # Title
    /// ut_io_reader_cached_slice
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::cached_slice`.
    /// 3. Check the return value against the following conditions:
    ///     - Returns `None` if caching is not enabled, otherwise returns
    ///     `Some(..)`.
    #[test]
    fn ut_io_reader_cached_slice() {
        let io = TestIo::new(Vec::new());
        let mut io_reader = IoReader::new(io);
        assert_eq!(io_reader.cached_slice(), None);
        io_reader.start_caching();
        assert_eq!(io_reader.cached_slice(), Some([].as_slice()));

        // Test 1025 bytes.
        let mut input = vec![0; 1024];
        input.push(1);
        let io = TestIo::new(input);
        let mut io_reader = IoReader::new(io);
        for _ in 0..1023 {
            let _ = io_reader.next();
        }
        io_reader.start_caching();
        assert_eq!(io_reader.next().unwrap(), Some(0));
        assert_eq!(io_reader.next().unwrap(), Some(1));
        assert_eq!(io_reader.cached_len(), Some(2));
    }

    /// UT test case for `IoReader::cached_data`.
    ///
    /// # Title
    /// ut_io_reader_cached_slice
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::cached_data`.
    /// 3. Check the return value against the following conditions:
    ///     - Returns `None` if caching is not enabled, otherwise returns
    ///     `Some(..)`.
    #[test]
    fn ut_io_reader_cached_data() {
        let io = TestIo::new(Vec::new());
        let mut io_reader = IoReader::new(io);
        assert_eq!(io_reader.cached_data(), None);
        io_reader.start_caching();
        assert_eq!(io_reader.cached_data(), Some(Vec::new()));
    }

    /// UT test case for `IoReader::end_caching`.
    ///
    /// # Title
    /// ut_io_reader_end_caching
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::end_caching`.
    /// 3. Check if `cache` is correct.
    #[test]
    fn ut_io_reader_end_caching() {
        let io = TestIo::new(Vec::new());
        let mut io_reader = IoReader::new(io);
        io_reader.start_caching();
        assert!(io_reader.cache.is_some());
        io_reader.end_caching();
        assert!(io_reader.cache.is_none());
    }

    /// UT test case for `IoReader::take_cached_data`.
    ///
    /// # Title
    /// ut_io_reader_take_cached_data
    ///
    /// # Brief
    /// 1. Create a `IoReader`.
    /// 2. Call `IoReader::take_cached_data`.
    /// 3. Check if the return value is correct.
    #[test]
    fn ut_io_reader_take_cached_data() {
        let io = TestIo::new(Vec::new());
        let mut io_reader = IoReader::new(io);
        io_reader.start_caching();
        assert!(io_reader.cache.is_some());
        assert_eq!(io_reader.take_cached_data(), Some(Vec::new()));
        assert!(io_reader.cache.is_none());
    }
}
