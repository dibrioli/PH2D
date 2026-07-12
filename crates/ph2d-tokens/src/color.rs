//! Color token system.
//!
//! Pipeline: OKLCH design tokens (`docs/design/tokens.json`) →
//! `Color::from_oklch` → sRGB 8-bit → consumed by widgets.
//!
//! ### Why OKLCH at the source
//!
//! - Perceptually uniform (same `L` ⇒ same perceived brightness across hues).
//! - Easy theme variants (keep structure, change `H`).
//! - WCAG contrast still computed in linear sRGB space (per spec).
//!
//! **Source-of-truth: `docs/design/tokens.json`** (Wave 2 PR 11.1.0 sync,
//! 2026-05-16). Round 9 color study values were lifted from this file
//! into tokens.json as authoritative. The literals in `resolve_forge` /
//! `resolve_workshop` / `resolve_sunstone` / `resolve_blueprint` below
//! are now a manual mirror — Wave 2 PR 11.1 will replace them with
//! build.rs codegen from tokens.json so divergence becomes impossible.

use crate::theme::Theme;

/// sRGB 8-bit color with alpha. Constructed via `from_hex` ou
/// `from_oklch`; stored as four bytes para cheap copy + tests.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Construct from `0xRRGGBB` (alpha defaults to 0xFF / opaque).
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 0xFF,
        }
    }

    /// Construct from `0xRRGGBBAA`.
    pub const fn from_hex_alpha(hex: u32) -> Self {
        Self {
            r: ((hex >> 24) & 0xFF) as u8,
            g: ((hex >> 16) & 0xFF) as u8,
            b: ((hex >> 8) & 0xFF) as u8,
            a: (hex & 0xFF) as u8,
        }
    }

    /// Construct from OKLCH (L 0..1, C 0..0.4-ish, H 0..360 degrees).
    /// Alpha defaults to opaque. Out-of-gamut colors are clamped to
    /// sRGB [0,1] without warning — drawing outside the gamut is a
    /// design problem, not a function problem.
    pub fn from_oklch(l: f64, c: f64, h_deg: f64) -> Self {
        let [r, g, b] = oklch_to_srgb(l, c, h_deg);
        Self { r, g, b, a: 0xFF }
    }

    /// Construct from OKLCH with alpha (0..1).
    pub fn from_oklch_alpha(l: f64, c: f64, h_deg: f64, alpha: f64) -> Self {
        let [r, g, b] = oklch_to_srgb(l, c, h_deg);
        Self {
            r,
            g,
            b,
            a: (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
        }
    }

    /// Linearize sRGB channel (per WCAG 2.2 contrast formula).
    fn linearize(channel: u8) -> f64 {
        let c = channel as f64 / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Relative luminance per WCAG 2.2 (alpha ignored — assume opaque
    /// over canonical surface for the contrast calculation).
    pub fn relative_luminance(&self) -> f64 {
        let r = Self::linearize(self.r);
        let g = Self::linearize(self.g);
        let b = Self::linearize(self.b);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// WCAG contrast ratio between two opaque colors.
    /// Range: 1.0 (identical) to 21.0 (black on white).
    /// AA text minimum: 4.5. AA UI minimum: 3.0. AAA text: 7.0.
    pub fn contrast_ratio(&self, other: &Self) -> f64 {
        let l1 = self.relative_luminance();
        let l2 = other.relative_luminance();
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }
}

/// Convert OKLCH → sRGB 8-bit per Björn Ottosson's algorithm
/// (https://bottosson.github.io/posts/oklab/). Out-of-gamut colors
/// are clamped per channel.
///
/// `l` in 0..1, `c` ~0..0.4, `h_deg` in degrees.
pub fn oklch_to_srgb(l: f64, c: f64, h_deg: f64) -> [u8; 3] {
    let [lr, lg, lb] = oklch_to_linear_srgb(l, c, h_deg);
    [
        linear_to_srgb_byte(lr),
        linear_to_srgb_byte(lg),
        linear_to_srgb_byte(lb),
    ]
}

/// OKLCH → **linear** sRGB, NOT clamped. Returns the raw linear RGB so
/// callers can test gamut membership ([`oklch_in_gamut`]) before the
/// per-channel clamp that [`oklch_to_srgb`] applies. `l` in 0..1, `c`
/// ~0..0.4, `h_deg` in degrees.
pub fn oklch_to_linear_srgb(l: f64, c: f64, h_deg: f64) -> [f64; 3] {
    // OKLCH → OKLAB
    let h_rad = h_deg.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    // OKLAB → linear LMS (cube)
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    // LMS → linear sRGB
    [
        4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3,
        -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3,
        -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3,
    ]
}

/// True if OKLCH `(l, c, h_deg)` lands inside the sRGB gamut — i.e. all
/// three linear channels fall within `[0, 1]` (small epsilon) BEFORE
/// the clamp in [`oklch_to_srgb`]. Used by the color picker to scale
/// the Chroma slider against the maximum representable chroma for the
/// current lightness + hue (so the slider top is always reachable).
pub fn oklch_in_gamut(l: f64, c: f64, h_deg: f64) -> bool {
    const EPS: f64 = 1e-4;
    oklch_to_linear_srgb(l, c, h_deg)
        .iter()
        .all(|&v| (-EPS..=1.0 + EPS).contains(&v))
}

fn linear_to_srgb_byte(linear: f64) -> u8 {
    let x = linear.clamp(0.0, 1.0);
    let srgb = if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (srgb.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Inverse of [`oklch_to_srgb`]: sRGB 8-bit → OKLCH (L, C, H_deg).
/// Used by [`ColorValue::from_rgba8`] to keep both representations in
/// sync. Per Björn Ottosson's reference. Output: L 0..1, C 0..~0.4,
/// H 0..360.
pub fn srgb_to_oklch(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let lr = srgb_byte_to_linear(r);
    let lg = srgb_byte_to_linear(g);
    let lb = srgb_byte_to_linear(b);
    // linear sRGB → LMS
    let l = 0.412_221_470_8 * lr + 0.536_332_536_3 * lg + 0.051_445_992_9 * lb;
    let m = 0.211_903_498_2 * lr + 0.680_699_545_1 * lg + 0.107_396_956_6 * lb;
    let s = 0.088_302_461_9 * lr + 0.281_718_837_6 * lg + 0.629_978_700_5 * lb;
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();
    // LMS → OKLAB
    let lab_l = 0.210_454_255_3 * l_ + 0.793_617_785_0 * m_ - 0.004_072_046_8 * s_;
    let lab_a = 1.977_998_495_1 * l_ - 2.428_592_205_0 * m_ + 0.450_593_709_9 * s_;
    let lab_b = 0.025_904_037_1 * l_ + 0.782_771_766_2 * m_ - 0.808_675_766_0 * s_;
    let c = (lab_a * lab_a + lab_b * lab_b).sqrt();
    let mut h = lab_b.atan2(lab_a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    (lab_l, c, h)
}

fn srgb_byte_to_linear(channel: u8) -> f64 {
    let c = channel as f64 / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Color value carrying both sRGB 8-bit and OKLCH in sync. Editor
/// code (paint helpers, ColorSwatch) consumes `rgba`; design tokens
/// and ColorPicker math operate on `oklch`. Construct via
/// [`ColorValue::from_rgba8`] or [`ColorValue::from_oklch`] — both
/// constructors compute the missing representation so the pair stays
/// consistent.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColorValue {
    pub rgba: [u8; 4],
    /// `(L, C, H_deg, alpha)` — alpha mirrors `rgba[3] / 255.0`.
    pub oklch: (f64, f64, f64, f64),
}

impl ColorValue {
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let (l, c, h) = srgb_to_oklch(r, g, b);
        Self {
            rgba: [r, g, b, a],
            oklch: (l, c, h, a as f64 / 255.0),
        }
    }

    pub fn from_oklch(l: f64, c: f64, h_deg: f64, alpha: f64) -> Self {
        let [r, g, b] = oklch_to_srgb(l, c, h_deg);
        let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
        Self {
            rgba: [r, g, b, a],
            oklch: (l, c, h_deg, alpha.clamp(0.0, 1.0)),
        }
    }

    pub const TRANSPARENT: Self = Self {
        rgba: [0, 0, 0, 0],
        oklch: (0.0, 0.0, 0.0, 0.0),
    };

    pub const BLACK: Self = Self {
        rgba: [0, 0, 0, 255],
        oklch: (0.0, 0.0, 0.0, 1.0),
    };

    pub const WHITE: Self = Self {
        rgba: [255, 255, 255, 255],
        oklch: (1.0, 0.0, 0.0, 1.0),
    };
}

/// Declarative source of truth for the semantic color palette (ADR-0107,
/// Camada 0). ONE list drives both the `ColorToken` enum and its `key()`
/// map — no separate `match` to hand-maintain, so adding a token is a single
/// new line here (Mergiraf unions disjoint additions; no second site to
/// conflict on). Values live in `docs/design/tokens.json`, resolved below.
macro_rules! color_tokens {
    ($( $(#[$vmeta:meta])* $variant:ident => $key:literal ),* $(,)?) => {
        /// Semantic color slot — every widget references one of these by name;
        /// literal `from_hex`/`from_oklch` outside this crate is a code smell.
        ///
        /// Variants map 1:1 to `color.*` keys in `docs/design/tokens.json`.
        /// Adding a new slot: add ONE line to the `color_tokens!` list below +
        /// the value to `docs/design/tokens.json` (all themes).
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum ColorToken { $( $(#[$vmeta])* $variant ),* }

        impl ColorToken {
            /// Stable kebab-case key for this token, matching the entry in
            /// `docs/design/tokens.json`. Generated from the `color_tokens!`
            /// list — never hand-edited.
            pub const fn key(self) -> &'static str {
                match self { $( Self::$variant => $key ),* }
            }
        }
    };
}

color_tokens! {
    // ── Background scale (4 levels + elevated + scrim) ─────────────
    /// `bg-0` — base canvas backdrop.
    Bg0 => "bg-0",
    /// `bg-1` — first elevation (panels, sidebar).
    Bg1 => "bg-1",
    /// `bg-2` — second elevation (cards inside panels).
    Bg2 => "bg-2",
    /// `bg-3` — third elevation (input rows, list items).
    Bg3 => "bg-3",
    /// `bg-elev` — popovers/tooltips/floating panels (slight alpha).
    BgElev => "bg-elev",
    /// `bg-scrim` — modal backdrop (heavy alpha).
    BgScrim => "bg-scrim",
    /// `rail-bg` — semi-transparent "frosted glass" backing for chrome
    /// strips that sit directly on the canvas (LeftRail labels). Lower
    /// alpha than `BgScrim` so canvas content stays partially visible
    /// underneath; dark in dark themes, light in light themes to keep
    /// the existing chrome text colors readable on top.
    RailBg => "rail-bg",
    /// `panel-bg` — surface fill for floating side panels (Inspector,
    /// Hierarchy). Same hue as `BgElev` but slightly translucent (~8 %
    /// alpha bleed) so panels feel "floated over canvas" instead of
    /// pure overlay. Lower transparency than `RailBg` — content
    /// readability stays priority on panel bodies.
    PanelBg => "panel-bg",

    // ── Borders (3 levels) ─────────────────────────────────────────
    /// `border` — low-contrast separators.
    Border => "border",
    /// `border-strong` — visible dividers.
    BorderStrong => "border-strong",
    /// `border-emph` — focus rings, active selection borders.
    BorderEmph => "border-emph",

    // ── Text scale (3 levels + disabled) ───────────────────────────
    /// `text-1` — primary copy. ≥ 4.5:1 vs Bg1 (AA).
    Text1 => "text-1",
    /// `text-2` — labels, captions. ≥ 4.5:1 vs Bg1 (AA).
    Text2 => "text-2",
    /// `text-3` — tertiary/hints.
    Text3 => "text-3",
    /// `text-disabled` — explicit non-interactive (3:1 OK per WCAG).
    TextDisabled => "text-disabled",

    // ── Accent stack ───────────────────────────────────────────────
    /// `accent` — primary call-to-action / active state.
    Accent => "accent",
    /// `accent-hover` — hover state.
    AccentHover => "accent-hover",
    /// `accent-press` — pressed state.
    AccentPress => "accent-press",
    /// `accent-soft` — accent at low alpha (selection tints).
    AccentSoft => "accent-soft",
    /// `accent-fg` — foreground on accent (contrast guaranteed).
    AccentFg => "accent-fg",

    // ── Semantic states ────────────────────────────────────────────
    Danger => "danger",
    DangerSoft => "danger-soft",
    Success => "success",
    SuccessSoft => "success-soft",
    Warn => "warn",
    WarnSoft => "warn-soft",
    Info => "info",
    InfoSoft => "info-soft",

    // ── Editor-specific ────────────────────────────────────────────
    /// `selection` — selected entity highlight.
    Selection => "selection",
    /// `focus-ring` — keyboard focus indicator.
    FocusRing => "focus-ring",
    /// `grid-line` — minor grid stroke on canvas.
    GridLine => "grid-line",
    /// `grid-axis` — major axis line on canvas.
    GridAxis => "grid-axis",
    /// `curve-r` — red channel tint for the Curves editor (R tab curve + handle ring).
    CurveR => "curve-r",
    /// `curve-g` — green channel tint for the Curves editor.
    CurveG => "curve-g",
    /// `curve-b` — blue channel tint for the Curves editor.
    CurveB => "curve-b",
    /// `canvas` — viewport background (scene render target backdrop).
    Canvas => "canvas",

    // ── Motion Nodes: node-category header tints (M0.T5, §2.4) ─────
    // Category identity hues; the card header is `mix(cat, Bg2, 0.55)`.
    // Same value across all themes — node graph canvas is dark in every
    // theme (universal node-editor convention), so the identity holds.
    /// `node-cat-source` — generators / distribution sources.
    NodeCatSource => "node-cat-source",
    /// `node-cat-distribute` — cloners / distributions.
    NodeCatDistribute => "node-cat-distribute",
    /// `node-cat-transform` — modifiers (move/scale/rotate/tint).
    NodeCatTransform => "node-cat-transform",
    /// `node-cat-focus` — falloffs / focus fields.
    NodeCatFocus => "node-cat-focus",
    /// `node-cat-fx` — effects (blur/glow/mirror…).
    NodeCatFx => "node-cat-fx",
    /// `node-cat-output` — sinks / terminals.
    NodeCatOutput => "node-cat-output",
    /// `node-cat-utility` — expressions / adapters / values.
    NodeCatUtility => "node-cat-utility",

    // ── Motion Nodes: port colors by PortType axis ─────────────────
    // Socket color = Domain (or Clock for event/static); shape = Dim.
    /// `port-instances` — `Domain::Instances` (purple).
    PortInstances => "port-instances",
    /// `port-vector` — `Domain::Vector` (blue).
    PortVector => "port-vector",
    /// `port-field` — `Domain::Field` (orange).
    PortField => "port-field",
    /// `port-signal` — `Domain::Signal` (teal).
    PortSignal => "port-signal",
    /// `port-control` — `Domain::Control` (lime).
    PortControl => "port-control",
    /// `port-event` — `Clock::Event` pulse (magenta).
    PortEvent => "port-event",
    /// `port-static` — `Clock::Static` const (neutral).
    PortStatic => "port-static",

    // ── Motion Nodes: graph surfaces + wire activity ───────────────
    /// `graph-bg` — node canvas backdrop (dark in every theme).
    GraphBg => "graph-bg",
    /// `graph-grid` — faint background grid.
    GraphGrid => "graph-grid",
    /// `wire-fire-glow` — decaying glow on an edge whose value changed.
    WireFireGlow => "wire-fire-glow",

    // ── Motion Nodes: translucent group-backdrop tints (8) ─────────
    /// `graph-backdrop-1` — translucent group-region tint.
    GraphBackdrop1 => "graph-backdrop-1",
    /// `graph-backdrop-2` — translucent group-region tint.
    GraphBackdrop2 => "graph-backdrop-2",
    /// `graph-backdrop-3` — translucent group-region tint.
    GraphBackdrop3 => "graph-backdrop-3",
    /// `graph-backdrop-4` — translucent group-region tint.
    GraphBackdrop4 => "graph-backdrop-4",
    /// `graph-backdrop-5` — translucent group-region tint.
    GraphBackdrop5 => "graph-backdrop-5",
    /// `graph-backdrop-6` — translucent group-region tint.
    GraphBackdrop6 => "graph-backdrop-6",
    /// `graph-backdrop-7` — translucent group-region tint.
    GraphBackdrop7 => "graph-backdrop-7",
    /// `graph-backdrop-8` — translucent group-region tint.
    GraphBackdrop8 => "graph-backdrop-8",

    /// `graph-marquee` — translucent fill of the node graph's box-select band
    /// (the border uses `accent`). Translucent BY CONTRACT: the band is drawn over
    /// the very cards it is selecting, and an opaque fill would hide them.
    GraphMarquee => "graph-marquee",

    /// `graph-inert` — translucent scrim laid over a node the cook never pulled: a card
    /// nothing downstream consumes. Translucent BY CONTRACT (it is a veil over a card that
    /// must stay legible and grabbable, not a repaint of it), and the PANEL's own
    /// background hue, so a dead card reads as *receding into the canvas* rather than as a
    /// new colour that has to be learnt.
    GraphInert => "graph-inert",

    /// `attr-write` — gold badge for attribute-write on a node.
    AttrWrite => "attr-write",

    // ── Timeline panel (W2.E9) ─────────────────────────────────────
    // Dedicated, retintable slots for the animation timeline. Seeded
    // byte-identical to the generic tokens they used to borrow, so a
    // theme can recolor the timeline without touching global accent.
    /// `timeline-ruler-bg` — ruler strip background.
    TimelineRulerBg => "timeline-ruler-bg",
    /// `timeline-ruler-tick` — ruler tick marks (minor + major).
    TimelineRulerTick => "timeline-ruler-tick",
    /// `timeline-playhead` — playhead scrubber line across ruler + lanes.
    TimelinePlayhead => "timeline-playhead",
    /// `timeline-row-alt` — zebra alternate-row fill in the track list.
    TimelineRowAlt => "timeline-row-alt",
    /// `timeline-key` — unselected keyframe diamond / curve anchor.
    TimelineKey => "timeline-key",
    /// `timeline-key-selected` — selected keyframe diamond / curve anchor.
    TimelineKeySelected => "timeline-key-selected",
    /// `timeline-key-active` — keyframe / anchor / handle while dragging.
    TimelineKeyActive => "timeline-key-active",
    /// `timeline-curve` — animation curve stroke in the graph editor.
    TimelineCurve => "timeline-curve",
    /// `timeline-handle` — bézier tangent handle dot (idle).
    TimelineHandle => "timeline-handle",
    /// `timeline-handle-line` — bézier tangent line.
    TimelineHandleLine => "timeline-handle-line",
    /// `timeline-loop-region` — shaded loop-range band.
    TimelineLoopRegion => "timeline-loop-region",
    /// `timeline-loop-brace` — loop brace bars + inner ticks.
    TimelineLoopBrace => "timeline-loop-brace",
    /// `timeline-marker` — marker stem line + pennant triangle.
    TimelineMarker => "timeline-marker",
    /// `timeline-summary-key` — amber master (summary) diamond.
    TimelineSummaryKey => "timeline-summary-key",
    /// `timeline-summary-ring` — summary column selection ring.
    TimelineSummaryRing => "timeline-summary-ring",
    /// `timeline-missing` — track label whose bound object is missing.
    TimelineMissing => "timeline-missing",
}

impl ColorToken {
    /// Resolve token → concrete Color for the given Theme via lookup in
    /// the codegen'd `COLORS_<THEME>` table from
    /// `docs/design/tokens.json`. Wave 2 PR 11.1: values flow from
    /// canonical tokens.json → `build.rs` → `crate::generated` → here.
    /// Sync manual eliminado.
    pub fn resolve(self, theme: Theme) -> Color {
        let table: &[(&str, crate::generated::OklchRaw)] = match theme {
            Theme::Forge => crate::generated::COLORS_FORGE,
            Theme::Workshop => crate::generated::COLORS_WORKSHOP,
            Theme::Sunstone => crate::generated::COLORS_SUNSTONE,
            Theme::Blueprint => crate::generated::COLORS_BLUEPRINT,
        };
        lookup_color(table, self.key())
    }
}

/// Linear-scan lookup of a color token in a per-theme table generated
/// from `docs/design/tokens.json`. Tables have ~60 entries; ~ns per
/// lookup. Not a hot path (paint frame walks tokens once per widget,
/// not per pixel). Panics on missing key — design-canonical integrity
/// error (token added to ColorToken enum without matching entry in
/// tokens.json) caught immediately at first resolve.
fn lookup_color(table: &[(&str, crate::generated::OklchRaw)], key: &str) -> Color {
    for (k, v) in table.iter() {
        if *k == key {
            return if v.a < 1.0 {
                Color::from_oklch_alpha(v.l, v.c, v.h, v.a)
            } else {
                Color::from_oklch(v.l, v.c, v.h)
            };
        }
    }
    panic!(
        "color token {key:?} not found in canonical table — \
         check docs/design/tokens.json for missing entry"
    );
}

#[cfg(test)]
#[path = "color_tests.rs"]
mod tests;
