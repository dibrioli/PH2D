//! [`WidgetStore`] — retained per-widget interactive state.
//!
//! Pre-populated when a screen is constructed (typically inside
//! `Screen::new`); during the hot path callers only read or
//! mutate-in-place via [`WidgetStore::get_mut`]. Inserts only happen
//! at construction time via [`WidgetStore::register`] — see ADR-0024
//! §"Plano de conformidade HR-3".
//!
//! `NodeId` is the AccessKit-canonical identity (re-exported from
//! `ph2d_a11y`) so the store and the `accesskit::Tree` share keys
//! without translation.
//!
//! Note on `BTreeMap` over `HashMap`: workspace clippy bans
//! `HashMap` everywhere (HR-5/ADR-0022). `BTreeMap` allocates per
//! insert, but inserts only happen at construction time via
//! [`WidgetStore::register`]; the hot path uses `get`/`get_mut` on
//! existing entries, which is allocation-free. Lookup is O(log n)
//! instead of O(1), trivial at editor widget counts (~50).

mod blender_ops;
mod chrome_ops;
mod collapse_ops;
mod input_map_ops;
pub use input_map_ops::capture_if_listening;
/// **O que um estado de widget É** (o `InteractiveState` + a paleta nomeada).
mod kinds;
/// O PIE MENU (E4) — irmão do `chrome_ops` por assunto e pelo teto de LOC.
mod radial_ops;
pub use kinds::{InteractiveState, NamedPalette};

mod asset_drag_ops;
mod dock_width_ops;
mod graph_ops;
mod number_scrub;
mod panel_ops;
mod slot_ops;
pub use slot_ops::{TAB_DRAG_THRESHOLD_PX, TabDragAnchor};
mod store_census;
mod store_core;
mod store_hierarchy;
mod timeline_ops;
mod widget_accessors;

use ph2d_a11y::NodeId;
use std::collections::BTreeMap;

use super::ContextMenuRequest;
use super::drag::{
    HierarchyDragState, NumberInputDragState, NumberStepperHoldState, ScrollbarDragAnchor,
};
use super::types::{GraphGesture, GraphKey, GraphZoom, NoteData, TimelineGesture, TimelineWheel};
use super::util::format_number;

use crate::widget::{
    ButtonState, CheckboxState, DropdownState, TagState, TextInputState, ToggleState,
};
use crate::zones::Rect;

