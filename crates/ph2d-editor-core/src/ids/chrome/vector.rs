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
    VECTOR_SECTION_ALIGN,
    VECTOR_SECTION_ARRANGE,
    VECTOR_SECTION_PATH,
    VECTOR_SECTION_TEXT,
    VECTOR_SECTION_FONT,
    VECTOR_SECTION_PARAGRAPH,
    VECTOR_SECTION_AXES,
];
