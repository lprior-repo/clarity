//! Property-based tests for `clarity-web/src/domain/newtypes.rs`.
//!
//! | Field | Value |
//! |---|---|
//! | Bead | `cl-0n6` |
//! | Target | `clarity-web/src/domain/newtypes.rs` |
//! | Lane | **P** (proptest) — PO-NT-P-01..P-09 |
//! | Output | `proofs/newtypes_proptest.rs` |
//!
//! # Proptest obligations
//!
//! - PO-NT-P-01: `AnswerId` boundary (whitespace-only → Err, non-whitespace → Ok verbatim)
//! - PO-NT-P-02: `StepId` boundary (same as P-01)
//! - PO-NT-P-03: `BeadId` boundary (same as P-01)
//! - PO-NT-P-04: FromStr(s) ≡ `try_from(s.to_string())` for all three ID types
//! - PO-NT-P-05: Timestamp boundary (valid RFC 3339 → Ok, else → Err)
//! - PO-NT-P-06: `Timestamp::default().as_str()` parses via chrono
//! - PO-NT-P-07: Display round-trip: format!("{}", t) == `t.as_str()` for all 5 types
//! - PO-NT-P-08: `AnswerValue` round-trip: `new/from/as_str/From` impls
//! - PO-NT-P-09: `serde_json` round-trip for all 5 types
//!
//! # Lint constraints
//!
//! `clarity-web/src/domain/mod.rs` denies `unwrap_used`, `expect_used` at module
//! level. Proptest bodies use `?` or `if let / else` instead of `.unwrap()` to
//! satisfy this constraint. The proptest file lives outside the domain module tree
//! (at `proofs/newtypes_proptest.rs`) to avoid triggering the deny lints during
//! normal compilation of the library.
//!
//! # Wiring note
//!
//! This file lives at `proofs/newtypes_proptest.rs` per the proof-writer brief.
//! To execute it as a `cargo test` integration test, copy or move the file to
//! `clarity-web/tests/newtypes_proptest.rs`.

#![allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::match_same_arms
)]

use clarity_web::domain::{AnswerId, AnswerValue, BeadId, StepId, Timestamp};
use proptest::prelude::*;
use std::str::FromStr;

// ============================================================
// Helpers
// ============================================================

fn is_whitespace_only(s: &str) -> bool {
  s.trim().is_empty()
}

// ============================================================
// PO-NT-P-01: AnswerId boundary
// ============================================================

proptest! {
    #[test]
    fn proptest_answer_id_boundary(s in ".*") {
        let result = AnswerId::try_from(s.clone());
        if is_whitespace_only(&s) {
            // Whitespace-only → Err
            prop_assert!(result.is_err(), "AnswerId should reject whitespace-only string: {:?}", s);
        } else {
            // Non-whitespace → Ok with inner verbatim
            prop_assert!(result.is_ok(), "AnswerId should accept non-whitespace string: {:?}", s);
            if let Ok(id) = result {
                prop_assert_eq!(id.as_str(), s.as_str(), "AnswerId inner should be verbatim for {:?}", s);
            }
        }
    }
}

// ============================================================
// PO-NT-P-02: StepId boundary
// ============================================================

proptest! {
    #[test]
    fn proptest_step_id_boundary(s in ".*") {
        let result = StepId::try_from(s.clone());
        if is_whitespace_only(&s) {
            prop_assert!(result.is_err(), "StepId should reject whitespace-only string: {:?}", s);
        } else {
            prop_assert!(result.is_ok(), "StepId should accept non-whitespace string: {:?}", s);
            if let Ok(id) = result {
                prop_assert_eq!(id.as_str(), s.as_str(), "StepId inner should be verbatim for {:?}", s);
            }
        }
    }
}

// ============================================================
// PO-NT-P-03: BeadId boundary
// ============================================================

proptest! {
    #[test]
    fn proptest_bead_id_boundary(s in ".*") {
        let result = BeadId::try_from(s.clone());
        if is_whitespace_only(&s) {
            prop_assert!(result.is_err(), "BeadId should reject whitespace-only string: {:?}", s);
        } else {
            prop_assert!(result.is_ok(), "BeadId should accept non-whitespace string: {:?}", s);
            if let Ok(id) = result {
                prop_assert_eq!(id.as_str(), s.as_str(), "BeadId inner should be verbatim for {:?}", s);
            }
        }
    }
}

