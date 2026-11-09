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
    /// P11: a Slice Tool (`Shift+C`): cria, seleciona e edita as slices na tela.
    Slice,
    #[default]
    Pencil,
    Spray,
    Eraser,
    Eyedropper,
    /// O balde (*Paint Bucket*).
    Bucket,
    /// P8: o Gradient (`Shift+P`, no botão de grupo do balde): arrasta-se de A a B, fg → bg, na
    /// região que o balde pintaria.
    Gradient,
    /// P8: Blur e Jumble (`R` alterna), ferramentas de pincel num botão de grupo.
    Blur,
    Jumble,
    Line,
    Curve,
    /// Uma forma do Vector (P6b; P6e: o catálogo inteiro), num botão de grupo (P6c); cheia pela
    /// caixa «Fill» do painel. O retângulo e a elipse são os `U`/`Shift+U` do Aseprite.
    Shape(PixelShape),
    /// Traço livre que FECHA e enche ao largar.
    Contour,
    /// Vértices por clique; fecha e enche.
    Polygon,
    /// P8: o Text (`T`): o clique abre a caixa de texto na célula; Enter carimba os píxeis.
    Text,
    /// A mão: arrastar move a vista (`Espaço` segurado faz o mesmo com qualquer ferramenta).
    Hand,
}

/// As ferramentas que não são formas, na ordem da coluna; as formas entram depois da Curva.
const PLAIN: [PixelTool; 21] = [
    PixelTool::RectMarquee,
    PixelTool::EllipseMarquee,
    PixelTool::Lasso,
    PixelTool::PolygonLasso,
    PixelTool::MagicWand,
    PixelTool::Transform,
    PixelTool::Slice,
    PixelTool::Pencil,
    PixelTool::Spray,
    PixelTool::Eraser,
    PixelTool::Eyedropper,
    PixelTool::Bucket,
    PixelTool::Gradient,
    PixelTool::Blur,
    PixelTool::Jumble,
    PixelTool::Line,
    PixelTool::Curve,
    PixelTool::Contour,
    PixelTool::Polygon,
    PixelTool::Text,
    PixelTool::Hand,
];

/// Onde as formas entram em [`PixelTool::ALL`]: depois da Curva.
const SHAPES_AT: usize = 17;

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
            Self::Slice => "pixel.tool.slice",
            Self::Pencil => "pixel.tool.pencil",
            Self::Spray => "pixel.tool.spray",
            Self::Eraser => "pixel.tool.eraser",
            Self::Eyedropper => "pixel.tool.eyedropper",
            Self::Bucket => "pixel.tool.bucket",
            Self::Gradient => "pixel.tool.gradient",
            Self::Blur => "pixel.tool.blur",
            Self::Jumble => "pixel.tool.jumble",
            Self::Line => "pixel.tool.line",
            Self::Curve => "pixel.tool.curve",
            // O nome de cada forma é o do Vector ([`PixelShape::name`]).
            Self::Shape(_) => "pixel.tool.shape",
            Self::Contour => "pixel.tool.contour",
            Self::Polygon => "pixel.tool.polygon",
            Self::Text => "pixel.tool.text",
            Self::Hand => "pixel.tool.hand",
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

    /// O nome com o atalho do mapa VIVO, «Pencil (B)»; a que não tem tecla, só o nome.
    #[must_use]
    pub fn named(self, keys: &PixelToolKeys) -> String {
        let nome = self.name();
        match keys.of(self) {
            Some(key) => {
                ph2d_i18n::tr_with("pixel.tool_switched", &[("tool", &nome), ("key", &key)])
            }
            None => nome.to_owned(),
        }
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
                | Self::Blur
                | Self::Jumble
        )
    }

    /// Pinta no documento (a simetria e o ladrilho valem para ela).
    #[must_use]
    pub const fn paints(self) -> bool {
        self.uses_brush() || matches!(self, Self::Bucket | Self::Gradient | Self::Text)
    }

    /// O botão de grupo da coluna onde mora, se mora num.
    #[must_use]
    pub const fn group(self) -> Option<PixelToolGroup> {
        match self {
            Self::RectMarquee | Self::EllipseMarquee | Self::Lasso | Self::PolygonLasso => {
                Some(PixelToolGroup::Select)
            }
            Self::Shape(_) => Some(PixelToolGroup::Shape),
            Self::Bucket | Self::Gradient => Some(PixelToolGroup::Bucket),
            Self::Blur | Self::Jumble => Some(PixelToolGroup::Smudge),
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
    /// O balde e o Gradient (P8).
    Bucket,
    /// Blur e Jumble (P8).
    Smudge,
}

impl PixelToolGroup {
    pub const ALL: [Self; 4] = [Self::Select, Self::Shape, Self::Bucket, Self::Smudge];

    /// A cara de fábrica do botão.
    #[must_use]
    pub const fn first(self) -> PixelTool {
        match self {
            Self::Select => PixelTool::RectMarquee,
            Self::Shape => PixelTool::RECTANGLE,
            Self::Bucket => PixelTool::Bucket,
            Self::Smudge => PixelTool::Blur,
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// **O estado dos botões de grupo**: a última usada de cada um e o flyout aberto (um de cada vez).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelGroups {
    last: [PixelTool; PixelToolGroup::ALL.len()],
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

/// **As teclas das ferramentas no mapa VIVO do artista**, como ele as lê («B», «Shift+C»). A família
/// deriva-as do `Keymap` de `ph2d-pixel-keys` a cada quadro (`ph2d_pixel_keys::tool_keys`): não há
/// segunda tabela de teclas aqui.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PixelToolKeys(pub Vec<(PixelTool, String)>);

impl PixelToolKeys {
    /// A primeira tecla da ferramenta, se o mapa lhe dá alguma.
    #[must_use]
    pub fn of(&self, t: PixelTool) -> Option<&str> {
        self.0
            .iter()
            .find(|(x, _)| *x == t)
            .map(|(_, k)| k.as_str())
    }
}
