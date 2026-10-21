//! ⭐⭐⭐ **A PISTA DO MUNDO** — o `&mut World` emprestado à VM durante os ganchos de um tique
//! (`docs/CodeEditor/05_api_que_funciona.md` §3.0).
//!
//! As funções `ph2d.get` · `find_by_name` · `object` · `spawn` · `despawn` · `attach_script` são de Luau e chamam
//! a função da pista guardada numa tabela que só o Rust preenche — e só dentro de [`with_lane`], cujo
//! `Lua::scope` garante que nada do que ela empresta sobrevive ao tique. Fora dele a tabela está
//! vazia e a função diz porquê.
//!
//! ⚠️ **O HR-8 continua de pé**: as escritas (`ph2d.set`) vão para a fila e só pousam no fim do tique,
//! logo ler o mundo DURANTE o tique é ler a fotografia do início — para todo objecto, sem a copiar.
//! A única coisa que muda o mundo a meio é criar (imediato, como no Godot).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use mlua::{Lua, Table, Value};
use ph2d_ecs::{ChildOf, Entity, MasterRoot, Name, SimWorld, Spawned, StableId, Transform, World};

use crate::component::{LuauScript, ProjectScripts};
use crate::io::pose_field;

/// **O que só quem conhece mestres e documentos sabe fazer** — a porta da `Factory`
/// (`ph2d-app-components`). O `ph2d-script` não conhece instâncias nem o documento do vetor.
pub trait SceneDoors {
    /// Uma cópia TRANSIENTE do mestre cuja raiz é `master` (com [`Spawned`]), posta por `by`.
    ///
    /// # Errors
    /// Porque não nasceu, em texto para o script.
    fn instantiate(
        &mut self,
        sim: &mut SimWorld,
        master: Entity,
        by: Entity,
    ) -> Result<Entity, String>;

    /// O tique do passo fixo — o `born_tick` de quem nasce.
    fn tick(&self) -> u64;

    /// `ph2d.attach_script` acabou de pôr um script em `entity` — quem conhece instâncias regista-o
    /// como DELA (senão o sync mestre → instância, sem eco do mestre, apaga-o no passe seguinte).
    fn attached(&mut self, _sim: &mut SimWorld, _entity: Entity) {}
}

/// Sem portas: um objecto vazio nasce, uma cópia de mestre não.
pub struct NoDoors;

impl SceneDoors for NoDoors {
    fn instantiate(&mut self, _: &mut SimWorld, _: Entity, _: Entity) -> Result<Entity, String> {
        Err("this scene cannot copy masters".into())
    }

    fn tick(&self) -> u64 {
        0
    }
}

/// A tabela (no registo da VM) onde a pista põe as suas funções.
pub(crate) const LANE_KEY: &str = "ph2d.lane";

/// Como uma função da pista diz não: um ERRO pára o objecto (uma gralha); uma RECUSA é dita no
/// console e ele continua (a lei do verbo `Destroy`: o documento não é da corrida).
pub(crate) enum Refusal {
    Error(String),
    Warn(String),
}

/// O que um tique pediu que só a shell faz: as mortes.
#[derive(Default)]
pub(crate) struct LaneOut {
    pub(crate) despawned: Vec<Entity>,
}

pub(crate) struct Lane<'w> {
    sim: &'w mut SimWorld,
    doors: &'w mut dyn SceneDoors,
    /// Quem pediu para morrer neste quadro (continua achável até ao fim dele — Godot).
    dying: &'w mut BTreeSet<u64>,
    out: LaneOut,
    /// Nome → primeiro objecto em pré-ordem; refeito depois de cada nascimento.
    names: Option<BTreeMap<String, Entity>>,
    /// Quem fala agora (o `by` de quem nasce).
    pub(crate) speaker: Option<Entity>,
}

fn entity_of(id: f64) -> Option<Entity> {
    if !(id.is_finite() && id >= 0.0) {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "os bits de uma entidade cabem em 2^53 (o `self.id` que o script recebe)"
    )]
    Entity::try_from_bits(id as u64)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "os bits de uma entidade cabem em 2^53 enquanto a geração for < 2^21"
)]
fn id_of(e: Entity) -> f64 {
    e.to_bits() as f64
}

/// Nasceu nesta corrida — ele, ou a raiz da cópia de que é peça (o `Spawned` só marca a raiz).
fn born_in_run(world: &World, e: Entity) -> bool {
    let mut at = Some(e);
    while let Some(x) = at {
        if ph2d_ecs::is_transient(world, x) {
            return true;
        }
        at = world.get::<ChildOf>(x).map(ChildOf::parent);
    }
    false
}

/// Um mestre (a raiz ou uma peça) é receita, não objecto da cena.
fn is_recipe(world: &World, e: Entity) -> bool {
    world.get::<MasterRoot>(e).is_some() || ph2d_ecs::is_master_piece(world, e)
}

