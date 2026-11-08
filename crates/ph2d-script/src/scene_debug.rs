//! ⭐ **O depurador na cena** (plano 13 Onda D, decisão 11 do dono): pontos de parada armados no
//! topo de cada módulo carregado, os objectos parados numa linha e o Continue / Next line.
//!
//! ⚠️ **Parar um objecto não pára os outros no MESMO quadro.** Cada gancho lê o mundo do início do
//! tique e as escritas só pousam no fim, por isso deixar um objecto suspenso enquanto os outros
//! acabam o quadro não muda o que o jogo calcula; o jogo congela depois (quem chama, por
//! [`SceneScripts::is_held`]). Os que batem no mesmo ponto ficam em fila, pela ordem da identidade:
//! Continue passa ao seguinte, como no Godot.

use std::collections::BTreeMap;

use mlua::Lua;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_script_debug::{StopFrame, StopKind};
use ph2d_script_doc::ScriptId;

use super::life::{call, finish_pass};
use super::wait::{Park, Parked, debugger, park, resume};
use super::{Deadline, SceneReport, SceneScripts, settle_call};
use crate::console::speaking;
use crate::lane::{Lane, SceneDoors, with_lane};

/// Os pontos de parada armados num módulo.
pub(super) struct Armed {
    /// O topo do módulo em que foram armados (outro = recarregou, rearma-se).
    main: usize,
    asked: Vec<u32>,
    got: Vec<u32>,
}

/// Um objecto que parou por um erro (Pause on Error): já não corre; o jogo espera o Continue.
pub(super) struct HeldError {
    entity: Entity,
    script: ScriptId,
    message: String,
    frame: StopFrame,
}

/// ⭐ **Onde o jogo está parado** — o que a aba mostra.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugStop {
    pub entity: Entity,
    pub script: ScriptId,
    pub why: StopKind,
    /// A linha, as variáveis e a pilha (`None` se a pilha já não se lê).
    pub frame: Option<StopFrame>,
    /// Quantos objectos esperam a vez depois deste.
    pub queued: usize,
}

impl SceneScripts {
    /// Arma `wanted` (linhas 1-based por script) em cada módulo carregado — só o que mudou.
    pub(crate) fn arm_breakpoints(&mut self, lua: &Lua, wanted: &BTreeMap<ScriptId, Vec<u32>>) {
        let Some(d) = debugger(lua) else { return };
        self.armed.retain(|id, _| self.slots.contains_key(id));
        for (id, slot) in &self.slots {
            let Some(m) = slot.module.as_ref() else {
                continue;
            };
            let main = m.main();
            let at = main.to_pointer() as usize;
            let asked = wanted.get(id).cloned().unwrap_or_default();
            let prev = self.armed.get(id).filter(|a| a.main == at);
            if prev.is_some_and(|a| a.asked == asked) {
                continue;
            }
            let previous = prev.map(|a| a.got.clone()).unwrap_or_default();
            if asked.is_empty() && previous.is_empty() {
                self.armed.remove(id);
                continue;
            }
            let got = d.arm(lua, main, &asked, &previous).unwrap_or_default();
            self.armed.insert(
                *id,
                Armed {
                    main: at,
                    asked,
                    got,
                },
            );
        }
    }

    /// As linhas onde os pontos de parada de `id` ficaram (a próxima linha com código).
    #[must_use]
    pub fn armed_lines(&self, id: ScriptId) -> &[u32] {
        self.armed.get(&id).map_or(&[], |a| &a.got)
    }

    /// De quem são os valores ao vivo de `id`: `prefer` se corre esse script, senão o 1.º pela
    /// identidade.
    pub(crate) fn live_target(&self, id: ScriptId, prefer: Option<Entity>) -> Option<Entity> {
        let runs = |bits: u64| {
            self.instances
                .get(&bits)
                .is_some_and(|i| i.script == id && i.runs())
        };
        if let Some(e) = prefer.filter(|e| runs(e.to_bits())) {
            return Some(e);
        }
        self.instances
            .iter()
            .filter(|(_, i)| i.script == id && i.runs())
            .min_by_key(|(_, i)| i.sid)
            .and_then(|(bits, _)| Entity::try_from_bits(*bits))
    }

