//! Vector Style panel state + the shell→panel Style snapshot.
//!
//! Per-instance state is empty (ADR-0029 §4.3, mirror of Padding): the
//! authoritative Style lives on the shell-side `VectorTool`. Each frame the
//! shell publishes a [`VectorStyleSnapshot`] via [`set_current_vector_style`]
//! BEFORE the panel paints; `paint` reads it to seed the Width chip + the two
//! colour swatches. Edits flow back out over `EditorAction::ToolPanelEvent`
//! (Width slider, Fill-None) and the colour-picker read-back (Stroke / Fill
//! swatches), so the panel holds no authoritative state.

use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_tool_vector::shapes::ShapeGroup;
use ph2d_tool_vector::{TextAlign, VectorStyleSnapshot, VertexType};
use ph2d_vec_scene::ShapeKind;
use ph2d_vector::BezPath;
use std::cell::{Cell, RefCell};

thread_local! {
    /// Live snapshot published by the host before each `paint`. `None` until
    /// the first push (panel paints defaults).
    static CURRENT_SNAPSHOT: RefCell<Option<VectorStyleSnapshot>> = const { RefCell::new(None) };
    /// Type of the currently-selected vertex (published by the shell each frame
    /// from the Pen). `None` = no vertex selected → the Vertex section hides.
    static CURRENT_VERTEX_TYPE: RefCell<Option<VertexType>> = const { RefCell::new(None) };
    /// Selected path's anchor bbox `[x, y, w, h]` (world), published each frame.
    /// `None` = no path selected → the Transform section hides.
    static CURRENT_TRANSFORM: Cell<Option<[f64; 4]>> = const { Cell::new(None) };
    /// Selected path's `closed` flag, published each frame (`None` = no selection).
    /// Drives the Close/Open toggle button's label.
    static CURRENT_PATH_CLOSED: Cell<Option<bool>> = const { Cell::new(None) };
    /// Selected path's fill kind (`None` = no path selected / no fill). Drives the
    /// Fill-type selector highlight + whether the gradient controls show.
    static CURRENT_FILL_KIND: Cell<Option<FillKind>> = const { Cell::new(None) };
    /// Selected path's linear-gradient angle in degrees (`None` unless Linear).
    static CURRENT_GRAD_ANGLE: Cell<Option<f64>> = const { Cell::new(None) };
    /// Selected multi-point gradient point's influence (`None` unless a point is
    /// selected) — drives the Influence slider's visibility + value.
    static CURRENT_GRAD_INFLUENCE: Cell<Option<f64>> = const { Cell::new(None) };
    /// Selected multi-point gradient point's jitter (`None` unless a point is
    /// selected) — drives the Jitter slider's visibility + value.
    static CURRENT_GRAD_JITTER: Cell<Option<f64>> = const { Cell::new(None) };
    /// Number of paths in the object selection — drives the Align (≥2) / Distribute
    /// (≥3) section visibility.
    static CURRENT_SELECTION_COUNT: Cell<usize> = const { Cell::new(0) };
    /// Fill rule of the selected path, `Some` only when it is a COMPOUND path —
    /// the two rules agree on a single contour, so the row would be a no-op there.
    static CURRENT_FILL_RULE: Cell<Option<PathFillRule>> = const { Cell::new(None) };
    /// Whether shape-snapping is on (mirrored from the shell). The GRID toggle
    /// lives in the editor's universal Grid Snap panel, not here.
    static CURRENT_SNAP: Cell<bool> = const { Cell::new(true) };
    /// "Set Center" armado: a próxima pressão no canvas reposiciona a origem.
    static CURRENT_PIVOT_EDIT: Cell<bool> = const { Cell::new(false) };
    /// A seleção tem alguma forma VIVA (paramétrica/texto, com `VecShape`) —
    /// habilita o botão "Convert to Curves". Publicado pela shell.
    static CURRENT_CONVERTIBLE: Cell<bool> = const { Cell::new(false) };
    static CURRENT_HAS_ENVELOPE: Cell<bool> = const { Cell::new(false) };
    static CURRENT_ENVELOPE_MESH: Cell<bool> = const { Cell::new(false) };
    /// Os rótulos dos presets de gaiola, PUBLICADOS pelo shell (a tabela mora no
    /// `ph2d_ecs::EnvelopeWarp`, que este painel não vê). Vazio = nenhum publicado ainda.
    static CURRENT_ENVELOPE_PRESETS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    /// O índice do preset ATIVO na lista acima; `None` = a gaiola é manual (a mão a promoveu).
    static CURRENT_ENVELOPE_WARP: Cell<Option<usize>> = const { Cell::new(None) };
    /// A força do preset ativo, `[-1, 1]`.
    static CURRENT_ENVELOPE_BEND: Cell<f64> = const { Cell::new(0.0) };
    /// Mostrar a seção Text: modo Text OU um objeto de TEXTO selecionado (as configs
    /// do texto ficam visíveis enquanto ele for texto — não-curva — mesmo no Select).
    static CURRENT_TEXT_VISIBLE: Cell<bool> = const { Cell::new(false) };
    /// A forma cujos PARÂMETROS o painel desenha: a da forma VIVA selecionada, quando há
    /// uma (os campos então a editam — Live Shape). `None` = sem forma viva na seleção;
    /// o foco vira a forma ATIVA do catálogo (o default do próximo traço).
    static CURRENT_SHAPE_FOCUS: Cell<Option<ShapeKind>> = const { Cell::new(None) };
    /// A aba de família aberta no seletor. `None` = segue a forma ativa.
    static CURRENT_SHAPE_GROUP: Cell<Option<ShapeGroup>> = const { Cell::new(None) };
    /// Semente ONE-SHOT dos sliders de texto `[size, weight, line_height, tracking]`,
    /// publicada quando o ALVO muda (sessão nova / outro objeto selecionado). O paint
    /// a consome e escreve no store — depois o store é a fonte (senão o seed brigaria
    /// com o arrasto do slider).
    static TEXT_SEED: Cell<Option<[f64; 4]>> = const { Cell::new(None) };
    /// Texto da sessão de edição ativa (modo Text). `None` = sem sessão. Só
    /// LEITURA no painel (display); a digitação segue no canvas (A2). Publicado
    /// pela shell a cada frame.
    static CURRENT_TEXT: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Rótulo da família de fonte corrente do texto (embutida ou do sistema),
    /// publicado pela shell. Exibido entre os botões `<`/`>` do seletor de fonte.
    static CURRENT_TEXT_FONT: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Alinhamento horizontal corrente do texto (L/C/R), publicado pela shell no modo
    /// Text — destaca o botão ativo na seção Paragraph. `None` fora do modo Text.
    static CURRENT_TEXT_ALIGN: Cell<Option<TextAlign>> = const { Cell::new(None) };
    /// Eixos de variação da fonte corrente (sem `wght`), publicados pela shell — a
    /// seção Axes desenha um campo por eixo. Vazio fora do modo Text / fonte estática.
    static CURRENT_TEXT_AXES: RefCell<Vec<TextAxisSlot>> = const { RefCell::new(Vec::new()) };
    /// Rotation-field accumulator: the angle (degrees) the Angle chip last
    /// reported THIS gesture. `event` emits the DELTA `(current − this)` so the
    /// shell rotates incrementally; reset to 0 by `paint` whenever the field is
    /// unfocused (gesture ended), so the shell stays stateless.
    static ROT_LAST: Cell<f64> = const { Cell::new(0.0) };
    /// Last measured scrollable content height (set by `paint`).
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
    /// Font dropdown previews: one [`FontPreview`] per pickable family, in the
    /// shell's pickable order (`[bundled] ++ imported ++ system`). Each carries the
    /// family name **pre-rendered in that family's own outline** (em-normalised,
    /// y-up) — the popover draws it as the row's real-style preview. Built lazily by
    /// the shell (only after [`request_font_previews`]) so the system-font scan +
    /// parse is paid the first time the dropdown opens, never on Text-mode entry.
    static FONT_PREVIEWS: RefCell<Vec<FontPreview>> = const { RefCell::new(Vec::new()) };
    /// One-shot: the popover sets this when it has no previews yet; the shell reads
    /// it (`take_want_font_previews`), builds + publishes, and it stays quiet after.
    static WANT_FONT_PREVIEWS: Cell<bool> = const { Cell::new(false) };
    /// Chip rect stashed by the body paint when the font dropdown is open, taken by
    /// the deferred popover pass so the list paints ON TOP of every section.
    static PENDING_FONT_DD: Cell<Option<Rect>> = const { Cell::new(None) };
    /// Idem para o chip de CATEGORIA do catálogo de formas. Sem o passe diferido o
    /// `push_clip` do scroll cortaria o popover na borda da seção.
    static PENDING_GROUP_DD: Cell<Option<Rect>> = const { Cell::new(None) };
    /// Idem para os chips de PONTA do traço — mas guardando também o SLOT (0 = começo,
    /// 1 = fim): os dois seletores compartilham o passe diferido, e o popover precisa
    /// saber de quem ele é. Só um fica aberto por vez (abrir o outro fecha este, pelo
    /// dispatch genérico do dropdown).
    static PENDING_MARKER_DD: Cell<Option<(usize, Rect)>> = const { Cell::new(None) };
}

