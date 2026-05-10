//! [`ColorPicker`] — multi-mode color selection panel.
//!
//! Mockup screen 08-color-picker.html shows 5 modes (Disc, Classic,
//! Harmony, Value, Palettes). v1 ships only the tab structure +
//! Classic mode (RGB and HSL sliders); other modes are stubs that
//! paint an "TODO" placeholder so the layout is testable.

use super::slider::{Slider, paint_slider};
use super::tabs::{TabItem, Tabs, TabsVariant, paint_tabs};
use crate::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Color as VelloColor, VectorScene};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorPickerMode {
    Disc,
    Classic,
    Harmony,
    Value,
    Palettes,
}

#[derive(Clone, Debug)]
pub struct ColorPicker {
    pub id: NodeId,
    pub label: String,
    pub rgba: [u8; 4],
    pub mode: ColorPickerMode,
    pub red: Slider,
    pub green: Slider,
    pub blue: Slider,
    pub hue: Slider,
    pub saturation: Slider,
    pub lightness: Slider,
    pub tabs: Tabs,
}

impl ColorPicker {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        let tabs = Tabs::new(
            NodeId(id.0 + 1),
            "Mode",
            vec![
                TabItem::new(NodeId(id.0 + 2), "Disc"),
                TabItem::new(NodeId(id.0 + 3), "Classic"),
                TabItem::new(NodeId(id.0 + 4), "Harmony"),
                TabItem::new(NodeId(id.0 + 5), "Value"),
                TabItem::new(NodeId(id.0 + 6), "Palettes"),
            ],
        )
        .selected(1)
        .variant(TabsVariant::Segmented);
        let mut s = Self {
            id,
            label: label.into(),
            rgba: [128, 128, 128, 255],
            mode: ColorPickerMode::Classic,
            red: Slider::new(NodeId(id.0 + 10), "R").accent(true),
            green: Slider::new(NodeId(id.0 + 11), "G").accent(true),
            blue: Slider::new(NodeId(id.0 + 12), "B").accent(true),
            hue: Slider::new(NodeId(id.0 + 13), "H").accent(true),
            saturation: Slider::new(NodeId(id.0 + 14), "S").accent(true),
            lightness: Slider::new(NodeId(id.0 + 15), "L").accent(true),
            tabs,
        };
        s.sync_sliders_from_rgba();
        s
    }

    pub fn rgba(mut self, rgba: [u8; 4]) -> Self {
        self.rgba = rgba;
        self.sync_sliders_from_rgba();
        self
    }

    pub fn mode(mut self, mode: ColorPickerMode) -> Self {
        self.mode = mode;
        let idx = match mode {
            ColorPickerMode::Disc => 0,
            ColorPickerMode::Classic => 1,
            ColorPickerMode::Harmony => 2,
            ColorPickerMode::Value => 3,
            ColorPickerMode::Palettes => 4,
        };
        self.tabs = self.tabs.clone().selected(idx);
        self
    }

    fn sync_sliders_from_rgba(&mut self) {
        self.red.set_value(self.rgba[0] as f32 / 255.0);
        self.green.set_value(self.rgba[1] as f32 / 255.0);
        self.blue.set_value(self.rgba[2] as f32 / 255.0);
        let (h, s, l) = rgb_to_hsl(self.rgba[0], self.rgba[1], self.rgba[2]);
        self.hue.set_value(h);
        self.saturation.set_value(s);
        self.lightness.set_value(l);
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Group)
            .label(&self.label)
            .bounds(x, y, w, h)
            .child(self.tabs.id)
            .build()
    }
}

/// Approximate RGB → HSL using the standard piecewise formula. Output
/// each channel in [0, 1].
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if (max - r).abs() < f32::EPSILON {
        ((g - b) / d) % 6.0
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    let h = ((h * 60.0).rem_euclid(360.0)) / 360.0;
    (h, s, l)
}

