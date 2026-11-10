//! **A forma solta aparece, e não se confunde com uma simulada** (decisão do dono 07/11, ficha
//! `f1_physics_static_shape.md`): a mesma geometria real, outra tinta, nenhuma anotação.

use super::tests::{camera, points, window};
use super::{Marca, Tinta, outlines, tinta_do_contorno};

const SOLTA: Tinta = Tinta::Camada {
    camada: 0,
    marca: Marca::Solta,
    corpo: None,
    tom: ph2d_panel_physics::tinta_canvas::TOM_NEUTRO,
};
const SOLIDA: Tinta = Tinta::Camada {
    camada: 0,
    marca: Marca::Solida,
    corpo: Some(BodyKind::Static),
    tom: ph2d_panel_physics::tinta_canvas::TOM_NEUTRO,
};
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, InitialVelocity, RigidBody};
use ph2d_tokens::{ColorToken, Theme};

fn forma() -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: 0.8,
            half_y: 0.3,
        },
        offset: [0.5, 0.25],
        ..Collider::default()
    }
}

fn pose() -> Transform {
    let mut t = Transform::from_translation(Vec2::new(1.0, 2.0));
    t.rotation = 0.6;
    t.scale = Vec2::new(2.0, -0.5);
    t
}

/// O pai sem corpo só compõe a pose: a forma continua solta.
fn sob_pai(sim: &mut SimWorld, com_corpo: bool) {
    let mut pai = Transform::from_translation(Vec2::new(-3.0, 1.0));
    pai.rotation = -0.4;
    let pai = sim.world_mut().spawn(pai).id();
    let e = sim.world_mut().spawn((forma(), pose(), ChildOf(pai))).id();
    if com_corpo {
        sim.world_mut().entity_mut(e).insert(RigidBody {
            kind: BodyKind::Static,
        });
    }
}

/// Mutação: voltar ao `continue` sem desenhar, ou desenhá-la pela pose sem o pai, a escala ou o Offset.
#[test]
fn a_forma_solta_aparece_com_a_geometria_de_uma_simulada_e_outra_tinta() {
    let mut solta = SimWorld::new();
    sob_pai(&mut solta, false);
    let mut simulada = SimWorld::new();
    sob_pai(&mut simulada, true);

    let (cam, win) = (camera(), window());
    let a = outlines(true, false, &mut solta, &[], false, &cam, win);
    let b = outlines(true, false, &mut simulada, &[], false, &cam, win);
    assert_eq!(a.len(), 1, "a forma solta tem de aparecer, uma vez");
    assert_eq!(a[0].1, SOLTA);
    assert_eq!(b[0].1, SOLIDA, "o controlo é simulado");
    assert_eq!(
        points(&a[0].0),
        points(&b[0].0),
        "o mesmo contorno que o solver daria: escala, rotação, Offset e pai"
    );
    // A conta à mão, sem o passe: um controlo que passa pelo mesmo código não veria um Offset esquecido.
    let gira = |r: f32, x: f32, y: f32| (x * r.cos() - y * r.sin(), x * r.sin() + y * r.cos());
    let (px, py) = gira(-0.4, 1.0, 2.0);
    let (ox, oy) = gira(0.2, 0.5 * 2.0, 0.25 * -0.5);
    let centro = (-3.0 + px + ox, 1.0 + py + oy);
    let pintados = points(&a[0].0);
    for (sx, sy) in [(1.0_f32, 1.0_f32), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
        let (cx, cy) = gira(0.2, sx * 0.8 * 2.0, sy * 0.3 * 0.5);
        let (ax, ay) = cam.world_to_screen([centro.0 + cx, centro.1 + cy], win);
        let perto = pintados
            .iter()
            .map(|p| (p.0 - f64::from(ax)).hypot(p.1 - f64::from(ay)))
            .fold(f64::MAX, f64::min);
        assert!(
            perto < 0.1,
            "canto ({sx}, {sy}) longe {perto} px do contorno"
        );
    }
    assert!(
        outlines(false, false, &mut solta, &[], false, &cam, win).is_empty(),
        "o interruptor dos contornos vale para ela"
    );
}

