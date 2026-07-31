//! Vector-tool Style UI vocabulary — the snapshot the docked
//! `ph2d-panel-vector` paints, plus the Width slider ↔ px mapping shared by
//! the panel (populate/paint) and the tool (`handle_panel_event`).
//!
//! Mirrors `ph2d_tool_padding::params`: the tool owns the authoritative Style,
//! projects it into a [`VectorStyleSnapshot`] each frame (published by the
//! shell bridge → the panel reads it), and both sides agree on the affine
//! slider mapping so a drag and the tool stay in lock-step.

use crate::shapes::{FieldDesc, FieldUnit};
use ph2d_vec_scene::{Marker, ShapeKind, ShapeValues};

/// **Head Size** — o tamanho da ponta, como MÚLTIPLO do que a largura do traço já dita
/// (`1.0` = o default). Um múltiplo, e não um tamanho absoluto: a ponta cresce com o traço,
/// senão engrossar a linha encolheria a seta visualmente até virar um cotoco.
pub const MARKER_SCALE: FieldDesc = FieldDesc {
    label: "Head Size",
    min: 0.25,
    max: 4.0,
    step: 0.05,
    unit: FieldUnit::Ratio,
};

/// **Head Round** — o arredondamento das quinas da PRÓPRIA ponta (`0` = afiada, `1` = o
/// máximo que a geometria dela comporta sem se descaracterizar).
pub const MARKER_ROUND: FieldDesc = FieldDesc {
    label: "Head Round",
    min: 0.0,
    max: 1.0,
    step: 0.01,
    unit: FieldUnit::Ratio,
};

/// Defaults das duas (o que o `StrokeSpec` já usa: ponta de tamanho natural, quinas vivas).
pub const DEFAULT_MARKER_SCALE: f64 = 1.0;
pub const DEFAULT_MARKER_ROUND: f64 = 0.0;

/// A ponta que a "dupla via" acende quando NENHUMA das duas existe — a seta canônica.
pub const DEFAULT_BOTH_ENDS_MARKER: Marker = Marker::Triangle;

/// Os dois rótulos do botão **Both Ends** (a "dupla via"), indexados pelo estado DERIVADO.
pub const BOTH_ENDS_NAMES: &[&str] = &["Off", "On"];

/// **Both Ends** — a dupla via, como o painel a rotula. Uma ESCOLHA de dois estados (não um
/// número): o botão mostra o corrente e o clique alterna, como o Route do conector.
///
/// O estado NÃO é guardado em lugar nenhum — é derivado das duas pontas
/// ([`VectorStyleSnapshot::both_ends`]). Este descritor é só o vocabulário da linha.
pub const BOTH_ENDS: FieldDesc = FieldDesc {
    label: "Both Ends",
    min: 0.0,
    max: 1.0,
    step: 1.0,
    unit: FieldUnit::Choice(BOTH_ENDS_NAMES),
};

/// O rótulo que o botão Both Ends mostra para o estado derivado `on`.
#[must_use]
pub fn both_ends_label(on: bool) -> &'static str {
    BOTH_ENDS_NAMES.get(usize::from(on)).copied().unwrap_or("")
}

/// Minimum / maximum stroke width in screen pixels (inclusive range the Width
/// slider spans).
/// **ZERO é alcançável, e significa SEM TRAÇO** (Enio, 2026-07-16).
///
/// Era `1.0`, e o slider batia numa parede: arrastar até o fim deixava uma linha de 1px que o
/// artista não pediu. `0` = sem traço é o Illustrator (stroke weight 0), e é o que o olho espera
/// de um slider que chega ao fim.
///
/// **O renderer já honra isso sem uma linha de código**: o Vello não encoda um traço de largura 0
/// (medido — `stroke_zero_tests`), então o `0` some de verdade em vez de virar hairline. Esse gate
/// não é cerimônia: sem ele, um `max(width, 0.5)` posto lá dentro um dia faria este `0` mentir, em
/// silêncio, e o slider prometeria uma coisa e entregaria outra.
///
/// Consequência assumida: há **duas** portas para "sem traço" — este zero e a swatch None. Elas não
/// divergem no que a TELA mostra (as duas não desenham nada); divergem no que o documento guarda
/// (`StrokeSpec{width:0}` preserva a COR, `None` a esquece). É o que faz o zero ser reversível:
/// arrastar de volta devolve o traço que estava lá.
pub const WIDTH_MIN_PX: f64 = 0.0;
pub const WIDTH_MAX_PX: f64 = 20.0;

