//! ⭐⭐ **O ciclo de vida do objecto** (GameEngineFuncional N0, `docs/GameEngineFuncional/04_R0_N0_plano.md`):
//! `ready`, `process`, `exit`, os sinais com emissor (`on_signal(self, nome, de)`) e o
//! `ph2d.connect`. Os nomes são os do Godot; a ordem dentro do passo fixo está no [`SceneScripts::tick`].
//!
//! - **`exit` uma vez por vida.** Despawn: no fim da passagem que o pediu (o objecto ainda existe).
//!   Morte, script tirado ou trocado entre quadros: o `sync` dá pela falta e o `exit` corre no início
//!   do tique seguinte (o objecto pode já não existir: `ph2d.get(self.id, …)` dá `nil`). Troca a meio
//!   do quadro: logo antes do `init` do novo. Stop: [`SceneScripts::stop`]. Um objecto PARADO por erro
//!   não corre o `exit`; o `clear` (a VM mudou) também não.
//! - ⚠️ **No Stop, o que o `exit` escreve NÃO pousa**: o mundo vai ser reposto, e uma escrita do Stop
//!   seria a única que sobreviveria ao rebobinar. Lê o mundo da corrida, sem portas (uma propriedade
//!   de componente é recusada), e `spawn`/`despawn`/`attach_script` são erro.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use mlua::{Function, IntoLuaMulti, Lua, Value};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_script_doc::ScriptId;

use super::this::{apply_props, born};
use super::wait::{Parked, micros, park, resume_due, run_hook};
use super::{
    Deadline, Instance, SceneReport, SceneScripts, Slot, absorb_lane, actors, apply_writes,
    chunk_in, library_id, module_in, settle_call,
};
use crate::console::speaking;
use crate::diagnostic::ScriptDiagnostic;
use crate::io::EntityWrite;
use crate::lane::{Lane, NoDoors, SceneDoors, with_lane};

/// Uma ligação de `ph2d.connect(obj, nome, fn)`: `fn(self, de)` de quem ligou corre quando
/// `emitter` emite `signal`. Mora no [`Instance`] de quem ligou.
pub(crate) struct Connection {
    pub(crate) emitter: u64,
    pub(crate) signal: String,
    pub(crate) func: Function,
}

/// Valida um `ph2d.connect` (a pista chama-o).
pub(crate) fn connection(
    speaker: Option<Entity>,
    emitter: Option<Entity>,
    signal: String,
    func: Function,
) -> mlua::Result<(Entity, Connection)> {
    let err = |m: &str| Err(mlua::Error::RuntimeError(format!("ph2d.connect: {m}")));
    let Some(by) = speaker else {
        return err("only inside a hook");
    };
    let Some(emitter) = emitter else {
        return err("the first argument is not an object");
    };
    if signal.trim().is_empty() {
        return err("the signal needs a name");
    }
    let emitter = emitter.to_bits();
    Ok((
        by,
        Connection {
            emitter,
            signal,
            func,
        },
    ))
}

/// Um vivo que saiu entre quadros (o `sync` deu pela falta): o `exit` dele corre no próximo tique.
pub(super) struct Leaving {
    pub(super) entity: Entity,
    pub(super) inst: Instance,
    /// Apanhado no `sync`, antes de o módulo poder sair com o último utilizador.
    pub(super) exit: Option<Function>,
}

/// O `exit` do script `id`, se ele o tem.
pub(super) fn exit_of(slots: &BTreeMap<ScriptId, Slot>, id: ScriptId) -> Option<Function> {
    slots
        .get(&id)
        .and_then(|s| s.module.as_ref())
        .and_then(|m| m.hook("exit"))
}

/// Uma chamada de gancho com prazo, espera e fecho. Devolve se o objecto continua a correr.
#[allow(clippy::too_many_arguments)]
pub(super) fn call(
    lua: &Lua,
    deadline: &Deadline,
    entity: Entity,
    inst: &mut Instance,
    hook: &Function,
    args: impl IntoLuaMulti,
    parked: &mut Vec<Parked>,
    writes: &mut Vec<EntityWrite>,
    report: &mut SceneReport,
) -> bool {
    report.calls += 1;
    let ran = speaking(lua, inst.script, Some(entity), || {
        deadline.within(|| run_hook(lua, hook, args))
    });
    let ran = park(ran, entity, inst.script, parked);
    settle_call(lua, entity, ran, inst, writes, report)
}

