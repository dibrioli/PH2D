//! Vector module chrome NodeIds (VGRAPH_* geometry-graph + VECTOR_INSPECTOR_*).
use super::painter::fnv_node_id_runtime;
use super::{NodeId, hash_node_id};

/// Vector Geometry-Graph panel (W3 T3.1) — docked panel that places the
/// `vector.source` node + drives its 8 params (sliders) and renders the cooked
/// `VectorNetwork` live. Outer-rect id for `z_order`; the 8 param sliders below.
pub const VGRAPH_PANEL: NodeId = hash_node_id("vgraph.panel");
pub const VGRAPH_KIND: NodeId = hash_node_id("vgraph.kind");
pub const VGRAPH_WIDTH: NodeId = hash_node_id("vgraph.width");
pub const VGRAPH_HEIGHT: NodeId = hash_node_id("vgraph.height");
pub const VGRAPH_SIDES: NodeId = hash_node_id("vgraph.sides");
pub const VGRAPH_INNER_RATIO: NodeId = hash_node_id("vgraph.inner_ratio");
pub const VGRAPH_TURNS: NodeId = hash_node_id("vgraph.turns");
pub const VGRAPH_SAMPLES_PER_TURN: NodeId = hash_node_id("vgraph.samples_per_turn");
pub const VGRAPH_ROTATION: NodeId = hash_node_id("vgraph.rotation");
/// Vector Inspector panel (W2 T2.4) — minimal right-docked panel hosting the
/// fill swatch (+ future vertex/node params). Outer-rect id for `z_order`.
pub const VECTOR_INSPECTOR_PANEL: NodeId = hash_node_id("vector_inspector.panel");
/// Vector Inspector close (X) button.
pub const VECTOR_INSPECTOR_CLOSE: NodeId = hash_node_id("vector_inspector.close");
/// Vector Inspector fill-color swatch — a picker swatch (opens the Blender
/// picker on Down via `is_picker_swatch`); the shell read-back applies the
/// picked color to the selected regions.
pub const VECTOR_INSPECTOR_FILL_SWATCH: NodeId = hash_node_id("vector_inspector.fill_swatch");
/// Vector Inspector Shape-kind picker (W2 §4.2) — the on-screen replacement for
/// the interim hotkeys 1-5. A vertical 5-option segmented control shown only
/// while the `vector_shape` tool is active. `_KIND` is the group/label id; the
/// five `_SHAPE_*` ids are the per-option Button hit-targets (the generic
/// RadioGroup dispatch is not wired yet, so each option is a Button — its
/// `Click` routes through the shell bridge to `VectorShapeTool::set_kind`).
pub const VECTOR_INSPECTOR_SHAPE_KIND: NodeId = hash_node_id("vector_inspector.shape_kind");
pub const VECTOR_INSPECTOR_SHAPE_RECT: NodeId = hash_node_id("vector_inspector.shape.rect");
pub const VECTOR_INSPECTOR_SHAPE_ELLIPSE: NodeId = hash_node_id("vector_inspector.shape.ellipse");
pub const VECTOR_INSPECTOR_SHAPE_POLYGON: NodeId = hash_node_id("vector_inspector.shape.polygon");
pub const VECTOR_INSPECTOR_SHAPE_STAR: NodeId = hash_node_id("vector_inspector.shape.star");
pub const VECTOR_INSPECTOR_SHAPE_SPIRAL: NodeId = hash_node_id("vector_inspector.shape.spiral");

// ── Vector tool Style panel (ADR-0108 cutover — docked `ph2d-panel-vector`) ──
// The `vector` tool's Style controls live in a right-docked `Panel<State>` (the
// tool `FloatingPanel` is unpainted). Width slider (1..20 px) + Stroke / Fill
// colour swatches (each opens the shared OKLCH picker via `is_picker_swatch`) +
// a Fill "None" affordance. Distinct slug family from the retired
// `vector_inspector.*` ids above.
/// Vector Style panel outer rect id (for `z_order` + hit-barrier).
pub const VECTOR_PANEL: NodeId = hash_node_id("vector.panel");
/// Vector Style panel close (X) button.
pub const VECTOR_CLOSE: NodeId = hash_node_id("vector.close");
/// Stroke-width slider (bipolar-less: track `0..1` → `1..20` px).
pub const VECTOR_WIDTH: NodeId = hash_node_id("vector.width");
/// Px-valued chip linked to [`VECTOR_WIDTH`].
pub const VECTOR_WIDTH_NUM: NodeId = hash_node_id("vector.width_num");
/// Stroke-colour swatch — a picker swatch (opens the Blender picker on Down);
/// the shell read-back applies the picked colour (RGB; alpha kept) to the path.
pub const VECTOR_STROKE_SWATCH: NodeId = hash_node_id("vector.stroke_swatch");
/// Fill-colour swatch — a picker swatch (RGB; the Opacity slider owns alpha).
pub const VECTOR_FILL_SWATCH: NodeId = hash_node_id("vector.fill_swatch");
/// Stroke / Fill **Opacity** sliders (0..100 %) — the single source of the
/// stroke/fill alpha. `0 %` = invisible (no fill). Each drives the matching
/// `VectorTool` colour's alpha channel.
// ── Fill type + gradient (ADR-0108 gradient group) ───────────────────────────
// Segmented selector switching the SELECTED path's fill between Solid / Linear /
// Radial (a document command via the shell drain). The Angle slider drives a
// Linear gradient's direction (shown only in Linear).
pub const VECTOR_FILL_KIND_SOLID: NodeId = hash_node_id("vector.fill_kind.solid");
pub const VECTOR_FILL_KIND_LINEAR: NodeId = hash_node_id("vector.fill_kind.linear");
pub const VECTOR_FILL_KIND_RADIAL: NodeId = hash_node_id("vector.fill_kind.radial");
/// Multi-point (Cavalry freeform IDW) fill.
pub const VECTOR_FILL_KIND_MULTI: NodeId = hash_node_id("vector.fill_kind.multi");
pub const VECTOR_GRAD_ANGLE: NodeId = hash_node_id("vector.grad.angle");
pub const VECTOR_GRAD_ANGLE_NUM: NodeId = hash_node_id("vector.grad.angle_num");
/// Multi-point gradient: add a point (bbox center) / remove the selected point.
pub const VECTOR_GRAD_ADD_POINT: NodeId = hash_node_id("vector.grad.add_point");
pub const VECTOR_GRAD_REMOVE_POINT: NodeId = hash_node_id("vector.grad.remove_point");
/// Influence (strength / reach) of the selected multi-point gradient point.
pub const VECTOR_GRAD_INFLUENCE: NodeId = hash_node_id("vector.grad.influence");
pub const VECTOR_GRAD_INFLUENCE_NUM: NodeId = hash_node_id("vector.grad.influence_num");
/// Jitter (per-texel grain, 0..1) of the selected multi-point gradient point.
pub const VECTOR_GRAD_JITTER: NodeId = hash_node_id("vector.grad.jitter");
pub const VECTOR_GRAD_JITTER_NUM: NodeId = hash_node_id("vector.grad.jitter_num");
/// Linear/Radial gradient: add an interior ramp stop / remove the selected one.
pub const VECTOR_GRAD_ADD_STOP: NodeId = hash_node_id("vector.grad.add_stop");
pub const VECTOR_GRAD_REMOVE_STOP: NodeId = hash_node_id("vector.grad.remove_stop");

