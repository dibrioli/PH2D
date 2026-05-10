//! [`BlenderColorPicker`] — Blender-style color picker widget.
//!
//! Layout (top to bottom):
//!
//! 1. **Wheel** (HSV disc) + **vertical value slider** at right.
//! 2. **Linear / Perceptual** segmented toggle (interpolation hint).
//! 3. **RGB / HSV** segmented toggle (which channel triple appears
//!    in the sliders below).
//! 4. **4 horizontal sliders** (R+G+B+A or H+S+V+A) — each row has a
//!    label, a slider track, and a NumberInput-style value chip
//!    (paints inline; full NumberInput interactivity wires when the
//!    Inspector hosts it via [`crate::interaction`]).
//! 5. **Hex field** (TextInput-style) + **eyedropper button**.
//! 6. **Palettes section** at the bottom: Tabs (palette names) +
//!    grid of [`ColorSwatch`]es + add/remove buttons.
//!
//! Output value is a [`ph2d_tokens::ColorValue`] (rgba + oklch in
//! sync). Theme tokens drive every chrome color; the wheel + value
//! slider show the user content.
//!
//! v1 ships the structure + paint scaffold. Per-control input wiring
//! (drag wheel, type hex) lands in subsequent passes via the
//! [`crate::interaction`] dispatcher.

use crate::icons::IconId;
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use crate::widget::{
    ColorSwatch, RadioGroup, RadioOption, RadioOrientation, TabItem, Tabs, TabsVariant,
    paint_color_swatch, paint_radio_group, paint_tabs,
};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ColorValue, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Color as VelloColor, Fill, Point, VectorScene};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum InterpolationMode {
    Linear,
    /// OKLCH-perceptually uniform (Blender's default).
    #[default]
    Perceptual,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ChannelMode {
    #[default]
    Rgb,
    Hsv,
}

/// One palette: a named collection of editable swatches.
#[derive(Clone, Debug)]
pub struct ColorPalette {
    pub name: String,
    pub swatches: Vec<ColorValue>,
    /// When false, the palette is read-only (built-in / shared);
    /// add/remove buttons paint disabled.
    pub editable: bool,
}

impl ColorPalette {
    pub fn new(name: impl Into<String>, swatches: Vec<ColorValue>) -> Self {
        Self {
            name: name.into(),
            swatches,
            editable: true,
        }
    }

    pub fn read_only(mut self) -> Self {
        self.editable = false;
        self
    }
}

/// A 12-swatch starter palette inspired by Blender's default brand
/// chips. Read-only.
pub fn default_palette() -> ColorPalette {
    let swatches = [
        [231u8, 231, 231, 255],
        [40, 40, 40, 255],
        [0, 122, 204, 255],
        [240, 96, 0, 255],
        [220, 50, 50, 255],
        [60, 200, 100, 255],
        [200, 200, 60, 255],
        [180, 80, 200, 255],
        [60, 200, 200, 255],
        [120, 120, 200, 255],
        [200, 120, 60, 255],
        [120, 80, 60, 255],
    ]
    .into_iter()
    .map(|[r, g, b, a]| ColorValue::from_rgba8(r, g, b, a))
    .collect();
    ColorPalette {
        name: "Default".into(),
        swatches,
        editable: false,
    }
}

#[derive(Clone, Debug)]
pub struct BlenderColorPicker {
    pub id: NodeId,
    pub label: String,
    pub value: ColorValue,
    pub interpolation: InterpolationMode,
    pub channel_mode: ChannelMode,
    pub palettes: Vec<ColorPalette>,
    pub active_palette: usize,
    /// Hex string preview (`"#RRGGBBAA"`). Mirrors `value.rgba` —
    /// updated by [`Self::sync_hex`] on every value change.
    pub hex: String,
}

impl BlenderColorPicker {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        let value = ColorValue::from_rgba8(231, 231, 231, 255);
        let mut s = Self {
            id,
            label: label.into(),
            value,
            interpolation: InterpolationMode::Perceptual,
            channel_mode: ChannelMode::Rgb,
            palettes: vec![default_palette()],
            active_palette: 0,
            hex: String::new(),
        };
        s.sync_hex();
        s
    }

    pub fn value(mut self, value: ColorValue) -> Self {
        self.value = value;
        self.sync_hex();
        self
    }

    pub fn interpolation(mut self, mode: InterpolationMode) -> Self {
        self.interpolation = mode;
        self
    }

    pub fn channel_mode(mut self, mode: ChannelMode) -> Self {
        self.channel_mode = mode;
        self
    }

    pub fn palettes(mut self, palettes: Vec<ColorPalette>) -> Self {
        self.palettes = palettes;
        self
    }

    pub fn active_palette(mut self, idx: usize) -> Self {
        if idx < self.palettes.len() {
            self.active_palette = idx;
        }
        self
    }

    /// Set [`Self::value`] from a swatch click — keeps oklch + rgba
    /// + hex consistent.
    pub fn set_value(&mut self, value: ColorValue) {
        self.value = value;
        self.sync_hex();
    }

    fn sync_hex(&mut self) {
        let [r, g, b, a] = self.value.rgba;
        self.hex = format!("#{r:02X}{g:02X}{b:02X}{a:02X}");
    }

    /// AccessKit `Role::Group` parent — children would be added by
    /// the consumer when wiring to the live store (each segmented
    /// toggle, slider row, swatch are their own nodes).
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Group)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(true)
            .action(Action::Focus)
            .build()
    }
}