/// As ligações cujo emissor é `emitter` morrem com ele.
fn purge(instances: &mut BTreeMap<u64, Instance>, emitter: u64) {
    for i in instances.values_mut() {
        i.connections.retain(|c| c.emitter != emitter);
    }
}

/// ⭐ **O `exit`** — uma vez: depois dele o vivo não corre mais nada.
#[allow(clippy::too_many_arguments)]
pub(super) fn exit(
    lua: &Lua,
    deadline: &Deadline,
    cell: &RefCell<Lane<'_>>,
    entity: Entity,
    inst: &mut Instance,
    hook: Option<Function>,
    writes: &mut Vec<EntityWrite>,
    report: &mut SceneReport,
) {
    if !inst.runs() {
        return;
    }
    inst.gone = true;
    inst.connections.clear();
    let Some(hook) = hook else {
        return;
    };
    cell.borrow_mut().speaker = Some(entity);
    let this = inst.this.clone();
    // Uma espera no `exit` não tem a quem voltar.
    call(
        lua,
        deadline,
        entity,
        inst,
        &hook,
        this,
        &mut Vec::new(),
        writes,
        report,
    );
}

/// Os `exit` de quem saiu entre quadros, no início do tique.
pub(super) fn run_leaving(
    lua: &Lua,
    deadline: &Deadline,
    cell: &RefCell<Lane<'_>>,
    leaving: Vec<Leaving>,
    instances: &mut BTreeMap<u64, Instance>,
    writes: &mut Vec<EntityWrite>,
    report: &mut SceneReport,
) {
    for mut l in leaving {
        purge(instances, l.entity.to_bits());
        exit(
            lua,
            deadline,
            cell,
            l.entity,
            &mut l.inst,
            l.exit,
            writes,
            report,
        );
    }
}

/// ⭐ **O fecho de cada passagem**: o `exit` de quem pediu para morrer nela (também quem o `exit`
/// de outro mandou), e as ligações pedidas entram no `self` de quem as pediu.
#[allow(clippy::too_many_arguments)]
pub(super) fn close_pass(
    lua: &Lua,
    deadline: &Deadline,
    cell: &RefCell<Lane<'_>>,
    slots: &BTreeMap<ScriptId, Slot>,
    instances: &mut BTreeMap<u64, Instance>,
    writes: &mut Vec<EntityWrite>,
    report: &mut SceneReport,
) {
    for i in 0.. {
        // O empréstimo da pista acaba aqui: o `exit` volta a pedi-la.
        let next = cell.borrow().despawned().get(i).copied();
        let Some(e) = next else {
            break;
        };
        purge(instances, e.to_bits());
        if let Some(inst) = instances.get_mut(&e.to_bits()) {
            let hook = exit_of(slots, inst.script);
            exit(lua, deadline, cell, e, inst, hook, writes, report);
        }
    }
    let pending = cell.borrow_mut().take_connects();
    for (by, c) in pending {
        if let Some(inst) = instances.get_mut(&by.to_bits()).filter(|i| i.runs()) {
            inst.connections.push(c);
        }
    }
}

/// O emissor como o script o recebe: o id do objecto, ou `nil`.
fn from_value(from: Option<u64>) -> Value {
    #[expect(
        clippy::cast_precision_loss,
        reason = "os bits de uma entidade cabem em 2^53 (o `self.id` que o script recebe)"
    )]
    from.map_or(Value::Nil, |b| Value::Number(b as f64))
}

