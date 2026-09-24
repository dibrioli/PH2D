//! **As MORTES que a física anuncia neste dispatch** — a porta que a shell drena (plano 28, W2).
//!
//! A ponte **anuncia** e nunca apaga: *«quando é que isto sai da cena?»* é uma pergunta só, e a
//! resposta é o dreno único da shell (`fase_fabrica_e_morte`). Esta porta junta as TRÊS causas que
//! a física conhece, na ordem em que nasceram:
//!
//! | causa | produtor | `DeathCause` |
//! |---|---|---|
//! | o VOO acabou (alcance, ricochetes) | `drive_projectiles` (TOP-20 #14) | `Spent` |
//! | a VIDA chegou a zero | `drive_health` (plano 28) | `Killed` |
//! | quem bateu e é `OnHit::Vanish` | `drive_health` (plano 28) | `Spent` |
//!
//! ⚠️⚠️ **O filtro é a lei que protege o trabalho do artista, e ele mora AQUI** — só sai quem
//! [`ph2d_ecs::is_transient`] (nasceu numa corrida). Um inimigo ou uma bala postos à mão são
//! documento: um inimigo de documento que morre **fica** (deixa de levar golpes, grita o sinal), e
//! uma bala de documento que bateu pára.
//!
//! ⭐ **Nasceu de um CORTE da shell:** o bloco do projéctil vivia no `fase_fabrica_e_morte`, a fase
//! estava a `~186` linhas contra o tecto de `200`, e a catraca `the_shell_only_shrinks` proíbe a
//! shell de crescer. A regra da casa — *código de família vive na crate; a shell é composição* —
//! diz para onde: a shell passa a chamar UMA porta e encolhe.

use bevy_ecs::world::World;
use ph2d_ecs::{Death, DeathCause, Entity, is_transient};

use super::PhysicsBridge;
use super::health::HealthEventKind;

impl PhysicsBridge {
    /// **As mortes deste dispatch**, já filtradas por `is_transient`, sem repetidos.
    ///
    /// ⚠️ **Caladas** (`signal: ""`): o sinal de morte da VIDA já saiu pelo `signal_events` (com o
    /// nome que o artista escreveu na `Health`), e o de um projéctil é assunto do `Lifetime`, que já
    /// o autora. Um segundo nome aqui seria o mesmo facto a gritar duas vezes.
    #[must_use]
    pub fn mortes_anunciadas(&self, world: &World) -> Vec<Death> {
        let mut out: Vec<Death> = Vec::new();
        let mut junta = |entity: Entity, why: DeathCause| {
            if is_transient(world, entity) && !out.iter().any(|d| d.entity == entity) {
                out.push(Death {
                    entity,
                    signal: String::new(),
                    why,
                });
            }
        };
        for &(e, _) in self.projectile_done() {
            junta(e, DeathCause::Spent);
        }
        for ev in self.health_events() {
            if ev.kind == HealthEventKind::Died {
                junta(ev.target, DeathCause::Killed);
            }
        }
        for &e in self.damage_spent() {
            junta(e, DeathCause::Spent);
        }
        out
    }
}
