// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{fmt, fs};

use bincode::ErrorKind;
use cpuid::Cpuid;
use logger::{error, info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::cpu::CustomCpuConfigurationError;

/// Contains all CPU feature configuration for CPUID and MSRs (x86) when using KVM.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomCpuConfiguration {
    /// File path that contains a full configuration set pf architecture-general features.
    pub base_arch_features_template_path: String,
    /// TODO - base_special_features_template_path currently ignored
    /// File path that contains a full configuration set for "special" registers.
    pub base_special_features_template_path: String,
    /// List of entries for CPU features to be configured for a vCPU.
    pub cpu_feature_overrides: Vec<CpuConfigurationAttribute>,
}

/// Configuration attribute
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CpuConfigurationAttribute {
    /// Symbolic name of the CPU feature.
    pub name: String,
    /// Flag to specify whether to enable or disable the feature on a vCPU.
    pub is_enabled: bool,
}

/// Errors associated with configuring the microVM.
#[derive(Debug, PartialEq, Error)]
pub enum CpuConfigError {
    /// Unknown/Undefined CPU feature name
    #[error("Unknown or undefined CPU feature name")]
    UndefinedCpuFeatureName,
    #[error("CPU feature override for [{0}] is not supported")]
    UnsupportedCpuFeatureOverride(String),
}

impl Display for CustomCpuConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut cpu_config_entries: String = String::from("CPU Feature Override Entries:\n");
        for config_entry in self.cpu_feature_overrides.as_slice().into_iter() {
            cpu_config_entries = format!(
                "{}(name: {}, is_enabled:{})\n",
                cpu_config_entries, config_entry.name, config_entry.is_enabled
            );
        }

        let cpu_base_config = format!(
            "{}\n{}\n{}\n{}\n",
            "General architecture base configuration template path: ",
            "Special registers base configuration: ",
            self.base_arch_features_template_path,
            self.base_special_features_template_path,
        );

        write!(f, "{}\n{}\n", cpu_base_config, cpu_config_entries)
    }
}

pub fn load_template_file(file_path_str: &str) -> Result<Cpuid, CustomCpuConfigurationError> {
    let file_path: &Path = Path::new(file_path_str);

    if file_path.is_file() {
        load_cpuid_template_file(file_path)
    } else {
        error!("Template path [{}] is not a file", file_path_str);
        Err(CustomCpuConfigurationError::InvalidFilePath(String::from(
            file_path_str,
        )))
    }
}

fn load_cpuid_template_file(
    template_file_path: &Path,
) -> Result<Cpuid, CustomCpuConfigurationError> {
    let extension_option: Option<&str> = template_file_path.extension().and_then(OsStr::to_str);

    match extension_option {
        // Without a file extension, assume the file is a binary formatted template file,
        // and attempt to serialize the binary data into a Cpuid instance
        None => {
            warn!("No file extension found on CPUID configuration file, assuming binary file");
            read_binary_file(String::from(template_file_path.to_str().unwrap()))
        }
        // A file extension exists on the file, if the file is JSON, it will need to be
        // deserialized into
        Some(file_ext) => {
            if file_ext.eq_ignore_ascii_case("json") {
                read_json_file(String::from(template_file_path.to_str().unwrap()))
            } else {
                Err(CustomCpuConfigurationError::InvalidFileType(String::from(
                    template_file_path.to_str().unwrap(),
                )))
            }
        }
    }
}

fn read_json_file(json_template_file_path: String) -> Result<Cpuid, CustomCpuConfigurationError> {
    let cpuid_json_string = fs::read_to_string(&json_template_file_path)
        .expect(format!("Unable to read json file [{}]", json_template_file_path).as_str());

    let cpuid_deserialization_result = serde_json::from_str(&cpuid_json_string.as_str());

    if cpuid_deserialization_result.is_ok() {
        info!(
            "Loaded JSON file [{}] successfully",
            json_template_file_path
        );
        Ok(cpuid_deserialization_result.unwrap())
    } else {
        error!("Failed to load JSON file [{}]", json_template_file_path);
        Err(CustomCpuConfigurationError::InvalidFileFormat(
            json_template_file_path,
        ))
    }
}

fn read_binary_file(file_name: String) -> Result<Cpuid, CustomCpuConfigurationError> {
    let file_path = Path::new(&file_name);
    warn!(
        "Loading binary file for CPU configuration - [{}]",
        file_path.to_str().unwrap()
    );
    let mut vm_config_file =
        File::open(file_path).expect(&format!("Error reading from file {}", file_name));

    let metadata = fs::metadata(&file_name)
        .expect(format!("unable to read metadata for [{}]", file_name).as_str());
    let mut cpu_config_buffer = vec![0; metadata.len() as usize];
    vm_config_file
        .read(&mut cpu_config_buffer)
        .expect(&format!("Failed to read binary file [{}]", file_name));
    let cpuid_result: Result<Cpuid, bincode::Error> = bincode::deserialize(&cpu_config_buffer);

    match cpuid_result {
        Ok(cpuid) => Ok(cpuid),
        Err(err) => {
            match *err {
                ErrorKind::Io(error) => {
                    error!("IO Error: {}", error);
                }
                ErrorKind::InvalidUtf8Encoding(error) => {
                    error!("Utf8 encoding error: {}", error);
                }
                ErrorKind::InvalidBoolEncoding(error) => {
                    error!("u8 bool encoding error: {}", error);
                }
                ErrorKind::InvalidCharEncoding => {
                    error!("Char encoding Error");
                }
                ErrorKind::InvalidTagEncoding(_error) => {
                    error!("Tag encoding Error");
                }
                ErrorKind::DeserializeAnyNotSupported => {
                    error!("DeserializeAnyNotSupported Error");
                }
                ErrorKind::SizeLimit => {
                    error!("SizeLimit Error");
                }
                ErrorKind::SequenceMustHaveLength => {
                    error!("SequenceMustHaveLength Error");
                }
                ErrorKind::Custom(error) => {
                    error!("Custom error [{}]", error);
                }
            }

            Err(CustomCpuConfigurationError::InvalidFileFormat(
                file_name.parse().unwrap(),
            ))
        }
    }
}
