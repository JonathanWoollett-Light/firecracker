// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// Error type for `impl<T:std::fmt::Display> std::convert::TryFrom<std::collections::HashSet<T>>
/// for YourBitField` with zero-able backing type.
#[derive(Debug, thiserror::Error)]
#[error("Feature flag given in set which is not defined in bit field.")]
pub struct TryFromFlagSetError;

/// Error type for `impl<T:std::fmt::Display> std::convert::TryFrom<std::collections::HashSet<T>>
/// for YourBitField` with non-zero backing type.
#[derive(Debug, thiserror::Error)]
pub enum NonZeroTryFromFlagSetError {
    /// Feature flag given in set which is not defined in bit field.
    #[error("Feature flag given in set which is not defined in bit field.")]
    MissingFlag,
    /// No feature flags given, which would results in zero NonZero type.
    #[error("No feature flags given, which would results in zero NonZero type.")]
    ZeroFlags,
}

/// Error type for `impl<T:std::fmt::Display>
/// std::convert::TryFrom<std::collections::HashMap<T,YourBitField>> for YourBitField` with
/// zero-able backing type.
#[derive(Debug, thiserror::Error)]
pub enum TryFromFieldMapError {
    /// Bit range given in map which is not defined in bit field.
    #[error("Bit range given in map which is not defined in bit field.")]
    UnknownRange,
    /// Failed to assign value from field map.
    #[error("Failed to assign value from field map: {0}")]
    CheckedAssign(#[from] CheckedAssignError),
}

/// Error type for `impl<T:std::fmt::Display>
/// std::convert::TryFrom<std::collections::HashMap<T,YourBitField>> for YourBitField` with non-zero
/// backing type.
#[derive(Debug, thiserror::Error)]
pub enum NonZeroTryFromFieldMapError {
    /// Bit range given in map which is not defined in bit field.
    #[error("Bit range given in map which is not defined in bit field.")]
    UnknownRange,
    /// Failed to assign value from field map.
    #[error("Failed to assign value from field map: {0}")]
    CheckedAssign(#[from] CheckedAssignError),
    /// No non-zero bit ranges given, which would results in zero NonZero type.
    #[error("No non-zero bit ranges given, which would results in zero NonZero type.")]
    ZeroRanges,
}

/// Error type for `impl<T:std::fmt::Display>
/// std::convert::TryFrom<(std::collections::HashSet<T>,std::collections::HashMap<T,YourBitField>)>
/// for YourBitField` with zero-able types.
#[derive(Debug, thiserror::Error)]
pub enum TryFromFlagSetAndFieldMapError {
    /// Failed to parse flag set.
    #[error("Feature flag given in set which is not defined in bit field.")]
    MissingFlag,
    /// Bit range given in map which is not defined in bit field.
    #[error("Bit range given in map which is not defined in bit field.")]
    UnknownRange,
    /// Failed to assign value from field map.
    #[error("Failed to assign value from field map: {0}")]
    CheckedAssign(#[from] CheckedAssignError),
}

/// Error type for `impl<T:std::fmt::Display>
/// std::convert::TryFrom<(std::collections::HashSet<T>,std::collections::HashMap<T,YourBitField>)>
/// for YourBitField` with non-zero types.
#[derive(Debug, thiserror::Error)]
pub enum NonZeroTryFromFlagSetAndFieldMapError {
    /// Failed to parse flag set.
    #[error("Feature flag given in set which is not defined in bit field.")]
    MissingFlag,
    /// Bit range given in map which is not defined in bit field.
    #[error("Bit range given in map which is not defined in bit field.")]
    UnknownRange,
    /// Failed to assign value from field map.
    #[error("Failed to assign value from field map: {0}")]
    CheckedAssign(#[from] CheckedAssignError),
    /// No bit flags or non-zero bit ranges given, which would results in zero NonZero type.
    #[error(
        "No bit flags or non-zero bit ranges given, which would results in zero NonZero type."
    )]
    Zero,
}

/// Error type for [`crate::BitRangeMut<u8, _, _>::checked_add_assign()`], [`crate::BitRangeMut<u16,
/// _, _>::checked_add_assign()`], etc.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CheckedAddAssignError {
    /// Operation would result in overflow of bit range.
    #[error("Operation would result in overflow of bit range.")]
    Overflow,
    /// Given value is more than maximum value storable in bit range.
    #[error("Given value is more than maximum value storable in bit range.")]
    OutOfRange,
}

/// Error type for [`crate::BitRangeMut<u8, _, _>::checked_sub_assign()`], [`crate::BitRangeMut<u16,
/// _, _>::checked_sub_assign()`], etc for zero-able types.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CheckedSubAssignError {
    /// Operation would result in underflow of bit range.
    #[error("Operation would result in underflow of bit range.")]
    Underflow,
    /// Given value is more than maximum value storable in bit range.
    #[error("Given value is more than maximum value storable in bit range.")]
    OutOfRange,
}

/// Error type for [`crate::BitRangeMut<u8, _, _>::checked_sub_assign()`], [`crate::BitRangeMut<u16,
/// _, _>::checked_sub_assign()`], etc for non-zero types.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum NonZeroCheckedSubAssignError {
    /// Operation would result in underflow of bit range.
    #[error("Operation would result in underflow of bit range.")]
    Underflow,
    /// Given value is more than maximum value storable in bit range.
    #[error("Given value is more than maximum value storable in bit range.")]
    OutOfRange,
    /// Operation would result in zero value.
    #[error("Operation would result in zero value.")]
    Zero,
}

/// Error type for [`crate::BitRangeMut<u8, _, _>::checked_assign()`], [`crate::BitRangeMut<u16, _,
/// _>::checked_assign()`], etc for zero-able types.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Given value is greater than maximum storable value in bit range.")]
pub struct CheckedAssignError;

/// Error type for [`crate::BitRangeMut<u8, _, _>::checked_assign()`], [`crate::BitRangeMut<u16, _,
/// _>::checked_assign()`], etc for non-zero types.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum NonZeroCheckedAssignError {
    #[error("Given value is greater than maximum storable value in bit range.")]
    Overflow,
    #[error("Given value is zero.")]
    Zero,
}
