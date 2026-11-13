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
    /// Como o Short parou (a quina da Platform High bate na cabeça).
    short_kind: Option<ph2d_physics_ecs::jump_arc::TouchKind>,
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
    let m = |a: &ph2d_physics_ecs::jump_arc::JumpArc| {
        a.touch.map(|t| [chao[0] + t.at[0], chao[1] + t.at[1]])
    };
    Toques {
        chao,
        full: m(&a.full),
        short: m(&a.short),
        standing: m(&a.standing),
        short_kind: a.short.touch.map(|t| t.kind),
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
        full[1].abs() < 0.05 && full[0] > ALTA_X[1],
        "o Full passa a Platform High inteira e desce ao chão de baixo: {full:?}"
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
    assert_eq!(
        t.short_kind,
        Some(ph2d_physics_ecs::jump_arc::TouchKind::Bump),
        "o Short pára por CHOQUE: a foto pinta o corpo-fantasma"
    );
}

#[test]
fn a_foto_do_pulo_deixa_o_player_onde_a_cena_o_pos() {
    let t = toques(&mut nivel(), CasoFisica::PlayerPulo);
    assert!((t.chao[0] + 8.0).abs() < 1e-4 && (t.chao[1] - 1.05).abs() < 1e-4);
}

/// Que curva o jogo corre: Full (salto seguro a correr), Standing (de parado), Short (toque curto).
#[derive(Copy, Clone, Debug)]
enum Salto {
    Full,
    Standing,
    Short,
}

/// O player REAL (ponte, cápsula sem atrito, `LockRotation`, como o `Character` do menu) no nível da foto,
/// assentado 60 tiques no pé da pose e a saltar. A «Platform Low» prolonga-se para a esquerda (borda direita e
/// topo iguais) para ele chegar à velocidade da curva. Devolve a curva cortada e o salto do jogo, ambos
/// relativos ao pé do arranque, e o voo livre da lei.
fn salto_do_jogo(
    salto: Salto,
) -> (
    ph2d_physics_ecs::jump_arc::JumpArc,
    Vec<[f32; 2]>,
    Vec<[f32; 2]>,
) {
    use ph2d_physics_ecs::{LockRotation, MaterialCombine, PhysicsBridge, PlayerInput};
    let alvo_x = 4.7 - RECUO_DA_BORDA;
    let monta = |x0: f32| {
        let mut sim = nivel();
        let low = por_nome(&mut sim, "Platform Low").expect("Platform Low");
        let pl = por_nome(&mut sim, "Player").expect("Player");
        let w = sim.world_mut();
        w.get_mut::<Transform>(low).expect("pose").translation =
            Vec2::new((4.7 - 30.0) * 0.5, 0.85);
        w.get_mut::<Collider>(low).expect("caixa").shape = ColliderShape::Cuboid {
            half_x: (4.7 + 30.0) * 0.5,
            half_y: 0.25,
        };
        let forma = MaterialCombine::sem_atrito(ColliderShape::Capsule {
            half_height: 0.15,
            radius: 0.2,
        });
        w.entity_mut(pl).insert((forma, LockRotation));
        w.get_mut::<PlatformPlayer>(pl).expect("player").air_jumps = 1;
        w.get_mut::<Transform>(pl).expect("pose").translation = Vec2::new(x0, 1.1 + 0.45);
        (sim, PhysicsBridge::new(), pl)
    };
    let drive = if matches!(salto, Salto::Standing) {
        0.0
    } else {
        1.0
    };
    let assenta = |sim: &mut SimWorld, b: &mut PhysicsBridge, pl: Entity| {
        for tick in 1..=60 {
            b.set_player_input(
                pl,
                PlayerInput {
                    drive,
                    ..PlayerInput::default()
                },
            );
            b.dispatch(sim, true, tick);
        }
    };
    let pe = |sim: &SimWorld, pl: Entity| {
        let t = sim.world().get::<Transform>(pl).expect("pose");
        [t.translation.x, t.translation.y]
    };
    // Ensaio: quanto anda a assentar, para a pose assentada cair no pé da foto.
    let (mut sim, mut b, pl) = monta(alvo_x);
    assenta(&mut sim, &mut b, pl);
    let anda = pe(&sim, pl)[0] - alvo_x;
    let (mut sim, mut b, pl) = monta(alvo_x - anda);
    assenta(&mut sim, &mut b, pl);
    let p0 = pe(&sim, pl);
    let s = b.settings();
    let (mut arcs, fh) = {
        let p = sim.world().get::<PlatformPlayer>(pl).expect("player");
        (p.jump_arcs(arc_world(&s, 1.0)), p.float_height)
    };
    let pick = |a: &ph2d_physics_ecs::jump_arc::JumpArcs| match salto {
        Salto::Full => a.full.clone(),
        Salto::Standing => a.standing.clone(),
        Salto::Short => a.short.clone(),
    };
    let voo = pick(&arcs).points;
    let chao = [p0[0], p0[1] - fh];
    ph2d_physics_ecs::arc_terrain::cut_on_terrain(sim.world(), &s, pl, chao, [0.0, 1.0], &mut arcs);
    let mut jogo = vec![[0.0, 0.0]];
    for i in 0..120_u64 {
        let jump = !matches!(salto, Salto::Short) || i == 0;
        b.set_player_input(
            pl,
            PlayerInput {
                drive: 1.0,
                jump,
                ..PlayerInput::default()
            },
        );
        b.dispatch(&mut sim, true, 61 + i);
        let p = pe(&sim, pl);
        jogo.push([p[0] - p0[0], p[1] - p0[1]]);
    }
    (pick(&arcs), jogo, voo)
}

