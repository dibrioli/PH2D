//! As alças da forma (ficha f1, onda 3) pelo funil real: ponteiro em ecrã → barramento → `apply_physics_edit` →
//! `Collider`, medidas contra o contorno que o overlay pinta, os canais do Auto Key e o solver.

use super::*;
use crate::overlay::outline::{outlines, tests::points};
use ph2d_editor_core::action_bus::{ActionBus, EditorAction};
use ph2d_host::WindowSize;
use ph2d_physics_ecs::PhysicsBridge;
use ph2d_render::Camera2d;
use ph2d_shape_handles::{Alca, Medida, Porta, Pose, Teclas, alterna, no_ecra};

const WIN: WindowSize = WindowSize {
    width: 1400,
    height: 900,
};

fn corpo(sim: &mut SimWorld, pose: Transform, shape: ColliderShape, kind: BodyKind) -> Entity {
    let col = Collider {
        shape,
        offset: [0.2, 0.1],
        ..Collider::default()
    };
    sim.world_mut().spawn((pose, RigidBody { kind }, col)).id()
}

fn rodado(sx: f32) -> Transform {
    let mut t = Transform::from_translation(Vec2::new(1.0, 2.0));
    t.scale = Vec2::new(sx, 1.0);
    t.rotation = 30f32.to_radians();
    t
}

/// Um gesto inteiro em ecrã pelo caminho do produto: agarra a alça onde o overlay a mostra e leva-a a `para`.
fn arrasta(sim: &mut SimWorld, e: Entity, cam: Camera2d, a: Alca, para: (f32, f32), t: Teclas) {
    let de = ecra(sim, e, cam, a);
    let mut bus = ActionBus::default();
    let mut porta = Porta {
        world: sim.world_mut(),
        camera: cam,
        window: WIN,
        bus: &mut bus,
    };
    assert!(porta.desce(de), "o dedo não acertou {a:?}");
    assert!(porta.move_(para, t));
    assert!(porta.sobe());
    for acao in bus.drain() {
        let EditorAction::InspectorPhysicsEdit { edit, entity_bits } = acao else {
            panic!("{acao:?}");
        };
        assert_eq!(entity_bits, e.to_bits(), "só o objecto agarrado");
        apply(sim, e, edit);
    }
}

fn ecra(sim: &SimWorld, e: Entity, cam: Camera2d, a: Alca) -> (f32, f32) {
    let w = sim.world();
    let m = Medida::de(w.get::<Collider>(shape_of(sim, e)).expect("forma"));
    no_ecra(&Pose::de(w, e).expect("pose"), &m, a, &cam, WIN)
}

/// Os vértices do contorno que o overlay PINTA para este único corpo.
fn pintado(sim: &mut SimWorld, cam: Camera2d) -> Vec<(f64, f64)> {
    let o = outlines(true, false, sim, &[], false, &cam, WIN);
    points(&o[0].0)
}

/// O canto da caixa rodada e escalada, com e sem espelho: é um vértice do contorno pintado antes e depois do
/// arrasto, e depois está debaixo do ponteiro. A lição: a régua pára em ECRÃ, nunca no mundo.
#[test]
fn a_alca_da_caixa_e_um_vertice_do_contorno_pintado_e_vai_ao_ponteiro() {
    for sx in [2.0, -2.0] {
        for zoom in [5.0, 20.0] {
            let mut sim = SimWorld::new();
            let caixa = ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.5,
            };
            let e = corpo(&mut sim, rodado(sx), caixa, BodyKind::Static);
            alterna(sim.world_mut(), e.to_bits());
            let cam = Camera2d::new([1.0, 2.0], zoom);
            let vertice = |sim: &mut SimWorld, a| {
                let s = ecra(sim, e, cam, a);
                let v = pintado(sim, cam);
                v.iter()
                    .any(|p| (p.0 - f64::from(s.0)).hypot(p.1 - f64::from(s.1)) < 0.05)
            };
            for a in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
                assert!(
                    vertice(&mut sim, Alca::Lado(a.0, a.1)),
                    "{sx} {zoom}: {a:?}"
                );
            }
            let a = Alca::Lado(1, 1);
            let parado = ecra(&sim, e, cam, Alca::Lado(-1, -1));
            let s = ecra(&sim, e, cam, a);
            let (cx, cy) = ecra(&sim, e, cam, Alca::Miolo);
            let para = (s.0 + (s.0 - cx) * 0.5, s.1 + (s.1 - cy) * 0.5);
            arrasta(&mut sim, e, cam, a, para, Teclas::default());
            let d = ecra(&sim, e, cam, a);
            assert!(
                (d.0 - para.0).hypot(d.1 - para.1) < 0.05,
                "{d:?} × {para:?}"
            );
            let p = ecra(&sim, e, cam, Alca::Lado(-1, -1));
            assert!(
                (p.0 - parado.0).hypot(p.1 - parado.1) < 0.05,
                "o oposto andou"
            );
            assert!(vertice(&mut sim, a), "depois: a alça saiu do contorno");
        }
    }
}