// ── Align + Distribute (multi-path object selection) ─────────────────────────
// Shown when ≥2 paths are selected (Align) / ≥3 (Distribute). Align snaps each
// selected path's bbox edge/center to the selection's bbox; Distribute evenly
// spaces the middle paths' centers between the two extremes.
pub const VECTOR_ALIGN_LEFT: NodeId = hash_node_id("vector.align.left");
pub const VECTOR_ALIGN_HCENTER: NodeId = hash_node_id("vector.align.hcenter");
pub const VECTOR_ALIGN_RIGHT: NodeId = hash_node_id("vector.align.right");
pub const VECTOR_ALIGN_TOP: NodeId = hash_node_id("vector.align.top");
pub const VECTOR_ALIGN_VCENTER: NodeId = hash_node_id("vector.align.vcenter");
pub const VECTOR_ALIGN_BOTTOM: NodeId = hash_node_id("vector.align.bottom");
pub const VECTOR_DISTRIBUTE_H: NodeId = hash_node_id("vector.distribute.h");
pub const VECTOR_DISTRIBUTE_V: NodeId = hash_node_id("vector.distribute.v");
/// Arm the transform gizmo's "Set Center" mode (redefine the rotation/scale pivot).
pub const VECTOR_PIVOT_EDIT: NodeId = hash_node_id("vector.pivot.edit");

pub const VECTOR_STROKE_OPACITY: NodeId = hash_node_id("vector.stroke_opacity");
pub const VECTOR_STROKE_OPACITY_NUM: NodeId = hash_node_id("vector.stroke_opacity_num");
pub const VECTOR_FILL_OPACITY: NodeId = hash_node_id("vector.fill_opacity");
pub const VECTOR_FILL_OPACITY_NUM: NodeId = hash_node_id("vector.fill_opacity_num");

// ── Stroke details (ADR-0108 Fase 1 — cap / join / dash + gap) ───────────────
// Line cap (Butt/Round/Square) + join (Miter/Round/Bevel) segmented rows + a
// Dash length slider (0 = solid) + a Gap length slider (space between dashes).
// Both are multiples of the stroke width. Drive the matching `VectorTool`
// fields; the bridge applies them to new + selected paths (like colour/width).
pub const VECTOR_CAP_BUTT: NodeId = hash_node_id("vector.cap.butt");
pub const VECTOR_CAP_ROUND: NodeId = hash_node_id("vector.cap.round");
pub const VECTOR_CAP_SQUARE: NodeId = hash_node_id("vector.cap.square");
pub const VECTOR_JOIN_MITER: NodeId = hash_node_id("vector.join.miter");
pub const VECTOR_JOIN_ROUND: NodeId = hash_node_id("vector.join.round");
pub const VECTOR_JOIN_BEVEL: NodeId = hash_node_id("vector.join.bevel");
pub const VECTOR_DASH: NodeId = hash_node_id("vector.dash");
pub const VECTOR_DASH_NUM: NodeId = hash_node_id("vector.dash_num");
pub const VECTOR_GAP: NodeId = hash_node_id("vector.gap");
pub const VECTOR_GAP_NUM: NodeId = hash_node_id("vector.gap_num");

// ── Pontas de traço / markers (arrowheads) ───────────────────────────────────
// Bloco APPEND-ONLY. A ponta é propriedade do **stroke** (herda cor e largura do
// traço, gira com a tangente da curva), então os dois seletores moram na seção
// STROKE, ao lado de cap/join/dash — não são formas do catálogo.
//
// São **chips de dropdown**, não botões que ciclam: oito opções (`ALL_MARKERS`) é
// o mesmo tamanho de lista da CATEGORIA do catálogo (sete famílias), que já virou
// chip pela mesma razão — ciclar às cegas por oito nomes para achar o que se quer
// é pior que ler os oito e apontar. O popover pinta no passe DIFERIDO do painel
// (dentro do corpo, o `push_clip` do scroll o cortaria).
/// Chip da ponta no COMEÇO do caminho.
pub const VECTOR_MARKER_START_DD: NodeId = hash_node_id("vector.marker.start_dd");
/// Chip da ponta no FIM do caminho.
pub const VECTOR_MARKER_END_DD: NodeId = hash_node_id("vector.marker.end_dd");

