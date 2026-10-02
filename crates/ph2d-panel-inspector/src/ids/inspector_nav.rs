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
/// (W6) O CHIP da tag do modo `Tag` — as entradas dele são [`INSP_NAV_TAG_OPT`].
pub const INSP_NAV_TAG_PICK: NodeId = hash_node_id("insp_nav_tag_pick");
/// (W6) As opções da tag — ⛔ ids PRÓPRIOS (a lei do `INSP_PHYS_TAG_OPT`: o despacho resolve a opção
/// por `position()` sobre o array, e com o mesmo array dois braços casariam). `64` = o mesmo tecto
/// das outras duas listas de tags do Inspector.
pub const INSP_NAV_TAG_OPT: [NodeId; 64] = [
    hash_node_id("insp_nav_tag_opt_00"),
    hash_node_id("insp_nav_tag_opt_01"),
    hash_node_id("insp_nav_tag_opt_02"),
    hash_node_id("insp_nav_tag_opt_03"),
    hash_node_id("insp_nav_tag_opt_04"),
    hash_node_id("insp_nav_tag_opt_05"),
    hash_node_id("insp_nav_tag_opt_06"),
    hash_node_id("insp_nav_tag_opt_07"),
    hash_node_id("insp_nav_tag_opt_08"),
    hash_node_id("insp_nav_tag_opt_09"),
    hash_node_id("insp_nav_tag_opt_10"),
    hash_node_id("insp_nav_tag_opt_11"),
    hash_node_id("insp_nav_tag_opt_12"),
    hash_node_id("insp_nav_tag_opt_13"),
    hash_node_id("insp_nav_tag_opt_14"),
    hash_node_id("insp_nav_tag_opt_15"),
    hash_node_id("insp_nav_tag_opt_16"),
    hash_node_id("insp_nav_tag_opt_17"),
    hash_node_id("insp_nav_tag_opt_18"),
    hash_node_id("insp_nav_tag_opt_19"),
    hash_node_id("insp_nav_tag_opt_20"),
    hash_node_id("insp_nav_tag_opt_21"),
    hash_node_id("insp_nav_tag_opt_22"),
    hash_node_id("insp_nav_tag_opt_23"),
    hash_node_id("insp_nav_tag_opt_24"),
    hash_node_id("insp_nav_tag_opt_25"),
    hash_node_id("insp_nav_tag_opt_26"),
    hash_node_id("insp_nav_tag_opt_27"),
    hash_node_id("insp_nav_tag_opt_28"),
    hash_node_id("insp_nav_tag_opt_29"),
    hash_node_id("insp_nav_tag_opt_30"),
    hash_node_id("insp_nav_tag_opt_31"),
    hash_node_id("insp_nav_tag_opt_32"),
    hash_node_id("insp_nav_tag_opt_33"),
    hash_node_id("insp_nav_tag_opt_34"),
    hash_node_id("insp_nav_tag_opt_35"),
    hash_node_id("insp_nav_tag_opt_36"),
    hash_node_id("insp_nav_tag_opt_37"),
    hash_node_id("insp_nav_tag_opt_38"),
    hash_node_id("insp_nav_tag_opt_39"),
    hash_node_id("insp_nav_tag_opt_40"),
    hash_node_id("insp_nav_tag_opt_41"),
    hash_node_id("insp_nav_tag_opt_42"),
    hash_node_id("insp_nav_tag_opt_43"),
    hash_node_id("insp_nav_tag_opt_44"),
    hash_node_id("insp_nav_tag_opt_45"),
    hash_node_id("insp_nav_tag_opt_46"),
    hash_node_id("insp_nav_tag_opt_47"),
    hash_node_id("insp_nav_tag_opt_48"),
    hash_node_id("insp_nav_tag_opt_49"),
    hash_node_id("insp_nav_tag_opt_50"),
    hash_node_id("insp_nav_tag_opt_51"),
    hash_node_id("insp_nav_tag_opt_52"),
    hash_node_id("insp_nav_tag_opt_53"),
    hash_node_id("insp_nav_tag_opt_54"),
    hash_node_id("insp_nav_tag_opt_55"),
    hash_node_id("insp_nav_tag_opt_56"),
    hash_node_id("insp_nav_tag_opt_57"),
    hash_node_id("insp_nav_tag_opt_58"),
    hash_node_id("insp_nav_tag_opt_59"),
    hash_node_id("insp_nav_tag_opt_60"),
    hash_node_id("insp_nav_tag_opt_61"),
    hash_node_id("insp_nav_tag_opt_62"),
    hash_node_id("insp_nav_tag_opt_63"),
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
/// (W5) Desvia dos outros corpos que andam.
pub const INSP_NAV_AVOIDANCE: NodeId = hash_node_id("insp_nav_avoidance");
/// O sinal ao CHEGAR — vazio = calado.
pub const INSP_NAV_ON_ARRIVED: NodeId = hash_node_id("insp_nav_on_arrived");
/// O sinal quando não há caminho (ou só um parcial).
pub const INSP_NAV_ON_NO_PATH: NodeId = hash_node_id("insp_nav_on_no_path");
/// O sinal ao ficar PRESO.
pub const INSP_NAV_ON_STUCK: NodeId = hash_node_id("insp_nav_on_stuck");