/// Affine slider mapping `display_px = track * SCALE + OFFSET` (track `0..=1`),
/// consumed by `WidgetStore::link_slider_number_mapped` so the px chip mirrors
/// the slider. `SCALE = MAX - MIN`, `OFFSET = MIN`.
pub const WIDTH_SLIDER_SCALE: f32 = (WIDTH_MAX_PX - WIDTH_MIN_PX) as f32;
pub const WIDTH_SLIDER_OFFSET: f32 = WIDTH_MIN_PX as f32;

/// Normalized slider track `0..=1` → stroke width px `MIN..=MAX`.
#[must_use]
pub fn slider_to_px(track: f32) -> f64 {
    WIDTH_MIN_PX + f64::from(track.clamp(0.0, 1.0)) * (WIDTH_MAX_PX - WIDTH_MIN_PX)
}

/// Stroke width px → normalized slider track `0..=1` (inverse of
/// [`slider_to_px`]). Used to seed the slider knob from the tool's authoritative
/// width so it renders correctly before the first drag.
#[must_use]
pub fn px_to_slider(px: f64) -> f32 {
    (((px - WIDTH_MIN_PX) / (WIDTH_MAX_PX - WIDTH_MIN_PX)) as f32).clamp(0.0, 1.0)
}

/// **Os params do LÁPIS** (Fidelity + Stabilizer) — irmão pelo teto de 700 LOC, e o corte é por
/// responsabilidade: eles descrevem como a MÃO é capturada, não a geometria de uma forma. As
/// tabelas MEDIDAS que escolhem cada número viajam com eles.
#[path = "params_pencil.rs"]
mod pencil;
pub use pencil::*;

/// Horizontal text alignment for a text block (mirror of the panel's L / C / R row).
/// `Left` = lines start at the click origin; `Center` = centred on it; `Right` = lines
/// end at it. Lives in the tool crate (the panel deps this, not the shell).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// The canvas gesture the Vector tool performs (ADR-0108 Fase 1). `Pen` is the
/// draw + edit-anchor gesture (`PenTool`); the shape modes are drag-to-size
/// (`ShapeTool`). The tool owns the mode; the docked panel's segmented row sets
/// it and highlights the active one from the published snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DrawMode {
    /// Seta preta: seleciona e TRANSFORMA a forma pelo gizmo. Não toca a geometria.
    #[default]
    Select,
    /// Seta branca: edita âncoras e handles do path selecionado. Nunca cria um path,
    /// e o gizmo não aparece (as alças dele comeriam o clique do nó).
    Node,
    /// Caneta: cria path novo e edita os nós que ela mesma pôs. Sem gizmo.
    Pen,
    /// **Lápis**: arrasta e a curva sai — a mão livre. O gesto grava amostras, o decimador as
    /// reduz a nós e o ajuste de Hobby devolve a spline que PASSA por eles
    /// (`ph2d_vec_edit::Pencil`). É um modo e não uma variante da caneta porque o gesto é o
    /// oposto: a caneta é uma sequência de cliques DISCRETOS, o lápis é um arrasto contínuo.
    Pencil,
    /// **Forma**: arrasta para dimensionar a forma ATIVA do catálogo
    /// (`VectorTool::shape_kind`). É UM modo para todas as formas — retângulo, estrela,
    /// seta, balão… — porque a forma é dado, não código. Antes cada forma era um modo, e
    /// vinte e cinco formas seriam vinte e cinco variantes aqui, no painel e no dispatch.
    Shape,
    /// Texto: clica no canvas e digita; cada glyph vira um `VecPath` preenchido
    /// (ADR-0108). Não é uma shape-tool nem cria pelo Pen — o shell trata o gesto.
    Text,
    /// **Shape Builder**: com 2+ formas selecionadas, o cursor arrasta sobre as REGIÕES em
    /// que elas se dividem — o que ele pinta vira uma forma só; com Alt, some.
    ///
    /// É um modo e não um botão de Pathfinder porque a unidade de trabalho não é a FORMA, é
    /// a **face do arranjo**: a região "dentro da A e fora da B" não existe como objeto até o
    /// dedo passar por cima dela. Um Pathfinder obriga a pensar em operações; isto deixa
    /// desenhar o resultado.
    Build,
    /// **Conector**: pressiona sobre uma forma, arrasta, solta sobre outra — nasce uma
    /// linha que gruda nas duas e as SEGUE (soltar no vazio deixa a ponta solta ali;
    /// pressionar e soltar na mesma forma faz um laço).
    ///
    /// Não é uma forma do catálogo, e é por isso que é um MODO: a geometria de um
    /// conector não é autorada, é **derivada** (uma função pura de a quem cada ponta se
    /// prende), e a shell a re-cozinha a cada frame (`connector_live`).
    Connect,
    /// **Pick Shapes** (Blend): coleta as formas fechadas clicadas **na ordem**; o botão Blend as
    /// liga nessa sequência (ADR-0128 C2b). É um modo — como o Build e o Connect — porque o gesto é
    /// escolher formas no canvas, não editar a selecionada; a ORDEM da cadeia é a de clique, não a
    /// de z.
    PickBlend,
    /// **Fillet** (arredondar quina): pressiona sobre uma quina e arrasta — o recuo cresce com o
    /// arrasto e a quina ARREDONDA (arco). Se o ponto clicado não é quina (é suave), a ferramenta
    /// primeiro o transforma em quina. É o Live Corners (ADR-0121) virado ferramenta própria, com
    /// gesto de clicar-e-arrastar, em vez de uma alça escondida no modo Node.
    Fillet,
    /// **Chamfer** (chanfrar quina): idêntico ao [`DrawMode::Fillet`], mas a ligação é uma RETA em
    /// vez de arco (o SINAL do `corner_radius`, ADR-0121). O par Fillet/Chamfer consolida numa
    /// dupla de ferramentas o que estava espalhado entre a alça do Node e o toggle da seção Vertex.
    Chamfer,
    /// **Width**: as alças de LARGURA na curva (plano 25 §5, ADR-0148). Uma alça por parada do
    /// perfil, fora da curva à distância que a fita tem ali; afastar engrossa, aproximar afina,
    /// andar ao longo move a parada. Clicar na curva acrescenta uma parada; o botão direito
    /// sobre uma alça a apaga.
    ///
    /// É um modo pela MESMA razão do Fillet/Chamfer: no Node estas alças competiriam com as
    /// âncoras — uma parada de multiplicador pequeno senta a milímetros da curva, ou seja em
    /// cima delas. O Illustrator também o faz uma ferramenta (Shift+W).
    Width,
}

