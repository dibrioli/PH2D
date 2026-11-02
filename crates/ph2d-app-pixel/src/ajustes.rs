//! **Edit ▸ Adjustments do lado da família** (P8, `docs/PixelArt/P8_plano.md` §4): o pedido do
//! diálogo traduzido para `filter::adjust` — o resto (Preview, OK = um passo) é o caminho dos
//! filtros (`filtros.rs`).

use ph2d_editor_core::pixel_panel::{PixelChannels, PixelDialogChoice, PixelHueMode};
use ph2d_pixel_core::filter::Filter;
use ph2d_pixel_core::filter::adjust::{BrightnessContrast, Curve, HueMode, HueSaturation};

/// O filtro de Adjustments do pedido `c` e os canais dele (`None` = não é um ajuste).
pub(crate) fn filtro(c: PixelDialogChoice) -> Option<(Box<dyn Filter>, PixelChannels)> {
    Some(match c {
        PixelDialogChoice::BrightnessContrast {
            brightness,
            contrast,
            channels,
        } => (
            Box::new(BrightnessContrast::new(brightness, contrast)),
            channels,
        ),
        PixelDialogChoice::HueSaturation {
            mode,
            hue,
            saturation,
            lightness,
            channels,
        } => (
            Box::new(HueSaturation {
                mode: match mode {
                    PixelHueMode::Hsl => HueMode::Hsl,
                    PixelHueMode::Hsv => HueMode::Hsv,
                    PixelHueMode::HslPlus => HueMode::HslPlus,
                    PixelHueMode::HsvPlus => HueMode::HsvPlus,
                },
                hue,
                saturation,
                lightness,
                alpha: 0,
            }),
            channels,
        ),
        PixelDialogChoice::ColorCurve { curve, channels } => {
            (Box::new(Curve::new(curve.points())), channels)
        }
        _ => return None,
    })
}