/// A peça dentro de um corpo com pai fora da origem: é a forma dela que muda, e o contorno pintado acompanha.
#[test]
fn a_peca_com_pai_fora_da_origem_vai_ao_ponteiro() {
    let mut sim = SimWorld::new();
    let mut p = rodado(1.5);
    p.translation = Vec2::new(-3.0, 4.0);
    let pai = sim
        .world_mut()
        .spawn((
            p,
            RigidBody {
                kind: BodyKind::Static,
            },
        ))
        .id();
    let col = Collider {
        shape: ColliderShape::Cuboid {
            half_x: 0.5,
            half_y: 0.25,
        },
        ..Collider::default()
    };
    let peca = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.5, 1.0)),
            col,
            ph2d_ecs::ChildOf(pai),
        ))
        .id();
    alterna(sim.world_mut(), peca.to_bits());
    let cam = Camera2d::new([-3.0, 4.0], 12.0);
    let a = Alca::Lado(1, 0);
    let (s, c) = (ecra(&sim, peca, cam, a), ecra(&sim, peca, cam, Alca::Miolo));
    let para = (s.0 + (s.0 - c.0), s.1 + (s.1 - c.1));
    arrasta(&mut sim, peca, cam, a, para, Teclas::default());
    let d = ecra(&sim, peca, cam, a);
    assert!(
        (d.0 - para.0).hypot(d.1 - para.1) < 0.05,
        "{d:?} × {para:?}"
    );
    let col = sim.world().get::<Collider>(peca).expect("peça");
    assert_ne!(
        col.offset,
        [0.0, 0.0],
        "o lado oposto ficou: o Offset compensa"
    );
}

/// Forma e Offset saem da mesma edição: nenhum dos dois se perde na fila diferida (doc 17b A3).
#[test]
fn forma_e_offset_chegam_juntos_e_os_canais_do_auto_key_veem_os_dois() {
    use ph2d_ecs::canal_fisica as f;
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    // Opção B: as alças são da forma, e só com ELA selecionada (dono 08/11).
    let forma = shape_of(&sim, e);
    alterna(sim.world_mut(), forma.to_bits());
    let cam = Camera2d::new([0.0, 3.0], 10.0);
    let antes = crate::canais_corpo::retrato(&sim, e.to_bits()).expect("retrato");
    let a = Alca::Lado(1, 0);
    let (s, c) = (
        ecra(&sim, forma, cam, a),
        ecra(&sim, forma, cam, Alca::Miolo),
    );
    arrasta(
        &mut sim,
        forma,
        cam,
        a,
        (s.0 + (s.0 - c.0) * 0.5, s.1),
        Teclas::default(),
    );
    let depois = crate::canais_corpo::retrato(&sim, e.to_bits()).expect("retrato");
    let ler = crate::canais_corpo::ler;
    assert!(ler(&depois, f::MEIO_X) > ler(&antes, f::MEIO_X), "Half X");
    assert!(
        ler(&depois, f::DESLOC_X) > ler(&antes, f::DESLOC_X),
        "Offset X"
    );
    assert_eq!(
        ler(&depois, f::MEIO_Y),
        ler(&antes, f::MEIO_Y),
        "o outro eixo não é chave"
    );
    assert_eq!(ler(&depois, f::DESLOC_Y), ler(&antes, f::DESLOC_Y));
}