impl DrawMode {
    /// As ferramentas de QUINA (Fillet / Chamfer): clicar-e-arrastar sobre uma quina para
    /// arredondá-la ou chanfrá-la. Uma porta única para os sítios que roteiam o gesto delas.
    #[must_use]
    pub fn is_corner_tool(self) -> bool {
        matches!(self, DrawMode::Fillet | DrawMode::Chamfer)
    }

    /// A ferramenta de quina quer CHANFRO (reta) em vez de arredondado? Só faz sentido quando
    /// [`Self::is_corner_tool`] — o `Chamfer` chanfra, todo o resto arredonda.
    #[must_use]
    pub fn corner_is_chamfer(self) -> bool {
        self == DrawMode::Chamfer
    }
}

/// UI-facing vertex type for the docked panel's Vertex section (mirror of
/// `ph2d_vec_scene::VertexKind`; the shell maps between them). Lives in the tool
/// crate — the panel deps this, not `ph2d-vec-scene` — alongside [`DrawMode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexType {
    Corner,
    Smooth,
    Symmetric,
}

/// **O que a SELEÇÃO de vértices tem em comum** — o que o painel precisa para destacar (ou não
/// destacar) um chip da seção Vertex.
///
/// ⚠️ O `Mixed` existe porque publicar só o tipo do vértice PRIMÁRIO fazia um dos três chips ficar
/// aceso descrevendo a seleção INTEIRA: com dois nós de tipos diferentes selecionados, o painel
/// afirmava um deles. Nenhum chip aceso é a resposta honesta (e é a do Illustrator).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexSel {
    /// Todos os vértices selecionados são deste tipo.
    Uniform(VertexType),
    /// A seleção mistura tipos — chip nenhum descreve o todo.
    Mixed,
}

/// UI-facing line cap / join (mirror of `ph2d_vec_scene::{LineCap, LineJoin}`;
/// the shell maps between them — the tool crate doesn't dep `ph2d-vec-scene`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StrokeCap {
    #[default]
    Butt,
    Round,
    Square,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StrokeJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// Dash range as a **multiple of the stroke width** (`0` = solid). Width-aware:
/// the render draws dash/gap of `dash·width`, so a thick line keeps its gaps.
pub const DASH_MIN: f64 = 0.0;
pub const DASH_MAX: f64 = 8.0;
pub const DASH_SLIDER_SCALE: f32 = (DASH_MAX - DASH_MIN) as f32;
pub const DASH_SLIDER_OFFSET: f32 = DASH_MIN as f32;

/// Normalized track `0..=1` → dash multiple `MIN..=MAX`.
#[must_use]
pub fn slider_to_dash(track: f32) -> f64 {
    DASH_MIN + f64::from(track.clamp(0.0, 1.0)) * (DASH_MAX - DASH_MIN)
}
/// Dash multiple → normalized track (inverse of [`slider_to_dash`]).
#[must_use]
pub fn dash_to_slider(m: f64) -> f32 {
    ((m.clamp(DASH_MIN, DASH_MAX) - DASH_MIN) / (DASH_MAX - DASH_MIN)) as f32
}

