//! Colour-palette interchange: parse / serialize the palette formats famous art apps and sites use —
//! GIMP `.gpl`, a plain hex list (coolors.co / Adobe Color web / CSS), Adobe `.ase` (Swatch Exchange)
//! and `.aco` (Color). Pure data — straight-sRGB RGBA8 swatches + a name — independent of the picker;
//! the UI layer converts to/from its own `ColorValue`. All binary formats are big-endian.

mod binary;
#[cfg(test)]
mod tests;
mod text;

/// A named palette as interchange data: straight-sRGB RGBA8 swatches + a display name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaletteData {
    /// Display name (may be empty — hex lists and most `.aco` carry none).
    pub name: String,
    /// Straight-sRGB RGBA8 swatches in author order.
    pub colors: Vec<[u8; 4]>,
}

/// The interchange formats [`PaletteData`] round-trips through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteFormat {
    /// GIMP / Inkscape / Krita / Blender `.gpl` (text).
    Gpl,
    /// One `#RRGGBB[AA]` (or `#RGB`) per line — coolors.co / Adobe Color web / CSS (text).
    HexList,
    /// Adobe Swatch Exchange `.ase` (binary).
    Ase,
    /// Adobe Color `.aco` (binary).
    Aco,
}

impl PaletteFormat {
    /// Every format, for building a UI list (import filters / export menu).
    pub const ALL: [Self; 4] = [Self::Gpl, Self::HexList, Self::Ase, Self::Aco];

    /// Pick a format from a file extension (case-insensitive, leading dot optional); `None` if
    /// unrecognised. `txt` / `css` map to the hex list (where pasted site palettes usually land).
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.trim_start_matches('.').to_ascii_lowercase().as_str() {
            "gpl" => Some(Self::Gpl),
            "hex" | "txt" | "css" => Some(Self::HexList),
            "ase" => Some(Self::Ase),
            "aco" => Some(Self::Aco),
            _ => None,
        }
    }

    /// The conventional lowercase extension (no dot).
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Self::Gpl => "gpl",
            Self::HexList => "hex",
            Self::Ase => "ase",
            Self::Aco => "aco",
        }
    }

    /// Short human label for a UI menu.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Gpl => "GIMP palette (.gpl)",
            Self::HexList => "Hex list (.hex)",
            Self::Ase => "Adobe Swatch Exchange (.ase)",
            Self::Aco => "Adobe Color (.aco)",
        }
    }
}

/// Why a palette failed to parse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteError(pub &'static str);

impl core::fmt::Display for PaletteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "malformed palette: {}", self.0)
    }
}
impl std::error::Error for PaletteError {}

/// Parse `bytes` as `format` into a [`PaletteData`].
///
/// # Errors
/// Returns [`PaletteError`] when the bytes are not valid for `format` (wrong signature, truncated,
/// or yielding zero colours).
pub fn parse(format: PaletteFormat, bytes: &[u8]) -> Result<PaletteData, PaletteError> {
    match format {
        PaletteFormat::Gpl => text::parse_gpl(bytes),
        PaletteFormat::HexList => text::parse_hex_list(bytes),
        PaletteFormat::Ase => binary::parse_ase(bytes),
        PaletteFormat::Aco => binary::parse_aco(bytes),
    }
}

/// Serialize `palette` as `format` (always succeeds — names are synthesised where a format needs one).
#[must_use]
pub fn write(format: PaletteFormat, palette: &PaletteData) -> Vec<u8> {
    match format {
        PaletteFormat::Gpl => text::write_gpl(palette),
        PaletteFormat::HexList => text::write_hex_list(palette),
        PaletteFormat::Ase => binary::write_ase(palette),
        PaletteFormat::Aco => binary::write_aco(palette),
    }
}
