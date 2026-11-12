//! A foto `fisica_player_pulo_terreno` (ficha GEF 04, 12/11) mostra o que promete: na borda da «Platform Low»,
//! uma curva desce ao chão de baixo e outra pousa na «Platform High».

use super::*;
use ph2d_physics_ecs::{PhysicsSettings, PlatformPlayer, arc_world};

/// Os blocos do nível 2 que a foto enquadra (`ph2d_app_game::smoke::BLOCOS_2`: a crate do jogo está acima desta).
const BLOCOS: [(&str, [f32; 2], [f32; 2]); 4] = [
    ("Ground Left", [-5.0, -0.5], [7.0, 0.5]),
    ("Ground Right", [11.0, -0.5], [6.0, 0.5]),
    ("Platform Low", [3.5, 0.85], [1.2, 0.25]),
    ("Platform High", [8.5, 2.0], [1.5, 0.25]),
];
const ALTA_TOPO: f32 = 2.25;
const ALTA_X: [f32; 2] = [7.0, 10.0];

fn nivel() -> SimWorld {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    for (nome, c, h) in BLOCOS {
        w.spawn((
            Transform::from_translation(Vec2::new(c[0], c[1])),
            Name::new(nome),
            Sprite::atlas(WHITE_TILE_KEY, [2.0 * h[0], 2.0 * h[1]], [1.0; 4]),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: h[0],
                    half_y: h[1],
                },
                ..Collider::default()
            },
        ));
    }
    w.spawn((
        Transform::from_translation(Vec2::new(-8.0, 1.5)),
        Name::new("Player"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.15,
                radius: 0.2,
            },
            ..Collider::default()
        },
        PlatformPlayer {
            float_height: 0.45,
            ..PlatformPlayer::default()
        },
    ));
    sim
}

/// Onde cada curva toca o terreno, em mundo, e o pé do arranque.
struct Toques {
    chao: [f32; 2],
    full: Option<[f32; 2]>,
    short: Option<[f32; 2]>,
    standing: Option<[f32; 2]>,
}

fn toques(sim: &mut SimWorld, caso: CasoFisica) -> Toques {
    let bits = prepara(sim, caso, [0.0, 0.0]).expect("o Player");
    let e = Entity::from_bits(bits);
    let w = sim.world();
    let p = w.get::<PlatformPlayer>(e).expect("player");
    let t = w.get::<Transform>(e).expect("pose");
    assert!(p.show_jump_arc && p.air_jumps == 1, "a curva ligada");
    let s = PhysicsSettings::default();
    let mundo = arc_world(&s, 1.0);
    let up = ph2d_physics_ecs::jump_arc::up_of_gravity(mundo.gravity);
    let chao = [
        t.translation.x - up[0] * p.float_height,
        t.translation.y - up[1] * p.float_height,
    ];
    let mut a = p.jump_arcs(mundo);
    ph2d_physics_ecs::arc_terrain::cut_on_terrain(w, &s, e, chao, up, &mut a);
    let m =
        |a: &ph2d_physics_ecs::jump_arc::JumpArc| a.touch.map(|t| [chao[0] + t[0], chao[1] + t[1]]);
    Toques {
        chao,
        full: m(&a.full),
        short: m(&a.short),
        standing: m(&a.standing),
    }
}

#[test]
fn a_foto_do_terreno_desce_ao_chao_e_pousa_na_plataforma_alta() {
    assert_eq!(
        caso("fisica_player_pulo_terreno"),
        Some(CasoFisica::PlayerPuloTerreno)
    );
    let t = toques(&mut nivel(), CasoFisica::PlayerPuloTerreno);
    assert!(
        (t.chao[1] - 1.1).abs() < 1e-4 && (t.chao[0] - (4.7 - RECUO_DA_BORDA)).abs() < 1e-4,
        "o pé sobre a Platform Low: {:?}",
        t.chao
    );
    let full = t.full.expect("o Full toca");
    assert!(
        full[1].abs() < 0.05 && full[0] > 5.0,
        "o Full desce ao chão de baixo, além do vão: {full:?}"
    );
    let st = t.standing.expect("o Standing toca");
    assert!(
        (st[1] - ALTA_TOPO).abs() < 0.05 && (ALTA_X[0]..=ALTA_X[1]).contains(&st[0]),
        "o Standing pousa em cima da Platform High: {st:?}"
    );
    let sh = t.short.expect("o Short toca");
    assert!(
        sh[0] < ALTA_X[0] && sh[1] > t.chao[1] && sh[1] < ALTA_TOPO,
        "o Short pára no ar, na parede da Platform High (o pé abaixo do topo dela): {sh:?}"
    );
}

#[test]
fn a_foto_do_pulo_deixa_o_player_onde_a_cena_o_pos() {
    let t = toques(&mut nivel(), CasoFisica::PlayerPulo);
    assert!((t.chao[0] + 8.0).abs() < 1e-4 && (t.chao[1] - 1.05).abs() < 1e-4);
}
