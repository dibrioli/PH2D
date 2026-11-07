//! **A forma solta aparece, e não se confunde com uma simulada** (decisão do dono 07/11, ficha
//! `f1_physics_static_shape.md`): a mesma geometria real, outra tinta, nenhuma anotação.

use super::tests::{camera, points, window};
use super::{DYNAMIC_RGBA, LOOSE_INK, STATIC_RGBA, outlines, tinta_do_contorno};
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
    assert_eq!(a[0].1, LOOSE_INK);
    assert_eq!(b[0].1, STATIC_RGBA, "o controlo é simulado");
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

/// Mutação: tirar o tracejado, trocar o token, ou tracejar quem é simulado.
#[test]
fn a_tinta_da_solta_e_tracejada_e_vem_do_token_e_a_simulada_nao() {
    for theme in [Theme::Forge, Theme::Sunstone] {
        let (traco, cor) = tinta_do_contorno(LOOSE_INK, theme);
        assert!(!traco.dash_pattern.is_empty(), "a solta é tracejada");
        assert_eq!(
            cor,
            ph2d_editor_core::paint::resolve(ColorToken::Text3, theme)
        );
        assert!(cor.components[3] > 0.0, "a marca não pode sair invisível");
        for simulada in [STATIC_RGBA, DYNAMIC_RGBA] {
            let (traco, cor) = tinta_do_contorno(simulada, theme);
            assert!(traco.dash_pattern.is_empty(), "a simulada é traço cheio");
            assert_eq!(cor.components, simulada);
        }
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
