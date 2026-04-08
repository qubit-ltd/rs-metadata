/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Unit tests for [`qubit_metadata::MetadataFilter`] and [`qubit_metadata::Condition`].

use qubit_metadata::{
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

// ── Eq / Ne ──────────────────────────────────────────────────────────────────

#[test]
fn eq_matches_equal_string() {
    let f = MetadataFilter::eq("status", "active");
    assert!(f.matches(&sample()));
}

#[test]
fn eq_does_not_match_different_string() {
    let f = MetadataFilter::eq("status", "inactive");
    assert!(!f.matches(&sample()));
}

#[test]
fn eq_missing_key_does_not_match() {
    let f = MetadataFilter::eq("missing", "x");
    assert!(!f.matches(&sample()));
}

#[test]
fn ne_matches_different_value() {
    let f = MetadataFilter::ne("status", "inactive");
    assert!(f.matches(&sample()));
}

#[test]
fn ne_does_not_match_equal_value() {
    let f = MetadataFilter::ne("status", "active");
    assert!(!f.matches(&sample()));
}

#[test]
fn ne_missing_key_matches() {
    let f = MetadataFilter::ne("missing", "anything");
    assert!(f.matches(&sample()));
}

// ── Gt / Gte / Lt / Lte ──────────────────────────────────────────────────────

#[test]
fn gt_integer() {
    assert!(MetadataFilter::gt("score", 10_i64).matches(&sample()));
    assert!(!MetadataFilter::gt("score", 42_i64).matches(&sample()));
    assert!(!MetadataFilter::gt("score", 100_i64).matches(&sample()));
}

#[test]
fn gte_integer() {
    assert!(MetadataFilter::gte("score", 42_i64).matches(&sample()));
    assert!(MetadataFilter::gte("score", 10_i64).matches(&sample()));
    assert!(!MetadataFilter::gte("score", 43_i64).matches(&sample()));
}

#[test]
fn lt_integer() {
    assert!(MetadataFilter::lt("score", 100_i64).matches(&sample()));
    assert!(!MetadataFilter::lt("score", 42_i64).matches(&sample()));
    assert!(!MetadataFilter::lt("score", 10_i64).matches(&sample()));
}

#[test]
fn lte_integer() {
    assert!(MetadataFilter::lte("score", 42_i64).matches(&sample()));
    assert!(MetadataFilter::lte("score", 100_i64).matches(&sample()));
    assert!(!MetadataFilter::lte("score", 41_i64).matches(&sample()));
}

#[test]
fn gt_string_lexicographic() {
    assert!(MetadataFilter::gt("status", "aaa").matches(&sample()));
    assert!(!MetadataFilter::gt("status", "zzz").matches(&sample()));
}

#[test]
fn range_filter_missing_key_does_not_match() {
    assert!(!MetadataFilter::gt("missing", 0_i64).matches(&sample()));
    assert!(!MetadataFilter::gte("missing", 0_i64).matches(&sample()));
    assert!(!MetadataFilter::lt("missing", 100_i64).matches(&sample()));
    assert!(!MetadataFilter::lte("missing", 100_i64).matches(&sample()));
}

#[test]
fn range_filter_float_values() {
    assert!(MetadataFilter::gt("ratio", 0.5_f64).matches(&sample()));
    assert!(MetadataFilter::gte("ratio", 0.75_f64).matches(&sample()));
    assert!(MetadataFilter::lt("ratio", 1.0_f64).matches(&sample()));
    assert!(MetadataFilter::lte("ratio", 0.75_f64).matches(&sample()));
}

#[test]
fn range_filter_u64_values() {
    let mut m = Metadata::new();
    m.set("count", 10_u64);

    assert!(MetadataFilter::gt("count", 9_u64).matches(&m));
    assert!(MetadataFilter::gte("count", 10_u64).matches(&m));
    assert!(MetadataFilter::lt("count", 11_u64).matches(&m));
    assert!(MetadataFilter::lte("count", 10_u64).matches(&m));
}

#[test]
fn range_filter_mixed_signed_unsigned_values() {
    let mut a = Metadata::new();
    a.set("score", -1_i64);
    assert!(MetadataFilter::lt("score", 0_u64).matches(&a));

    let mut b = Metadata::new();
    b.set("score", 5_u64);
    assert!(MetadataFilter::gt("score", 4_i64).matches(&b));
}

#[test]
fn range_filter_mixed_signed_unsigned_with_huge_unsigned_values() {
    let huge = (i64::MAX as u64) + 10;

    let mut negative = Metadata::new();
    negative.set("score", -1_i64);
    assert!(MetadataFilter::lt("score", huge).matches(&negative));

    let mut positive = Metadata::new();
    positive.set("score", 5_i64);
    assert!(MetadataFilter::lt("score", huge).matches(&positive));

    let mut huge_unsigned = Metadata::new();
    huge_unsigned.set("score", huge);
    assert!(MetadataFilter::gt("score", i64::MAX).matches(&huge_unsigned));
    assert!(MetadataFilter::gt("score", -1_i64).matches(&huge_unsigned));
    assert!(MetadataFilter::gt("score", huge - 1).matches(&huge_unsigned));
}

#[test]
fn range_filter_mixed_u64_and_f64() {
    let mut m = Metadata::new();
    m.set("count", 5_u64);

    assert!(MetadataFilter::gt("count", 4.5_f64).matches(&m));
    assert!(!MetadataFilter::lt("count", 4.5_f64).matches(&m));
}

#[test]
fn range_filter_large_integer_vs_float_precision_regression() {
    let mut m = Metadata::new();
    m.set("n", 9_007_199_254_740_993_i64);

    assert!(MetadataFilter::gt("n", 9_007_199_254_740_992_f64).matches(&m));
    assert!(MetadataFilter::gte("n", 9_007_199_254_740_992_f64).matches(&m));
}

#[test]
fn range_filter_large_unsigned_vs_float_precision_regression() {
    let mut m = Metadata::new();
    m.set("n", 9_007_199_254_740_993_u64);

    assert!(MetadataFilter::gt("n", 9_007_199_254_740_992_f64).matches(&m));
    assert!(MetadataFilter::gte("n", 9_007_199_254_740_992_f64).matches(&m));
}

#[test]
fn range_filter_float_vs_integer_and_huge_unsigned() {
    let huge_u = (i64::MAX as u64) + 1;

    let mut m = Metadata::new();
    m.set("ratio", 3.5_f64);
    assert!(MetadataFilter::gt("ratio", 3_i64).matches(&m));

    let mut n = Metadata::new();
    n.set("value", 9_223_372_036_854_777_856_f64);
    assert!(MetadataFilter::gt("value", huge_u).matches(&n));
}

#[test]
fn range_filter_large_integer_float_non_integral_fallback() {
    let mut signed = Metadata::new();
    signed.set("n", 9_007_199_254_740_993_i64);
    assert!(!MetadataFilter::gt("n", 0.5_f64).matches(&signed));

    let mut unsigned = Metadata::new();
    unsigned.set("n", (i64::MAX as u64) + 123);
    assert!(!MetadataFilter::gt("n", 0.5_f64).matches(&unsigned));
    assert!(MetadataFilter::gt("n", -1.0_f64).matches(&unsigned));
}

#[test]
fn range_filter_incomparable_types_do_not_match() {
    assert!(!MetadataFilter::gt("status", 1_i64).matches(&sample()));
    assert!(!MetadataFilter::lt("verified", 1_i64).matches(&sample()));
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

// ── And ──────────────────────────────────────────────────────────────────────

#[test]
fn and_all_true() {
    let f = MetadataFilter::eq("status", "active")
        .and(MetadataFilter::gte("score", 10_i64))
        .and(MetadataFilter::exists("verified"));
    assert!(f.matches(&sample()));
}

#[test]
fn and_one_false() {
    let f = MetadataFilter::eq("status", "active").and(MetadataFilter::gt("score", 100_i64));
    assert!(!f.matches(&sample()));
}

#[test]
fn and_flattens_children() {
    let f = MetadataFilter::eq("status", "active")
        .and(MetadataFilter::exists("score"))
        .and(MetadataFilter::exists("tag"));
    if let MetadataFilter::And(children) = &f {
        assert_eq!(children.len(), 3);
    } else {
        panic!("expected And node");
    }
}

// ── Or ───────────────────────────────────────────────────────────────────────

#[test]
fn or_one_true() {
    let f = MetadataFilter::eq("status", "inactive").or(MetadataFilter::eq("status", "active"));
    assert!(f.matches(&sample()));
}

#[test]
fn or_all_false() {
    let f = MetadataFilter::eq("status", "inactive").or(MetadataFilter::eq("status", "pending"));
    assert!(!f.matches(&sample()));
}

#[test]
fn or_flattens_children() {
    let f = MetadataFilter::eq("status", "a")
        .or(MetadataFilter::eq("status", "b"))
        .or(MetadataFilter::eq("status", "active"));
    if let MetadataFilter::Or(children) = &f {
        assert_eq!(children.len(), 3);
    } else {
        panic!("expected Or node");
    }
}

// ── Not ──────────────────────────────────────────────────────────────────────

#[test]
fn not_inverts_true() {
    let f = MetadataFilter::eq("status", "active").not();
    assert!(!f.matches(&sample()));
}

#[test]
fn not_inverts_false() {
    let f = MetadataFilter::eq("status", "inactive").not();
    assert!(f.matches(&sample()));
}

// ── Complex compositions ─────────────────────────────────────────────────────

#[test]
fn complex_and_or_not() {
    // (status == "active" AND score >= 10) OR (NOT exists("nope"))
    let f = MetadataFilter::eq("status", "active")
        .and(MetadataFilter::gte("score", 10_i64))
        .or(MetadataFilter::exists("nope").not());
    assert!(f.matches(&sample()));
}

#[test]
fn empty_and_matches_everything() {
    let f = MetadataFilter::And(vec![]);
    assert!(f.matches(&sample()));
    assert!(f.matches(&Metadata::new()));
}

#[test]
fn empty_or_matches_nothing() {
    let f = MetadataFilter::Or(vec![]);
    assert!(!f.matches(&sample()));
    assert!(!f.matches(&Metadata::new()));
}

// ── Serde round-trip ─────────────────────────────────────────────────────────

#[test]
fn filter_serde_round_trip() {
    let f = MetadataFilter::eq("status", "active")
        .and(MetadataFilter::gte("score", 10_i64))
        .or(MetadataFilter::exists("tag").not());

    let json = serde_json::to_string(&f).unwrap();
    let restored: MetadataFilter = serde_json::from_str(&json).unwrap();
    assert_eq!(f, restored);
    assert_eq!(f.matches(&sample()), restored.matches(&sample()));
}
