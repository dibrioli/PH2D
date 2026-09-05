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
use ph2d_tool_vector::shapes::ShapeGroup;
use ph2d_tool_vector::{TextAlign, VectorStyleSnapshot, VertexSel};
use ph2d_vec_scene::ShapeKind;
use ph2d_vector::BezPath;
use std::cell::{Cell, RefCell};

#[path = "state_text.rs"]
mod text;
pub use text::*;

/// **As CONTAGENS da seleção** — quantos caminhos, quantos nós. Irmão pelo teto de 600 LOC, e o
/// corte é por RESPONSABILIDADE: o resto deste arquivo publica *o que a seleção É*; ali mora
/// *quantos há*, a pergunta que decide se um gesto de CONJUNTO é sequer oferecido.
#[path = "state_counts.rs"]
mod counts;
pub(crate) use counts::{current_selection_count, current_vertex_count};
pub use counts::{set_current_selection_count, set_current_vertex_count};

/// **A LINHA DE CORTE existe?** — irmão pela mesma razão dos counts: é a pergunta que decide se
/// os dois botões do corte são oferecidos.
#[path = "state_cut.rs"]
mod cut;
pub(crate) use cut::cut_line_exists;
pub use cut::set_cut_line_exists;

/// **A BOOLEANA VIVA** — o modo dos oito botões (escolha do artista) e se há grupo booleano na
/// seleção (fato da cena). Irmão dos dois acima, e as duas metades têm donos diferentes.
#[path = "state_bool.rs"]
mod bool_state;
pub(crate) use bool_state::{bool_group_selected, bool_shape_row};
pub use bool_state::{bool_live_on, set_bool_group_selected, set_bool_live_on, set_bool_shape_row};

/// **O RECORTE e a MOLDURA da seleção** (plano UI/UX W0; separados em 2026-08-21) — o
/// `Option<bool>` responde *"a seleção oferece o recorte, e ele está ligado?"* para qualquer forma
/// FECHADA, e o bool ao lado responde *"…e ela é uma moldura?"*, que é outra pergunta.
#[path = "state_frame.rs"]
mod frame_state;
pub(crate) use frame_state::{frame_clip, frame_panel_open, frame_present};
pub use frame_state::{set_frame_clip, set_frame_panel_open, set_frame_present};

/// **RESIZE BOX** (plano UI/UX W3b) — o que a alça do gizmo faz ao objeto selecionado. Irmão do
/// `frame_clip` na forma (`Option<bool>` = *existe resposta* + *qual é*) e na razão: a verdade
/// mora no ECS, isto é a projeção por frame.
#[path = "state_resize_box.rs"]
mod resize_box_state;
pub(crate) use resize_box_state::resize_box;
pub use resize_box_state::set_resize_box;

/// **ESTA FORMA TEM TRAÇO?** (plano 34) — irmão do `resize_box` na forma e na razão. ⚠️ É a
/// **única** resposta a essa pergunta neste painel: ela substituiu o `TokenBindings::stroke_exists`.
#[path = "state_stroke.rs"]
mod stroke_state;
pub use stroke_state::{
    StrokePaintKind, set_current_brush, set_stroke_paint_kind, set_stroke_present,
};
pub(crate) use stroke_state::{current_brush, stroke_paint_kind, stroke_present};

/// **O ÍMÃ e as RÉGUAS** — irmão pelo teto de 600 LOC dos painéis, e o corte é por assunto: as
/// cinco chaves respondem *a que a ponta se agarra, e o que a borda do canvas mostra*, e nenhuma
/// outra parte deste arquivo fala do gesto de apontar.
#[path = "state_snap.rs"]
mod snap_state;
pub(crate) use snap_state::{
    current_rulers, current_snap, current_snap_crossings, current_snap_guides, current_snap_path,
};
pub use snap_state::{set_current_guides, set_current_snap, set_current_snap_position};

/// **AS ÂNCORAS da seleção** (plano UI/UX W3) — a regra do filho que NÃO flui.
#[path = "state_anchors.rs"]
mod anchor_state;
pub(crate) use anchor_state::anchor_state;
pub use anchor_state::{AnchorState, set_anchor_state};

