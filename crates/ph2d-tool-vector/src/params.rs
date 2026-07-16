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
pub const WIDTH_MIN_PX: f64 = 1.0;
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

/// Text glyph size in WORLD units — the Text-mode Size slider's range. The editor
/// view is ~10 world-units tall, so 1.0 reads comfortably. Shared by the panel
/// (seed + chip mapping) and the shell drain (track → size), like the Width family.
pub const TEXT_SIZE_MIN: f64 = 0.05;
pub const TEXT_SIZE_MAX: f64 = 8.0;
/// Default glyph size for a new text session (world units).
pub const DEFAULT_TEXT_SIZE: f64 = 1.0;

/// Affine slider mapping `size = track * SCALE + OFFSET` (track `0..=1`), consumed
/// by `WidgetStore::link_slider_number_mapped` so the size chip mirrors the slider.
pub const TEXT_SIZE_SLIDER_SCALE: f32 = (TEXT_SIZE_MAX - TEXT_SIZE_MIN) as f32;
pub const TEXT_SIZE_SLIDER_OFFSET: f32 = TEXT_SIZE_MIN as f32;

/// Normalized slider track `0..=1` → glyph size (world units) `MIN..=MAX`.
#[must_use]
pub fn slider_to_text_size(track: f32) -> f64 {
    TEXT_SIZE_MIN + f64::from(track.clamp(0.0, 1.0)) * (TEXT_SIZE_MAX - TEXT_SIZE_MIN)
}

/// Glyph size (world units) → normalized slider track `0..=1` (inverse of
/// [`slider_to_text_size`]); seeds the knob from the shell's current size.
#[must_use]
pub fn text_size_to_slider(size: f64) -> f32 {
    (((size.clamp(TEXT_SIZE_MIN, TEXT_SIZE_MAX) - TEXT_SIZE_MIN) / (TEXT_SIZE_MAX - TEXT_SIZE_MIN))
        as f32)
        .clamp(0.0, 1.0)
}

/// Variable-font Weight (`wght` axis) — the Text-mode Weight slider's range. Inter
/// Variable spans 100..900 (default 400), the CSS/OpenType weight scale. Shared by the
/// panel (seed + chip mapping) and the shell drain (track → weight).
pub const TEXT_WEIGHT_MIN: f64 = 100.0;
pub const TEXT_WEIGHT_MAX: f64 = 900.0;
/// Default weight for a new text session (`wght` 400 = Regular).
pub const DEFAULT_TEXT_WEIGHT: f64 = 400.0;

/// Affine slider mapping `weight = track * SCALE + OFFSET` (track `0..=1`).
pub const TEXT_WEIGHT_SLIDER_SCALE: f32 = (TEXT_WEIGHT_MAX - TEXT_WEIGHT_MIN) as f32;
pub const TEXT_WEIGHT_SLIDER_OFFSET: f32 = TEXT_WEIGHT_MIN as f32;

/// Normalized slider track `0..=1` → font weight `MIN..=MAX`.
#[must_use]
pub fn slider_to_text_weight(track: f32) -> f64 {
    TEXT_WEIGHT_MIN + f64::from(track.clamp(0.0, 1.0)) * (TEXT_WEIGHT_MAX - TEXT_WEIGHT_MIN)
}

/// Font weight → normalized slider track `0..=1` (inverse of [`slider_to_text_weight`]);
/// seeds the knob from the shell's current weight.
#[must_use]
pub fn text_weight_to_slider(weight: f64) -> f32 {
    (((weight.clamp(TEXT_WEIGHT_MIN, TEXT_WEIGHT_MAX) - TEXT_WEIGHT_MIN)
        / (TEXT_WEIGHT_MAX - TEXT_WEIGHT_MIN)) as f32)
        .clamp(0.0, 1.0)
}