/// Which kind of fill the selected path has (published by the shell each frame so
/// the panel's Fill-type selector reflects + drives it). Mirror of the scene
/// `Paint` variants (kept panel-local so the panel needn't depend on the scene).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FillKind {
    Solid,
    Linear,
    Radial,
    MultiPoint,
}

/// Fill rule of a compound path — which nested contours are holes. Mirror of the
/// scene `FillRule` (kept panel-local so the panel needn't depend on the scene).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PathFillRule {
    NonZero,
    EvenOdd,
}

/// Retained per-instance state slot for `VectorPanel`. Intentionally empty —
/// the authoritative Style lives on the shell-side `VectorTool`; the panel
/// renders the per-frame snapshot. `Default` is required by the
/// `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct VectorPanelState;

/// Publish the current Style snapshot. Called by the shell once per frame while
/// the `vector` tool is active; pass `None` to clear (tool inactive).
pub fn set_current_vector_style(snapshot: Option<VectorStyleSnapshot>) {
    CURRENT_SNAPSHOT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the snapshot the host published this frame, falling back to
/// [`VectorStyleSnapshot::default`] when none was pushed.
pub(crate) fn current_snapshot() -> VectorStyleSnapshot {
    CURRENT_SNAPSHOT.with(|c| c.borrow().unwrap_or_default())
}

/// Publish the selected vertex's type (or `None` when no vertex is selected).
/// Called by the shell each frame while the `vector` tool is active.
pub fn set_selected_vertex_type(kind: Option<VertexType>) {
    CURRENT_VERTEX_TYPE.with(|c| *c.borrow_mut() = kind);
}

/// The selected vertex's type this frame (`None` ⇒ hide the Vertex section).
pub(crate) fn current_vertex_type() -> Option<VertexType> {
    CURRENT_VERTEX_TYPE.with(|c| *c.borrow())
}

/// Publish the selected path's anchor bbox `[x, y, w, h]` (world), or `None`.
/// Called by the shell each frame while the `vector` tool is active.
pub fn set_current_transform(bbox: Option<[f64; 4]>) {
    CURRENT_TRANSFORM.with(|c| c.set(bbox));
}

/// The selected path's bbox this frame (`None` ⇒ hide the Transform section).
pub(crate) fn current_transform() -> Option<[f64; 4]> {
    CURRENT_TRANSFORM.with(|c| c.get())
}

/// Publish the selected path's `closed` flag (or `None` when no path is selected).
pub fn set_current_path_closed(closed: Option<bool>) {
    CURRENT_PATH_CLOSED.with(|c| c.set(closed));
}

/// The selected path's `closed` flag this frame (drives the toggle button label).
pub(crate) fn current_path_closed() -> Option<bool> {
    CURRENT_PATH_CLOSED.with(|c| c.get())
}

/// Publish the selected path's fill kind + linear angle (both `None` when no path
/// is selected or it has no fill / isn't linear).
pub fn set_current_fill(kind: Option<FillKind>, angle_deg: Option<f64>) {
    CURRENT_FILL_KIND.with(|c| c.set(kind));
    CURRENT_GRAD_ANGLE.with(|c| c.set(angle_deg));
}

/// The selected path's fill kind this frame (`None` ⇒ hide the Fill-type selector).
pub(crate) fn current_fill_kind() -> Option<FillKind> {
    CURRENT_FILL_KIND.with(|c| c.get())
}

/// The selected path's linear-gradient angle this frame (`None` unless Linear).
pub(crate) fn current_grad_angle() -> Option<f64> {
    CURRENT_GRAD_ANGLE.with(|c| c.get())
}

/// Publish the selected multi-point gradient point's influence (`None` = no point).
pub fn set_current_grad_influence(v: Option<f64>) {
    CURRENT_GRAD_INFLUENCE.with(|c| c.set(v));
}

/// The selected multi-point point's influence this frame (drives the slider).
pub(crate) fn current_grad_influence() -> Option<f64> {
    CURRENT_GRAD_INFLUENCE.with(|c| c.get())
}

/// Publish the selected multi-point gradient point's jitter (`None` = no point).
pub fn set_current_grad_jitter(v: Option<f64>) {
    CURRENT_GRAD_JITTER.with(|c| c.set(v));
}

/// The selected multi-point point's jitter this frame (drives the slider).
pub(crate) fn current_grad_jitter() -> Option<f64> {
    CURRENT_GRAD_JITTER.with(|c| c.get())
}

/// Publish the number of paths in the object selection (drives Align/Distribute).
pub fn set_current_selection_count(n: usize) {
    CURRENT_SELECTION_COUNT.with(|c| c.set(n));
}

/// The object-selection path count this frame (≥2 shows Align, ≥3 shows Distribute).
pub(crate) fn current_selection_count() -> usize {
    CURRENT_SELECTION_COUNT.with(|c| c.get())
}

/// Publish the selected path's fill rule — `None` unless it is a compound path
/// (the Fill Rule row hides otherwise, since both rules would paint the same).
pub fn set_current_fill_rule(rule: Option<PathFillRule>) {
    CURRENT_FILL_RULE.with(|c| c.set(rule));
}

/// The selected compound path's fill rule this frame (`None` = not compound).
pub(crate) fn current_fill_rule() -> Option<PathFillRule> {
    CURRENT_FILL_RULE.with(|c| c.get())
}

/// Publish whether shape-snapping is on, so the Snap section reflects it.
pub fn set_current_snap(on: bool) {
    CURRENT_SNAP.with(|c| c.set(on));
}

/// Whether shape-snapping is on this frame.
pub(crate) fn current_snap() -> bool {
    CURRENT_SNAP.with(|c| c.get())
}

/// The angle the Angle chip last reported this gesture (for the delta emit).
pub(crate) fn rot_last() -> f64 {
    ROT_LAST.with(Cell::get)
}

/// Record the Angle chip's current value as the gesture baseline (or reset to 0
/// between gestures).
pub(crate) fn set_rot_last(v: f64) {
    ROT_LAST.with(|c| c.set(v));
}

#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

#[must_use]
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}