/// **O COMPONENTE da seleção** (plano UI/UX W5) — que verbos de prefab fazem sentido agora.
#[path = "state_components.rs"]
mod component_state;
pub use component_state::{
    ComponentState, InstancePiece, VariantRow, set_component_state, set_instance_pieces,
    set_variant_rows, set_z_index,
};
pub(crate) use component_state::{
    component_state, instance_pieces, instance_pieces_beyond, variant_rows, variant_rows_beyond,
    z_index,
};

/// **O AUTO LAYOUT da seleção** (plano UI/UX W2, ADR-0153) — o fluxo da moldura, o comportamento
/// do filho, e o modo do recuo (este último panel-local).
#[path = "state_layout.rs"]
mod layout_state;
pub use layout_state::{LayoutFlow, LayoutItem, set_layout_flow, set_layout_item};
pub(crate) use layout_state::{layout_flow, layout_item, layout_pad_each, set_layout_pad_each};

/// **A PELE da seleção** (plano UI/UX W6.2) — que widget do catálogo esta forma veste.
#[path = "state_widget.rs"]
mod widget_state;
pub use widget_state::{WidgetSkinState, set_widget_skin_state};
pub(crate) use widget_state::{
    set_pending_icon_dd, take_pending_icon_dd, widget_kinds_beyond, widget_skin_state,
};

/// ⭐⭐⭐ **A APARÊNCIA da seleção** (estudo 42 item 2) — a opacidade e o modo de mistura do
/// OBJECTO, que são propriedades da forma e não da tinta dela.
#[path = "state_appearance.rs"]
mod appearance_state;
pub use appearance_state::{Appearance, set_current_appearance};
pub(crate) use appearance_state::{
    blend_option_index, current_appearance, set_pending_obj_blend_dd, take_pending_obj_blend_dd,
};

/// **OS ESTADOS de UI da seleção** (plano UI/UX W7) — que poses ela tem, e quanto tempo o tween
/// entre elas leva.
#[path = "state_ui_states.rs"]
mod ui_states_state;
pub(crate) use ui_states_state::ui_states_state;
pub use ui_states_state::{UiStatesState, set_ui_states_state};

/// ⭐ **QUAL POPOVER ESTÁ ABERTO neste quadro** — irmão pelo teto de 600 LOC, e o corte é por
/// ASSUNTO: os quatro slots respondem a **uma** pergunta (*que chip guardou o rect para o passe
/// diferido?*), e nenhuma outra parte deste ficheiro fala de popovers.
///
/// ⚠️ Eles nasceram aqui e mudaram-se quando o ficheiro bateu no teto — a linha de corte já era
/// esta.
#[path = "state_pending_dd.rs"]
mod pending_dd;
pub(crate) use pending_dd::{
    set_pending_blend_dd, set_pending_font_dd, set_pending_group_dd, set_pending_marker_dd,
    take_pending_blend_dd, take_pending_font_dd, take_pending_group_dd, take_pending_marker_dd,
};

/// **AS SETAS do Morph da seleção** (plano 32 W4) — que arestas a máquina tem, e qual delas a cena
/// está a percorrer.
#[path = "state_morph_states.rs"]
mod morph_states_state;
pub use morph_states_state::{MorphShapeRow, MorphStatesState, set_morph_states_state};
pub(crate) use morph_states_state::{
    morph_states_state, set_pending_morph_key_dd, take_pending_morph_key_dd,
};

/// **COM QUE TINTA a forma aparece** — o tipo de preenchimento, o ângulo do gradiente, os dois
/// números do ponto selecionado e a regra do caminho composto. Irmão pelo teto de 600 LOC, e o
/// corte é o mesmo que a `ph2d-vec-scene` fez no `lib.rs` desta linha: *com que tinta a forma
/// aparece* × *o que a forma É*.
#[path = "state_fill.rs"]
mod fill_state;
pub use fill_state::set_current_grad_jitter;
pub use fill_state::{
    PatternArt, TexturePatternRow, set_current_fill, set_current_fill_rule,
    set_current_grad_influence, set_current_texture_pattern,
};
pub(crate) use fill_state::{
    current_fill_kind, current_fill_rule, current_grad_angle, current_grad_influence,
    current_grad_jitter, current_texture_pattern,
};

