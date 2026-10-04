//! Vector module chrome NodeIds (VGRAPH_* geometry-graph + VECTOR_INSPECTOR_*).
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

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

/// **Padrão de textura** (plano 33): uma arte repetida num reticulado.
pub const VECTOR_FILL_KIND_PATTERN: NodeId = hash_node_id("vector.fill_kind.pattern");

pub const VECTOR_GRAD_ANGLE_NUM: NodeId = hash_node_id("vector.grad.angle_num");

pub const VECTOR_GRAD_INFLUENCE_NUM: NodeId = hash_node_id("vector.grad.influence_num");

pub const VECTOR_GRAD_JITTER_NUM: NodeId = hash_node_id("vector.grad.jitter_num");

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

pub const VECTOR_STROKE_OPACITY: NodeId = hash_node_id("vector.stroke_opacity");

pub const VECTOR_STROKE_OPACITY_NUM: NodeId = hash_node_id("vector.stroke_opacity_num");

pub const VECTOR_FILL_OPACITY: NodeId = hash_node_id("vector.fill_opacity");

pub const VECTOR_FILL_OPACITY_NUM: NodeId = hash_node_id("vector.fill_opacity_num");

// ── Stroke details (ADR-0108 Fase 1 — cap / join / dash + gap) ───────────────
// Line cap (Butt/Round/Square) + join (Miter/Round/Bevel) segmented rows + a
// Dash length slider (0 = solid) + a Gap length slider (space between dashes).
// Both are multiples of the stroke width. Drive the matching `VectorTool`
// fields; the bridge applies them to new + selected paths (like colour/width).
/// **Alinhamento do traço** — Centre / Inner / Outer (`ph2d_vec_scene::StrokeAlign`).
///
/// Vive ao lado de Cap/Join porque é a mesma família: *como a caneta se comporta*. O que o
/// distingue é que Inner/Outer só significam alguma coisa sobre uma REGIÃO, então o painel
/// pergunta a `StrokeAlign::needs_a_region` antes de anunciar as duas.
pub const VECTOR_ALIGN_CENTRE: NodeId = hash_node_id("vector.align.centre");

pub const VECTOR_ALIGN_INNER: NodeId = hash_node_id("vector.align.inner");

pub const VECTOR_ALIGN_OUTER: NodeId = hash_node_id("vector.align.outer");

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
    hash_node_id_runtime(&format!("vector.marker.opt.{slot}.{index}"))
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

/// Botão da forma `index` no catálogo (o seletor de formas do painel).
#[must_use]
pub fn vector_shape_id(index: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.shape.{index}"))
}

/// Aba da família `index` (Basic / Round / Arrows / Flow / Bubbles / Symbols…).
#[must_use]
pub fn vector_shape_group_id(index: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.shape.group.{index}"))
}

/// Campo numérico do parâmetro `index` da forma ativa (o rótulo e a faixa vêm do
/// catálogo). Teto = `ph2d_vec_scene::MAX_SHAPE_FIELDS`.
#[must_use]
pub fn vector_shape_field_id(index: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.shape.field.{index}"))
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
    hash_node_id_runtime(&format!("vector.shape.choice.{index}"))
}

/// Teto de campos de forma que o painel registra (espelha `MAX_SHAPE_FIELDS`).
pub const MAX_SHAPE_FIELD_SLOTS: usize = 8;

pub const VECTOR_EXPAND_OFFSET_NUM: NodeId = hash_node_id("vector.expand.offset_num");

pub const VECTOR_EXPAND_JOIN_MITER: NodeId = hash_node_id("vector.expand.join.miter");

pub const VECTOR_EXPAND_JOIN_ROUND: NodeId = hash_node_id("vector.expand.join.round");

pub const VECTOR_EXPAND_JOIN_BEVEL: NodeId = hash_node_id("vector.expand.join.bevel");

