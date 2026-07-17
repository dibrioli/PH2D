//! [`TextSystem`] — owns the long-lived parley contexts.
//!
//! Two parley state machines live here:
//!   - `FontContext` holds the loaded fonts + the system fallback
//!     chain. Construction enumerates installed system fonts (slow
//!     — 50-200 ms), so callers create ONE per app and reuse it.
//!   - `LayoutContext` holds reusable allocation buffers used during
//!     layout. Cheap to construct, but pooling avoids per-frame
//!     allocator churn.
//!
//! [`TextSystem::layout`] is the convenience entry: build a layout
//! for a `&str` at a given size with the default font stack (system
//! sans-serif + emoji fallback).

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use parley::{
    Alignment, FontContext, FontSettings, FontStack, FontVariation, FontWeight, Layout,
    LayoutContext, StyleProperty,
    fontique::{Blob, Collection, CollectionOptions, FontInfoOverride, SourceCache},
    swash::tag_from_bytes,
};

/// OpenType axis tag for "Optical Size" (`opsz`). Inter 4.x supports
/// the range 14–32: at smaller `opsz` Inter substitutes a "Text" cut
/// (heavier strokes, more open counters) so glyphs hold up under
/// GPU rasterization at 12-14 px; at larger `opsz` it shifts toward
/// a "Display" cut (tighter spacing, finer details). Values outside
/// the axis range are clamped by skrifa.
const OPSZ_TAG: u32 = tag_from_bytes(b"opsz");

/// OpenType axis tag for "Weight" (`wght`). MUST be pushed alongside
/// `opsz` in the `FontVariations` array — `StyleProperty::FontVariations`
/// REPLACES the implicit axis settings that parley would otherwise
/// derive from `StyleProperty::FontWeight`. Without an explicit `wght`
/// entry, variable fonts (Inter Variable) fall back to the font's
/// default weight (~Regular 400) regardless of the FontWeight selection,
/// making FontWeight bumps invisible — exactly the "Crisp Heavy looks
/// identical to Crisp" symptom seen on 2026-05-25.
const WGHT_TAG: u32 = tag_from_bytes(b"wght");

/// Inter Variable (v4.0, SIL OFL) — bundled so chrome text rasterizes
/// to the same glyphs everywhere, independent of installed system fonts.
/// Inter was designed for screen rendering without LCD subpixel AA,
/// which matches Vello's glyph pipeline (vs. system fonts like SF that
/// are tuned for CoreText's subpixel rendering and look soft here).
/// Source: <https://github.com/rsms/inter/releases/tag/v4.0> (LICENSE.txt
/// in this directory).
const INTER_VARIABLE_TTF: &[u8] = include_bytes!("../fonts/InterVariable.ttf");

/// Os bytes da fonte embutida (InterVariable, OFL). Exposto para quem precisa dos
/// contornos crus dos glyphs — ex. texto VETORIAL (skrifa → `VecPath`), que não passa
/// pelo pipeline parley/vello de UI.
#[must_use]
pub fn inter_variable_ttf() -> &'static [u8] {
    INTER_VARIABLE_TTF
}

/// Family name registered when bundled Inter loads successfully.
/// Falls back to `sans-serif` if registration fails (corrupted bytes,
/// future fontique breaking change, etc.) so we never panic at startup.
const INTER_FAMILY: &str = "InterVariable";

/// Cache key for a shaped layout. The layout is fully determined by
/// text, size, wrap width, weight, and letter-spacing (the font stack
/// and opsz variation are fixed per `TextSystem` / derived from
/// `font_size`), so these fields are an exact identity — no collision
/// risk (unlike a hashed key). f32s are stored as raw bits so the key
/// is `Eq + Ord` (workspace clippy bans `HashMap` per ADR-0022, so this
/// is a `BTreeMap` key).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LayoutCacheKey {
    text: String,
    font_size_bits: u32,
    max_width_bits: u32,
    weight_bits: u32,
    letter_spacing_px_bits: u32,
}

/// Cap on cached shaped layouts. Steady-state UI text (panel labels, row
/// names, counts) is well under this; ever-changing text (a focused
/// NumberInput/TextInput buffer) generates unique keys, so the cache is
/// cleared wholesale once it overflows to bound memory. // LITERAL-OK: cache budget
const LAYOUT_CACHE_CAP: usize = 1024;

