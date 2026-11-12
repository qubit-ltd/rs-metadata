// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Generated semantic checks against an independent three-valued expression
//! model.

use qubit_datatype::DataType;
use qubit_datatype::NumericComparisonPolicy;
use qubit_metadata::FilterExpression;
use qubit_metadata::FilterMatchOptions;
use qubit_metadata::Metadata;
use qubit_metadata::MetadataFilter;
use qubit_value::Value;

#[derive(Debug, Clone)]
enum RawExpr {
    Constant(bool),
    Leaf(usize),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Not(Box<Self>),
}

fn and(left: Option<bool>, right: Option<bool>) -> Option<bool> {
    match (left, right) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}

fn or(left: Option<bool>, right: Option<bool>) -> Option<bool> {
    match (left, right) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    }
}

fn eval_raw(expr: &RawExpr, leaves: &[Option<bool>]) -> Option<bool> {
    match expr {
        RawExpr::Constant(value) => Some(*value),
        RawExpr::Leaf(index) => leaves[*index],
        RawExpr::And(left, right) => and(eval_raw(left, leaves), eval_raw(right, leaves)),
        RawExpr::Or(left, right) => or(eval_raw(left, leaves), eval_raw(right, leaves)),
        RawExpr::Not(inner) => eval_raw(inner, leaves).map(|value| !value),
    }
}

fn compile_raw(expr: &RawExpr, leaves: &[FilterExpression], context: &str) -> FilterExpression {
    match expr {
        RawExpr::Constant(true) => FilterExpression::match_all(),
        RawExpr::Constant(false) => FilterExpression::match_none(),
        RawExpr::Leaf(index) => leaves[*index].clone(),
        RawExpr::And(left, right) => compile_raw(left, leaves, context)
            .try_and(compile_raw(right, leaves, context))
            .unwrap_or_else(|error| {
                panic!("{context}: generated AND construction failed: {error}")
            }),
        RawExpr::Or(left, right) => compile_raw(left, leaves, context)
            .try_or(compile_raw(right, leaves, context))
            .unwrap_or_else(|error| panic!("{context}: generated OR construction failed: {error}")),
        RawExpr::Not(inner) => compile_raw(inner, leaves, context)
            .try_not()
            .unwrap_or_else(|error| {
                panic!("{context}: generated NOT construction failed: {error}")
            }),
    }
}

struct LeafCase {
    id: &'static str,
    expression: FilterExpression,
    exact: Option<bool>,
    approximate: Option<bool>,
}

