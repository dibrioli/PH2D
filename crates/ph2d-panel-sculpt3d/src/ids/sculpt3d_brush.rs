//! **O pincel e o espelho** do painel da cena 3D (`SCULPT3D_*`) — cortados do `sculpt3d.rs` por
//! assunto, pelo tecto de 600 linhas por ficheiro de painel.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/sculpt3d.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── O pincel ────────────────────────────────────────────────────────────────
/// As curvas de `ph2d_sculpt3d::Falloff::ALL`.
///
/// ⚠️ **O tamanho se CONTA, não se escolhe** — o seam
/// `the_panel_offers_every_falloff_the_engine_has` compara este array com o
/// `ALL` do motor, então uma curva nova que não passe por aqui nasce
/// inalcançável no painel e o gate fica vermelho em vez de o botão sumir em
/// silêncio.
pub const SCULPT3D_FALLOFF: [NodeId; 12] = [
    hash_node_id("sculpt3d.falloff.0"),
    hash_node_id("sculpt3d.falloff.1"),
    hash_node_id("sculpt3d.falloff.2"),
    hash_node_id("sculpt3d.falloff.3"),
    hash_node_id("sculpt3d.falloff.4"),
    hash_node_id("sculpt3d.falloff.5"),
    hash_node_id("sculpt3d.falloff.6"),
    hash_node_id("sculpt3d.falloff.7"),
    hash_node_id("sculpt3d.falloff.8"),
    hash_node_id("sculpt3d.falloff.9"),
    hash_node_id("sculpt3d.falloff.10"),
    hash_node_id("sculpt3d.falloff.11"),
];

/// Raio do pincel, em **pixels de tela**.
pub const SCULPT3D_RADIUS: NodeId = hash_node_id("sculpt3d.radius");

/// Chip ligado a [`SCULPT3D_RADIUS`].
pub const SCULPT3D_RADIUS_NUM: NodeId = hash_node_id("sculpt3d.radius_num");

/// Força do dab, em `[0, 1]`.
pub const SCULPT3D_STRENGTH: NodeId = hash_node_id("sculpt3d.strength");

/// Chip ligado a [`SCULPT3D_STRENGTH`].
pub const SCULPT3D_STRENGTH_NUM: NodeId = hash_node_id("sculpt3d.strength_num");

/// Deslocamento do plano, em fração do raio (só os verbos de plano o leem).
pub const SCULPT3D_PLANE_OFFSET: NodeId = hash_node_id("sculpt3d.plane_offset");

/// Chip ligado a [`SCULPT3D_PLANE_OFFSET`].
pub const SCULPT3D_PLANE_OFFSET_NUM: NodeId = hash_node_id("sculpt3d.plane_offset_num");

/// Quanto o Crease aperta lateralmente.
pub const SCULPT3D_PINCH: NodeId = hash_node_id("sculpt3d.pinch");

/// Chip ligado a [`SCULPT3D_PINCH`].
pub const SCULPT3D_PINCH_NUM: NodeId = hash_node_id("sculpt3d.pinch_num");

/// **α do Surface Smooth** — quanto o `b` se ancora na pose do PEN-DOWN em vez
/// da posição de agora (`surface_smooth_shape_preservation`).
pub const SCULPT3D_HC_SHAPE: NodeId = hash_node_id("sculpt3d.hc_shape");

/// Chip ligado a [`SCULPT3D_HC_SHAPE`].
pub const SCULPT3D_HC_SHAPE_NUM: NodeId = hash_node_id("sculpt3d.hc_shape_num");

/// **β do Surface Smooth** — que fração da correção vem do `b` do PRÓPRIO
/// vértice em vez da média dos vizinhos (`surface_smooth_current_vertex`).
///
/// ⚠️ **A faixa deste knob começa em `0,5` e o piso NÃO é dele** — ver
/// `ph2d_sculpt3d::HC_VERTEX_MIN`: abaixo disso o operador AMPLIFICA (medido,
/// `β = 0,3` leva a rugosidade a 43,8× a da base em dezasseis dabs), e quem
/// impede a malha de rebentar é o clamp no MOTOR. O `min` da row existe para o
/// artista não alcançar o disfuncional com o dedo; o clamp existe para um
/// documento que traga o valor errado ser corrigido em vez de explodir.
pub const SCULPT3D_HC_VERTEX: NodeId = hash_node_id("sculpt3d.hc_vertex");

