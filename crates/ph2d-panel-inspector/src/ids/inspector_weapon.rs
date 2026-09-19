//! **Os ids da secção WEAPON** — a arma do jogador.
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do
//! [`super::inspector_ray`] e do [`super::inspector_projectile`].
//!
//! # ⚠️ Não há segmentado nenhum, e isso é o desenho
//!
//! A arma são **dois números e seis nomes**: ⇒ **não há posição-no-array a ser tag de clique
//! aqui**, que é a armadilha que os três segmentados do `SCULPT3D_POSE_MODE` carregam, e que o
//! `SignalVerb::ALL` desta linha pagou em 19/09 (o verbo existia, tinha lei e gates, e o artista
//! não lhe chegava porque o array de ids ficou com um a menos).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O sinal que puxa o gatilho — ⚠️ vazio = **nunca dispara**, e a secção di-lo na queixa.
pub const INSP_WEAPON_ON_SIGNAL: NodeId = hash_node_id("insp_weapon_on_signal");
/// O intervalo mínimo entre dois tiros, em milissegundos. `0` = sem cadência.
pub const INSP_WEAPON_COOLDOWN: NodeId = hash_node_id("insp_weapon_cooldown");
/// O nome do contador que É o pente — ⚠️ vazio = munição **infinita**.
pub const INSP_WEAPON_AMMO: NodeId = hash_node_id("insp_weapon_ammo");
/// Quanto demora a recarregar, em milissegundos. `0` = não recarrega.
pub const INSP_WEAPON_RELOAD_MS: NodeId = hash_node_id("insp_weapon_reload_ms");
/// O sinal que manda recarregar antes de esvaziar.
pub const INSP_WEAPON_RELOAD_ON: NodeId = hash_node_id("insp_weapon_reload_on");
/// O sinal publicado a cada tiro — ⭐ **o fio para a `Factory`**, e sem ele nada nasce.
pub const INSP_WEAPON_ON_FIRE: NodeId = hash_node_id("insp_weapon_on_fire");
/// O clique seco — publicado quando o gatilho encontra o pente vazio.
pub const INSP_WEAPON_ON_EMPTY: NodeId = hash_node_id("insp_weapon_on_empty");
/// Publicado quando o pente fica cheio.
pub const INSP_WEAPON_ON_RELOADED: NodeId = hash_node_id("insp_weapon_on_reloaded");