// Thread-local global text-rendering strategy. Lives here (not in
// `ph2d-editor-core::paint`) so `TextSystem::prefix_width` can read it
// internally — that fixes the caret-position bug where measurements
// (which set caret X) used Medium 500 while glyphs rendered with the
// boosted weight (ExtraBold 800) under CrispHeavy/Plus. Caret would
// drift into the right-hand glyphs because the measurement underestimated
// the advance.
//
// The thread-local is OWNED here; `ph2d-editor-core::paint` re-exposes
// it via `set_text_rendering` / `text_rendering` thin wrappers so the
// public API surface in that crate stays unchanged.
thread_local! {
    static ACTIVE_TEXT_RENDERING: std::cell::Cell<ph2d_tokens::TextRendering> =
        const { std::cell::Cell::new(ph2d_tokens::TextRendering::Default) };
}

/// Set the active text-rendering strategy for the current thread.
/// Called by the shell once per frame from `paint::set_text_rendering`,
/// which delegates here. Read by `TextSystem::prefix_width` so
/// measurements match what `paint_text*` will render.
pub fn set_active_text_rendering(mode: ph2d_tokens::TextRendering) {
    ACTIVE_TEXT_RENDERING.with(|c| c.set(mode));
}

/// Read the active text-rendering strategy. Defaults to
/// `TextRendering::Default`.
pub fn active_text_rendering() -> ph2d_tokens::TextRendering {
    ACTIVE_TEXT_RENDERING.with(|c| c.get())
}

pub struct TextSystem {
    font_context: FontContext,
    layout_context: LayoutContext<()>,
    /// Resolved primary stack — "InterVariable, sans-serif" when the
    /// bundled font registered, plain "sans-serif" otherwise.
    primary_stack: String,
    /// Shaped-layout cache (perf): `layout_with_weight` rebuilt a parley
    /// `Layout` (shape + line-break + align) from scratch on EVERY call,
    /// every frame — the dominant per-frame cost under the continuous
    /// `ControlFlow::Poll` redraw (a Hierarchy panel re-shapes every row
    /// each frame). `parley::Layout` is `Clone`, and layout is geometry
    /// independent of theme/colour, so a hit clones the cached layout and
    /// skips shaping entirely. Invalidation: none needed — `primary_stack`
    /// is fixed per instance; if a future path mutates the font set it
    /// must `layout_cache.clear()`.
    layout_cache: BTreeMap<LayoutCacheKey, Layout<()>>,
}

impl TextSystem {
    /// Build a new TextSystem. Loads system fonts (50-200 ms cold) and
    /// registers bundled Inter Variable.
    pub fn new() -> Self {
        let mut font_context = FontContext::new();
        let primary_stack = match register_inter(&mut font_context) {
            Some(name) => format!("{name}, sans-serif"),
            None => "sans-serif".to_string(),
        };
        Self {
            font_context,
            layout_context: LayoutContext::new(),
            primary_stack,
            layout_cache: BTreeMap::new(),
        }
    }

    /// Build a TextSystem that skips the system-font scan (`fontique::CollectionOptions::system_fonts = false`).
    /// Only bundled Inter Variable is available — sufficient for any
    /// Latin layout but no CJK/emoji fallback.
    ///
    /// Why: `FontContext::new()` enumerates installed system fonts via
    /// CoreText/Fontconfig/DirectWrite, which on some macOS machines
    /// takes 25-70 s cold per call. Test suites build ~30 TextSystems
    /// each, multiplying that cost across the workspace. Tests that
    /// only need ASCII chrome text can use this faster path.
    pub fn without_system_fonts() -> Self {
        let collection = Collection::new(CollectionOptions {
            shared: false,
            system_fonts: false,
        });
        let mut font_context = FontContext {
            collection,
            source_cache: SourceCache::default(),
        };
        // Force the registered family name. Without a system-font
        // fallback chain, parley resolves the FontStack by exact
        // family_by_name() lookup — so the in-collection name MUST
        // match the string we put in `primary_stack`. The TTF's own
        // name table reports "Inter" (not "InterVariable"), so without
        // this override `family_by_name("InterVariable")` returns None
        // and tests get zero glyphs.
        let override_info = FontInfoOverride {
            family_name: Some(INTER_FAMILY),
            ..Default::default()
        };
        let blob = Blob::new(Arc::new(INTER_VARIABLE_TTF));
        let registered = font_context
            .collection
            .register_fonts(blob, Some(override_info));
        let primary_stack = if registered.is_empty() {
            "sans-serif".to_string()
        } else {
            INTER_FAMILY.to_string()
        };
        Self {
            font_context,
            layout_context: LayoutContext::new(),
            primary_stack,
            layout_cache: BTreeMap::new(),
        }
    }

