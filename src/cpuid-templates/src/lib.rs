// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use cpuid::intel::*;
use cpuid::{Cpuid, KvmCpuidFlags};
pub fn c3() -> Cpuid {
    include!("c3.in")
}
pub fn t2() -> Cpuid {
    include!("t2.in")
}
pub fn t2s() -> Cpuid {
    include!("t2s.in")
}
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::time::Duration;

    use cpuid::{Cpuid, RawCpuid};

    use super::*;
    #[test]
    fn c3_test() {
        let deserialize_elapsed = {
            let string = std::fs::read_to_string("./templates/c3.json").unwrap();
            let now = std::time::Instant::now();
            let raw = serde_json::from_str::<RawCpuid>(&string).unwrap();
            let _cpuid = Cpuid::try_from(raw).unwrap();
            now.elapsed()
        };
        dbg!(&deserialize_elapsed);
        let allocate_elapsed = {
            let now = std::time::Instant::now();
            let _ = c3();
            now.elapsed()
        };
        dbg!(&allocate_elapsed);
        assert!(allocate_elapsed < deserialize_elapsed.mul_f32(0.3f32));
        assert!(allocate_elapsed < Duration::from_micros(100u64));
    }
    #[test]
    fn t2_test() {
        let deserialize_elapsed = {
            let string = std::fs::read_to_string("./templates/t2.json").unwrap();
            let now = std::time::Instant::now();
            let raw = serde_json::from_str::<RawCpuid>(&string).unwrap();
            let _cpuid = Cpuid::try_from(raw).unwrap();
            now.elapsed()
        };
        dbg!(&deserialize_elapsed);
        let allocate_elapsed = {
            let now = std::time::Instant::now();
            let _ = t2();
            now.elapsed()
        };
        dbg!(&allocate_elapsed);
        assert!(allocate_elapsed < deserialize_elapsed.mul_f32(0.3f32));
        assert!(allocate_elapsed < Duration::from_micros(100u64));
    }
    #[test]
    fn t2s_test() {
        let deserialize_elapsed = {
            let string = std::fs::read_to_string("./templates/t2s.json").unwrap();
            let now = std::time::Instant::now();
            let raw = serde_json::from_str::<RawCpuid>(&string).unwrap();
            let _cpuid = Cpuid::try_from(raw).unwrap();
            now.elapsed()
        };
        dbg!(&deserialize_elapsed);
        let allocate_elapsed = {
            let now = std::time::Instant::now();
            let _ = t2s();
            now.elapsed()
        };
        dbg!(&allocate_elapsed);
        assert!(allocate_elapsed < deserialize_elapsed.mul_f32(0.3f32));
        assert!(allocate_elapsed < Duration::from_micros(100u64));
    }
}
