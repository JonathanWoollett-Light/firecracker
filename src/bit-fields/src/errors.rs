// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// Error type for `impl<T:std::fmt::Display> std::convert::TryFrom<std::collections::HashSet<T>>
/// for YourBitField`.
// There will only ever be 1 possible error as a result of `TryFrom<HashSet<..>>`, thus we will
// never alter the layout of this struct.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, thiserror::Error)]
#[error("Feature flag given in set which is not defined in bit field.")]
pub struct TryFromFlagSetError;

/// Error type for `impl<T:std::fmt::Display>
/// std::convert::TryFrom<std::collections::HashMap<T,YourBitField>> for YourBitField`.
#[derive(Debug, thiserror::Error)]
pub enum TryFromFieldMapError {
    /// Field given in map which is not defined in bit field.
    #[error("Field given in map which is not defined in bit field.")]
    UnknownField,
    /// Failed to assign value from field map.
    #[error("Failed to assign value from field map: {0}")]
    CheckedAssign(#[from] CheckedAssignError),
}
/// Error type for `impl<T:std::fmt::Display>
/// std::convert::TryFrom<(std::collections::HashSet<T>,std::collections::HashMap<T,YourBitField>)>
/// for YourBitField`.
#[derive(Debug, thiserror::Error)]
pub enum TryFromFlagSetAndFieldMapError {
    /// Failed to parse flag set.
    #[error("Failed to parse flag set: {0}")]
    FlagSet(#[from] TryFromFlagSetError),
    /// Failed to parse field map.
    #[error("Failed to parse field map: {0}")]
    FieldMap(#[from] TryFromFieldMapError),
}

impl From<CheckedAssignError> for TryFromFlagSetAndFieldMapError {
    #[inline]
    fn from(err: CheckedAssignError) -> Self {
        Self::FieldMap(TryFromFieldMapError::CheckedAssign(err))
    }
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
/// _, _>::checked_sub_assign()`], etc.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CheckedSubAssignError {
    /// Operation would result in underflow of bit range.
    #[error("Operation would result in underflow of bit range.")]
    Underflow,
    /// Given value is more than maximum value storable in bit range.
    #[error("Given value is more than maximum value storable in bit range.")]
    OutOfRange,
}

/// Error type for [`crate::BitRangeMut<u8, _, _>::checked_assign()`], [`crate::BitRangeMut<u16, _,
/// _>::checked_assign()`], etc.
// There will only ever be 1 possible error as a result of `CheckedAssign`, thus we will never alter
// the layout of this struct.
#[allow(clippy::exhaustive_structs)]
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Given value is greater than maximum storable value in bit range.")]
pub struct CheckedAssignError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_flag_set_error_debug() {
        assert_eq!(format!("{:?}", TryFromFlagSetError), "TryFromFlagSetError");
    }

    #[test]
    fn try_from_flag_set_error_display() {
        assert_eq!(
            TryFromFlagSetError.to_string(),
            "Feature flag given in set which is not defined in bit field."
        );
    }

    #[test]
    fn try_from_field_map_error_debug() {
        assert_eq!(
            format!("{:?}", TryFromFieldMapError::UnknownField),
            "UnknownField"
        );
        assert_eq!(
            format!(
                "{:?}",
                TryFromFieldMapError::CheckedAssign(CheckedAssignError)
            ),
            "CheckedAssign(CheckedAssignError)"
        );
    }

    #[test]
    fn try_from_field_map_error_display() {
        assert_eq!(
            TryFromFieldMapError::UnknownField.to_string(),
            "Field given in map which is not defined in bit field."
        );
        assert_eq!(
            TryFromFieldMapError::CheckedAssign(CheckedAssignError).to_string(),
            "Failed to assign value from field map: Given value is greater than maximum storable \
             value in bit range."
        );
    }

    #[test]
    fn try_from_flag_set_and_field_map_error_debug() {
        assert_eq!(
            format!(
                "{:?}",
                TryFromFlagSetAndFieldMapError::FlagSet(TryFromFlagSetError)
            ),
            "FlagSet(TryFromFlagSetError)"
        );
        assert_eq!(
            format!(
                "{:?}",
                TryFromFlagSetAndFieldMapError::FieldMap(TryFromFieldMapError::UnknownField)
            ),
            "FieldMap(UnknownField)"
        );
        assert_eq!(
            format!(
                "{:?}",
                TryFromFlagSetAndFieldMapError::FieldMap(TryFromFieldMapError::CheckedAssign(
                    CheckedAssignError
                ))
            ),
            "FieldMap(CheckedAssign(CheckedAssignError))"
        );
    }

    #[test]
    fn try_from_flag_set_and_field_map_error_display() {
        assert_eq!(
            TryFromFlagSetAndFieldMapError::FlagSet(TryFromFlagSetError).to_string(),
            "Failed to parse flag set: Feature flag given in set which is not defined in bit \
             field."
        );
        assert_eq!(
            TryFromFlagSetAndFieldMapError::FieldMap(TryFromFieldMapError::UnknownField)
                .to_string(),
            "Failed to parse field map: Field given in map which is not defined in bit field."
        );
        assert_eq!(
            TryFromFlagSetAndFieldMapError::FieldMap(TryFromFieldMapError::CheckedAssign(
                CheckedAssignError
            ))
            .to_string(),
            "Failed to parse field map: Failed to assign value from field map: Given value is \
             greater than maximum storable value in bit range."
        );
    }

    #[test]
    fn checked_add_assign_error_debug() {
        assert_eq!(format!("{:?}", CheckedAddAssignError::Overflow), "Overflow");
        assert_eq!(
            format!("{:?}", CheckedAddAssignError::OutOfRange),
            "OutOfRange"
        );
    }
    #[test]
    fn checked_add_assign_error_display() {
        assert_eq!(
            CheckedAddAssignError::Overflow.to_string(),
            "Operation would result in overflow of bit range."
        );
        assert_eq!(
            CheckedAddAssignError::OutOfRange.to_string(),
            "Given value is more than maximum value storable in bit range."
        );
    }

    #[test]
    fn checked_sub_assign_error_debug() {
        assert_eq!(
            format!("{:?}", CheckedSubAssignError::Underflow),
            "Underflow"
        );
        assert_eq!(
            format!("{:?}", CheckedSubAssignError::OutOfRange),
            "OutOfRange"
        );
    }
    #[test]
    fn checked_sub_assign_error_display() {
        assert_eq!(
            CheckedSubAssignError::Underflow.to_string(),
            "Operation would result in underflow of bit range."
        );
        assert_eq!(
            CheckedSubAssignError::OutOfRange.to_string(),
            "Given value is more than maximum value storable in bit range."
        );
    }
}