/// **Head Size** — o tamanho da ponta, como MÚLTIPLO do que a largura do traço já dita
/// (`1.0` = o default). Múltiplo, e não tamanho absoluto: a ponta tem de crescer com o
/// traço, senão engrossar a linha faz a seta encolher visualmente até virar um cotoco.
pub const VECTOR_MARKER_SCALE: NodeId = hash_node_id("vector.marker.scale");
/// **Head Round** — o arredondamento das quinas da PRÓPRIA ponta (`0` = afiada).
pub const VECTOR_MARKER_ROUND: NodeId = hash_node_id("vector.marker.round");
/// **Both Ends** — a "dupla via". O estado é **DERIVADO** (`start != None && end != None`),
/// nunca guardado: duas fontes de verdade sobre "é bidirecional?" divergiriam no dia em que
/// alguém trocasse uma das pontas pelo chip.
pub const VECTOR_MARKER_BOTH: NodeId = hash_node_id("vector.marker.both");

/// Quantos seletores de ponta existem: começo (`slot = 0`) e fim (`slot = 1`).
pub const MARKER_SLOTS: usize = 2;

/// Teto de pontas que o painel registra por seletor — espelha
/// `ph2d_vec_scene::ALL_MARKERS` com folga (o editor-core não depende da crate de
/// geometria; um marcador novo entra lá e já nasce clicável, sem tocar aqui).
pub const MAX_MARKER_OPTIONS: usize = 16;

/// [`NodeId`] da opção `index` no popover do seletor `slot` (0 = começo, 1 = fim).
/// Runtime `format!` (a lista de pontas é dado), gêmeo FNV no mesmo espaço de ids —
/// espelho das fábricas do catálogo de formas.
#[must_use]
pub fn vector_marker_option_id(slot: usize, index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.marker.opt.{slot}.{index}"))
}

// ── Seletor de MODO (ADR-0108/0112) ──────────────────────────────────────────
// Quatro modos, e só: Select (gizmo) · Node (âncoras) · Pen (cria) · Text. As
// FORMAS não são modos — são um catálogo (`vector_shape_id`), e escolher uma põe a
// tool no modo Shape. Sem isso, cada forma nova exigiria um id e um variante de modo.
/// Seta preta — seleciona/transforma a forma pelo gizmo (ADR-0112).
pub const VECTOR_MODE_SELECT: NodeId = hash_node_id("vector.mode.select");
/// Seta branca — edita âncoras/handles, sem gizmo (ADR-0112).
pub const VECTOR_MODE_NODE: NodeId = hash_node_id("vector.mode.node");
/// Caneta — cria paths.
pub const VECTOR_MODE_PEN: NodeId = hash_node_id("vector.mode.pen");
/// Text mode: clica no canvas e digita (glyphs viram VecPaths). Botão do mode row.
pub const VECTOR_MODE_TEXT: NodeId = hash_node_id("vector.mode.text");
/// Text "Size" slider (world units) — shown only in Text mode; drives the glyph
/// size of the active session + the size a new session starts at.
pub const VECTOR_TEXT_SIZE: NodeId = hash_node_id("vector.text.size");
/// Value chip paired with [`VECTOR_TEXT_SIZE`].
pub const VECTOR_TEXT_SIZE_NUM: NodeId = hash_node_id("vector.text.size_num");
/// Text "Weight" slider (`wght` axis 100..900) — shown only in Text mode; drives the
/// variable-font weight of the active session + the weight a new session starts at.
pub const VECTOR_TEXT_WEIGHT: NodeId = hash_node_id("vector.text.weight");
/// Value chip paired with [`VECTOR_TEXT_WEIGHT`].
pub const VECTOR_TEXT_WEIGHT_NUM: NodeId = hash_node_id("vector.text.weight_num");
/// Text font-family picker prev / next buttons (`<` / `>`) — shown only in Text mode;
/// cycle the chosen system font family (or the bundled default) of the text.
pub const VECTOR_TEXT_FONT_PREV: NodeId = hash_node_id("vector.text.font_prev");
pub const VECTOR_TEXT_FONT_NEXT: NodeId = hash_node_id("vector.text.font_next");
/// Text "Import Font…" button — opens a native file picker for a `.ttf`/`.otf`,
/// loads it as the current text font (and adds it to the cycle).
pub const VECTOR_TEXT_FONT_IMPORT: NodeId = hash_node_id("vector.text.font_import");
/// Text font **dropdown** chip (between the `<` / `>` arrows) — a `Dropdown` whose
/// open popover lists every pickable family rendered **in its own outline** (real
/// style preview). Option clicks route by [`vector_text_font_option_id`].
pub const VECTOR_TEXT_FONT_DD: NodeId = hash_node_id("vector.text.font_dd");
/// Paragraph section (Text mode): horizontal alignment L / C / R (segmented, sets
/// `VecTextEdit::align`), line height (leading, × size) + its chip, and tracking
/// (letter-spacing, em fraction) + its chip.
pub const VECTOR_TEXT_ALIGN_LEFT: NodeId = hash_node_id("vector.text.align_left");
pub const VECTOR_TEXT_ALIGN_CENTER: NodeId = hash_node_id("vector.text.align_center");
pub const VECTOR_TEXT_ALIGN_RIGHT: NodeId = hash_node_id("vector.text.align_right");
pub const VECTOR_TEXT_LINE_HEIGHT: NodeId = hash_node_id("vector.text.line_height");
pub const VECTOR_TEXT_LINE_HEIGHT_NUM: NodeId = hash_node_id("vector.text.line_height_num");
pub const VECTOR_TEXT_TRACKING: NodeId = hash_node_id("vector.text.tracking");
pub const VECTOR_TEXT_TRACKING_NUM: NodeId = hash_node_id("vector.text.tracking_num");
/// "Convert to Curves" — assa a forma VIVA selecionada (texto/paramétrica) em paths
/// crus: texto explode num grupo de paths por-letra; formas descartam só o `VecShape`.
pub const VECTOR_CONVERT_TO_CURVES: NodeId = hash_node_id("vector.convert_to_curves");

