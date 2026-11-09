use super::*;
use ph2d_asset::AssetId;
use ph2d_ecs::ShownSheet;

fn com_duas(db: &AssetDb) -> (World, Entity, AssetId, AssetId) {
    let idle = db.insert_image_rgba8(8, 2, vec![255; 8 * 2 * 4]);
    let run = db.insert_image_rgba8(12, 4, vec![200; 12 * 4 * 4]);
    let mut w = World::new();
    let e = w
        .spawn(SpriteSheets(
            [ExtraSheet {
                asset: run,
                size: [1.5, 1.5],
                anchor: [0.0, 0.0],
                hframes: 3,
                vframes: 2,
            }]
            .into_iter()
            .collect(),
        ))
        .id();
    (w, e, idle, run)
}

/// ⭐ Com a folha `run` à vista, o Painter lê os píxeis DELA (a volta é o funil das ferramentas, `shown_edit`); com a
/// folha `0` à vista, nada disto responde (o caminho de sempre). *Mutação: ignorar a folha visível ⇒ `run` lido com a 0 à vista.*
#[test]
fn o_painter_abre_a_folha_visivel() {
    let db = AssetDb::default();
    let (mut w, e, _, run) = com_duas(&db);
    assert!(read_shown_extra(&w, e, &db).is_none(), "a folha 0 à vista");
    w.entity_mut(e).insert(ShownSheet(1));
    let (px, pw, ph) = read_shown_extra(&w, e, &db).expect("a folha run");
    assert_eq!((pw, ph, px[0]), (12, 4, 200));
}
