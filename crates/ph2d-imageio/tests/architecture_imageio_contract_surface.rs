//! Arch-gate: the contracts that *external imageio crates implement* (each
//! `crates/ph2d-imageio-<slug>/`) must stay tiny. If `ImageImporter` /
//! `ImageExporter` / `DecodedImage` / `ExportOpts` / `ColorProfile` /
//! `Error` grow, the change ripples to every format crate and serializes
//! the multi-agent fan-out (the failure mode ADR-0054 was built to avoid).
//!
//! Mirrors `ph2d-editor-core`'s `architecture_tool_contract_surface` and
//! `ph2d-nodegraph`'s `architecture_contract_surface` — same discipline,
//! same rationale.
//!
//! **FROZEN at ADR-0054 W0.T4 (2026-05-26):** the caps below are pinned
//! to the *current* surface (no headroom on enums/fields; 1 slot headroom
//! on the two traits to make a single future method addition possible
//! without an immediate amendment). ANY addition past the cap trips this
//! gate and forces a conscious cap bump + ADR amendment. This is what
//! makes the freeze bite — the W1/W2/W3 fan-out builds against a fixed
//! contract.
//!
//! How to raise a cap: bump the number here, write the ADR-0054
//! amendment justifying it, and Coord-A reviews. A contract change is a
//! rare event.

/// Count `fn ` declarations inside the body of `trait_decl` (up to its
/// first closing `\n}`). Doc-comment lines (`///`) are skipped so prose
/// containing `fn` does not inflate the count. Copy of the helper from
/// `ph2d-editor-core/tests/architecture_tool_contract_surface.rs`.
fn trait_method_count(src: &str, trait_decl: &str) -> usize {
    let start = src
        .find(trait_decl)
        .unwrap_or_else(|| panic!("trait declaration {trait_decl:?} present"));
    let rest = &src[start..];
    let end = rest.find("\n}").expect("trait body closes");
    rest[..end]
        .lines()
        .filter(|l| !l.trim_start().starts_with("///"))
        .filter(|l| l.contains("fn "))
        .count()
}

/// Count variants of an enum starting at `enum_decl`. Variants are lines
/// whose stripped form starts with an uppercase ASCII letter — skips
/// doc-comments (`///`) and blank lines. Copy of the helper from
/// `architecture_tool_contract_surface.rs`.
fn enum_variant_count(src: &str, enum_decl: &str) -> usize {
    let start = src
        .find(enum_decl)
        .unwrap_or_else(|| panic!("enum declaration {enum_decl:?} present"));
    let body_start = src[start..].find('{').expect("opening brace") + start;
    let body_end = src[body_start..].find("\n}").expect("enum closes") + body_start;
    src[body_start..body_end]
        .lines()
        .filter(|line| {
            line.trim_start()
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase())
        })
        .count()
}

/// Count `pub` fields of a struct starting at `struct_decl`. Fields are
/// `pub <name>:` lines inside the body — skips `///` doc-comments and
/// private fields (we don't have any in `ExportOpts`, but the helper
/// gates only the public surface, which is what implementers see).
fn struct_pub_field_count(src: &str, struct_decl: &str) -> usize {
    let start = src
        .find(struct_decl)
        .unwrap_or_else(|| panic!("struct declaration {struct_decl:?} present"));
    let body_start = src[start..].find('{').expect("opening brace") + start;
    let body_end = src[body_start..].find("\n}").expect("struct closes") + body_start;
    src[body_start..body_end]
        .lines()
        .filter(|line| line.trim_start().starts_with("pub "))
        .count()
}

#[test]
fn image_importer_contract_is_capped() {
    let src = include_str!("../src/importer.rs");
    let n = trait_method_count(src, "pub trait ImageImporter");
    assert!(
        n <= 3,
        "ImageImporter has {n} methods; cap is 3. FROZEN at ADR-0054 W0.T4 \
         to the current surface (supports / import) with 1 slot of \
         headroom — every `ph2d-imageio-*` crate implements this, so any \
         growth ripples the whole fan-out. Adding a method is a Coord-A \
         contract change: bump the cap here + write the ADR-0054 amendment."
    );
}

#[test]
fn image_exporter_contract_is_capped() {
    let src = include_str!("../src/exporter.rs");
    let n = trait_method_count(src, "pub trait ImageExporter");
    assert!(
        n <= 3,
        "ImageExporter has {n} methods; cap is 3. FROZEN at ADR-0054 W0.T4 \
         to the current surface (supports_format / export) with 1 slot of \
         headroom. Adding a method is a Coord-A contract change: bump the \
         cap here + write the ADR-0054 amendment."
    );
}

#[test]
fn decoded_image_variant_count_is_capped() {
    let src = include_str!("../src/decoded.rs");
    let n = enum_variant_count(src, "pub enum DecodedImage {");
    assert!(
        n <= 5,
        "DecodedImage has {n} variants; cap is 5. FROZEN at ADR-0054 W0.T4 \
         to the current surface (Flat / FlatHdr / Layered / Animated / \
         Vector). Every importer constructs one of these, every exporter \
         matches on the full set. Adding a variant is a Coord-A contract \
         change: bump the cap here + write the ADR-0054 amendment."
    );
}

#[test]
fn color_profile_variant_count_is_capped() {
    let src = include_str!("../src/color.rs");
    let n = enum_variant_count(src, "pub enum ColorProfile {");
    assert!(
        n <= 8,
        "ColorProfile has {n} variants; cap is 8. FROZEN at ADR-0054 W0.T4 \
         to the current surface (Srgb / DisplayP3 / AdobeRgb / ProPhoto / \
         LinearRec709 / LinearRec2020 / Custom / Unknown). Adding a profile \
         tag ripples to every importer that detects + every exporter that \
         emits color-managed bytes. Coord-A contract change: bump + amendment."
    );
}

#[test]
fn error_variant_count_is_capped() {
    let src = include_str!("../src/error.rs");
    let n = enum_variant_count(src, "pub enum Error {");
    assert!(
        n <= 8,
        "Error has {n} variants; cap is 8. FROZEN at ADR-0054 W0.T4 to the \
         current surface (Decode / Encode / Unsupported / Truncated / \
         IccCorrupted / MissingLayer / HdrUnsupported / Custom). Per-format \
         detail goes in the inner `String` payload — prefer that to a new \
         variant. Coord-A contract change: bump + amendment."
    );
}

#[test]
fn export_opts_field_count_is_capped() {
    let src = include_str!("../src/opts.rs");
    let n = struct_pub_field_count(src, "pub struct ExportOpts {");
    assert!(
        n <= 6,
        "ExportOpts has {n} pub fields; cap is 6. FROZEN at ADR-0054 W0.T4 \
         to the current surface (format / quality / color_profile / \
         preserve_layers / tone_map / metadata). Every exporter reads this \
         struct; a new field is a contract change every implementer must \
         honour. Coord-A contract change: bump + amendment."
    );
}

// `ImportOpts` is intentionally *not* capped here — it's caller-side
// preferences that importers may ignore. New ImportOpts fields are
// additive and don't break existing importers. We document this asymmetry
// in ADR-0054 §2.1 so it doesn't read as an oversight.
