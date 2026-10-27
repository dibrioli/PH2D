//! **As formas do Vector no Pixel** (P6b, dono 26/10): o retângulo, a elipse, o polígono e a estrela
//! do Pixel são as [`ShapeKind`] do Vector com os valores dele, na ordem dos campos do catálogo
//! (`ph2d_tool_vector::shapes::SHAPES`, que o painel lê); os campos em píxeis do Vector valem
//! CÉLULAS aqui. A família desenha-as pelas leis inteiras do `ph2d-pixel-core`.

use ph2d_vec_scene::ShapeKind;

use super::PixelTool;

/// Uma forma paramétrica do Pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelShape {
    Rectangle,
    Ellipse,
    Polygon,
    Star,
}

impl PixelShape {
    pub const ALL: [Self; 4] = [Self::Rectangle, Self::Ellipse, Self::Polygon, Self::Star];

    /// A forma do catálogo do Vector.
    #[must_use]
    pub const fn kind(self) -> ShapeKind {
        match self {
            Self::Rectangle => ShapeKind::RoundRect,
            Self::Ellipse => ShapeKind::Ellipse,
            Self::Polygon => ShapeKind::Polygon,
            Self::Star => ShapeKind::Star,
        }
    }

    /// A forma que a ferramenta desenha.
    #[must_use]
    pub const fn of_tool(t: PixelTool) -> Option<Self> {
        Some(match t {
            PixelTool::Rectangle => Self::Rectangle,
            PixelTool::Ellipse => Self::Ellipse,
            PixelTool::RegularPolygon => Self::Polygon,
            PixelTool::Star => Self::Star,
            _ => return None,
        })
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Os valores de uma forma, na ordem dos campos do catálogo — guardados em bits (o `Eq` do
/// instantâneo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelShapeValues(pub [u64; ph2d_vec_scene::MAX_SHAPE_FIELDS]);

impl PixelShapeValues {
    /// Os de omissão do Vector. O raio do round-rect dele é de MUNDO (`0,12`): no Pixel o canto
    /// nasce vivo, como no Aseprite — o painel prova que todo campo em píxeis nasce a zero.
    #[must_use]
    pub fn defaults(s: PixelShape) -> Self {
        let mut v = s.kind().defaults();
        if s == PixelShape::Rectangle {
            v[0] = 0.0;
        }
        Self(v.map(f64::to_bits))
    }

    #[must_use]
    pub fn get(&self, i: usize) -> f64 {
        self.0.get(i).map_or(0.0, |b| f64::from_bits(*b))
    }

    #[must_use]
    pub fn with(mut self, i: usize, v: f64) -> Self {
        if let Some(b) = self.0.get_mut(i) {
            *b = v.to_bits();
        }
        self
    }

    /// Os valores como o `cook` do Vector os lê.
    #[must_use]
    pub fn values(&self) -> ph2d_vec_scene::ShapeValues {
        self.0.map(f64::from_bits)
    }
}

/// Os valores das quatro formas (o Aseprite tem *Same in all Tools*; cada forma guarda os seus).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelShapes(pub [PixelShapeValues; 4]);

impl Default for PixelShapes {
    fn default() -> Self {
        Self(PixelShape::ALL.map(PixelShapeValues::defaults))
    }
}

impl PixelShapes {
    #[must_use]
    pub fn get(&self, s: PixelShape) -> PixelShapeValues {
        self.0[s.index()]
    }

    pub fn set(&mut self, s: PixelShape, v: PixelShapeValues) {
        self.0[s.index()] = v;
    }
}
