/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Unit tests for [`qubit_metadata::Condition`] (leaf predicates via
//! [`qubit_metadata::MetadataFilter`] constructors).

use qubit_metadata::{
    Condition,
    Metadata,
    MetadataFilter,
};

fn sample() -> Metadata {
    let mut m = Metadata::new();
    m.set("status", "active");
    m.set("score", 42_i64);
    m.set("ratio", 0.75_f64);
    m.set("verified", true);
    m.set("tag", "rust");
    m
}

// ── Equal / NotEqual ─────────────────────────────────────────────────────────

#[test]
fn eq_matches_equal_string() {
    let f = MetadataFilter::equal("status", "active");
    assert!(f.matches(&sample()));
}

#[test]
fn eq_does_not_match_different_string() {
    let f = MetadataFilter::equal("status", "inactive");
    assert!(!f.matches(&sample()));
}

#[test]
fn eq_missing_key_does_not_match() {
    let f = MetadataFilter::equal("missing", "x");
    assert!(!f.matches(&sample()));
}

#[test]
fn ne_matches_different_value() {
    let f = MetadataFilter::not_equal("status", "inactive");
    assert!(f.matches(&sample()));
}

#[test]
fn ne_does_not_match_equal_value() {
    let f = MetadataFilter::not_equal("status", "active");
    assert!(!f.matches(&sample()));
}

#[test]
fn ne_missing_key_matches() {
    let f = MetadataFilter::not_equal("missing", "anything");
    assert!(f.matches(&sample()));
}

// ── Greater / GreaterEqual / Less / LessEqual ────────────────────────────────

#[test]
fn gt_integer() {
    assert!(MetadataFilter::greater("score", 10_i64).matches(&sample()));
    assert!(!MetadataFilter::greater("score", 42_i64).matches(&sample()));
    assert!(!MetadataFilter::greater("score", 100_i64).matches(&sample()));
}

#[test]
fn gte_integer() {
    assert!(MetadataFilter::greater_equal("score", 42_i64).matches(&sample()));
    assert!(MetadataFilter::greater_equal("score", 10_i64).matches(&sample()));
    assert!(!MetadataFilter::greater_equal("score", 43_i64).matches(&sample()));
}

#[test]
fn lt_integer() {
    assert!(MetadataFilter::less("score", 100_i64).matches(&sample()));
    assert!(!MetadataFilter::less("score", 42_i64).matches(&sample()));
    assert!(!MetadataFilter::less("score", 10_i64).matches(&sample()));
}

#[test]
fn lte_integer() {
    assert!(MetadataFilter::less_equal("score", 42_i64).matches(&sample()));
    assert!(MetadataFilter::less_equal("score", 100_i64).matches(&sample()));
    assert!(!MetadataFilter::less_equal("score", 41_i64).matches(&sample()));
}

#[test]
fn gt_string_lexicographic() {
    assert!(MetadataFilter::greater("status", "aaa").matches(&sample()));
    assert!(!MetadataFilter::greater("status", "zzz").matches(&sample()));
}

#[test]
fn range_filter_missing_key_does_not_match() {
    assert!(!MetadataFilter::greater("missing", 0_i64).matches(&sample()));
    assert!(!MetadataFilter::greater_equal("missing", 0_i64).matches(&sample()));
    assert!(!MetadataFilter::less("missing", 100_i64).matches(&sample()));
    assert!(!MetadataFilter::less_equal("missing", 100_i64).matches(&sample()));
}

#[test]
fn range_filter_float_values() {
    assert!(MetadataFilter::greater("ratio", 0.5_f64).matches(&sample()));
    assert!(MetadataFilter::greater_equal("ratio", 0.75_f64).matches(&sample()));
    assert!(MetadataFilter::less("ratio", 1.0_f64).matches(&sample()));
    assert!(MetadataFilter::less_equal("ratio", 0.75_f64).matches(&sample()));
}

#[test]
fn range_filter_u64_values() {
    let mut m = Metadata::new();
    m.set("count", 10_u64);

    assert!(MetadataFilter::greater("count", 9_u64).matches(&m));
    assert!(MetadataFilter::greater_equal("count", 10_u64).matches(&m));
    assert!(MetadataFilter::less("count", 11_u64).matches(&m));
    assert!(MetadataFilter::less_equal("count", 10_u64).matches(&m));
}

#[test]
fn range_filter_mixed_signed_unsigned_values() {
    let mut a = Metadata::new();
    a.set("score", -1_i64);
    assert!(MetadataFilter::less("score", 0_u64).matches(&a));

    let mut b = Metadata::new();
    b.set("score", 5_u64);
    assert!(MetadataFilter::greater("score", 4_i64).matches(&b));
}

#[test]
fn range_filter_mixed_signed_unsigned_with_huge_unsigned_values() {
    let huge = (i64::MAX as u64) + 10;

    let mut negative = Metadata::new();
    negative.set("score", -1_i64);
    assert!(MetadataFilter::less("score", huge).matches(&negative));

    let mut positive = Metadata::new();
    positive.set("score", 5_i64);
    assert!(MetadataFilter::less("score", huge).matches(&positive));

    let mut huge_unsigned = Metadata::new();
    huge_unsigned.set("score", huge);
    assert!(MetadataFilter::greater("score", i64::MAX).matches(&huge_unsigned));
    assert!(MetadataFilter::greater("score", -1_i64).matches(&huge_unsigned));
    assert!(MetadataFilter::greater("score", huge - 1).matches(&huge_unsigned));
}

#[test]
fn range_filter_mixed_u64_and_f64() {
    let mut m = Metadata::new();
    m.set("count", 5_u64);

    assert!(MetadataFilter::greater("count", 4.5_f64).matches(&m));
    assert!(!MetadataFilter::less("count", 4.5_f64).matches(&m));
}

#[test]
fn range_filter_large_integer_vs_float_precision_regression() {
    let mut m = Metadata::new();
    m.set("n", 9_007_199_254_740_993_i64);

    assert!(MetadataFilter::greater("n", 9_007_199_254_740_992_f64).matches(&m));
    assert!(MetadataFilter::greater_equal("n", 9_007_199_254_740_992_f64).matches(&m));
}

#[test]
fn range_filter_large_unsigned_vs_float_precision_regression() {
    let mut m = Metadata::new();
    m.set("n", 9_007_199_254_740_993_u64);

    assert!(MetadataFilter::greater("n", 9_007_199_254_740_992_f64).matches(&m));
    assert!(MetadataFilter::greater_equal("n", 9_007_199_254_740_992_f64).matches(&m));
}

#[test]
fn range_filter_float_vs_integer_and_huge_unsigned() {
    let huge_u = (i64::MAX as u64) + 1;

    let mut m = Metadata::new();
    m.set("ratio", 3.5_f64);
    assert!(MetadataFilter::greater("ratio", 3_i64).matches(&m));

    let mut n = Metadata::new();
    n.set("value", 9_223_372_036_854_777_856_f64);
    assert!(MetadataFilter::greater("value", huge_u).matches(&n));
}

#[test]
fn range_filter_large_integer_float_non_integral_fallback() {
    let mut signed = Metadata::new();
    signed.set("n", 9_007_199_254_740_993_i64);
    assert!(!MetadataFilter::greater("n", 0.5_f64).matches(&signed));

    let mut unsigned = Metadata::new();
    unsigned.set("n", (i64::MAX as u64) + 123);
    assert!(!MetadataFilter::greater("n", 0.5_f64).matches(&unsigned));
    assert!(MetadataFilter::greater("n", -1.0_f64).matches(&unsigned));
}

#[test]
fn range_filter_incomparable_types_do_not_match() {
    assert!(!MetadataFilter::greater("status", 1_i64).matches(&sample()));
    assert!(!MetadataFilter::less("verified", 1_i64).matches(&sample()));
}

// ── Exists / NotExists ───────────────────────────────────────────────────────

#[test]
fn exists_present_key() {
    assert!(MetadataFilter::exists("status").matches(&sample()));
}

#[test]
fn exists_missing_key() {
    assert!(!MetadataFilter::exists("nope").matches(&sample()));
}

#[test]
fn not_exists_missing_key() {
    assert!(MetadataFilter::not_exists("nope").matches(&sample()));
}

#[test]
fn not_exists_present_key() {
    assert!(!MetadataFilter::not_exists("status").matches(&sample()));
}

// ── In / NotIn ───────────────────────────────────────────────────────────────

#[test]
fn in_values_matches() {
    let f = MetadataFilter::in_values("status", ["active", "pending"]);
    assert!(f.matches(&sample()));
}

#[test]
fn in_values_no_match() {
    let f = MetadataFilter::in_values("status", ["inactive", "pending"]);
    assert!(!f.matches(&sample()));
}

#[test]
fn in_values_missing_key() {
    let f = MetadataFilter::in_values("missing", ["x"]);
    assert!(!f.matches(&sample()));
}

#[test]
fn not_in_values_matches() {
    let f = MetadataFilter::not_in_values("status", ["inactive", "pending"]);
    assert!(f.matches(&sample()));
}

#[test]
fn not_in_values_no_match() {
    let f = MetadataFilter::not_in_values("status", ["active", "pending"]);
    assert!(!f.matches(&sample()));
}

#[test]
fn not_in_values_missing_key_matches() {
    let f = MetadataFilter::not_in_values("missing", ["x"]);
    assert!(f.matches(&sample()));
}

// ── Serde (Condition) ────────────────────────────────────────────────────────

#[test]
fn condition_serde_round_trip() {
    let c = Condition::Equal {
        key: "status".into(),
        value: serde_json::json!("active"),
    };
    let json = serde_json::to_string(&c).unwrap();
    let restored: Condition = serde_json::from_str(&json).unwrap();
    assert_eq!(c, restored);

    let meta = sample();
    let f = MetadataFilter::Condition(c.clone());
    let f2 = MetadataFilter::Condition(restored);
    assert_eq!(f.matches(&meta), f2.matches(&meta));
}
