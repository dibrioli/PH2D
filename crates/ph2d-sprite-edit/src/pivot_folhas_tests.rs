//! O Pivot de um Animated de várias folhas (plano GEF 26 §3.2, decisão 8): edita-se o da folha VISÍVEL no painel.

use super::*;
use ph2d_ecs::{ExtraSheet, ShownSheet};

/// 8 × 8 com desenho só em `x ∈ 2..6`, `y ∈ 4..8`.
fn boneco() -> Vec<u8> {
    let mut px = vec![0_u8; 8 * 8 * 4];
    for y in 4..8 {
        for x in 2..6 {
            px[(y * 8 + x) * 4 + 3] = 255;
        }
    }
    px
}

/// Folha 0 = um quad branco 2 × 2 com um filho e colisão; folha 1 = o boneco num quad 3 × 3, à vista.
fn duas_folhas(db: &AssetDb) -> (World, Entity, Entity) {
    let mut w = World::new();
    let extra = ExtraSheet {
        asset: db.insert_image_rgba8(8, 8, boneco()),
        size: [3.0, 3.0],
        anchor: [0.0, 0.0],
        hframes: 1,
        vframes: 1,
    };
    let e = w
        .spawn((
            Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [2.0, 2.0], [1.0; 4]),
            Transform {
                translation: Vec2::new(5.0, 7.0),
                ..Default::default()
            },
            SpriteSheets([extra].into_iter().collect()),
            ShownSheet(1),
            Collider::default(),
        ))
        .id();
    let filho = w
        .spawn((
            Transform {
                translation: Vec2::new(1.0, 1.0),
                ..Default::default()
            },
            ph2d_ecs::ChildOf(e),
        ))
        .id();
    (w, e, filho)
}

fn ancora_extra(w: &World, e: Entity) -> [f32; 2] {
    w.get::<SpriteSheets>(e).unwrap().0[0].anchor
}

/// O que não pode mexer quando se edita uma folha extra.
fn o_objeto(w: &World, e: Entity, filho: Entity) -> (Transform, [f32; 2], [f32; 2], Transform) {
    (
        *w.get::<Transform>(e).unwrap(),
        w.get::<Sprite>(e).unwrap().anchor,
        w.get::<Collider>(e).unwrap().offset,
        *w.get::<Transform>(filho).unwrap(),
    )
}

/// ⭐ Com a folha 1 à vista, o Pivot de 9 pontos conta sobre o desenho DELA e só ela muda: a posição, o Pivot da
/// folha 0, a colisão e os filhos ficam. Com a folha 0 à vista, a lei de sempre, e a folha 1 fica.
///
/// *Mutações: ignorar a folha visível ⇒ muda o `Sprite` e reprova; medir a caixa opaca da folha 0 (o quad inteiro)
/// ⇒ Top Left cai em −1,5 e reprova; mover a origem como na folha 0 ⇒ a posição anda e reprova.*
#[test]
fn o_pivot_mexe_na_folha_visivel() {
    let db = AssetDb::default();
    let atlas = BTreeMap::new();
    let (mut w, e, filho) = duas_folhas(&db);
    let antes = o_objeto(&w, e, filho);
    assert!(set_pivot(
        &mut w,
        e,
        &db,
        &atlas,
        100.0,
        SpritePivot::Ponto([0, 0])
    ));
    // O desenho em u 0,25..0,75, v 0,5..1 de um quad 3 × 3: Top Left = (−0,75, 0) desde o centro.
    assert_eq!(ancora_extra(&w, e), [0.75, 0.0]);
    assert_eq!(o_objeto(&w, e, filho), antes, "nada mais se mexe");
    assert_eq!(
        pivot_de(&w, e, &db, &atlas, 100.0).map(|p| p.0),
        Some(Some([0, 0])),
        "o cartão lê o Pivot da folha visível"
    );
    assert!(set_pivot(
        &mut w,
        e,
        &db,
        &atlas,
        100.0,
        SpritePivot::Ponto([1, 2])
    ));
    assert_eq!(ancora_extra(&w, e), [0.0, 1.5], "Bottom = o pé do desenho");
    assert!(!set_pivot(
        &mut w,
        e,
        &db,
        &atlas,
        100.0,
        SpritePivot::Ponto([1, 2])
    ));
    // A folha 0 à vista: a lei de sempre (a origem compensa), e a folha 1 não muda.
    w.entity_mut(e).insert(ShownSheet(0));
    assert!(set_pivot(
        &mut w,
        e,
        &db,
        &atlas,
        100.0,
        SpritePivot::Px([0.0, -100.0])
    ));
    assert_eq!(w.get::<Sprite>(e).unwrap().anchor, [0.0, 1.0]);
    assert_ne!(
        w.get::<Transform>(e).unwrap().translation,
        antes.0.translation
    );
    assert_eq!(ancora_extra(&w, e), [0.0, 1.5]);
}