// **Qual contorno o Offset Path move** (`OffsetSide`) — só o de fora, só os furos, ou ambos.
// É por-contorno, e é o que faz a quina (Round/Bevel) aparecer no furo, não só no externo.
pub const VECTOR_EXPAND_SIDE_OUTER: NodeId = hash_node_id("vector.expand.side.outer");

pub const VECTOR_EXPAND_SIDE_INNER: NodeId = hash_node_id("vector.expand.side.inner");

pub const VECTOR_EXPAND_SIDE_BOTH: NodeId = hash_node_id("vector.expand.side.both");

pub const VECTOR_EXPAND_OFFSET_PATH: NodeId = hash_node_id("vector.expand.offset_path");

pub const VECTOR_EXPAND_OUTLINE_STROKE: NodeId = hash_node_id("vector.expand.outline_stroke");

pub const VECTOR_EXPAND_W_START_NUM: NodeId = hash_node_id("vector.expand.w_start_num");

pub const VECTOR_EXPAND_W_MID_NUM: NodeId = hash_node_id("vector.expand.w_mid_num");

pub const VECTOR_EXPAND_W_END_NUM: NodeId = hash_node_id("vector.expand.w_end_num");

pub const VECTOR_EXPAND_W_POS_NUM: NodeId = hash_node_id("vector.expand.w_pos_num");

pub const VECTOR_EXPAND_POWER_STROKE: NodeId = hash_node_id("vector.expand.power_stroke");

// ── Vertex type (ADR-0108 Fase 1 — rich handle editing) ──────────────────────
// Retype the SELECTED vertex (Corner cusp / Smooth colinear / Symmetric mirror).
// A document edit (mutates the path via the shell-side PenTool), shown only when
// a vertex is selected; each `Click` routes through the shell drain.
pub const VECTOR_VERT_CORNER: NodeId = hash_node_id("vector.vert.corner");

pub const VECTOR_VERT_SMOOTH: NodeId = hash_node_id("vector.vert.smooth");

pub const VECTOR_VERT_SYMMETRIC: NodeId = hash_node_id("vector.vert.symmetric");

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

pub const VECTOR_SECTION_EXPAND: NodeId = hash_node_id("vector.section.expand");

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

pub const VECTOR_BLEND_STEPS_NUM: NodeId = hash_node_id("vector.blend.steps_num");

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

/// O campo numérico gêmeo do [`VECTOR_ENVELOPE_BEND`].
pub const VECTOR_ENVELOPE_BEND_NUM: NodeId = hash_node_id("vector.envelope.bend.num");

// ── Effects: a PILHA de Live Path Effects (bloco APPEND-ONLY, ADR-0132) ─────
// A pilha é dado de documento (`VecPath.effects`) e o `cooked()` a avalia logo depois do estágio
// da quina. Até esta seção existir, a única porta para pôr um efeito num caminho era a env
// `PH2D_BUILD_SMOKE=13` — o motor existia e o artista não o alcançava.
/// Seção **EFFECTS** — os efeitos não-destrutivos e empilháveis do caminho selecionado.
pub const VECTOR_SECTION_EFFECTS: NodeId = hash_node_id("vector.section.effects");

/// O teto de TIPOS de efeito que o menu "Add" oferece. O painel registra este número de
/// botões, sempre, e pinta só os que a tabela publicada de fato traz.
///
/// ⚠️ Tem de ser `>=` a `PathEffect::KINDS.len()` do motor, senão os últimos tipos ficam
/// inalcançáveis no menu Add (o `.take(MAX_FX_KINDS)` os corta). Gate em
/// `ph2d_vec_scene::effect::tests::the_engine_and_panel_agree_on_the_kind_ceiling`. 8→9 com a
/// família Warp (Arc/Bulge/Wave/Fisheye/Rise); 9→13 com o catálogo ÚNICO (Enio 2026-07-25), que
/// alinhou a lista do Warp à do Envelope: 4 base + 9 estilos; 13→17 com o **Falloff** (Cavalry,
/// Enio 2026-07-25): 4 formas analíticas (Radial/Linear/Rect/Sweep) apendadas ao FIM da tabela;
/// 17→18 com o **Twist** (o remoinho, um KIND só) apendado depois delas; 18→19 com o **Knot** (o
/// entrelace celta, também um KIND só) no fim; 19→21 com **Sketch** (traço à mão) e **Hatch**
/// (hachura), dois KINDs só, apendados no fim.
pub const MAX_FX_KINDS: usize = 21;

