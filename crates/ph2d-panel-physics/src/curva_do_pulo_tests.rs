//! O desenho da curva do pulo: quem a mostra, a alça no ápice e o nome sob o cursor.

use super::*;
use ph2d_core::Vec2;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_vector::Brush;

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

/// Pinta a cena (com terreno: anéis, fantasma e ponto) e devolve as ops gravadas e o desenho.
fn pinta(theme: Theme, sem_marcas: bool) -> (Vec<ph2d_vector::DrawOp>, Desenho) {
    let (mut sim, e) = cena(true);
    if !sem_marcas {
        let pe = 2.0 - PlatformPlayer::default().float_height;
        caixa(&mut sim, -30.0, 30.0, pe, 1.0);
        caixa(&mut sim, 2.0, 3.0, pe + 10.0, 10.0);
    }
    let d = desenho(&sim, &s(), Some(e.to_bits()), true).unwrap();
    let mut scene = VectorScene::new();
    scene.record();
    draw(
        Some(&d),
        true,
        &ProjectSettings::default(),
        (0.0, 0.0),
        &Camera2d::default(),
        WindowSize {
            width: 800,
            height: 600,
        },
        theme,
        &mut scene,
        &mut ph2d_text::TextSystem::new(),
    );
    (scene.take_record(), d)
}

/// ⭐ Dono 12/11 (smoke GEF 04): *«coloque sombra nas fontes, pontos e linhas, pois em fundos claros não posso
/// ver»*. Cada primitiva da curva (as quatro linhas, o anel do pouso, o fantasma, o ponto do contacto) sai em par:
/// o halo escuro primeiro — mesmo caminho, mais largo, o mesmo tracejado — e a tinta por cima. O texto (nome e
/// readout) usa o mesmo halo (`paint_halo_tests`).
#[test]
fn cada_linha_e_ponto_da_curva_sai_sobre_o_seu_halo() {
    use ph2d_vector::DrawOp;
    for theme in [Theme::Forge, Theme::Sunstone] {
        for sem_marcas in [true, false] {
            let (ops, d) = pinta(theme, sem_marcas);
            let primitivas = d.curvas().count()
                + d.marcas()
                    .map(|m| match m {
                        Marca::Pouso { .. } => 1,
                        Marca::Choque { .. } => 2,
                    })
                    .sum::<usize>();
            assert_eq!(
                ops.len(),
                2 * primitivas,
                "{theme:?}: um halo por primitiva ({primitivas}): {} ops",
                ops.len()
            );
            for par in ops.chunks(2) {
                let DrawOp::Stroke {
                    style: h,
                    brush: Brush::Solid(hc),
                    path: hp,
                    ..
                } = &par[0]
                else {
                    panic!("o halo vem primeiro, em traço: {:?}", par[0]);
                };
                let (tc, tp) = match &par[1] {
                    DrawOp::Stroke {
                        style: t,
                        brush: Brush::Solid(tc),
                        path,
                        ..
                    } => {
                        assert!(h.width > t.width, "o halo é mais largo");
                        assert_eq!(h.dash_pattern, t.dash_pattern, "o mesmo tracejado");
                        (tc, path)
                    }
                    DrawOp::Fill {
                        brush: Brush::Solid(tc),
                        path,
                        ..
                    } => (tc, path),
                    o => panic!("a tinta: {o:?}"),
                };
                assert_eq!(*hc, ph2d_editor_core::paint_halo::cor_do_halo(*tc, theme));
                assert_eq!(hp, tp, "o halo segue o mesmo caminho");
            }
        }
    }
}

thread_local! {
    static PINTADOS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn ouve(t: &str, _r: [f32; 4], _corpo: f32) {
    PINTADOS.with(|v| v.borrow_mut().push(t.to_owned()));
}

/// ⭐ Dono 12/11 (GEF 04), «os números não são idênticos»: o readout que o DESENHO pinta a arrastar diz o
/// texto do campo Jump Height — prova o ponto de chamada (`draw` usa `numero_da_alca`), não só a função
/// (`o_numero_da_alca_e_o_do_campo`).
#[test]
fn o_readout_pintado_a_arrastar_e_o_texto_do_campo() {
    use ph2d_editor_core::project::DisplayUnit;
    use ph2d_editor_core::widget::format_number;
    let mut falhas = Vec::new();
    for h in [2.0_f32, 2.5] {
        let (mut sim, e) = cena(true);
        sim.world_mut()
            .get_mut::<PlatformPlayer>(e)
            .unwrap()
            .jump_height = h;
        let d = desenho(&sim, &s(), Some(e.to_bits()), true).unwrap();
        for (unit, ppm) in [(DisplayUnit::Pixels, 100.0), (DisplayUnit::Meters, 100.0)] {
            let project = ProjectSettings {
                display_unit: unit,
                pixels_per_meter: ppm,
                ..ProjectSettings::default()
            };
            let campo = format!(
                "{} {}",
                format_number(unit.from_meters_f64(f64::from(h), ppm)),
                unit.suffix()
            );
            PINTADOS.with(|v| v.borrow_mut().clear());
            ph2d_editor_core::paint_halo::ouve_o_texto(Some(ouve));
            draw(
                Some(&d),
                true,
                &project,
                (-1000.0, -1000.0),
                &Camera2d::default(),
                WindowSize {
                    width: 800,
                    height: 600,
                },
                Theme::Forge,
                &mut VectorScene::new(),
                &mut ph2d_text::TextSystem::new(),
            );
            ph2d_editor_core::paint_halo::ouve_o_texto(None);
            let pintados = PINTADOS.with(|v| v.borrow().clone());
            let alvo = format!("{campo} high");
            if pintados.is_empty() || !pintados.iter().all(|t| t.contains(&alvo)) {
                falhas.push(format!(
                    "{h} m em {unit:?}: campo «{campo}», pintado {pintados:?}"
                ));
            }
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}
