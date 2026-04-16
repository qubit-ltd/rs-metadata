/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Tests for [`qubit_metadata::MetadataError`].
//!
//! Deserialization/serialization error construction is also covered through
//! `Metadata::try_get` / `try_set` in `test_metadata.rs`.

use qubit_metadata::{MetadataError, MetadataValueType};

#[test]
fn display_formats_all_variants() {
    assert_eq!(
        MetadataError::MissingKey("k".to_string()).to_string(),
        "Metadata key not found: k"
    );

    let ser = MetadataError::SerializationError {
        key: "a".to_string(),
        message: "oops".to_string(),
    };
    assert_eq!(
        ser.to_string(),
        "Failed to serialize metadata value for key 'a': oops"
    );

    let de = MetadataError::DeserializationError {
        key: "b".to_string(),
        expected: std::any::type_name::<bool>(),
        actual: MetadataValueType::Number,
        message: "bad".to_string(),
    };
    assert_eq!(
        de.to_string(),
        format!(
            "Failed to deserialize metadata key 'b' as {} from JSON number: bad",
            std::any::type_name::<bool>()
        )
    );
}

#[test]
fn error_source_is_none() {
    let e: MetadataError = MetadataError::MissingKey("x".to_string());
    assert!(std::error::Error::source(&e).is_none());
}

#[test]
fn partial_eq_distinct_missing_keys() {
    assert_eq!(
        MetadataError::MissingKey("a".to_string()),
        MetadataError::MissingKey("a".to_string())
    );
    assert_ne!(
        MetadataError::MissingKey("a".to_string()),
        MetadataError::MissingKey("b".to_string())
    );
}
