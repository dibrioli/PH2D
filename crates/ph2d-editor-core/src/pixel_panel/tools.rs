//! **As ferramentas do Pixel e os botões de grupo da coluna** (a lista, os atalhos, quem mora em
//! que grupo e o estado deles).

use super::{PixelShape, SHAPE_COUNT};

/// A ferramenta na mão — as do Aseprite, na ordem da barra dele. A família reexporta-a como `Tool`
/// (um dono).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelTool {
    /// Selecção (P6): Rectangular Marquee `M`, Elliptical `Shift+M`, Lasso `Q`, Polygonal Lasso
    /// `Shift+Q` (as quatro num botão de grupo), Magic Wand `W`; Transform `V` (ou `Ctrl+T`)
    /// levanta-as no gizmo do sprite, e arrastar por dentro move (P6c: o Move saiu).
    RectMarquee,
    EllipseMarquee,
    Lasso,
    PolygonLasso,
    MagicWand,
    Transform,
    #[default]
    Pencil,
    Spray,
    Eraser,
    Eyedropper,
    /// O balde (*Paint Bucket*).
    Bucket,
    Line,
    Curve,
    /// Uma forma do Vector (P6b; P6e: o catálogo inteiro), num botão de grupo (P6c); cheia pela
    /// caixa «Fill» do painel. O retângulo e a elipse são os `U`/`Shift+U` do Aseprite.
    Shape(PixelShape),
    /// Traço livre que FECHA e enche ao largar.
    Contour,
    /// Vértices por clique; fecha e enche.
    Polygon,
    /// A mão: arrastar move a vista (`Espaço` segurado faz o mesmo com qualquer ferramenta).
    Hand,
}

/// As ferramentas que não são formas, na ordem da coluna; as formas entram depois da Curva.
const PLAIN: [PixelTool; 16] = [
    PixelTool::RectMarquee,
    PixelTool::EllipseMarquee,
    PixelTool::Lasso,
    PixelTool::PolygonLasso,
    PixelTool::MagicWand,
    PixelTool::Transform,
    PixelTool::Pencil,
    PixelTool::Spray,
    PixelTool::Eraser,
    PixelTool::Eyedropper,
    PixelTool::Bucket,
    PixelTool::Line,
    PixelTool::Curve,
    PixelTool::Contour,
    PixelTool::Polygon,
    PixelTool::Hand,
];

/// Onde as formas entram em [`PixelTool::ALL`]: depois da Curva.
const SHAPES_AT: usize = 13;

impl PixelTool {
    /// O retângulo do Aseprite (`U`).
    pub const RECTANGLE: Self = Self::Shape(PixelShape::RECTANGLE);
    /// A elipse do Aseprite (`Shift+U`).
    pub const ELLIPSE: Self = Self::Shape(PixelShape::ELLIPSE);

    /// Todas, na ordem da coluna (as formas na ordem do enum do Vector; a do flyout é a do catálogo
    /// dele, que o painel lê).
    pub const ALL: [Self; PLAIN.len() + SHAPE_COUNT] = {
        let mut out = [Self::Pencil; PLAIN.len() + SHAPE_COUNT];
        let mut i = 0;
        while i < out.len() {
            out[i] = if i < SHAPES_AT {
                PLAIN[i]
            } else if i < SHAPES_AT + SHAPE_COUNT {
                Self::Shape(PixelShape::ALL[i - SHAPES_AT])
            } else {
                PLAIN[i - SHAPE_COUNT]
            };
            i += 1;
        }
        out
    };

