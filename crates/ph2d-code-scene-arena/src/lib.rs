//! ⭐ **Cena 15 — muitos problemas e muitos donos** (o pedido do dono de 10/11): catorze formas correm
//! o MESMO `arena.luau` (sete propriedades, de cada tipo da grelha), dez com um valor próprio (as
//! células âmbar), e o script com uns dez problemas que o lint acusa sem o `luau-lsp`, mais um erro
//! do Play (`hud.x` sobre `nil`). Aqui moram a montagem e o roteiro; [`steps`] é o roteador da
//! família com a 15 (`ph2d_code_smoke::steps` serve as outras).

use ph2d_code_scenes::{SHAPE_R, SHAPE_Y, tinta};
use ph2d_code_smoke::{Gesture, Until};
use ph2d_code_viewmodel::Aim;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_script::{LuauScript, ProjectScripts, ScriptEditRequests, ScriptValue};
use ph2d_vec_entities::entities::{VecEntityMap, sync};
use ph2d_vec_scene::{VecScene, ellipse, rectangle, regular_polygon, star};

pub const ARENA_NAME: &str = "arena.luau";
pub const ARENA_LUAU: &str =
    include_str!("../../../docs/CodeEditor/ferramentas/smoke/arena_crowd.luau");

/// Quantas formas correm o script (as linhas da grelha): mais do que o terço da coluna mostra, a
/// grelha rola.
pub const DONOS: usize = 14;

/// O valor próprio da forma `i` (`None` = segue o script).
fn proprio(i: usize) -> Option<(&'static str, ScriptValue)> {
    Some(match i {
        1 => ("speed", ScriptValue::Number(6.0)),
        2 => ("visible", ScriptValue::Bool(false)),
        3 => ("tint", ScriptValue::Color([0.2, 0.6, 1.0, 1.0])),
        4 => ("offset", ScriptValue::Vec2([2.0, -1.0])),
        5 => ("mode", ScriptValue::Text("fly".into())),
        6 => ("jump_height", ScriptValue::Number(5.0)),
        7 => ("label", ScriptValue::Text("boss".into())),
        9 => ("speed", ScriptValue::Number(9.5)),
        12 => ("visible", ScriptValue::Bool(false)),
        13 => ("mode", ScriptValue::Text("run".into())),
        _ => return None,
    })
}

/// Monta a cena 15; devolve as oito formas, pela ordem da cena.
pub fn monta(sim: &mut SimWorld, cena: &mut VecScene, mapa: &mut VecEntityMap) -> Vec<Entity> {
    let script = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert(ARENA_NAME, ARENA_LUAU);
    let r = SHAPE_R * 0.6;
    let nomes: [&str; DONOS] = [
        "Runner", "Jumper", "Ghost", "Painter", "Drifter", "Flyer", "Bouncer", "Boss", "Walker",
        "Sprinter", "Guard", "Scout", "Shadow", "Rover",
    ];
    let ids: Vec<_> = nomes
        .into_iter()
        .enumerate()
        .map(|(i, nome)| {
            let p = match i % 4 {
                0 => tinta(ellipse([0.0, 0.0], r, r), [96, 168, 235]),
                1 => tinta(star([0.0, 0.0], r, r, 5, 0.45), [240, 196, 72]),
                2 => tinta(regular_polygon([0.0, 0.0], r, r, 6), [150, 120, 230]),
                _ => tinta(rectangle([-r, -r], [r, r]), [235, 128, 96]),
            };
            (i, nome, cena.push_path(p))
        })
        .collect();
    sync(sim, cena, mapa);
    let mut out = Vec::new();
    for (i, nome, pid) in ids {
        let Some(&bits) = mapa.get(&pid) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        #[allow(clippy::cast_precision_loss)]
        let (x, y) = (-6.0 + 1.7 * (i % 7) as f32, SHAPE_Y + 1.5 * (i / 7) as f32);
        let t = Transform::from_translation(ph2d_core::Vec2::new(x, y));
        let mut cfg = LuauScript::library(script);
        if let Some((k, v)) = proprio(i) {
            cfg.own.insert(k.into(), v);
        }
        sim.world_mut()
            .entity_mut(e)
            .insert((Name::new(nome), t, cfg));
        out.push(e);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    sim.world_mut()
        .get_resource_or_init::<ScriptEditRequests>()
        .0
        .push(script);
    out
}

/// ⭐ **O roteiro da cena 15**: 1 = a aba com os Problems à frente e a grelha; 2 = o rato numa
/// célula âmbar (a do `speed` do Jumper): a frase do pé diz o que é; 3 = a roda sobre a grelha (a
/// mão do dono; nas fotos, `RODA` do `cena14/fotos_q3.sh`) leva ao fim.
pub const STEPS_15: &[&[Gesture]] = &[
    &[
        Gesture::Wait(Until::Script),
        Gesture::Wait(Until::Problems(8)),
        Gesture::ClickBandTab(1),
        Gesture::Frame,
    ],
    &[Gesture::PointAim(Aim::Cell(1, 0)), Gesture::Frame],
    &[Gesture::Frame],
];

/// O roteiro da cena `level` (`None` = a cena não tem fotos conduzidas).
#[must_use]
pub fn steps(level: u32) -> Option<&'static [&'static [Gesture]]> {
    if level == 15 {
        return Some(STEPS_15);
    }
    ph2d_code_smoke::steps(level)
}

#[cfg(test)]
mod tests {
    #[test]
    fn o_roteador_serve_a_15_daqui_e_as_outras_da_code_smoke() {
        assert_eq!(super::steps(15), Some(super::STEPS_15));
        assert_eq!(super::steps(13), ph2d_code_smoke::steps(13));
        assert!(super::steps(13).is_some());
        assert_eq!(super::steps(16), None);
    }
}
