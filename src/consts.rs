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

#![allow(dead_code)]
pub(crate) const COLON: u8 = b':';
pub(crate) const COMMA: u8 = b',';
pub(crate) const DECIMAL_POINT: u8 = b'.';
pub(crate) const LEFT_CURLY_BRACKET: u8 = b'{';
pub(crate) const LEFT_SQUARE_BRACKET: u8 = b'[';
pub(crate) const MINUS: u8 = b'-';
pub(crate) const PLUS: u8 = b'+';
pub(crate) const RIGHT_CURLY_BRACKET: u8 = b'}';
pub(crate) const RIGHT_SQUARE_BRACKET: u8 = b']';
pub(crate) const SPACE: u8 = b' ';

pub(crate) const ZERO: u8 = b'0';
pub(crate) const ONE: u8 = b'1';
pub(crate) const NINE: u8 = b'9';
pub(crate) const A_LOWER: u8 = b'a';
pub(crate) const A_UPPER: u8 = b'A';
pub(crate) const E_LOWER: u8 = b'e';
pub(crate) const E_UPPER: u8 = b'E';
pub(crate) const F_LOWER: u8 = b'f';
pub(crate) const F_UPPER: u8 = b'F';
pub(crate) const N_LOWER: u8 = b'n';
pub(crate) const T_LOWER: u8 = b't';

pub(crate) const WHITE_SPACE_SET: [u8; 4] =
    [SPACE, HT_UNICODE as u8, LF_UNICODE as u8, CR_UNICODE as u8];

pub(crate) const BS: u8 = b'b';
pub(crate) const BS_UNICODE: char = '\u{0008}';
pub(crate) const BS_UNICODE_U8: u8 = 0x08;
pub(crate) const HT: u8 = b't';
pub(crate) const HT_UNICODE: char = '\u{0009}';
pub(crate) const HT_UNICODE_U8: u8 = 0x09;
pub(crate) const FF: u8 = b'f';
pub(crate) const FF_UNICODE: char = '\u{000c}';
pub(crate) const FF_UNICODE_U8: u8 = 0x0c;
pub(crate) const CR: u8 = b'r';
pub(crate) const CR_UNICODE: char = '\u{000d}';
pub(crate) const CR_UNICODE_U8: u8 = 0x0d;
pub(crate) const LF: u8 = b'n';
pub(crate) const LF_UNICODE: char = '\u{000a}';
pub(crate) const LF_UNICODE_U8: u8 = 0x0a;
pub(crate) const UNICODE: u8 = b'u';
pub(crate) const QUOTATION_MARK: u8 = b'\"';
pub(crate) const REVERSE_SOLIDUS: u8 = b'\\';
pub(crate) const SOLIDUS: u8 = b'/';

pub(crate) const JSON_REVERSE_SOLIDUS: &[u8] = b"\\\\";
pub(crate) const JSON_QUOTATION_MARK: &[u8] = b"\\\"";
pub(crate) const JSON_BS: &[u8] = b"\\b";
pub(crate) const JSON_FF: &[u8] = b"\\f";
pub(crate) const JSON_LF: &[u8] = b"\\n";
pub(crate) const JSON_CR: &[u8] = b"\\r";
pub(crate) const JSON_HT: &[u8] = b"\\t";

pub(crate) const NULL_STR: &[u8] = b"null";
pub(crate) const NULL_LEFT_STR: &[u8] = b"ull";
pub(crate) const FALSE_STR: &[u8] = b"false";
pub(crate) const FALSE_LEFT_STR: &[u8] = b"alse";
pub(crate) const TRUE_STR: &[u8] = b"true";
pub(crate) const TRUE_LEFT_STR: &[u8] = b"rue";
pub(crate) const UNICODE_START_STR: &[u8] = b"\\u";
pub(crate) const COLON_STR: &[u8] = b":";
pub(crate) const COMMA_STR: &[u8] = b",";
pub(crate) const FOUR_SPACES_STR: &[u8] = b"    ";
pub(crate) const LEFT_CURLY_BRACKET_STR: &[u8] = b"{";
pub(crate) const LEFT_SQUARE_BRACKET_STR: &[u8] = b"[";
pub(crate) const LINE_FEED_STR: &[u8] = b"\n";
pub(crate) const QUOTATION_MARK_STR: &[u8] = b"\"";
pub(crate) const RIGHT_CURLY_BRACKET_STR: &[u8] = b"}";
pub(crate) const RIGHT_SQUARE_BRACKET_STR: &[u8] = b"]";
pub(crate) const SPACE_STR: &[u8] = b" ";