    /// A chave i18n do nome.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::RectMarquee => "pixel.tool.rect_marquee",
            Self::EllipseMarquee => "pixel.tool.ellipse_marquee",
            Self::Lasso => "pixel.tool.lasso",
            Self::PolygonLasso => "pixel.tool.polygon_lasso",
            Self::MagicWand => "pixel.tool.magic_wand",
            Self::Transform => "pixel.tool.transform",
            Self::Pencil => "pixel.tool.pencil",
            Self::Spray => "pixel.tool.spray",
            Self::Eraser => "pixel.tool.eraser",
            Self::Eyedropper => "pixel.tool.eyedropper",
            Self::Bucket => "pixel.tool.bucket",
            Self::Line => "pixel.tool.line",
            Self::Curve => "pixel.tool.curve",
            // O nome de cada forma é o do Vector ([`PixelShape::name`]).
            Self::Shape(_) => "pixel.tool.shape",
            Self::Contour => "pixel.tool.contour",
            Self::Polygon => "pixel.tool.polygon",
            Self::Hand => "pixel.tool.hand",
        }
    }

    /// A tecla do atalho por omissão (a do Aseprite; o `V` do Move dele escolhe o Transform).
    #[must_use]
    pub const fn shortcut(self) -> &'static str {
        match self {
            Self::RectMarquee | Self::EllipseMarquee => "M",
            Self::Lasso | Self::PolygonLasso => "Q",
            Self::MagicWand => "W",
            Self::Transform => "V",
            Self::Pencil | Self::Spray => "B",
            Self::Eraser => "E",
            Self::Eyedropper => "I",
            Self::Bucket => "G",
            Self::Line | Self::Curve => "L",
            Self::RECTANGLE | Self::ELLIPSE => "U",
            Self::Contour | Self::Polygon => "D",
            Self::Hand => "H",
            // O Aseprite não as tem: sem atalho.
            Self::Shape(_) => "",
        }
    }

    /// O atalho leva `Shift` (a sub-ferramenta do Aseprite).
    #[must_use]
    pub const fn shifted(self) -> bool {
        matches!(
            self,
            Self::Spray
                | Self::Curve
                | Self::ELLIPSE
                | Self::Polygon
                | Self::EllipseMarquee
                | Self::PolygonLasso
        )
    }

    /// O atalho como o artista o lê: «B», «Shift+B».
    #[must_use]
    pub fn shortcut_label(self) -> String {
        if self.shifted() {
            ph2d_i18n::tr_with("pixel.shortcut_shift", &[("key", &self.shortcut())])
        } else {
            self.shortcut().to_owned()
        }
    }

    /// O nome: o do Aseprite, ou o do Vector para uma forma.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Shape(s) => s.name(),
            _ => ph2d_i18n::tr(self.label_key()),
        }
    }

    /// O nome com o atalho, «Pencil (B)»; a que não tem atalho, só o nome.
    #[must_use]
    pub fn named(self) -> String {
        let nome = self.name();
        if self.shortcut().is_empty() {
            return nome.to_owned();
        }
        ph2d_i18n::tr_with(
            "pixel.tool_switched",
            &[("tool", &nome), ("key", &self.shortcut_label())],
        )
    }

    /// Deposita com o pincel (tamanho/forma/ângulo valem para ela).
    #[must_use]
    pub const fn uses_brush(self) -> bool {
        matches!(
            self,
            Self::Pencil
                | Self::Spray
                | Self::Eraser
                | Self::Line
                | Self::Curve
                | Self::Shape(_)
                | Self::Contour
                | Self::Polygon
        )
    }

    /// Pinta no documento (a simetria e o ladrilho valem para ela).
    #[must_use]
    pub const fn paints(self) -> bool {
        self.uses_brush() || matches!(self, Self::Bucket)
    }

    /// O botão de grupo da coluna onde mora, se mora num.
    #[must_use]
    pub const fn group(self) -> Option<PixelToolGroup> {
        match self {
            Self::RectMarquee | Self::EllipseMarquee | Self::Lasso | Self::PolygonLasso => {
                Some(PixelToolGroup::Select)
            }
            Self::Shape(_) => Some(PixelToolGroup::Shape),
            _ => None,
        }
    }

    /// Constrói a selecção (marquee, laços, varinha).
    #[must_use]
    pub const fn selects(self) -> bool {
        matches!(
            self,
            Self::RectMarquee
                | Self::EllipseMarquee
                | Self::Lasso
                | Self::PolygonLasso
                | Self::MagicWand
        )
    }
}

/// **Um botão de grupo da coluna** (dono 26/10): as ferramentas que partilham um lugar, com a cara
/// da última usada; o clique põe-na na mão e abre (ou fecha) o flyout com todas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelToolGroup {
    /// As quatro selecções (marquee, elipse, laço, laço poligonal).
    Select,
    /// As formas do Vector (o catálogo inteiro, na ordem dele: o painel lê-a).
    Shape,
}

impl PixelToolGroup {
    pub const ALL: [Self; 2] = [Self::Select, Self::Shape];

    /// A cara de fábrica do botão.
    #[must_use]
    pub const fn first(self) -> PixelTool {
        match self {
            Self::Select => PixelTool::RectMarquee,
            Self::Shape => PixelTool::RECTANGLE,
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// **O estado dos botões de grupo**: a última usada de cada um e o flyout aberto (um de cada vez).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelGroups {
    last: [PixelTool; 2],
    pub open: Option<PixelToolGroup>,
}

impl Default for PixelGroups {
    fn default() -> Self {
        Self {
            last: PixelToolGroup::ALL.map(PixelToolGroup::first),
            open: None,
        }
    }
}

impl PixelGroups {
    /// A cara do botão: a última usada do grupo.
    #[must_use]
    pub const fn face(&self, g: PixelToolGroup) -> PixelTool {
        self.last[g.index()]
    }

    /// A ferramenta `t` foi para a mão: passa a ser a cara do grupo dela, e o flyout fecha.
    pub const fn chose(&mut self, t: PixelTool) {
        if let Some(g) = t.group() {
            self.last[g.index()] = t;
        }
        self.open = None;
    }
}
