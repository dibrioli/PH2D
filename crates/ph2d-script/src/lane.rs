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
use crate::io::{EntityWrite, WriteQueue, WriteValue, pose_field};
use crate::lane_property::{from_lua, to_lua};
use ph2d_component_desc::FieldKind;

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

    /// ⭐ `ph2d.get(e, "health.max")` — uma propriedade nomeada (doc CodeEditor 08), com o tipo do
    /// campo para o valor chegar ao Luau na forma certa.
    ///
    /// # Errors
    /// Porque não se lê, em texto para o script.
    fn property_get(
        &mut self,
        _sim: &mut SimWorld,
        _entity: Entity,
        _name: &str,
    ) -> Result<(serde_json::Value, FieldKind), String> {
        Err(NO_PROPERTIES.into())
    }

    /// `ph2d.set(e, "health.max", v)` — `dry` só verifica (na chamada); sem ele, escreve (no fim
    /// do tique).
    ///
    /// # Errors
    /// Porque não se escreve, em texto para o script.
    fn property_set(
        &mut self,
        _sim: &mut SimWorld,
        _entity: Entity,
        _name: &str,
        _value: &serde_json::Value,
        _dry: bool,
    ) -> Result<(), String> {
        Err(NO_PROPERTIES.into())
    }

    /// `ph2d.damage`/`ph2d.heal` — um pedido à VIDA, que a corrida aplica no próximo tique.
    ///
    /// # Errors
    /// Porque não se pede, em texto para o script.
    fn health_request(
        &mut self,
        _sim: &mut SimWorld,
        _entity: Entity,
        _request: HealthRequest,
        _dry: bool,
    ) -> Result<(), String> {
        Err(NO_PROPERTIES.into())
    }

    /// `ph2d.properties(e)` — os nomes que o objecto tem, na ordem do Inspector.
    fn property_names(&mut self, _sim: &mut SimWorld, _entity: Entity) -> Vec<String> {
        Vec::new()
    }
}

const NO_PROPERTIES: &str = "this scene cannot reach component properties";

/// O que `ph2d.damage`/`ph2d.heal` pedem (pontos finitos `> 0`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HealthRequest {
    Damage(f64),
    Heal(f64),
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

/// A ordem da lista de objectos — a mesma porta que o painel lê ([`ph2d_ecs::list_order`]).
fn preorder(world: &mut World) -> Vec<Entity> {
    ph2d_ecs::list_order(world)
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
            return Err(crate::io::unknown_field("ph2d.get", field));
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

    /// `ph2d.get` — a pose, ou uma propriedade nomeada; `nil` se o id já não é de ninguém.
    fn get(&mut self, lua: &Lua, id: f64, field: &str) -> mlua::Result<Value> {
        if pose_field(field).is_some() || !is_property_name(field) {
            let v = self.pose(id, field).map_err(mlua::Error::RuntimeError)?;
            return Ok(v.map_or(Value::Nil, Value::Number));
        }
        let Some(e) = self.alive(id) else {
            return Ok(Value::Nil);
        };
        let (v, kind) = self
            .doors
            .property_get(self.sim, e, field)
            .map_err(|why| mlua::Error::RuntimeError(format!("ph2d.get: {why}")))?;
        to_lua(lua, &v, kind)
    }

    /// `ph2d.set` de uma propriedade: verifica JÁ (o erro aponta a linha) e pousa no fim do tique.
    fn set_property(&mut self, lua: &Lua, id: f64, field: &str, value: &Value) -> mlua::Result<()> {
        if !is_property_name(field) {
            return Err(mlua::Error::RuntimeError(crate::io::unknown_field(
                "ph2d.set", field,
            )));
        }
        let Some(e) = self.alive(id) else {
            return Ok(());
        };
        let v =
            from_lua(value).map_err(|why| mlua::Error::RuntimeError(format!("ph2d.set: {why}")))?;
        self.doors
            .property_set(self.sim, e, field, &v, true)
            .map_err(|why| mlua::Error::RuntimeError(format!("ph2d.set: {why}")))?;
        push(lua, e, field, WriteValue::Property(v))
    }

    /// `ph2d.properties` — a pose primeiro, depois os nomes dos componentes que ele tem.
    fn properties(&mut self, id: f64) -> Vec<String> {
        let Some(e) = self.alive(id) else {
            return Vec::new();
        };
        let mut out: Vec<String> = if self.sim.world().get::<Transform>(e).is_some() {
            crate::io::POSE_FIELDS
                .iter()
                .map(|f| (*f).to_owned())
                .collect()
        } else {
            Vec::new()
        };
        out.extend(self.doors.property_names(self.sim, e));
        out
    }

    /// `ph2d.damage`/`ph2d.heal`.
    fn health(&mut self, lua: &Lua, verb: &str, id: f64, amount: f64) -> mlua::Result<()> {
        if !(amount.is_finite() && amount > 0.0) {
            return Err(mlua::Error::RuntimeError(format!(
                "ph2d.{verb}: the amount must be a number above 0, not {amount}"
            )));
        }
        let Some(e) = self.alive(id) else {
            return Ok(());
        };
        let request = if verb == "heal" {
            HealthRequest::Heal(amount)
        } else {
            HealthRequest::Damage(amount)
        };
        self.doors
            .health_request(self.sim, e, request, true)
            .map_err(|why| mlua::Error::RuntimeError(format!("ph2d.{verb}: {why}")))?;
        push(lua, e, verb, WriteValue::Health(request))
    }

    fn alive(&self, id: f64) -> Option<Entity> {
        entity_of(id).filter(|&e| self.sim.world().get_entity(e).is_ok())
    }

    fn label(&self, e: Entity) -> String {
        self.sim
            .world()
            .get::<Name>(e)
            .map_or_else(|| format!("{}", e.to_bits()), |n| n.as_str().to_owned())
    }
}