/// Publica se "Set Center" está armado (a shell manda; o painel só reflete).
pub fn set_current_pivot_edit(armed: bool) {
    CURRENT_PIVOT_EDIT.with(|c| c.set(armed));
}

pub(crate) fn pivot_edit_armed() -> bool {
    CURRENT_PIVOT_EDIT.with(|c| c.get())
}

/// Publica se a seleção tem forma viva convertível (habilita "Convert to Curves").
pub fn set_current_convertible(v: bool) {
    CURRENT_CONVERTIBLE.with(|c| c.set(v));
}

pub(crate) fn convertible() -> bool {
    CURRENT_CONVERTIBLE.with(Cell::get)
}

/// Publica se a seleção vive sob UM container de Envelope (ADR-0129) — habilita **Expand**/
/// **Release**, que só fazem sentido sobre um envelope existente. Quem responde no shell é o
/// `envelope_live::sole_container`, a MESMA porta que a seleção e o `dissolve` usam.
pub fn set_current_has_envelope(v: bool) {
    CURRENT_HAS_ENVELOPE.with(|c| c.set(v));
}

pub(crate) fn has_envelope() -> bool {
    CURRENT_HAS_ENVELOPE.with(Cell::get)
}

/// Publica se o envelope da seleção está no gesto **Mesh** (lados curvos) e não no **Perspective**.
///
/// Só é lido quando [`has_envelope`] é `true` — sem envelope não há gesto a mostrar. Um `bool` e não
/// um espelho do enum: o painel oferece exatamente dois chips, e um enum aqui obrigaria esta crate a
/// conhecer o `ph2d_ecs::EnvelopeKind` só para desenhar qual está aceso.
pub fn set_current_envelope_mesh(v: bool) {
    CURRENT_ENVELOPE_MESH.with(|c| c.set(v));
}