const WHEEL_SIZE: f32 = 232.0;
const VALUE_SLIDER_W: f32 = 24.0;
const ROW_GAP: f32 = 8.0;
const TOGGLE_H: f32 = 28.0;
const SLIDER_ROW_H: f32 = 22.0;
const HEX_ROW_H: f32 = 28.0;

/// Paint the picker. Rect minimum: ~272×500 (wheel + 4 sliders + hex
/// + palette grid). Larger rects center horizontally.
pub fn paint_blender_color_picker(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let pad = Spacing::Lg.px();
    let inner_w = rect.w - pad * 2.0;
    let mut y = rect.y + pad;

    // 1. Wheel + vertical value slider.
    let wheel_block_w = WHEEL_SIZE + Spacing::Md.px() + VALUE_SLIDER_W;
    let wheel_x = rect.x + (rect.w - wheel_block_w) * 0.5;
    let wheel_rect = Rect::new(wheel_x, y, WHEEL_SIZE, WHEEL_SIZE);
    paint_color_wheel(cp, wheel_rect, scene);
    let value_rect = Rect::new(
        wheel_x + WHEEL_SIZE + Spacing::Md.px(),
        y,
        VALUE_SLIDER_W,
        WHEEL_SIZE,
    );
    paint_value_slider(cp, value_rect, scene, theme);
    y += WHEEL_SIZE + ROW_GAP;

    // 2. Linear/Perceptual segmented toggle.
    let interp_rect = Rect::new(rect.x + pad, y, inner_w, TOGGLE_H);
    let interp_group = RadioGroup::new(
        NodeId(0),
        "Interpolation",
        vec![
            RadioOption::new(NodeId(0), "linear", "Linear"),
            RadioOption::new(NodeId(0), "perceptual", "Perceptual"),
        ],
    )
    .orientation(RadioOrientation::Segmented)
    .selected(match cp.interpolation {
        InterpolationMode::Linear => "linear",
        InterpolationMode::Perceptual => "perceptual",
    });
    paint_radio_group(&interp_group, interp_rect, scene, theme);
    y += TOGGLE_H + ROW_GAP;

    // 3. RGB/HSV segmented toggle.
    let chan_rect = Rect::new(rect.x + pad, y, inner_w, TOGGLE_H);
    let chan_group = RadioGroup::new(
        NodeId(0),
        "Channel mode",
        vec![
            RadioOption::new(NodeId(0), "rgb", "RGB"),
            RadioOption::new(NodeId(0), "hsv", "HSV"),
        ],
    )
    .orientation(RadioOrientation::Segmented)
    .selected(match cp.channel_mode {
        ChannelMode::Rgb => "rgb",
        ChannelMode::Hsv => "hsv",
    });
    paint_radio_group(&chan_group, chan_rect, scene, theme);
    y += TOGGLE_H + ROW_GAP;

    // 4. 4 sliders (R/G/B/A or H/S/V/A).
    let labels = match cp.channel_mode {
        ChannelMode::Rgb => ["Red", "Green", "Blue", "Alpha"],
        ChannelMode::Hsv => ["Hue", "Saturation", "Value", "Alpha"],
    };
    let values = match cp.channel_mode {
        ChannelMode::Rgb => [
            cp.value.rgba[0] as f32 / 255.0,
            cp.value.rgba[1] as f32 / 255.0,
            cp.value.rgba[2] as f32 / 255.0,
            cp.value.rgba[3] as f32 / 255.0,
        ],
        ChannelMode::Hsv => {
            let (h, s, v, a) = rgba_to_hsv(cp.value.rgba);
            [h, s, v, a]
        }
    };
    for (i, (label, val)) in labels.iter().zip(values.iter()).enumerate() {
        let row_y = y + (SLIDER_ROW_H + 4.0) * i as f32;
        let row_rect = Rect::new(rect.x + pad, row_y, inner_w, SLIDER_ROW_H);
        paint_slider_row(label, *val, row_rect, scene, text_system, theme);
    }
    y += (SLIDER_ROW_H + 4.0) * 4.0 + ROW_GAP;

    // 5. Hex field + eyedropper.
    let hex_rect = Rect::new(rect.x + pad, y, inner_w - 32.0, HEX_ROW_H);
    let eye_rect = Rect::new(hex_rect.x + hex_rect.w + 4.0, y, HEX_ROW_H, HEX_ROW_H);
    paint_hex_field(&cp.hex, hex_rect, scene, text_system, theme);
    paint_eyedropper(eye_rect, scene, theme);
    y += HEX_ROW_H + ROW_GAP;

    // 6. Palettes section.
    let palette_h = (rect.y + rect.h - y - pad).max(0.0);
    let palette_rect = Rect::new(rect.x + pad, y, inner_w, palette_h);
    if palette_h > 60.0 {
        paint_palettes(cp, palette_rect, scene, text_system, theme);
    }
}

