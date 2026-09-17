//! Os ids do pincel de **POSE** — irmão (`#[path]`) do [`super::sculpt3d`],
//! cortado por assunto, como os do tecido.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **Qual das CINCO deformações** — `ph2d_sculpt3d::PoseDeformacao::ALL`.
///
/// ⭐⭐ **Eram TRÊS até 2026-09-15**, e a subida é ordem do dono: cada um dos
/// três modos tinha uma segunda metade atrás do `Ctrl`, e *um gesto que só
/// existe se o artista adivinhar o modificador é meio gesto*.
///
/// ⚠️ **O tamanho CONTA-SE, não se escolhe:** o censo compara este array com o
/// `ALL` do motor, logo uma deformação nova que não passe por aqui nasce
/// inalcançável e o gate fica vermelho — em vez de o chip sumir em silêncio.
///
/// ⚠️ **A fileira só é desenhada com o verbo Pose na mão** — e foi exactamente
/// isso que a deixou **morta sob o ponteiro** até 2026-09-15: nenhuma fixtura
/// de costura armava este pincel. O `populate` regista-a agora, e o censo
/// derivado (`populate_censo_tests`) impede a oitava ocorrência.
pub const SCULPT3D_POSE_MODE: [NodeId; 5] = [
    hash_node_id("sculpt3d.pose_mode.0"),
    hash_node_id("sculpt3d.pose_mode.1"),
    hash_node_id("sculpt3d.pose_mode.2"),
    hash_node_id("sculpt3d.pose_mode.3"),
    hash_node_id("sculpt3d.pose_mode.4"),
];

/// **Quantos SEGMENTOS a cadeia tem.** Com mais de um ela dobra como um braço.
pub const SCULPT3D_POSE_SEGMENTS: NodeId = hash_node_id("sculpt3d.pose_segments");
pub const SCULPT3D_POSE_SEGMENTS_NUM: NodeId = hash_node_id("sculpt3d.pose_segments.num");

/// **O DESVIO DA ORIGEM** — afasta o pivô do cursor, em múltiplos do raio.
pub const SCULPT3D_POSE_OFFSET: NodeId = hash_node_id("sculpt3d.pose_offset");
pub const SCULPT3D_POSE_OFFSET_NUM: NodeId = hash_node_id("sculpt3d.pose_offset.num");

/// **Quantas SUAVIZAÇÕES os pesos levam.**
pub const SCULPT3D_POSE_SMOOTHINGS: NodeId = hash_node_id("sculpt3d.pose_smoothings");
pub const SCULPT3D_POSE_SMOOTHINGS_NUM: NodeId = hash_node_id("sculpt3d.pose_smoothings.num");

/// ***ANCORADO*** — prende a extremidade distante da cadeia.
///
/// ⭐ **Com um segmento e ancorado, a deformação é rotação PURA em torno do
/// pivô** — que é o que o nome do pincel promete, e é por isso que ele nasce
/// ligado. ⚠️ Esse default é **recomendação nossa, declarada**: qual dos dois o
/// artista encontra no alvo **não foi medido**.
pub const SCULPT3D_POSE_ANCHORED: NodeId = hash_node_id("sculpt3d.pose_anchored");

/// ***TRAVA DE ROTAÇÃO*** — no modo de escala, escala **sem rodar**.
pub const SCULPT3D_POSE_ROT_LOCK: NodeId = hash_node_id("sculpt3d.pose_rot_lock");
