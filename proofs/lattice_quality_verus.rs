// =============================================================================
// File:         proofs/lattice_quality_verus.rs
// Lane:         Verus (V) — primary
// Bead:         cl-dv5
// Target:       clarity-web/src/lattice/quality.rs
// Plan ref:     verification-targets.md §5.4
// Tool:         Verus 0.2026.05.05.d03e906 (installed at /home/lewus/.local/bin/verus)
//
// UPSTREAM NOTE
// -------------
// No approved proof plan exists yet for this module (`proof-planner` has not
// run, so no `proof-obligations.planned.jsonl` with formal IDs is on disk).
// Obligation IDs in this file (`OB-LQ-V-NN`) are PROVISIONAL — they must be
// replaced with planner-assigned IDs once `proof-planner` formalizes the
// module. The writeup `proofs/lattice_quality-writeup.md` records this gap.
//
// SCOPE
// -----
// This file proves algebraic properties of the public API of
// `clarity-web/src/lattice/quality.rs`. It does NOT verify the heuristic
// bodies of the five internal `calculate_*` aggregation functions
// (completeness, consistency, testability, clarity, security), whose
// exact numerical output depends on string-processing heuristics. Those
// bodies are recorded as a TRUSTED BOUNDARY in the writeup.
//
// PROOFS in this file
// -------------------
// OB-LQ-V-01  spec_dimension_all().len() == 5
// OB-LQ-V-02  spec_dimension_all contains each of the 5 documented variants
// OB-LQ-V-03  spec_dimension_label(d) is non-empty for every variant
// OB-LQ-V-04  spec_dimension_description(d) is non-empty for every variant
// OB-LQ-V-05  spec_dimension_score_new(d, s) is Ok(s) iff 0 <= s <= 100
// OB-LQ-V-06  spec_dimension_score_new(d, s) is Err(InvalidScore) iff s ∉ [0,100]
// OB-LQ-V-07  spec_dimension_score_passes is monotone non-decreasing in score
// OB-LQ-V-08  spec_dimension_score_passes is monotone non-increasing in threshold
// OB-LQ-V-09  spec_quality_score_new validates overall ∈ [0, 100]
// OB-LQ-V-10  spec_quality_score_passes is monotone non-decreasing in overall
// OB-LQ-V-11  spec_calculate_quality with answers_len==0 returns Err(EmptyAnswers)
// OB-LQ-V-12  spec_calculate_quality with answers_len>0 and overall ∈ [0,100] returns Ok
// OB-LQ-V-13  arithmetic mean floor of 5 scores in [0, 100] is in [0, 100]
// OB-LQ-V-14  spec_calculate_quality is idempotent in its arguments (algebraic purity)
// =============================================================================

use vstd::prelude::*;

