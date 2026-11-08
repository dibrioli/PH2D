//! ⭐ **O tipo do corpo lê-se pelo traço** (dono 07/11, onda 9g: *«Static grosso, Dynamic fino e
//! Kinematic pontilhado - pode ser assim»*), sempre na cor da camada. Lido do traço que o passe
//! entrega para pintar (`outlines` → `tinta_do_contorno`), não da tabela.

use super::tests::{camera, window};
use super::{outlines, tinta_do_contorno};
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::Stroke;

fn corpo(sim: &mut SimWorld, kind: BodyKind, layer: u8, x: f32, sensor: bool) -> ph2d_ecs::Entity {
    let col = Collider {
        shape: ColliderShape::Ball { radius: 0.4 },
        layer,
        is_sensor: sensor,
        ..Collider::default()
    };
    let t = Transform::from_translation(Vec2::new(x, 0.0));
    sim.world_mut().spawn((RigidBody { kind }, col, t)).id()
}

/// Mutações: Static e Dynamic com o mesmo traço; Kinematic sem pontilhado; sensor igual ao
/// Kinematic; a peça de um composto sem o tipo do dono; outra camada.
#[test]
fn o_tipo_do_corpo_le_se_pelo_traco_na_cor_da_camada() {
    let mut sim = SimWorld::new();
    corpo(&mut sim, BodyKind::Static, 1, -3.0, false);
    corpo(&mut sim, BodyKind::Dynamic, 2, -1.0, false);
    corpo(&mut sim, BodyKind::Kinematic, 3, 1.0, false);
    corpo(&mut sim, BodyKind::Kinematic, 4, 3.0, true);
    let dono = corpo(&mut sim, BodyKind::Static, 5, 5.0, false);
    let peca = Collider {
        layer: 6,
        ..Collider::default()
    };
    let t = Transform::from_translation(Vec2::new(0.5, 0.0));
    sim.world_mut().spawn((peca, t, ChildOf(dono)));

    for theme in [Theme::Forge, Theme::Sunstone] {
        let out = outlines(true, false, &mut sim, &[], false, &camera(), window());
        assert_eq!(out.len(), 6, "seis formas, seis contornos");
        let por_camada = |n: u8| -> Stroke {
            let (_, tinta) = out
                .iter()
                .find(|(_, t)| matches!(t, super::Tinta::Camada { camada, .. } if *camada == n))
                .unwrap_or_else(|| panic!("a camada {n} não foi desenhada"));
            let (traco, cor) = tinta_do_contorno(*tinta, theme);
            let base = ph2d_editor_core::paint::resolve(ColorToken::layer(usize::from(n)), theme);
            assert_eq!(cor.components[..3], base.components[..3], "a cor da camada {n}");
            traco
        };
        let (estatico, dinamico, cinematico, sensor, composto) = (
            por_camada(1),
            por_camada(2),
            por_camada(3),
            por_camada(4),
            por_camada(6),
        );
        assert!(
            estatico.width > dinamico.width,
            "Static grosso, Dynamic fino: {} × {}",
            estatico.width,
            dinamico.width
        );
        assert!(estatico.dash_pattern.is_empty() && dinamico.dash_pattern.is_empty());
        assert_eq!(cinematico.width, dinamico.width, "Kinematic também é fino");
        let p = &cinematico.dash_pattern;
        assert!(
            p.len() == 2 && p[0] <= cinematico.width,
            "Kinematic pontilhado: o traço do tamanho da espessura, {p:?}"
        );
        let s = &sensor.dash_pattern;
        assert!(
            s.len() == 2 && s[0] > p[0] * 4.0,
            "o sensor tem traços LONGOS, nunca os pontos do Kinematic: {s:?} × {p:?}"
        );
        assert_eq!(composto.width, estatico.width, "a peça segue o corpo dono");
        assert!(composto.dash_pattern.is_empty());
    }
}
