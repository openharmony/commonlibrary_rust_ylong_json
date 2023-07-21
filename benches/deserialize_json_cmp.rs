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

//! Benchmarks for the json deserialization

#![feature(test)]

mod task_helpers;

#[cfg(test)]
mod serialize_cmp {
    extern crate test;
    use crate::task_helpers::*;
    use test::Bencher;

    use serde_json::Value;
    use std::str::FromStr;
    use ylong_json::JsonValue;

    #[bench]
    fn null_deserialize_perf_ylong_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value = JsonValue::from_str(NULL_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn boolean_deserialize_perf_ylong_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value = JsonValue::from_str(BOOLEAN_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn number_deserialize_perf_ylong_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value = JsonValue::from_str(NUMBER_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn string_deserialize_perf_ylong_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value = JsonValue::from_str(STRING_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn array_deserialize_perf_ylong_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value = JsonValue::from_str(ARRAY_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn object_deserialize_perf_ylong_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value = JsonValue::from_str(OBJECT_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn exp1_deserialize_perf_ylong_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value = JsonValue::from_str(RFC7159_EXAMPLE1).unwrap();
            }
        });
    }

    #[bench]
    fn exp2_deserialize_perf_ylong_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value = JsonValue::from_str(RFC7159_EXAMPLE2).unwrap();
            }
        });
    }

    #[bench]
    fn exp3_deserialize_perf_ylong_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value = JsonValue::from_str(JSON_PARSE_TEST).unwrap();
            }
        });
    }

    #[bench]
    fn exp4_deserialize_perf_ylong_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value = JsonValue::from_str(LONG_KEY_VALUE).unwrap();
            }
        });
    }

    #[bench]
    fn null_deserialize_perf_serde_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value: Value = serde_json::from_str(NULL_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn boolean_deserialize_perf_serde_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value: Value = serde_json::from_str(BOOLEAN_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn number_deserialize_perf_serde_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value: Value = serde_json::from_str(NUMBER_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn string_deserialize_perf_serde_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value: Value = serde_json::from_str(STRING_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn array_deserialize_perf_serde_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value: Value = serde_json::from_str(ARRAY_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn object_deserialize_perf_serde_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value: Value = serde_json::from_str(OBJECT_EXAMPLE).unwrap();
            }
        });
    }

    #[bench]
    fn exp1_deserialize_perf_serde_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value: Value = serde_json::from_str(RFC7159_EXAMPLE1).unwrap();
            }
        });
    }

    #[bench]
    fn exp2_deserialize_perf_serde_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value: Value = serde_json::from_str(RFC7159_EXAMPLE2).unwrap();
            }
        });
    }

    #[bench]
    fn exp3_deserialize_perf_serde_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value: Value = serde_json::from_str(JSON_PARSE_TEST).unwrap();
            }
        });
    }

    #[bench]
    fn exp4_deserialize_perf_serde_json(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..LOOPS_NUM {
                let _value: Value = serde_json::from_str(LONG_KEY_VALUE).unwrap();
            }
        });
    }
}
