//! **Os ids da secção PATH FOLLOW** (suplente #23).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do
//! [`super::inspector_projectile`].
//!
//! # ⚠️⚠️ AQUI HÁ SEGMENTADOS, e a POSIÇÃO no array É a tag
//!
//! Ao contrário do projéctil, esta secção tem **quatro** grupos de chips (ciclo · família da curva ·
//! modo · ao acabar), e em todos eles o índice do array é o que o despachante converte na tag do
//! enum. ⇒ **reordenar um destes arrays muda a semântica de toda cena gravada**, sem uma linha de
//! erro. A cura é a mesma das irmãs: acrescentar no FIM, nunca a meio.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **O NOME da forma a percorrer** — um campo de texto, como o alvo da câmera.
pub const INSP_PF_CAMINHO: NodeId = hash_node_id("insp_pf_caminho");
/// O ÍNDICE do relógio que o faz andar.
pub const INSP_PF_RELOGIO: NodeId = hash_node_id("insp_pf_relogio");
/// A duração desse relógio, em segundos — ⭐ a mesma porta que a secção TIMERS escreve.
pub const INSP_PF_DURACAO: NodeId = hash_node_id("insp_pf_duracao");
/// O `repeat` desse relógio.
pub const INSP_PF_REPEAT: NodeId = hash_node_id("insp_pf_repeat");
/// O `autostart` desse relógio.
pub const INSP_PF_AUTOSTART: NodeId = hash_node_id("insp_pf_autostart");
/// Onde no percurso ele entra, em fracção `0..1`.
pub const INSP_PF_DESLOCAMENTO: NodeId = hash_node_id("insp_pf_deslocamento");
/// Roda para a tangente?
pub const INSP_PF_ALINHA: NodeId = hash_node_id("insp_pf_alinha");
/// O ângulo somado à tangente, em GRAUS.
pub const INSP_PF_ANGULO: NodeId = hash_node_id("insp_pf_angulo");
/// O deslocamento perpendicular, em metros.
pub const INSP_PF_LADO: NodeId = hash_node_id("insp_pf_lado");

/// **Os chips do CICLO** — a posição É a tag do `ph2d_tween::Ciclo`.
pub const INSP_PF_CICLO: [NodeId; 2] = [
    hash_node_id("insp_pf_ciclo_0"),
    hash_node_id("insp_pf_ciclo_1"),
];

/// **Os chips do AO ACABAR** — a posição É a tag do `ph2d_tween::AoAcabar`.
pub const INSP_PF_AO_ACABAR: [NodeId; 2] =
    [hash_node_id("insp_pf_fim_0"), hash_node_id("insp_pf_fim_1")];

/// **Os chips da FAMÍLIA da curva** — a posição É a tag do `ph2d_anim::EasingFamily`.
pub const INSP_PF_FAMILIA: [NodeId; 11] = [
    hash_node_id("insp_pf_fam_0"),
    hash_node_id("insp_pf_fam_1"),
    hash_node_id("insp_pf_fam_2"),
    hash_node_id("insp_pf_fam_3"),
    hash_node_id("insp_pf_fam_4"),
    hash_node_id("insp_pf_fam_5"),
    hash_node_id("insp_pf_fam_6"),
    hash_node_id("insp_pf_fam_7"),
    hash_node_id("insp_pf_fam_8"),
    hash_node_id("insp_pf_fam_9"),
    hash_node_id("insp_pf_fam_10"),
];

/// **Os chips do MODO da curva** — a posição É a tag do `ph2d_anim::EasingMode`.
pub const INSP_PF_MODO: [NodeId; 3] = [
    hash_node_id("insp_pf_modo_0"),
    hash_node_id("insp_pf_modo_1"),
    hash_node_id("insp_pf_modo_2"),
];