pub(crate) fn envelope_mesh() -> bool {
    CURRENT_ENVELOPE_MESH.with(Cell::get)
}

/// Publica os presets de gaiola: os rótulos (na ordem de `ph2d_ecs::EnvelopeWarp::ALL`), qual está
/// ativo e a força dele.
///
/// **O painel se auto-popula desta lista** — o mesmo idioma da rack de áudio (`set_fx_kind_names`):
/// acrescentar um preset é uma linha na tabela do componente e **zero mudança de painel**. Uma lista
/// escrita à mão aqui driftaria da tabela no primeiro preset novo.
pub fn set_current_envelope_presets(labels: &[&'static str], active: Option<usize>, bend: f64) {
    CURRENT_ENVELOPE_PRESETS.with(|c| {
        let mut v = c.borrow_mut();
        v.clear();
        v.extend_from_slice(labels);
    });
    CURRENT_ENVELOPE_WARP.with(|c| c.set(active));
    CURRENT_ENVELOPE_BEND.with(|c| c.set(bend));
}

pub(crate) fn envelope_presets() -> Vec<&'static str> {
    CURRENT_ENVELOPE_PRESETS.with(|c| c.borrow().clone())
}

pub(crate) fn envelope_warp() -> Option<usize> {
    CURRENT_ENVELOPE_WARP.with(Cell::get)
}

