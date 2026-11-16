//! **As fontes da onda 2 do `Motion Input`** (ficha `08_motion_input.md` §«Desenho da onda 2»): o
//! objecto From, a velocidade, o chão, a acção, o campo, o Remap e o Smooth — cada uma pela porta
//! que já existe (sem segunda conta). A ponte e o Inspector leem daqui.

use crate::canais_componente;
use ph2d_component_desc::{FieldDesc, FieldKind, View};
use ph2d_ecs::canal_componente as cc;
use ph2d_ecs::motion_inputs::{Valor, no_tamanho};
use ph2d_ecs::scene::property_ref::RefNames;
use ph2d_ecs::scene::registry::ComponentRegistry;
use ph2d_ecs::{ActionSample, Entity, MotionInput, StableId, TextosDosCanais, World};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, RigidBody};
use std::collections::BTreeMap;

/// O que a ponte lê fora do mundo, neste quadro.
#[derive(Clone, Copy)]
pub struct Fontes<'a> {
    /// O tempo do quadro, em segundos (o Smooth).
    pub dt: f32,
    /// A física viva (Speed, Grounded); `None` = sem física.
    pub physics: Option<&'a PhysicsBridge>,
    /// As acções resolvidas deste quadro, pelo nome (Action).
    pub accoes: &'a BTreeMap<String, ActionSample>,
    /// O registo dos componentes (Property); `None` = Property não existe.
    pub registry: Option<&'a ComponentRegistry>,
}

/// O objecto From: `0` = o dono (o PAI do objeto «Motion Input», MI-10); um `StableId` que ninguém tem
/// = `None` (a fonte não existe).
#[must_use]
pub fn alvo(world: &World, e: Entity, from: u64) -> Option<Entity> {
    if from == 0 {
        Some(ph2d_ecs::motion_input::dono(world, e))
    } else {
        ph2d_ecs::entity_with_stable_id(world, StableId(from))
    }
}

/// O corpo que leva o alvo: ele mesmo, ou o corpo dono (`owner_body`).
#[must_use]
pub fn corpo(world: &World, alvo: Entity) -> Option<Entity> {
    if world.get::<RigidBody>(alvo).is_some() {
        Some(alvo)
    } else {
        ph2d_physics_ecs::owner_body(world, alvo)
    }
}

/// O corpo do alvo é um personagem de plataforma (o que tem «no chão»)?
#[must_use]
pub fn personagem(world: &World, alvo: Entity) -> Option<Entity> {
    corpo(world, alvo).filter(|&b| world.get::<PlatformPlayer>(b).is_some())
}

/// Velocity: a velocidade do corpo, em unidades do mundo por segundo — a do personagem
/// (`PlayerView::velocity`) ou a do solver (`body_velocity`).
#[must_use]
pub fn velocidade_par(
    world: &World,
    physics: Option<&PhysicsBridge>,
    alvo: Entity,
) -> Option<[f32; 2]> {
    let b = corpo(world, alvo)?;
    let p = physics?;
    p.player_view(b)
        .map(|v| v.velocity)
        .or_else(|| p.body_velocity(b))
}

/// Speed: o tamanho da [`velocidade_par`].
#[must_use]
pub fn velocidade(world: &World, physics: Option<&PhysicsBridge>, alvo: Entity) -> Option<f32> {
    velocidade_par(world, physics, alvo).map(|v| v[0].hypot(v[1]))
}

/// Move: `(direita − esquerda, cima − baixo)` pelas forças das acções `move_*` (o `axis` do jogador e
/// o `action_strength` do Luau); sem nenhuma delas no mapa = `None`.
#[must_use]
pub fn mover(accoes: &BTreeMap<String, ActionSample>) -> Option<[f32; 2]> {
    use ph2d_input::{PLAYER_MOVE_DOWN, PLAYER_MOVE_LEFT, PLAYER_MOVE_RIGHT, PLAYER_MOVE_UP};
    let quatro = [
        PLAYER_MOVE_RIGHT,
        PLAYER_MOVE_LEFT,
        PLAYER_MOVE_UP,
        PLAYER_MOVE_DOWN,
    ];
    if !quatro.iter().any(|n| accoes.contains_key(*n)) {
        return None;
    }
    let f = quatro.map(|n| accao(accoes, n).unwrap_or(0.0));
    Some([f[0] - f[1], f[2] - f[3]])
}

/// Grounded: `1` com a postura `Ground` do personagem (a lei das animações), `0` no ar.
#[must_use]
pub fn no_chao(world: &World, physics: Option<&PhysicsBridge>, alvo: Entity) -> Option<f32> {
    let v = physics?.player_view(personagem(world, alvo)?)?;
    Some(if v.footing == ph2d_physics_ecs::FootingKind::Ground {
        1.0
    } else {
        0.0
    })
}

/// Action: a força `0..1` da acção pelo nome; desconhecida = `None`.
#[must_use]
pub fn accao(accoes: &BTreeMap<String, ActionSample>, nome: &str) -> Option<f32> {
    accoes.get(nome.trim()).map(|a| a.strength)
}

/// Um campo que a fonte Property aceita: um número de UMA faixa ou um par, nunca uma referência.
fn numerico(f: &FieldDesc) -> bool {
    matches!(
        f.kind,
        FieldKind::Scalar | FieldKind::Int | FieldKind::Angle | FieldKind::Toggle | FieldKind::Vec2
    ) && !matches!(f.view, View::Named(_))
}

