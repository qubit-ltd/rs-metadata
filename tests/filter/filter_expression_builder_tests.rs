// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for [`qubit_metadata::FilterExpressionBuilder`].

use qubit_datatype::DataType;
use qubit_metadata::FilterExpression;
use qubit_metadata::FilterExpressionView;
use qubit_metadata::FilterLimitKind;
use qubit_metadata::FilterLimits;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataError;
use qubit_metadata::MetadataFilter;
use qubit_value::Value;

use crate::support::test_support::sample;

#[derive(Debug, Clone, Copy)]
struct TenantId(u64);

impl From<TenantId> for Value {
    /// Converts a tenant identifier into its filter operand representation.
    fn from(value: TenantId) -> Self {
        Self::UInt64(value.0)
    }
}

#[test]
fn test_eq_accepts_value_and_domain_newtype() {
    let raw = FilterExpression::builder()
        .eq("status", Value::String("active".to_owned()))
        .build()
        .expect("Value operand should build");
    let tenant = FilterExpression::builder()
        .eq("tenant_id", TenantId(42))
        .build()
        .expect("newtype operand should build");

    assert!(
        MetadataFilter::builder()
            .expression(raw)
            .build()
            .expect("raw filter should build")
            .matches(&sample())
    );
    assert!(matches!(tenant.view(), FilterExpressionView::Condition(_)));
}

#[test]
fn test_builder_rejects_unset_and_non_finite_filter_operands() {
    assert_eq!(
        FilterExpression::builder()
            .eq("status", Value::Unset(DataType::String))
            .build(),
        Err(MetadataError::InvalidFilterOperand {
            operator: "eq",
            data_type: DataType::String,
            message: "filter operands must be concrete values".to_owned(),
        })
    );
    assert!(FilterExpression::builder().eq("score", f64::NAN).build().is_err());
    assert!(FilterExpression::builder().eq("score", f64::INFINITY).build().is_err());
}

#[test]
fn test_builder_creates_nested_boolean_expression() {
    let expression = FilterExpression::builder()
        .eq("status", "active")
        .gt("score", 40_i64)
        .or_group(|group| group.eq("tag", "rust").exists("owner"))
        .build()
        .expect("expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");

    assert!(filter.matches(&sample()));
    assert!(matches!(filter.expression().view(), FilterExpressionView::Or(_)));
}

#[test]
fn test_or_group_matches_either_explicit_branch_within_group() {
    let expression = FilterExpression::builder()
        .eq("tenant", "acme")
        .or_group(|group| {
            group
                .eq("priority", 1_i64)
                .or_group(|alternative| alternative.eq("priority", 2_i64))
        })
        .build()
        .expect("expression should build");
    let filter = MetadataFilter::builder()
        .expression(expression)
        .build()
        .expect("filter should build");
    let metadata = Metadata::new().with("tenant", "other").with("priority", 2_i64);

    assert!(filter.matches(&metadata));
}

#[test]
fn test_builder_rejects_empty_groups() {
    let result = FilterExpression::builder().and_group(|group| group).build();

    assert!(result.is_err());
}

#[test]
fn test_expression_owns_boolean_composition() {
    let active = FilterExpression::builder()
        .eq("status", "active")
        .build()
        .expect("expression should build");
    let high_score = FilterExpression::builder()
        .gt("score", 40_i64)
        .build()
        .expect("expression should build");
    let expression = active
        .try_and(high_score)
        .expect("AND composition should fit hard limits")
        .try_not()
        .expect("NOT composition should fit hard limits");

    assert!(matches!(
        FilterExpression::match_all().view(),
        FilterExpressionView::True
    ));
    assert!(matches!(
        FilterExpression::match_none().view(),
        FilterExpressionView::False
    ));
    assert!(matches!(expression.view(), FilterExpressionView::Not(_)));
}

#[test]
fn test_not_propagates_node_limit_error() {
    let mut builder = FilterExpression::builder();
    for index in 0..(FilterLimits::MAX.max_nodes() - 1) {
        builder = builder.exists(&format!("key_{index}"));
    }

    assert_eq!(
        builder.not().build(),
        Err(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::Nodes,
            value: FilterLimits::MAX.max_nodes() + 1,
            maximum: FilterLimits::MAX.max_nodes(),
        })
    );
}

#[test]
fn test_and_group_preserves_nested_error() {
    assert_eq!(
        FilterExpression::builder().and_group(|group| group.not()).build(),
        Err(MetadataError::InvalidFilterExpression {
            message: "cannot negate an empty filter expression".to_owned(),
        })
    );
}

#[test]
fn test_empty_groups_report_operator_specific_errors() {
    assert_eq!(
        FilterExpression::builder().and_group(|group| group).build(),
        Err(MetadataError::InvalidFilterExpression {
            message: "AND group must contain at least one condition".to_owned(),
        })
    );
    assert_eq!(
        FilterExpression::builder().or_group(|group| group).build(),
        Err(MetadataError::InvalidFilterExpression {
            message: "OR group must contain at least one condition".to_owned(),
        })
    );
}

#[test]
fn test_membership_stops_consuming_at_detection_item() {
    let consumed = std::cell::Cell::new(0);
    let values = (0..10_000_i64).inspect(|_| consumed.set(consumed.get() + 1));
    let result = FilterExpression::builder().in_set("id", values).build();
    assert_eq!(consumed.get(), FilterLimits::MAX.max_set_values() + 1);
    assert_eq!(
        result,
        Err(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::SetValues,
            value: 129,
            maximum: 128,
        })
    );
}

#[test]
fn test_failed_builder_does_not_consume_membership_or_run_groups() {
    let consumed = std::cell::Cell::new(0);
    let values = (0..10_i64).inspect(|_| consumed.set(consumed.get() + 1));
    let result = FilterExpression::builder()
        .not()
        .not_in_set("id", values)
        .and_group(|_| panic!("failed builder must skip group callback"))
        .build();
    assert_eq!(consumed.get(), 0);
    assert!(matches!(result, Err(MetadataError::InvalidFilterExpression { .. })));
}

#[test]
fn test_node_limit_stops_further_operand_conversion() {
    let mut builder = FilterExpression::builder();
    for _ in 0..FilterLimits::MAX.max_nodes() {
        builder = builder.exists("id");
    }
    let consumed = std::cell::Cell::new(0);
    let values = (0..10_i64).inspect(|_| consumed.set(consumed.get() + 1));
    let result = builder.in_set("id", values).build();
    assert_eq!(consumed.get(), 0);
    assert!(matches!(
        result,
        Err(MetadataError::FilterLimitExceeded {
            kind: FilterLimitKind::Nodes,
            ..
        })
    ));
}