/// A solta: a matiz da SUA camada, tracejada, mais apagada que o sensor vazio; a simulada é cheia.
/// Mutação: tirar o tracejado, pintá-la de outra camada, ou tracejar quem é simulado.
#[test]
fn a_tinta_da_solta_e_tracejada_na_cor_da_camada_e_a_simulada_nao() {
    for theme in [Theme::Forge, Theme::Sunstone] {
        let base = ph2d_editor_core::paint::resolve(ColorToken::layer(5), theme).components;
        let solta = Tinta::Camada {
            camada: 5,
            marca: Marca::Solta,
            corpo: None,
            tom: ph2d_panel_physics::tinta_canvas::TOM_NEUTRO,
        };
        let (traco, cor) = tinta_do_contorno(solta, theme);
        assert!(!traco.dash_pattern.is_empty(), "a solta é tracejada");
        assert_eq!(cor.components[..3], base[..3], "a cor é a da camada 5");
        let vazio = Tinta::Camada {
            camada: 5,
            marca: Marca::SensorVazio,
            corpo: Some(BodyKind::Static),
            tom: ph2d_panel_physics::tinta_canvas::TOM_NEUTRO,
        };
        let (_, sensor) = tinta_do_contorno(vazio, theme);
        assert!(cor.components[3] > 0.0 && cor.components[3] < sensor.components[3]);
        let (traco, cor) = tinta_do_contorno(
            Tinta::Camada {
                camada: 5,
                marca: Marca::Solida,
                corpo: Some(BodyKind::Dynamic),
                tom: ph2d_panel_physics::tinta_canvas::TOM_NEUTRO,
            },
            theme,
        );
        assert!(traco.dash_pattern.is_empty(), "a simulada é traço cheio");
        assert_eq!(cor.components[..3], base[..3]);
    }
}

/// A seta diria que algo a põe em movimento; o solver nem a vê.
#[test]
fn a_forma_solta_nao_leva_anotacao_de_corpo() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        forma(),
        pose(),
        InitialVelocity {
            linvel: [3.0, 3.0],
            angvel: 0.0,
        },
    ));
    let saiu = outlines(true, true, &mut sim, &[], false, &camera(), window());
    assert_eq!(saiu.len(), 1, "só o contorno: saíram {}", saiu.len());
}

/// ⭐ **O Look da forma pinta o contorno** (GEF02 onda 8): o Color substitui a cor do tema e a Opacity multiplica o alfa, no
/// contorno e na sombra dele; sem `ShapeLook`, a cor do tema intacta. E o contorno fica onde o pivô põe a forma.
#[test]
fn o_look_da_forma_tinge_o_contorno_e_o_pivo_o_move() {
    let theme = Theme::Dark;
    let corre = |look: Option<ph2d_physics_ecs::ShapeLook>| {
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((forma(), Transform::from_translation(Vec2::new(0.0, 2.0))))
            .id();
        if let Some(l) = look {
            sim.world_mut().entity_mut(e).insert(l);
        }
        let out = outlines(true, false, &mut sim, &[], false, &camera(), window());
        assert_eq!(out.len(), 1);
        let xs: Vec<f64> = points(&out[0].0).iter().map(|p| p.0).collect();
        (tinta_do_contorno(out[0].1, theme).1, xs)
    };
    let (tema, xs0) = corre(None);
    let (tinto, xs1) = corre(Some(ph2d_physics_ecs::ShapeLook {
        pivot: [-1.0, 0.0],
        tint: [1.0, 0.0, 0.0, 1.0],
        opacity: 0.5,
        cor_propria: true,
    }));
    let (t, k) = (tema.components, tinto.components);
    assert_eq!(
        k[..3],
        [1.0, 0.0, 0.0],
        "a cor própria SUBSTITUI a da camada (dono 10/11)"
    );
    assert!(
        (k[3] - t[3] * 0.5).abs() < 1e-6,
        "metade do alfa: {} × {}",
        k[3],
        t[3]
    );
    let menor = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        menor(&xs1) > menor(&xs0) + 1.0,
        "o pivô na borda esquerda empurra a forma para a direita"
    );
}
