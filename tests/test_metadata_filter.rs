/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! Unit tests for [`qubit_metadata::MetadataFilter`] combinators (`and`, `or`,
//! `not`) and full-tree serde. Leaf [`qubit_metadata::Condition`] tests live
//! in `test_condition.rs`.

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

// ── And ──────────────────────────────────────────────────────────────────────

#[test]
fn and_all_true() {
    let f = MetadataFilter::equal("status", "active")
        .and(MetadataFilter::greater_equal("score", 10_i64))
        .and(MetadataFilter::exists("verified"));
    assert!(f.matches(&sample()));
}

#[test]
fn and_one_false() {
    let f =
        MetadataFilter::equal("status", "active").and(MetadataFilter::greater("score", 100_i64));
    assert!(!f.matches(&sample()));
}

#[test]
fn and_flattens_children() {
    let f = MetadataFilter::equal("status", "active")
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
    let f =
        MetadataFilter::equal("status", "inactive").or(MetadataFilter::equal("status", "active"));
    assert!(f.matches(&sample()));
}

#[test]
fn or_all_false() {
    let f =
        MetadataFilter::equal("status", "inactive").or(MetadataFilter::equal("status", "pending"));
    assert!(!f.matches(&sample()));
}

#[test]
fn or_flattens_children() {
    let f = MetadataFilter::equal("status", "a")
        .or(MetadataFilter::equal("status", "b"))
        .or(MetadataFilter::equal("status", "active"));
    if let MetadataFilter::Or(children) = &f {
        assert_eq!(children.len(), 3);
    } else {
        panic!("expected Or node");
    }
}

// ── Not ──────────────────────────────────────────────────────────────────────

#[test]
fn not_inverts_true() {
    let f = MetadataFilter::equal("status", "active").not();
    assert!(!f.matches(&sample()));
}

#[test]
fn not_inverts_false() {
    let f = MetadataFilter::equal("status", "inactive").not();
    assert!(f.matches(&sample()));
}

// ── Complex compositions ─────────────────────────────────────────────────────

#[test]
fn complex_and_or_not() {
    // (status == "active" AND score >= 10) OR (NOT exists("nope"))
    let f = MetadataFilter::equal("status", "active")
        .and(MetadataFilter::greater_equal("score", 10_i64))
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

// ── Serde (MetadataFilter tree) ──────────────────────────────────────────────

#[test]
fn filter_serde_round_trip() {
    let f = MetadataFilter::equal("status", "active")
        .and(MetadataFilter::greater_equal("score", 10_i64))
        .or(MetadataFilter::exists("tag").not());

    let json = serde_json::to_string(&f).unwrap();
    let restored: MetadataFilter = serde_json::from_str(&json).unwrap();
    assert_eq!(f, restored);
    assert_eq!(f.matches(&sample()), restored.matches(&sample()));
}
