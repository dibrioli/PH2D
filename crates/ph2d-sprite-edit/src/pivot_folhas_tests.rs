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
        offsets: Default::default(),
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

/// O que fica no MUNDO quando se edita o Pivot da folha 1 (sem rotação nem escala): o centro do quad dela, o filho, a
/// colisão — e o Pivot da folha 0, que ela guarda para seguir a origem.
fn no_mundo(w: &World, e: Entity, filho: Entity) -> ([f32; 2], [f32; 2], [f32; 2], [f32; 2]) {
    let t = w.get::<Transform>(e).unwrap().translation;
    let a = ancora_extra(w, e);
    let f = w.get::<Transform>(filho).unwrap().translation;
    let c = w.get::<Collider>(e).unwrap().offset;
    (
        [t.x + a[0], t.y + a[1]],
        [t.x + f.x, t.y + f.y],
        [t.x + c[0], t.y + c[1]],
        w.get::<Sprite>(e).unwrap().anchor,
    )
}

/// ⭐ Com a folha 1 à vista, o Pivot de 9 pontos conta sobre o desenho DELA; o desenho dela, o filho e a colisão
/// ficam no mundo e a origem anda (smoke do dono 09/11: «quem move é a imagem»). A folha 0 guarda o Pivot dela e
/// segue a origem. Com a folha 0 à vista, a mesma lei, e a folha 1 guarda o dela.
///
/// *Mutações: ignorar a folha visível ⇒ muda o `Sprite` e reprova; medir a caixa opaca da folha 0 (o quad inteiro)
/// ⇒ Top Left cai em −1,5 e reprova; a origem ficar (a lei antiga) ⇒ o desenho da folha 1 anda e reprova.*
#[test]
fn o_pivot_mexe_na_folha_visivel() {
    let db = AssetDb::default();
    let atlas = BTreeMap::new();
    let (mut w, e, filho) = duas_folhas(&db);
    let antes = no_mundo(&w, e, filho);
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
    assert_eq!(no_mundo(&w, e, filho), antes, "nada se mexe no mundo");
    assert_eq!(
        w.get::<Transform>(e).unwrap().translation,
        Vec2::new(4.25, 7.0),
        "a origem foi ao Top Left"
    );
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
    assert_eq!(no_mundo(&w, e, filho), antes);
    assert!(!set_pivot(
        &mut w,
        e,
        &db,
        &atlas,
        100.0,
        SpritePivot::Ponto([1, 2])
    ));
    // A folha 0 à vista: a mesma lei (a origem compensa), e a folha 1 guarda o Pivot dela.
    w.entity_mut(e).insert(ShownSheet(0));
    let origem = w.get::<Transform>(e).unwrap().translation;
    assert!(set_pivot(
        &mut w,
        e,
        &db,
        &atlas,
        100.0,
        SpritePivot::Px([0.0, -100.0])
    ));
    assert_eq!(w.get::<Sprite>(e).unwrap().anchor, [0.0, 1.0]);
    assert_ne!(w.get::<Transform>(e).unwrap().translation, origem);
    assert_eq!(ancora_extra(&w, e), [0.0, 1.5]);
}

/// ⭐ A ferramenta Pivot do canvas sobre a folha extra: o pivô vai ao cursor e o desenho DELA fica (smoke do dono
/// 09/11). A âncora e a origem chegam do arrasto, que mantém o centro do quad: `origem + âncora = (5, 7)`.
///
/// *Mutação: a lei antiga (só a âncora, a origem fica) ⇒ a origem não vai a `(4, 6)` e o desenho anda; reprova.*
#[test]
fn arrastar_o_pivot_da_folha_extra_deixa_o_desenho_dela_no_lugar() {
    let db = AssetDb::default();
    let (mut w, e, filho) = duas_folhas(&db);
    let antes = no_mundo(&w, e, filho);
    arrasta(&mut w, e, [4.0, 6.0], [1.0, 1.0]);
    assert_eq!(ancora_extra(&w, e), [1.0, 1.0]);
    assert_eq!(
        w.get::<Transform>(e).unwrap().translation,
        Vec2::new(4.0, 6.0)
    );
    assert_eq!(no_mundo(&w, e, filho), antes);
    assert_eq!(
        sprite_do_pivot(&w, e).map(|(k, s)| (k, s.anchor, s.size)),
        Some((1, [1.0, 1.0], [3.0, 3.0]))
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
    let s = w.get::<SpriteSheets>(e).unwrap().0[0].clone();
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

/// ⭐ Auditoria do fecho 08/11 (J): a folha 0 de um Animated segue a MESMA lei das extra — a união das células da
/// grelha dela sobre o quad de UMA, virada com o flip. Duas células de 8 px com o boneco em 2..6 e 1..5 ⇒ 1..6 da
/// célula; com `flip_x` o lado vira. *Mutação: a lei antiga (a imagem inteira como uma célula, sem flip) ⇒
/// `[−0,0625, −0,5]`.*
#[test]
fn o_encaixe_da_folha_zero_segue_a_grelha_e_o_flip() {
    let db = AssetDb::default();
    let (mut w, e, _) = duas_folhas(&db);
    let mut px = vec![0_u8; 16 * 8 * 4];
    for y in 4..8 {
        for x in (2..6).chain(9..13) {
            px[(y * 16 + x) * 4 + 3] = 255;
        }
    }
    let img = ph2d_render::SpriteImage::from_bytes(16, 8, px, ph2d_render::AlphaMode::Straight);
    w.entity_mut(e)
        .insert((ShownSheet(0), SpriteGrid::new(2, 1)));
    w.get_mut::<Sprite>(e).unwrap().flip_x = true;
    assert_eq!(centro_do_desenho(&w, e, &img), Some([0.125, -0.5]));
}
