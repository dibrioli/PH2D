//! ⭐ **O raio no script** (GEF 07, RS-7 e RS-8, dono 15/11) — `ph2d.ray(id)` lê o que um objecto «Ray
//! Sensor» vê AGORA, e `ph2d.raycast(de, até, filtro)` dá um tiro pontual com a resposta na hora
//! (hitscan, o `intersect_ray` do Godot). A lei dos dois é a da física (`ph2d-physics-ecs`, o `lanca`
//! do raio): o `ph2d-script` só traduz para o Luau, pela porta [`super::SceneDoors`].
//!
//! E as portas sem cena ([`NoDoors`]) e o pedido à vida ([`HealthRequest`]), que saíram da `lane.rs`
//! pelo tecto de 700 linhas.

use mlua::{Lua, Table, Value};
use ph2d_ecs::{Entity, SimWorld};

use super::{Lane, SceneDoors, entity_of, id_of};
use crate::props::ScriptValue;
use crate::valores::tabela_de;

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

/// **Um acerto**, como o Luau o lê (`RayHit` no `ph2d.d.luau`).
#[derive(Clone, Debug, PartialEq)]
pub struct RayHitView {
    /// Quem foi visto (o objecto com o corpo).
    pub object: Entity,
    /// A FORMA acertada — a «Collision Shape» filha, ou o próprio objecto.
    pub shape: Entity,
    /// Onde, em mundo.
    pub point: [f64; 2],
    /// A normal da superfície ali.
    pub normal: [f64; 2],
    /// A que distância da origem.
    pub distance: f64,
    /// As tags da FORMA (o «material» do acerto, RS-4): os caminhos («Ground/Ice»).
    pub tags: Vec<String>,
    /// O atrito e o quique da forma (`None` sem `Collider`).
    pub friction: Option<f64>,
    pub bounce: Option<f64>,
}

/// **O que um Ray Sensor vê AGORA** (`RayView`): `seeing` e os acertos, do mais perto ao mais longe
/// (um só sem «See all hits» nem «Cone»).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RayView {
    pub seeing: bool,
    pub hits: Vec<RayHitView>,
}

const NO_PHYSICS: &str = "this scene has no physics to cast rays in";

/// As duas portas do raio na [`SceneDoors`] — por omissão, a cena não tem física.
pub(crate) fn sem_fisica<T>() -> Result<T, String> {
    Err(NO_PHYSICS.into())
}

fn vec2(lua: &Lua, p: [f64; 2]) -> mlua::Result<Value> {
    Ok(tabela_de(lua, &ScriptValue::Vec2(p))?.unwrap_or(Value::Nil))
}

fn acerto(lua: &Lua, h: &RayHitView) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.raw_set("object", id_of(h.object))?;
    t.raw_set("shape", id_of(h.shape))?;
    t.raw_set("point", vec2(lua, h.point)?)?;
    t.raw_set("normal", vec2(lua, h.normal)?)?;
    t.raw_set("distance", h.distance)?;
    t.raw_set(
        "tags",
        lua.create_sequence_from(h.tags.iter().map(String::as_str))?,
    )?;
    t.raw_set("friction", h.friction)?;
    t.raw_set("bounce", h.bounce)?;
    Ok(t)
}

fn acertos(lua: &Lua, hits: &[RayHitView]) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for h in hits {
        t.raw_push(acerto(lua, h)?)?;
    }
    Ok(t)
}

/// Um ponto do script: um `Vec2` (`ph2d.vec2`) ou `{x, y}`.
fn ponto(v: &Value, qual: &str) -> mlua::Result<[f64; 2]> {
    if let Value::Table(t) = v
        && let (Ok(x), Ok(y)) = (t.get::<f64>("x"), t.get::<f64>("y"))
        && x.is_finite()
        && y.is_finite()
    {
        return Ok([x, y]);
    }
    Err(mlua::Error::RuntimeError(format!(
        "ph2d.raycast: `{qual}` must be a position, such as ph2d.vec2(0, 0)"
    )))
}

impl Lane<'_> {
    /// `ph2d.ray(id)` — `nil` para quem não é um Ray Sensor; senão `{ seeing, object, point, normal,
    /// distance, tags, friction, bounce, hits }` (os campos de cima são os do mais perto).
    pub(super) fn ray(&mut self, lua: &Lua, id: f64) -> mlua::Result<Value> {
        let Some(e) = entity_of(id).filter(|&e| self.sim.world().get_entity(e).is_ok()) else {
            return Ok(Value::Nil);
        };
        let Some(v) = self
            .doors
            .ray_view(self.sim, e)
            .map_err(mlua::Error::RuntimeError)?
        else {
            return Ok(Value::Nil);
        };
        let t = match v.hits.first() {
            Some(h) => acerto(lua, h)?,
            None => lua.create_table()?,
        };
        t.raw_set("seeing", v.seeing)?;
        t.raw_set("hits", acertos(lua, &v.hits)?)?;
        Ok(Value::Table(t))
    }

    /// `ph2d.raycast(de, até, filtro)` — a lista dos acertos, do mais perto ao mais longe (vazia = nada).
    pub(super) fn raycast(
        &mut self,
        lua: &Lua,
        (de, ate, filtro): (Value, Value, Option<Table>),
    ) -> mlua::Result<Table> {
        let (de, ate) = (ponto(&de, "from")?, ponto(&ate, "to")?);
        let filtro = match filtro {
            Some(t) => match crate::lane_property::from_lua(&Value::Table(t)) {
                Ok(serde_json::Value::Object(o)) => o,
                Ok(serde_json::Value::Array(a)) if a.is_empty() => serde_json::Map::new(),
                Ok(_) => {
                    return Err(mlua::Error::RuntimeError(
                        "ph2d.raycast: the filter is a table of names, such as { only_see = \"only_tag\", tag = \"Enemy\" }".into(),
                    ));
                }
                Err(e) => return Err(mlua::Error::RuntimeError(format!("ph2d.raycast: {e}"))),
            },
            None => serde_json::Map::new(),
        };
        let hits = self
            .doors
            .raycast(self.sim, self.speaker, (de, ate), &filtro)
            .map_err(|e| mlua::Error::RuntimeError(format!("ph2d.raycast: {e}")))?;
        acertos(lua, &hits)
    }
}