/// Gap range as a **multiple of the stroke width**, independent of the dash
/// length — the render draws the space between dashes as `gap·width`. Same
/// width-aware model as [`slider_to_dash`]. Default `1` (Dash = 0 ⇒ solid, so
/// the gap only bites once Dash > 0).
pub const GAP_MIN: f64 = 0.0;
pub const GAP_MAX: f64 = 8.0;
pub const GAP_DEFAULT: f64 = 1.0;
pub const GAP_SLIDER_SCALE: f32 = (GAP_MAX - GAP_MIN) as f32;
pub const GAP_SLIDER_OFFSET: f32 = GAP_MIN as f32;

/// Normalized track `0..=1` → gap multiple `MIN..=MAX`.
#[must_use]
pub fn slider_to_gap(track: f32) -> f64 {
    GAP_MIN + f64::from(track.clamp(0.0, 1.0)) * (GAP_MAX - GAP_MIN)
}
/// Gap multiple → normalized track (inverse of [`slider_to_gap`]).
#[must_use]
pub fn gap_to_slider(m: f64) -> f32 {
    ((m.clamp(GAP_MIN, GAP_MAX) - GAP_MIN) / (GAP_MAX - GAP_MIN)) as f32
}

/// Os parâmetros de TEXTO (tamanho · peso · entrelinha · tracking) — módulo irmão pelo teto
/// de 700 LOC deste arquivo. Uma família inteira, com os seus mapas de slider ao lado das
/// suas faixas.
#[path = "params_text.rs"]
mod text;
pub use text::*;

/// **Offset Path** — o slider fala **FRAÇÃO do tamanho da forma**, não unidades de mundo.
///
/// Bipolar de propósito: o negativo ENCOLHE, e é a mesma operação com o sinal trocado (o
/// diálogo do Illustrator também aceita negativo num campo só). Um par de botões
/// "crescer/encolher" seriam dois controles para um número.
///
/// ⚠️ **A faixa antiga (±4 unidades de mundo, "ergonômica" pela ALTURA DA VISTA) era o bug
/// do report de 2026-07-20** ("se selecionar Round, não consegue mudar"): o alcance útil de
/// um offset é propriedade da FORMA, não do viewport — no donut do smoke (2,4 de lado, ~104
/// px de track) o gesto natural SATURAVA em d=±4, e os dois extremos são regimes onde os
/// joins não podem diferir visivelmente (à esquerda a forma ANIQUILA — três joins produzem
/// o mesmo nada; à direita ela estoura a tela e as quinas, onde o join mora, saem de vista).
/// A janela de retune funcionava; o dial é que entregava o artista a regimes join-inertes.
///
/// A faixa nova é a LEI DA FORMA: `d = fração × (maxdim/2)` da seleção, então **−100% é
/// morte garantida** (o inradius de qualquer forma ≤ maxdim/2 — não existe d mais negativo
/// que ainda mostre alguma coisa) **e +100% é dobrar a forma** (o eixo maior cresce
/// exatamente 2×, com as quinas na vizinhança da tela). Todo o curso do slider é
/// significativo, e a precisão no donut fica 3,3× mais fina (0,023 vs 0,077 unidades/px).
/// Quem resolve fração→mundo é a shell (`vec_expand::offset_scale`, porta única, congelada
/// na sessão do arrasto). O motor não tem teto — quem quiser mais offseta duas vezes.
pub const OFFSET_FRAC_MIN: f64 = -1.0;
pub const OFFSET_FRAC_MAX: f64 = 1.0;
/// Default `+0.25` (um quarto do meio-tamanho da forma): visível de imediato em QUALQUER
/// escala de forma — o antigo `0.5` absoluto era invisível numa forma de 100 unidades e
/// letal numa de 0,6. Zero seria um botão que não faz nada no primeiro clique.
pub const OFFSET_DEFAULT_FRAC: f64 = 0.25;
/// O chip numérico mostra PERCENTUAL (−100..+100) — o mapa do store é estático, então a
/// UI nunca precisa saber o tamanho da seleção (e o rótulo nunca mente).
pub const OFFSET_SLIDER_SCALE: f32 = 200.0;
pub const OFFSET_SLIDER_OFFSET: f32 = -100.0;