pub fn paint_color_picker(
    cp: &ColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Lg.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let pad = Spacing::Lg.px();
    let tab_h = 32.0;
    let preview_h = 56.0;
    let tabs_rect = Rect::new(rect.x + pad, rect.y + pad, rect.w - pad * 2.0, tab_h);
    paint_tabs(&cp.tabs, tabs_rect, scene, text_system, theme);

    let preview_rect = Rect::new(
        rect.x + pad,
        tabs_rect.y + tabs_rect.h + Spacing::Md.px(),
        rect.w - pad * 2.0,
        preview_h,
    );
    fill_rounded_rect(
        scene,
        preview_rect,
        Radius::Md.px(),
        VelloColor::from_rgba8(cp.rgba[0], cp.rgba[1], cp.rgba[2], cp.rgba[3]),
    );
    stroke_rounded_rect(
        scene,
        preview_rect,
        Radius::Md.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );

    let body_y = preview_rect.y + preview_rect.h + Spacing::Lg.px();
    let body_rect = Rect::new(
        rect.x + pad,
        body_y,
        rect.w - pad * 2.0,
        (rect.y + rect.h - body_y - pad).max(0.0),
    );

    match cp.mode {
        ColorPickerMode::Classic => paint_classic(cp, body_rect, scene, text_system, theme),
        _ => paint_placeholder(cp.mode, body_rect, scene, text_system, theme),
    }
}

fn paint_classic(
    cp: &ColorPicker,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let row_h = 24.0;
    let gap = Spacing::Md.px();
    let label_w = 18.0;
    let labels = [
        ("R", &cp.red),
        ("G", &cp.green),
        ("B", &cp.blue),
        ("H", &cp.hue),
        ("S", &cp.saturation),
        ("L", &cp.lightness),
    ];
    let font = TypeToken::Sm.px();
    for (i, (label, slider)) in labels.iter().enumerate() {
        let y = rect.y + (row_h + gap) * i as f32;
        if y + row_h > rect.y + rect.h {
            break;
        }
        let label_rect = Rect::new(rect.x, y, label_w, row_h);
        paint_text_centered(
            text_system,
            scene,
            label,
            label_rect,
            font,
            resolve(ColorToken::Text2, theme),
        );
        let slider_rect = Rect::new(
            rect.x + label_w + gap,
            y,
            (rect.w - label_w - gap).max(0.0),
            row_h,
        );
        paint_slider(slider, slider_rect, scene, theme);
    }
}

fn paint_placeholder(
    mode: ColorPickerMode,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    fill_rounded_rect(
        scene,
        rect,
        Radius::Md.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let label = match mode {
        ColorPickerMode::Disc => "Disc — coming soon",
        ColorPickerMode::Harmony => "Harmony — coming soon",
        ColorPickerMode::Value => "Value — coming soon",
        ColorPickerMode::Palettes => "Palettes — coming soon",
        ColorPickerMode::Classic => "Classic",
    };
    paint_text(
        text_system,
        scene,
        label,
        rect.x + Spacing::Lg.px(),
        rect.y + (rect.h - TypeToken::Base.px()) * 0.5,
        TypeToken::Base.px(),
        rect.w,
        resolve(ColorToken::Text3, theme),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ColorPicker {
        ColorPicker::new(NodeId(1), "Color")
    }

    #[test]
    fn defaults_match_spec() {
        let c = fixture();
        assert_eq!(c.rgba, [128, 128, 128, 255]);
        assert_eq!(c.mode, ColorPickerMode::Classic);
        assert_eq!(c.tabs.selected, 1);
    }

    #[test]
    fn rgba_setter_syncs_rgb_sliders() {
        let c = fixture().rgba([255, 0, 0, 255]);
        assert!((c.red.value - 1.0).abs() < f32::EPSILON);
        assert_eq!(c.green.value, 0.0);
        assert_eq!(c.blue.value, 0.0);
    }

    #[test]
    fn mode_setter_swaps_tabs_selection() {
        let c = fixture().mode(ColorPickerMode::Palettes);
        assert_eq!(c.tabs.selected, 4);
    }

    #[test]
    fn rgb_to_hsl_red_yields_zero_hue_full_sat() {
        let (h, s, _) = rgb_to_hsl(255, 0, 0);
        assert!(h.abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
    }

    #[test]
    fn rgb_to_hsl_grey_zero_sat() {
        let (_, s, _) = rgb_to_hsl(128, 128, 128);
        assert!(s < 0.01);
    }

    #[test]
    fn a11y_role_is_group() {
        let node = fixture().build_a11y(0.0, 0.0, 320.0, 360.0);
        assert_eq!(node.role(), Role::Group);
    }

    fn smoke(c: ColorPicker, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_color_picker(
            &c,
            Rect::new(0.0, 0.0, 320.0, 360.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_classic() {
        smoke(fixture().rgba([240, 32, 96, 255]), Theme::ForgeSdf);
    }

    #[test]
    fn paint_smoke_palettes_placeholder() {
        smoke(fixture().mode(ColorPickerMode::Palettes), Theme::Sunstone);
    }
}