/// Stable [`NodeId`] for the `index`-th family row in the open font dropdown
/// (index into the shell's pickable list `[bundled] ++ imported ++ system`). Runtime
/// `format!` (the family count is only known at runtime); the FNV twin keeps it in
/// the same id space as the `hash_node_id` consts. Mirrors the Painter option-id
/// factories (`painter_brush_*_option_id`).
#[must_use]
pub fn vector_text_font_option_id(index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.text.fontopt.{index}"))
}

/// Max variation-axis number fields the Text panel shows (besides the dedicated
/// Weight slider) — one per non-`wght` axis the current font exposes. 6 covers every
/// registered axis a real variable font ships (`wdth`/`slnt`/`opsz`/`ital`/`GRAD`/…).
pub const MAX_TEXT_VARIATION_AXES: usize = 6;

/// NodeId for the `index`-th variation-axis field in the Text panel — bound to the
/// `index`-th non-`wght` axis of the current font (its name/range/value published by
/// the shell). Runtime `format!` (the axis set is per-font). Mirrors the font-option
/// factory.
#[must_use]
pub fn vector_text_axis_id(index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.text.axis.{index}"))
}
// ── Formas: seletor + campos GENÉRICOS (catálogo data-driven) ───────────────
// Com 25+ formas, um id por forma e um id por parâmetro seria insustentável (e o
// painel, um pântano de `match`). Os ids são GERADOS por índice: o painel itera o
// catálogo (`ph2d_tool_vector::shapes`) e pinta o botão `i` e o campo `j`; a tool
// resolve o índice de volta para a forma / o parâmetro. Uma forma nova não precisa
// de id nenhum.

/// Botão da forma `index` no catálogo (o seletor de formas do painel).
#[must_use]
pub fn vector_shape_id(index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.shape.{index}"))
}

/// Aba da família `index` (Basic / Round / Arrows / Flow / Bubbles / Symbols…).
#[must_use]
pub fn vector_shape_group_id(index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.shape.group.{index}"))
}

/// Campo numérico do parâmetro `index` da forma ativa (o rótulo e a faixa vêm do
/// catálogo). Teto = `ph2d_vec_scene::MAX_SHAPE_FIELDS`.
#[must_use]
pub fn vector_shape_field_id(index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.shape.field.{index}"))
}

/// **Botão** do parâmetro `index` quando ele é uma ESCOLHA (`FieldUnit::Choice`) em vez de
/// um número — o ponto de vista de um sólido isométrico, por exemplo.
///
/// Id próprio, e não o `vector_shape_field_id`, porque os dois são widgets DIFERENTES: um
/// slot é registrado uma vez, no `populate`, que é estático e não sabe qual forma está em
/// foco. O botão emite `SetValue` **no id do campo**, então o valor continua morando num
/// lugar só — o slot numérico é apenas o depósito, e deixa de ser clicável.
#[must_use]
pub fn vector_shape_choice_id(index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.shape.choice.{index}"))
}

/// Teto de campos de forma que o painel registra (espelha `MAX_SHAPE_FIELDS`).
pub const MAX_SHAPE_FIELD_SLOTS: usize = 8;
/// Teto de botões de forma que o painel registra (o catálogo pode crescer até aqui).
pub const MAX_SHAPE_SLOTS: usize = 64;
/// Teto de abas de família.
pub const MAX_SHAPE_GROUP_SLOTS: usize = 12;

// ── Boolean ops (ADR-0108 Fase 1 — edit-time union/subtract/intersect) ───────
// Act on the DOCUMENT (shell-owned `vec_scene`), NOT the tool's Style: the
// panel forwards a `Click` over `ToolPanelEvent` and the shell drain applies
// the op to the two last closed regions (mirror of the U/I/D hotkeys).
pub const VECTOR_BOOL_UNION: NodeId = hash_node_id("vector.bool.union");
pub const VECTOR_BOOL_SUBTRACT: NodeId = hash_node_id("vector.bool.subtract");
pub const VECTOR_BOOL_INTERSECT: NodeId = hash_node_id("vector.bool.intersect");
pub const VECTOR_BOOL_EXCLUDE: NodeId = hash_node_id("vector.bool.exclude");

// ── Vertex type (ADR-0108 Fase 1 — rich handle editing) ──────────────────────
// Retype the SELECTED vertex (Corner cusp / Smooth colinear / Symmetric mirror).
// A document edit (mutates the path via the shell-side PenTool), shown only when
// a vertex is selected; each `Click` routes through the shell drain.
pub const VECTOR_VERT_CORNER: NodeId = hash_node_id("vector.vert.corner");
pub const VECTOR_VERT_SMOOTH: NodeId = hash_node_id("vector.vert.smooth");
pub const VECTOR_VERT_SYMMETRIC: NodeId = hash_node_id("vector.vert.symmetric");
/// "Delete Node" button — removes the selected vertex (re-stitching neighbors);
/// a document edit routed through the shell drain (mirror of the vertex-type
/// buttons). Insert is a canvas gesture (click a segment) — no button.
pub const VECTOR_VERT_DELETE: NodeId = hash_node_id("vector.vert.delete");