/// Normalized track `0..=1` → fração do offset `−1..=+1` (−100%..+100% do meio-tamanho).
#[must_use]
pub fn slider_to_offset_frac(track: f32) -> f64 {
    OFFSET_FRAC_MIN + f64::from(track.clamp(0.0, 1.0)) * (OFFSET_FRAC_MAX - OFFSET_FRAC_MIN)
}
/// Fração do offset → normalized track (inverse of [`slider_to_offset_frac`]).
#[must_use]
pub fn offset_frac_to_slider(frac: f64) -> f32 {
    ((frac.clamp(OFFSET_FRAC_MIN, OFFSET_FRAC_MAX) - OFFSET_FRAC_MIN)
        / (OFFSET_FRAC_MAX - OFFSET_FRAC_MIN)) as f32
}

/// **O perfil de LARGURA** (o Power Stroke) — irmão pelo teto de 700 LOC. A faixa, o mapa do
/// slider, o default e o catálogo de perfis nomeados moram lá; o pai continua sendo o vocabulário
/// de ESTILO do traço.
#[path = "params_width.rs"]
mod width;
pub use width::*;

/// Minimum / maximum polygon sides (inclusive range the Sides slider spans).
pub const SIDES_MIN: u32 = 3;
pub const SIDES_MAX: u32 = 12;

/// Affine Sides-slider mapping `display_n = track * SCALE + OFFSET` (track
/// `0..=1`), consumed by `WidgetStore::link_slider_number_mapped` so the chip
/// mirrors the slider. `SCALE = MAX - MIN`, `OFFSET = MIN`.
pub const SIDES_SLIDER_SCALE: f32 = (SIDES_MAX - SIDES_MIN) as f32;
pub const SIDES_SLIDER_OFFSET: f32 = SIDES_MIN as f32;

/// Normalized slider track `0..=1` → polygon sides `MIN..=MAX` (rounded).
#[must_use]
pub fn slider_to_sides(track: f32) -> u32 {
    (SIDES_MIN as f32 + track.clamp(0.0, 1.0) * (SIDES_MAX - SIDES_MIN) as f32).round() as u32
}

/// Polygon sides → normalized slider track `0..=1` (inverse of
/// [`slider_to_sides`]); seeds the knob from the tool's authoritative sides.
#[must_use]
pub fn sides_to_slider(n: u32) -> f32 {
    ((n.clamp(SIDES_MIN, SIDES_MAX) - SIDES_MIN) as f32 / (SIDES_MAX - SIDES_MIN) as f32)
        .clamp(0.0, 1.0)
}

/// Star point count range (the Points slider spans this).
pub const STAR_POINTS_MIN: u32 = 3;
pub const STAR_POINTS_MAX: u32 = 12;
pub const STAR_POINTS_SLIDER_SCALE: f32 = (STAR_POINTS_MAX - STAR_POINTS_MIN) as f32;
pub const STAR_POINTS_SLIDER_OFFSET: f32 = STAR_POINTS_MIN as f32;

/// Normalized track `0..=1` → star points `MIN..=MAX` (rounded).
#[must_use]
pub fn slider_to_star_points(track: f32) -> u32 {
    (STAR_POINTS_MIN as f32 + track.clamp(0.0, 1.0) * STAR_POINTS_SLIDER_SCALE).round() as u32
}
/// Star points → normalized track (inverse of [`slider_to_star_points`]).
#[must_use]
pub fn star_points_to_slider(n: u32) -> f32 {
    ((n.clamp(STAR_POINTS_MIN, STAR_POINTS_MAX) - STAR_POINTS_MIN) as f32
        / STAR_POINTS_SLIDER_SCALE)
        .clamp(0.0, 1.0)
}

/// Star inner/outer radius ratio range (the Inner slider spans this).
pub const STAR_INNER_MIN: f64 = 0.1;
pub const STAR_INNER_MAX: f64 = 0.9;
pub const STAR_INNER_SLIDER_SCALE: f32 = (STAR_INNER_MAX - STAR_INNER_MIN) as f32;
pub const STAR_INNER_SLIDER_OFFSET: f32 = STAR_INNER_MIN as f32;

/// Normalized track `0..=1` → star inner ratio `MIN..=MAX`.
#[must_use]
pub fn slider_to_star_inner(track: f32) -> f64 {
    STAR_INNER_MIN + f64::from(track.clamp(0.0, 1.0)) * (STAR_INNER_MAX - STAR_INNER_MIN)
}
/// Star inner ratio → normalized track (inverse of [`slider_to_star_inner`]).
#[must_use]
pub fn star_inner_to_slider(r: f64) -> f32 {
    ((r.clamp(STAR_INNER_MIN, STAR_INNER_MAX) - STAR_INNER_MIN) / (STAR_INNER_MAX - STAR_INNER_MIN))
        as f32
}