/// Chip ligado a [`SCULPT3D_HC_VERTEX`].
pub const SCULPT3D_HC_VERTEX_NUM: NodeId = hash_node_id("sculpt3d.hc_vertex_num");

/// **A DUREZA DO DAB** — o platô de peso cheio no miolo da pegada.
///
/// ⚠️ Ele NÃO é o [`SCULPT3D_MASK_HARDNESS`], embora os nomes se pareçam: aquele
/// é o expoente da curva PRÓPRIA do canal de máscara, este remapeia a DISTÂNCIA
/// que qualquer falloff consome (`apply_hardness_to_distances` do Blender). Dois
/// controles, duas perguntas, e o gate de costura pinta os dois para o mesmo
/// verbo nunca oferecer um pelo outro.
pub const SCULPT3D_HARDNESS: NodeId = hash_node_id("sculpt3d.hardness");

/// Chip ligado a [`SCULPT3D_HARDNESS`].
pub const SCULPT3D_HARDNESS_NUM: NodeId = hash_node_id("sculpt3d.hardness_num");

/// **O alisamento que corre depois de cada dab** — o `autosmooth_factor` do
/// Blender, e o VIZINHO do [`SCULPT3D_HARDNESS`] no declarador de propriedades
/// dele.
///
/// ⚠️ A adjacência não é acaso e a fileira a honra: são os dois knobs que trocam
/// **borda dura** por **superfície que a malha consegue carregar**, e lê-los
/// juntos é o que faz o segundo ser aprendido quando o primeiro morde.
pub const SCULPT3D_AUTO_SMOOTH: NodeId = hash_node_id("sculpt3d.auto_smooth");

/// Chip ligado a [`SCULPT3D_AUTO_SMOOTH`].
pub const SCULPT3D_AUTO_SMOOTH_NUM: NodeId = hash_node_id("sculpt3d.auto_smooth_num");

/// A dureza da borda do canal de MÁSCARA — o `_hardness` da tool `Masking` do
/// SculptGL. ⚠️ Ele NÃO é um falloff: o canal tem curva própria
/// (`(1 − d)^{2(1 − hardness)}`), e o seletor de [`Falloff`] governa a
/// geometria.
pub const SCULPT3D_MASK_HARDNESS: NodeId = hash_node_id("sculpt3d.mask_hardness");

/// Chip ligado a [`SCULPT3D_MASK_HARDNESS`].
pub const SCULPT3D_MASK_HARDNESS_NUM: NodeId = hash_node_id("sculpt3d.mask_hardness_num");

/// **O PADRÃO que decide onde, dentro da pegada, o verbo age** — a primeira
/// opção é NENHUM e as outras são os padrões de `ph2d_sculpt3d::Alpha::ALL`.
///
/// ⚠️ O tamanho é `Alpha::ALL.len() + 2`, e os DOIS a mais não são padrões: o
/// primeiro é o pincel LISO e o último é o slot de IMAGEM, que carrega o nome do
/// sprite em vez de um nome de fórmula. É a mesma aritmética do
/// [`SCULPT3D_MATCAP`] com um degrau a mais, e pelo mesmo motivo: um chip
/// sobrando pinta uma opção que o motor não tem, um faltando deixa um padrão
/// inalcançável. Gateado.
///
/// ⚠️ **O chip da imagem é o ÚLTIMO, e a posição é load-bearing:** os índices
/// `1..=9` são um deslocamento sobre `Alpha::ALL`, então pôr a imagem no meio
/// re-numeraria os nove e todo clique passaria a armar o padrão vizinho.
pub const SCULPT3D_ALPHA: [NodeId; 11] = [
    hash_node_id("sculpt3d.alpha.none"),
    hash_node_id("sculpt3d.alpha.0"),
    hash_node_id("sculpt3d.alpha.1"),
    hash_node_id("sculpt3d.alpha.2"),
    hash_node_id("sculpt3d.alpha.3"),
    hash_node_id("sculpt3d.alpha.4"),
    hash_node_id("sculpt3d.alpha.5"),
    hash_node_id("sculpt3d.alpha.6"),
    hash_node_id("sculpt3d.alpha.7"),
    hash_node_id("sculpt3d.alpha.8"),
    hash_node_id("sculpt3d.alpha.image"),
];

