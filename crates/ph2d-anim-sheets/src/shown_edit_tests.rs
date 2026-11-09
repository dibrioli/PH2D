use super::*;
use ph2d_ecs::{ExtraSheet, ShownSheet};

/// Um objeto com a folha `0` e a folha `run` (12 × 4 px, corte 3 × 2, célula de 1,5 m), no ponto `(10, 20)`.
fn com_run(db: &AssetDb) -> (World, Entity, AssetId) {
    let run = db.insert_image_rgba8(12, 4, vec![200; 12 * 4 * 4]);
    let mut w = World::new();
    let e = w
        .spawn((
            {
                let mut t = Transform::default();
                t.translation.x = 10.0;
                t.translation.y = 20.0;
                t
            },
            ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            SpriteSheets(
                [ExtraSheet {
                    asset: run,
                    size: [1.5, 1.5],
                    anchor: [0.25, 0.0],
                    hframes: 3,
                    vframes: 2,
                }]
                .into_iter()
                .collect(),
            ),
        ))
        .id();
    (w, e, run)
}

/// ⭐ Com `run` à vista o verbo lê os píxeis DELA, com o tamanho da imagem inteira (célula × corte); com a folha `0`
/// à vista, nada responde. *Mutações: ignorar a folha visível ⇒ lida com a 0 à vista; `whole` = a célula ⇒ 1,5.*
#[test]
fn o_verbo_le_a_folha_visivel_inteira() {
    let db = AssetDb::default();
    let (mut w, e, _) = com_run(&db);
    assert!(read_extra(&w, e, &db).is_none());
    w.entity_mut(e).insert(ShownSheet(1));
    let r = read_extra(&w, e, &db).expect("a folha run");
    assert_eq!((r.sheet, r.w, r.h, r.rgba[0]), (1, 12, 4, 200));
    assert_eq!(r.whole, [4.5, 3.0]);
    assert_eq!(r.anchor, [0.25, 0.0]);
}

/// ⭐ Gravar troca a imagem DA folha e devolve o tamanho à célula: o Trim que deixa a imagem com 9 × 4 px a 3 m
/// de largo dá células de 1 m. *Mutações: gravar sem trocar o nome ⇒ continua `run`; não dividir pelo corte ⇒ 3 m.*
#[test]
fn gravar_troca_a_imagem_da_folha_e_volta_a_celula() {
    let db = AssetDb::default();
    let (mut w, e, run) = com_run(&db);
    let novo = db.insert_image_rgba8(9, 4, vec![9; 9 * 4 * 4]);
    assert_ne!(novo, run);
    assert!(set_image(&mut w, e, 1, novo, [3.0, 3.0]));
    let s = w.get::<SpriteSheets>(e).unwrap().0[0];
    assert_eq!(s.asset, novo);
    assert_eq!(s.size, [1.0, 1.5]);
    assert!(!set_image(&mut w, e, 2, novo, [1.0, 1.0]), "a folha 2 não existe");
}

/// ⭐ Na folha extra o objeto NÃO anda (é de todas as folhas): o que ele andaria vai para o Pivot dela; na folha `0`
/// o objeto anda como sempre. *Mutações: mexer o `Transform` na folha extra ⇒ o objeto sai de (10, 20); somar com o
/// sinal trocado ⇒ o Pivot vai para −0,75.*
#[test]
fn na_folha_extra_quem_anda_e_o_pivot_dela() {
    let db = AssetDb::default();
    let (mut w, e, _) = com_run(&db);
    place(&mut w, e, 1, [[10.0, 20.0], [11.0, 19.5]], None);
    let t = w.get::<Transform>(e).unwrap().translation;
    assert_eq!((t.x, t.y), (10.0, 20.0), "o objeto fica");
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[0].anchor, [1.25, -0.5]);
    // Com um Pivot dado pelo verbo (o Padding), é dele que se parte.
    place(&mut w, e, 1, [[10.0, 20.0], [10.5, 20.0]], Some([0.0, 0.0]));
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[0].anchor, [0.5, 0.0]);
    place(&mut w, e, 0, [[10.0, 20.0], [12.0, 21.0]], Some([0.1, 0.2]));
    let t = w.get::<Transform>(e).unwrap().translation;
    assert_eq!((t.x, t.y), (12.0, 21.0));
    assert_eq!(w.get::<ph2d_render::Sprite>(e).unwrap().anchor, [0.1, 0.2]);
}