/// Os canais de `"<canonical>/<serde_key>"` pelo descritor: um, ou dois num par.
fn chaves(reg: &ComponentRegistry, property: &str) -> Option<Vec<ph2d_ecs::FieldKey>> {
    let (canonical, key) = property.trim().split_once('/')?;
    let d = reg.get_by_name(canonical)?.desc?;
    let f = d
        .fields
        .iter()
        .find(|f| f.serde_key() == key && numerico(f))?;
    let n = if f.kind == FieldKind::Vec2 { 2 } else { 1 };
    (0..n)
        .map(|faixa| cc::chave(d.canonical_name, f.field_id, faixa))
        .collect()
}

/// Property: o valor do campo pelo funil da Timeline/Auto Key — um número, ou o par de um campo de
/// duas faixas; ausente ou não numérico = `None`.
#[must_use]
pub fn propriedade(
    reg: Option<&ComponentRegistry>,
    world: &World,
    alvo: Entity,
    property: &str,
) -> Option<Valor> {
    let reg = reg?;
    let mut textos = TextosDosCanais::default();
    let mut ler =
        |k| canais_componente::ler(reg, world, alvo, k, &mut textos, RefNames { tags: None });
    match chaves(reg, property)?.as_slice() {
        [k] => ler(*k).map(Valor::Numero),
        [x, y] => Some(Valor::Vetor([ler(*x)?, ler(*y)?])),
        _ => None,
    }
}

/// Os campos numéricos dos componentes do alvo: `(«Health · Max», "Health/max")`, pela ordem do
/// registo e do descritor.
#[must_use]
pub fn campos_numericos(
    reg: &ComponentRegistry,
    world: &World,
    alvo: Entity,
) -> Vec<(String, String)> {
    reg.iter()
        .filter_map(|entry| Some((entry.desc?, (entry.value_of)(world, alvo).ok()??)))
        .flat_map(|(d, _)| {
            d.fields.iter().filter(|f| numerico(f)).map(move |f| {
                (
                    format!(
                        "{} \u{b7} {}",
                        ph2d_i18n::tr(d.display_key),
                        ph2d_i18n::tr(f.label_key)
                    ),
                    format!("{}/{}", d.canonical_name, f.serde_key()),
                )
            })
        })
        .collect()
}

/// Remap e curva: `clamp((v − from)/(to − from), 0, 1)` (a lei do `value.map_range`; `from == to`
/// ⇒ `0`), depois a curva `ph2d-curve` se houver. Sem a caixa, o número passa.
#[must_use]
pub fn remapeia(m: &MotionInput, v: f32) -> f32 {
    if !m.remap {
        return v;
    }
    let span = m.remap_to - m.remap_from;
    let t = if span == 0.0 {
        0.0
    } else {
        ((v - m.remap_from) / span).clamp(0.0, 1.0)
    };
    ph2d_curve::parse(&m.curve).map_or(t, |c| c.eval(t))
}

/// A forma inteira (MI-5): num número, Remap → curva → Smooth; num par, o Remap e a curva no TAMANHO
/// (a direção fica) e o Smooth por componente.
#[must_use]
pub fn forma(m: &MotionInput, antes: Option<Valor>, v: Valor, dt: f32) -> Valor {
    if let Valor::Vetor(p) = v {
        let p = no_tamanho(p, |l| remapeia(m, l));
        let a = match antes {
            Some(Valor::Vetor(a)) => Some(a),
            _ => None,
        };
        return Valor::Vetor([0, 1].map(|i| suaviza(a.map(|a| a[i]), p[i], dt, m.smooth)));
    }
    let a = match antes {
        Some(Valor::Numero(a)) => Some(a),
        _ => None,
    };
    Valor::Numero(suaviza(a, remapeia(m, v.numero()), dt, m.smooth))
}

/// Smooth: um passo da lei do `value.follow` Smooth (`v += (alvo − v)·(1 − 2^(−dt/s))`); o primeiro
/// quadro (`antes = None`) nasce no alvo; `s <= 0` = sem suavizar.
#[must_use]
pub fn suaviza(antes: Option<f32>, alvo: f32, dt: f32, s: f32) -> f32 {
    match antes {
        Some(v) if s > 0.0 => {
            ph2d_node_value_follow::passo(
                ph2d_node_value_follow::MODE_SMOOTH,
                s,
                0.0,
                v,
                0.0,
                alvo,
                dt,
            )
            .0
        }
        _ => alvo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_component_desc::{Propagation, Target};

    /// ⭐ A Property aceita um número de UMA faixa; um inteiro que é uma REFERÊNCIA (`View::Named`) não é
    /// um número que se anime, e um texto também não.
    #[test]
    fn so_numeros_que_nao_sao_referencias() {
        let campo = |kind, view| FieldDesc {
            field_id: 1,
            label_key: "",
            kind,
            policy: Propagation::Propagate,
            is_ref: None,
            name: "x",
            key: "",
            view,
        };
        assert!(numerico(&campo(FieldKind::Int, View::Same)));
        assert!(numerico(&campo(FieldKind::Scalar, View::Degrees)));
        assert!(!numerico(&campo(
            FieldKind::Int,
            View::Named(Target::Stable)
        )));
        assert!(!numerico(&campo(FieldKind::Text, View::Same)));
    }
}
