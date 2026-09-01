// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Redaction tests for metadata diagnostics.

#[cfg(feature = "filter")]
use qubit_metadata::Condition;
#[cfg(feature = "filter")]
use qubit_metadata::FilterExpression;
#[cfg(feature = "filter")]
use qubit_metadata::FilterExpressionView;
use qubit_metadata::Metadata;
use qubit_redact::MaskPolicy;
use qubit_redact::Redact;
use qubit_redact::RedactionPolicy;
use qubit_redact::RedactionTextOutput;
use qubit_redact::RedactionWriter;
use qubit_redact::Redactor;
use qubit_redact::Sensitivity;
#[cfg(feature = "filter")]
use qubit_value::Value;

fn redacted_output<T: Redact>(value: &T, policy: &RedactionPolicy) -> RedactionTextOutput {
    Redactor::new(policy.clone()).redact(value)
}

fn complete_redacted_text<T: Redact>(value: &T, policy: &RedactionPolicy) -> String {
    redacted_output(value, policy)
        .into_complete_text()
        .expect("the redaction output should be complete")
        .into_string()
}

/// Builds a policy with explicit domain-structure limits.
fn policy_with_domain_limits(max_nodes: usize, max_collection_items: usize) -> RedactionPolicy {
    RedactionPolicy::builder()
        .limits(|limits| {
            limits
                .max_nodes(max_nodes)
                .max_collection_items(max_collection_items)
                .max_depth(32);
        })
        .expect("the test domain limits should build a policy")
        .build()
        .expect("the test domain limits should build a policy")
}

#[test]
fn test_metadata_debug_and_redacted_output_hide_sensitive_string_values() {
    let metadata = Metadata::new()
        .with("password", "correct-horse-battery-staple")
        .with("name", "Ada");
    let debug = format!("{metadata:?}");
    let explicit = Redactor::strict()
        .redact(&metadata)
        .into_complete_text()
        .expect("strict metadata redaction should be complete")
        .into_string();

    assert!(!debug.contains("correct-horse-battery-staple"));
    assert!(!explicit.contains("correct-horse-battery-staple"));
    assert!(!debug.contains("Ada"));
}

#[test]
fn test_metadata_uses_structured_writer_signature() {
    let _: for<'session> fn(&Metadata, &'session mut RedactionWriter<'session>) = <Metadata as Redact>::write_redacted;
}

#[test]
fn test_metadata_stops_before_unadmitted_collection_entries() {
    let metadata = Metadata::new()
        .with("a_visible", "visible")
        .with("z_blocked", "must-not-be-formatted");
    let policy = policy_with_domain_limits(64, 1);

    let output = redacted_output(&metadata, &policy);
    let text = output.text().as_str();

    assert_eq!(
        output.text_or_marker("<redaction incomplete>"),
        "<redaction incomplete>"
    );
    assert!(!text.contains("\"visible\""), "{text}");
    assert!(!text.contains("must-not-be-formatted"), "{text}");
    assert!(text.contains("<truncated>"), "{text}");
}

#[test]
fn test_metadata_exact_collection_limit_is_complete() {
    let metadata = Metadata::new().with("field", "visible");
    let policy = policy_with_domain_limits(64, 1);

    let output = complete_redacted_text(&metadata, &policy);

    assert!(!output.contains("visible"), "{output}");
    assert!(!output.contains("<truncated>"), "{output}");
}

#[test]
fn test_metadata_exact_structural_node_budget_is_complete() {
    let metadata = Metadata::new()
        .with("first_secret", "first-value")
        .with("second_secret", "second-value");
    let policy = RedactionPolicy::builder()
        .fields(|fields| {
            fields
                .raise("first_secret", Sensitivity::Secret)
                .raise("second_secret", Sensitivity::Secret);
        })
        .expect("the test field rule should be valid")
        .limits(|limits| {
            limits.max_nodes(3).max_collection_items(2).max_depth(32);
        })
        .expect("the test limits should be valid")
        .build()
        .expect("policy should build");

    let output = complete_redacted_text(&metadata, &policy);

    assert!(!output.contains("first-value"), "{output}");
    assert!(!output.contains("second-value"), "{output}");
    assert!(!output.contains("<truncated>"), "{output}");
}

#[test]
fn test_metadata_one_less_structural_node_truncates() {
    let metadata = Metadata::new().with("secret", "secret-value");
    let policy = RedactionPolicy::builder()
        .fields(|fields| {
            fields.raise("secret", Sensitivity::Secret);
        })
        .expect("the test field rule should be valid")
        .limits(|limits| {
            limits.max_nodes(1).max_collection_items(1).max_depth(32);
        })
        .expect("the test limits should be valid")
        .build()
        .expect("policy should build");

    let output = redacted_output(&metadata, &policy);
    let text = output.text().as_str();

    assert_eq!(
        output.text_or_marker("<redaction incomplete>"),
        "<redaction incomplete>"
    );
    assert!(!text.contains("secret-value"), "{text}");
    assert!(text.contains("<truncated>"), "{text}");
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
    let policy = policy_with_domain_limits(2, 8);

    let output = redacted_output(condition, &policy);
    let text = output.text().as_str();

    assert_eq!(
        output.text_or_marker("<redaction incomplete>"),
        "<redaction incomplete>"
    );
    assert!(text.contains("operator"), "{text}");
    assert!(!text.contains("must-not-be-formatted"), "{text}");
    assert!(text.contains("<truncated>"), "{text}");
}

#[test]
fn test_metadata_redaction_masks_sensitive_non_strings_as_opaque_values() {
    let metadata = Metadata::new().with("secret_number", 12345_i32);
    let policy = RedactionPolicy::builder()
        .fields(|fields| {
            fields
                .disable_floor()
                .raise("secret_number", Sensitivity::Low)
                .mask(Sensitivity::Low, MaskPolicy::preserve_edges(1, 1, "OPAQUE", 0));
        })
        .expect("the test policy should be valid")
        .build()
        .expect("policy should build");

    let output = complete_redacted_text(&metadata, &policy);

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
        .fields(|fields| {
            fields
                .raise("api_key", Sensitivity::Secret)
                .raise("token", Sensitivity::Secret);
        })
        .expect("the test policy should be valid")
        .build()
        .expect("policy should build");

    let debug = complete_redacted_text(&metadata, &policy);
    let display = debug.clone();

    assert!(!debug.contains("nested-secret"));
    assert!(!debug.contains("outer-secret"));
    assert!(!debug.contains("Ada"));
    assert!(!display.contains("nested-secret"));
    assert!(!display.contains("outer-secret"));
    assert!(!display.contains('\n'));
}
