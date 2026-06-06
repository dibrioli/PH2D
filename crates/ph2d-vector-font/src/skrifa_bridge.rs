//! skrifa bridge (ADR-0066 §2.1): parse a variable font and produce an
//! axis-aware [`GlyphOutline`] for a glyph at given axis values. The **only**
//! module that touches skrifa — everything downstream consumes the neutral
//! [`GlyphOutline`]/[`PathCommand`] boundary (decoupling, ADR-0066 §2.1 file
//! split).

use core::fmt;

use glam::Vec2;
use skrifa::instance::{Location, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::{FontRef, MetadataProvider};
use smallvec::SmallVec;

use crate::axis::{AxisTag, FontAxis};
use crate::glyph_to_network::{GlyphOutline, PathCommand};
use crate::{GlyphId, MAX_AXES};

/// A font could not be parsed or a glyph could not be drawn.
#[derive(Debug, Clone, PartialEq)]
pub enum FontFaceError {
    /// The bytes are not a readable font.
    Parse(String),
    /// The glyph id has no outline (e.g. a bitmap-only or missing glyph).
    NoOutline,
    /// skrifa failed to draw the outline.
    Draw(String),
}

impl fmt::Display for FontFaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontFaceError::Parse(e) => write!(f, "font parse error: {e}"),
            FontFaceError::NoOutline => write!(f, "glyph has no outline"),
            FontFaceError::Draw(e) => write!(f, "outline draw error: {e}"),
        }
    }
}

impl std::error::Error for FontFaceError {}

/// An owned, parsed variable font. Holds the raw bytes and re-borrows a
/// `FontRef` per call (skrifa's reader is a zero-copy borrow).
pub struct VariableFont {
    data: Vec<u8>,
    units_per_em: u16,
}

impl VariableFont {
    /// Parse owned font bytes (`.ttf`/`.otf`). Validates the header up front so
    /// later borrows are infallible.
    pub fn new(data: Vec<u8>) -> Result<Self, FontFaceError> {
        let units_per_em = {
            let font = FontRef::new(&data).map_err(|e| FontFaceError::Parse(e.to_string()))?;
            font.head()
                .map(|h| h.units_per_em())
                .map_err(|e| FontFaceError::Parse(e.to_string()))?
        };
        Ok(Self {
            data,
            units_per_em,
        })
    }

    fn font(&self) -> FontRef<'_> {
        FontRef::new(&self.data).expect("validated in VariableFont::new")
    }

    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    /// The variable axes the font exposes (≤ [`MAX_AXES`] inline).
    pub fn axes(&self) -> SmallVec<[FontAxis; MAX_AXES]> {
        let font = self.font();
        font.axes()
            .iter()
            .map(|a| {
                let tag = AxisTag::new(a.tag().to_be_bytes());
                FontAxis::new(
                    axis_name(tag),
                    tag,
                    a.min_value(),
                    a.default_value(),
                    a.max_value(),
                )
            })
            .collect()
    }

    /// The glyph id for a character via the font's `cmap`, if present.
    pub fn glyph_for_char(&self, ch: char) -> Option<GlyphId> {
        self.font().charmap().map(ch).map(|g| GlyphId(g.to_u32()))
    }

    /// Draw `glyph` at the given axis values into a neutral [`GlyphOutline`]
    /// (font design units, y-up). Unset axes use their default.
    pub fn outline(
        &self,
        glyph: GlyphId,
        axis_values: &[(AxisTag, f32)],
    ) -> Result<GlyphOutline, FontFaceError> {
        let font = self.font();
        let gid = skrifa::GlyphId::new(glyph.0);
        let outlines = font.outline_glyphs();
        let outline = outlines.get(gid).ok_or(FontFaceError::NoOutline)?;

        // Axis settings as (tag-string, value); skrifa normalizes to f2dot14.
        let settings: Vec<(String, f32)> = axis_values
            .iter()
            .map(|(t, v)| (t.as_str().to_owned(), *v))
            .collect();
        let location: Location = font
            .axes()
            .location(settings.iter().map(|(t, v)| (t.as_str(), *v)));

        let mut pen = CommandPen::default();
        let draw = DrawSettings::unhinted(Size::unscaled(), &location);
        outline
            .draw(draw, &mut pen)
            .map_err(|e| FontFaceError::Draw(e.to_string()))?;
        Ok(GlyphOutline::new(pen.commands, self.units_per_em))
    }
}

/// Collects skrifa pen calls into neutral [`PathCommand`]s.
#[derive(Default)]
struct CommandPen {
    commands: Vec<PathCommand>,
}

impl OutlinePen for CommandPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::MoveTo(Vec2::new(x, y)));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::LineTo(Vec2::new(x, y)));
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::QuadTo {
            ctrl: Vec2::new(cx0, cy0),
            to: Vec2::new(x, y),
        });
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::CurveTo {
            c1: Vec2::new(cx0, cy0),
            c2: Vec2::new(cx1, cy1),
            to: Vec2::new(x, y),
        });
    }
    fn close(&mut self) {
        self.commands.push(PathCommand::Close);
    }
}

/// Friendly name for the registered axes; custom axes fall back to the tag.
fn axis_name(tag: AxisTag) -> String {
    match tag {
        AxisTag::WEIGHT => "Weight",
        AxisTag::WIDTH => "Width",
        AxisTag::SLANT => "Slant",
        AxisTag::OPTICAL_SIZE => "Optical Size",
        AxisTag::ITALIC => "Italic",
        AxisTag::GRADE => "Grade",
        _ => return tag.as_str().to_owned(),
    }
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::VariableFontAxis;
    use crate::glyph_to_network::outline_to_network_em;

    // The variable font fixture already vendored for ph2d-text (read-only).
    fn inter() -> VariableFont {
        let data = include_bytes!("../../ph2d-text/fonts/InterVariable.ttf").to_vec();
        VariableFont::new(data).expect("InterVariable parses")
    }

    #[test]
    fn inter_exposes_weight_axis() {
        let font = inter();
        let axes = font.axes();
        assert!(
            axes.iter().any(|a| a.tag() == AxisTag::WEIGHT),
            "InterVariable must expose wght; got {:?}",
            axes.iter().map(|a| a.tag()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn glyph_a_solves_to_a_valid_network() {
        let font = inter();
        let gid = font.glyph_for_char('A').expect("'A' has a glyph");
        let outline = font.outline(gid, &[(AxisTag::WEIGHT, 400.0)]).unwrap();
        let net = outline_to_network_em(&outline);
        assert!(net.validate().is_ok(), "glyph network must be valid");
        assert!(!net.regions.is_empty(), "'A' has fillable contours");
    }

    #[test]
    fn weight_axis_changes_the_outline() {
        // The whole point of ADR-0066: the axis drives the *vector geometry*.
        let font = inter();
        let gid = font.glyph_for_char('a').unwrap();
        let light = outline_to_network_em(&font.outline(gid, &[(AxisTag::WEIGHT, 100.0)]).unwrap());
        let bold = outline_to_network_em(&font.outline(gid, &[(AxisTag::WEIGHT, 900.0)]).unwrap());
        assert_ne!(light, bold, "weight must vary the glyph network");
    }
}
