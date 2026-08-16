// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Redaction tests for metadata diagnostics.

use std::fmt;

#[cfg(feature = "filter")]
use qubit_metadata::FilterExpression;
#[cfg(feature = "filter")]
use qubit_metadata::FilterExpressionView;
use qubit_metadata::Metadata;
use qubit_redact::InputOutputLimit;
use qubit_redact::MaskPolicy;
use qubit_redact::RedactionPolicy;
use qubit_redact::RedactionSession;
use qubit_redact::Sensitivity;
use qubit_redact::domain::Redact;
use qubit_redact::policy::DomainRedactionLimits;

/// Builds a policy with explicit domain-structure limits.
fn policy_with_domain_limits(
    max_nodes: usize,
    max_collection_items: usize,
) -> RedactionPolicy {
    let limits = DomainRedactionLimits::builder()
        .max_nodes(max_nodes)
        .max_collection_items(max_collection_items)
        .max_depth(DomainRedactionLimits::DEFAULT_MAX_DEPTH)
        .build()
        .expect("the test domain limits should be valid");
    let mut builder = RedactionPolicy::builder();
    builder.limits().domain(limits);
    builder
        .build()
        .expect("the test domain limits should build a policy")
}

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
fn test_metadata_stops_before_unadmitted_collection_entries() {
    let metadata = Metadata::new()
        .with("a_visible", "visible")
        .with("z_blocked", "must-not-be-formatted");
    let policy = policy_with_domain_limits(64, 1);

    let output = format!("{:?}", metadata.redacted_with(&policy));

    assert!(output.contains("visible"), "{output}");
    assert!(!output.contains("must-not-be-formatted"), "{output}");
    assert!(output.contains("<truncated>"), "{output}");
}

#[test]
fn test_metadata_exact_collection_limit_is_complete() {
    let metadata = Metadata::new().with("field", "visible");
    let policy = policy_with_domain_limits(64, 1);

    let output = format!("{:?}", metadata.redacted_with(&policy));

    assert!(output.contains("visible"), "{output}");
    assert!(!output.contains("<truncated>"), "{output}");
}

#[test]
fn test_metadata_exact_structural_node_budget_is_complete() {
    let metadata = Metadata::new()
        .with("first_secret", "first-value")
        .with("second_secret", "second-value");
    let limits = DomainRedactionLimits::builder()
        .max_nodes(3)
        .max_collection_items(2)
        .max_depth(DomainRedactionLimits::DEFAULT_MAX_DEPTH)
        .build()
        .expect("the test domain limits should be valid");
    let mut builder = RedactionPolicy::builder();
    builder
        .fields()
        .raise("first_secret", Sensitivity::Secret)
        .expect("the test field rule should be valid")
        .raise("second_secret", Sensitivity::Secret)
        .expect("the test field rule should be valid");
    builder.limits().domain(limits);
    let policy = builder.build().expect("policy should build");

    let output = format!("{:?}", metadata.redacted_with(&policy));

    assert!(!output.contains("first-value"), "{output}");
    assert!(!output.contains("second-value"), "{output}");
    assert!(!output.contains("<truncated>"), "{output}");
}

#[test]
fn test_metadata_one_less_structural_node_truncates() {
    let metadata = Metadata::new().with("secret", "secret-value");
    let limits = DomainRedactionLimits::builder()
        .max_nodes(2)
        .max_collection_items(1)
        .max_depth(DomainRedactionLimits::DEFAULT_MAX_DEPTH)
        .build()
        .expect("the test domain limits should be valid");
    let mut builder = RedactionPolicy::builder();
    builder
        .fields()
        .raise("secret", Sensitivity::Secret)
        .expect("the test field rule should be valid");
    builder.limits().domain(limits);
    let policy = builder.build().expect("policy should build");

    let output = format!("{:?}", metadata.redacted_with(&policy));

    assert!(!output.contains("secret-value"), "{output}");
    assert!(output.contains("<truncated>"), "{output}");
}

#[cfg(feature = "filter")]
#[test]
fn test_condition_stops_before_unadmitted_key_field() {
    let expression = FilterExpression::builder()
        .eq("must-not-be-formatted", 42_i32)
        .build()
        .expect("the test filter expression should build");
    let FilterExpressionView::Condition(condition) = expression.view() else {
        panic!("the builder should produce one condition")
    };
    let policy = policy_with_domain_limits(1, 8);

    let output = format!("{:?}", condition.redacted_with(&policy));

    assert!(output.contains("operator"), "{output}");
    assert!(!output.contains("must-not-be-formatted"), "{output}");
    assert!(output.contains("<truncated>"), "{output}");
}

#[test]
fn test_metadata_redaction_masks_sensitive_non_strings_as_opaque_values() {
    let metadata = Metadata::new().with("secret_number", 12345_i32);
    let mut builder = RedactionPolicy::builder();
    builder
        .fields()
        .disable_floor()
        .raise("secret_number", Sensitivity::Low)
        .expect("field classification should be valid")
        .mask(
            Sensitivity::Low,
            MaskPolicy::preserve_edges(1, 1, "OPAQUE", 0),
        )
        .expect("mask policy should be valid");
    let policy = builder.build().expect("policy should build");

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
    let mut builder = RedactionPolicy::default().to_builder();
    builder
        .fields()
        .raise("api_key", Sensitivity::Secret)
        .expect("field classification should be valid")
        .raise("token", Sensitivity::Secret)
        .expect("field classification should be valid");
    let policy = builder.build().expect("policy should build");

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