/// Uma escrita para a fila do fim do tique (HR-8), com o nome de quem a pediu no erro.
fn push(lua: &Lua, e: Entity, field: &str, value: WriteValue) -> mlua::Result<()> {
    let q = lua
        .app_data_ref::<WriteQueue>()
        .ok_or_else(|| mlua::Error::RuntimeError("ph2d.set: WriteQueue not registered".into()))?;
    q.push(EntityWrite {
        entity: e.to_bits(),
        field: field.to_owned(),
        value,
    })
    .map_err(|err| mlua::Error::RuntimeError(format!("ph2d.{field}: {err}")))
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
        let get = scope.create_function(|lua, (id, field): (f64, String)| {
            cell.borrow_mut().get(lua, id, &field)
        })?;
        let set_property = scope.create_function(|lua, (id, field, v): (f64, String, Value)| {
            cell.borrow_mut().set_property(lua, id, &field, &v)
        })?;
        let properties =
            scope.create_function(|_, id: f64| Ok(cell.borrow_mut().properties(id)))?;
        let damage = scope.create_function(|lua, (id, n): (f64, f64)| {
            cell.borrow_mut().health(lua, "damage", id, n)
        })?;
        let heal = scope.create_function(|lua, (id, n): (f64, f64)| {
            cell.borrow_mut().health(lua, "heal", id, n)
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
            ("set_property", set_property),
            ("properties", properties),
            ("damage", damage),
            ("heal", heal),
        ] {
            table.raw_set(k, v)?;
        }
        let r = f(&cell);
        for k in [
            "get",
            "find_by_name",
            "object",
            "spawn",
            "despawn",
            "attach_script",
            "set_property",
            "properties",
            "damage",
            "heal",
        ] {
            table.raw_set(k, Value::Nil)?;
        }
        Ok(r)
    })?;
    Ok((r, cell.into_inner().finish()))
}

/// Um nome de propriedade: `componente.campo`, ou o curto de um campo que se chama como o
/// componente (`gravity_scale`). O resto é a pose (ou um nome que não existe).
fn is_property_name(field: &str) -> bool {
    static CURTOS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    field.contains('.')
        || CURTOS
            .get_or_init(|| {
                crate::api_table::property_names()
                    .into_iter()
                    .filter(|n| !n.contains('.'))
                    .collect()
            })
            .iter()
            .any(|n| n == field)
}

/// As funções `ph2d.*` que passam pela pista — Luau, para o erro apontar a linha do script.
pub(crate) const LANE_LUAU: &str = r#"
local lane, POSE = ...
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
ph2d.properties = through("properties")
ph2d.damage = through("damage")
ph2d.heal = through("heal")
-- A pose vai pela fila do `ph2d.set` de sempre; qualquer outro nome (`componente.campo`, ou o curto
-- `gravity_scale`) pela pista.
local set_pose = ph2d.set
local set_property = through("set_property")
ph2d.set = function(entity, field, value)
  if type(field) == "string" and not POSE[field] then
    return set_property(entity, field, value)
  end
  return set_pose(entity, field, value)
end
"#;

/// Instala as funções da pista (antes da sandbox).
pub(crate) fn wire(lua: &Lua) -> mlua::Result<()> {
    let table = lua.create_table()?;
    lua.set_named_registry_value(LANE_KEY, table.clone())?;
    let pose = lua.create_table()?;
    for f in crate::io::POSE_FIELDS {
        pose.raw_set(f, true)?;
    }
    lua.load(LANE_LUAU)
        .set_name("=ph2d")
        .call::<()>((table, pose))
}
