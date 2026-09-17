//! Os ids do pincel de **PROJECTAR NA CENA** — irmão (`#[path]`) do
//! [`super::sculpt3d`], cortado por assunto, como os do tecido, da pose, do
//! contorno e do esfregão.
//!
//! ⚠️ **São TRÊS superfícies e a espec conta TRÊS** (§6.2, §6.3.2 e §6.3.4): a
//! direcção do raio, procurar também para trás, e a folga. Os outros knobs dele
//! (raio, força, curva, dureza) são os partilhados, que já vivem na secção do
//! pincel. *Um painel nosso com um quarto knob está a inventar.*

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **PARA ONDE O RAIO APONTA** — `ph2d_sculpt3d::ProjectMode::ALL`.
///
/// ⚠️ **O tamanho CONTA-SE, não se escolhe:** o censo compara este array com o
/// `ALL` do motor, logo um modo novo que não passe por aqui nasce inalcançável
/// e o gate fica vermelho — em vez de o chip sumir em silêncio.
pub const SCULPT3D_PROJECT_MODE: [NodeId; 2] = [
    hash_node_id("sculpt3d.project_mode.0"),
    hash_node_id("sculpt3d.project_mode.1"),
];

/// **PROCURAR TAMBÉM PARA TRÁS** — a caixa do `use_bidirectional` (espec §6.3.2).
pub const SCULPT3D_PROJECT_BIDIR: NodeId = hash_node_id("sculpt3d.project_bidir");

/// **A FOLGA** — quanto barro fica antes de encostar (espec §6.3.4).
pub const SCULPT3D_PROJECT_MIN_DIST: NodeId = hash_node_id("sculpt3d.project_min_dist");
/// O chip numérico da folga — o par de sempre de um slider deste painel.
pub const SCULPT3D_PROJECT_MIN_DIST_NUM: NodeId = hash_node_id("sculpt3d.project_min_dist.num");
