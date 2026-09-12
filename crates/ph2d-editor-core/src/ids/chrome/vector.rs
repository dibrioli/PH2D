//! Vector module chrome NodeIds (VGRAPH_* geometry-graph + VECTOR_INSPECTOR_*).
use super::{NodeId, hash_node_id};
use ph2d_tool_registry::hash_node_id_runtime;

/// Vector Geometry-Graph panel (W3 T3.1) — docked panel that places the
/// `vector.source` node + drives its 8 params (sliders) and renders the cooked
/// `VectorNetwork` live. Outer-rect id for `z_order`; the 8 param sliders below.
pub const VGRAPH_PANEL: NodeId = hash_node_id("vgraph.panel");
/// Vector Inspector panel (W2 T2.4) — minimal right-docked panel hosting the
/// fill swatch (+ future vertex/node params). Outer-rect id for `z_order`.
pub const VECTOR_INSPECTOR_PANEL: NodeId = hash_node_id("vector_inspector.panel");

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

pub const VECTOR_GRAD_ANGLE: NodeId = hash_node_id("vector.grad.angle");

/// Multi-point gradient: add a point (bbox center) / remove the selected point.
pub const VECTOR_GRAD_ADD_POINT: NodeId = hash_node_id("vector.grad.add_point");
pub const VECTOR_GRAD_REMOVE_POINT: NodeId = hash_node_id("vector.grad.remove_point");
/// Influence (strength / reach) of the selected multi-point gradient point.
pub const VECTOR_GRAD_INFLUENCE: NodeId = hash_node_id("vector.grad.influence");

/// Jitter (per-texel grain, 0..1) of the selected multi-point gradient point.
pub const VECTOR_GRAD_JITTER: NodeId = hash_node_id("vector.grad.jitter");

/// Linear/Radial gradient: add an interior ramp stop / remove the selected one.
pub const VECTOR_GRAD_ADD_STOP: NodeId = hash_node_id("vector.grad.add_stop");
pub const VECTOR_GRAD_REMOVE_STOP: NodeId = hash_node_id("vector.grad.remove_stop");

/// Arm the transform gizmo's "Set Center" mode (redefine the rotation/scale pivot).
pub const VECTOR_PIVOT_EDIT: NodeId = hash_node_id("vector.pivot.edit");

/// "Convert to Curves" — assa a forma VIVA selecionada (texto/paramétrica) em paths
/// crus: texto explode num grupo de paths por-letra; formas descartam só o `VecShape`.
pub const VECTOR_CONVERT_TO_CURVES: NodeId = hash_node_id("vector.convert_to_curves");
// ── Formas: seletor + campos GENÉRICOS (catálogo data-driven) ───────────────
// Com 25+ formas, um id por forma e um id por parâmetro seria insustentável (e o
// painel, um pântano de `match`). Os ids são GERADOS por índice: o painel itera o
// catálogo (`ph2d_tool_vector::shapes`) e pinta o botão `i` e o campo `j`; a tool
// resolve o índice de volta para a forma / o parâmetro. Uma forma nova não precisa
// de id nenhum.

// ── Expand: Outline Stroke + Offset Path ─────────────────────────────────────
// Os dois são COMANDOS destrutivos sobre a seleção, irmãos das booleanas acima e
// pelo mesmo motor (`ph2d_vec_boolean::expand`). O `join` é do OFFSET, não do
// traço: um path pode não ter traço nenhum e ainda assim ser offsetado.
pub const VECTOR_EXPAND_OFFSET: NodeId = hash_node_id("vector.expand.offset");

// **Power Stroke** — a largura VARIA ao longo do caminho. Três valores de controle + onde o
// do meio senta: é o *perfil* que o Illustrator vende pronto, e cobre o caso de 90%
// (afina-no-fim, afina-nos-dois, engrossa-no-meio). As alças arrastáveis na linha são um
// gesto de canvas, outra wave — esta os antecede sem os bloquear.
pub const VECTOR_EXPAND_W_START: NodeId = hash_node_id("vector.expand.w_start");

pub const VECTOR_EXPAND_W_MID: NodeId = hash_node_id("vector.expand.w_mid");

pub const VECTOR_EXPAND_W_END: NodeId = hash_node_id("vector.expand.w_end");

pub const VECTOR_EXPAND_W_POS: NodeId = hash_node_id("vector.expand.w_pos");

// NOTA: o antigo `VECTOR_VERT_CHAMFER` (o toggle de estilo de quina na seção Vertex) foi
// REMOVIDO — o estilo virou o par de ferramentas Fillet / Chamfer no rail (`VECTOR_MODE_*`
// abaixo), a consolidação que o Enio pediu ("muito espalhado").
/// "Delete Node" button — removes the selected vertex (re-stitching neighbors);
/// a document edit routed through the shell drain (mirror of the vertex-type
/// buttons). Insert is a canvas gesture (click a segment) — no button.
pub const VECTOR_VERT_DELETE: NodeId = hash_node_id("vector.vert.delete");
/// **Select Subpath** — todos os nós dos CONTORNOS que a seleção toca. Num compound (forma com
/// furos) é o que separa *este buraco* de *a forma inteira*, distinção que o `Ctrl+A` não faz.
pub const VECTOR_VERT_SEL_SUBPATH: NodeId = hash_node_id("vector.vert.sel.subpath");
/// **Select Same** — todos os nós do MESMO tipo do primário (o *Select Same* do Inkscape). É o que
/// transforma *"afiar as 12 quinas desta estrela"* de doze cliques em dois.
pub const VECTOR_VERT_SEL_SAME: NodeId = hash_node_id("vector.vert.sel.same");

