//! **A projeção por-frame do estilo** (`VectorStyleSnapshot`) — irmão de [`super::params`] pelo
//! teto de 700 LOC, e o corte é por responsabilidade: aqui mora o que a tool **PUBLICA** para o
//! painel pintar; no irmão, o que ela guarda e as conversões de unidade dos knobs.

use ph2d_vec_scene::{Marker, ShapeKind, ShapeValues};

use super::params::*;

/// Per-frame projection of the tool's Style, published by the shell bridge for
/// the docked panel to paint. `stroke` / `fill` are sRGB8; `fill[3] == 0` ⇒ no
/// fill ("None"). `mode` / `polygon_sides` drive the draw-mode segmented row +
/// the Sides slider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorStyleSnapshot {
    pub stroke: [u8; 4],
    pub fill: [u8; 4],
    pub stroke_width_px: f64,
    pub mode: DrawMode,
    /// **Blend:** cada passo nasce ACIMA do anterior (o checkbox da seção Blend). A tool é a dona;
    /// o painel só o pinta.
    pub blend_stack_up: bool,
    /// **De onde a largura do traço de lápis vem** (W1d) — o painel pinta os três chips a partir
    /// disto e não sabe o que uma fonte É; a tool é a dona.
    pub pencil_width_source: ph2d_vec_edit::pencil_width::WidthSource,
    /// A forma ATIVA do catálogo + os parâmetros dela (unidade de UI) — o painel pinta o
    /// seletor e os campos a partir disto, sem saber que formas existem.
    pub shape: ShapeKind,
    pub values: ShapeValues,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    /// Dash as a multiple of stroke width (`0` = solid).
    pub dash: f64,
    /// Gap between dashes as a multiple of stroke width.
    pub gap: f64,
    /// **Pontas do traço** (arrowheads) — a do começo e a do fim do caminho. São
    /// propriedade do STROKE (herdam cor e largura, giram com a tangente), não formas
    /// do catálogo: por isso viajam no snapshot ao lado de cap/join/dash, e o painel
    /// as pinta na seção Stroke. Aqui usamos o tipo do DOCUMENTO
    /// (`ph2d_vec_scene::Marker`) em vez de um espelho de UI — a tool já depende da
    /// crate de geometria (`ShapeKind`/`ShapeValues`), e um espelho a mais só criaria
    /// uma tabela de conversão para manter em dia.
    pub marker_start: Marker,
    pub marker_end: Marker,
    /// Tamanho da ponta ([`MARKER_SCALE`]) + arredondamento das quinas dela
    /// ([`MARKER_ROUND`]). Viajam no snapshot porque as caixas do painel são semeadas com o
    /// valor EFETIVO da tool a cada frame — um campo que nascesse em `0` daria uma seta
    /// invisível ao primeiro toque.
    pub marker_scale: f64,
    pub marker_round: f64,
}

impl VectorStyleSnapshot {
    /// **"É bidirecional?" é uma pergunta DERIVADA** — a linha tem ponta nos dois extremos.
    ///
    /// O botão Both Ends lê daqui e o clique reescreve as PONTAS (nunca um flag próprio):
    /// um booleano guardado seria uma segunda verdade, e divergiria no instante em que o
    /// usuário trocasse uma das pontas pelo chip de Start/End.
    #[must_use]
    pub fn both_ends(&self) -> bool {
        self.marker_start != Marker::None && self.marker_end != Marker::None
    }
}

impl Default for VectorStyleSnapshot {
    fn default() -> Self {
        Self {
            stroke: [240, 240, 245, 255],
            fill: [90, 150, 230, 255],
            stroke_width_px: super::tool::DEFAULT_STROKE_WIDTH_PX,
            mode: DrawMode::Pen,
            blend_stack_up: true,
            pencil_width_source: ph2d_vec_edit::pencil_width::WidthSource::default(),
            shape: ShapeKind::Rectangle,
            values: ShapeKind::Rectangle.defaults(),
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            dash: 0.0,
            gap: GAP_DEFAULT,
            marker_start: Marker::None,
            marker_end: Marker::None,
            marker_scale: DEFAULT_MARKER_SCALE,
            marker_round: DEFAULT_MARKER_ROUND,
        }
    }
}