impl SceneScripts {
    /// ⭐⭐⭐ **UM PASSO FIXO**, em fases pela ordem da identidade: o `exit` de quem saiu entre
    /// quadros · nasce quem ainda não nasceu (todos os `init`) · `ready` de quem nasceu NESTE tique
    /// (já vê os outros iniciados) · por objecto, os números que o artista mexeu, `physics(self, dt)`
    /// e depois `update(self, dt)` · as esperas que chegaram · o `exit` de quem pediu para morrer ·
    /// as escritas.
    pub(crate) fn tick(
        &mut self,
        lua: &Lua,
        deadline: &Deadline,
        sim: &mut SimWorld,
        dt: f64,
        doors: &mut dyn SceneDoors,
    ) -> SceneReport {
        let mut report = SceneReport::default();
        let actors = actors(sim.world_mut(), &self.dying);
        let mut writes: Vec<EntityWrite> = Vec::new();
        let leaving = std::mem::take(&mut self.leaving);
        let Self {
            slots,
            instances,
            parked,
            dying,
            ..
        } = self;
        let lane = Lane::new(sim, doors, dying);
        let ran = with_lane(lua, lane, |cell| {
            run_leaving(
                lua,
                deadline,
                cell,
                leaving,
                instances,
                &mut writes,
                &mut report,
            );
            let mut born_now = Vec::new();
            for a in &actors {
                let Some(module) = module_in(slots, &a.cfg) else {
                    continue; // sem script, partido, por importar ou por carregar: o painel já o diz
                };
                let bits = a.entity.to_bits();
                cell.borrow_mut().speaker = Some(a.entity);
                if cell.borrow().is_dying(a.entity) {
                    continue;
                }
                // Trocou de script a meio do quadro (`ph2d.attach_script`): o `exit` do anterior, e
                // renasce já, com o `init` do novo — o `self` do anterior não é dele.
                if instances
                    .get(&bits)
                    .is_some_and(|i| Some(i.script) != library_id(&a.cfg))
                {
                    let mut old = instances.remove(&bits).expect("visto acima");
                    parked.retain(|p| p.entity != a.entity);
                    let hook = exit_of(slots, old.script);
                    exit(
                        lua,
                        deadline,
                        cell,
                        a.entity,
                        &mut old,
                        hook,
                        &mut writes,
                        &mut report,
                    );
                    cell.borrow_mut().speaker = Some(a.entity);
                }
                let Entry::Vacant(slot) = instances.entry(bits) else {
                    continue;
                };
                let chunk = chunk_in(slots, &a.cfg);
                let inst = match born(lua, a, bits, chunk.clone()) {
                    Ok(inst) => slot.insert(inst),
                    Err(e) => {
                        let d = ScriptDiagnostic::from_error(&e, &chunk);
                        if let Some(c) = crate::console::of(lua) {
                            c.diagnostic(
                                library_id(&a.cfg).unwrap_or_default(),
                                Some(a.entity),
                                &d,
                                false,
                            );
                        }
                        report.failed.push((a.entity, d));
                        continue;
                    }
                };
                apply_props(lua, inst, &module.decls, &a.cfg, &mut |n| {
                    cell.borrow_mut().object_id(n)
                });
                if let Some(init) = module.hook("init") {
                    let this = inst.this.clone();
                    let w = &mut writes;
                    if !call(
                        lua,
                        deadline,
                        a.entity,
                        inst,
                        &init,
                        this,
                        parked,
                        w,
                        &mut report,
                    ) {
                        continue;
                    }
                }
                born_now.push(a);
            }
            for a in born_now {
                let ready = module_in(slots, &a.cfg).and_then(|m| m.hook("ready"));
                let (Some(ready), Some(inst)) = (ready, instances.get_mut(&a.entity.to_bits()))
                else {
                    continue;
                };
                if !inst.runs() || cell.borrow().is_dying(a.entity) {
                    continue;
                }
                cell.borrow_mut().speaker = Some(a.entity);
                let this = inst.this.clone();
                let w = &mut writes;
                call(
                    lua,
                    deadline,
                    a.entity,
                    inst,
                    &ready,
                    this,
                    parked,
                    w,
                    &mut report,
                );
            }
            for a in &actors {
                let Some(module) = module_in(slots, &a.cfg) else {
                    continue;
                };
                let Some(inst) = instances
                    .get_mut(&a.entity.to_bits())
                    .filter(|i| Some(i.script) == library_id(&a.cfg))
                else {
                    continue;
                };
                if !inst.runs() || cell.borrow().is_dying(a.entity) {
                    continue;
                }
                cell.borrow_mut().speaker = Some(a.entity);
                apply_props(lua, inst, &module.decls, &a.cfg, &mut |n| {
                    cell.borrow_mut().object_id(n)
                });
                for hook in [module.hook("physics"), module.hook("update")]
                    .into_iter()
                    .flatten()
                {
                    if !inst.runs() || cell.borrow().is_dying(a.entity) {
                        break;
                    }
                    let this = inst.this.clone();
                    let w = &mut writes;
                    call(
                        lua,
                        deadline,
                        a.entity,
                        inst,
                        &hook,
                        (this, dt),
                        parked,
                        w,
                        &mut report,
                    );
                }
            }
            resume_due(
                lua,
                deadline,
                cell,
                instances,
                parked,
                micros(dt),
                &mut writes,
                &mut report,
            );
            close_pass(
                lua,
                deadline,
                cell,
                slots,
                instances,
                &mut writes,
                &mut report,
            );
        });
        finish_pass(lua, sim, doors, ran, &writes, report)
    }