// ── Arrange (ADR-0108 — path ops: duplicate + z-order) ───────────────────────
// Act on the SELECTED path (shell-side PenTool selection); document commands
// routed through the shell drain (mirror of Boolean/Vertex). Duplicate clones
// the path with a small offset; the four z-order buttons restack it (render
// order = paths-vec order, index 0 = back).
pub const VECTOR_ARRANGE_DUPLICATE: NodeId = hash_node_id("vector.arrange.duplicate");
pub const VECTOR_ARRANGE_TO_BACK: NodeId = hash_node_id("vector.arrange.to_back");
pub const VECTOR_ARRANGE_BACKWARD: NodeId = hash_node_id("vector.arrange.backward");
pub const VECTOR_ARRANGE_FORWARD: NodeId = hash_node_id("vector.arrange.forward");
pub const VECTOR_ARRANGE_TO_FRONT: NodeId = hash_node_id("vector.arrange.to_front");
// Mirror the selected path (H = left↔right, V = up↔down) around its bbox center.
pub const VECTOR_ARRANGE_FLIP_H: NodeId = hash_node_id("vector.arrange.flip_h");
pub const VECTOR_ARRANGE_FLIP_V: NodeId = hash_node_id("vector.arrange.flip_v");
// Rotate the selected path 90° (CW / CCW) around its bbox center.
pub const VECTOR_ARRANGE_ROTATE_CW: NodeId = hash_node_id("vector.arrange.rotate_cw");
pub const VECTOR_ARRANGE_ROTATE_CCW: NodeId = hash_node_id("vector.arrange.rotate_ccw");

// ── Transform (ADR-0108 — precise numeric position + size) ───────────────────
// Standalone NumberInputs (NOT slider-linked) showing the selected path's anchor
// bbox: X/Y = top-left (world), W/H = size. Seeded each frame from the published
// bbox (unless focused); editing routes a document command through the shell
// drain (X/Y → translate, W/H → scale about the bbox min).
pub const VECTOR_TRANSFORM_X: NodeId = hash_node_id("vector.transform.x");
pub const VECTOR_TRANSFORM_Y: NodeId = hash_node_id("vector.transform.y");
pub const VECTOR_TRANSFORM_W: NodeId = hash_node_id("vector.transform.w");
pub const VECTOR_TRANSFORM_H: NodeId = hash_node_id("vector.transform.h");
/// Rotation (degrees) — a RELATIVE scrub field (not a bbox readout): each change
/// rotates the selected path by the delta about its bbox center. Seeded to 0 while
/// unfocused; the panel owns the per-gesture accumulator.
pub const VECTOR_TRANSFORM_R: NodeId = hash_node_id("vector.transform.r");

// ── Path shape (ADR-0108 — whole-path handle ops) ────────────────────────────
// One-shot buttons acting on ALL vertices of the SELECTED path (document commands
// via the shell drain, mirror of Arrange). Smooth = auto-colinear handles from
// neighbors (Inkscape 1/3, curve-ifies a polygon/hand path); Sharpen = collapse
// handles onto the anchor (straight-segment corners).
pub const VECTOR_PATH_SMOOTH: NodeId = hash_node_id("vector.path.smooth");
pub const VECTOR_PATH_SHARPEN: NodeId = hash_node_id("vector.path.sharpen");
/// Simplify = drop redundant/near-colinear anchors (RDP-style vertex reduction).
pub const VECTOR_PATH_SIMPLIFY: NodeId = hash_node_id("vector.path.simplify");
/// Subdivide = insert a midpoint on every segment (exact de Casteljau split).
pub const VECTOR_PATH_SUBDIVIDE: NodeId = hash_node_id("vector.path.subdivide");
/// Close/Open toggle — flips the selected path between a closed loop and an open
/// ribbon (label driven by the published `closed` flag).
pub const VECTOR_PATH_CLOSE: NodeId = hash_node_id("vector.path.close");

// ── Compound paths (ADR-0108 — subpaths + fill rule) ─────────────────────────
/// Merge the selected closed paths into ONE compound path (a contour inside
/// another becomes a hole, via `EvenOdd`). Inverse: [`VECTOR_COMPOUND_RELEASE`].
pub const VECTOR_COMPOUND_MAKE: NodeId = hash_node_id("vector.compound.make");
/// Split the selected compound path's subpaths back into standalone paths.
pub const VECTOR_COMPOUND_RELEASE: NodeId = hash_node_id("vector.compound.release");
/// Fill rule of the selected COMPOUND path — the two agree on a single contour,
/// so the row only shows when the path actually has subpaths.
pub const VECTOR_FILL_RULE_NONZERO: NodeId = hash_node_id("vector.fill.rule.nonzero");
pub const VECTOR_FILL_RULE_EVENODD: NodeId = hash_node_id("vector.fill.rule.evenodd");

// ── Snap (ADR-0108 — smart guides) ───────────────────────────────────────────
/// Snap to the other shapes' anchors / bbox key points. Held Alt bypasses it.
/// The GRID toggle is NOT here: the editor's universal Grid Snap panel owns it
/// (`grid_snap::ids`), and the Vector module just asks `GridSnapState::snap_world`.
pub const VECTOR_SNAP_OFF: NodeId = hash_node_id("vector.snap.off");
pub const VECTOR_SNAP_ON: NodeId = hash_node_id("vector.snap.on");

// ── Reforma da UI do painel (categoria ≠ tipo) ───────────────────────────────
// Bloco APPEND-ONLY (isolamento de linha): o 5º pill de modo, o chip de família do
// catálogo e os cabeçalhos COLAPSÁVEIS de cada seção. O painel hand-rolava 14
// rótulos de seção e ZERO `SectionHeader`, violando o canon
// (`docs/UI_Padrao/components/section_header.md`: toda seção usa
// `paint_section_header` e toda seção é colapsável).

/// **Forma** — o 5º pill do seletor de modo. Escolher uma forma no catálogo já põe a
/// tool em `DrawMode::Shape`, mas SEM este botão a fileira de modos ficava toda apagada
/// enquanto se desenhava (nenhum pill correspondia ao modo vivo) — o usuário não via em
/// que modo estava.
pub const VECTOR_MODE_SHAPE: NodeId = hash_node_id("vector.mode.shape");

/// **Conector** — o 6º pill. Arrasta de uma forma a outra e cria uma LINHA que gruda nas
/// duas e as segue (o conector do draw.io / Figma). Não é uma forma do catálogo: a
/// geometria dele é uma função pura da RELAÇÃO (quem, e como), re-cozida a cada frame —
/// por isso é um modo, e não mais um item na grade de tipos.
pub const VECTOR_MODE_CONNECT: NodeId = hash_node_id("vector.mode.connect");