fn leaf_cases() -> (Metadata, Vec<LeafCase>) {
    let mut metadata = Metadata::new();
    metadata.set("bool_true", true);
    metadata.set("bool_false", false);
    metadata.set("int", 18_i64);
    metadata.set("missing_comparison_guard", 1_i64);
    metadata.set("unset", Value::Unset(DataType::Int64));
    metadata.set("wrong_type", "eighteen");
    metadata.set("nan", f64::NAN);
    metadata.set("mixed_equal", 18_i64);
    metadata.set("mixed_range", 18_i64);
    metadata.set("member_incomparable", "rust");
    metadata.set("member_unknown", "rust");
    metadata.set("member_match", "rust");
    metadata.set("empty", "rust");
    metadata.set("empty_missing_guard", true);

    let make = |id, expression, expected| LeafCase {
        id,
        expression,
        exact: expected,
        approximate: expected,
    };
    let mixed_equal = FilterExpression::builder()
        .eq("mixed_equal", 18.0_f64)
        .build()
        .expect("mixed equality leaf");
    let mixed_range = FilterExpression::builder()
        .lt("mixed_range", 19.0_f64)
        .build()
        .expect("mixed range leaf");
    let cases = vec![
        make(
            "bool_equal_true",
            FilterExpression::builder()
                .eq("bool_true", true)
                .build()
                .expect("bool leaf"),
            Some(true),
        ),
        make(
            "bool_equal_false",
            FilterExpression::builder()
                .eq("bool_false", true)
                .build()
                .expect("bool leaf"),
            Some(false),
        ),
        make(
            "int_equal",
            FilterExpression::builder()
                .eq("int", 18_i64)
                .build()
                .expect("integer leaf"),
            Some(true),
        ),
        make(
            "int_range",
            FilterExpression::builder()
                .ge("int", 18_i64)
                .build()
                .expect("range leaf"),
            Some(true),
        ),
        make(
            "missing_comparison",
            FilterExpression::builder()
                .eq("missing", 1_i64)
                .build()
                .expect("missing leaf"),
            None,
        ),
        make(
            "unset_comparison",
            FilterExpression::builder()
                .eq("unset", 18_i64)
                .build()
                .expect("unset leaf"),
            None,
        ),
        make(
            "incomparable_type",
            FilterExpression::builder()
                .eq("wrong_type", 18_i64)
                .build()
                .expect("type leaf"),
            None,
        ),
        make(
            "stored_nan",
            FilterExpression::builder()
                .lt("nan", 1.0_f64)
                .build()
                .expect("NaN storage leaf"),
            None,
        ),
        LeafCase {
            id: "mixed_numeric_equal",
            expression: mixed_equal,
            exact: Some(true),
            approximate: Some(true),
        },
        LeafCase {
            id: "mixed_numeric_range",
            expression: mixed_range,
            exact: Some(true),
            approximate: Some(true),
        },
        make(
            "exists_missing",
            FilterExpression::builder()
                .exists("missing")
                .build()
                .expect("exists leaf"),
            Some(false),
        ),
        make(
            "not_exists_missing",
            FilterExpression::builder()
                .not_exists("missing")
                .build()
                .expect("not exists leaf"),
            Some(true),
        ),
        make(
            "exists_unset",
            FilterExpression::builder()
                .exists("unset")
                .build()
                .expect("exists leaf"),
            Some(false),
        ),
        make(
            "not_exists_unset",
            FilterExpression::builder()
                .not_exists("unset")
                .build()
                .expect("not exists leaf"),
            Some(true),
        ),
        make(
            "exists_concrete",
            FilterExpression::builder()
                .exists("int")
                .build()
                .expect("exists leaf"),
            Some(true),
        ),
        make(
            "in_match_prefers_match",
            FilterExpression::builder()
                .in_set(
                    "member_incomparable",
                    [Value::from(18_i64), Value::from("rust")],
                )
                .build()
                .expect("membership leaf"),
            Some(true),
        ),
        make(
            "in_unknown_candidate",
            FilterExpression::builder()
                .in_set("member_unknown", [18_i64])
                .build()
                .expect("membership leaf"),
            None,
        ),
        make(
            "not_in_unknown_candidate",
            FilterExpression::builder()
                .not_in_set("member_unknown", [18_i64])
                .build()
                .expect("membership leaf"),
            None,
        ),
        make(
            "empty_in_concrete",
            FilterExpression::builder()
                .in_set::<_, Value>("empty", [])
                .build()
                .expect("empty membership leaf"),
            Some(false),
        ),
        make(
            "empty_not_in_concrete",
            FilterExpression::builder()
                .not_in_set::<_, Value>("empty", [])
                .build()
                .expect("empty membership leaf"),
            Some(true),
        ),
        make(
            "empty_in_missing",
            FilterExpression::builder()
                .in_set::<_, Value>("missing", [])
                .build()
                .expect("empty membership leaf"),
            None,
        ),
        make(
            "empty_not_in_missing",
            FilterExpression::builder()
                .not_in_set::<_, Value>("missing", [])
                .build()
                .expect("empty membership leaf"),
            None,
        ),
    ];
    (metadata, cases)
}

fn policy_filter(expression: FilterExpression, policy: NumericComparisonPolicy) -> MetadataFilter {
    let options = FilterMatchOptions::builder()
        .numeric_comparison_policy(policy)
        .build();
    MetadataFilter::builder()
        .expression(expression)
        .options(options)
        .build()
        .expect("filter should build")
}

fn assert_semantics(
    raw: &RawExpr,
    metadata: &Metadata,
    leaves: &[LeafCase],
    seed: u64,
    iteration: usize,
    label: &str,
) {
    let raw_values: Vec<_> = leaves
        .iter()
        .map(|leaf| match label {
            "Exact" => leaf.exact,
            _ => leaf.approximate,
        })
        .collect();
    let leaf_ids: Vec<_> = leaves.iter().map(|leaf| leaf.id).collect();
    let context = format!(
        "seed={seed}, iteration={iteration}, tree={raw:?}, sample=combined-metadata, leaf_ids={leaf_ids:?}, policy={label}"
    );
    let compiled = compile_raw(
        raw,
        &leaves
            .iter()
            .map(|leaf| leaf.expression.clone())
            .collect::<Vec<_>>(),
        &context,
    );
    let expected = eval_raw(raw, &raw_values) == Some(true);
    assert_eq!(
        policy_filter(compiled.clone(), policy(label)).matches(metadata),
        expected,
        "{context}"
    );
    for (name, expression) in [
        ("double_not", raw_not(raw_not(raw.clone()))),
        (
            "and_true",
            RawExpr::And(Box::new(raw.clone()), Box::new(RawExpr::Constant(true))),
        ),
        (
            "or_false",
            RawExpr::Or(Box::new(raw.clone()), Box::new(RawExpr::Constant(false))),
        ),
    ] {
        let transformed = compile_raw(
            &expression,
            &leaves
                .iter()
                .map(|leaf| leaf.expression.clone())
                .collect::<Vec<_>>(),
            &format!("{context}, transform={name}"),
        );
        let transformed_expected = eval_raw(&expression, &raw_values) == Some(true);
        assert_eq!(
            policy_filter(transformed, policy(label)).matches(metadata),
            transformed_expected,
            "{context}, transform={name}"
        );
    }
}