// ============================================================
// PO-NT-P-04: FromStr(s) ≡ try_from(s.to_string()) for all three ID types
// ============================================================

proptest! {
    #[test]
    fn proptest_from_str_equivalence_answer_id(s in ".*") {
        let try_result = AnswerId::try_from(s.clone());
        let from_str_result = AnswerId::from_str(&s);
        match (try_result, from_str_result) {
            (Ok(t1), Ok(t2)) => {
                prop_assert_eq!(t1.as_str(), t2.as_str());
            },
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(format!("{:?}", e1), format!("{:?}", e2));
            },
            _ => {
                prop_assert!(false, "try_from and from_str must agree for {:?}", s);
            }
        }
    }
}

proptest! {
    #[test]
    fn proptest_from_str_equivalence_step_id(s in ".*") {
        let try_result = StepId::try_from(s.clone());
        let from_str_result = StepId::from_str(&s);
        match (try_result, from_str_result) {
            (Ok(t1), Ok(t2)) => {
                prop_assert_eq!(t1.as_str(), t2.as_str());
            },
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(format!("{:?}", e1), format!("{:?}", e2));
            },
            _ => {
                prop_assert!(false, "try_from and from_str must agree for {:?}", s);
            }
        }
    }
}

proptest! {
    #[test]
    fn proptest_from_str_equivalence_bead_id(s in ".*") {
        let try_result = BeadId::try_from(s.clone());
        let from_str_result = BeadId::from_str(&s);
        match (try_result, from_str_result) {
            (Ok(t1), Ok(t2)) => {
                prop_assert_eq!(t1.as_str(), t2.as_str());
            },
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(format!("{:?}", e1), format!("{:?}", e2));
            },
            _ => {
                prop_assert!(false, "try_from and from_str must agree for {:?}", s);
            }
        }
    }
}

// ============================================================
// PO-NT-P-05: Timestamp boundary (valid RFC 3339 → Ok, else → Err)
// ============================================================

proptest! {
    #[test]
    fn proptest_timestamp_try_from(s in ".*") {
        let parseable = chrono::DateTime::parse_from_rfc3339(&s).is_ok();
        let result = Timestamp::try_from(s.clone());
        if parseable && !is_whitespace_only(&s) {
            prop_assert!(result.is_ok(), "Timestamp should accept RFC 3339 parseable string: {:?}", s);
            if let Ok(ts) = result {
                prop_assert_eq!(ts.as_str(), s.as_str(), "Timestamp inner should be verbatim for {:?}", s);
            }
        } else {
            prop_assert!(result.is_err(), "Timestamp should reject non-RFC-3339 or whitespace string: {:?}", s);
        }
    }
}

// ============================================================
// PO-NT-P-06: Timestamp::default().as_str() parses via chrono
// ============================================================

proptest! {
    #[test]
    fn proptest_timestamp_default_roundtrip(_i in 0..64) {
        let ts = Timestamp::default();
        let parsed: Result<chrono::DateTime<chrono::Utc>, _> = ts.as_str().parse();
        prop_assert!(parsed.is_ok(), "Timestamp::default().as_str() must parse as RFC 3339: {:?}", ts.as_str());
    }
}

// ============================================================
// PO-NT-P-07: Display round-trip: format!("{}", t) == t.as_str() for all 5 types
// ============================================================

proptest! {
    #[test]
    fn proptest_display_roundtrip_answer_id(s in ".*") {
        if is_whitespace_only(&s) {
            return Ok(());
        }
        let id = AnswerId::try_from(s)?;
        let formatted = format!("{id}");
        prop_assert_eq!(formatted.as_str(), id.as_str(), "format!() must equal as_str() for AnswerId");
    }
}

proptest! {
    #[test]
    fn proptest_display_roundtrip_step_id(s in ".*") {
        if is_whitespace_only(&s) {
            return Ok(());
        }
        let id = StepId::try_from(s)?;
        let formatted = format!("{id}");
        prop_assert_eq!(formatted.as_str(), id.as_str(), "format!() must equal as_str() for StepId");
    }
}