/// **Shape Builder** — o 7º pill. Arrasta sobre as regiões de 2+ formas selecionadas: o que
/// o cursor pinta vira uma forma só; com Alt, some.
pub const VECTOR_MODE_BUILD: NodeId = hash_node_id("vector.mode.build");

/// Chip de **CATEGORIA** do catálogo de formas (`Dropdown`): a família (Basic / Round /
/// Arrows / Flow / Bubbles / Symbols / 3D) escolhida numa LINHA, com um widget
/// visualmente distinto da grade de tipos abaixo — é essa diferença de widget que separa
/// "categoria" de "tipo" (antes os dois eram `paint_segmented_button` do mesmo tamanho,
/// indistinguíveis). As OPÇÕES do popover reusam os ids já existentes
/// [`vector_shape_group_id`].
pub const VECTOR_SHAPE_GROUP_DD: NodeId = hash_node_id("vector.shape.group_dd");

// Cabeçalhos de seção. Não têm `InteractiveState`: o collapse vem do dispatch genérico
// (`WidgetStore::mark_collapsible_section` no populate + o hit-rect do header), e o
// painter lê `store.is_collapsed(id)`.
pub const VECTOR_SECTION_TOOL: NodeId = hash_node_id("vector.section.tool");
pub const VECTOR_SECTION_SHAPE: NodeId = hash_node_id("vector.section.shape");
/// A seção dos PARÂMETROS da forma em foco — o rótulo dela é o NOME da forma (`STAR`,
/// `GEAR`…), que é como o usuário descobre a quem os campos pertencem.
pub const VECTOR_SECTION_SHAPE_PARAMS: NodeId = hash_node_id("vector.section.shape_params");
pub const VECTOR_SECTION_STROKE: NodeId = hash_node_id("vector.section.stroke");
pub const VECTOR_SECTION_FILL: NodeId = hash_node_id("vector.section.fill");
pub const VECTOR_SECTION_FILL_TYPE: NodeId = hash_node_id("vector.section.fill_type");
pub const VECTOR_SECTION_SNAP: NodeId = hash_node_id("vector.section.snap");
pub const VECTOR_SECTION_TRANSFORM: NodeId = hash_node_id("vector.section.transform");
pub const VECTOR_SECTION_VERTEX: NodeId = hash_node_id("vector.section.vertex");
pub const VECTOR_SECTION_BOOLEAN: NodeId = hash_node_id("vector.section.boolean");
pub const VECTOR_SECTION_ALIGN: NodeId = hash_node_id("vector.section.align");
pub const VECTOR_SECTION_ARRANGE: NodeId = hash_node_id("vector.section.arrange");
pub const VECTOR_SECTION_PATH: NodeId = hash_node_id("vector.section.path");
pub const VECTOR_SECTION_TEXT: NodeId = hash_node_id("vector.section.text");
pub const VECTOR_SECTION_FONT: NodeId = hash_node_id("vector.section.font");
pub const VECTOR_SECTION_PARAGRAPH: NodeId = hash_node_id("vector.section.paragraph");
pub const VECTOR_SECTION_AXES: NodeId = hash_node_id("vector.section.axes");

// ── Conector: os campos da RELAÇÃO (bloco APPEND-ONLY) ───────────────────────
// A seção só existe quando há um conector na seleção — os três campos editam
// TODOS os selecionados de uma vez (é o que permite calibrar um diagrama inteiro
// sem visitar linha por linha).
/// Seção **CONNECTOR** — some quando nenhum conector está selecionado.
pub const VECTOR_SECTION_CONNECTOR: NodeId = hash_node_id("vector.section.connector");
/// **Route** — reta ou ortogonal. Um botão que CICLA (o gêmeo do campo de escolha do
/// catálogo de formas): "Route: 1" não é UI; "Route: Orthogonal" é.
pub const VECTOR_CONNECTOR_ROUTE: NodeId = hash_node_id("vector.connector.route");
/// **Jetty** — o quanto a linha avança reta antes de poder dobrar. Caixa numérica: o
/// valor pintado é o EFETIVO (o automático, quando o campo é `None`), então o campo
/// nasce onde o olho vê a linha e não salta no primeiro toque.
pub const VECTOR_CONNECTOR_JETTY: NodeId = hash_node_id("vector.connector.jetty");
/// **Spread** — o afastamento que separa dois conectores no mesmo par de formas.
pub const VECTOR_CONNECTOR_SPREAD: NodeId = hash_node_id("vector.connector.spread");
/// **Corner** — o raio das quinas do PERCURSO (as dobras do cotovelo), em unidades de
/// mundo. `0` = afiado, o default do fluxograma clássico. Não confundir com o
/// [`VECTOR_MARKER_ROUND`], que arredonda as quinas da SETA.
pub const VECTOR_CONNECTOR_CORNER: NodeId = hash_node_id("vector.connector.corner");
/// **Curve** — o braço dos handles da rota curva ("mais perto ou mais longe do ponto").
pub const VECTOR_CONNECTOR_CURVE: NodeId = hash_node_id("vector.connector.curve");

