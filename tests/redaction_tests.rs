// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Redaction tests for metadata diagnostics.

#[cfg(feature = "schema")]
use qubit_datatype::DataType;
#[cfg(feature = "filter")]
use qubit_metadata::FilterExpression;
#[cfg(feature = "filter")]
use qubit_metadata::FilterExpressionView;
use qubit_metadata::Metadata;
#[cfg(feature = "filter")]
use qubit_metadata::MetadataFilter;
#[cfg(feature = "schema")]
use qubit_metadata::MetadataSchema;
use qubit_redact::MaskPolicy;
use qubit_redact::Redact;
use qubit_redact::RedactionPolicy;
use qubit_redact::RedactionTextOutput;
use qubit_redact::RedactionWriter;
use qubit_redact::Redactor;
use qubit_redact::Sensitivity;

fn redacted_output<T: Redact>(value: &T, policy: &RedactionPolicy) -> RedactionTextOutput {
    Redactor::new(policy.clone()).redact_text(value)
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

/// Builds a policy with exact structural and output ceilings.
#[cfg(any(feature = "filter", feature = "schema"))]
fn policy_with_exact_limits(
    max_nodes: usize,
    max_collection_items: usize,
    max_depth: usize,
    max_output_bytes: usize,
) -> RedactionPolicy {
    RedactionPolicy::default()
        .to_builder()
        .limits(|limits| {
            limits
                .max_nodes(max_nodes)
                .max_collection_items(max_collection_items)
                .max_depth(max_depth)
                .max_output_bytes(max_output_bytes);
        })
        .expect("the test limits should build")
        .build()
        .expect("the test policy should build")
}

/// Verifies that each diagnostic resource ceiling is shared by nested values.
#[cfg(any(feature = "filter", feature = "schema"))]
fn assert_exact_diagnostic_budget_boundaries<T: Redact>(value: &T) {
    let generous = policy_with_exact_limits(512, 512, 64, 65_536);
    let complete = redacted_output(value, &generous);
    let usage = complete.summary().usage();
    let output_bytes = usage.output_bytes();
    let nodes = usage.visited_nodes();
    let collection_items = usage.visited_collection_items();
    let depth = usage.max_depth();
    assert!(nodes > 1);
    assert!(collection_items > 1);
    assert!(depth > 1);

    let exact = redacted_output(
        value,
        &policy_with_exact_limits(nodes, collection_items, depth, output_bytes),
    );
    assert_eq!(
        exact.summary().completion(),
        qubit_redact::RedactionCompletion::Complete
    );
    for limits in [
        policy_with_exact_limits(nodes - 1, collection_items, depth, output_bytes),
        policy_with_exact_limits(nodes, collection_items - 1, depth, output_bytes),
        policy_with_exact_limits(nodes, collection_items, depth - 1, output_bytes),
        policy_with_exact_limits(nodes, collection_items, depth, output_bytes - 1),
    ] {
        let rejected = redacted_output(value, &limits);
        assert_ne!(
            rejected.summary().completion(),
            qubit_redact::RedactionCompletion::Complete
        );
        assert_eq!(
            rejected.text_or_marker("<redaction incomplete>"),
            "<redaction incomplete>"
        );
    }
}

#[test]
fn test_metadata_debug_and_redacted_output_hide_sensitive_string_values() {
    let metadata = Metadata::new()
        .with("password", "correct-horse-battery-staple")
        .with("name", "Ada");
    let debug = format!("{metadata:?}");
    let explicit = Redactor::strict()
        .redact_text(&metadata)
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

#[cfg(feature = "filter")]
#[test]
fn test_all_condition_variants_have_redacted_diagnostics() {
    let expressions = [
        FilterExpression::builder().eq("k", 1_i64).build(),
        FilterExpression::builder().ne("k", 1_i64).build(),
        FilterExpression::builder().lt("k", 1_i64).build(),
        FilterExpression::builder().le("k", 1_i64).build(),
        FilterExpression::builder().gt("k", 1_i64).build(),
        FilterExpression::builder().ge("k", 1_i64).build(),
        FilterExpression::builder().in_set("k", [1_i64, 2]).build(),
        FilterExpression::builder().not_in_set("k", [1_i64, 2]).build(),
        FilterExpression::builder().exists("k").build(),
        FilterExpression::builder().not_exists("k").build(),
    ];
    for expression in expressions {
        let expression = expression.expect("condition should build");
        let FilterExpressionView::Condition(condition) = expression.view() else {
            panic!("builder should produce a condition");
        };
        let text = complete_redacted_text(condition, &RedactionPolicy::default());
        assert!(text.contains("Condition"), "{text}");
    }
}

#[cfg(feature = "filter")]
#[test]
fn test_composite_filter_types_implement_redact_and_share_budgets() {
    let expression = FilterExpression::builder()
        .and_group(|group| group.eq("account", "operand-secret").exists("active"))
        .build()
        .expect("expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression.clone())
        .build()
        .expect("filter should build");
    let _: &dyn Redact = &expression;
    let _: &dyn Redact = &filter;

    let adequate = policy_with_domain_limits(64, 64);
    let expression_text = complete_redacted_text(&expression, &adequate);
    assert!(expression_text.contains("And"), "{expression_text}");
    assert!(!expression_text.contains("operand-secret"), "{expression_text}");

    let restrictive = policy_with_domain_limits(2, 64);
    let output = redacted_output(&expression, &restrictive);
    assert_eq!(
        output.text_or_marker("<redaction incomplete>"),
        "<redaction incomplete>"
    );
    let filter_output = redacted_output(&filter, &restrictive);
    assert_eq!(
        filter_output.text_or_marker("<redaction incomplete>"),
        "<redaction incomplete>"
    );

    let collection_limited = redacted_output(&expression, &policy_with_domain_limits(64, 1));
    assert_eq!(
        collection_limited.text_or_marker("<redaction incomplete>"),
        "<redaction incomplete>"
    );
    assert!(!collection_limited.text().as_str().contains("active"));

    let depth_limited = redacted_output(&expression, &policy_with_exact_limits(64, 64, 1, 16_384));
    assert_eq!(
        depth_limited.text_or_marker("<redaction incomplete>"),
        "<redaction incomplete>"
    );

    let debug = format!("{filter:?}");
    assert!(!debug.contains("operand-secret"), "{debug}");
    assert_exact_diagnostic_budget_boundaries(&filter);
}

#[cfg(feature = "filter")]
#[test]
fn test_filter_expression_diagnostics_cover_every_operator_branch() {
    let expressions = [
        FilterExpression::match_all(),
        FilterExpression::match_none(),
        FilterExpression::builder()
            .exists("root")
            .and_group(|group| group.exists("a").exists("b"))
            .build()
            .expect("and expression should build"),
        FilterExpression::builder()
            .exists("root")
            .or_group(|group| group.exists("a").exists("b"))
            .build()
            .expect("or expression should build"),
        FilterExpression::builder()
            .exists("a")
            .not()
            .build()
            .expect("not expression should build"),
        FilterExpression::builder()
            .exists("a")
            .build()
            .expect("condition expression should build"),
    ];
    let outputs = expressions
        .iter()
        .map(|expression| complete_redacted_text(expression, &policy_with_domain_limits(128, 128)))
        .collect::<Vec<_>>();

    for (output, expected) in outputs.iter().zip(["True", "False", "And", "Or", "Not", "Condition"]) {
        assert!(output.contains(expected), "expected {expected} in {output}");
    }
}

#[cfg(feature = "filter")]
#[test]
fn test_filter_expression_exact_budget_boundaries_are_shared_across_children() {
    let expression = FilterExpression::builder()
        .exists("root")
        .and_group(|group| group.exists("first").exists("second"))
        .build()
        .expect("expression should build");
    let generous = policy_with_exact_limits(128, 128, 32, 16_384);
    let complete = redacted_output(&expression, &generous);
    let usage = complete.summary().usage();
    let output_bytes = usage.output_bytes();
    let nodes = usage.visited_nodes();
    let collection_items = usage.visited_collection_items();
    let depth = usage.max_depth();
    assert!(nodes > 1);
    assert!(collection_items > 1);
    assert!(depth > 1);

    let exact = redacted_output(
        &expression,
        &policy_with_exact_limits(nodes, collection_items, depth, output_bytes),
    );
    assert_eq!(
        exact.summary().completion(),
        qubit_redact::RedactionCompletion::Complete
    );
    for limits in [
        policy_with_exact_limits(nodes - 1, collection_items, depth, output_bytes),
        policy_with_exact_limits(nodes, collection_items - 1, depth, output_bytes),
        policy_with_exact_limits(nodes, collection_items, depth - 1, output_bytes),
        policy_with_exact_limits(nodes, collection_items, depth, output_bytes - 1),
    ] {
        let rejected = redacted_output(&expression, &limits);
        assert_ne!(
            rejected.summary().completion(),
            qubit_redact::RedactionCompletion::Complete
        );
        assert_eq!(
            rejected.text_or_marker("<redaction incomplete>"),
            "<redaction incomplete>"
        );
    }
}

#[cfg(feature = "schema")]
#[test]
fn test_metadata_schema_uses_strict_shared_diagnostic_budget() {
    let schema = (0..256)
        .fold(MetadataSchema::builder(), |builder, index| {
            builder.optional(&format!("field-{index}"), DataType::String)
        })
        .build()
        .expect("schema should build");
    let _: &dyn Redact = &schema;

    assert_eq!(format!("{schema:?}"), "<redaction incomplete>");
    let long_key = "k".repeat(100_000);
    let long_key_schema = MetadataSchema::builder()
        .optional(&long_key, DataType::String)
        .build()
        .expect("unbounded in-memory schema keys should build");
    assert_eq!(format!("{long_key_schema:?}"), "<redaction incomplete>");

    let small = MetadataSchema::builder()
        .required("account", DataType::String)
        .optional("z_blocked", DataType::String)
        .build()
        .expect("schema should build");
    let explicit = complete_redacted_text(&small, &policy_with_domain_limits(32, 8));
    assert!(explicit.contains("MetadataSchema"), "{explicit}");
    assert!(explicit.contains("String"), "{explicit}");
    assert_exact_diagnostic_budget_boundaries(&small);

    let limited = redacted_output(&small, &policy_with_domain_limits(64, 1));
    assert_eq!(
        limited.text_or_marker("<redaction incomplete>"),
        "<redaction incomplete>"
    );
    assert!(limited.text().as_str().contains("account"));
    assert!(!limited.text().as_str().contains("z_blocked"));
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

    let explicit = complete_redacted_text(&metadata, &policy);
    let debug = format!("{metadata:?}");
    let display = format!("{metadata}");

    for (label, output) in [
        ("explicit", explicit.as_str()),
        ("Debug", debug.as_str()),
        ("Display", display.as_str()),
    ] {
        for secret in ["nested-secret", "outer-secret", "Ada"] {
            assert!(!output.contains(secret), "{label} exposed {secret}: {output}");
        }
    }
    assert!(!display.contains('\n'), "Display must be single-line: {display}");
}
