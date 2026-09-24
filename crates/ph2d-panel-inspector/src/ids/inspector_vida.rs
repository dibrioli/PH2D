//! **Os ids das secções HEALTH e DAMAGE** (plano 28, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do
//! [`super::inspector_projectile`].
//!
//! # ⚠️ Não há segmentado nenhum, e isso é o desenho
//!
//! O `OnHit` do dano tem DOIS estados e é pintado como uma CAIXA (*Vanishes on hit*): um segmentado
//! de duas opções seria uma posição-no-array a ser tag de clique, a armadilha que os segmentados do
//! irmão carregam, para dizer uma coisa que uma caixa diz.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── HEALTH — os números ─────────────────────────────────────────────────────
/// O máximo. `0` = sem máximo.
pub const INSP_VIDA_MAX: NodeId = hash_node_id("insp_vida_max");
/// A vida com que nasce.
pub const INSP_VIDA_START: NodeId = hash_node_id("insp_vida_start");
/// A invencibilidade depois de um golpe, em segundos.
pub const INSP_VIDA_INVINCIBLE: NodeId = hash_node_id("insp_vida_invincible");
/// Regeneração, pontos por segundo.
pub const INSP_VIDA_REGEN: NodeId = hash_node_id("insp_vida_regen");
/// Quanto tempo depois do último golpe a regeneração arranca. ⚠️ Só pintado com regeneração.
pub const INSP_VIDA_REGEN_DELAY: NodeId = hash_node_id("insp_vida_regen_delay");
/// O escudo com que nasce.
pub const INSP_VIDA_SHIELD_START: NodeId = hash_node_id("insp_vida_shield_start");
/// O máximo do escudo.
pub const INSP_VIDA_SHIELD_MAX: NodeId = hash_node_id("insp_vida_shield_max");
/// Quanto dura um escudo. ⚠️ Só pintado com escudo.
pub const INSP_VIDA_SHIELD_DURATION: NodeId = hash_node_id("insp_vida_shield_duration");
/// Regeneração do escudo. ⚠️ Só pintado com escudo.
pub const INSP_VIDA_SHIELD_REGEN: NodeId = hash_node_id("insp_vida_shield_regen");
/// O atraso da regeneração do escudo. ⚠️ Só pintado com escudo que regenera.
pub const INSP_VIDA_SHIELD_REGEN_DELAY: NodeId = hash_node_id("insp_vida_shield_regen_delay");
/// Armadura plana.
pub const INSP_VIDA_ARMOR: NodeId = hash_node_id("insp_vida_armor");
/// Armadura percentual, fracção `0..1`.
pub const INSP_VIDA_ARMOR_PCT: NodeId = hash_node_id("insp_vida_armor_pct");
/// A chance de esquivar, fracção `0..1`.
pub const INSP_VIDA_DODGE: NodeId = hash_node_id("insp_vida_dodge");
/// A semente da esquiva. ⚠️ Só pintada com esquiva.
pub const INSP_VIDA_SEED: NodeId = hash_node_id("insp_vida_seed");

// ── HEALTH — as caixas e os nomes ───────────────────────────────────────────
/// Uma cura pode passar do máximo. ⚠️ Só pintada com máximo.
pub const INSP_VIDA_OVERHEAL: NodeId = hash_node_id("insp_vida_overheal");
/// O golpe que parte o escudo não passa à vida. ⚠️ Só pintada com escudo.
pub const INSP_VIDA_SHIELD_BLOCKS: NodeId = hash_node_id("insp_vida_shield_blocks");
/// A equipa.
pub const INSP_VIDA_TEAM: NodeId = hash_node_id("insp_vida_team");
/// O sinal ao levar dano.
pub const INSP_VIDA_ON_DAMAGE: NodeId = hash_node_id("insp_vida_on_damage");
/// O sinal ao ser curado.
pub const INSP_VIDA_ON_HEAL: NodeId = hash_node_id("insp_vida_on_heal");
/// O sinal ao morrer.
pub const INSP_VIDA_ON_DEATH: NodeId = hash_node_id("insp_vida_on_death");

// ── DAMAGE ──────────────────────────────────────────────────────────────────
/// Quanto tira por golpe (ou por segundo, com *Per Second*).
pub const INSP_DANO_AMOUNT: NodeId = hash_node_id("insp_dano_amount");
/// A equipa de quem bate.
pub const INSP_DANO_TEAM: NodeId = hash_node_id("insp_dano_team");
/// Fere por segundo enquanto toca.
pub const INSP_DANO_PER_SECOND: NodeId = hash_node_id("insp_dano_per_second");
/// Atravessa o escudo.
pub const INSP_DANO_IGNORES_SHIELD: NodeId = hash_node_id("insp_dano_ignores_shield");
/// Atravessa a armadura.
pub const INSP_DANO_IGNORES_ARMOR: NodeId = hash_node_id("insp_dano_ignores_armor");
/// Sai da cena ao bater (uma bala).
pub const INSP_DANO_VANISH: NodeId = hash_node_id("insp_dano_vanish");
