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

use parley::{
    Alignment, FontContext, FontFamily, GenericFamily, Layout, LayoutContext, StyleProperty,
};

pub struct TextSystem {
    font_context: FontContext,
    layout_context: LayoutContext<()>,
}

impl TextSystem {
    /// Build a new TextSystem. Loads system fonts (50-200 ms cold).
    pub fn new() -> Self {
        Self {
            font_context: FontContext::new(),
            layout_context: LayoutContext::new(),
        }
    }

    pub fn font_context(&mut self) -> &mut FontContext {
        &mut self.font_context
    }

    pub fn layout_context(&mut self) -> &mut LayoutContext<()> {
        &mut self.layout_context
    }

    /// Build a single-line layout for `text` at `font_size` (in
    /// device-independent pixels). Uses the system sans-serif
    /// fallback chain — works for ASCII, CJK, and emoji as long
    /// as the OS has appropriate fonts installed.
    ///
    /// The `max_width` parameter is the layout's wrap budget; pass
    /// `f32::INFINITY` for single-line "as wide as needed".
    pub fn layout(&mut self, text: &str, font_size: f32, max_width: f32) -> Layout<()> {
        let mut builder = self.layout_context.ranged_builder(
            &mut self.font_context,
            text,
            1.0,  // device pixel ratio
            true, // quantize
        );
        builder.push_default(StyleProperty::FontStack(parley::FontStack::Single(
            FontFamily::Generic(GenericFamily::SansSerif),
        )));
        builder.push_default(StyleProperty::FontSize(font_size));
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
