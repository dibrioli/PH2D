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

use std::sync::Arc;

use std::borrow::Cow;

use parley::{
    Alignment, FontContext, FontSettings, FontStack, FontVariation, FontWeight, Layout,
    LayoutContext, StyleProperty, fontique::Blob, swash::tag_from_bytes,
};

/// OpenType axis tag for "Optical Size" (`opsz`). Inter 4.x supports
/// the range 14–32: at smaller `opsz` Inter substitutes a "Text" cut
/// (heavier strokes, more open counters) so glyphs hold up under
/// GPU rasterization at 12-14 px; at larger `opsz` it shifts toward
/// a "Display" cut (tighter spacing, finer details). Values outside
/// the axis range are clamped by skrifa.
const OPSZ_TAG: u32 = tag_from_bytes(b"opsz");

/// Inter Variable (v4.0, SIL OFL) — bundled so chrome text rasterizes
/// to the same glyphs everywhere, independent of installed system fonts.
/// Inter was designed for screen rendering without LCD subpixel AA,
/// which matches Vello's glyph pipeline (vs. system fonts like SF that
/// are tuned for CoreText's subpixel rendering and look soft here).
/// Source: <https://github.com/rsms/inter/releases/tag/v4.0> (LICENSE.txt
/// in this directory).
const INTER_VARIABLE_TTF: &[u8] = include_bytes!("../fonts/InterVariable.ttf");

/// Family name registered when bundled Inter loads successfully.
/// Falls back to `sans-serif` if registration fails (corrupted bytes,
/// future fontique breaking change, etc.) so we never panic at startup.
const INTER_FAMILY: &str = "InterVariable";

pub struct TextSystem {
    font_context: FontContext,
    layout_context: LayoutContext<()>,
    /// Resolved primary stack — "InterVariable, sans-serif" when the
    /// bundled font registered, plain "sans-serif" otherwise.
    primary_stack: String,
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
        // Drive Inter's optical-size axis from the actual render
        // size: at 12-14 px we get the heavier "Text" cut (better
        // GPU rasterization without subpixel AA), at 20+ px the
        // "Display" cut (tighter, more elegant proportions).
        // `font_size` falls outside [14, 32] for headings/sub-text;
        // skrifa clamps to the axis range so this is safe.
        let variations = [FontVariation {
            tag: OPSZ_TAG,
            value: font_size,
        }];
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
    pub fn prefix_width(&mut self, prefix: &str, font_size: f32) -> f32 {
        if prefix.is_empty() {
            return 0.0;
        }
        // Fast path: prefix has no trailing space → the plain
        // layout already returns the correct width.
        if !prefix.ends_with(' ') && !prefix.ends_with('\t') {
            return self.layout(prefix, font_size, f32::INFINITY).width();
        }
        const SENTINEL: &str = "|";
        let mut combined = String::with_capacity(prefix.len() + SENTINEL.len());
        combined.push_str(prefix);
        combined.push_str(SENTINEL);
        let w_with = self.layout(&combined, font_size, f32::INFINITY).width();
        let w_sentinel = self.layout(SENTINEL, font_size, f32::INFINITY).width();
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

#[cfg(test)]
mod tests {
    use super::*;

    /// "Hello PH2D" at 16 px must produce at least one glyph run.
    /// This is the M11 ASCII gate — ensures parley + system fonts
    /// + our wrapper compose into something usable.
    #[test]
    fn ascii_hello_ph2d_produces_glyphs() {
        let mut sys = TextSystem::new();
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
        let mut sys = TextSystem::new();
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
        let mut sys = TextSystem::new();
        let layout = sys.layout("Hello", 16.0, f32::INFINITY);
        let height = layout.height();
        assert!(
            (10.0..40.0).contains(&height),
            "layout height {height} px out of expected range for 16 px font"
        );
    }
}