    pub fn font_context(&mut self) -> &mut FontContext {
        &mut self.font_context
    }

    pub fn layout_context(&mut self) -> &mut LayoutContext<()> {
        &mut self.layout_context
    }

    /// Build a single-line layout for `text` at `font_size` (in
    /// device-independent pixels). Uses Inter Variable @ Medium (500)
    /// — see [`Self::layout_with_weight`] for the weight rationale.
    ///
    /// The `max_width` parameter is the layout's wrap budget; pass
    /// `f32::INFINITY` for single-line "as wide as needed".
    pub fn layout(&mut self, text: &str, font_size: f32, max_width: f32) -> Layout<()> {
        self.layout_with_weight(text, font_size, max_width, FontWeight::MEDIUM)
    }

    /// Like [`Self::layout_with_weight`] but applies the
    /// [`ph2d_tokens::TextRendering`] strategy: in `Default` it is
    /// identical to `layout_with_weight(text, size, max_width, weight)`;
    /// in Crisp/CrispHeavy/CrispHeavyPlus it bumps the nominal weight
    /// by [`ph2d_tokens::crisp_weight_boost_for`] before shaping AND
    /// applies the preset's `letter_spacing_em_dense` for body sizes.
    /// Both affect shaping (advance widths) — must enter BEFORE the
    /// parley layout, not after.
    pub fn layout_for_rendering(
        &mut self,
        text: &str,
        font_size: f32,
        max_width: f32,
        weight_nominal: FontWeight,
        rendering: ph2d_tokens::TextRendering,
    ) -> Layout<()> {
        let effective_weight_val = effective_weight(weight_nominal, font_size, rendering);
        let letter_spacing_px = effective_letter_spacing_px(font_size, rendering);
        self.layout_inner(
            text,
            font_size,
            max_width,
            effective_weight_val,
            letter_spacing_px,
        )
    }

    /// Like [`Self::layout`] but with an explicit `weight`. Use for
    /// titles (SemiBold 600) — diagonals in glyphs like "y" / "k" /
    /// "v" don't hint to the pixel grid cleanly at small sizes without
    /// LCD subpixel AA, so they read softer than vertical-stem
    /// letters. SemiBold adds ~25 % more pen weight on those
    /// diagonals, closing the perceptual gap. Linear / Notion use
    /// this exact split (body 500 / titles 600).
    pub fn layout_with_weight(
        &mut self,
        text: &str,
        font_size: f32,
        max_width: f32,
        weight: FontWeight,
    ) -> Layout<()> {
        self.layout_inner(text, font_size, max_width, weight, 0.0)
    }

    /// Private builder. Single source of truth for the parley layout
    /// path + cache. Other entry points (`layout`, `layout_with_weight`,
    /// `layout_for_rendering`) are thin wrappers.
    fn layout_inner(
        &mut self,
        text: &str,
        font_size: f32,
        max_width: f32,
        weight: FontWeight,
        letter_spacing_px: f32,
    ) -> Layout<()> {
        let key = LayoutCacheKey {
            text: text.to_string(),
            font_size_bits: font_size.to_bits(),
            max_width_bits: max_width.to_bits(),
            weight_bits: weight.value().to_bits(),
            letter_spacing_px_bits: letter_spacing_px.to_bits(),
        };
        if let Some(cached) = self.layout_cache.get(&key) {
            // Hit: clone the shaped layout (skips shape + line-break +
            // align — the dominant per-frame cost).
            return cached.clone();
        }
        let mut builder = self.layout_context.ranged_builder(
            &mut self.font_context,
            text,
            1.0,  // device pixel ratio
            true, // quantize
        );
        builder.push_default(StyleProperty::FontStack(FontStack::Source(Cow::Borrowed(
            self.primary_stack.as_str(),
        ))));
        builder.push_default(StyleProperty::FontSize(font_size));
        // Inter at Regular 400 looks washed out at small UI sizes
        // without LCD subpixel AA — the strokes are too thin to
        // hold opacity in partial-coverage edge pixels, so glyphs
        // read as "soft gray haze" instead of "solid letterform".
        // Default body weight (500) adds ~12 % pen mass; titles
        // bump to 600 for crisp diagonals. The variable axis lets
        // skrifa interpolate exactly without a separate font file.
        builder.push_default(StyleProperty::FontWeight(weight));
        // Optional letter-spacing (in DIPs / px). 0.0 = no adjustment.
        // CrispHeavyPlus uses negative values at small sizes to tighten
        // the density that ExtraBold naturally opens.
        if letter_spacing_px != 0.0 {
            builder.push_default(StyleProperty::LetterSpacing(letter_spacing_px));
        }
        // Drive Inter's optical-size + weight axes explicitly. The
        // opsz axis maps to render size (Text cut at 12-14 px → Display
        // cut at 20+ px); `font_size` outside [14, 32] is clamped by
        // skrifa. The wght axis MUST be pushed here — see WGHT_TAG
        // docstring — otherwise `StyleProperty::FontWeight` selections
        // are silently ignored on variable fonts.
        let variations = [
            FontVariation {
                tag: OPSZ_TAG,
                value: font_size,
            },
            FontVariation {
                tag: WGHT_TAG,
                value: weight.value(),
            },
        ];
        builder.push_default(StyleProperty::FontVariations(FontSettings::List(
            Cow::Borrowed(&variations),
        )));
        let mut layout: Layout<()> = builder.build(text);
        layout.break_all_lines(Some(max_width));
        layout.align(
            Some(max_width),
            Alignment::Start,
            parley::AlignmentOptions::default(),
        );
        if self.layout_cache.len() >= LAYOUT_CACHE_CAP {
            self.layout_cache.clear();
        }
        self.layout_cache.insert(key, layout.clone());
        layout
    }

