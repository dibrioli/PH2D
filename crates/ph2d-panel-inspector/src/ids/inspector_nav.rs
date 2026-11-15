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
pub const INSP_NAV_TARGET_MODE: [NodeId; 5] = [
    hash_node_id("insp_nav_target_none"),
    hash_node_id("insp_nav_target_object"),
    hash_node_id("insp_nav_target_point"),
    // (W6) APENDADOS — a posição é a tag do clique.
    hash_node_id("insp_nav_target_tag"),
    hash_node_id("insp_nav_target_patrol"),
];
/// (W6) O CHIP da tag do modo `Tag` — as entradas dele são [`super::INSP_NAV_TAG_OPT`].
pub const INSP_NAV_TAG_PICK: NodeId = hash_node_id("insp_nav_tag_pick");
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
/// (W5) Desvia dos outros corpos que andam.
pub const INSP_NAV_AVOIDANCE: NodeId = hash_node_id("insp_nav_avoidance");
/// O sinal ao CHEGAR — vazio = calado.
pub const INSP_NAV_ON_ARRIVED: NodeId = hash_node_id("insp_nav_on_arrived");
/// O sinal quando não há caminho (ou só um parcial).
pub const INSP_NAV_ON_NO_PATH: NodeId = hash_node_id("insp_nav_on_no_path");
/// O sinal ao ficar PRESO.
pub const INSP_NAV_ON_STUCK: NodeId = hash_node_id("insp_nav_on_stuck");
/// (W7) Evita as zonas que o FEREM (um `Damage` parado que o `Health` dele sente).
pub const INSP_NAV_AVOID_HARM: NodeId = hash_node_id("insp_nav_avoid_harm");
/// (W7) O custo da ÁREA, em múltiplos do chão.
pub const INSP_NAV_AREA_COST: NodeId = hash_node_id("insp_nav_area_cost");
/// (W7) A área PROIBIDA — um furo para todos.
pub const INSP_NAV_AREA_FORBIDDEN: NodeId = hash_node_id("insp_nav_area_forbidden");
/// (W7) O NOME da saída do atalho — ⚠️ o nome e nunca os bits (a lei do alvo `Object`).
pub const INSP_NAV_LINK_TO: NodeId = hash_node_id("insp_nav_link_to");
/// (W7) O atalho também da saída para a entrada.
pub const INSP_NAV_LINK_TWO_WAY: NodeId = hash_node_id("insp_nav_link_two_way");
/// (W7) O corpo SALTA para a saída; desligado, anda a direito até ela.
pub const INSP_NAV_LINK_TELEPORT: NodeId = hash_node_id("insp_nav_link_teleport");
/// (W7) O custo a mais de atravessar, em metros de chão.
pub const INSP_NAV_LINK_COST: NodeId = hash_node_id("insp_nav_link_cost");
/// (W7) O sinal quando um agente o atravessa — vazio = calado.
pub const INSP_NAV_LINK_ON_CROSSED: NodeId = hash_node_id("insp_nav_link_on_crossed");

/// (dono 09/11) **Quantos corpos de formas todas desligadas ganham botão** no aviso da NAV REGION. O
/// recurso é a tabela compilada de ids (o censo de colisões só vê ids fixos), não o motor; `32` é o tecto
/// das outras filas de chips do Inspector (`INSP_PHYS_LAYERS`, os chips de enum do script). O resto vira
/// «+N more», sem clique.
pub const MAX_NAV_OFF_BODIES: usize = 32;
/// O botão do corpo `k` da lista (`InspectorNavInfo::corpos_desligados`): seleciona-o na cena e no Inspector.
pub const INSP_NAV_OFF_BODY: [NodeId; MAX_NAV_OFF_BODIES] =
    super::inspector_audio::tabela_de_linha(hash_node_id("insp_nav_off_body"));