/// **X do nó**, em MUNDO e na unidade do artista — o último buraco da W6 PRECISÃO: dava para
/// arrastar e encaixar um nó, não para **dizer** onde ele vai.
///
/// ⚠️ **O par é a MEDIANA da seleção, e o que ele aplica é um DESLOCAMENTO** (o modelo do
/// Blender). Com um nó selecionado a mediana É o nó e "delta até o alvo" É "posição absoluta",
/// então o caso simples lê como o Illustrator; com vários, o conjunto anda junto em vez de
/// colapsar num X só — que é o que o campo do Inkscape faz, e é um *alinhar* disfarçado de
/// coordenada.
pub const VECTOR_VERT_X: NodeId = hash_node_id("vector.vert.x");
/// **Y do nó** — o irmão do [`VECTOR_VERT_X`], mesma lei.
pub const VECTOR_VERT_Y: NodeId = hash_node_id("vector.vert.y");

// ── Arrange (ADR-0108 — path ops: duplicate + z-order) ───────────────────────
// Act on the SELECTED path (shell-side PenTool selection); document commands
// routed through the shell drain (mirror of Boolean/Vertex). Duplicate clones
// the path with a small offset; the four z-order buttons restack it (render
// order = paths-vec order, index 0 = back).
pub const VECTOR_ARRANGE_DUPLICATE: NodeId = hash_node_id("vector.arrange.duplicate");
/// **O Z-INDEX GLOBAL da forma** (Enio, 2026-08-04) — o número que SOBREPÕE a ordem da
/// hierarquia: só quando dois objetos empatam nele é que a árvore decide quem fica na frente.
/// Maior = mais à frente (a convenção do `CanvasItem.z_index` do Godot).
pub const VECTOR_ARRANGE_Z: NodeId = hash_node_id("vector.arrange.z");

/// Rotation (degrees) — a RELATIVE scrub field (not a bbox readout): each change
/// rotates the selected path by the delta about its bbox center. Seeded to 0 while
/// unfocused; the panel owns the per-gesture accumulator.
pub const VECTOR_TRANSFORM_R: NodeId = hash_node_id("vector.transform.r");

/// **Resize Box** — o checkbox que decide o que a alça do gizmo faz a ESTE objeto (plano UI/UX
/// W3b, decisão do Enio 2026-08-03).
///
/// Marcado, arrastar a alça reescreve a **CAIXA** (a geometria). Desmarcado, escala a **POSE**,
/// que é herdada por todo descendente — o comportamento correto para um objeto de **game**, e o
/// que este editor sempre fez.
///
/// ⚠️ Molduras e os filhos delas nascem marcados; o resto nasce desmarcado. O default é DERIVADO
/// da hierarquia (`ph2d_ecs::resize_box_default`) e o componente só grava a discordância — então
/// re-marcar no valor de fábrica DESTACA, e o ficheiro fica limpo.
pub const VECTOR_TRANSFORM_RESIZE_BOX: NodeId = hash_node_id("vector.transform.resize_box");

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

/// **Blend** — cria (ou re-cria) os passos entre as duas formas selecionadas.
pub const VECTOR_BLEND_RUN: NodeId = hash_node_id("vector.blend.run");
/// **Steps** — quantas formas nascem no meio do caminho.
pub const VECTOR_BLEND_STEPS: NodeId = hash_node_id("vector.blend.steps");

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
    hash_node_id_runtime(&format!("vector.envelope.preset.{index}"))
}
/// **Bend** — a força do preset, `[-1, 1]`. Só é oferecido com um preset ativo: sem ele o slider
/// não teria o que re-carimbar, e seria um knob morto.
pub const VECTOR_ENVELOPE_BEND: NodeId = hash_node_id("vector.envelope.bend");

/// **Pins** — o gesto do *puppet warp* (MLS-rigid, ADR-0129 Fatia E). Não há gaiola: o artista prega
/// pontos no modo Node e arrasta. ⚠️ Com **2 pinos não se deforma nada** (uma isometria de um par
/// determina uma rigidez única); é preciso um **3º pino não-colinear**.
pub const VECTOR_ENVELOPE_PINS: NodeId = hash_node_id("vector.envelope.pins");
/// **Clear Pins** — apaga todos os pinos. É a única porta de remoção da Fatia E: apagar UM exige um
/// gesto que compete com "clicar no vazio prega", e sem porta nenhuma um pino mal pregado seria
/// permanente.
pub const VECTOR_ENVELOPE_CLEAR_PINS: NodeId = hash_node_id("vector.envelope.clear_pins");

/// ⭐ **Stroke (a caixa de marcar)** — *esta forma TEM traço?* (plano 34).
///
/// ⚠️ **Um checkbox e não um `segmented`**, pela lei que este painel já escreveu: *um `segmented` é
/// uma escolha entre MODOS nomeados; um checkbox é uma PROPRIEDADE que o objeto tem ou não tem*.
///
/// ⛔ **O rótulo NÃO é "Outline"**: já existe [`VECTOR_EXPAND_OUTLINE_STROKE`], que é outra coisa
/// (converter o traço numa forma preenchida). Duas palavras para um conceito e uma palavra para
/// dois é como um painel passa a mentir.
pub const VECTOR_STROKE_PRESENT: NodeId = hash_node_id("vector.stroke.present");