/// Paint the HSV color wheel + cursor crosshair at the position
/// implied by `cp.value.oklch` (h_deg → angle, c → distance from
/// center).
///
/// v1 ships a solid neutral disc — full HSV gradient painting needs
/// either a sweep gradient (Vello has no native primitive yet) or
/// per-pixel sampling, both follow-up work. The cursor + crosshair
/// already track value correctly so the wheel is functionally
/// useful as a value-position indicator.
fn paint_color_wheel(cp: &BlenderColorPicker, rect: Rect, scene: &mut VectorScene) {
    use ph2d_vector::Circle;
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let radius = (rect.w.min(rect.h)) * 0.5 - 2.0;
    let disc = Circle::new(Point::new(cx as f64, cy as f64), radius as f64);
    // Solid neutral disc (~Bg2 grey). Hue ring approximation lands
    // in a follow-up when Vello gains sweep gradient support.
    let disc_brush = Brush::Solid(VelloColor::from_rgba8(120, 120, 120, 255));
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, &disc_brush, None, &disc);

    // Outer hue ring sample — 12 wedges, each filled with a
    // representative hue. Approximates a sweep gradient at low cost.
    let inner_r = radius * 0.78;
    for step in 0..12 {
        let h = step as f32 * 30.0;
        // Build a small triangle-ish path between inner ring and
        // outer ring at this hue angle. Simpler: a small filled
        // circle along the rim.
        let theta = h.to_radians();
        let mid_r = (radius + inner_r) * 0.5;
        let px = cx + theta.cos() * mid_r;
        let py = cy + theta.sin() * mid_r;
        let dot = Circle::new(
            Point::new(px as f64, py as f64),
            ((radius - inner_r) * 0.5) as f64,
        );
        let cv = ColorValue::from_oklch(0.7, 0.18, h as f64, 1.0);
        let brush = Brush::Solid(VelloColor::from_rgba8(
            cv.rgba[0], cv.rgba[1], cv.rgba[2], 255,
        ));
        scene
            .inner_mut()
            .fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &dot);
    }

    // Cursor crosshair.
    let (_, c, h, _) = cp.value.oklch;
    let normalized_c = (c / 0.4).clamp(0.0, 1.0) as f32;
    let theta = (h as f32).to_radians();
    let cur_x = cx + theta.cos() * radius * normalized_c;
    let cur_y = cy + theta.sin() * radius * normalized_c;
    let cursor_r: f64 = 6.0;
    let cursor = Circle::new(Point::new(cur_x as f64, cur_y as f64), cursor_r);
    let stroke = ph2d_vector::Stroke::new(2.0);
    scene.inner_mut().stroke(
        &stroke,
        Affine::IDENTITY,
        &Brush::Solid(VelloColor::WHITE),
        None,
        &cursor,
    );
}

fn paint_value_slider(cp: &BlenderColorPicker, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    // Filled portion proportional to L (OKLCH lightness).
    let l = cp.value.oklch.0 as f32;
    let fill_h = rect.h * l;
    let fill_rect = Rect::new(rect.x, rect.y + rect.h - fill_h, rect.w, fill_h);
    fill_rounded_rect(scene, fill_rect, radius, resolve(ColorToken::Text1, theme));
}