fn policy(label: &str) -> NumericComparisonPolicy {
    if label == "Exact" {
        NumericComparisonPolicy::Exact
    } else {
        NumericComparisonPolicy::Approximate
    }
}

fn raw_not(expr: RawExpr) -> RawExpr {
    RawExpr::Not(Box::new(expr))
}

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
}

fn generate(rng: &mut Lcg, depth: usize, leaf_count: usize) -> RawExpr {
    let choice = if depth == 1 {
        (rng.next() % 3) as usize
    } else {
        (rng.next() % 6) as usize
    };
    match choice {
        0 => RawExpr::Constant(true),
        1 => RawExpr::Constant(false),
        2 => RawExpr::Leaf((rng.next() as usize) % leaf_count),
        3 => RawExpr::And(
            Box::new(generate(rng, depth - 1, leaf_count)),
            Box::new(generate(rng, depth - 1, leaf_count)),
        ),
        4 => RawExpr::Or(
            Box::new(generate(rng, depth - 1, leaf_count)),
            Box::new(generate(rng, depth - 1, leaf_count)),
        ),
        _ => RawExpr::Not(Box::new(generate(rng, depth - 1, leaf_count))),
    }
}

#[test]
fn test_leaf_matrix_and_negation_match_hand_annotated_expectations() {
    let (metadata, leaves) = leaf_cases();
    for policy in [
        NumericComparisonPolicy::Exact,
        NumericComparisonPolicy::Approximate,
    ] {
        for leaf in &leaves {
            let expected = if policy == NumericComparisonPolicy::Exact {
                leaf.exact
            } else {
                leaf.approximate
            };
            let filter = policy_filter(leaf.expression.clone(), policy);
            let context = format!("leaf={}, policy={policy:?}", leaf.id);
            assert_eq!(
                filter.matches(&metadata),
                expected == Some(true),
                "{context}"
            );
            let negated = leaf
                .expression
                .clone()
                .try_not()
                .unwrap_or_else(|error| panic!("{context}: NOT construction failed: {error}"));
            assert_eq!(
                policy_filter(negated, policy).matches(&metadata),
                expected == Some(false),
                "negated {context}"
            );
        }
    }
}

#[test]
fn test_generated_trees_match_independent_three_valued_oracle() {
    let (metadata, leaves) = leaf_cases();
    let leaf_count = leaves.len();
    for seed in [1_u64, 7, 42, 20261106] {
        let mut rng = Lcg(seed);
        for iteration in 0..256 {
            let raw = generate(&mut rng, 5, leaf_count);
            for label in ["Exact", "Approximate"] {
                assert_semantics(&raw, &metadata, &leaves, seed, iteration, label);
            }
        }
    }
}

#[test]
fn test_fixed_shapes_cover_grouping_alternation_not_and_constants() {
    let (metadata, leaves) = leaf_cases();
    let a = RawExpr::Leaf(0);
    let b = RawExpr::Leaf(1);
    let c = RawExpr::Leaf(2);
    let shapes = [
        RawExpr::And(
            Box::new(RawExpr::And(Box::new(a.clone()), Box::new(b.clone()))),
            Box::new(c.clone()),
        ),
        RawExpr::Or(
            Box::new(RawExpr::Or(Box::new(a.clone()), Box::new(b.clone()))),
            Box::new(c.clone()),
        ),
        RawExpr::And(
            Box::new(a.clone()),
            Box::new(RawExpr::Or(Box::new(b.clone()), Box::new(c.clone()))),
        ),
        RawExpr::Not(Box::new(RawExpr::Not(Box::new(RawExpr::Not(Box::new(
            a.clone(),
        )))))),
        RawExpr::And(
            Box::new(RawExpr::Constant(false)),
            Box::new(RawExpr::Not(Box::new(b.clone()))),
        ),
        RawExpr::Or(
            Box::new(RawExpr::Constant(true)),
            Box::new(RawExpr::Not(Box::new(c.clone()))),
        ),
    ];
    for (index, raw) in shapes.iter().enumerate() {
        for label in ["Exact", "Approximate"] {
            assert_semantics(raw, &metadata, &leaves, 0, index, label);
        }
    }
}
