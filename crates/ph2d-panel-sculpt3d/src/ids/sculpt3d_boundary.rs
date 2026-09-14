//! Os ids do pincel de **CONTORNO** — irmão (`#[path]`) do [`super::sculpt3d`],
//! cortado por assunto, como os do tecido e os da pose.
//!
//! ⚠️⚠️ **São TRÊS superfícies e é a espec que as conta** (§15): o painel do
//! alvo tem exactamente quatro controlos próprios, e o quarto — o alvo de
//! deformação — depende do solver de pano, que é outra espec. *Um painel nosso
//! com um quinto knob está a inventar, e um com dois está a esconder.*

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **Qual das SEIS deformações** — `ph2d_sculpt3d::BoundaryModo::ALL`.
///
/// ⚠️ **O tamanho CONTA-SE, não se escolhe:** o censo compara este array com o
/// `ALL` do motor, logo um modo novo que não passe por aqui nasce inalcançável
/// e o gate fica vermelho — em vez de o chip sumir em silêncio.
pub const SCULPT3D_BOUNDARY_MODE: [NodeId; 6] = [
    hash_node_id("sculpt3d.boundary_mode.0"),
    hash_node_id("sculpt3d.boundary_mode.1"),
    hash_node_id("sculpt3d.boundary_mode.2"),
    hash_node_id("sculpt3d.boundary_mode.3"),
    hash_node_id("sculpt3d.boundary_mode.4"),
    hash_node_id("sculpt3d.boundary_mode.5"),
];

/// **Como a deformação esmorece AO LONGO da borda** — os quatro modos de queda
/// no contorno.
///
/// ⚠️ Ela é uma pergunta **diferente** da curva do pincel: aquela gradua a
/// profundidade (para dentro da peça), esta gradua ao longo do troço de borda.
pub const SCULPT3D_BOUNDARY_FALLOFF: [NodeId; 4] = [
    hash_node_id("sculpt3d.boundary_falloff.0"),
    hash_node_id("sculpt3d.boundary_falloff.1"),
    hash_node_id("sculpt3d.boundary_falloff.2"),
    hash_node_id("sculpt3d.boundary_falloff.3"),
];

/// **O DESLOCAMENTO DA ORIGEM** — ⚠️⚠️ ele alonga a **PROPAGAÇÃO** (e portanto o
/// braço de alavanca da dobra) e **NÃO** o troço de borda afectado. Confundir os
/// dois raios é o erro caro deste pincel.
pub const SCULPT3D_BOUNDARY_OFFSET: NodeId = hash_node_id("sculpt3d.boundary_offset");
pub const SCULPT3D_BOUNDARY_OFFSET_NUM: NodeId = hash_node_id("sculpt3d.boundary_offset.num");