/// Faixa dos RAIOS DE CANTO (round-rect, polígono, pontas/vales da estrela), em
/// **PIXELS** — a unidade em que o usuário pensa (a de mundo é pequena: a viewport
/// inteira tem ~10 unidades, então um raio útil seria `0.3`, ilegível numa caixa).
///
/// Era um SLIDER de 0..40 px, e o teto não alcançava formas grandes. Agora é uma
/// **caixa numérica** de 0..500 px: faixa ampla demais para um knob, e o que se quer é
/// digitar/arrastar o número exato. A conversão px → mundo é feita na fronteira (a
/// geometria é mundo), como sempre foi.
pub const RADIUS_MIN_PX: f64 = 0.0;
pub const RADIUS_MAX_PX: f64 = 500.0;
/// Passo do arrasto/setas na caixa de raio (px).
pub const RADIUS_STEP_PX: f64 = 1.0;

/// Clampa um raio de canto autorado (px) à faixa da caixa.
#[must_use]
pub fn clamp_radius_px(v: f64) -> f64 {
    v.clamp(RADIUS_MIN_PX, RADIUS_MAX_PX)
}

/// Spiral turn count range (the Turns slider spans this).
pub const SPIRAL_TURNS_MIN: u32 = 1;
pub const SPIRAL_TURNS_MAX: u32 = 8;
pub const SPIRAL_TURNS_SLIDER_SCALE: f32 = (SPIRAL_TURNS_MAX - SPIRAL_TURNS_MIN) as f32;
pub const SPIRAL_TURNS_SLIDER_OFFSET: f32 = SPIRAL_TURNS_MIN as f32;

/// Normalized track `0..=1` → spiral turns `MIN..=MAX` (rounded).
#[must_use]
pub fn slider_to_spiral_turns(track: f32) -> u32 {
    (SPIRAL_TURNS_MIN as f32 + track.clamp(0.0, 1.0) * SPIRAL_TURNS_SLIDER_SCALE).round() as u32
}
/// Span mínimo/máximo de um arco (graus). O slider mapeia linearmente.
pub const ARC_DEGREES_MIN: f64 = 1.0;
pub const ARC_DEGREES_MAX: f64 = 360.0;
pub const ARC_DEGREES_SLIDER_SCALE: f32 = (ARC_DEGREES_MAX - ARC_DEGREES_MIN) as f32;
pub const ARC_DEGREES_SLIDER_OFFSET: f32 = ARC_DEGREES_MIN as f32;

/// Track `[0,1]` → graus `[1, 360]`.
#[must_use]
pub fn slider_to_arc_degrees(track: f32) -> f64 {
    f64::from(ARC_DEGREES_SLIDER_OFFSET + track.clamp(0.0, 1.0) * ARC_DEGREES_SLIDER_SCALE)
}

/// Graus → track `[0,1]`.
#[must_use]
pub fn arc_degrees_to_slider(deg: f64) -> f32 {
    (((deg as f32) - ARC_DEGREES_SLIDER_OFFSET) / ARC_DEGREES_SLIDER_SCALE).clamp(0.0, 1.0)
}

/// Spiral turns → normalized track (inverse of [`slider_to_spiral_turns`]).
#[must_use]
pub fn spiral_turns_to_slider(n: u32) -> f32 {
    ((n.clamp(SPIRAL_TURNS_MIN, SPIRAL_TURNS_MAX) - SPIRAL_TURNS_MIN) as f32
        / SPIRAL_TURNS_SLIDER_SCALE)
        .clamp(0.0, 1.0)
}

/// Opacity slider: track `0..=1` → alpha `0..=255`; the chip shows `0..=100`
/// (percent), so `SCALE = 100`, `OFFSET = 0`.
pub const OPACITY_SLIDER_SCALE: f32 = 100.0;
pub const OPACITY_SLIDER_OFFSET: f32 = 0.0;

/// Normalized track `0..=1` → alpha byte `0..=255` (rounded).
#[must_use]
pub fn slider_to_opacity(track: f32) -> u8 {
    (track.clamp(0.0, 1.0) * 255.0).round() as u8
}
/// Alpha byte → normalized track (inverse of [`slider_to_opacity`]).
#[must_use]
pub fn opacity_to_slider(a: u8) -> f32 {
    f32::from(a) / 255.0
}

