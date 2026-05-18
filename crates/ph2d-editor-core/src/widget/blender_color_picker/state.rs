//! `BlenderColorPicker` data + supporting enums + palettes.

use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::ColorValue;

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
        editable: true,
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
    pub hex: String,
    /// Retained HSV hue (0..1). The SV-rect / hue-strip painters
    /// read this instead of deriving from `value.rgba` so the user's
    /// chosen hue survives V→0 (pure black) collapses where RGBA
    /// loses all chromaticity.
    pub hsv_h: f32,
    /// Retained HSV saturation (0..1). Same role as `hsv_h`.
    pub hsv_s: f32,
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
            // Default hue/saturation: the gray default value has no
            // hue, so we anchor at "red, full sat" — the conventional
            // starting point for color pickers when the user hasn't
            // expressed a preference yet.
            hsv_h: 0.0,
            hsv_s: 1.0,
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

    pub fn set_value(&mut self, value: ColorValue) {
        self.value = value;
        self.sync_hex();
    }

    pub fn sync_hex(&mut self) {
        let [r, g, b, a] = self.value.rgba;
        self.hex = format!("#{r:02X}{g:02X}{b:02X}{a:02X}");
    }

    /// Pick the swatch at `(palette_index, swatch_index)` and set
    /// it as the picker's current value. No-op if either index is
    /// out of range.
    pub fn pick_swatch(&mut self, palette_index: usize, swatch_index: usize) -> bool {
        let Some(palette) = self.palettes.get(palette_index) else {
            return false;
        };
        let Some(swatch) = palette.swatches.get(swatch_index).copied() else {
            return false;
        };
        self.set_value(swatch);
        self.active_palette = palette_index;
        true
    }

    /// Try to commit a hex string into the picker's value. Returns
    /// `true` on a successful parse (and updates `value` + `hex`).
    pub fn try_set_hex(&mut self, hex: &str) -> bool {
        match super::hex_field::parse_hex(hex) {
            Some(cv) => {
                self.set_value(cv);
                true
            }
            None => false,
        }
    }

    /// Push a new swatch to the active palette. Returns the new
    /// swatch's index, or `None` if the active palette is read-only.
    pub fn add_swatch(&mut self, value: ColorValue) -> Option<usize> {
        let palette = self.palettes.get_mut(self.active_palette)?;
        if !palette.editable {
            return None;
        }
        palette.swatches.push(value);
        Some(palette.swatches.len() - 1)
    }

    /// Remove the swatch at `index` from the active palette.
    pub fn remove_swatch(&mut self, index: usize) -> bool {
        let Some(palette) = self.palettes.get_mut(self.active_palette) else {
            return false;
        };
        if !palette.editable || index >= palette.swatches.len() {
            return false;
        }
        palette.swatches.remove(index);
        true
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Group)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(true)
            .action(Action::Focus)
            .build()
    }
}