/// **Os papéis da foto são o jogo**: o player real, na pose da foto, pousa/bate no MESMO tique em que a curva
/// acaba e no mesmo sítio — o Full no chão de baixo, o Standing em cima da Platform High, o Short pára na parede
/// dela (o x deixa o voo livre). A 2,1 m o Full agarrava a borda da alta com a perna, no jogo e na curva.
#[test]
fn a_foto_do_terreno_e_o_que_o_player_real_faz() {
    // O primeiro tique em que o jogo sai do voo livre da lei, no eixo dado.
    let sai = |jogo: &[[f32; 2]], voo: &[[f32; 2]], eixo: usize| {
        (1..jogo.len().min(voo.len())).find(|&i| (jogo[i][eixo] - voo[i][eixo]).abs() > 1.0e-3)
    };
    for (salto, eixo, y) in [
        (Salto::Full, 1, -1.1),
        (Salto::Standing, 1, ALTA_TOPO - 1.1),
        (Salto::Short, 0, f32::NAN),
    ] {
        let (arco, jogo, voo) = salto_do_jogo(salto);
        let toque = arco
            .touch
            .unwrap_or_else(|| panic!("{salto:?}: a curva não toca"));
        let tique = arco.points.len() - 1;
        let i =
            sai(&jogo, &voo, eixo).unwrap_or_else(|| panic!("{salto:?}: o jogo nunca sai do voo"));
        let tipo = if y.is_nan() {
            ph2d_physics_ecs::jump_arc::TouchKind::Bump
        } else {
            ph2d_physics_ecs::jump_arc::TouchKind::Land
        };
        assert_eq!(toque.kind, tipo, "{salto:?}: {toque:?}");
        assert_eq!(
            i, tique,
            "{salto:?}: tique do jogo {i} × curva {tique} ({toque:?})"
        );
        assert!(
            (jogo[tique][0] - toque.at[0]).abs() < 0.05,
            "{salto:?}: x do jogo {:?} × curva {:?}",
            jogo[tique],
            toque.at
        );
        if !y.is_nan() {
            assert!(
                (toque.at[1] - y).abs() < 0.02,
                "{salto:?}: a curva pousa à altura errada {toque:?}"
            );
        }
    }
}