/// O teto de efeitos numa pilha.
pub const MAX_FX_ROWS: usize = 4;

/// O teto de parâmetros por efeito.
///
/// ⚠️ Estes três espelham constantes do motor (`ph2d_vec_scene::effect`), que o painel não
/// alcança — ele vive de snapshots. Há gate a exigir que os dois lados concordem.
pub const MAX_FX_ROW_PARAMS: usize = 6;

/// **Add \<tipo\>** — o botão que põe um efeito do tipo `kind` na pilha.
#[must_use]
pub fn vector_fx_add_id(kind: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.fx.add.{kind}"))
}

/// **Remove** o efeito da linha `row`.
#[must_use]
pub fn vector_fx_remove_id(row: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.fx.remove.{row}"))
}

/// Sobe o efeito da linha `row` na pilha. A ORDEM muda a geometria (ADR-0132), então
/// reordenar é feature, não enfeite.
#[must_use]
pub fn vector_fx_up_id(row: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.fx.up.{row}"))
}

/// Desce o efeito da linha `row`.
#[must_use]
pub fn vector_fx_down_id(row: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.fx.down.{row}"))
}

/// **O card** da linha `row` — a moldura. Id próprio, e não o do ✕: o card e o botão de
/// apagar são coisas diferentes para a a11y, e partilhar o id faria o leitor de tela anunciar
/// a moldura inteira como "remover".
#[must_use]
pub fn vector_fx_card_id(row: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.fx.card.{row}"))
}

/// **O olho** da linha `row` — desarma o efeito sem o apagar. Desarmar não pode custar os
/// parâmetros: zerar a amplitude para "desligar" e depois querer o valor de volta é obrigar o
/// artista a lembrar-se de números.
#[must_use]
pub fn vector_fx_hide_id(row: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.fx.hide.{row}"))
}

/// O slider do parâmetro `param` do efeito da linha `row`.
#[must_use]
pub fn vector_fx_param_id(row: usize, param: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.fx.p.{row}.{param}"))
}

/// O campo numérico do [`vector_fx_param_id`].
#[must_use]
pub fn vector_fx_param_num_id(row: usize, param: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.fx.p.{row}.{param}.num"))
}

/// **A CAIXINHA** do parâmetro `param` da linha `row` — id PRÓPRIO, e a razão é dura.
///
/// Um parâmetro de caixinha é pintado como BOTÃO. Reusar o id do slider parecia grátis (o painel
/// pinta um OU outro), mas **um id só pode ter um tipo de widget no store**: o `populate` regista
/// o teto sem saber que efeito cai na linha, então o slot era sempre um `Slider` — e o dispatch
/// diz, com todas as letras, *"sliders emit no Click on release"*.
///
/// O ramo que tratava a caixinha nunca chegou a correr. O que a alternava era o `ValueChanged` do
/// slider, cujo valor é a posição HORIZONTAL do cursor: clicar no mesmo sítio dava sempre o mesmo
/// estado (Enio, 2026-07-18). E recusar essa escrita — sem dar um id próprio à caixinha — matou-a
/// por completo, porque tirou o único escritor que existia.
#[must_use]
pub fn vector_fx_toggle_id(row: usize, param: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.fx.p.{row}.{param}.toggle"))
}