    /// Measure the width of `prefix` for caret / selection
    /// positioning. Unlike `layout(prefix).width()`, this includes
    /// trailing whitespace — parley's line layout trims trailing
    /// spaces from the measured width, which made the caret appear
    /// stuck after the user pressed SPACE (the space was inserted
    /// into the buffer but the visual caret didn't advance).
    ///
    /// Strategy: layout `prefix + SENTINEL`, then subtract the
    /// sentinel's width measured in isolation. The sentinel is a
    /// non-whitespace glyph so parley never trims it.
    ///
    /// **Uses the active `TextRendering` strategy** (read from the
    /// `active_text_rendering()` thread-local) — without that, body
    /// text caret in CrispHeavy/CrispHeavyPlus drifts INTO the next
    /// glyph because measurements use Medium 500 while glyphs render
    /// with the boosted weight. The fix is invisible to callers (same
    /// signature, no new params).
    pub fn prefix_width(&mut self, prefix: &str, font_size: f32) -> f32 {
        if prefix.is_empty() {
            return 0.0;
        }
        let rendering = active_text_rendering();
        // Fast path: prefix has no trailing space → the plain layout
        // (now rendering-aware) already returns the correct width.
        if !prefix.ends_with(' ') && !prefix.ends_with('\t') {
            return self
                .layout_for_rendering(
                    prefix,
                    font_size,
                    f32::INFINITY,
                    FontWeight::MEDIUM,
                    rendering,
                )
                .width();
        }
        const SENTINEL: &str = "|";
        let mut combined = String::with_capacity(prefix.len() + SENTINEL.len());
        combined.push_str(prefix);
        combined.push_str(SENTINEL);
        let w_with = self
            .layout_for_rendering(
                &combined,
                font_size,
                f32::INFINITY,
                FontWeight::MEDIUM,
                rendering,
            )
            .width();
        let w_sentinel = self
            .layout_for_rendering(
                SENTINEL,
                font_size,
                f32::INFINITY,
                FontWeight::MEDIUM,
                rendering,
            )
            .width();
        (w_with - w_sentinel).max(0.0)
    }
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Register the bundled Inter Variable into `font_context.collection`.
/// Returns the family name to reference in `FontStack` on success,
/// `None` if registration produced no usable family (e.g. fontique
/// rejected the bytes). Callers fall back to `sans-serif`.
fn register_inter(font_context: &mut FontContext) -> Option<&'static str> {
    let blob = Blob::new(Arc::new(INTER_VARIABLE_TTF));
    let registered = font_context.collection.register_fonts(blob, None);
    if registered.is_empty() {
        return None;
    }
    Some(INTER_FAMILY)
}

/// Min/max valid CSS font weights (parley/skrifa clamp to this range
/// downstream; we clamp here for predictable test assertions and to
/// keep the `FontWeight::new(...)` value within the spec-defined band).
const WEIGHT_MIN: f32 = 100.0;
const WEIGHT_MAX: f32 = 900.0;