/// Text line height (leading) as a MULTIPLE of the glyph size. 1.2 is the common
/// default; the range spans tight (0.8) to airy (3.0). Shared by the panel (seed +
/// chip mapping) and the shell drain (track → line height).
pub const TEXT_LINE_HEIGHT_MIN: f64 = 0.8;
pub const TEXT_LINE_HEIGHT_MAX: f64 = 3.0;
/// Default line height for a new text session (1.2× the size).
pub const DEFAULT_TEXT_LINE_HEIGHT: f64 = 1.2;

/// Affine slider mapping `line_height = track * SCALE + OFFSET` (track `0..=1`).
pub const TEXT_LINE_HEIGHT_SLIDER_SCALE: f32 = (TEXT_LINE_HEIGHT_MAX - TEXT_LINE_HEIGHT_MIN) as f32;
pub const TEXT_LINE_HEIGHT_SLIDER_OFFSET: f32 = TEXT_LINE_HEIGHT_MIN as f32;

/// Normalized slider track `0..=1` → line height `MIN..=MAX`.
#[must_use]
pub fn slider_to_text_line_height(track: f32) -> f64 {
    TEXT_LINE_HEIGHT_MIN
        + f64::from(track.clamp(0.0, 1.0)) * (TEXT_LINE_HEIGHT_MAX - TEXT_LINE_HEIGHT_MIN)
}

/// Line height → normalized slider track `0..=1` (inverse of [`slider_to_text_line_height`]).
#[must_use]
pub fn text_line_height_to_slider(line_height: f64) -> f32 {
    (((line_height.clamp(TEXT_LINE_HEIGHT_MIN, TEXT_LINE_HEIGHT_MAX) - TEXT_LINE_HEIGHT_MIN)
        / (TEXT_LINE_HEIGHT_MAX - TEXT_LINE_HEIGHT_MIN)) as f32)
        .clamp(0.0, 1.0)
}

/// Text letter-spacing (tracking) as a FRACTION of the glyph size (em), added between
/// glyphs. 0 = the font's native spacing; negative tightens, positive opens up.
pub const TEXT_TRACKING_MIN: f64 = -0.1;
pub const TEXT_TRACKING_MAX: f64 = 0.5;
/// Default tracking for a new text session (0 = native spacing).
pub const DEFAULT_TEXT_TRACKING: f64 = 0.0;

/// Affine slider mapping `tracking = track * SCALE + OFFSET` (track `0..=1`).
pub const TEXT_TRACKING_SLIDER_SCALE: f32 = (TEXT_TRACKING_MAX - TEXT_TRACKING_MIN) as f32;
pub const TEXT_TRACKING_SLIDER_OFFSET: f32 = TEXT_TRACKING_MIN as f32;

/// Normalized slider track `0..=1` → tracking (em fraction) `MIN..=MAX`.
#[must_use]
pub fn slider_to_text_tracking(track: f32) -> f64 {
    TEXT_TRACKING_MIN + f64::from(track.clamp(0.0, 1.0)) * (TEXT_TRACKING_MAX - TEXT_TRACKING_MIN)
}

/// Tracking (em fraction) → normalized slider track `0..=1` (inverse of
/// [`slider_to_text_tracking`]).
#[must_use]
pub fn text_tracking_to_slider(tracking: f64) -> f32 {
    (((tracking.clamp(TEXT_TRACKING_MIN, TEXT_TRACKING_MAX) - TEXT_TRACKING_MIN)
        / (TEXT_TRACKING_MAX - TEXT_TRACKING_MIN)) as f32)
        .clamp(0.0, 1.0)
}

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
    /// liga nessa sequência (ADR-0122 C2b). É um modo — como o Build e o Connect — porque o gesto é
    /// escolher formas no canvas, não editar a selecionada; a ORDEM da cadeia é a de clique, não a
    /// de z.
    PickBlend,
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
        }
    }
}

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

/// **Steps do Blend** — quantas formas nascem entre CADA par. No modelo VIVO (ADR-0122) os passos
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
}
