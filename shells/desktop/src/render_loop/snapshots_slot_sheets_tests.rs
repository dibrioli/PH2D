use super::*;
use ph2d_ecs::{ExtraSheet, ShownSheet, SpritePixels};

/// ⭐ O cartão recebe uma face por folha, na ordem do objeto, com o corte e os píxeis de cada uma e a visível certa;
/// fora de Animated, nada.
///
/// *Mutações: esquecer as `SpriteSheets` ⇒ uma face só; `shown` fixo em 0 ⇒ reprova; sem o filtro de Animated ⇒ a
/// Still publica e reprova.*
#[test]
fn as_folhas_publicadas_sao_as_do_objeto() {
    let db = AssetDb::default();
    let idle = db.insert_image_rgba8(8, 2, vec![255; 8 * 2 * 4]);
    let run = db.insert_image_rgba8(12, 4, vec![200; 12 * 4 * 4]);
    let mut sim = SimWorld::new();
    let extra = ExtraSheet {
        asset: run,
        size: [1.5, 1.5],
        anchor: [0.0, 0.0],
        hframes: 3,
        vframes: 2,
    };
    let e = sim
        .world_mut()
        .spawn((
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            SpritePixels(idle),
            SpriteGrid::new(4, 1),
            SpriteSheets([extra].into_iter().collect()),
            ShownSheet(1),
        ))
        .id();
    let atlas = BTreeMap::new();
    assert_eq!(
        sheet_card(&mut sim, &db, &atlas, 32.0, e.to_bits()),
        None,
        "uma Still não tem folhas"
    );
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_ecs::SpriteAnimations::new());
    let card = sheet_card(&mut sim, &db, &atlas, 32.0, e.to_bits()).expect("duas folhas");
    assert_eq!(card.shown, 1);
    assert_eq!(
        card.sheets
            .iter()
            .map(|s| (s.px, s.grid))
            .collect::<Vec<_>>(),
        [((8, 2), [4, 1]), ((12, 4), [3, 2])]
    );
    assert!(
        card.sheets
            .iter()
            .all(|s| s.image.entity_bits == e.to_bits())
    );
}
