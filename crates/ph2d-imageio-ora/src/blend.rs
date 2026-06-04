use ph2d_imageio::BlendMode;

/// Map ORA `composite-op` attribute (15 `svg:*` enum values per
/// OpenRaster spec §6) to our [`BlendMode`]. Unknown ORA strings fall
/// through to `Normal`.
pub(crate) fn parse_blend_mode(svg_op: &str) -> BlendMode {
    match svg_op {
        "svg:src-over" => BlendMode::Normal,
        "svg:multiply" => BlendMode::Multiply,
        "svg:screen" => BlendMode::Screen,
        "svg:overlay" => BlendMode::Overlay,
        "svg:darken" => BlendMode::Darken,
        "svg:lighten" => BlendMode::Lighten,
        "svg:color-dodge" => BlendMode::ColorDodge,
        "svg:color-burn" => BlendMode::ColorBurn,
        "svg:hard-light" => BlendMode::HardLight,
        "svg:soft-light" => BlendMode::SoftLight,
        "svg:difference" => BlendMode::Difference,
        "svg:color" => BlendMode::Color,
        "svg:luminosity" => BlendMode::Luminosity,
        "svg:hue" => BlendMode::Hue,
        "svg:saturation" => BlendMode::Saturation,
        _ => BlendMode::Normal,
    }
}

/// Reverse of [`parse_blend_mode`]. Non-ORA modes export as
/// `svg:src-over` (documented loss; spec-extensibility means
/// downstream tools may ignore custom `composite-op` strings).
pub(crate) fn write_blend_mode(b: BlendMode) -> &'static str {
    match b {
        BlendMode::Normal => "svg:src-over",
        BlendMode::Multiply => "svg:multiply",
        BlendMode::Screen => "svg:screen",
        BlendMode::Overlay => "svg:overlay",
        BlendMode::Darken => "svg:darken",
        BlendMode::Lighten => "svg:lighten",
        BlendMode::ColorDodge => "svg:color-dodge",
        BlendMode::ColorBurn => "svg:color-burn",
        BlendMode::HardLight => "svg:hard-light",
        BlendMode::SoftLight => "svg:soft-light",
        BlendMode::Difference => "svg:difference",
        BlendMode::Color => "svg:color",
        BlendMode::Luminosity => "svg:luminosity",
        BlendMode::Hue => "svg:hue",
        BlendMode::Saturation => "svg:saturation",
        // Modes not in ORA spec — fall back to Normal with documented
        // loss. PSD round-trip via ORA loses these; round-trip via
        // .ph2d-native preserves byte-exact.
        _ => "svg:src-over",
    }
}