/// **O TAMANHO DO CARIMBO, em fração da ALTURA DA TELA** — ver
/// `ph2d_sculpt3d::Brush::alpha_stencil_scale`.
///
/// ⚠️ **Id PRÓPRIO, e não o do `Pattern Size`.** Uma imagem é um estêncil preso
/// ao viewport e é medida na TELA; os nove procedurais são campos 3-D e são
/// medidos no MODELO. O mesmo widget com duas réguas trocaria de significado em
/// silêncio ao trocar de padrão — e o artista não teria como saber qual das duas
/// está segurando.
pub const SCULPT3D_STAMP_SCALE: NodeId = hash_node_id("sculpt3d.stamp_scale");

/// Chip ligado a [`SCULPT3D_STAMP_SCALE`].
pub const SCULPT3D_STAMP_SCALE_NUM: NodeId = hash_node_id("sculpt3d.stamp_scale_num");

/// **ONDE o carimbo POUSA**, ao longo da tangente do frame e em fração da
/// ALTURA DA TELA — ver `ph2d_sculpt3d::Brush::alpha_offset`.
///
/// ⚠️ **Dois ids e não um par XY num controle só**, porque as duas pistas deste
/// painel são de UM número: um widget de dois eixos seria o primeiro do painel e
/// pediria hit-test, arrasto e chip próprios — trabalho que não compra nada que
/// duas pistas irmãs não deem.
pub const SCULPT3D_ALPHA_OFF_X: NodeId = hash_node_id("sculpt3d.alpha_off_x");

/// Chip ligado a [`SCULPT3D_ALPHA_OFF_X`].
pub const SCULPT3D_ALPHA_OFF_X_NUM: NodeId = hash_node_id("sculpt3d.alpha_off_x_num");

/// A outra metade da colocação — ver [`SCULPT3D_ALPHA_OFF_X`].
pub const SCULPT3D_ALPHA_OFF_Y: NodeId = hash_node_id("sculpt3d.alpha_off_y");

/// Chip ligado a [`SCULPT3D_ALPHA_OFF_Y`].
pub const SCULPT3D_ALPHA_OFF_Y_NUM: NodeId = hash_node_id("sculpt3d.alpha_off_y_num");

/// Tamanho de uma feature do alpha, em unidades de objeto.
pub const SCULPT3D_ALPHA_SCALE: NodeId = hash_node_id("sculpt3d.alpha_scale");

/// Chip ligado a [`SCULPT3D_ALPHA_SCALE`].
pub const SCULPT3D_ALPHA_SCALE_NUM: NodeId = hash_node_id("sculpt3d.alpha_scale_num");

/// **O AZIMUTE do eixo de um padrão DIRECIONAL.**
///
/// ⚠️ **Não é a lâmpada, e a distinção importa mais do que parece:** os dois
/// pares de pistas falam a mesma língua (azimute + elevação em graus, o rotor do
/// app) e descrevem coisas diferentes — um aponta a LUZ, o outro aponta o
/// PADRÃO. Ids separados são o que impede um clique de virar o outro.
pub const SCULPT3D_ALPHA_AZ: NodeId = hash_node_id("sculpt3d.alpha_az");

/// Chip ligado a [`SCULPT3D_ALPHA_AZ`].
pub const SCULPT3D_ALPHA_AZ_NUM: NodeId = hash_node_id("sculpt3d.alpha_az_num");

/// A ELEVAÇÃO do eixo — ver [`SCULPT3D_ALPHA_AZ`].
pub const SCULPT3D_ALPHA_ELEV: NodeId = hash_node_id("sculpt3d.alpha_elev");

/// Chip ligado a [`SCULPT3D_ALPHA_ELEV`].
pub const SCULPT3D_ALPHA_ELEV_NUM: NodeId = hash_node_id("sculpt3d.alpha_elev_num");

/// **O preview do padrão NO BARRO** — o interruptor do tinto que mostra, na
/// peça, o que o próximo traço vai depositar.
pub const SCULPT3D_ALPHA_PREVIEW: NodeId = hash_node_id("sculpt3d.alpha_preview");

// ── O espelho ───────────────────────────────────────────────────────────────
// TRÊS botões e não um rádio: os eixos são independentes (o ZBrush espelha em
// dois ao mesmo tempo), e um segmented é *um de N* por construção.
/// Espelho em X.
pub const SCULPT3D_SYM_X: NodeId = hash_node_id("sculpt3d.sym.x");

/// Espelho em Y.
pub const SCULPT3D_SYM_Y: NodeId = hash_node_id("sculpt3d.sym.y");

/// Espelho em Z.
pub const SCULPT3D_SYM_Z: NodeId = hash_node_id("sculpt3d.sym.z");
