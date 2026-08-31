//! **A projeção por-frame do estilo** (`VectorStyleSnapshot`) — irmão de [`super::params`] pelo
//! teto de 700 LOC, e o corte é por responsabilidade: aqui mora o que a tool **PUBLICA** para o
//! painel pintar; no irmão, o que ela guarda e as conversões de unidade dos knobs.

use ph2d_vec_scene::{Marker, ShapeKind, StrokeAlign};

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
    /// **O estilo da SIMETRIA de desenho** (W6.3) — o painel pinta os chips a partir disto e não
    /// sabe o que um espelho É; a tool é a dona.
    pub symmetry: ph2d_symmetry::SymmetryStyle,
    /// **A forma PEGAJOSA do marquee** (`Box | Lasso`) — o painel pinta os dois chips a partir
    /// disto. A tool é a dona; o Ctrl do gesto compõe com ela na shell.
    pub marquee: MarqueeShape,
    /// A forma ATIVA do catálogo — o painel pinta o seletor a partir disto, sem saber que formas
    /// existem.
    ///
    /// ⛔⛔ **E os VALORES dela NÃO viajam aqui.** Havia um `values: ShapeValues` ao lado, e este
    /// doc-comment prometia que *"o painel pinta os campos a partir disto"* — **falso**: os campos
    /// saem do `WidgetStore`, semeados pela shell (`vec_shape_params::seed_shape_fields`). Medido em
    /// 2026-08-31: **zero leitores no produto**, e o único que existia era um **gate** a afirmar o
    /// round-trip dele. *Uma declaração com um default é decoração até alguém a ler* — e um teste
    /// que afirma o round-trip de um campo que nenhum produto consome não protege nada: ele DEFENDE
    /// a decoração, e é o que faz a próxima pessoa mantê-la em sincronia com a fonte de verdade.
    ///
    /// ⚠️ ⛔ **Não confundir com o `VectorDrawConfig::values`**, que é vivo e é o que o GESTO cozinha
    /// — aquele sai do kind EFECTIVO do modo (`DrawMode::shape_kind`), não do botão aceso.
    pub shape: ShapeKind,
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
    /// **De que lado da linha a faixa cai** — Centre/Inner/Outer.
    ///
    /// Tipo do DOCUMENTO (`ph2d_vec_scene::StrokeAlign`), pelo mesmo motivo das PONTAS logo
    /// acima: um espelho de UI a mais só criaria uma tabela de conversão para manter em dia.
    /// (`cap`/`join` são os espelhos ANTIGOS, e é por isso que o `vector_bridge` ainda carrega
    /// um `match` de três braços para cada um.)
    pub align: StrokeAlign,
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
            symmetry: ph2d_symmetry::SymmetryStyle::default(),
            marquee: MarqueeShape::default(),
            shape: ShapeKind::Rectangle,
            cap: StrokeCap::Butt,
            align: StrokeAlign::Centre,
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