pub(crate) fn envelope_bend() -> f64 {
    CURRENT_ENVELOPE_BEND.with(Cell::get)
}

/// Publica se a seção Text deve aparecer (modo Text OU objeto de texto selecionado).
pub fn set_current_text_visible(v: bool) {
    CURRENT_TEXT_VISIBLE.with(|c| c.set(v));
}

pub(crate) fn text_visible() -> bool {
    CURRENT_TEXT_VISIBLE.with(Cell::get)
}

/// Publica a forma em FOCO: `Some(kind)` = há uma forma VIVA selecionada (os campos
/// dela aparecem e a editam, mesmo na ferramenta Select); `None` = os campos são os da
/// forma ativa do catálogo. A shell resolve o alvo e semeia os campos.
pub fn set_current_shape_focus(kind: Option<ShapeKind>) {
    CURRENT_SHAPE_FOCUS.with(|c| c.set(kind));
}

/// A forma em foco deste frame (`None` ⇒ cai na forma ativa do catálogo).
pub(crate) fn current_shape_focus() -> Option<ShapeKind> {
    CURRENT_SHAPE_FOCUS.with(Cell::get)
}

/// A aba de família aberta (o painel a define no clique; `None` = segue a forma ativa).
pub(crate) fn current_shape_group() -> Option<ShapeGroup> {
    CURRENT_SHAPE_GROUP.with(Cell::get)
}

pub(crate) fn set_current_shape_group(g: Option<ShapeGroup>) {
    CURRENT_SHAPE_GROUP.with(|c| c.set(g));
}

/// Publica a semente ONE-SHOT dos sliders de texto (só quando o alvo muda).
pub fn set_current_text_seed(seed: Option<[f64; 4]>) {
    TEXT_SEED.with(|c| c.set(seed));
}

/// O paint consome a semente (uma vez) e escreve no store.
pub(crate) fn take_text_seed() -> Option<[f64; 4]> {
    TEXT_SEED.with(|c| c.take())
}

/// Publica o texto da sessão de edição ativa (`None` = sem sessão de texto).
/// A shell chama a cada frame; o painel só o exibe (read-only na A2).
pub fn set_current_text(text: Option<String>) {
    CURRENT_TEXT.with(|c| *c.borrow_mut() = text);
}

/// O texto da sessão ativa este frame (`None` ⇒ nada a exibir).
pub(crate) fn current_text() -> Option<String> {
    CURRENT_TEXT.with(|c| c.borrow().clone())
}

/// Publica o rótulo da família de fonte corrente do texto (a shell resolve o nome,
/// inclusive o da embutida). `None` fora do modo Text.
pub fn set_current_text_font(name: Option<String>) {
    CURRENT_TEXT_FONT.with(|c| *c.borrow_mut() = name);
}

/// O rótulo da família de fonte corrente este frame (para o seletor `<` nome `>`).
pub(crate) fn current_text_font() -> Option<String> {
    CURRENT_TEXT_FONT.with(|c| c.borrow().clone())
}

