//! Vector Module contract-surface arch-gate.
//!
//! Audits the source of `ph2d-vector-doc` (and, when they nasce, the
//! downstream `ph2d-node-vector-*` / `ph2d-tool-vector-*` crates) for
//! the caps congelados in ADR-0056 §2.3:
//!
//! | Symbol               | Cap | Source |
//! |----------------------|-----|--------|
//! | `VectorOp` variants  | ≤ 16 | ADR-0056 §2.3 + §2.8 |
//! | `Vertex` fields      | ≤ 5  | ADR-0056 §2.3 |
//! | `Segment` fields     | ≤ 6  | ADR-0056 §2.3 |
//! | `Region` fields      | ≤ 5  | ADR-0056 §2.3 |
//! | `VertexKind` variants| = 4 (FROZEN) | ADR-0056 §2.3 |
//! | `RepresentationMode` variants | = 3 (FROZEN) | ADR-0056 §2.3 |
//! | `WindingRule` variants | = 2 (FROZEN) | ADR-0056 §2.3 |
//!
//! ## Política de vacuous-pass
//!
//! Mirrors `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs`.
//! When a downstream crate doesn't exist yet, helpers return `None` and
//! tests pass vacuously. As crates land (W2+ fan-out for nodes/tools),
//! arch-gates activate without further edits here.

use std::path::PathBuf;
use walkdir::WalkDir;

// ----------------------------------------------------------------------------
// Helpers — textual gate over crate sources.
// ----------------------------------------------------------------------------

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir has parent")
        .parent()
        .expect("crates/ has parent")
        .to_path_buf()
}

/// Concatenate every `src/**/*.rs` file of `crate_name`.
fn crate_source(crate_name: &str) -> String {
    let dir = workspace_root().join("crates").join(crate_name).join("src");
    if !dir.exists() {
        return String::new();
    }
    let mut out = String::new();
    for entry in WalkDir::new(&dir).into_iter().flatten() {
        if entry.path().extension().is_some_and(|e| e == "rs")
            && let Ok(s) = std::fs::read_to_string(entry.path())
        {
            out.push('\n');
            out.push_str(&s);
        }
    }
    out
}

/// Locate the `{ ... }` body of `pub <kind> <name>` (enum or struct).
/// Guards against tuple-struct (`pub struct X(...)`) by returning `None`
/// when an opening `(` precedes the first `{` after the declaration head.
fn find_decl_body<'a>(src: &'a str, kind: &str, name: &str) -> Option<&'a str> {
    let prefix = format!("pub {kind} {name}");
    let start = src.find(&prefix)?;
    let tail = &src[start + prefix.len()..];
    let paren_pos = tail.find('(');
    let brace_pos = tail.find('{');
    match (paren_pos, brace_pos) {
        (Some(p), Some(b)) if p < b => return None,
        (Some(_), None) => return None,
        (None, None) => return None,
        _ => {}
    }
    let body_start = start + prefix.len() + brace_pos.unwrap();
    let body_start_inner = body_start + 1;
    let bytes = src.as_bytes();
    let mut depth = 1usize;
    let mut i = body_start_inner;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&src[body_start_inner..i]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Count enum variants in `pub enum <name>`. `None` = enum absent
/// (vacuous pass).
fn count_enum_variants(crate_name: &str, enum_name: &str) -> Option<usize> {
    let src = crate_source(crate_name);
    let body = find_decl_body(&src, "enum", enum_name)?;
    let count = body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("///"))
        .filter(|l| l.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
        .count();
    Some(count)
}

/// Count struct fields (`pub <name>: ...`). `None` = struct absent.
fn count_struct_fields(crate_name: &str, struct_name: &str) -> Option<usize> {
    let src = crate_source(crate_name);
    let body = find_decl_body(&src, "struct", struct_name)?;
    let count = body
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("pub ") && l.contains(':'))
        .count();
    Some(count)
}