/// **OS TOKENS da seleção** (plano UI/UX W4) — que propriedade dela segue um token, e qual.
#[path = "state_tokens.rs"]
mod token_state;
pub use token_state::{TokenBindings, set_token_bindings};
pub(crate) use token_state::{set_pending_token_dd, take_pending_token_dd, token_bindings};

thread_local! {
    static CURRENT_TEXT_VISIBLE: Cell<bool> = const { Cell::new(false) };
    static CURRENT_TEXT: RefCell<Option<String>> = const { RefCell::new(None) };
    static CURRENT_TEXT_FONT: RefCell<Option<String>> = const { RefCell::new(None) };
    static CURRENT_TEXT_ALIGN: Cell<Option<TextAlign>> = const { Cell::new(None) };
    /// A largura de refluxo corrente do texto (`None` = Auto, sem caixa). Publicada pela
    /// shell como todo knob de texto; quem a lê é a fileira **Width** e o slider que só vive
    /// no modo `Fixed`.
    static CURRENT_TEXT_WRAP: Cell<Option<f64>> = const { Cell::new(None) };
    static CURRENT_TEXT_AXES: RefCell<Vec<TextAxisSlot>> = const { RefCell::new(Vec::new()) };
    /// **A fonte corrente expõe o eixo `wght`?** — a TERCEIRA pergunta da tipografia, e a que
    /// faltava. ⚠️ Ela **não** é derivável do [`CURRENT_TEXT_AXES`]: aquela lista é *"os eixos
    /// ALÉM do peso"*, então uma variável só-de-peso publica-a VAZIA e continua a ter um Weight
    /// vivo. Nasce `false` de propósito — *um painel a quem ninguém disse nada não promete um
    /// controlo que não faz nada*; a shell publica-a a cada quadro, ao lado dos eixos.
    static CURRENT_TEXT_HAS_WEIGHT: Cell<bool> = const { Cell::new(false) };
    static WANT_FONT_PREVIEWS: Cell<bool> = const { Cell::new(false) };
    /// Live snapshot published by the host before each `paint`. `None` until
    /// the first push (panel paints defaults).
    static CURRENT_SNAPSHOT: RefCell<Option<VectorStyleSnapshot>> = const { RefCell::new(None) };
    /// Type of the currently-selected vertex (published by the shell each frame
    /// from the Pen). `None` = no vertex selected → the Vertex section hides.
    static CURRENT_VERTEX_TYPE: RefCell<Option<VertexSel>> = const { RefCell::new(None) };
    /// **Onde a seleção de nós está** — a MEDIANA das âncoras, em MUNDO e **já na unidade do
    /// artista** (a shell converte na fronteira, como faz para a bbox do Transform).
    ///
    /// ⚠️ `None` ⇒ as duas fileiras somem. Ele é separado do `CURRENT_VERTEX_TYPE` porque
    /// responde outra pergunta: um índice que já não existe não descreve vértice nenhum e sai da
    /// mediana, mas a seleção pode continuar a ter tipo.
    static CURRENT_VERTEX_POS: Cell<Option<[f64; 2]>> = const { Cell::new(None) };
    /// Selected path's anchor bbox `[x, y, w, h]` **na unidade do ARTISTA**, published each
    /// frame. `None` = no path selected → the Transform section hides.
    ///
    /// ⚠️ Dizia *(world)* e passou a mentir com a fronteira de display: a shell converte na
    /// publicação e na volta, e o painel nunca vê metros.
    static CURRENT_TRANSFORM: Cell<Option<[f64; 4]>> = const { Cell::new(None) };
    /// O SUFIXO da unidade que o artista escolheu (`"px"` / `"m"`), publicado pela shell.
    ///
    /// ⚠️ **É o sufixo, não a REGRA.** A conversão mora inteira na fronteira da shell
    /// (`LengthDisplay`), e o painel recebe números já na face do artista — este `&'static str`
    /// existe só para o cabeçalho poder DIZER em que unidade eles estão, que é o precedente do
    /// Inspector (`Position (px)`). Guardar aqui a escala seria a segunda cópia da regra.
    static LENGTH_SUFFIX: Cell<&'static str> = const { Cell::new("m") };
    /// Selected path's `closed` flag, published each frame (`None` = no selection).
    /// Drives the Close/Open toggle button's label.
    static CURRENT_PATH_CLOSED: Cell<Option<bool>> = const { Cell::new(None) };
    /// "Set Center" armado: a próxima pressão no canvas reposiciona a origem.
    static CURRENT_PIVOT_EDIT: Cell<bool> = const { Cell::new(false) };
    /// A seleção tem alguma forma VIVA (paramétrica/texto, com `VecShape`) —
    /// habilita o botão "Convert to Curves". Publicado pela shell.
    static CURRENT_CONVERTIBLE: Cell<bool> = const { Cell::new(false) };
    /// Mostrar a seção Text: modo Text OU um objeto de TEXTO selecionado (as configs
    /// do texto ficam visíveis enquanto ele for texto — não-curva — mesmo no Select).
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
    /// Rótulo da família de fonte corrente do texto (embutida ou do sistema),
    /// publicado pela shell. Exibido entre os botões `<`/`>` do seletor de fonte.
    /// Alinhamento horizontal corrente do texto (L/C/R), publicado pela shell no modo
    /// Text — destaca o botão ativo na seção Paragraph. `None` fora do modo Text.
    /// Eixos de variação da fonte corrente (sem `wght`), publicados pela shell — a
    /// seção Axes desenha um campo por eixo. Vazio fora do modo Text / fonte estática.
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
    /// Padrão de textura (plano 33).
    Pattern,
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

/// Publish **a MEDIANA das âncoras selecionadas**, em mundo e na unidade do artista.
pub fn set_current_vertex_pos(pos: Option<[f64; 2]>) {
    CURRENT_VERTEX_POS.with(|c| c.set(pos));
}

/// A mediana deste frame (`None` ⇒ as duas fileiras X/Y somem).
pub(crate) fn current_vertex_pos() -> Option<[f64; 2]> {
    CURRENT_VERTEX_POS.with(Cell::get)
}

/// Publish **o que a seleção de vértices tem em comum** (ou `None` quando não há vértice
/// selecionado). Chamado pela shell a cada frame com a tool `vector` ativa.
pub fn set_selected_vertex_type(sel: Option<VertexSel>) {
    CURRENT_VERTEX_TYPE.with(|c| *c.borrow_mut() = sel);
}

/// A seleção de vértices deste frame (`None` ⇒ esconde a seção Vertex).
pub(crate) fn current_vertex_type() -> Option<VertexSel> {
    CURRENT_VERTEX_TYPE.with(|c| *c.borrow())
}

/// Publica a bbox de âncoras do caminho selecionado `[x, y, w, h]` **já na unidade do artista**,
/// ou `None`. Chamada pela shell a cada frame com a ferramenta `vector` em mãos.
///
/// ⚠️ **Os quatro números chegam CONVERTIDOS, e é isso que mantém o painel unit-agnóstico.** A
/// régua do canvas, o Inspector e o painel de Grid Snap respondem à MESMA pergunta — *onde está
/// esta coisa?* — e antes desta wave este painel era o único que respondia em metros de mundo:
/// com os defaults (100 px/m, Pixels) os três diziam `150` e este dizia `1.5`.
///
/// A conversão vive inteira na fronteira da shell (`ph2d_editor::LengthDisplay`), nos dois
/// sentidos: o número sai UMA vez na face do artista e volta pela mesma porta.
pub fn set_current_transform(bbox: Option<[f64; 4]>) {
    CURRENT_TRANSFORM.with(|c| c.set(bbox));
}

/// The selected path's bbox this frame (`None` ⇒ hide the Transform section).
pub(crate) fn current_transform() -> Option<[f64; 4]> {
    CURRENT_TRANSFORM.with(|c| c.get())
}

/// Publica o sufixo da unidade que o artista lê (`"px"` / `"m"`) — o que o cabeçalho da seção
/// Transform mostra entre parênteses.
pub fn set_length_suffix(suffix: &'static str) {
    LENGTH_SUFFIX.with(|c| c.set(suffix));
}

/// O sufixo da unidade corrente.
pub(crate) fn length_suffix() -> &'static str {
    LENGTH_SUFFIX.with(Cell::get)
}