/// A caixa que seguia a imagem passa a «Custom size» quando é puxada; Fit to picture volta.
#[test]
fn a_caixa_puxada_passa_a_custom_e_o_fit_volta() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    let forma = shape_of(&sim, e);
    alterna(sim.world_mut(), forma.to_bits());
    let segue = |sim: &SimWorld| {
        crate::canais_corpo::retrato(sim, e.to_bits())
            .expect("retrato")
            .shape_follows
    };
    assert_eq!(segue(&sim), Some(true));
    let cam = Camera2d::new([0.0, 3.0], 10.0);
    let s = ecra(&sim, forma, cam, Alca::Lado(1, 1));
    arrasta(
        &mut sim,
        forma,
        cam,
        Alca::Lado(1, 1),
        (s.0 + 30.0, s.1 - 30.0),
        Teclas::default(),
    );
    assert_eq!(segue(&sim), Some(false), "puxada");
    apply(&mut sim, e, PhysicsFieldEdit::FitToPicture);
    assert_eq!(segue(&sim), Some(true), "Fit to picture");
    let info = crate::canais_corpo::retrato(&sim, forma.to_bits()).expect("retrato");
    assert!(info.shape_edit_on, "o botão acende pelo retrato da forma");
}

/// O motor colide na medida nova: uma bola pousa no topo puxado da caixa, a 1 m acima de onde pousava.
#[test]
fn a_bola_pousa_onde_a_forma_puxada_passou_a_estar() {
    let pousa = |puxa: bool, rodar: bool| {
        let mut sim = SimWorld::new();
        let caixa = ColliderShape::Cuboid {
            half_x: 3.0,
            half_y: 0.5,
        };
        let mut chao = Transform::from_translation(Vec2::new(0.0, 0.0));
        if rodar {
            chao.scale = Vec2::new(-1.0, 2.0);
        }
        let e = corpo(&mut sim, chao, caixa, BodyKind::Static);
        sim.world_mut()
            .get_mut::<Collider>(e)
            .expect("forma")
            .offset = [0.0, 0.0];
        if puxa {
            alterna(sim.world_mut(), e.to_bits());
            let cam = Camera2d::new([0.0, 0.0], 20.0);
            let a = Alca::Lado(0, 1);
            let de = ecra(&sim, e, cam, a);
            let para = cam.world_to_screen(
                {
                    let w = cam.screen_to_world(de, WIN);
                    [w[0], w[1] + 1.0]
                },
                WIN,
            );
            arrasta(&mut sim, e, cam, a, para, Teclas::default());
        }
        let bola = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(0.0, 6.0)),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.5 },
                    ..Collider::default()
                },
            ))
            .id();
        let mut bridge = PhysicsBridge::new();
        for tick in 1..=240 {
            bridge.dispatch(&mut sim, true, tick);
        }
        sim.world()
            .get::<Transform>(bola)
            .expect("bola")
            .translation
            .y
    };
    let (sem, com) = (pousa(false, false), pousa(true, false));
    assert!(
        (sem - 1.0).abs() < 0.05,
        "o controlo: pousa no topo de partida, {sem}"
    );
    assert!((com - 2.0).abs() < 0.05, "pousa no topo puxado, {com}");
    // Com espelho e Scale Y 2: topo de partida a 1, puxado a 2; a bola a 2,5.
    let esp = pousa(true, true);
    assert!((esp - 2.5).abs() < 0.05, "com espelho, {esp}");
}

/// Desfazer devolve forma e Offset juntos: é um `Collider` por inteiro, a fotografia do undo vê-o num só diff.
#[test]
fn o_valor_do_down_devolve_forma_e_offset_juntos() {
    let (mut sim, e) = sprite_scene();
    apply(&mut sim, e, PhysicsFieldEdit::Add);
    let antes = Medida::de(
        sim.world()
            .get::<Collider>(shape_of(&sim, e))
            .expect("forma"),
    );
    apply(
        &mut sim,
        e,
        PhysicsFieldEdit::ShapeDrag {
            medidas: [3.0, 2.0],
            offset: [0.5, -0.5],
        },
    );
    let depois = Medida::de(
        sim.world()
            .get::<Collider>(shape_of(&sim, e))
            .expect("forma"),
    );
    assert_eq!(
        depois.shape,
        ColliderShape::Cuboid {
            half_x: 3.0,
            half_y: 2.0
        }
    );
    assert_eq!(depois.offset, [0.5, -0.5]);
    apply(&mut sim, e, antes.edicao());
    assert_eq!(
        Medida::de(
            sim.world()
                .get::<Collider>(shape_of(&sim, e))
                .expect("forma")
        ),
        antes
    );
}
