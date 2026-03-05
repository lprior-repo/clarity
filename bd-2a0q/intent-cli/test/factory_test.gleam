import gleam/list
import gleeunit/should
import intent/spec_builder

/// CUPID-DRIVEN: Spec builder MUST compose via list operations
///
/// DESIGN PRESSURE: Can you build a Spec with 50 behaviors without loops/mutation?
///
/// Forces:
/// - C: Composability - list.range |> list.map builds behaviors
/// - U: Single purpose - One builder fn per type (Behavior, Feature, Spec)
/// - P: Purity - Deterministic generation from index
/// - I: Idiomatic - Pipelines, no imperative code
/// - D: Domain - Builder functions express spec construction
///
/// 30-LINE RULE: Implementer needs ~20 lines for pure builder in src/intent/spec_builder.gleam
///
/// DISCOMFORT: "How do I build N behaviors without for-loops?"
/// Answer forces functional thinking: list.range |> list.map(builder_fn)
pub fn spec_with_many_behaviors_composes_functionally_test() {
  // Must exist: src/intent/spec_builder.gleam with create_test_spec
  let spec = spec_builder.create_test_spec(50)

  // Verify it built 1 feature
  list.length(spec.features)
  |> should.equal(1)

  // Verify the feature has 50 behaviors
  let assert [feature] = spec.features
  list.length(feature.behaviors)
  |> should.equal(50)

  // Verify purity: same input = same output (deterministic)
  let spec2 = spec_builder.create_test_spec(50)
  spec.name
  |> should.equal(spec2.name)

  let assert [feature2] = spec2.features
  list.length(feature2.behaviors)
  |> should.equal(50)
}

/// CUPID-DRIVEN: Regex cache MUST avoid recompilation via ETS
///
/// DESIGN PRESSURE: Checking 1000 emails with SAME pattern must compile ONCE
///
/// SKIPPED: This test depends on v2.0 HTTP types (Request, Response, Check)
/// and http_client module which were removed in v3.0
pub fn regex_cache_avoids_recompilation_with_ets_test_skip() {
  // Test skipped - depends on removed modules
  True |> should.be_true
}