/// Publish the selected path's `closed` flag (or `None` when no path is selected).
pub fn set_current_path_closed(closed: Option<bool>) {
    CURRENT_PATH_CLOSED.with(|c| c.set(closed));
}

/// The selected path's `closed` flag this frame (drives the toggle button label).
pub(crate) fn current_path_closed() -> Option<bool> {
    CURRENT_PATH_CLOSED.with(|c| c.get())
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

/// O estado dos **Effects** (ADR-0132) — módulo irmão pelo teto de 600 LOC deste arquivo.
#[path = "state_effects.rs"]
mod effects;
pub use effects::{FalloffRole, FxParamView, FxRowView, set_current_effects};
pub(crate) use effects::{has_target, kinds, stack};

/// O estado do **Envelope** (ADR-0129) — módulo irmão pelo mesmo teto de 600 LOC.
#[path = "state_envelope.rs"]
mod envelope;
pub(crate) use envelope::{
    envelope_bend, envelope_mode, envelope_presets, envelope_warp, has_envelope,
};
pub use envelope::{
    set_current_envelope_mode, set_current_envelope_presets, set_current_has_envelope,
};

/// O estado do **Text on Path** (plano 22) — módulo irmão pelo mesmo teto de 600 LOC.
#[path = "state_textpath.rs"]
mod textpath;
pub(crate) use textpath::{can_link, flip, linked, offset};
pub use textpath::{set_current_textpath, set_current_textpath_can_link};

#[path = "state_patternpath.rs"] // Pattern on Path (plano 23), irmão pelo teto de 600 LOC
mod patternpath;
pub(crate) use patternpath::{
    can_link as pp_can_link, can_pick as pp_can_pick, end as pp_end, flip as pp_flip,
    linked as pp_linked, offset as pp_offset, rotation as pp_rotation, spacing as pp_spacing,
    start as pp_start,
};
pub use patternpath::{
    set_current_patternpath, set_current_patternpath_can_link, set_current_patternpath_can_pick,
};

#[path = "state_expand.rs"] // Os knobs do Offset Path, irmão pelo teto de 600 LOC
mod expand;
pub use expand::{expand_join, expand_side, set_expand_join, set_expand_side};

#[path = "state_contour.rs"] // Contour (pesquisa `20_*` #9), irmão pelo teto de 600 LOC
pub(crate) mod contour;
pub use contour::{set_current_contour, set_current_contour_can_add};

#[path = "state_filters.rs"] // Filters (FX raster, plano 24), irmão pelo teto de 600 LOC
pub(crate) mod filters;
pub use filters::{
    FILTER_DETAIL_MAX, FilterKindView, FilterRowView, RAMP_PREVIEW_N, selected_stop,
    set_current_filter_can_add, set_current_filters, set_filter_blend_names, set_filter_kinds,
};

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

/// O paint consome a semente (uma vez) e escreve no store.
pub(crate) fn take_text_seed() -> Option<[f64; 4]> {
    TEXT_SEED.with(|c| c.take())
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

/// Roda `f` com as previews publicadas (sem clonar o `Vec`/`BezPath`).
pub(crate) fn with_font_previews<R>(f: impl FnOnce(&[FontPreview]) -> R) -> R {
    FONT_PREVIEWS.with(|c| f(&c.borrow()))
}

/// O popover pede à shell que construa as previews (quando ainda não há nenhuma).
pub(crate) fn request_font_previews() {
    WANT_FONT_PREVIEWS.with(|c| c.set(true));
}

/// Índice da família cujo id de opção do dropdown é `id` (`None` se não for uma
/// opção de fonte). Casa contra as previews publicadas na ordem selecionável.
pub(crate) fn font_option_index(id: NodeId) -> Option<usize> {
    with_font_previews(|p| (0..p.len()).find(|&i| crate::ids::vector_text_font_option_id(i) == id))
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
