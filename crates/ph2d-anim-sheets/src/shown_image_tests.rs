use super::*;
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

/// ⭐ Com a folha `run` à vista, o Painter lê os píxeis DELA e a pintura volta para ela; com a folha `0` à vista, nada
/// disto responde (o caminho de sempre). *Mutações: ignorar a folha visível ⇒ `run` lido com a 0 à vista; gravar sem
/// trocar o nome ⇒ a folha continua `run`.*
#[test]
fn o_painter_abre_e_grava_a_folha_visivel() {
    let db = AssetDb::default();
    let (mut w, e, _, run) = com_duas(&db);
    assert!(read_shown_extra(&w, e, &db).is_none(), "a folha 0 à vista");
    w.entity_mut(e).insert(ShownSheet(1));
    let (px, pw, ph) = read_shown_extra(&w, e, &db).expect("a folha run");
    assert_eq!((pw, ph, px[0]), (12, 4, 200));
    let pintada = vec![7; 12 * 4 * 4];
    let (antes, novo) = commit_shown_extra(&mut w, e, &db, pintada, 12, 4).expect("grava");
    assert_eq!(antes, run);
    assert_ne!(novo, run);
    let s = w.get::<SpriteSheets>(e).unwrap().0[0];
    assert_eq!(s.asset, novo);
    assert_eq!(
        s.size,
        [1.5, 1.5],
        "o mesmo tamanho de imagem, o mesmo quad"
    );
    let (px, ..) = read_shown_extra(&w, e, &db).unwrap();
    assert_eq!(px[0], 7);
}

/// Uma imagem que volta com outro tamanho mantém o metro por pixel. *Mutação: sem a escala ⇒ 1,5.*
#[test]
fn o_tamanho_novo_mantem_o_metro_por_pixel() {
    let db = AssetDb::default();
    let (mut w, e, ..) = com_duas(&db);
    w.entity_mut(e).insert(ShownSheet(1));
    commit_shown_extra(&mut w, e, &db, vec![1; 24 * 4 * 4], 24, 4).unwrap();
    let s = w.get::<SpriteSheets>(e).unwrap().0[0];
    assert_eq!(s.size, [3.0, 1.5]);
}
