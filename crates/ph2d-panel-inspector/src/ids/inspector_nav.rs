//! **Os ids das secções NAV REGION e NAV AGENT** (plano 30, W4).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do [`super::inspector_ray`].
//!
//! # ⚠️ Dois arrays de botões, e a POSIÇÃO é a tag do clique
//!
//! As oito camadas da máscara e os três modos do alvo: reordenar faria um clique escrever outra
//! coisa — e compila (a armadilha que o `SignalVerb::ALL` desta linha pagou em 19/09).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Meia-largura da região, em metros.
pub const INSP_NAV_HALF_W: NodeId = hash_node_id("insp_nav_half_w");
/// Meia-altura da região, em metros.
pub const INSP_NAV_HALF_H: NodeId = hash_node_id("insp_nav_half_h");
/// As OITO camadas cujos corpos estáticos bloqueiam — **um botão por camada**, e cada clique troca
/// UM bit da máscara (não é um segmentado de escolha única: uma parede pode estar em várias).
pub const INSP_NAV_LAYERS: [NodeId; 8] = [
    hash_node_id("insp_nav_layer_0"),
    hash_node_id("insp_nav_layer_1"),
    hash_node_id("insp_nav_layer_2"),
    hash_node_id("insp_nav_layer_3"),
    hash_node_id("insp_nav_layer_4"),
    hash_node_id("insp_nav_layer_5"),
    hash_node_id("insp_nav_layer_6"),
    hash_node_id("insp_nav_layer_7"),
];
/// O segmentado do ALVO — `None · Object · Point`, na ordem de `NavAlvoModo::ALL`.
pub const INSP_NAV_TARGET_MODE: [NodeId; 3] = [
    hash_node_id("insp_nav_target_none"),
    hash_node_id("insp_nav_target_object"),
    hash_node_id("insp_nav_target_point"),
];
/// O NOME do alvo (modo `Object`) — ⚠️ o nome e nunca os bits (a lei do alvo do projéctil).
pub const INSP_NAV_TARGET_NAME: NodeId = hash_node_id("insp_nav_target_name");
/// O ponto do alvo (modo `Point`) — o eixo X.
pub const INSP_NAV_TARGET_X: NodeId = hash_node_id("insp_nav_target_x");
/// …e o eixo Y.
pub const INSP_NAV_TARGET_Y: NodeId = hash_node_id("insp_nav_target_y");
/// O raio do corpo para efeitos de caminho — ⚠️ `0` é DERIVADO do colisor.
pub const INSP_NAV_RADIUS: NodeId = hash_node_id("insp_nav_radius");
/// A esta distância do alvo ele conta como chegado.
pub const INSP_NAV_ARRIVE: NodeId = hash_node_id("insp_nav_arrive");
/// O alvo tem de andar mais do que isto para valer um recálculo.
pub const INSP_NAV_REPATH: NodeId = hash_node_id("insp_nav_repath");
/// Sem progresso durante isto, ele está PRESO — `0` desliga.
pub const INSP_NAV_STUCK: NodeId = hash_node_id("insp_nav_stuck");
/// Ligado / desligado.
pub const INSP_NAV_ACTIVE: NodeId = hash_node_id("insp_nav_active");
/// O sinal ao CHEGAR — vazio = calado.
pub const INSP_NAV_ON_ARRIVED: NodeId = hash_node_id("insp_nav_on_arrived");
/// O sinal quando não há caminho (ou só um parcial).
pub const INSP_NAV_ON_NO_PATH: NodeId = hash_node_id("insp_nav_on_no_path");
/// O sinal ao ficar PRESO.
pub const INSP_NAV_ON_STUCK: NodeId = hash_node_id("insp_nav_on_stuck");