/// Mode + shape parameters the shell mirrors from the tool each frame to route
/// canvas gestures (pen vs shape) and drive the [`ShapeTool`] without a downcast.
///
/// [`ShapeTool`]: ph2d_vec_edit::ShapeTool
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorDrawConfig {
    pub mode: DrawMode,
    /// **A estabilização autorada do lápis** (0 = ponteiro cru). Viaja no config porque quem a
    /// aplica é o `input_dispatch` da shell, por movimento de ponteiro — e ali a única alça para o
    /// tool é este espelho publicado a cada frame; alcançar o tool por downcast num handler de move
    /// seria trabalho por evento para ler um `f32`.
    pub pencil_stabilizer: f32,
    /// **De onde a largura do traço de lápis vem** (W1d). Viaja no config pela MESMA razão do
    /// estabilizador: quem a consome é o laço de frame da shell, e alcançar o tool por downcast
    /// para ler um enum seria trabalho por frame para uma pergunta que o espelho já responde.
    pub pencil_width_source: ph2d_vec_edit::pencil_width::WidthSource,
    /// A forma ATIVA do catálogo (só importa no modo [`DrawMode::Shape`]).
    pub shape: ShapeKind,
    /// Os parâmetros dela, na unidade em que o usuário os autora (px para raios). A
    /// shell converte para MUNDO na fronteira (`shapes::to_world`) antes de cozinhar.
    pub values: ShapeValues,
}

impl Default for VectorDrawConfig {
    fn default() -> Self {
        Self {
            mode: DrawMode::Select,
            shape: ShapeKind::Rectangle,
            values: ShapeKind::Rectangle.defaults(),
            pencil_stabilizer: PENCIL_STABILIZER_DEFAULT,
            pencil_width_source: ph2d_vec_edit::pencil_width::WidthSource::default(),
        }
    }
}

/// Os dois seletores de ponta, como o painel os endereça: `0` = começo, `1` = fim.
/// Um `usize` (e não um enum) porque é exatamente o que o id de opção carrega
/// (`ph2d_editor_core::ids::vector_marker_option_id(slot, index)`) — a tool, o painel
/// e o teste de seam falam o mesmo índice.
pub const MARKER_SLOT_START: usize = 0;
pub const MARKER_SLOT_END: usize = 1;

/// A ponta cujo discriminante é `v` (o valor que o `SetValue` do painel carrega). Um
/// discriminante desconhecido (versão futura) vira [`Marker::None`] — "sem ponta" —
/// em vez de entrar em pânico, que é a mesma regra do `Marker::from_u8`.
#[must_use]
pub fn marker_from_value(v: f64) -> Marker {
    // `f64 as u8` satura em Rust (NaN → 0), então nenhum valor doido escapa.
    Marker::from_u8(v as u8).unwrap_or(Marker::None)
}

/// **Steps do Blend** — quantas formas nascem entre CADA par. No modelo VIVO (ADR-0128) os passos
/// são virtuais (desenho, não paths na cena), então centenas são baratas — o Enio pediu "não 12,
/// mas centenas". O slider é grosso neste teto; o chip numérico ligado dá o valor exato.
pub const MAX_BLEND_STEPS: u32 = 200;
/// O default do Illustrator para um blend novo.
pub const BLEND_STEPS_DEFAULT: u32 = 3;

/// O `t` de um morph recém-criado: **no meio do caminho**.
///
/// No meio, e não em `0`: um morph que nasce em `t=0` é uma cópia exata da forma A, EM CIMA da
/// forma A — o artista clica e não vê nada acontecer. No meio, o objeto novo se anuncia.
/// (O mesmo número que o `VecMorph::new` escolhe; aqui ele é o que o slider mostra ao nascer.)
pub const MORPH_T_DEFAULT: f32 = 0.5;

/// O passo do campo numérico do `t` — **1% do caminho**.
///
/// O `t` é uma fração de `0` a `1`, então o passo é o quantum do DOMÍNIO, não uma medida de tela:
/// 100 paradas entre as duas formas é fino o bastante para o olho não ver degrau e grosso o
/// bastante para a seta do teclado ser útil. (O slider é contínuo; isto é só a caixa.)
pub const MORPH_T_STEP: f64 = 0.01;

/// Track (0..1) → nº de passos (1..=[`MAX_BLEND_STEPS`]).
#[must_use]
pub fn blend_steps_from_track(track: f64) -> u32 {
    let n = 1.0 + track.clamp(0.0, 1.0) * f64::from(MAX_BLEND_STEPS - 1);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = n.round() as u32;
    n.clamp(1, MAX_BLEND_STEPS)
}

/// Nº de passos → track (0..1). O inverso exato de [`blend_steps_from_track`].
#[must_use]
pub fn blend_steps_to_track(steps: u32) -> f32 {
    let n = steps.clamp(1, MAX_BLEND_STEPS);
    (f64::from(n - 1) / f64::from(MAX_BLEND_STEPS - 1)) as f32
}

