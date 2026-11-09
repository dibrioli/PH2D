//! ⭐ **Cena 13 — a Bancada** (plano 13 Onda B): «Hero» e «Hero 2» correm o MESMO `player.luau`
//! (o Hero 2 com `speed` 180 no Inspector), cinco moedas correm o `coin.luau` e emitem `coin`, e o
//! `door.luau` não é de ninguém. A aba do `player.luau` abre-se: o «In this script», as marcas, a
//! grelha «Who has what» com o mapa, e a frase do pé.

use crate::{SHAPE_R, tinta};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_script::{LuauScript, ProjectScripts, ScriptEditRequests, ScriptValue};
use ph2d_vec_entities::entities::{VecEntityMap, sync};
use ph2d_vec_scene::{VecScene, ellipse, rectangle, star};

pub const PLAYER_NAME: &str = "player.luau";
pub const PLAYER_LUAU: &str =
    include_str!("../../../docs/CodeEditor/ferramentas/smoke/bancada_player.luau");
pub const COIN_NAME: &str = "coin.luau";
pub const COIN_LUAU: &str =
    include_str!("../../../docs/CodeEditor/ferramentas/smoke/bancada_coin.luau");
pub const DOOR_NAME: &str = "door.luau";
pub const DOOR_LUAU: &str =
    include_str!("../../../docs/CodeEditor/ferramentas/smoke/bancada_door.luau");

/// Monta a cena 13; devolve `[Hero, Hero 2]`.
pub fn monta(sim: &mut SimWorld, cena: &mut VecScene, mapa: &mut VecEntityMap) -> Vec<Entity> {
    let (player, coin) = {
        let mut lib = sim.world_mut().get_resource_or_init::<ProjectScripts>();
        let ids = (
            lib.insert(PLAYER_NAME, PLAYER_LUAU),
            lib.insert(COIN_NAME, COIN_LUAU),
        );
        lib.insert(DOOR_NAME, DOOR_LUAU);
        ids
    };
    let r = SHAPE_R;
    let small = r * 0.35;
    let mut pecas = vec![
        (
            "Hero",
            -4.5,
            tinta(ellipse([0.0, 0.0], r, r), [96, 168, 235]),
            Some(player),
        ),
        (
            "Hero 2",
            -2.0,
            tinta(ellipse([0.0, 0.0], r, r), [235, 128, 96]),
            Some(player),
        ),
        (
            "Door",
            5.5,
            tinta(rectangle([-r * 0.6, -r], [r * 0.6, r]), [130, 134, 146]),
            None,
        ),
    ];
    for (i, name) in ["Coin 1", "Coin 2", "Coin 3", "Coin 4", "Coin 5"]
        .into_iter()
        .enumerate()
    {
        let coin_shape = tinta(star([0.0, 0.0], small, small, 5, 0.45), [240, 196, 72]);
        pecas.push((name, 0.5 + 0.9 * i as f64, coin_shape, Some(coin)));
    }
    let ids: Vec<_> = pecas
        .into_iter()
        .map(|(n, x, p, s)| (n, x, cena.push_path(p), s))
        .collect();
    sync(sim, cena, mapa);
    let mut heroes = Vec::new();
    for (name, x, pid, script) in ids {
        let Some(&bits) = mapa.get(&pid) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        #[allow(clippy::cast_possible_truncation)]
        let t = Transform::from_translation(ph2d_core::Vec2::new(x as f32, crate::SHAPE_Y));
        let mut em = sim.world_mut().entity_mut(e);
        em.insert((Name::new(name), t));
        if let Some(s) = script {
            let mut cfg = LuauScript::library(s);
            if name == "Hero 2" {
                cfg.own.insert("speed".into(), ScriptValue::Number(180.0));
            }
            em.insert(cfg);
        }
        if script == Some(player) {
            heroes.push(e);
        }
    }
    sim.world_mut()
        .get_resource_or_init::<ScriptEditRequests>()
        .0
        .push(player);
    heroes
}