/// Vacuous-pass `assert ≤ max`: `None` (symbol absent) is allowed.
///
/// Use for **downstream / cross-crate** symbols that legitimately may
/// not exist yet (W2+ fan-out crates). Currently unused (all in-tree
/// symbols moved to the strict variant per audit M9); kept for the next
/// downstream gate landing, mirroring `assert_exact`.
#[allow(dead_code)]
#[track_caller]
fn assert_capped(actual: Option<usize>, max: usize, ctx: &str) {
    if let Some(n) = actual {
        assert!(n <= max, "[{ctx}] count {n} exceeds cap {max}");
    }
}

/// Vacuous-pass `assert == expected`. Kept for symmetry with
/// `assert_capped` — downstream tests may want to assert exact counts
/// for cross-crate FROZEN enums. None currently use it but the helper
/// stays for the next gate landing.
#[allow(dead_code)]
#[track_caller]
fn assert_exact(actual: Option<usize>, expected: usize, ctx: &str) {
    if let Some(n) = actual {
        assert_eq!(n, expected, "[{ctx}] count {n} != expected {expected}");
    }
}

/// **Strict** `assert ≤ max`: `None` (symbol absent) is a FAIL.
///
/// Use for **home-crate** (this crate's own) symbols that MUST exist —
/// if `VectorOp` gets renamed to `VectorOperation` without updating the
/// gate, this catches it instead of passing silently (R1 audit
/// Lens-D HIGH-D6 mitigation).
#[track_caller]
fn assert_capped_strict(actual: Option<usize>, max: usize, ctx: &str) {
    match actual {
        Some(n) => assert!(n <= max, "[{ctx}] count {n} exceeds cap {max}"),
        None => panic!("[{ctx}] required symbol NOT FOUND — refactor likely renamed it"),
    }
}

/// **Strict** `assert == expected`: `None` is a FAIL.
#[track_caller]
fn assert_exact_strict(actual: Option<usize>, expected: usize, ctx: &str) {
    match actual {
        Some(n) => assert_eq!(n, expected, "[{ctx}] count {n} != expected {expected}"),
        None => panic!("[{ctx}] required symbol NOT FOUND — refactor likely renamed it"),
    }
}

// ----------------------------------------------------------------------------
// ADR-0056 §2.3 — VectorOp / Vertex / Segment / Region / FROZEN enums.
// ----------------------------------------------------------------------------

const CRATE: &str = "ph2d-vector-doc";

#[test]
fn vector_op_variant_count_is_capped_at_16() {
    assert_capped_strict(
        count_enum_variants(CRATE, "VectorOp"),
        16,
        "VectorOp (ADR-0056 §2.3 + §2.8)",
    );
}

#[test]
fn vertex_field_count_is_capped_at_5() {
    assert_capped_strict(
        count_struct_fields(CRATE, "Vertex"),
        5,
        "Vertex (ADR-0056 §2.3)",
    );
}

#[test]
fn segment_field_count_is_capped_at_6() {
    assert_capped_strict(
        count_struct_fields(CRATE, "Segment"),
        6,
        "Segment (ADR-0056 §2.3)",
    );
}

#[test]
fn region_field_count_is_capped_at_5() {
    assert_capped_strict(
        count_struct_fields(CRATE, "Region"),
        5,
        "Region (ADR-0056 §2.3)",
    );
}

#[test]
fn vertex_kind_is_frozen_at_4_variants() {
    assert_exact_strict(
        count_enum_variants(CRATE, "VertexKind"),
        4,
        "VertexKind FROZEN (ADR-0056 §2.3)",
    );
}

#[test]
fn representation_mode_is_frozen_at_3_variants() {
    assert_exact_strict(
        count_enum_variants(CRATE, "RepresentationMode"),
        3,
        "RepresentationMode FROZEN (ADR-0056 §2.3)",
    );
}

#[test]
fn winding_rule_is_frozen_at_2_variants() {
    assert_exact_strict(
        count_enum_variants(CRATE, "WindingRule"),
        2,
        "WindingRule FROZEN (ADR-0056 §2.3)",
    );
}

