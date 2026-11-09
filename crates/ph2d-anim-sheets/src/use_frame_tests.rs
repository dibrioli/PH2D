use super::*;
use ph2d_ecs::{SimWorld, SpriteAnimations, Transform};

fn sprite(sim: &mut SimWorld, grid: Option<(u32, u32)>) -> Entity {
    let mut e = sim.world_mut().spawn((
        Transform::default(),
        ph2d_render::Sprite::atlas(0, [1.0, 2.0], [1.0; 4]),
    ));
    if let Some((h, v)) = grid {
        e.insert(SpriteGrid {
            hframes: h,
            vframes: v,
            frame: 0,
            sheet: 0,
        });
    }
    e.id()
}

/// ⭐ A etiqueta diz o uso, e a grelha só quando é animada: Still não tem moldura (smoke do dono 09/11).
#[test]
fn a_etiqueta_diz_o_uso_e_a_grelha_da_animada() {
    let mut sim = SimWorld::new();
    let parada = sprite(&mut sim, None);
    assert_eq!(label(sim.world(), parada), None, "Still não leva etiqueta");
    let animada = sprite(&mut sim, Some((6, 2)));
    sim.world_mut()
        .entity_mut(animada)
        .insert(SpriteAnimations::new());
    assert_eq!(
        label(sim.world(), animada).as_deref(),
        Some("Animated \u{00b7} 6 \u{00d7} 2")
    );
    let regiao = sprite(&mut sim, None);
    sim.world_mut()
        .entity_mut(regiao)
        .insert(ph2d_ecs::ImageRegion::default());
    assert_eq!(label(sim.world(), regiao).as_deref(), Some("Region"));
}

/// Cada uso tem a sua cor, e as três são diferentes.
#[test]
fn cada_uso_tem_a_sua_cor() {
    let cores = [ImageUse::Animated, ImageUse::Repeated, ImageUse::Region].map(use_color);
    assert!(cores.iter().all(Option::is_some));
    assert!(cores[0] != cores[1] && cores[1] != cores[2] && cores[0] != cores[2]);
    assert_eq!(use_color(ImageUse::Still), None);
}

/// ⭐ Em Edit a moldura envolve a folha INTEIRA e conta os quadros; fora dele, o quad de uma célula.
#[test]
fn em_edit_a_moldura_envolve_a_folha_inteira() {
    let mut sim = SimWorld::new();
    let e = sprite(&mut sim, Some((6, 2)));
    sim.world_mut()
        .entity_mut(e)
        .insert(SpriteAnimations::new());
    let fora = desenho(sim.world(), e, 100.0).expect("quad");
    assert_eq!((fora.w, fora.h, fora.celulas.is_none()), (1.0, 2.0, true));
    sim.world_mut().insert_resource(ph2d_ecs::ImagemAberta {
        entity: e,
        quad: [6.0, 4.0],
    });
    let dentro = desenho(sim.world(), e, 100.0).expect("folha");
    assert_eq!((dentro.w, dentro.h), (6.0, 4.0));
    let (cols, rows, viva) = dentro.celulas.expect("grelha em Edit");
    assert_eq!((cols, rows), (6, 2));
    assert!(viva.is_some(), "o quadro que toca é desta folha");
}