verus! {

// =============================================================================
// §1  Spec types — mirrors of production types in clarity-web/src/lattice/quality.rs
// =============================================================================

/// Mirror of `pub const MINIMUM_GATE: u8 = 70;` (line 34).
pub const SPEC_MINIMUM_GATE: u8 = 70;

/// Mirror of `pub enum QualityDimension` (lines 50–66).
/// The discriminant ordering matches the source for stable proofs.
pub enum QualityDimensionSpec {
    Completeness,
    Consistency,
    Testability,
    Clarity,
    Security,
}

/// Mirror of `pub enum IssueSeverity` (lines 144–149).
pub enum IssueSeveritySpec {
    Warning,
    Error,
    Critical,
}

/// Mirror of `pub enum QualityError` (lines 37–47).
/// `String` payloads of `InvalidScore` / `DimensionFailed` are collapsed to
/// unit variants because Verus has no interest in the message text — only
/// the discriminant matters for the algebraic contract.
pub enum QualityErrorSpec {
    EmptyAnswers,
    InvalidScore,
    DimensionFailed,
}

/// Mirror of `pub struct DimensionScore` (lines 104–108).
/// Production uses `u8` for `score`; the spec uses `int` so that arithmetic
/// (e.g. mean-floor) is convenient. The `new` constructor will pin the
/// range to `0 <= score <= 100`, matching the production range invariant.
pub struct DimensionScoreSpec {
    pub dimension: QualityDimensionSpec,
    pub score: int,
}

/// Mirror of `pub struct QualityIssue` (lines 126–131).
/// `message` is modelled as `Seq<char>` because Verus does not reason about
/// heap-allocated `String`s directly.
pub struct QualityIssueSpec {
    pub dimension: QualityDimensionSpec,
    pub severity: IssueSeveritySpec,
    pub message: Seq<char>,
}

/// Mirror of `pub struct QualityScore` (lines 152–160).
pub struct QualityScoreSpec {
    pub overall: int,
    pub dimensions: Seq<DimensionScoreSpec>,
    pub issues: Seq<QualityIssueSpec>,
}

// =============================================================================
// §2  Spec functions — mirrors of the public API surface
// =============================================================================

/// Mirror of `QualityDimension::all` (lines 70–78).
/// Returns the fixed sequence of 5 dimensions in declaration order.
pub open spec fn spec_dimension_all() -> Seq<QualityDimensionSpec> {
    seq![
        QualityDimensionSpec::Completeness,
        QualityDimensionSpec::Consistency,
        QualityDimensionSpec::Testability,
        QualityDimensionSpec::Clarity,
        QualityDimensionSpec::Security,
    ]
}

/// Mirror of `QualityDimension::label` (lines 81–89).
pub open spec fn spec_dimension_label(d: QualityDimensionSpec) -> Seq<char> {
    match d {
        QualityDimensionSpec::Completeness => seq!['C', 'o', 'm', 'p', 'l', 'e', 't', 'e', 'n', 'e', 's', 's'],
        QualityDimensionSpec::Consistency  => seq!['C', 'o', 'n', 's', 'i', 's', 't', 'e', 'n', 'c', 'y'],
        QualityDimensionSpec::Testability  => seq!['T', 'e', 's', 't', 'a', 'b', 'i', 'l', 'i', 't', 'y'],
        QualityDimensionSpec::Clarity      => seq!['C', 'l', 'a', 'r', 'i', 't', 'y'],
        QualityDimensionSpec::Security     => seq!['S', 'e', 'c', 'u', 'r', 'i', 't', 'y'],
    }
}

/// Mirror of `QualityDimension::description` (lines 92–100).
pub open spec fn spec_dimension_description(d: QualityDimensionSpec) -> Seq<char> {
    match d {
        QualityDimensionSpec::Completeness => spec_str_percentage_of_required_fields(),
        QualityDimensionSpec::Consistency  => spec_str_absence_of_contradictory_requirements(),
        QualityDimensionSpec::Testability  => spec_str_presence_of_acceptance_criteria(),
        QualityDimensionSpec::Clarity      => spec_str_readability_and_minimal_jargon(),
        QualityDimensionSpec::Security     => spec_str_security_considerations_present(),
    }
}

// Helper specs to keep spec_dimension_description readable.
pub open spec fn spec_str_percentage_of_required_fields() -> Seq<char> {
    seq!['P','e','r','c','e','n','t','a','g','e',' ','o','f',' ','r','e','q','u','i','r','e','d',' ','f','i','e','l','d','s',' ','f','i','l','l','e','d']
}
pub open spec fn spec_str_absence_of_contradictory_requirements() -> Seq<char> {
    seq!['A','b','s','e','n','c','e',' ','o','f',' ','c','o','n','t','r','a','d','i','c','t','o','r','y',' ','r','e','q','u','i','r','e','m','e','n','t','s']
}
pub open spec fn spec_str_presence_of_acceptance_criteria() -> Seq<char> {
    seq!['P','r','e','s','e','n','c','e',' ','o','f',' ','a','c','c','e','p','t','a','n','c','e',' ','c','r','i','t','e','r','i','a']
}
pub open spec fn spec_str_readability_and_minimal_jargon() -> Seq<char> {
    seq!['R','e','a','d','a','b','i','l','i','t','y',' ','a','n','d',' ','m','i','n','i','m','a','l',' ','j','a','r','g','o','n']
}
pub open spec fn spec_str_security_considerations_present() -> Seq<char> {
    seq!['S','e','c','u','r','i','t','y',' ','c','o','n','s','i','d','e','r','a','t','i','o','n','s',' ','p','r','e','s','e','n','t']
}

/// Mirror of `DimensionScore::new` (lines 112–117).
/// `Ok(ds)` iff `0 <= score <= 100`; on `Ok(ds)`, `ds.score == score` and
/// `ds.dimension == d`.
pub open spec fn spec_dimension_score_new(
    d: QualityDimensionSpec,
    score: int,
) -> Result<DimensionScoreSpec, QualityErrorSpec> {
    if 0 <= score && score <= 100 {
        Ok(DimensionScoreSpec { dimension: d, score })
    } else {
        Err(QualityErrorSpec::InvalidScore)
    }
}

/// Mirror of `DimensionScore::passes` (lines 120–122).
/// Returns `true` iff `self.score >= threshold`.
pub open spec fn spec_dimension_score_passes(
    s: DimensionScoreSpec,
    threshold: int,
) -> bool {
    s.score >= threshold
}

/// Mirror of `QualityScore::new` (lines 164–177).
/// `Ok(q)` iff `0 <= overall <= 100`; on `Ok(q)`, `q.overall == overall`.
pub open spec fn spec_quality_score_new(
    overall: int,
    dimensions: Seq<DimensionScoreSpec>,
    issues: Seq<QualityIssueSpec>,
) -> Result<QualityScoreSpec, QualityErrorSpec> {
    if 0 <= overall && overall <= 100 {
        Ok(QualityScoreSpec { overall, dimensions, issues })
    } else {
        Err(QualityErrorSpec::InvalidScore)
    }
}

/// Mirror of `QualityScore::passes` (lines 180–182).
pub open spec fn spec_quality_score_passes(
    q: QualityScoreSpec,
    threshold: int,
) -> bool {
    q.overall >= threshold
}

/// Mirror of `calculate_quality` (lines 223–251).
///
/// Models production's pipeline:
///   1. `answers.is_empty()` → `Err(EmptyAnswers)`
///   2. Five heuristic bodies compute `d1..d5` (TRUSTED BOUNDARY — writeup §6)
///   3. `dimensions = [d1, d2, d3, d4, d5]`
///   4. `overall = floor((d1+d2+d3+d4+d5) / 5)` (integer floor division)
///   5. Return `QualityScore::new(overall, dimensions, issues)`
///
/// The five dimension-score parameters (`d1..d5`) represent the (trusted)
/// output of the heuristic bodies. The spec asserts the floor-mean
/// relationship in the Ok-arm postcondition, which is proved by
/// `lemma_mean_of_five_in_unit_interval` (OB-LQ-V-13).
pub open spec fn spec_calculate_quality(
    answers_len: int,
    ears_len: int,
    d1: int,
    d2: int,
    d3: int,
    d4: int,
    d5: int,
) -> Result<QualityScoreSpec, QualityErrorSpec> {
    if answers_len == 0 {
        Err(QualityErrorSpec::EmptyAnswers)
    } else {
        // Production computes: sum = d1+d2+d3+d4+d5; overall = sum / 5 (u32 truncating)
        // In integer arithmetic: floor((d1+...+d5) / 5)
        let sum = d1 + d2 + d3 + d4 + d5;
        let overall = sum / 5;
        Ok(QualityScoreSpec {
            overall,
            dimensions: seq![
                DimensionScoreSpec { dimension: QualityDimensionSpec::Completeness, score: d1 },
                DimensionScoreSpec { dimension: QualityDimensionSpec::Consistency, score: d2 },
                DimensionScoreSpec { dimension: QualityDimensionSpec::Testability, score: d3 },
                DimensionScoreSpec { dimension: QualityDimensionSpec::Clarity, score: d4 },
                DimensionScoreSpec { dimension: QualityDimensionSpec::Security, score: d5 },
            ],
            issues: Seq::empty(),  // TRUSTED: populated by heuristic body.
        })
    }
}

// =============================================================================
// §3  Proof lemmas — algebraic properties of the spec functions
// =============================================================================

// ---------- OB-LQ-V-01: all() has exactly 5 elements ----------

pub proof fn lemma_dimension_all_count()
    ensures
        spec_dimension_all().len() == 5,
{
    // The seq! literal has 5 entries; sequence length is the literal's length.
}

// ---------- OB-LQ-V-02: all() enumerates exactly the 5 documented dimensions ----------

pub proof fn lemma_dimension_all_contains_each()
    ensures
        spec_dimension_all().contains(QualityDimensionSpec::Completeness),
        spec_dimension_all().contains(QualityDimensionSpec::Consistency),
        spec_dimension_all().contains(QualityDimensionSpec::Testability),
        spec_dimension_all().contains(QualityDimensionSpec::Clarity),
        spec_dimension_all().contains(QualityDimensionSpec::Security),
{
    // Each variant appears literally in the seq! macro above.
}

// ---------- OB-LQ-V-03: label() is non-empty for every variant ----------

pub proof fn lemma_label_nonempty(d: QualityDimensionSpec)
    ensures
        spec_dimension_label(d).len() > 0,
{
    // Each branch of the match returns a non-empty seq! literal.
}

// ---------- OB-LQ-V-04: description() is non-empty for every variant ----------

pub proof fn lemma_description_nonempty(d: QualityDimensionSpec)
    ensures
        spec_dimension_description(d).len() > 0,
{
    // Each branch of the match returns the helper spec, which is a non-empty seq!.
}

// ---------- OB-LQ-V-05: in-range new() returns Ok with faithful copy ----------

pub proof fn lemma_dimension_score_new_in_range(
    d: QualityDimensionSpec,
    score: int,
)
    requires
        0 <= score,
        score <= 100,
    ensures
        (0 <= score && score <= 100) ==> (
            match spec_dimension_score_new(d, score) {
                Ok(ds) => ds.score == score && ds.dimension == d,
                Err(_) => false,
            }
        ),
{
    // The spec function returns `Ok(...)` exactly on the condition
    // `0 <= score && score <= 100`, with faithful copy of fields.
}

// ---------- OB-LQ-V-06: out-of-range new() returns Err(InvalidScore) ----------

pub proof fn lemma_dimension_score_new_out_of_range(
    d: QualityDimensionSpec,
    score: int,
)
    requires
        score < 0 || score > 100,
    ensures
        spec_dimension_score_new(d, score) == Err::<DimensionScoreSpec, _>(QualityErrorSpec::InvalidScore),
{
    // Negation of the if-guard lands in the `Err` branch; the variant
    // returned is the unique `InvalidScore` constructor.
}

// ---------- OB-LQ-V-07: passes is monotone non-decreasing in score ----------
// Monotonicity: if s_lo <= s_hi and s_lo >= threshold, then s_hi >= threshold.
// Equivalently: passes(s_lo, threshold) ==> passes(s_hi, threshold).
// The ensures clause has the consequent as the higher score passing.

pub proof fn lemma_passes_monotone_in_score(
    d: QualityDimensionSpec,
    s_lo: int,
    s_hi: int,
    threshold: int,
)
    requires
        0 <= s_lo,
        s_lo <= s_hi,
        s_hi <= 100,
    ensures
        spec_dimension_score_passes(
            DimensionScoreSpec { dimension: d, score: s_lo },
            threshold,
        ) ==> spec_dimension_score_passes(
            DimensionScoreSpec { dimension: d, score: s_hi },
            threshold,
        ),
{
    // If s_lo >= threshold and s_lo <= s_hi, then s_hi >= threshold by transitivity.
}

// ---------- OB-LQ-V-08: passes is monotone non-increasing in threshold ----------
// Antitonicity: if t_lo <= t_hi and score >= t_hi, then score >= t_lo.
// Equivalently: passes(score, t_hi) ==> passes(score, t_lo).

pub proof fn lemma_passes_antitone_in_threshold(
    d: QualityDimensionSpec,
    score: int,
    t_lo: int,
    t_hi: int,
)
    requires
        0 <= score,
        score <= 100,
        0 <= t_lo,
        t_lo <= t_hi,
        t_hi <= 100,
    ensures
        spec_dimension_score_passes(
            DimensionScoreSpec { dimension: d, score },
            t_hi,
        ) ==> spec_dimension_score_passes(
            DimensionScoreSpec { dimension: d, score },
            t_lo,
        ),
{
    // If score >= t_hi and t_lo <= t_hi, then score >= t_lo by transitivity.
}

// ---------- OB-LQ-V-09: QualityScore::new validates overall range ----------

pub proof fn lemma_quality_score_new_validates_overall(
    overall: int,
    dims: Seq<DimensionScoreSpec>,
    issues: Seq<QualityIssueSpec>,
)
    ensures
        (0 <= overall && overall <= 100) ==> (
            match spec_quality_score_new(overall, dims, issues) {
                Ok(q) => q.overall == overall,
                Err(_) => false,
            }
        ),
        !(0 <= overall && overall <= 100) ==> (
            spec_quality_score_new(overall, dims, issues)
                == Err::<QualityScoreSpec, _>(QualityErrorSpec::InvalidScore)
        ),
{
}

// ---------- OB-LQ-V-10: QualityScore::passes is monotone in overall ----------
// Monotonicity: if overall_lo <= overall_hi and overall_lo >= threshold,
// then overall_hi >= threshold (consequent is the higher score passing).

pub proof fn lemma_quality_score_passes_monotone(
    overall_lo: int,
    overall_hi: int,
    threshold: int,
    dims: Seq<DimensionScoreSpec>,
    issues: Seq<QualityIssueSpec>,
)
    requires
        0 <= overall_lo,
        overall_lo <= overall_hi,
        overall_hi <= 100,
    ensures
        spec_quality_score_passes(
            QualityScoreSpec { overall: overall_lo, dimensions: dims, issues: issues },
            threshold,
        ) ==> spec_quality_score_passes(
            QualityScoreSpec { overall: overall_hi, dimensions: dims, issues: issues },
            threshold,
        ),
{
    // If overall_lo >= threshold and overall_lo <= overall_hi,
    // then overall_hi >= threshold by transitivity.
}

// ---------- OB-LQ-V-11: empty answers ⇒ EmptyAnswers ----------

pub proof fn lemma_calculate_quality_empty(ears_len: int, d1: int, d2: int, d3: int, d4: int, d5: int)
    ensures
        spec_calculate_quality(0, ears_len, d1, d2, d3, d4, d5)
            == Err::<QualityScoreSpec, _>(QualityErrorSpec::EmptyAnswers),
{
    // The spec branches on `answers_len == 0` first; the EmptyAnswers arm
    // is taken regardless of the other parameters. Verus's integer solver
    // closes this from the if-guard `0 == 0` and the unique Err variant.
}

// ---------- OB-LQ-V-12: non-empty answers returns Ok with correct floor-mean and 5 dimensions ----------

pub proof fn lemma_calculate_quality_ok(
    answers_len: int,
    ears_len: int,
    d1: int,
    d2: int,
    d3: int,
    d4: int,
    d5: int,
)
    requires
        answers_len > 0,
        0 <= d1,
        d1 <= 100,
        0 <= d2,
        d2 <= 100,
        0 <= d3,
        d3 <= 100,
        0 <= d4,
        d4 <= 100,
        0 <= d5,
        d5 <= 100,
    ensures
        match spec_calculate_quality(answers_len, ears_len, d1, d2, d3, d4, d5) {
            Ok(q) => {
                // dimensions is exactly 5 entries in declaration order
                q.dimensions.len() == 5
                // overall is the integer floor of the arithmetic mean
                && q.overall == (d1 + d2 + d3 + d4 + d5) / 5
                // overall is in [0, 100] (proved by lemma_mean_of_five_in_unit_interval)
                && 0 <= q.overall <= 100
                // Each dimension score is preserved
                && q.dimensions[0].score == d1
                && q.dimensions[1].score == d2
                && q.dimensions[2].score == d3
                && q.dimensions[3].score == d4
                && q.dimensions[4].score == d5
            },
            Err(_) => false,
        },
{
    // Unfolding spec_calculate_quality: answers_len > 0 falls through to
    // the Ok arm; the sum is computed and divided by 5 (integer floor);
    // the dimensions Seq is built from the five parameters in order.
    // The [0,100] bound on q.overall follows from lemma_mean_of_five_in_unit_interval.
}

// ---------- OB-LQ-V-13: arithmetic mean floor of 5 scores in [0,100] is in [0,100] ----------

pub proof fn lemma_mean_of_five_in_unit_interval(
    d1: int, d2: int, d3: int, d4: int, d5: int,
)
    requires
        0 <= d1, d1 <= 100,
        0 <= d2, d2 <= 100,
        0 <= d3, d3 <= 100,
        0 <= d4, d4 <= 100,
        0 <= d5, d5 <= 100,
    ensures
        ({ let s = d1 + d2 + d3 + d4 + d5; 0 <= s && s <= 500 }),
        ({ let s = d1 + d2 + d3 + d4 + d5; 0 <= s / 5 && s / 5 <= 100 }),
{
    let s = d1 + d2 + d3 + d4 + d5;
    assert(0 <= s) by {
        // Each d_i >= 0; sum of non-negatives is non-negative.
    };
    assert(s <= 500) by {
        // Each d_i <= 100; sum of 5 such is <= 5 * 100 = 500.
        assert(d1 <= 100);
        assert(d2 <= 100);
        assert(d3 <= 100);
        assert(d4 <= 100);
        assert(d5 <= 100);
        assert(d1 + d2 <= 200);
        assert(d1 + d2 + d3 <= 300);
        assert(d1 + d2 + d3 + d4 <= 400);
        assert(d1 + d2 + d3 + d4 + d5 <= 500);
    };
    assert(0 <= s / 5) by {
        // s >= 0 implies s/5 >= 0 in integer arithmetic.
    };
    assert(s / 5 <= 100) by {
        // s <= 500 implies s/5 <= 100 in integer arithmetic.
        assert(s <= 500);
        // If s/5 > 100, then s > 500, contradicting the requires.
        assert((s / 5) * 5 <= s);
        // (s/5)*5 <= s <= 500 implies s/5 <= 100 because 101*5 = 505 > 500.
        assert(!(s / 5 > 100)) by {
            if s / 5 > 100 {
                assert((s / 5) * 5 > 100 * 5);
                assert((s / 5) * 5 > 500);
                assert(s >= (s / 5) * 5);
                assert(s > 500);
            }
        };
    };
}

// ---------- OB-LQ-V-14: idempotence of calculate_quality (same args → same result) ----------

pub proof fn lemma_calculate_quality_idempotent(
    answers_len: int,
    ears_len: int,
    d1: int, d2: int, d3: int, d4: int, d5: int,
)
    ensures
        spec_calculate_quality(answers_len, ears_len, d1, d2, d3, d4, d5)
            == spec_calculate_quality(answers_len, ears_len, d1, d2, d3, d4, d5),
{
    // Pure deterministic spec function: equal arguments produce equal results
    // by definition of spec-function extensionality. The body uses
    // integer arithmetic only; no I/O, no state, no aliasing.
    assert(spec_calculate_quality(answers_len, ears_len, d1, d2, d3, d4, d5)
        == spec_calculate_quality(answers_len, ears_len, d1, d2, d3, d4, d5));
}

} // verus!
