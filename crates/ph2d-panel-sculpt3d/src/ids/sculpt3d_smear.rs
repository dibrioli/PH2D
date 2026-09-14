//! Os ids do pincel de **ESFREGAR DESLOCAMENTO** — irmão (`#[path]`) do
//! [`super::sculpt3d`], cortado por assunto, como os do tecido, da pose e do
//! contorno.
//!
//! ⚠️ **É UMA superfície e a espec conta UMA** (§5.3): o alvo dá a este pincel
//! um selector próprio e mais nada — os outros knobs dele (raio, força, curva,
//! dureza) são os partilhados, que já vivem na secção do pincel. *Um painel
//! nosso com um segundo knob está a inventar.*

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **PARA ONDE ele empurra o deslocamento** — `ph2d_sculpt3d::SmearMode::ALL`.
///
/// ⚠️ **O tamanho CONTA-SE, não se escolhe:** o censo compara este array com o
/// `ALL` do motor, logo um modo novo que não passe por aqui nasce inalcançável
/// e o gate fica vermelho — em vez de o chip sumir em silêncio.
pub const SCULPT3D_SMEAR_MODE: [NodeId; 3] = [
    hash_node_id("sculpt3d.smear_mode.0"),
    hash_node_id("sculpt3d.smear_mode.1"),
    hash_node_id("sculpt3d.smear_mode.2"),
];