fn paint_slider_row(
    label: &str,
    value: f32,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let label_w = 70.0;
    let val_w = 60.0;
    let track_x = rect.x + label_w + Spacing::Sm.px();
    let track_w = rect.w - label_w - val_w - Spacing::Sm.px() * 2.0;
    let label_rect = Rect::new(rect.x, rect.y, label_w, rect.h);
    let track_rect = Rect::new(track_x, rect.y + 6.0, track_w, rect.h - 12.0);
    let val_rect = Rect::new(rect.x + rect.w - val_w, rect.y, val_w, rect.h);

    // Label chip on the left (Blender style — solid blue background).
    fill_rounded_rect(
        scene,
        label_rect,
        Radius::Xs.px(),
        resolve(ColorToken::AccentPress, theme),
    );
    paint_text_centered(
        text_system,
        scene,
        label,
        label_rect,
        TypeToken::Xs.px() - 1.0,
        resolve(ColorToken::AccentFg, theme),
    );

    // Track + filled portion.
    fill_rounded_rect(
        scene,
        track_rect,
        Radius::Xs.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let fill_w = track_rect.w * value.clamp(0.0, 1.0);
    if fill_w > 0.0 {
        let filled = Rect::new(track_rect.x, track_rect.y, fill_w, track_rect.h);
        fill_rounded_rect(
            scene,
            filled,
            Radius::Xs.px(),
            resolve(ColorToken::Border, theme),
        );
    }

    // Value chip on the right (NumberInput-style numeric display).
    fill_rounded_rect(
        scene,
        val_rect,
        Radius::Xs.px(),
        resolve(ColorToken::Bg3, theme),
    );
    let display = format!("{value:.3}");
    paint_text_centered(
        text_system,
        scene,
        &display,
        val_rect,
        TypeToken::Xs.px(),
        resolve(ColorToken::Text1, theme),
    );
}

fn paint_hex_field(
    hex: &str,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    let pad = Spacing::Md.px();
    let label_w = 36.0;
    let label_rect = Rect::new(rect.x + pad, rect.y, label_w, rect.h);
    paint_text(
        text_system,
        scene,
        "Hex",
        label_rect.x,
        label_rect.y + (label_rect.h - TypeToken::Xs.px()) * 0.5,
        TypeToken::Xs.px(),
        label_w,
        resolve(ColorToken::Text2, theme),
    );
    paint_text(
        text_system,
        scene,
        hex,
        rect.x + pad + label_w,
        rect.y + (rect.h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        rect.w - pad * 2.0 - label_w,
        resolve(ColorToken::Text1, theme),
    );
}

fn paint_eyedropper(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    paint_icon(
        scene,
        IconId::EyePencil,
        rect,
        resolve(ColorToken::Text2, theme),
        1.5,
    );
}

fn paint_palettes(
    cp: &BlenderColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let tabs_h = 28.0_f32;
    let tabs_rect = Rect::new(rect.x, rect.y, rect.w, tabs_h);
    let tab_items: Vec<TabItem> = cp
        .palettes
        .iter()
        .enumerate()
        .map(|(i, p)| TabItem::new(NodeId(i as u64), p.name.clone()))
        .collect();
    if tab_items.is_empty() {
        return;
    }
    let tabs = Tabs::new(NodeId(0), "Palettes", tab_items)
        .selected(cp.active_palette)
        .variant(TabsVariant::Segmented);
    paint_tabs(&tabs, tabs_rect, scene, text_system, theme);

    let body_y = rect.y + tabs_h + Spacing::Md.px();
    let body_rect = Rect::new(rect.x, body_y, rect.w, rect.y + rect.h - body_y);
    if let Some(palette) = cp.palettes.get(cp.active_palette) {
        paint_palette_grid(palette, body_rect, scene, text_system, theme);
    }
}

fn paint_palette_grid(
    palette: &ColorPalette,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let swatch_size = 24.0_f32;
    let gap = Spacing::Xs.px();
    let cols = ((rect.w + gap) / (swatch_size + gap)).max(1.0) as usize;
    for (i, value) in palette.swatches.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let x = rect.x + (swatch_size + gap) * col as f32;
        let y = rect.y + (swatch_size + gap) * row as f32;
        if y + swatch_size > rect.y + rect.h {
            break;
        }
        let swatch_rect = Rect::new(x, y, swatch_size, swatch_size);
        let mut sw = ColorSwatch::new(NodeId(i as u64), &palette.name, value.rgba);
        sw.size = crate::widget::SwatchSize::Sm;
        paint_color_swatch(&sw, swatch_rect, scene, theme);
    }
    if !palette.editable {
        // Indicate locked palette with a small "lock" hint at the
        // bottom-left.
        let hint_y = rect.y + rect.h - TypeToken::Xs.px();
        if hint_y > rect.y {
            paint_text(
                text_system,
                scene,
                "Read-only",
                rect.x,
                hint_y,
                TypeToken::Xs.px() - 2.0,
                rect.w,
                resolve(ColorToken::Text3, theme),
            );
        }
    }
}

fn rgba_to_hsv(rgba: [u8; 4]) -> (f32, f32, f32, f32) {
    let r = rgba[0] as f32 / 255.0;
    let g = rgba[1] as f32 / 255.0;
    let b = rgba[2] as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let v = max;
    let s = if max == 0.0 { 0.0 } else { (max - min) / max };
    let h = if (max - min).abs() < f32::EPSILON {
        0.0
    } else if (max - r).abs() < f32::EPSILON {
        ((g - b) / (max - min) % 6.0) / 6.0
    } else if (max - g).abs() < f32::EPSILON {
        ((b - r) / (max - min) + 2.0) / 6.0
    } else {
        ((r - g) / (max - min) + 4.0) / 6.0
    };
    let h = h.rem_euclid(1.0);
    (h, s, v, rgba[3] as f32 / 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let cp = BlenderColorPicker::new(NodeId(1), "Color");
        assert_eq!(cp.interpolation, InterpolationMode::Perceptual);
        assert_eq!(cp.channel_mode, ChannelMode::Rgb);
        assert_eq!(cp.palettes.len(), 1);
        assert_eq!(cp.palettes[0].name, "Default");
        assert!(!cp.palettes[0].editable);
        assert_eq!(cp.value.rgba, [231, 231, 231, 255]);
        assert!(cp.hex.starts_with('#'));
    }

    #[test]
    fn set_value_resyncs_hex() {
        let mut cp = BlenderColorPicker::new(NodeId(1), "x");
        cp.set_value(ColorValue::from_rgba8(255, 0, 0, 255));
        assert_eq!(cp.hex, "#FF0000FF");
    }

    #[test]
    fn channel_mode_swap_does_not_change_value() {
        let cp = BlenderColorPicker::new(NodeId(1), "x")
            .channel_mode(ChannelMode::Hsv)
            .value(ColorValue::from_rgba8(120, 200, 80, 255));
        assert_eq!(cp.value.rgba, [120, 200, 80, 255]);
    }

    #[test]
    fn interpolation_setter_round_trips() {
        let cp = BlenderColorPicker::new(NodeId(1), "x").interpolation(InterpolationMode::Linear);
        assert_eq!(cp.interpolation, InterpolationMode::Linear);
    }

    #[test]
    fn rgba_to_hsv_red() {
        let (h, s, v, _) = rgba_to_hsv([255, 0, 0, 255]);
        assert!(h.abs() < 1e-3 || (h - 1.0).abs() < 1e-3);
        assert!((s - 1.0).abs() < 1e-3);
        assert!((v - 1.0).abs() < 1e-3);
    }

    #[test]
    fn default_palette_has_12_swatches() {
        let p = default_palette();
        assert_eq!(p.swatches.len(), 12);
        assert!(!p.editable);
    }

    #[test]
    fn palette_constructor_defaults_editable() {
        let p = ColorPalette::new("Custom", vec![ColorValue::WHITE]);
        assert!(p.editable);
        assert_eq!(p.name, "Custom");
    }

    #[test]
    fn a11y_role_is_group() {
        let cp = BlenderColorPicker::new(NodeId(1), "Color");
        let node = cp.build_a11y(0.0, 0.0, 280.0, 540.0);
        assert_eq!(node.role(), Role::Group);
    }

    fn smoke(cp: BlenderColorPicker, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_blender_color_picker(
            &cp,
            Rect::new(0.0, 0.0, 280.0, 560.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_default() {
        smoke(BlenderColorPicker::new(NodeId(1), "x"), Theme::ForgeSdf);
    }

    #[test]
    fn paint_smoke_hsv_mode() {
        smoke(
            BlenderColorPicker::new(NodeId(1), "x")
                .channel_mode(ChannelMode::Hsv)
                .interpolation(InterpolationMode::Linear),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_red_value() {
        smoke(
            BlenderColorPicker::new(NodeId(1), "x").value(ColorValue::from_rgba8(220, 40, 40, 255)),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_with_alpha() {
        smoke(
            BlenderColorPicker::new(NodeId(1), "x").value(ColorValue::from_rgba8(40, 40, 220, 128)),
            Theme::PaintStudio,
        );
    }
}
