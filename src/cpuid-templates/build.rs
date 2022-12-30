// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::convert::TryFrom;
use std::io::Write;

use construct::Inline;
use proc_macro2::TokenStream;
use quote::quote;

/// In tests we assert that the time it took to instantiate the template with inline code is less
/// than `DELTA*x` where `x` is the time it took to deserialize the template from json.
const DELTA: f32 = 0.3f32;
/// In tests we assert that the time it took to instantiate the template with inline code is less
/// than `MAX_MICROS` microseconds.
const MAX_MICROS: u64 = 150;

/// License to write into auto-generated `lib.rs`.
const LICENSE: &str = "// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights \
                       Reserved.\n// SPDX-License-Identifier: Apache-2.0\n\n";

fn main() {
    // We sort the entries, so our code generation is deterministic e.g. generated code only
    // changes when we change our templates.
    let mut template_paths = std::fs::read_dir("./templates")
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect::<Vec<_>>();
    template_paths.sort_by_key(|entry| entry.path());

    let (functions, tests): (TokenStream, TokenStream) = template_paths
        .into_iter()
        .map(|template_entry| {
            let template_path = template_entry.path();
            // Re-build if this template file changed.
            println!("cargo:rerun-if-changed={}", template_path.display());
            // Get file contents as string.
            let string = std::fs::read_to_string(&template_path).unwrap();
            // Deserialize json string to cpuid structure.
            let raw_cpuid = serde_json::from_str::<cpuid::RawCpuid>(&string).unwrap();
            let cpuid = cpuid::Cpuid::try_from(raw_cpuid).unwrap();
            // Get rust code that allocate this specific cpuid structure.
            let inline = cpuid.inline();

            // Get identifier
            let identifier = template_path
                .as_path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();

            // Create construction code file
            {
                // Create `<template>.in` file under `src/`
                let mut template_file = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(format!("./src/{}.in", identifier))
                    .unwrap();
                // Write to template file
                template_file
                    .write_all(inline.to_string().as_bytes())
                    .unwrap();
            }

            // Append static
            let function = {
                let ident = quote::format_ident!("{}", identifier);
                let file = format!("{}.in", identifier);

                quote! {
                    pub fn #ident() -> Cpuid {
                        include!(#file)
                    }
                }
            };

            // Create initialization speed test
            let tests = {
                let test_fn = quote::format_ident!("{}_test", identifier);
                let path = template_path.as_path().to_str().unwrap();
                let ident = quote::format_ident!("{}", identifier);
                quote! {
                    #[test]
                    fn #test_fn() {
                        // Create template from file
                        let deserialize_elapsed = {
                            let string = std::fs::read_to_string(#path).unwrap();
                            let now = std::time::Instant::now();
                            let raw = serde_json::from_str::<RawCpuid>(&string).unwrap();
                            let _cpuid = Cpuid::try_from(raw).unwrap();
                            now.elapsed()
                        };
                        dbg!(&deserialize_elapsed);

                        // Create template from allocation
                        let allocate_elapsed = {
                            let now = std::time::Instant::now();
                            let _ = #ident();
                            now.elapsed()
                        };
                        dbg!(&allocate_elapsed);

                        assert!(allocate_elapsed < deserialize_elapsed.mul_f32(#DELTA));
                        assert!(allocate_elapsed < Duration::from_micros(#MAX_MICROS));
                    }
                }
            };

            (function, tests)
        })
        .unzip();

    let lib = quote! {
        use cpuid::{
            Cpuid,
            KvmCpuidFlags,
            CpuidKey,
            CpuidEntry,
            CCpuidResult,
            IntelCpuid
        };

        #functions

        #[cfg(test)]
        mod tests {
            use std::convert::TryFrom;
            use std::time::Duration;

            use cpuid::{Cpuid, RawCpuid};

            use super::*;
            #tests
        }
    };

    // Make lib.rs
    let mut lib_file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("./src/lib.rs")
        .unwrap();
    lib_file.write_all(LICENSE.as_bytes()).unwrap();
    lib_file.write_all(lib.to_string().as_bytes()).unwrap();

    // Format lib.rs
    std::process::Command::new("cargo")
        .args([
            "fmt",
            "--",
            "--config=comment_width=100,wrap_comments=true,format_code_in_doc_comments=true,\
             format_strings=true,imports_granularity=Module,normalize_comments=true,\
             normalize_doc_attributes=true,group_imports=StdExternalCrate",
        ])
        .output()
        .unwrap();
}