/// ⭐ **A ordem da lista de objectos** (raízes por `root_key`, filhos por `sibling_key`), com as
/// cópias da corrida no fim — a mesma porta que o painel lê.
fn preorder(world: &mut World) -> Vec<Entity> {
    let mut roots: Vec<Entity> = world
        .query_filtered::<Entity, (
            bevy_ecs::query::With<Transform>,
            bevy_ecs::query::Without<ChildOf>,
        )>()
        .iter(world)
        .collect();
    roots.sort_unstable_by_key(|&e| ph2d_ecs::root_key(world, e));
    let mut out = Vec::with_capacity(roots.len());
    let mut stack: Vec<Entity> = roots.into_iter().rev().collect();
    while let Some(e) = stack.pop() {
        out.push(e);
        let kids = ph2d_ecs::ordered_children(world, e);
        stack.extend(kids.into_iter().rev());
    }
    out
}

impl<'w> Lane<'w> {
    pub(crate) fn new(
        sim: &'w mut SimWorld,
        doors: &'w mut dyn SceneDoors,
        dying: &'w mut BTreeSet<u64>,
    ) -> Self {
        Self {
            sim,
            doors,
            dying,
            out: LaneOut::default(),
            names: None,
            speaker: None,
        }
    }

    pub(crate) fn finish(self) -> LaneOut {
        self.out
    }

    /// Pediu para morrer neste quadro?
    pub(crate) fn is_dying(&self, e: Entity) -> bool {
        self.dying.contains(&e.to_bits())
    }

    /// `ph2d.get` — um campo da pose de QUALQUER objecto; `nil` se o id não é de um objecto com pose.
    fn pose(&self, id: f64, field: &str) -> Result<Option<f64>, String> {
        let Some(i) = pose_field(field) else {
            return Err(format!(
                "ph2d.get: unknown field '{field}' (x, y, rotation, scale_x, scale_y)"
            ));
        };
        let Some(t) = entity_of(id).and_then(|e| self.sim.world().get::<Transform>(e)) else {
            return Ok(None);
        };
        Ok(Some(crate::scene::pose_of(t)[i]))
    }

    /// `ph2d.find_by_name` — o primeiro objecto (não-receita) com esse nome, pela ordem da lista.
    fn find(&mut self, name: &str) -> Option<Entity> {
        if self.names.is_none() {
            let mut map = BTreeMap::new();
            for e in preorder(self.sim.world_mut()) {
                let world = self.sim.world();
                if is_recipe(world, e) {
                    continue;
                }
                if let Some(n) = world.get::<Name>(e) {
                    map.entry(n.as_str().to_owned()).or_insert(e);
                }
            }
            self.names = Some(map);
        }
        self.names.as_ref().and_then(|m| m.get(name).copied())
    }

    /// O mestre com esse nome (a raiz), pela ordem da lista.
    fn master_named(&mut self, name: &str) -> Option<Entity> {
        let order = preorder(self.sim.world_mut());
        let world = self.sim.world();
        order.into_iter().find(|&e| {
            world.get::<MasterRoot>(e).is_some()
                && world.get::<Name>(e).is_some_and(|n| n.as_str() == name)
        })
    }

    /// `ph2d.spawn` — imediato e TRANSIENTE.
    fn spawn(&mut self, master: Option<&str>, at: Option<[f64; 2]>) -> Result<Entity, String> {
        let Some(by) = self.speaker else {
            return Err("ph2d.spawn: only inside a hook".into());
        };
        let e = match master {
            Some(name) => {
                let Some(m) = self.master_named(name) else {
                    return Err(format!("ph2d.spawn: no master named '{name}'"));
                };
                self.doors
                    .instantiate(self.sim, m, by)
                    .map_err(|why| format!("ph2d.spawn('{name}'): {why}"))?
            }
            None => {
                let sid = self.sim.world().get::<StableId>(by).map_or(0, |s| s.0);
                let born_tick = self.doors.tick();
                self.sim
                    .world_mut()
                    .spawn((
                        Transform::IDENTITY,
                        Name::new("Object"),
                        Spawned { by: sid, born_tick },
                    ))
                    .id()
            }
        };
        if let Some([x, y]) = at
            && let Some(mut t) = self.sim.world_mut().get_mut::<Transform>(e)
        {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "o `Transform` da casa é f32"
            )]
            {
                t.translation.x = x as f32;
                t.translation.y = y as f32;
            }
        }
        self.names = None;
        Ok(e)
    }

    /// `ph2d.despawn` — `Ok(true)` pedido; `Ok(false)` o id já não é de ninguém.
    fn despawn(&mut self, id: f64) -> Result<bool, Refusal> {
        let Some(e) = entity_of(id).filter(|&e| self.sim.world().get_entity(e).is_ok()) else {
            return Ok(false);
        };
        if !born_in_run(self.sim.world(), e) {
            return Err(Refusal::Warn(format!(
                "ph2d.despawn: '{}' is part of the scene the artist made — only objects created while \
                 the scene runs can be removed",
                self.label(e)
            )));
        }
        if self.dying.insert(e.to_bits()) {
            self.out.despawned.push(e);
        }
        Ok(true)
    }

    /// `ph2d.attach_script` — só num objecto da corrida; o `init` dele corre no tique seguinte.
    fn attach(&mut self, id: f64, script: &str) -> Result<bool, Refusal> {
        let Some(sid) = self
            .sim
            .world()
            .get_resource::<ProjectScripts>()
            .and_then(|lib| lib.by_name(script))
        else {
            return Err(Refusal::Error(format!(
                "ph2d.attach_script: no project script named '{script}'"
            )));
        };
        let Some(e) = entity_of(id).filter(|&e| self.sim.world().get_entity(e).is_ok()) else {
            return Ok(false);
        };
        if !born_in_run(self.sim.world(), e) {
            return Err(Refusal::Warn(format!(
                "ph2d.attach_script: '{}' is part of the scene the artist made — a script can only be \
                 attached to objects created while the scene runs",
                self.label(e)
            )));
        }
        self.sim
            .world_mut()
            .entity_mut(e)
            .insert(LuauScript::library(sid));
        self.doors.attached(self.sim, e);
        self.names = None;
        Ok(true)
    }

    fn label(&self, e: Entity) -> String {
        self.sim
            .world()
            .get::<Name>(e)
            .map_or_else(|| format!("{}", e.to_bits()), |n| n.as_str().to_owned())
    }
}