#[derive(Debug, Default)]
pub struct WidgetStore {
    pub(super) states: BTreeMap<NodeId, InteractiveState>,
    /// Insertion order, used for keyboard Tab traversal.
    pub(super) focus_order: Vec<NodeId>,
    pub(super) hot_id: Option<NodeId>,
    pub(super) active_id: Option<NodeId>,
    pub(super) focus_id: Option<NodeId>,
    /// Rect of the active widget at the moment of Down. Used by
    /// drag dispatch (Slider) to compute new value from pointer
    /// position relative to the original geometry.
    pub(super) active_rect: Option<Rect>,
    /// Slider id ↔ NumberInput id pairs that should mirror each
    /// other's value. When the slider's value changes via drag, the
    /// number input's `value` (and `buffer`, when not focused) is
    /// updated; when the number input's buffer commits via Enter or
    /// Blur, the slider's value is updated. Pre-populated by the
    /// hosting screen at construction time.
    pub(super) slider_to_number: BTreeMap<NodeId, NodeId>,
    pub(super) number_to_slider: BTreeMap<NodeId, NodeId>,
    /// Affine projection `(scale, offset)` such that
    /// `chip_display_value = slider_storage * scale + offset`,
    /// keyed by chip id. Default (when missing) = `(1.0, 0.0)` —
    /// identity, matching the legacy `link_slider_number` contract.
    /// Mapped links (`link_slider_number_mapped`) are the canonical
    /// way to wire a slider+chip pair when the chip's painted unit
    /// differs from the slider's 0..1 storage (Grow `±1`, Min Px
    /// integer count, ...). Without this map the chip's keyboard
    /// commit silently writes display-space text into the slider as
    /// if it were storage — the 2026-05-27 "type 0.2 see -0.6" bug.
    pub(super) number_to_slider_mapping: BTreeMap<NodeId, (f32, f32)>,
    /// Per-NumberInput **(min, max, step)** range, registered by panels via `set_number_range`. The
    /// drag-scrub then maps the cursor displacement PROPORTIONALLY to `[min, max]` (a fixed drag spans
    /// the whole range regardless of magnitude) + clamps to it, and the stepper increments by `step`
    /// (Enio 2026-06-25 — a `±1` box no longer races past 100 on a few pixels).
    pub(super) number_range: BTreeMap<NodeId, (f64, f64, f64)>,
    /// Per-widget horizontal drag-scrub rate in **value-units per cursor pixel**,
    /// for UNBOUNDED number boxes (no `number_range`): the drag adds `rate·dx`
    /// (vertical = `rate/10`, precise) with NO clamp. Lets a panel calibrate the
    /// scrub feel for a large-magnitude, limitless field (e.g. world-space Vector
    /// Transform X/Y/W/H) without the range-proportional model's implicit bounds.
    pub(super) number_drag_rate: BTreeMap<NodeId, f64>,
    /// Chip ids that should `.round()` their typed display value before
    /// inverse-projecting into the slider's `0..1` storage. Used for
    /// integer-domain chips (Min Px / Tile Grid / Posterize Dither
    /// Grain) so the chip's persisted value matches the painter's
    /// rounded `display_override` — without this, typing "50.5" left
    /// the chip stuck at 50.5 while the painter showed "50" (audit
    /// finding #3, 2026-05-28).
    pub(super) number_to_slider_snap_integer: std::collections::BTreeSet<NodeId>,
    /// NumberInput ids that are painted as bare `paint_number_chip`
    /// pills (no up/down arrows). The dispatch's
    /// `apply_number_stepper_if_hit` carves a stepper column out of
    /// the right edge of EVERY NumberInput's hit rect by default —
    /// fine for the boxed `paint_number_input_with_buffer` widget
    /// (Inspector position etc.) which paints arrows visually, but
    /// for pill chips it produces phantom-stepper continuous-hold
    /// (mouse stopped, value still climbing). Membership here makes
    /// the dispatch skip the stepper hit-test for the id.
    pub(super) chips_without_steppers: std::collections::BTreeSet<NodeId>,
    /// NumberInput ids that commit on an explicit ENTER even when the typed value
    /// equals the one already shown. Opt-in (default off), for a chip whose
    /// displayed value is a DERIVED readout distinct from an AUTHORED state — the
    /// timeline Dur(s) box shows the content's derived end, and typing that same
    /// number must still author the explicit duration (Enio, 2026-07-23). Every
    /// other chip keeps the delta-gate that suppresses a same-value commit.
    pub(super) number_commit_always: std::collections::BTreeSet<NodeId>,
    /// NodeIds eligible for collapse-toggle on left-click. Populated by
    /// `pre_populate` / panel `populate` for every `paint_section_header`
    /// site. The dispatch consults this set to decide whether a click
    /// on `id` should flip the section's collapse state (vs. doing
    /// nothing). Separate from `collapsed` (defined in chrome_ops:
    /// `is_collapsed` / `toggle_collapsed`) because the absence of a
    /// key in `collapsed` means "open by default", not "not
    /// collapsible" — without this guard the dispatch couldn't tell a
    /// section header click from any other Plain hit.
    pub(super) collapsible_sections: std::collections::BTreeSet<NodeId>,
    /// Hex `TextInput` id → its parent `BlenderPicker` id, so the
    /// dispatch can parse the typed buffer on Enter / blur and apply
    /// the resulting color to the parent state.
    pub(super) hex_to_blender_parent: BTreeMap<NodeId, NodeId>,
    /// Active-palette rename `TextInput` field id → parent `BlenderPicker`, and the reverse (parent →
    /// field). Enter on the field renames the active palette to the typed buffer; the dispatch syncs
    /// the buffer to the active name whenever the active palette changes.
    pub(super) palette_name_to_parent: BTreeMap<NodeId, NodeId>,
    pub(super) parent_to_palette_name: BTreeMap<NodeId, NodeId>,
    /// Channel `NumberInput` chip id → (parent `BlenderPicker`,
    /// channel index 0..=3). Lets dispatch rewrite the parent's
    /// color value when the user commits a new channel value.
    pub(super) blender_channel_chip: BTreeMap<NodeId, (NodeId, u8)>,
    /// Most recent pointer-Down event, used for double-click
    /// detection. Stores the hit `NodeId` (or `None` if the click
    /// missed every widget) and the event timestamp.
    pub(super) last_down_id: Option<NodeId>,
    pub(super) last_down_at_ns: u128,
    /// `Some(id)` between a double-click Mouse Down and the matching
    /// Up — `apply_click` consumes this to upgrade `Click(id)` →
    /// `DoubleClick(id)`. Reset on every confirmed take.
    pub(super) pending_double_click: Option<NodeId>,
    /// Named colour palettes per BlenderPicker — a `Vec<NamedPalette>` (≥1) per parent picker id; the
    /// active index is the picker state's `active_palette`. Seeded at populate time; mutated by the
    /// "+ swatch" / delete / import / and palette-CRUD (new / rename / delete / select) dispatch paths.
    pub(super) blender_palettes: BTreeMap<NodeId, Vec<NamedPalette>>,
    /// Per-picker drag offset (dx, dy) applied to the rect chosen by
    /// the host painter. Mutated by drag-handle clicks; defaults to
    /// (0, 0). When the drag handle is `active`, `drag_anchor_px`
    /// stores the (cursor.x − rect.x, cursor.y − rect.y) at Down so
    /// Move events can keep the picker stuck to the cursor.
    pub(super) blender_picker_offset: BTreeMap<NodeId, (f32, f32)>,
    /// In-progress picker drag: (parent_id, cursor_x_at_down,
    /// cursor_y_at_down, offset_x_at_down, offset_y_at_down). Move
    /// events compute `new_offset = offset_at_down + (cursor − down_cursor)`.
    /// Cleared on pointer Up.
    pub(super) blender_drag_anchor: Option<(NodeId, f32, f32, f32, f32)>,
    /// Per-panel manual resize delta (dw, dh) applied on top of the
    /// layout's base width/height. Mutated by dragging the bottom-
    /// right resize gripper.
    pub(super) panel_resize_delta: BTreeMap<NodeId, (f32, f32)>,
    /// In-progress panel resize: (parent_id, last_cursor_x,
    /// last_cursor_y). Move events apply (cursor − last) to the
    /// stored `panel_resize_delta`, then re-anchor.
    pub(super) panel_resize_anchor: Option<(NodeId, f32, f32)>,
    /// In-progress panel resize from the bottom-LEFT corner — same
    /// shape as [`panel_resize_anchor`] but the Move handler also
    /// shifts the panel's stored offset (`panel_drag_offset`) so the
    /// right edge stays anchored. Companion field rather than a mode
    /// tag because only one resize is active at a time and the dispatch
    /// can check both Options cheaply.
    pub(super) panel_resize_anchor_bl: Option<(NodeId, f32, f32)>,
    /// ⭐⭐ **AS EXCEPÇÕES de onde cada painel está** (decisão D4) — só quem o artista MOVEU. Quem
    /// não está aqui responde pelo `Panel::DEFAULT_SLOT`. Ver `state::slot_ops`.
    pub(super) panel_slot: std::collections::BTreeMap<NodeId, crate::screens::slot::Slot>,
    /// Arrasto de uma aba em curso — ver [`TabDragAnchor`]. O início fica guardado porque é a
    /// distância a ele que separa um **clique** (trocar de aba) de um **arrasto** (mudar de
    /// encaixe).
    pub(super) tab_drag: Option<TabDragAnchor>,
    /// Onde uma aba foi largada, à espera de quem saiba julgar: o hero tem o layout e o
    /// `ALLOWED_SLOTS`, o store não tem nenhum dos dois.
    pub(super) tab_drop: Option<(NodeId, (f32, f32))>,
    /// ⭐ **O layout por tarefa activo** (D7) — qual aba da barra de cima está escolhida.
    pub(super) active_layout: crate::screens::task_layout::TaskLayout,
    /// Clipboard outbox — set by Cmd+C/X handlers; shell drains each
    /// frame via `take_clipboard_copy` and writes to the OS
    /// clipboard. `String` rather than a reference so the data lives
    /// independently of any widget buffer that might mutate next.
    pub(super) pending_clipboard_copy: Option<String>,
    /// Clipboard paste request — set by Cmd+V on a focused text
    /// widget; shell reads the OS clipboard and calls back into
    /// `apply_clipboard_paste` with the text.
    pub(super) pending_clipboard_paste: Option<NodeId>,
    /// Currently-loaded scene name shown on the TopBar project chip.
    /// Mutated by `ContextMenuKind::SceneList` row clicks.
    pub(super) current_scene_name: String,
    /// Coordinate-space toggle for the TOOL_SPACE rail button.
    /// `false` = Global, `true` = Local. Flipped on click.
    pub(super) tool_space_local: bool,
    /// Camera-framing mode for the TOOL_HOME rail button.
    /// Cycle: 0 = Selected, 1 = Camera, 2 = All. Bumped on click.
    pub(super) tool_view_mode: u8,
    /// `true` while the Painter rail's **Shapes** flyout is revealed (the
    /// column of shape chips painted to the right of the Shapes button).
    /// Opened on press of the Shapes button, closed on shape pick / press
    /// elsewhere. Transient UI state, like the dropdown popover flags.
    pub(super) painter_shapes_flyout_open: bool,
    /// `true` while the Painter rail's **Mask** group flyout is revealed (the
    /// column of Mask + Selection chips to the right of the Mask button).
    /// Same lifecycle as [`Self::painter_shapes_flyout_open`]; the two are
    /// mutually exclusive (opening one closes the other in dispatch).
    pub(super) painter_mask_flyout_open: bool,
    /// Per-panel Z order — last element paints LAST (= topmost).
    /// Mutated by `bump_panel_z` whenever the user clicks inside a
    /// panel, drags it, or it newly opens (color picker). Painters
    /// walk this in order so the most-recently-touched panel sits
    /// on top of any overlapping siblings.
    pub(super) panel_z_order: Vec<NodeId>,
    /// Eyedropper pending: when Some(parent), the next pointer Down
    /// (anywhere except on the eyedropper button itself) is intercepted
    /// by the dispatch and emitted as `WidgetEvent::EyedropperPick`,
    /// signaling the host to readback the pixel under the cursor.
    pub(super) eyedropper_pending: Option<NodeId>,
    /// Palette import/export pending: when `Some((picker, kind))`, the host opens a file dialog next
    /// frame and loads/saves the picker's palette (see [`WidgetStore::take_palette_io_pending`]).
    pub(super) palette_io_pending: Option<(NodeId, crate::interaction::PaletteIoKind)>,
    /// Picker (parent id) whose palette-select dropdown popover is OPEN, else `None`. Transient UI
    /// state; cleared when the picker is dismissed (see [`WidgetStore::set_picker_target`]).
    pub(super) palette_dropdown_open: Option<NodeId>,
    /// Vertical scroll offset per panel. Wheel events advance the
    /// offset; painters subtract it from content y. Clamped on each
    /// scroll to `[0, content_h - visible_h]` by the painter (which
    /// knows both heights). See `docs/UI_Bugs/README.md` §1
    /// (hit-testing) — content rendered with offset must compensate
    /// in hit-test too.
    pub(super) panel_scroll: BTreeMap<NodeId, f32>,
    /// O deslocamento VIVO; o alvo é o `panel_scroll`. Argumento na porta: `panel_scroll()`.
    pub(super) panel_scroll_live: BTreeMap<NodeId, f32>,
    /// **Quanto do hover está PRESENTE**, por id — o `t` vivo que o tique da UI viva integra a
    /// partir dos alvos que o [`WidgetStore::hover_targets`] publica.
    ///
    /// ⚠️ **Mora aqui pela razão EXACTA do `panel_scroll_live`, e o precedente é medido:** aquele
    /// vivo deu suavidade a ~130 sítios sem uma linha de mudança, porque a porta que eles já
    /// perguntavam passou a devolvê-lo. O `t` do hover tem a mesma forma — um escalar por id, um
    /// escritor (o tique), muitos leitores (os pintores) — e a alternativa era somar um parâmetro
    /// `motion` a toda a corrente que já leva o `store`: **56 assinaturas só no
    /// `ph2d-panel-inspector`**, para 20 botões.
    ///
    /// ⚠️ **Não é uma segunda cópia do mesmo facto:** o alvo é *o rato está em cima?* e o vivo é
    /// *quanto do aceso já chegou*. Quem os concilia é o [`crate::motion::UiMotion`], que continua
    /// a ser o dono do relógio, do carácter e do *reduced motion* — o store carrega o NÚMERO, não
    /// a lei que o produz.
    pub(super) hover_live: BTreeMap<NodeId, f32>,
    /// Painter-published rect of each scrollable panel — populated
    /// every frame so the wheel dispatch can find which panel sits
    /// under the cursor. Cleared together with `clear_for_frame` on
    /// the hit_index by the host (or hero) at frame start.
    pub(super) panel_rects: BTreeMap<NodeId, Rect>,
    /// ⭐⭐ **As SUB-REGIÕES roláveis** — uma região dentro de um painel que rola por conta própria.
    ///
    /// ⚠️ **Slot próprio, e não o `panel_rects`:** o `panel_at` é consultado por muita gente (o
    /// menu de contexto, a queda de um asset, a roda) e poluí-lo faria uma sub-região responder
    /// *«sou um painel»* a todas elas. É a mesma separação que o popover do dropdown já faz.
    pub(super) sub_scroll_rects: BTreeMap<NodeId, Rect>,
    /// Painter-published total content height per panel (sum of
    /// every section's height + separators). `dispatch_wheel` reads
    /// this to clamp scroll deltas at the upper bound
    /// (`content_h - visible_h`) — without it, wheeling past the
    /// last element produces a one-frame "jump" as the next paint
    /// clamps the over-scroll back.
    pub(super) panel_content_h: BTreeMap<NodeId, f32>,
    /// Exact visible body height per panel, also painter-published.
    /// Pairs with `panel_content_h` so `dispatch_wheel` can compute
    /// `max_scroll = content_h - visible_h` precisely (no heuristic).
    pub(super) panel_visible_h: BTreeMap<NodeId, f32>,
    /// Tooltip text per widget id. Read by `paint_hover_tooltip`
    /// when the user hovers over a registered widget. Populated by
    /// `populate` / paint passes via `set_tooltip`. Replaces the old
    /// hardcoded `tooltip_for(id)` match — every widget can now
    /// participate without per-id boilerplate.
    pub(super) tooltips: BTreeMap<NodeId, String>,
    /// Collapsed/expanded state per id. `true` = collapsed; missing
    /// entry defaults to "expanded" so newly-registered sections
    /// open by default. Toggled by `apply_event` on Click and
    /// consumed by section painters that early-out when collapsed.
    pub(super) collapsed: BTreeMap<NodeId, bool>,
    /// **Quanto de cada secção está ABERTO** — o `t` VIVO que o chevron veste, publicado pelo
    /// tique (`set_section_open_live`) e semeado pela partida no `toggle_collapsed`.
    ///
    /// ⚠️ Mapa PARALELO ao `collapsed`, e não um campo dentro dele: aquele é o estado
    /// **semântico** (o que o painel consulta para decidir se pinta as rows, e o que um gate
    /// compara), este é APARÊNCIA. Misturá-los faria cada `assert_eq!` de estado passar a
    /// depender de *quando* foi lido — o mesmo argumento que mantém o `UiMotion` fora do
    /// `InteractiveState`.
    pub(super) fold_live: BTreeMap<NodeId, f32>,
    /// **A altura que o CORPO de cada secção mediu da última vez que foi pintado.**
    ///
    /// ⚠️ **É a única coisa neste store escrita através de um `&self`, e a razão é ESTRUTURAL,
    /// não conveniência:** o recorte da dobra precisa da altura **antes** de pintar (a altura só
    /// se sabe depois de percorrer as rows), e quem a mede é o pintor — que, pela API do próprio
    /// host, **nunca** pode segurar `&mut WidgetStore` e `&mut HitIndex` ao mesmo tempo
    /// (`PanelHostInternal::store_and_hit_index_mut` devolve o par `(&, &mut)` de propósito, para
    /// a dança de borrows sobre um `dyn`). As alternativas medidas eram: um parâmetro `&mut` a
    /// cascatear por ~40 assinaturas em 10 crates, ou um mapa global fora do store — que
    /// vazaria medições entre dois `WidgetStore` do mesmo processo de teste.
    ///
    /// ⚠️ **Um valor velho por um quadro é INOFENSIVO por construção:** ele alimenta **só** o
    /// recorte, nunca o `y` de saída (esse sai da medição FRESCA do próprio quadro), e a altura
    /// de um corpo não muda enquanto ele se dobra — o que muda é o `t`. Ausente ⇒ recorte zero:
    /// a secção que nunca foi pintada aberta não aparece no primeiro quadro da estreia e mede-se
    /// nele, o que é invisível porque a mola parte de `t ≈ 0`.
    pub(super) fold_body_h: core::cell::RefCell<BTreeMap<NodeId, f32>>,
    /// Pending right-click context menu. `Some` when a Secondary
    /// Down landed somewhere a menu should appear (e.g. an empty
    /// inspector panel or a section header); `None` when no menu
    /// is open. The hero painter consumes this to render a floating
    /// menu over everything; clicking outside the menu or on a menu
    /// item clears the slot.
    pub(super) context_menu: Option<ContextMenuRequest>,
    /// New-image modal: the currently-selected square size (px) + background (`0` transparent / `1`
    /// black / `2` white). The two radio groups in the modal write these; Create reads them.
    pub(super) new_image_size: u32,
    pub(super) new_image_bg: u8,
    /// Set by the modal's Create button — `Some((size_px, bg))` the shell polls + clears to spawn a
    /// blank canvas. Decouples the editor-core modal from the shell's `spawn_blank_canvas` (no I/O here).
    pub(super) new_image_request: Option<(u32, u8)>,
    /// Resolução escolhida no modal da folha, em pixels de lado.
    pub(super) sheet_size: u32,
    /// Pedido pendente de criação de folha, em pixels de lado — a shell drena-o.
    pub(super) sheet_size_request: Option<u32>,
    /// Fill (Bucket) "Fill adjust" floating modal: `Some((x, y))` = the card's top-left in screen px
    /// (open); `None` = closed. Opened at the ColorDrop release point by the shell; dragging the title
    /// band offsets it. Its threshold slider's value lives in the `PAINTER_FILL_MODAL_SLIDER` widget.
    /// `(posição, âncora)` — as duas no MESMO campo, ver `open_fill_modal`.
    pub(super) fill_modal: Option<((f32, f32), (f32, f32))>,
    /// **A janela do INPUT MAP**: onde ela está, ou `None` se estiver fechada (plano 30 §0.2).
    ///
    /// ⚠️ **Uma só posição, e não a âncora dupla do `fill_modal` acima:** aquele nasce no ponto em
    /// que o dedo largou a tinta e precisa de lembrar onde foi; esta abre-se por um gesto de menu e
    /// só tem *onde está agora*.
    pub(super) input_map_window: Option<(f32, f32)>,
    /// **Qual acção está à ESCUTA de uma tecla** (o gesto *press-to-bind*).
    ///
    /// ⚠️ Enquanto isto é `Some`, a próxima tecla é **conteúdo**, não atalho — e é o despacho de
    /// teclado que tem de perguntar primeiro. Sem essa ordem, ligar `S` a uma acção salva o
    /// projecto e a ligação nunca acontece.
    pub(super) input_map_listening: Option<ph2d_input::ActionId>,
    /// **A tecla que a escuta acabou de apanhar**, à espera de quem a saiba ligar.
    ///
    /// ⚠️ Ela vive um instante: o despacho de teclado escreve-a e o handler de chrome **drena-a**
    /// no mesmo quadro. Guardá-la sem drenar faria a mesma tecla ligar-se outra vez ao gesto
    /// seguinte — a família do canal que se lê com `take`, que este repo já pinou noutro sítio.
    pub(super) input_map_captured: Option<ph2d_input::Key>,
    /// **Quanto a lista da janela está rolada**, em px (`0` = topo).
    ///
    /// ⛔ Report do Enio (2026-08-24): *"estreito e **sem scroll**"*. Um cartão que cresce com a
    /// lista sai do ecrã, e a última acção fica inalcançável — pior que uma lista curta, porque
    /// nada na tela o diz.
    pub(super) input_map_scroll: f32,
    /// Onion settings floating modal (ADR-0142 W3b): `Some((x, y))` = the card's top-left in screen
    /// px (open); `None` = closed. Opened by the timeline's Onion-settings button (shell-side, so it
    /// can seed the widgets from `TimelineState::onion`); dragging the title band offsets it. Its
    /// slider/swatch values live in the `TIMELINE_ONION_MODAL_*` widgets — the shell reads them back
    /// into `TimelineState::onion` each frame the modal is open (WidgetStore is the shared blackboard).
    pub(super) onion_modal: Option<(f32, f32)>,
    /// Full-screen command palette (Motion's "Add Node"): `Some(model)` = open, painted over the whole
    /// app by `chrome::paint_command_palette`; `None` = closed. The model (grouped, coloured items) is
    /// set ONCE on open by the shell — never rebuilt per frame — mirroring `open_onion_modal`'s
    /// value-seeding. The picked item id lands in `command_pick` for the shell to read back and route.
    pub(super) command_palette: Option<crate::widget::command_palette::PaletteModel>,
    /// **O PIE MENU aberto** (estudo de UI viva, E4) — centro, itens e o sector aceso.
    ///
    /// ⚠️ Ele mora aqui, ao lado da paleta, porque é a **outra vista da mesma lista**: as duas são
    /// chrome de tela cheia que o pintor lê do store e que o teclado abre.
    pub(super) radial: Option<crate::widget::RadialOpen>,
    /// The item id the user picked in the command palette, awaiting the shell's read-back (`take_command_pick`).
    /// `None` = nothing picked since the last take. This is the generic-widget/shell-routes seam (mirrors the
    /// colour picker's `picker_target` read-back), so editor-core never learns what an item *means*.
    pub(super) command_pick: Option<NodeId>,
    /// The live search text typed into the open palette (empty = show everything). The shell feeds
    /// characters here while the palette is open (it is a full-screen modal, so it eats the keys); the
    /// widget filters the model by it, and `Enter` picks the top match. Cleared on open/close so a fresh
    /// palette never inherits the last query.
    pub(super) command_palette_query: String,
    /// ⭐ **A caixa da banda foi clicada** (ADR-0166 / F3), à espera de quem a abriu.
    ///
    /// ⚠️ **É um sinal, não o estado.** O widget não sabe o que *"mostrar tudo"* significa — quem
    /// abriu a paleta é que sabe, e é ele que reconstrói o modelo. É a mesma lei do `command_pick`:
    /// o editor-core nunca aprende o que um item quer dizer.
    pub(super) command_palette_toggled: bool,
    /// ⭐ **Quanto a lista da paleta está rolada, em px** (F3 / ADR-0166).
    ///
    /// ⚠️ **Zerado ao ABRIR e ao FECHAR** — uma paleta nova nunca herda a rolagem da anterior, pela
    /// mesma razão que ela não herda a busca. E o **teto** não vive aqui: quem sabe até onde ir é
    /// quem MEDE o conteúdo (`command_palette::max_scroll`), porque a altura depende de medir texto.
    pub(super) command_palette_scroll: f32,
    /// Section-header id → highlighter color index (0..4 for the 5
    /// canonical colors; missing entry == "no outline"). Painted by
    /// the inspector as a colored stroke around the section block.
    pub(super) section_outline_color: BTreeMap<NodeId, u8>,
    /// Per-panel list of user-created notes. Each note carries a
    /// background color index into the highlighter palette. New
    /// notes append; right-click → delete removes by index. The
    /// painter walks this list once per panel each frame.
    pub(super) notes_per_panel: BTreeMap<NodeId, Vec<NoteData>>,
    /// Sticky source of the most recently completed context-menu
    /// request, captured at apply-event time so the inspector can
    /// route the click → side-table mutation. The dispatch clears
    /// `context_menu` on the same Down event that selects an item;
    /// without this snapshot the inspector loses the request
    /// before it can read it.
    pub(super) last_context_menu: Option<ContextMenuRequest>,
    /// Currently-active color picker target. `Some(id)` means the
    /// floating BlenderColorPicker is open and editing the color
    /// stored at `widget_colors[id]`. `None` hides the picker. Set
    /// by clicks on color targets (section color circles, color
    /// swatches, …) and cleared by any click outside the picker
    /// and outside another color target.
    pub(super) picker_target: Option<NodeId>,
    /// Per-widget current color. Keyed by the target widget's id
    /// (section color circles, color swatches). The picker writes
    /// here on every frame while editing; painters read here to
    /// display the widget's current color.
    pub(super) widget_colors: BTreeMap<NodeId, [u8; 4]>,
    /// In-progress scrollbar drag. Captured on Down inside a
    /// scrollbar thumb's hit rect; consumed by Move events to
    /// translate cursor delta into a `panel_scroll` delta; cleared
    /// on Up. `track_h` and `content_h` are snapshotted so the
    /// drag stays linear even if the painter republishes them
    /// mid-drag.
    pub(super) scrollbar_drag: Option<ScrollbarDragAnchor>,
    /// The currently-open dropdown popover that owns scroll: `(dropdown id, popover rect)`. Republished
    /// each frame the popover paints; lets `dispatch_wheel` + the `DROPDOWN_SCROLLBAR_ID` drag route to
    /// it (the scroll value + content/visible heights live in the `panel_scroll`/`panel_*_h` tables
    /// keyed by the dropdown id, so the generic scrollbar drag works unchanged). Stale after close —
    /// consumers gate on the dropdown still being `open`.
    pub(super) dropdown_popover: Option<(NodeId, Rect)>,
    /// Editor-wide corner-radius scale. `1.0` = canonical, `0.0` =
    /// sharp / squared, `1.6` = round. Painters that want to follow
    /// the user's preset multiply their `Radius::*.px()` by this.
    /// Centralized so the topbar theme menu drives the look in one
    /// place.
    pub(super) radius_scale: f32,
    /// Rail button size preset (Small / Medium / Large). User-toggled
    /// via the Themes menu (2026-05-24); painters in
    /// [`crate::widget::tool_rail`] and the hero orchestrator
    /// ([`crate::screens::hero`]) read this to pick the chip edge
    /// and rail column width.
    pub(super) rail_button_size: crate::widget::RailButtonSize,
    /// Largura AUTORADA de cada coluna docada (`None` = a de fábrica). Ver
    /// [`WidgetStore::dock_width`].
    pub(super) dock_w_left: Option<f32>,
    pub(super) dock_w_right: Option<f32>,
    /// Cached present-mode (VSync ON vs OFF). Source of truth still
    /// lives in the shell (it owns the swap chain), but the core
    /// mirrors the last value the user picked in Settings → Display
    /// so menu paint can show a "selected" bullet next to the active
    /// row. Default `true` matches the shell's `Fifo` baseline.
    pub(super) present_vsync: bool,
    /// Hierarchy row display order. When non-empty, the hierarchy
    /// painter walks this list instead of the fixture's default
    /// order. Mutated by drag-and-drop (`Down + Move > threshold +
    /// Up`) to reorder rows.
    pub(super) hierarchy_order: Vec<NodeId>,
    /// Parent map for tree-style hierarchy. `child → parent`; absent
    /// entries are roots. Mutated by drop-inside DnD; consumed by the
    /// painter to indent rows by depth.
    pub(super) hierarchy_parent: BTreeMap<NodeId, NodeId>,
    /// M14.6C: parents whose subtree is collapsed in the panel.
    /// View-only state (does NOT touch ECS hierarchy); just hides
    /// descendants in the row list. Click on the chevron toggles
    /// membership.
    pub(super) hierarchy_collapsed: std::collections::BTreeSet<NodeId>,
    /// In-progress hierarchy drag. `Some` when a Primary Down landed
    /// on a hierarchy row and the cursor has moved past the drag
    /// threshold; cleared on Up (with reorder applied) or on Up at
    /// the original position (treated as a regular click).
    pub(super) hierarchy_drag: Option<HierarchyDragState>,
    /// ⭐⭐ **O arrasto que SAI do painel** (plano `docs/Components/07`, etapa B) — ver
    /// [`super::asset_drag_ops`]. `None` = nenhum em curso.
    pub(super) asset_drag: Option<crate::interaction::drag_payload::InFlightDrag>,
    /// Onde o `Down` do arrasto de asset aconteceu — o ponto contra o qual o limiar é medido.
    pub(super) asset_drag_origin: (f32, f32),
    /// Os cartões que o navegador de assets **pinta** agora (`id → índice`), republicados por
    /// quadro. Ver [`super::asset_drag_ops`].
    pub(super) asset_cells: std::collections::BTreeMap<NodeId, usize>,
    /// M14.6B: every NodeId currently displayed as a hierarchy row.
    /// Painter republishes the set each frame (fixture + live
    /// modes). Dispatch reads this to decide "this Down is on a
    /// draggable hierarchy row" without hardcoding any id range —
    /// the static `is_hierarchy_entity_id(400..=411)` check covers
    /// only the fixture range; live (ECS-bridge) rows start at
    /// `100_000+` and would silently fall through to "click,
    /// no drag" without this set.
    pub(super) hierarchy_row_ids: std::collections::BTreeSet<NodeId>,
    /// TextInput ids that should treat Enter as "insert newline"
    /// instead of the default "Submit + Blur" (single-line form
    /// behavior). Populated by widgets that wrap multi-line content
    /// (TextArea, note bodies). Default-empty so a freshly registered
    /// TextInput is single-line — matches user expectation that Enter
    /// confirms the value rather than wrapping.
    pub(super) multiline_text_ids: std::collections::BTreeSet<NodeId>,
    /// TextInputs that **abort on Esc** rather than merely blurring — they emit
    /// `WidgetEvent::Cancel` so the panel can drop whatever mode the field was serving
    /// (a rename, a search).
    ///
    /// It was a hardcoded `id == HIER_RENAME_INPUT || id == TIMELINE_MARKER_RENAME_INPUT`
    /// inside `dispatch_key` — a second list of ids that every new cancel-on-escape field
    /// had to be added to, in a file it has no other business in. A widget's behaviour
    /// belongs to the widget.
    pub(super) cancel_on_escape_ids: std::collections::BTreeSet<NodeId>,
    /// M14.A polish: in-progress drag on a NumberInput body. Captured
    /// on Down inside the box (NOT inside the up/down arrow), held
    /// across Move events to convert cursor delta → value delta
    /// (Blender-style: horizontal fast, vertical slow, Shift = fine).
    /// On Up: a drag that NEVER crossed the threshold becomes a
    /// regular "click → enter edit mode"; one that did becomes a
    /// committed value (no edit mode).
    pub(super) number_input_drag: Option<NumberInputDragState>,
    /// M14.A polish: in-progress continuous-hold on a NumberInput
    /// stepper arrow. The dispatcher fires one tick on Down, then
    /// `dispatch_tick` repeats while held (initial delay + repeat
    /// interval matching macOS Aqua text-field steppers).
    pub(super) number_stepper_hold: Option<NumberStepperHoldState>,
    /// Latest Shift modifier state, pushed by the shell on every
    /// `ModifiersChanged`. Used by `dispatch_pointer` to scale the
    /// NumberInput drag delta (Shift = 0.001× multiplier = fine
    /// adjustment). Pointer events don't carry modifiers natively in
    /// `ph2d-host::PointerEvent`; this is the canonical cache.
    pub(super) shift_held: bool,
    /// Fase 0c: latest Cmd (macOS) / Ctrl (Linux/Windows) modifier
    /// state, mirror of [`Self::shift_held`]. Shell pushes via
    /// [`Self::set_cmd_held`] on every `ModifiersChanged`, OR'ing
    /// `super_key()` and `control_key()` so panel handlers can treat
    /// the two as interchangeable (toggle-select modifier).
    pub(super) cmd_held: bool,
    /// In-progress Painter layers-panel row drag (W3 T3.8 — reorder +
    /// drop-into-group). Reuses the generic [`HierarchyDragState`] anchor
    /// (dragged id + down/cursor pos + active threshold). Unlike the
    /// hierarchy drag, the dispatch never mutates structure here — the
    /// painter tool owns the `LayerStack` and resolves the drop.
    pub(super) painter_layer_drag: Option<HierarchyDragState>,
    /// W4 §3 — pending Curves/Levels control-point drag, set by the dispatch
    /// when a [`InteractiveState::CurvePoint`] is dragged and drained by the
    /// panel each frame: `(parent, channel, index, x01, y01)`. Like the layer
    /// drags, the dispatch never mutates curve state here — the painter tool
    /// owns the curve and applies it via `set_curve_point`.
    pub(super) curve_point_drag: Option<(NodeId, u8, u8, f32, f32)>,
    /// Every `NodeId` currently displayed as a Painter layer row. The
    /// layers panel republishes the set each frame; dispatch reads it to
    /// decide "this Down is on a draggable layer row" (mirror of
    /// [`Self::hierarchy_row_ids`]).
    pub(super) painter_layer_row_ids: std::collections::BTreeSet<NodeId>,
    /// Every `NodeId` that is a "picker swatch" — a [`crate::widget::ColorSwatch`]
    /// whose Down opens the canonical Blender color picker seeded with the
    /// swatch's current `widget_color`. Panels register their picker swatches
    /// here as they paint (idempotent; the ids are stable). Generalizes the
    /// former per-id `PAINTER_COLOR_THUMB` special-case so any panel swatch
    /// (Painter brush color, Vector fill, …) opens the picker uniformly.
    pub(super) picker_swatch_ids: std::collections::BTreeSet<NodeId>,
    /// Motion Nodes M0.T2 — graph pointer gestures stashed by dispatch, drained
    /// by the motion-graph panel each frame. Dispatch never interprets them
    /// (mirror of `curve_point_drag`, but a queue: one frame can hold a Begin
    /// followed by several Updates).
    pub(super) graph_gestures: Vec<GraphGesture>,
    /// Per-surface accumulated anchored-zoom (wheel over the graph canvas),
    /// drained by the panel. Mirror of `panel_scroll`, plus the zoom anchor.
    pub(super) graph_zoom: BTreeMap<NodeId, GraphZoom>,
    /// Graph keyboard commands (Delete/F/A/Esc/K/P/Ctrl+D) produced while a
    /// graph surface holds focus, drained by the panel.
    pub(super) graph_keys: Vec<GraphKey>,
    /// Per-surface canvas rect, republished by the panel each frame so
    /// `dispatch_wheel` can tell when the cursor is over a graph (mirror of
    /// `panel_rects`, scoped to graph surfaces).
    pub(super) graph_canvas: BTreeMap<NodeId, Rect>,
    /// The graph surface that currently holds keyboard focus (set by the panel
    /// when the motion tool is active + the graph is hovered/focused), so
    /// `dispatch_key` routes graph shortcuts. `None` = no graph focus.
    pub(super) graph_focused: Option<NodeId>,
    /// Whether the active graph capture has moved since Down — decides End vs
    /// Click on Up (mirror of the NumberInput drag's threshold flag).
    pub(super) graph_moved: bool,
    /// Whether the active graph capture began as the SECOND Down of a double-click
    /// (mirror of `timeline_double`). The Up reads it back to choose `DoubleClick`
    /// over `Click` — without it a graph surface can never see a double-click at all,
    /// because the graph's Down returns early, past the general detection path.
    pub(super) graph_double: bool,
    /// Timeline dope-sheet pointer gestures stashed by dispatch, drained by the
    /// timeline panel each frame (mirror of `graph_gestures`).
    pub(super) timeline_gestures: Vec<TimelineGesture>,
    /// Whether the active timeline capture has moved past the drag slop since
    /// Down — decides End vs Click on Up (mirror of `graph_moved`).
    pub(super) timeline_moved: bool,
    /// Pointer position at the timeline capture's Down, the origin the drag slop
    /// is measured from.
    pub(super) timeline_press: (f32, f32),
    /// Whether the active timeline capture began as the second Down of a
    /// double-click — read on Up so a marker tap can open its rename field.
    pub(super) timeline_double: bool,
    /// Per-surface accumulated wheel (anchored zoom + pan + row scroll), drained
    /// by the timeline panel. Mirror of `graph_zoom`.
    pub(super) timeline_wheel: BTreeMap<NodeId, TimelineWheel>,
    /// Per-surface time-axis rect, republished by the panel each frame so
    /// `dispatch_wheel` can tell when the cursor is over the dope-sheet (mirror
    /// of `graph_canvas`). Cleared while the panel is hidden.
    pub(super) timeline_canvas: BTreeMap<NodeId, Rect>,
    /// O canal da **tira do Flip** (gestos + a folga do arrasto), agrupado num campo só —
    /// os métodos e o porquê vivem em [`super::flip_strip`].
    pub(super) flip_strip: super::flip_strip::FlipStripChannel,
    /// Latest Alt modifier state, mirror of [`Self::shift_held`]/[`Self::cmd_held`].
    /// Pushed by the shell on `ModifiersChanged`; folded into `GestureMods.alt`.
    pub(super) alt_held: bool,
}

#[cfg(test)]
mod tests;