/// O snapshot publicado por-frame — mora no irmão [`super::params_snapshot`] pelo teto de LOC,
/// e é re-exportado aqui porque é por este caminho que os cerca de cinquenta sítios já o escrevem.
pub use super::params_snapshot::VectorStyleSnapshot;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_px_round_trip_endpoints() {
        assert_eq!(slider_to_px(0.0), WIDTH_MIN_PX);
        assert_eq!(slider_to_px(1.0), WIDTH_MAX_PX);
        assert!((px_to_slider(WIDTH_MIN_PX) - 0.0).abs() < 1e-6);
        assert!((px_to_slider(WIDTH_MAX_PX) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn slider_mapping_matches_affine_consts() {
        // The panel's chip display uses `track * SCALE + OFFSET`; it must equal
        // the tool's `slider_to_px` for the chip to mirror the slider exactly.
        for &t in &[0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            let via_affine = f64::from(t * WIDTH_SLIDER_SCALE + WIDTH_SLIDER_OFFSET);
            assert!((via_affine - slider_to_px(t)).abs() < 1e-6);
        }
    }

    #[test]
    fn sides_slider_round_trip_endpoints() {
        assert_eq!(slider_to_sides(0.0), SIDES_MIN);
        assert_eq!(slider_to_sides(1.0), SIDES_MAX);
        assert!((sides_to_slider(SIDES_MIN) - 0.0).abs() < 1e-6);
        assert!((sides_to_slider(SIDES_MAX) - 1.0).abs() < 1e-6);
        // Mid-track rounds to the nearest integer side count.
        assert_eq!(slider_to_sides(0.5), (SIDES_MIN + SIDES_MAX) / 2 + 1);
    }

    /// ADR-0112: a ferramenta abre na SELEÇÃO (seta preta), como qualquer editor
    /// vetorial. A caneta é um modo, não o ponto de partida.
    #[test]
    fn draw_mode_defaults_to_select() {
        assert_eq!(DrawMode::default(), DrawMode::Select);
        assert_eq!(VectorDrawConfig::default().mode, DrawMode::Select);
    }

    /// **Escrever um perfil do catálogo o torna o ATIVO** — o gate central da fileira de perfis
    /// (W2b): o que o clique escreve nos sliders é exatamente o que a linha acesa procura.
    ///
    /// ⚠️ **Ele nasceu para pegar uma comparação em MULTIPLICADOR**, que é o erro natural aqui e
    /// que nenhum outro teste veria: o ida-e-volta pelo trilho `f32` devolve `1.0000000298…` para
    /// `1.0`, então uma fileira que comparasse multiplicadores ficaria **permanentemente apagada**
    /// — pintada, clicável, e incapaz de mostrar o que o artista acabou de escolher.
    #[test]
    fn writing_a_preset_makes_it_the_active_one() {
        for (i, p) in ph2d_vec_scene::WIDTH_PRESETS.iter().enumerate() {
            let tracks = preset_tracks(&p.profile);
            assert_eq!(
                active_preset(&tracks),
                Some(i),
                "escrever o perfil `{}` não acende a linha dele",
                p.key
            );
        }
    }

    /// **A ida-e-volta em MULTIPLICADOR não fecha** — o número que torna o gate acima
    /// load-bearing, pinado aqui para ninguém "simplificar" a comparação de volta.
    #[test]
    fn a_multiplier_does_not_survive_the_round_trip_through_the_track() {
        let back = slider_to_wprofile(wprofile_to_slider(1.0));
        assert!(
            back != 1.0,
            "o trilho passou a round-tripar exato — se isto virou verdade, a comparação em \
             multiplicador deixou de ser uma armadilha, e este gate pode ir embora com ela"
        );
        assert!(
            (back - 1.0).abs() < 1e-6,
            "e a diferença é de precisão, não de faixa: {back}"
        );
    }

    /// **O default dos sliders NÃO é um perfil do catálogo**, e a fileira o diz não acendendo
    /// nada. É a forma que o artista vê ao abrir a seção (o traço de nanquim, `0.25/1.6/0.25`):
    /// acender uma linha ali seria nomear como *Taper* uma curva que não é.
    #[test]
    fn the_default_profile_lights_no_row() {
        assert_eq!(active_preset(&preset_tracks(&WPROFILE_DEFAULT)), None);
    }

    /// **Um trilho arrastado apaga a fileira.** É a metade que torna o readout honesto: depois de
    /// mexer num slider (ou numa alça do Width Tool) a forma não é mais nenhum dos nomes, e dizer
    /// que ainda é seria o painel mentindo sobre o que está na tela.
    #[test]
    fn a_dragged_track_lights_no_row() {
        let mut tracks = preset_tracks(&ph2d_vec_scene::WIDTH_PRESETS[1].profile);
        assert_eq!(active_preset(&tracks), Some(1));
        tracks[1] += 0.01;
        assert_eq!(
            active_preset(&tracks),
            None,
            "a fileira continua acesa depois de o artista mexer no perfil"
        );
    }
}