/// Compute the effective `FontWeight` given the nominal weight, font
/// size, and rendering strategy. Reads the preset's
/// `TextRenderingParams` once and dispatches through
/// [`ph2d_tokens::crisp_weight_boost_for`]. `Default` returns the
/// nominal unchanged because its params are all-zero. Result is
/// clamped to `[100, 900]`.
fn effective_weight(
    nominal: FontWeight,
    font_size: f32,
    rendering: ph2d_tokens::TextRendering,
) -> FontWeight {
    let boost = ph2d_tokens::crisp_weight_boost_for(rendering.params(), font_size);
    if boost == 0 {
        return nominal;
    }
    let raw = nominal.value() + boost as f32;
    FontWeight::new(raw.clamp(WEIGHT_MIN, WEIGHT_MAX))
}

/// Compute the effective letter-spacing (in DIPs / px) for the given
/// font size + rendering strategy. The preset declares
/// `letter_spacing_em_dense` (in ems) which is applied ONLY at body
/// sizes (≤16 px) — large sizes don't need the tightening. Convert
/// em→px by multiplying by `font_size`. Default and CrispHeavy both
/// return 0.0 (no spacing change); CrispHeavyPlus returns a small
/// negative at body sizes.
const LETTER_SPACING_BODY_MAX_PX: f32 = 16.0;
fn effective_letter_spacing_px(font_size: f32, rendering: ph2d_tokens::TextRendering) -> f32 {
    let p = rendering.params();
    if p.letter_spacing_em_dense == 0.0 || font_size > LETTER_SPACING_BODY_MAX_PX {
        return 0.0;
    }
    p.letter_spacing_em_dense * font_size
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "Hello PH2D" at 16 px must produce at least one glyph run.
    /// This is the M11 ASCII gate — ensures parley + system fonts
    /// + our wrapper compose into something usable.
    #[test]
    fn ascii_hello_ph2d_produces_glyphs() {
        let mut sys = TextSystem::without_system_fonts();
        let layout = sys.layout("Hello PH2D", 16.0, f32::INFINITY);
        let glyph_count: usize = layout
            .lines()
            .flat_map(|l| l.items())
            .filter_map(|item| match item {
                parley::PositionedLayoutItem::GlyphRun(run) => Some(run.glyphs().count()),
                _ => None,
            })
            .sum();
        assert!(
            glyph_count >= 10,
            "expected ≥ 10 glyphs for 'Hello PH2D', got {glyph_count}"
        );
    }

    /// Empty string is a valid layout: zero glyphs but no panic.
    /// Important because UIs commonly relayout with an empty string
    /// (placeholder fields, deleted text).
    #[test]
    fn empty_string_layout_is_safe() {
        let mut sys = TextSystem::without_system_fonts();
        let layout = sys.layout("", 16.0, 100.0);
        let glyph_count: usize = layout
            .lines()
            .flat_map(|l| l.items())
            .filter_map(|item| match item {
                parley::PositionedLayoutItem::GlyphRun(run) => Some(run.glyphs().count()),
                _ => None,
            })
            .sum();
        assert_eq!(glyph_count, 0);
    }

    /// Multi-script line: ASCII + CJK + emoji. Per the M11 plan,
    /// font fallback for CJK + emoji color is the visual gate.
    /// We can't render here (no GPU) but we CAN verify parley
    /// produces non-zero glyphs across all three scripts — the
    /// system fallback chain found something for each.
    ///
    /// Note: this test is `#[ignore]` because CI runners (and the
    /// dev's machine) may lack one of the three font families,
    /// and a missing CJK font would produce a "tofu" run that
    /// still has glyphs. We exercise the codepath; real rendering
    /// validation happens in the editor stub (M12).
    #[test]
    #[ignore = "depends on system fonts; manual smoke test only"]
    fn cjk_and_emoji_round_trip() {
        let mut sys = TextSystem::new();
        let layout = sys.layout("Hello 世界 🎮", 16.0, f32::INFINITY);
        let glyph_count: usize = layout
            .lines()
            .flat_map(|l| l.items())
            .filter_map(|item| match item {
                parley::PositionedLayoutItem::GlyphRun(run) => Some(run.glyphs().count()),
                _ => None,
            })
            .sum();
        assert!(glyph_count >= 6);
    }

    /// Layout dimensions are reasonable: 16 px text on a wide line
    /// produces a width > 0 and height ~ font_size.
    #[test]
    fn layout_dimensions_are_sane() {
        let mut sys = TextSystem::without_system_fonts();
        let layout = sys.layout("Hello", 16.0, f32::INFINITY);
        let height = layout.height();
        assert!(
            (10.0..40.0).contains(&height),
            "layout height {height} px out of expected range for 16 px font"
        );
    }

    /// Diagnostic: dump glyph widths for the same text at 8 weights.
    /// Use `cargo test -p ph2d-text -- --nocapture diag_weight_widths`
    /// to see the numbers. If widths don't change, the wght FontVariation
    /// isn't actually driving Inter's variable axis.
    #[test]
    fn diag_weight_widths() {
        let mut sys = TextSystem::without_system_fonts();
        let text = "Inspector Hierarchy 0127";
        eprintln!("\n=== weight diagnostic: '{text}' @ 11px ===");
        for w in [300.0_f32, 400.0, 500.0, 550.0, 600.0, 700.0, 800.0, 900.0] {
            let layout = sys.layout_with_weight(text, 11.0, f32::INFINITY, FontWeight::new(w));
            eprintln!("  wght={:>5.0}  width={:>7.2}", w, layout.width());
        }
    }

    #[test]
    fn rendering_default_is_passthrough_weight() {
        let mut sys = TextSystem::without_system_fonts();
        let a = sys.layout_with_weight("Hello", 11.0, f32::INFINITY, FontWeight::MEDIUM);
        let b = sys.layout_for_rendering(
            "Hello",
            11.0,
            f32::INFINITY,
            FontWeight::MEDIUM,
            ph2d_tokens::TextRendering::Default,
        );
        assert_eq!(a.width(), b.width());
        assert_eq!(a.height(), b.height());
    }

    #[test]
    fn rendering_crisp_heavy_bumps_weight_in_body() {
        let mut sys = TextSystem::without_system_fonts();
        let a = sys.layout_with_weight("WWWWWW", 11.0, f32::INFINITY, FontWeight::MEDIUM);
        let b = sys.layout_for_rendering(
            "WWWWWW",
            11.0,
            f32::INFINITY,
            FontWeight::MEDIUM,
            ph2d_tokens::TextRendering::CrispHeavy,
        );
        // Boost of +300 (Medium 500 → ExtraBold 800) at 11 px gives Inter
        // significantly heavier strokes → measurably wider advance over
        // 6× W. Strict `>` (not `>=`) because the body tier guarantees
        // a real bump, not just snap-X behavior.
        assert!(
            b.width() > a.width(),
            "CrispHeavy layout must be wider than default at small sizes; default={} crisp_heavy={}",
            a.width(),
            b.width()
        );
    }

    #[test]
    fn rendering_crisp_heavy_no_op_at_large_size() {
        let mut sys = TextSystem::without_system_fonts();
        let a = sys.layout_with_weight("Hello", 32.0, f32::INFINITY, FontWeight::MEDIUM);
        let b = sys.layout_for_rendering(
            "Hello",
            32.0,
            f32::INFINITY,
            FontWeight::MEDIUM,
            ph2d_tokens::TextRendering::CrispHeavy,
        );
        // Boost is 0 above 20 px → identical to default.
        assert_eq!(a.width(), b.width());
    }

    #[test]
    fn effective_weight_default_passthrough() {
        let nominal = FontWeight::MEDIUM;
        let out = effective_weight(nominal, 11.0, ph2d_tokens::TextRendering::Default);
        assert_eq!(out.value(), nominal.value());
    }

    #[test]
    fn effective_weight_crisp_heavy_clamps_to_900() {
        // FontWeight::BLACK is 900; +300 boost would overshoot to 1200.
        let out = effective_weight(
            FontWeight::BLACK,
            11.0,
            ph2d_tokens::TextRendering::CrispHeavy,
        );
        assert_eq!(out.value(), WEIGHT_MAX);
    }

    #[test]
    fn layout_cache_hits_return_identical_geometry_and_populate() {
        let mut sys = TextSystem::without_system_fonts();
        // First call populates the cache; second (same inputs) hits it.
        let a = sys.layout("Hierarchy", 13.0, 120.0);
        assert_eq!(sys.layout_cache.len(), 1, "first call should cache");
        let b = sys.layout("Hierarchy", 13.0, 120.0);
        assert_eq!(sys.layout_cache.len(), 1, "hit must not add an entry");
        // The cloned (cached) layout is geometrically identical.
        assert_eq!(a.width(), b.width());
        assert_eq!(a.height(), b.height());
        // A different input is a distinct key (miss → new entry).
        let _ = sys.layout("Hierarchy", 14.0, 120.0);
        assert_eq!(sys.layout_cache.len(), 2, "different size is a new key");
    }
}