// ── Blend: a interpolação de formas (bloco APPEND-ONLY) ──────────────────────
// A correspondência entre duas formas é o problema que NINGUÉM resolveu (nem o
// GSAP, nem o Corel, nem o Lottie — `docs/Vector Module/20_*` §1.3). Por isso os
// dois botões de ESCAPE não são enfeite: são a saída do artista no dia em que o
// automático errar, e toda ferramenta séria do mercado teve de ter uma.
/// Seção **BLEND** — os passos intermediários entre as DUAS formas selecionadas.
pub const VECTOR_SECTION_BLEND: NodeId = hash_node_id("vector.section.blend");
/// A seção **Morph** — o `t` animável (irmã da Blend, mas objeto próprio: uma forma, não N).
pub const VECTOR_SECTION_MORPH: NodeId = hash_node_id("vector.section.morph");
/// **Blend** — cria (ou re-cria) os passos entre as duas formas selecionadas.
pub const VECTOR_BLEND_RUN: NodeId = hash_node_id("vector.blend.run");
/// **Steps** — quantas formas nascem no meio do caminho.
pub const VECTOR_BLEND_STEPS: NodeId = hash_node_id("vector.blend.steps");
pub const VECTOR_BLEND_STEPS_NUM: NodeId = hash_node_id("vector.blend.steps_num");
/// **Stack Up** — cada passo nasce ACIMA do anterior (ou abaixo). A ordem de z de uma
/// sequência é parte do resultado: um blend cuja 1ª intermediária fica DEBAIXO da forma
/// que a originou não lê como transição.
pub const VECTOR_BLEND_STACK_UP: NodeId = hash_node_id("vector.blend.stack_up");
/// **Reset Spine** — volta o spine do blend selecionado ao AUTOMÁTICO (a reta pelos centros das
/// fontes), desfazendo a edição do modo Node (ADR-0128 C2b). Sem ele, a única saída da edição do
/// spine é o undo global.
pub const VECTOR_BLEND_RESET_SPINE: NodeId = hash_node_id("vector.blend.reset_spine");
/// **Expand** — materializa os passos VIRTUAIS do blend em formas REAIS e descarta o objeto vivo
/// (ADR-0128 Fase D): o "vira múltiplas formas com um botão" do Enio. As fontes persistem; o que
/// morre é a relação. Espelho do "Convert to Curves" da Live Shape.
pub const VECTOR_BLEND_EXPAND: NodeId = hash_node_id("vector.blend.expand");
/// **Release** — desfaz o blend: os passos somem e as fontes ficam (ADR-0128 Fase D). Nada é
/// materializado. É a saída do blend pelo canvas: a linha não é selecionável no modo Select, então o
/// Delete não a alcança. NÃO confundir com `VECTOR_COMPOUND_RELEASE` (que solta um compound path).
pub const VECTOR_BLEND_RELEASE: NodeId = hash_node_id("vector.blend.release");
/// **Morph** — cria o objeto morph vivo (a forma única entre DUAS formas, com o `t` animável).
pub const VECTOR_MORPH_RUN: NodeId = hash_node_id("vector.morph.run");
/// O `t` do morph selecionado: onde no caminho entre as duas fontes a forma está.
pub const VECTOR_MORPH_T: NodeId = hash_node_id("vector.morph.t");
pub const VECTOR_MORPH_T_NUM: NodeId = hash_node_id("vector.morph.t_num");

/// **Pick Shapes** — o 8º pill de modo (ADR-0128 C2b): coleta as formas fechadas clicadas **na
/// ordem**, e o botão Blend as liga nessa sequência. É o análogo do Build (que captura faces) e do
/// Connect (que pega a forma sob o cursor) — a ORDEM da cadeia é escolhida a dedo, não a de z.
pub const VECTOR_MODE_PICKBLEND: NodeId = hash_node_id("vector.mode.pickblend");

// ── Envelope: a deformação por gaiola (bloco APPEND-ONLY, ADR-0129) ──────────
// O envelope é um CONTAINER (Fatia 3): a gaiola vive num componente da entidade-container e as
// formas envolvidas são FILHAS dela. Até esta seção existir, a única porta para criar um envelope
// era a env `PH2D_BUILD_SMOKE` — a feature não existia no produto.
/// Seção **ENVELOPE** — a deformação das formas selecionadas por uma gaiola de 4 cantos.
pub const VECTOR_SECTION_ENVELOPE: NodeId = hash_node_id("vector.section.envelope");
/// **Envelope** — envolve as formas selecionadas (1..N) num container com gaiola em repouso. Os
/// cantos são arrastáveis no modo Node; no Select o gizmo move o envelope inteiro.
pub const VECTOR_ENVELOPE_RUN: NodeId = hash_node_id("vector.envelope.run");
/// **Expand** — materializa a deformação: a geometria DEFORMADA vira o desenho definitivo e a
/// gaiola morre. Espelho do `VECTOR_BLEND_EXPAND`.
pub const VECTOR_ENVELOPE_EXPAND: NodeId = hash_node_id("vector.envelope.expand");
/// **Release** — desfaz o envelope: a fonte AUTORADA volta (a deformação é descartada) e a gaiola
/// morre. Sem ele, envolver seria porta de mão única. NÃO confundir com `VECTOR_BLEND_RELEASE`.
pub const VECTOR_ENVELOPE_RELEASE: NodeId = hash_node_id("vector.envelope.release");
/// **Perspective** — o gesto da homografia: a gaiola tem 4 cantos e os lados são RETOS, então toda
/// reta interior continua reta. É o gesto com que o envelope nasce.
pub const VECTOR_ENVELOPE_PERSPECTIVE: NodeId = hash_node_id("vector.envelope.perspective");
/// **Mesh** — o gesto do patch de Coons: os LADOS dobram (2 controles por lado, alças no modo Node).
/// NÃO é o mesmo mapa que o Perspective — com lados retos ele é bilinear, não projetivo, e os dois
/// só coincidem com a gaiola em repouso (ADR-0129 §4).
pub const VECTOR_ENVELOPE_MESH: NodeId = hash_node_id("vector.envelope.mesh");
/// Teto de presets de gaiola que o painel oferece (`ph2d_ecs::EnvelopeWarp::ALL`). O `populate`
/// registra os `MAX` botões de uma vez e o `paint` desenha só os que o shell PUBLICOU — assim
/// acrescentar um preset é uma linha na tabela do componente, e nenhum sítio de UI.
pub const MAX_ENVELOPE_PRESETS: usize = 16;
/// [`NodeId`] do botão do preset `index`. Runtime `format!` (a lista é dado), gêmeo FNV no mesmo
/// espaço de ids — espelho das fábricas do catálogo de formas e das pontas de traço.
#[must_use]
pub fn vector_envelope_preset_id(index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.envelope.preset.{index}"))
}
/// **Bend** — a força do preset, `[-1, 1]`. Só é oferecido com um preset ativo: sem ele o slider
/// não teria o que re-carimbar, e seria um knob morto.
pub const VECTOR_ENVELOPE_BEND: NodeId = hash_node_id("vector.envelope.bend");
/// O campo numérico gêmeo do [`VECTOR_ENVELOPE_BEND`].
pub const VECTOR_ENVELOPE_BEND_NUM: NodeId = hash_node_id("vector.envelope.bend.num");
/// **Pins** — o gesto do *puppet warp* (MLS-rigid, ADR-0129 Fatia E). Não há gaiola: o artista prega
/// pontos no modo Node e arrasta. ⚠️ Com **2 pinos não se deforma nada** (uma isometria de um par
/// determina uma rigidez única); é preciso um **3º pino não-colinear**.
pub const VECTOR_ENVELOPE_PINS: NodeId = hash_node_id("vector.envelope.pins");
/// **Clear Pins** — apaga todos os pinos. É a única porta de remoção da Fatia E: apagar UM exige um
/// gesto que compete com "clicar no vazio prega", e sem porta nenhuma um pino mal pregado seria
/// permanente.
pub const VECTOR_ENVELOPE_CLEAR_PINS: NodeId = hash_node_id("vector.envelope.clear_pins");

