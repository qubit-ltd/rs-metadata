// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Redaction tests for metadata diagnostics.

use std::fmt;

use qubit_metadata::Metadata;
use qubit_redact::InputOutputLimit;
use qubit_redact::MaskPolicy;
use qubit_redact::Redact;
use qubit_redact::RedactionPolicy;
use qubit_redact::RedactionSession;
use qubit_redact::Sensitivity;

#[test]
fn test_metadata_debug_and_redacted_output_hide_sensitive_string_values() {
    let metadata = Metadata::new()
        .with("password", "correct-horse-battery-staple")
        .with("name", "Ada");
    let debug = format!("{metadata:?}");
    let explicit = format!("{:?}", metadata.redacted());

    assert!(!debug.contains("correct-horse-battery-staple"));
    assert!(!explicit.contains("correct-horse-battery-staple"));
    assert!(debug.contains("Ada"));
}

#[test]
fn test_metadata_uses_mutable_redaction_session_signature() {
    let _: for<'session, 'policy, 'formatter> fn(
        &Metadata,
        &'session mut RedactionSession<'policy>,
        &'formatter mut fmt::Formatter<'formatter>,
    ) -> fmt::Result = <Metadata as Redact>::fmt_redacted;
}

#[test]
fn test_metadata_reports_finite_redaction_input_bytes() {
    let metadata = Metadata::new().with("field", "value");
    assert_ne!(Redact::redaction_input_bytes(&metadata), usize::MAX,);
}

#[test]
fn test_metadata_redaction_masks_sensitive_non_strings_as_opaque_values() {
    let metadata = Metadata::new().with("secret_number", 12345_i32);
    let policy = RedactionPolicy::builder()
        .disable_floor()
        .raise("secret_number", Sensitivity::Low)
        .expect("field classification should be valid")
        .mask(
            Sensitivity::Low,
            MaskPolicy::preserve_edges(1, 1, "OPAQUE", 0),
        )
        .expect("mask policy should be valid")
        .build()
        .expect("policy should build");

    let output = format!("{:?}", metadata.redacted_with(&policy));

    assert!(!output.contains("12345"), "{output}");
    assert!(output.contains("OPAQUE"), "{output}");
}

#[cfg(feature = "json")]
#[test]
fn test_metadata_redaction_recursively_hides_nested_value_secrets() {
    let metadata = Metadata::new()
        .with(
            "profile",
            serde_json::json!({"api_key": "nested-secret", "label": "Ada"}),
        )
        .with("token", "outer-secret");
    let policy = RedactionPolicy::default()
        .to_builder()
        .raise("api_key", Sensitivity::Secret)
        .expect("field classification should be valid")
        .raise("token", Sensitivity::Secret)
        .expect("field classification should be valid")
        .build()
        .expect("policy should build");

    let debug = format!("{:#?}", metadata.redacted_with(&policy));
    let display = format!("{}", metadata.redacted_with(&policy));

    assert!(!debug.contains("nested-secret"));
    assert!(!debug.contains("outer-secret"));
    assert!(debug.contains("Ada"));
    assert!(!display.contains("nested-secret"));
    assert!(!display.contains("outer-secret"));
    assert!(!display.contains('\n'));
}

#[test]
fn test_metadata_display_respects_the_default_diagnostic_output_limit() {
    let metadata = Metadata::new().with("description", "x".repeat(96 * 1024));

    let output = metadata.to_string();

    assert!(output.len() <= InputOutputLimit::default().max_output_bytes());
    assert!(output.ends_with("<truncated>"));
}

#[test]
fn test_metadata_debug_respects_the_default_diagnostic_output_limit() {
    let metadata = Metadata::new().with("description", "x".repeat(96 * 1024));

    let output = format!("{metadata:?}");

    assert!(output.len() <= InputOutputLimit::default().max_output_bytes());
    assert!(output.ends_with("<truncated>"));
}