/// Um «não» da pista: o erro sobe; a recusa vai ao console como aviso e a função devolve `false`.
fn refused(lua: &Lua, r: Result<bool, Refusal>) -> mlua::Result<bool> {
    match r {
        Ok(b) => Ok(b),
        Err(Refusal::Error(msg)) => Err(mlua::Error::RuntimeError(msg)),
        Err(Refusal::Warn(msg)) => {
            if let Some(c) = crate::console::of(lua) {
                c.warning(lua, &msg);
            }
            Ok(false)
        }
    }
}

/// ⭐ **Corre `f` com a pista aberta** e devolve o que ela pediu à shell.
pub(crate) fn with_lane<R>(
    lua: &Lua,
    lane: Lane<'_>,
    f: impl FnOnce(&RefCell<Lane<'_>>) -> R,
) -> mlua::Result<(R, LaneOut)> {
    let table: Table = lua.named_registry_value(LANE_KEY)?;
    let cell = RefCell::new(lane);
    let r = lua.scope(|scope| {
        let get = scope.create_function(|_, (id, field): (f64, String)| {
            cell.borrow()
                .pose(id, &field)
                .map_err(mlua::Error::RuntimeError)
        })?;
        let find = scope
            .create_function(|_, name: String| Ok(cell.borrow_mut().find(&name).map(id_of)))?;
        let object = scope.create_function(|_, name: String| {
            cell.borrow_mut().find(&name).map(id_of).ok_or_else(|| {
                mlua::Error::RuntimeError(format!("ph2d.object: no object is named \"{name}\""))
            })
        })?;
        let spawn = scope.create_function(
            |_, (master, x, y): (Option<String>, Option<f64>, Option<f64>)| {
                let at = match (x, y) {
                    (Some(x), Some(y)) => Some([x, y]),
                    (None, None) => None,
                    _ => {
                        return Err(mlua::Error::RuntimeError(
                            "ph2d.spawn: give both x and y, or neither".into(),
                        ));
                    }
                };
                cell.borrow_mut()
                    .spawn(master.as_deref(), at)
                    .map(id_of)
                    .map_err(mlua::Error::RuntimeError)
            },
        )?;
        let despawn = scope.create_function(|lua, id: f64| {
            let r = cell.borrow_mut().despawn(id);
            refused(lua, r)
        })?;
        let attach = scope.create_function(|lua, (id, script): (f64, String)| {
            let r = cell.borrow_mut().attach(id, &script);
            refused(lua, r)
        })?;
        for (k, v) in [
            ("get", get),
            ("find_by_name", find),
            ("object", object),
            ("spawn", spawn),
            ("despawn", despawn),
            ("attach_script", attach),
        ] {
            table.raw_set(k, v)?;
        }
        let r = f(&cell);
        for k in ["get", "find_by_name", "object", "spawn", "despawn", "attach_script"] {
            table.raw_set(k, Value::Nil)?;
        }
        Ok(r)
    })?;
    Ok((r, cell.into_inner().finish()))
}

/// As funções `ph2d.*` que passam pela pista — Luau, para o erro apontar a linha do script.
pub(crate) const LANE_LUAU: &str = r#"
local lane = ...
local function through(name)
  return function(...)
    local f = lane[name]
    if f == nil then
      error("ph2d." .. name .. ": only while the scene runs (inside init, update or on_signal)", 2)
    end
    return f(...)
  end
end
ph2d.get = through("get")
ph2d.find_by_name = through("find_by_name")
ph2d.object = through("object")
ph2d.spawn = through("spawn")
ph2d.despawn = through("despawn")
ph2d.attach_script = through("attach_script")
"#;

/// Instala as funções da pista (antes da sandbox).
pub(crate) fn wire(lua: &Lua) -> mlua::Result<()> {
    let table = lua.create_table()?;
    lua.set_named_registry_value(LANE_KEY, table.clone())?;
    lua.load(LANE_LUAU).set_name("=ph2d").call::<()>(table)
}