    /// ⭐ **Os sinais deste quadro chegam a quem os ouve**: por objecto (ordem da identidade) e por
    /// sinal, `on_signal(self, nome, de)` e depois as ligações `ph2d.connect` desse emissor, pela
    /// ordem em que foram feitas. Só em quem já nasceu. `de` = quem emitiu, `nil` sem emissor.
    ///
    /// ⚠️ **A mesma pista e a mesma fila do [`Self::tick`]**: lê-se o mundo do início, e as escritas
    /// pousam depois do último gancho. Uma espera começada aqui conta a partir do tique seguinte.
    pub(crate) fn hear(
        &mut self,
        lua: &Lua,
        deadline: &Deadline,
        sim: &mut SimWorld,
        signals: &[(&str, Option<u64>)],
        doors: &mut dyn SceneDoors,
    ) -> SceneReport {
        let mut report = SceneReport::default();
        if signals.is_empty() || self.instances.is_empty() {
            return report;
        }
        let actors = actors(sim.world_mut(), &self.dying);
        let mut writes: Vec<EntityWrite> = Vec::new();
        let Self {
            slots,
            instances,
            parked,
            dying,
            ..
        } = self;
        let lane = Lane::new(sim, doors, dying);
        let ran = with_lane(lua, lane, |cell| {
            for a in &actors {
                let hook = module_in(slots, &a.cfg).and_then(|m| m.hook("on_signal"));
                let Some(inst) = instances
                    .get_mut(&a.entity.to_bits())
                    .filter(|i| Some(i.script) == library_id(&a.cfg))
                else {
                    continue;
                };
                cell.borrow_mut().speaker = Some(a.entity);
                for &(name, from) in signals {
                    let mut fns: Vec<Function> = hook.iter().cloned().collect();
                    let first = usize::from(hook.is_some());
                    fns.extend(
                        inst.connections
                            .iter()
                            .filter(|c| Some(c.emitter) == from && c.signal == name)
                            .map(|c| c.func.clone()),
                    );
                    for (k, f) in fns.iter().enumerate() {
                        if !inst.runs() || cell.borrow().is_dying(a.entity) {
                            break;
                        }
                        let this = inst.this.clone();
                        let de = from_value(from);
                        if k < first {
                            call(
                                lua,
                                deadline,
                                a.entity,
                                inst,
                                f,
                                (this, name, de),
                                parked,
                                &mut writes,
                                &mut report,
                            );
                        } else {
                            call(
                                lua,
                                deadline,
                                a.entity,
                                inst,
                                f,
                                (this, de),
                                parked,
                                &mut writes,
                                &mut report,
                            );
                        }
                    }
                }
            }
            close_pass(
                lua,
                deadline,
                cell,
                slots,
                instances,
                &mut writes,
                &mut report,
            );
        });
        finish_pass(lua, sim, doors, ran, &writes, report)
    }