proptest! {
    #[test]
    fn proptest_display_roundtrip_bead_id(s in ".*") {
        if is_whitespace_only(&s) {
            return Ok(());
        }
        let id = BeadId::try_from(s)?;
        let formatted = format!("{id}");
        prop_assert_eq!(formatted.as_str(), id.as_str(), "format!() must equal as_str() for BeadId");
    }
}

proptest! {
    #[test]
    fn proptest_display_roundtrip_timestamp(_i in 0..64) {
        let ts = Timestamp::default();
        let formatted = format!("{ts}");
        prop_assert_eq!(formatted.as_str(), ts.as_str(), "format!() must equal as_str() for Timestamp");
    }
}

proptest! {
    #[test]
    fn proptest_display_roundtrip_answer_value(s in ".*") {
        let v = AnswerValue::new(s);
        let formatted = format!("{v}");
        prop_assert_eq!(formatted.as_str(), v.as_str(), "format!() must equal as_str() for AnswerValue");
    }
}

// ============================================================
// PO-NT-P-08: AnswerValue round-trip: new/from/as_str/From impls
// ============================================================

proptest! {
    #[test]
    fn proptest_answer_value_roundtrip(s in ".*") {
        // (a) new(s).as_str() == s.as_str()
        let v = AnswerValue::new(s.clone());
        prop_assert_eq!(v.as_str(), s.as_str(), "AnswerValue::new must preserve string");

        // (b) String::from(AnswerValue::new(s)) == s
        let s2 = std::string::String::from(v);
        prop_assert_eq!(s2.as_str(), s.as_str(), "String::from must extract inner");

        // (c) AnswerValue::from(s.as_str()).as_str() == s.as_str()
        let v2 = AnswerValue::from(s.as_str());
        prop_assert_eq!(v2.as_str(), s.as_str(), "AnswerValue::from(&str) must preserve string");

        // (d) AnswerValue::default().as_str() == ""
        let v_default = AnswerValue::default();
        prop_assert!(v_default.is_empty(), "AnswerValue::default() must be empty");
    }
}

// ============================================================
// PO-NT-P-09: serde_json round-trip for all 5 types
// ============================================================

proptest! {
    #[test]
    fn proptest_serde_roundtrip_answer_id(s in ".*") {
        if is_whitespace_only(&s) {
            return Ok(());
        }
        let id = AnswerId::try_from(s)?;
        let json = serde_json::to_string(&id)?;
        let roundtrip = serde_json::from_str::<AnswerId>(&json)?;
        prop_assert_eq!(roundtrip.as_str(), id.as_str(), "serde_json round-trip must preserve AnswerId");
    }
}

proptest! {
    #[test]
    fn proptest_serde_roundtrip_step_id(s in ".*") {
        if is_whitespace_only(&s) {
            return Ok(());
        }
        let id = StepId::try_from(s)?;
        let json = serde_json::to_string(&id)?;
        let roundtrip = serde_json::from_str::<StepId>(&json)?;
        prop_assert_eq!(roundtrip.as_str(), id.as_str(), "serde_json round-trip must preserve StepId");
    }
}

proptest! {
    #[test]
    fn proptest_serde_roundtrip_bead_id(s in ".*") {
        if is_whitespace_only(&s) {
            return Ok(());
        }
        let id = BeadId::try_from(s)?;
        let json = serde_json::to_string(&id)?;
        let roundtrip = serde_json::from_str::<BeadId>(&json)?;
        prop_assert_eq!(roundtrip.as_str(), id.as_str(), "serde_json round-trip must preserve BeadId");
    }
}

proptest! {
    #[test]
    fn proptest_serde_roundtrip_answer_value(s in ".*") {
        let v = AnswerValue::new(s);
        let json = serde_json::to_string(&v)?;
        let roundtrip = serde_json::from_str::<AnswerValue>(&json)?;
        prop_assert_eq!(roundtrip.as_str(), v.as_str(), "serde_json round-trip must preserve AnswerValue");
    }
}

proptest! {
    #[test]
    fn proptest_serde_roundtrip_timestamp(_i in 0..64) {
        let ts = Timestamp::default();
        let json = serde_json::to_string(&ts)?;
        let roundtrip = serde_json::from_str::<Timestamp>(&json)?;
        prop_assert_eq!(roundtrip.as_str(), ts.as_str(), "serde_json round-trip must preserve Timestamp");
    }
}
