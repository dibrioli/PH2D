//! **As formas do Vector no Pixel** (P6b; P6e, dono 27/10: *«traga todas as outras shapes presentes
//! em vector para pixel»*): cada forma do Pixel É uma [`ShapeKind`] do Vector, com os valores dele, na
//! ordem dos campos do catálogo (`ph2d_tool_vector::shapes::SHAPES`, que o painel lê e que dá a ordem
//! do flyout); os campos em píxeis do Vector valem CÉLULAS aqui. Nenhuma lista de formas mora aqui:
//! a lista é [`ALL_SHAPES`].

use ph2d_vec_scene::{ALL_SHAPES, ShapeKind};

use super::PixelTool;

/// Quantas formas o Vector tem.
pub const SHAPE_COUNT: usize = ALL_SHAPES.len();

/// As gavetas dos valores: uma por discriminante (o enum do Vector é append-only e tem lacunas).
const SLOTS: usize = {
    let mut n = 0;
    let mut i = 0;
    while i < SHAPE_COUNT {
        let d = ALL_SHAPES[i] as usize + 1;
        if d > n {
            n = d;
        }
        i += 1;
    }
    n
};

/// Uma forma paramétrica do Pixel: uma forma do catálogo do Vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixelShape(ShapeKind);

impl PixelShape {
    /// O retângulo do Aseprite (`U`): o round-rect do Vector, de canto vivo.
    pub const RECTANGLE: Self = Self(ShapeKind::RoundRect);
    /// A elipse do Aseprite (`Shift+U`).
    pub const ELLIPSE: Self = Self(ShapeKind::Ellipse);
    pub const POLYGON: Self = Self(ShapeKind::Polygon);
    pub const STAR: Self = Self(ShapeKind::Star);

    /// Todas, na ordem do enum do Vector.
    pub const ALL: [Self; SHAPE_COUNT] = {
        let mut out = [Self::RECTANGLE; SHAPE_COUNT];
        let mut i = 0;
        while i < SHAPE_COUNT {
            out[i] = Self(ALL_SHAPES[i]);
            i += 1;
        }
        out
    };

    #[must_use]
    pub const fn new(kind: ShapeKind) -> Self {
        Self(kind)
    }

    /// A forma do catálogo do Vector.
    #[must_use]
    pub const fn kind(self) -> ShapeKind {
        self.0
    }

    /// A forma que a ferramenta desenha.
    #[must_use]
    pub const fn of_tool(t: PixelTool) -> Option<Self> {
        match t {
            PixelTool::Shape(s) => Some(s),
            _ => None,
        }
    }

    /// A gaveta dos valores dela (o discriminante do Vector).
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// O nome do Vector, traduzido (o painel do Vector injecta a ponte dele: [`set_shape_names`]).
    #[must_use]
    pub fn name(self) -> &'static str {
        NOMES
            .get()
            .map_or_else(|| ph2d_i18n::tr("pixel.tool.shape"), |f| f(self.0))
    }
}

/// O nome de cada forma, como o Vector o pinta — uma ponte injectada pelo registo dos painéis (o
/// núcleo não vê o catálogo nem a tradução do Vector, e uma 2.ª tabela de nomes divergiria).
static NOMES: std::sync::OnceLock<fn(ShapeKind) -> &'static str> = std::sync::OnceLock::new();

/// Injecta a ponte dos nomes das formas.
pub fn set_shape_names(f: fn(ShapeKind) -> &'static str) {
    let _ = NOMES.set(f);
}

/// Os nomes dos CAMPOS do catálogo como o painel os pinta (a etiqueta do arrasto da forma viva fala
/// como o painel) — a mesma ponte injectada, pela mesma razão que [`NOMES`].
static CAMPOS: std::sync::OnceLock<fn(&'static str) -> &'static str> = std::sync::OnceLock::new();

/// Injecta a ponte dos nomes dos campos.
pub fn set_field_names(f: fn(&'static str) -> &'static str) {
    let _ = CAMPOS.set(f);
}

/// O nome a pintar do rótulo de campo `label` (sem ponte, o rótulo do motor).
#[must_use]
pub fn field_name(label: &'static str) -> &'static str {
    CAMPOS.get().map_or(label, |f| f(label))
}

/// Os valores de uma forma, na ordem dos campos do catálogo — guardados em bits (o `Eq` do
/// instantâneo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelShapeValues(pub [u64; ph2d_vec_scene::MAX_SHAPE_FIELDS]);

impl PixelShapeValues {
    /// Os de omissão do Vector, salvo os comprimentos de MUNDO: o raio que o round-rect e o balão
    /// dele trazem ([`DEFAULT_CORNER_RADIUS`](ph2d_vec_scene::DEFAULT_CORNER_RADIUS), no campo 0)
    /// nasce a zero — no Pixel o canto nasce vivo, como no Aseprite. O painel prova que todo campo
    /// em píxeis de TODA forma nasce a zero (o que apanha uma 3.ª forma que traga um).
    #[must_use]
    pub fn defaults(s: PixelShape) -> Self {
        let mut v = s.kind().defaults();
        if matches!(s.kind(), ShapeKind::RoundRect | ShapeKind::SpeechRect) {
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

/// Os valores de cada forma (o Aseprite tem *Same in all Tools*; cada forma guarda os seus), uma
/// gaveta por discriminante.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelShapes(pub [PixelShapeValues; SLOTS]);

impl Default for PixelShapes {
    fn default() -> Self {
        let mut v = [PixelShapeValues([0; ph2d_vec_scene::MAX_SHAPE_FIELDS]); SLOTS];
        for s in PixelShape::ALL {
            v[s.index()] = PixelShapeValues::defaults(s);
        }
        Self(v)
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