/// A ferramenta Pivot do canvas sobre a folha extra: a âncora dela e só.
///
/// *Mutação: o arrasto seguir a lei da folha 0 ⇒ a origem vai para `(9, 9)` e reprova.*
#[test]
fn arrastar_o_pivot_da_folha_extra_nao_move_o_objeto() {
    let db = AssetDb::default();
    let (mut w, e, filho) = duas_folhas(&db);
    let antes = o_objeto(&w, e, filho);
    arrasta(&mut w, e, [9.0, 9.0], [0.2, 0.3]);
    assert_eq!(ancora_extra(&w, e), [0.2, 0.3]);
    assert_eq!(o_objeto(&w, e, filho), antes);
    assert_eq!(
        sprite_do_pivot(&w, e).map(|(k, s)| (k, s.anchor, s.size)),
        Some((1, [0.2, 0.3], [3.0, 3.0]))
    );
}

/// Um undo pode tirar a folha que estava à vista: a visível cai na 0 em vez de apontar para nada.
#[test]
fn a_folha_visivel_que_saiu_cai_na_primeira() {
    let db = AssetDb::default();
    let (mut w, e, _) = duas_folhas(&db);
    assert_eq!(ph2d_ecs::shown_sheet(&w, e), 1);
    w.entity_mut(e).insert(ShownSheet(2));
    assert_eq!(ph2d_ecs::shown_sheet(&w, e), 0);
    w.entity_mut(e).insert(ShownSheet(1));
    w.entity_mut(e).remove::<SpriteSheets>();
    assert_eq!(ph2d_ecs::shown_sheet(&w, e), 0);
}

/// A imagem inteira da folha 1, como o `read_shown_source` da shell a lê.
fn imagem_da_folha_1(w: &World, e: Entity, db: &AssetDb) -> ph2d_render::SpriteImage {
    let s = w.get::<SpriteSheets>(e).unwrap().0[0];
    let asset = db.get(&s.asset).unwrap();
    let (lw, lh, px) = asset.image_rgba8().unwrap();
    ph2d_render::SpriteImage::from_bytes(lw, lh, px.into_owned(), ph2d_render::AlphaMode::Straight)
}

/// ⭐ O encaixe do Pivot ao desenho (CTRL) com a folha 1 à vista mede o desenho DELA no quad DELA (3 × 3), e numa
/// folha com várias células, a união das células sobre UMA célula (a lei do Pivot de 9 pontos).
///
/// *Mutações: o tamanho do `Sprite` (a folha 0, 2 × 2) ⇒ −0,5 e reprova; a imagem inteira como se fosse uma célula
/// ⇒ o centro em x de 16 px e reprova.*
#[test]
fn o_encaixe_do_pivot_mede_a_folha_visivel() {
    let db = AssetDb::default();
    let (mut w, e, _) = duas_folhas(&db);
    let img = imagem_da_folha_1(&w, e, &db);
    // O desenho em u 0,25..0,75, v 0,5..1 de um quad 3 × 3: o centro dele = (0, −0,75) desde o centro do quad.
    assert_eq!(centro_do_desenho(&w, e, &img), Some([0.0, -0.75]));
    // Duas células de 8 × 8: o boneco em x 2..6 na primeira e 1..5 na segunda ⇒ a união 1..6 da célula.
    let mut px = vec![0_u8; 16 * 8 * 4];
    for y in 4..8 {
        for x in (2..6).chain(9..13) {
            px[(y * 16 + x) * 4 + 3] = 255;
        }
    }
    let mut sheets = w.get::<SpriteSheets>(e).unwrap().clone();
    sheets.0[0].asset = db.insert_image_rgba8(16, 8, px);
    sheets.0[0].hframes = 2;
    w.entity_mut(e).insert(sheets);
    let img = imagem_da_folha_1(&w, e, &db);
    assert_eq!(centro_do_desenho(&w, e, &img), Some([-0.1875, -0.75]));
    // A folha 0 à vista: a lei de sempre (o quad do `Sprite`, 2 × 2, sobre a imagem lida: x 2..13 de 16).
    w.entity_mut(e).insert(ShownSheet(0));
    assert_eq!(centro_do_desenho(&w, e, &img), Some([-0.0625, -0.5]));
}
