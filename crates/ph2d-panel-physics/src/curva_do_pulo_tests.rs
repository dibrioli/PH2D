//! O desenho da curva do pulo: quem a mostra, a alça no ápice e o nome sob o cursor.

use super::*;
use ph2d_core::Vec2;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};

fn cena(show: bool) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            PlatformPlayer {
                show_jump_arc: show,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(1.0, 2.0)),
        ))
        .id();
    (sim, e)
}

fn s() -> PhysicsSettings {
    PhysicsSettings::default()
}

#[test]
fn so_o_selecionado_com_a_caixa_e_parado_mostra_a_curva() {
    let (sim, e) = cena(true);
    assert!(desenho(&sim, &s(), Some(e.to_bits()), true).is_some());
    assert!(desenho(&sim, &s(), None, true).is_none(), "sem seleção");
    assert!(
        desenho(&sim, &s(), Some(e.to_bits()), false).is_none(),
        "durante o Play"
    );
    let (sim, e) = cena(false);
    assert!(
        desenho(&sim, &s(), Some(e.to_bits()), true).is_none(),
        "caixa desmarcada"
    );
}

#[test]
fn a_curva_comeca_no_chao_sob_o_player_e_a_alca_mora_no_apice() {
    let (sim, e) = cena(true);
    let d = desenho(&sim, &s(), Some(e.to_bits()), true).unwrap();
    let p = PlatformPlayer::default();
    assert!((d.chao[0] - 1.0).abs() < 1e-5 && (d.chao[1] - (2.0 - p.float_height)).abs() < 1e-5);
    let h = handles(Some(&d));
    assert_eq!(h.len(), 1);
    assert_eq!(h[0].kind, PointHandleKind::JumpApex);
    assert_eq!(h[0].world, d.apice());
    let curvas: Vec<_> = d.curvas().map(|(c, _)| c).collect();
    assert_eq!(
        curvas,
        [Curva::Standing, Curva::Short, Curva::Full],
        "sem Air Jumps, sem curva do ar"
    );
}

#[test]
fn o_nome_aparece_sobre_a_curva_e_nao_longe_dela() {
    let (sim, e) = cena(true);
    let d = desenho(&sim, &s(), Some(e.to_bits()), true).unwrap();
    let cam = Camera2d::default();
    let win = WindowSize {
        width: 800,
        height: 600,
    };
    let topo = cam.world_to_screen(d.apice(), win);
    assert_eq!(sob_o_cursor(&d, &cam, win, topo), Some(Curva::Full));
    assert_eq!(sob_o_cursor(&d, &cam, win, (topo.0, topo.1 - 200.0)), None);
}

/// Uma caixa estática de `x0..x1` com o topo em `top` e altura `h`.
fn caixa(sim: &mut SimWorld, x0: f32, x1: f32, top: f32, h: f32) {
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: (x1 - x0) * 0.5,
                half_y: h * 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new((x0 + x1) * 0.5, top - h * 0.5)),
    ));
}

/// As marcas da cena: quantos pousos, quantos choques.
fn conta(sim: &SimWorld, e: Entity) -> (usize, usize) {
    let d = desenho(sim, &s(), Some(e.to_bits()), true).unwrap();
    d.marcas().fold((0, 0), |(p, c), m| match m {
        Marca::Pouso { .. } => (p + 1, c),
        Marca::Choque { .. } => (p, c + 1),
    })
}

#[test]
fn o_pouso_pinta_o_anel_e_o_choque_o_corpo_fantasma() {
    // Sem terreno: a curva cai e não acaba em nada.
    let (sim, e) = cena(true);
    assert_eq!(conta(&sim, e), (0, 0), "sem terreno, sem marca");
    // Chão plano largo: as três curvas pousam, nenhum fantasma.
    let (mut sim, e) = cena(true);
    let pe = 2.0 - PlatformPlayer::default().float_height;
    caixa(&mut sim, -30.0, 30.0, pe, 1.0);
    assert_eq!(conta(&sim, e), (3, 0), "no chão só anéis");
    // Uma parede alta logo à frente: a corrida e o corte batem no lado; o parado pousa.
    let (mut sim, e) = cena(true);
    caixa(&mut sim, -30.0, 30.0, pe, 1.0);
    caixa(&mut sim, 2.0, 3.0, pe + 10.0, 10.0);
    let (pousos, choques) = conta(&sim, e);
    assert!(
        choques >= 1,
        "a parede pinta o corpo-fantasma: {pousos}/{choques}"
    );
    assert_eq!(pousos + choques, 3);
    let d = desenho(&sim, &s(), Some(e.to_bits()), true).unwrap();
    let full = d.marcas().find_map(|m| match m {
        Marca::Choque {
            curva: Curva::Full,
            corpo,
            contacto,
        } => Some((corpo, contacto)),
        _ => None,
    });
    let (corpo, contacto) = full.expect("o Full bate na parede");
    assert!(
        (contacto[0] - 2.0).abs() < 0.02,
        "o ponto está na face: {contacto:?}"
    );
    // A cápsula (raio 0,2) encosta na face: o fantasma fica a 0,2 dela.
    assert!(
        (corpo.t[0] - 1.8).abs() < 0.02,
        "o fantasma encosta: {corpo:?}"
    );
}