    /// ⭐ **Uma vez por QUADRO desenhado**, com o jogo a correr: `process(self, dt)` em quem já
    /// nasceu, pela ordem da identidade, com o `dt` do quadro (variável). A mesma pista e fila.
    pub(crate) fn process(
        &mut self,
        lua: &Lua,
        deadline: &Deadline,
        sim: &mut SimWorld,
        dt: f64,
        doors: &mut dyn SceneDoors,
    ) -> SceneReport {
        let mut report = SceneReport::default();
        if self.instances.is_empty() {
            return report;
        }
        let actors = actors(sim.world_mut(), &self.dying);
        let mut writes: Vec<EntityWrite> = Vec::new();
        let Self {
            slots,
            instances,
            parked,
            dying,
            ..
        } = self;
        let lane = Lane::new(sim, doors, dying);
        let ran = with_lane(lua, lane, |cell| {
            for a in &actors {
                let Some(hook) = module_in(slots, &a.cfg).and_then(|m| m.hook("process")) else {
                    continue;
                };
                let Some(inst) = instances
                    .get_mut(&a.entity.to_bits())
                    .filter(|i| Some(i.script) == library_id(&a.cfg) && i.runs())
                else {
                    continue;
                };
                if cell.borrow().is_dying(a.entity) {
                    continue;
                }
                cell.borrow_mut().speaker = Some(a.entity);
                let this = inst.this.clone();
                call(
                    lua,
                    deadline,
                    a.entity,
                    inst,
                    &hook,
                    (this, dt),
                    parked,
                    &mut writes,
                    &mut report,
                );
            }
            close_pass(
                lua,
                deadline,
                cell,
                slots,
                instances,
                &mut writes,
                &mut report,
            );
        });
        finish_pass(lua, sim, doors, ran, &writes, report)
    }

    /// ⭐ **O Stop**: cada vivo (e quem saiu e ainda não correu o `exit`) recebe o `exit`, pela ordem
    /// da identidade; o que ele escreve é deitado fora; depois o [`Self::rewind`]. Devolve quantos
    /// objectos tinham vivo.
    pub(crate) fn stop(&mut self, lua: &Lua, deadline: &Deadline, sim: &mut SimWorld) -> usize {
        let mut order: Vec<(u64, u64)> = self
            .instances
            .iter()
            .map(|(&bits, i)| (i.sid, bits))
            .collect();
        order.sort_unstable();
        let leaving = std::mem::take(&mut self.leaving);
        let Self {
            slots,
            instances,
            dying,
            ..
        } = self;
        let mut doors = NoDoors;
        let lane = Lane::new(sim, &mut doors, dying).stopping();
        let mut report = SceneReport::default();
        let ran = with_lane(lua, lane, |cell| {
            let mut writes = Vec::new();
            run_leaving(
                lua,
                deadline,
                cell,
                leaving,
                instances,
                &mut writes,
                &mut report,
            );
            for (_, bits) in order {
                let (Some(e), Some(inst)) = (Entity::try_from_bits(bits), instances.get_mut(&bits))
                else {
                    continue;
                };
                let hook = exit_of(slots, inst.script);
                exit(lua, deadline, cell, e, inst, hook, &mut writes, &mut report);
            }
        });
        absorb_lane(lua, ran, &mut report);
        self.rewind()
    }
}

/// As escritas da passagem pousam (a mesma ordem do fim do `tick`).
pub(super) fn finish_pass(
    lua: &Lua,
    sim: &mut SimWorld,
    doors: &mut dyn SceneDoors,
    ran: mlua::Result<((), crate::lane::LaneOut)>,
    writes: &[EntityWrite],
    mut report: SceneReport,
) -> SceneReport {
    absorb_lane(lua, ran, &mut report);
    apply_writes(sim.world_mut(), writes, &mut report);
    crate::lane_property::apply(sim, doors, writes, &mut report);
    crate::motion_api::apply(sim.world_mut(), &report.motion);
    report.wrote.sort_by_key(|e| e.to_bits());
    report.wrote.dedup();
    report
}