// ── Effects: a PILHA de Live Path Effects (bloco APPEND-ONLY, ADR-0132) ─────
// A pilha é dado de documento (`VecPath.effects`) e o `cooked()` a avalia logo depois do estágio
// da quina. Até esta seção existir, a única porta para pôr um efeito num caminho era a env
// `PH2D_BUILD_SMOKE=13` — o motor existia e o artista não o alcançava.
/// Seção **EFFECTS** — os efeitos não-destrutivos e empilháveis do caminho selecionado.
pub const VECTOR_SECTION_EFFECTS: NodeId = hash_node_id("vector.section.effects");
/// O teto de TIPOS de efeito que o menu "Add" oferece. O painel registra este número de
/// botões, sempre, e pinta só os que a tabela publicada de facto traz.
pub const MAX_FX_KINDS: usize = 8;
/// O teto de efeitos numa pilha.
pub const MAX_FX_ROWS: usize = 4;
/// O teto de parâmetros por efeito.
///
/// ⚠️ Estes três espelham constantes do motor (`ph2d_vec_scene::effect`), que o painel não
/// alcança — ele vive de snapshots. Há gate a exigir que os dois lados concordem.
pub const MAX_FX_ROW_PARAMS: usize = 4;

/// **Add \<tipo\>** — o botão que põe um efeito do tipo `kind` na pilha.
#[must_use]
pub fn vector_fx_add_id(kind: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.fx.add.{kind}"))
}

/// **Remove** o efeito da linha `row`.
#[must_use]
pub fn vector_fx_remove_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.fx.remove.{row}"))
}

/// Sobe o efeito da linha `row` na pilha. A ORDEM muda a geometria (ADR-0132), então
/// reordenar é feature, não enfeite.
#[must_use]
pub fn vector_fx_up_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.fx.up.{row}"))
}

/// Desce o efeito da linha `row`.
#[must_use]
pub fn vector_fx_down_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.fx.down.{row}"))
}

/// **O card** da linha `row` — a moldura. Id próprio, e não o do ✕: o card e o botão de
/// apagar são coisas diferentes para a a11y, e partilhar o id faria o leitor de tela anunciar
/// a moldura inteira como "remover".
#[must_use]
pub fn vector_fx_card_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.fx.card.{row}"))
}

/// **O olho** da linha `row` — desarma o efeito sem o apagar. Desarmar não pode custar os
/// parâmetros: zerar a amplitude para "desligar" e depois querer o valor de volta é obrigar o
/// artista a lembrar-se de números.
#[must_use]
pub fn vector_fx_hide_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.fx.hide.{row}"))
}

/// O slider do parâmetro `param` do efeito da linha `row`.
#[must_use]
pub fn vector_fx_param_id(row: usize, param: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.fx.p.{row}.{param}"))
}

/// O campo numérico do [`vector_fx_param_id`].
#[must_use]
pub fn vector_fx_param_num_id(row: usize, param: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.fx.p.{row}.{param}.num"))
}

/// Todos os cabeçalhos de seção do painel Vector — o `populate` os marca como
/// colapsáveis por esta lista (uma seção nova entra aqui e ganha o collapse de graça;
/// esquecer a marca faz o header virar um título MORTO, que não dobra).
pub const VECTOR_SECTIONS: &[NodeId] = &[
    VECTOR_SECTION_TOOL,
    VECTOR_SECTION_SHAPE,
    VECTOR_SECTION_SHAPE_PARAMS,
    VECTOR_SECTION_STROKE,
    VECTOR_SECTION_FILL,
    VECTOR_SECTION_FILL_TYPE,
    VECTOR_SECTION_SNAP,
    VECTOR_SECTION_TRANSFORM,
    VECTOR_SECTION_VERTEX,
    VECTOR_SECTION_BOOLEAN,
    VECTOR_SECTION_BLEND,
    VECTOR_SECTION_MORPH,
    VECTOR_SECTION_ENVELOPE,
    VECTOR_SECTION_EFFECTS,
    VECTOR_SECTION_ALIGN,
    VECTOR_SECTION_ARRANGE,
    VECTOR_SECTION_PATH,
    VECTOR_SECTION_TEXT,
    VECTOR_SECTION_FONT,
    VECTOR_SECTION_PARAGRAPH,
    VECTOR_SECTION_AXES,
    VECTOR_SECTION_CONNECTOR,
];
