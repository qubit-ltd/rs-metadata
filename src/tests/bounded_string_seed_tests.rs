// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Event-level tests for the bounded string deserialization seed.

use serde::de::DeserializeSeed;
use serde::de::value::BorrowedStrDeserializer;
use serde::de::value::Error as ValueError;
use serde::de::value::StrDeserializer;
use serde::de::value::StringDeserializer;

use crate::internal::BoundedStringSeed;

#[test]
fn test_borrowed_string_is_validated_before_rejection_and_copied_after_acceptance() {
    let value = String::from("oversized");
    let borrowed = value.as_str();
    let original = borrowed.as_ptr();
    let seed = BoundedStringSeed::new(move |candidate: &str| {
        assert_eq!(candidate.as_ptr(), original);
        Err("rejected".to_owned())
    });
    assert!(
        seed.deserialize(BorrowedStrDeserializer::<ValueError>::new(borrowed))
            .is_err()
    );

    let seed = BoundedStringSeed::<_, String>::new(|_: &str| Ok(()));
    let decoded = seed
        .deserialize(BorrowedStrDeserializer::<ValueError>::new("accepted"))
        .expect("accepted borrowed string should become owned");
    assert_eq!(decoded, "accepted");
}

#[test]
fn test_transient_string_is_validated_before_rejection_and_copied_after_acceptance() {
    let seed = BoundedStringSeed::new(|candidate: &str| {
        assert_eq!(candidate, "temporary");
        Err("rejected".to_owned())
    });
    assert!(
        seed.deserialize(StrDeserializer::<ValueError>::new("temporary"))
            .is_err()
    );

    let seed = BoundedStringSeed::<_, String>::new(|_: &str| Ok(()));
    let decoded = seed
        .deserialize(StrDeserializer::<ValueError>::new("accepted"))
        .expect("accepted transient string should become owned");
    assert_eq!(decoded, "accepted");
}

#[test]
fn test_owned_string_is_validated_before_rejection_and_reused_after_acceptance() {
    let rejected = String::from("owned");
    let rejected_ptr = rejected.as_ptr();
    let seed = BoundedStringSeed::new(move |candidate: &str| {
        assert_eq!(candidate.as_ptr(), rejected_ptr);
        Err("rejected".to_owned())
    });
    assert!(
        seed.deserialize(StringDeserializer::<ValueError>::new(rejected))
            .is_err()
    );

    let accepted = String::from("owned");
    let accepted_ptr = accepted.as_ptr();
    let seed = BoundedStringSeed::<_, String>::new(|_: &str| Ok(()));
    let decoded = seed
        .deserialize(StringDeserializer::<ValueError>::new(accepted))
        .expect("accepted owned string should be returned");
    assert_eq!(decoded.as_ptr(), accepted_ptr);
}

#[test]
fn test_non_string_values_are_rejected() {
    let seed = BoundedStringSeed::<_, String>::new(|_: &str| Ok(()));
    assert!(
        seed.deserialize(serde::de::value::U8Deserializer::<ValueError>::new(1))
            .is_err()
    );
}