/// **Apply** — assa a pilha inteira no seu resultado cozido (`VecScene::bake_cooked`) e a
/// esvazia: o *Expand Appearance* do Illustrator. É um botão de SEÇÃO (um por painel, não
/// por-linha), por isso `const` e não `fn`. Só é pintado quando há pilha a assar; o mesmo bake
/// alimenta o "Convert to Curves" sobre um caminho com efeitos.
pub const VECTOR_FX_APPLY: NodeId = hash_node_id("vector.fx.apply");

// ── Ferramentas de QUINA: os pills Fillet / Chamfer (bloco APPEND-ONLY) ──────
// Clicar-e-arrastar sobre uma quina para arredondá-la (Fillet) ou chanfrá-la (Chamfer).
// Consolidam num par de ferramentas o que estava espalhado entre a alça de raio do Node e o
// toggle Chamfer da seção Vertex. O gesto e a conversão "vira-quina-primeiro" vivem no shell.
/// **Fillet** — o 9º pill de modo: arredondar quina por clicar-e-arrastar.
pub const VECTOR_MODE_FILLET: NodeId = hash_node_id("vector.mode.fillet");

/// **Chamfer** — o 10º pill de modo: chanfrar quina (reta) por clicar-e-arrastar.
pub const VECTOR_MODE_CHAMFER: NodeId = hash_node_id("vector.mode.chamfer");

/// **Lápis** — o 11º modo (o 10º pill da fileira TOOL): arrastar à mão livre e a curva sair.
/// O gesto que faltava ao módulo, que só sabia nascer de cliques discretos ou de arrasto
/// dimensionado.
pub const VECTOR_MODE_PENCIL: NodeId = hash_node_id("vector.mode.pencil");

/// **Width** — o 12º modo: as alças de largura na curva (plano 25 §5). Irmão dos pills de quina
/// (Fillet/Chamfer) na fileira TOOL: os três editam um atributo VIVO de uma forma que já existe,
/// apontando-a no canvas.
pub const VECTOR_MODE_WIDTH: NodeId = hash_node_id("vector.mode.width");

// ── A TINTA DO TRAÇO (plano 35, wave D) ────────────────────────────────────────
//
// ⚠️ **Um `segmented` e não um checkbox**, pela MESMA lei que escolheu o checkbox acima, aplicada ao
// contrário: *ter traço* é uma propriedade que a forma tem ou não tem; *com que tinta o traço
// desenha* é uma escolha entre MODOS nomeados. As duas fileiras vivem lado a lado e cada uma obedece
// à metade certa da lei.
//
// ⛔ **A lista é `Solid | Pattern` e NÃO a do preenchimento.** O renderer de traço não desenha
// gradiente, e um chip que produzisse um `StrokePaint::Linear` gravaria estado que nada pinta — a
// recusa está escrita no plano 35 §2.1 e o modelo (`StrokePaint`) tem duas variantes por isso.
/// **Solid** — o traço desenha com uma cor.
pub const VECTOR_STROKE_KIND_SOLID: NodeId = hash_node_id("vector.stroke.kind.solid");

/// **Pattern** — o traço desenha com uma arte repetida.
///
/// ⚠️ Clicar aqui numa forma cujo traço ainda não tem padrão **abre a porta da arte** — a 4ª
/// condição da costura (*a sequência tem de levar a algum lugar*), a mesma que o chip *Pattern* do
/// preenchimento já honra.
pub const VECTOR_STROKE_KIND_PATTERN: NodeId = hash_node_id("vector.stroke.kind.pattern");

/// ⭐⭐⭐ **Brush** — a arte PERCORRE o contorno (plano 36, W4), a 3.ª tinta do traço.
///
/// ⚠️ **Clicar aqui ARMA o gesto de duas mãos**, e não abre um diálogo de ficheiro: a arte de um
/// pincel é uma **forma do documento** (o motor copia geometria), não uma imagem. É a mesma porta
/// que o *Use Shape…* do padrão já usa, e a razão está no tipo — `BrushStroke::art` é um
/// `VecPathId`.
pub const VECTOR_STROKE_KIND_BRUSH: NodeId = hash_node_id("vector.stroke.kind.brush");