/// Publica o alinhamento horizontal corrente do texto (`None` fora do modo Text).
pub fn set_current_text_align(align: Option<TextAlign>) {
    CURRENT_TEXT_ALIGN.with(|c| c.set(align));
}

/// O alinhamento corrente este frame (destaca o botão L/C/R ativo). `None` = default.
pub(crate) fn current_text_align() -> Option<TextAlign> {
    CURRENT_TEXT_ALIGN.with(Cell::get)
}

/// Um eixo de variação da fonte corrente exposto como campo numérico na seção Axes do
/// painel (fora o Weight, que tem slider próprio). A shell (dona do `VariableFont`)
/// publica nome + range + valor; o painel só desenha o campo e devolve o valor.
#[derive(Clone, Debug)]
pub struct TextAxisSlot {
    /// Nome legível do eixo (`"Optical Size"`, `"Width"`, `"Slant"`…).
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub value: f64,
}

/// Publica os eixos de variação da fonte corrente (sem `wght`), na ordem em que a
/// shell os aplica. Vazio fora do modo Text ou para fontes não-variáveis.
pub fn set_current_text_axes(axes: Vec<TextAxisSlot>) {
    CURRENT_TEXT_AXES.with(|c| *c.borrow_mut() = axes);
}

/// Roda `f` com os eixos publicados (sem clonar o `Vec`).
pub(crate) fn with_text_axes<R>(f: impl FnOnce(&[TextAxisSlot]) -> R) -> R {
    CURRENT_TEXT_AXES.with(|c| f(&c.borrow()))
}

/// Índice do parâmetro de forma cujo id de campo é `id` (`None` se não for um).
pub(crate) fn shape_field_index(id: NodeId) -> Option<usize> {
    (0..crate::ids::MAX_SHAPE_FIELD_SLOTS).find(|&i| crate::ids::vector_shape_field_id(i) == id)
}

/// Índice do parâmetro cujo **botão de escolha** é `id` (o gêmeo clicável do slot numérico).
pub(crate) fn shape_choice_index(id: NodeId) -> Option<usize> {
    (0..crate::ids::MAX_SHAPE_FIELD_SLOTS).find(|&i| crate::ids::vector_shape_choice_id(i) == id)
}

/// Índice da forma no catálogo cujo id de botão é `id`.
pub(crate) fn shape_index(id: NodeId) -> Option<usize> {
    (0..ph2d_tool_vector::shapes::SHAPES.len()).find(|&i| crate::ids::vector_shape_id(i) == id)
}

/// Índice da família cujo id de aba é `id`.
pub(crate) fn shape_group_index(id: NodeId) -> Option<usize> {
    (0..ph2d_tool_vector::shapes::ALL_GROUPS.len())
        .find(|&i| crate::ids::vector_shape_group_id(i) == id)
}

/// Índice do eixo de variação cujo id de campo é `id` (`None` se não for um). Casa
/// contra o espaço fixo de slots (`MAX_TEXT_VARIATION_AXES`).
pub(crate) fn text_axis_index(id: NodeId) -> Option<usize> {
    (0..crate::ids::MAX_TEXT_VARIATION_AXES).find(|&i| crate::ids::vector_text_axis_id(i) == id)
}

/// Uma família selecionável no dropdown de fonte, já com o **nome renderizado no
/// próprio contorno** dela (preview de estilo real). Construída pela shell (que tem
/// os `VariableFont`) e publicada via [`set_current_text_font_previews`]; o painel
/// só desenha `outline` na linha, sem tocar em nenhuma fonte.
#[derive(Clone, Debug)]
pub struct FontPreview {
    /// A chave da família (`None` = a embutida) — o que a shell aplica ao escolher.
    pub family: Option<String>,
    /// Rótulo de exibição (fallback em texto quando o contorno sai vazio, ex. uma
    /// família sem os glyphs do próprio nome).
    pub display: String,
    /// O nome da família desenhado na fonte dela, **em espaço-em (1 = units_per_em),
    /// y-up** e com o avanço acumulado em x. O painel escala/translada por `Affine`.
    pub outline: BezPath,
    /// Largura total do nome em espaço-em (para não desenhar além da linha).
    pub advance_em: f64,
}