// ----------------------------------------------------------------------------
// ADR-0067 §2.6 — BrushEngine trait method cap + StampSpec field cap +
// total dep count cap. Counted textually from the brush-traits crate
// source (`pub fn` for methods, `pub <field>:` for StampSpec fields).
// ----------------------------------------------------------------------------

const BRUSH_TRAITS_CRATE: &str = "ph2d-brush-traits";

#[test]
fn brush_engine_method_count_is_capped_at_3() {
    // Count `fn <ident>(` inside the BrushEngine trait body. Default
    // impls + signatures both count toward the cap.
    let src = crate_source(BRUSH_TRAITS_CRATE);
    if src.is_empty() {
        return;
    }
    let Some(body) = find_decl_body(&src, "trait", "BrushEngine") else {
        return;
    };
    let count = body
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("//") && !l.starts_with("///"))
        .filter(|l| l.starts_with("fn ") || l.contains(" fn "))
        .count();
    assert!(
        count <= 3,
        "[BrushEngine method count (ADR-0067 §2.6)] {count} exceeds cap 3"
    );
}

#[test]
fn stamp_spec_field_count_is_capped_at_7() {
    // Strict: StampSpec is a W1 foundation symbol that exists today — a
    // rename must FAIL the gate, not pass vacuously (audit M9).
    assert_capped_strict(
        count_struct_fields(BRUSH_TRAITS_CRATE, "StampSpec"),
        7,
        "StampSpec (ADR-0067 §2.6)",
    );
}

// ----------------------------------------------------------------------------
// ADR-0056 §2.5 — AnimValue typed enum cap.
// ----------------------------------------------------------------------------

const VECTOR_TRAITS_CRATE: &str = "ph2d-vector-traits";

#[test]
fn anim_value_variant_count_is_capped_at_6() {
    // Strict: AnimValue is a W1 foundation symbol that exists today
    // (audit M9).
    assert_capped_strict(
        count_enum_variants(VECTOR_TRAITS_CRATE, "AnimValue"),
        6,
        "AnimValue (ADR-0056 §2.5)",
    );
}

// ----------------------------------------------------------------------------
// ADR-0056 §2.2 — SmallVec inline budgets (ABI-relevant: postcard layout +
// per-frame alloc behavior). The field/variant-count gates above do NOT catch
// a change to the inline size, so assert the exact declarations textually
// (audit M1).
// ----------------------------------------------------------------------------

/// `true` if any `src/**/*.rs` of `crate_name` contains `needle`.
fn crate_source_contains(crate_name: &str, needle: &str) -> bool {
    crate_source(crate_name).contains(needle)
}

#[track_caller]
fn assert_inline_decl(needle: &str) {
    assert!(
        crate_source_contains(CRATE, needle),
        "[ADR-0056 §2.2 inline budget] declaration `{needle}` not found in {CRATE} \
         — a refactor changed an inline SmallVec size (ABI/alloc drift)"
    );
}

#[test]
fn smallvec_inline_budgets_match_adr_0056() {
    assert_inline_decl("SmallVec<[Vertex; 32]>");
    assert_inline_decl("SmallVec<[Segment; 64]>");
    assert_inline_decl("SmallVec<[Region; 8]>");
    assert_inline_decl("SmallVec<[SegmentRef; 16]>");
}

// ----------------------------------------------------------------------------
// W0-ratified generation-time security sanitizers (audit H4). These caps
// were declared in CLAUDE.md but absent from code; assert presence + value so
// the contract is enforced, not verbal.
// ----------------------------------------------------------------------------

#[test]
fn w0_generation_security_caps_present_with_ratified_values() {
    assert_inline_decl("pub const MAX_VERTICES_PER_LLM_GEN: usize = 1_000;");
    assert_inline_decl("pub const MAX_POLYGON_SIDES: u32 = 128;");
    assert_inline_decl("pub const MAX_SPIRAL_TURNS: u32 = 64;");
}