pub(crate) const RECURSION_LIMIT: u32 = 128;

// Improves the string read rate by looking up tables.
pub(crate) static ESCAPE: [bool; 256] = {
    const CT: bool = true; // Control character \x00..=\x1F
    const QU: bool = true; // Quotation mark \x22
    const BS: bool = true; // Backslash \x5C
    const __: bool = false; // Other character
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
        __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

// TODO: Consider modifying the structure of PRINT_MAP.
#[cfg(not(feature = "ascii_only"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrintMapItem<'a> {
    Other,
    Control,
    Special(&'a [u8]),
}

// Improves the string output rate by looking up the table.
#[cfg(not(feature = "ascii_only"))]
pub(crate) static PRINT_MAP: [PrintMapItem; 256] = {
    const BS: PrintMapItem = PrintMapItem::Special(b"\\b"); // BS 退格 \x08
    const HT: PrintMapItem = PrintMapItem::Special(b"\\t"); // HT 水平定位符 \x09
    const LF: PrintMapItem = PrintMapItem::Special(b"\\n"); // LF 换行 \x0A
    const FF: PrintMapItem = PrintMapItem::Special(b"\\f"); // FF 换页 \x0C
    const CR: PrintMapItem = PrintMapItem::Special(b"\\r"); // CR 归位 \x0D
    const QU: PrintMapItem = PrintMapItem::Special(b"\\\""); // 双引号 \x22
    const SO: PrintMapItem = PrintMapItem::Special(b"/"); // 斜杠 \x2F
    const RS: PrintMapItem = PrintMapItem::Special(b"\\\\"); // 反斜杠 \x5C
    const CT: PrintMapItem = PrintMapItem::Control; // 控制字符 \x00..=\x1F
    const __: PrintMapItem = PrintMapItem::Other; // 其他字符
    [
        //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
        CT, CT, CT, CT, CT, CT, CT, CT, BS, HT, LF, CT, FF, CR, CT, CT, // 0
        CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
        __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, SO, // 2
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
        __, __, __, __, __, __, __, __, __, __, __, __, RS, __, __, __, // 5
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
        __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
    ]
};

#[cfg(not(feature = "ascii_only"))]
#[cfg(test)]
mod ut_consts {
    use crate::consts::PrintMapItem;

    /// UT test case for `PrintMapItem::clone`.
    ///
    /// # Title
    /// ut_print_map_item_clone
    ///
    /// # Brief
    /// 1. Creates a `PrintMapItem`.
    /// 2. Calls `PrintMapItem::clone`.
    /// 3. Checks if the results are correct.
    #[allow(clippy::clone_on_copy)]
    #[test]
    fn ut_print_map_item_clone() {
        let item = PrintMapItem::Other;
        let item = item.clone();
        assert_eq!(item, PrintMapItem::Other);

        let item = PrintMapItem::Control;
        let item = item.clone();
        assert_eq!(item, PrintMapItem::Control);

        let item = PrintMapItem::Special(b"abc");
        let item = item.clone();
        assert_eq!(item, PrintMapItem::Special(b"abc"));
    }

    /// UT test case for `PrintMapItem::copy`.
    ///
    /// # Title
    /// ut_print_map_item_copy
    ///
    /// # Brief
    /// 1. Creates a `PrintMapItem`.
    /// 2. Calls `PrintMapItem::copy`.
    /// 3. Checks if the results are correct.
    #[test]
    fn ut_print_map_item_copy() {
        let item1 = PrintMapItem::Other;
        let _item2 = item1;
        assert_eq!(item1, PrintMapItem::Other);

        let item1 = PrintMapItem::Control;
        let _item2 = item1;
        assert_eq!(item1, PrintMapItem::Control);

        let item1 = PrintMapItem::Special(b"abc");
        let _item2 = item1;
        assert_eq!(item1, PrintMapItem::Special(b"abc"));
    }
}
