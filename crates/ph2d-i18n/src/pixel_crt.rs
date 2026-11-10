//! **As palavras da CRT da Preview do Pixel** (P13 V/U-CRT): a vista de TV antiga, nunca o ficheiro.

/// A tradução de uma chave daqui, ou `None`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.pixel.preview.crt" => "CRT",
        "panel.pixel.preview.crt_tip" => {
            "Show the preview like an old TV screen. Only the view changes, never your drawing."
        }
        "panel.pixel.preview.crt.scanlines" => "Scanlines",
        "panel.pixel.preview.crt.mask" => "Mask",
        "panel.pixel.preview.crt.curvature" => "Curvature",
        "panel.pixel.preview.crt.brightness" => "Brightness",
        "panel.pixel.preview.crt.bloom" => "Bloom",
        _ => return None,
    })
}