/// Publica a lista de previews do dropdown de fonte (a shell constrói sob demanda).
pub fn set_current_text_font_previews(previews: Vec<FontPreview>) {
    FONT_PREVIEWS.with(|c| *c.borrow_mut() = previews);
}

/// Roda `f` com as previews publicadas (sem clonar o `Vec`/`BezPath`).
pub(crate) fn with_font_previews<R>(f: impl FnOnce(&[FontPreview]) -> R) -> R {
    FONT_PREVIEWS.with(|c| f(&c.borrow()))
}

/// O popover pede à shell que construa as previews (quando ainda não há nenhuma).
pub(crate) fn request_font_previews() {
    WANT_FONT_PREVIEWS.with(|c| c.set(true));
}

/// A shell drena o pedido (one-shot): `true` ⇒ construa + publique as previews.
pub fn take_want_font_previews() -> bool {
    WANT_FONT_PREVIEWS.with(|c| c.replace(false))
}

/// Índice da família cujo id de opção do dropdown é `id` (`None` se não for uma
/// opção de fonte). Casa contra as previews publicadas na ordem selecionável.
pub(crate) fn font_option_index(id: NodeId) -> Option<usize> {
    with_font_previews(|p| (0..p.len()).find(|&i| crate::ids::vector_text_font_option_id(i) == id))
}

/// O body-paint guarda o rect do chip quando o dropdown de fonte está aberto; o
/// pass diferido do popover o consome (para pintar POR CIMA das seções).
pub(crate) fn set_pending_font_dd(rect: Option<Rect>) {
    PENDING_FONT_DD.with(|c| c.set(rect));
}

pub(crate) fn take_pending_font_dd() -> Option<Rect> {
    PENDING_FONT_DD.with(|c| c.take())
}

/// Idem para o chip de CATEGORIA (o popover das famílias de forma).
pub(crate) fn set_pending_group_dd(rect: Option<Rect>) {
    PENDING_GROUP_DD.with(|c| c.set(rect));
}

pub(crate) fn take_pending_group_dd() -> Option<Rect> {
    PENDING_GROUP_DD.with(|c| c.take())
}

/// Idem para o chip de PONTA aberto (`slot` = 0 começo / 1 fim, + o rect do chip).
pub(crate) fn set_pending_marker_dd(slot_rect: Option<(usize, Rect)>) {
    PENDING_MARKER_DD.with(|c| c.set(slot_rect));
}

pub(crate) fn take_pending_marker_dd() -> Option<(usize, Rect)> {
    PENDING_MARKER_DD.with(|c| c.take())
}

/// `(slot, índice)` da opção de ponta cujo id é `id` (`None` se não for uma). Casa contra
/// o espaço FIXO de slots, como as outras fábricas de id do painel — a resolução não
/// depende de quantas pontas existem hoje.
pub(crate) fn marker_option(id: NodeId) -> Option<(usize, usize)> {
    (0..crate::ids::MARKER_SLOTS).find_map(|slot| {
        (0..crate::ids::MAX_MARKER_OPTIONS)
            .find(|&i| crate::ids::vector_marker_option_id(slot, i) == id)
            .map(|i| (slot, i))
    })
}

/// A família ATIVA do catálogo: a escolhida explicitamente, ou — sem escolha — a da forma
/// ativa (voltar à tool mostra a família do que se está desenhando). Única verdade sobre
/// isso: a grade, o chip e o popover leem daqui, senão os três discordariam.
pub(crate) fn active_shape_group(active: ShapeKind) -> ShapeGroup {
    current_shape_group().unwrap_or(ph2d_tool_vector::shapes::desc(active).group)
}

/// A chave i18n do rótulo de uma família. Vive AQUI (não em `ph2d-tool-vector::shapes`)
/// porque a chave é vocabulário de UI do painel, e o catálogo é do domínio da tool.
#[must_use]
pub(crate) fn group_i18n_key(group: ShapeGroup) -> &'static str {
    match group {
        ShapeGroup::Basic => "panel.vector.group.basic",
        ShapeGroup::Round => "panel.vector.group.round",
        ShapeGroup::Arrows => "panel.vector.group.arrows",
        ShapeGroup::Flow => "panel.vector.group.flow",
        ShapeGroup::Bubbles => "panel.vector.group.bubbles",
        ShapeGroup::Symbols => "panel.vector.group.symbols",
        ShapeGroup::Iso => "panel.vector.group.iso",
    }
}