    /// Há um objecto parado (ponto de parada, Next line ou um erro)? O jogo congela.
    #[must_use]
    pub fn is_held(&self) -> bool {
        self.held_error.is_some() || self.parked.iter().any(Parked::is_held)
    }

    /// Onde o jogo está parado.
    #[must_use]
    pub fn debug_stop(&self, lua: &Lua) -> Option<DebugStop> {
        let queued = self.parked.iter().filter(|p| p.is_held()).count();
        if let Some(e) = &self.held_error {
            return Some(DebugStop {
                entity: e.entity,
                script: e.script,
                why: StopKind::Error(e.message.clone()),
                frame: Some(e.frame.clone()),
                queued,
            });
        }
        let p = self.parked.iter().find(|p| p.is_held())?;
        let Park::Held { why, .. } = &p.kind else {
            return None;
        };
        let frame = debugger(lua).and_then(|d| d.frame(lua, &p.thread).ok().flatten());
        Some(DebugStop {
            entity: p.entity,
            script: p.script,
            why: why.clone(),
            frame,
            queued: queued.saturating_sub(1),
        })
    }

    /// O 1.º erro guardado num relatório vira a paragem (Pause on Error), se nada está parado.
    pub(crate) fn absorb_error_stop(&mut self, report: &mut SceneReport) {
        let stops = std::mem::take(&mut report.error_stops);
        if self.held_error.is_some() {
            return;
        }
        if let Some((entity, script, message, frame)) = stops.into_iter().next() {
            self.held_error = Some(HeldError {
                entity,
                script,
                message,
                frame,
            });
        }
    }

    /// ⭐ **Continue** (`step = false`) ou **Next line** (`step = true`) do 1.º objecto parado.
    /// Os ganchos que ele devia ter corrido entretanto correm a seguir, pela ordem; as escritas
    /// pousam como no fim de um tique.
    pub(crate) fn resume_held(
        &mut self,
        lua: &Lua,
        deadline: &Deadline,
        sim: &mut SimWorld,
        doors: &mut dyn SceneDoors,
        step: bool,
    ) -> SceneReport {
        let mut report = SceneReport::default();
        // Depois de um erro não há o que retomar: o objecto já parou, o jogo segue.
        if self.held_error.take().is_some() {
            return report;
        }
        let Some(at) = self.parked.iter().position(Parked::is_held) else {
            return report;
        };
        let p = self.parked.remove(at);
        let Park::Held { then, .. } = p.kind else {
            return report;
        };
        if let Some(d) = debugger(lua) {
            let _ = d.before_resume(lua, &p.thread, step);
        }
        let mut writes = Vec::new();
        let Self {
            instances,
            parked,
            dying,
            ..
        } = self;
        let lane = Lane::new(sim, doors, dying);
        let (entity, script, thread) = (p.entity, p.script, p.thread);
        let ran = with_lane(lua, lane, |cell| {
            let Some(inst) = instances.get_mut(&entity.to_bits()) else {
                return;
            };
            if inst.script != script || !inst.runs() || cell.borrow().is_dying(entity) {
                return;
            }
            cell.borrow_mut().speaker = Some(entity);
            report.calls += 1;
            let ran = speaking(lua, inst.script, Some(entity), || {
                deadline.within(|| resume(lua, thread, ()))
            });
            let mut again = Vec::new();
            let ran = park(ran, entity, script, &mut again);
            let alive = settle_call(lua, entity, ran, inst, &mut writes, &mut report);
            let mut then = Some(then);
            for mut q in again {
                if let Park::Held { then: t, .. } = &mut q.kind {
                    // Parou outra vez: à frente da fila, com os ganchos que esperavam.
                    *t = then.take().unwrap_or_default();
                    parked.insert(at, q);
                } else {
                    parked.push(q);
                }
            }
            if let Some(then) = then.filter(|_| alive) {
                for (hook, args) in then {
                    if !inst.runs() || cell.borrow().is_dying(entity) {
                        break;
                    }
                    call(
                        lua,
                        deadline,
                        entity,
                        inst,
                        &hook,
                        args,
                        parked,
                        &mut writes,
                        &mut report,
                    );
                }
            }
        });
        finish_pass(lua, sim, doors, ran, &writes, report)
    }
}
