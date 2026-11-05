use super::*;

/// O convite: o quadrado branco sai e a imagem pintada fica no lugar dele. Quem já tem imagem não sai.
#[test]
fn a_imagem_pintada_toma_o_lugar_do_vazio() {
    let mut w = World::new();
    let t = ph2d_ecs::Transform {
        translation: ph2d_core::Vec2::new(4.0, 5.0),
        ..Default::default()
    };
    let vazio = w
        .spawn((Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]), t))
        .id();
    let nova = w.spawn(ph2d_ecs::Transform::default()).id();
    substitui_o_vazio(&mut w, vazio, nova);
    assert!(w.get_entity(vazio).is_err());
    assert_eq!(
        w.get::<ph2d_ecs::Transform>(nova).unwrap().translation,
        t.translation
    );
    let db = ph2d_asset::AssetDb::default();
    let imagem = db.insert_image_rgba8(1, 1, vec![9, 9, 9, 255]);
    let mut w2 = World::new();
    let a = w2
        .spawn((
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
            SpritePixels(imagem),
        ))
        .id();
    let c = w2.spawn(ph2d_ecs::Transform::default()).id();
    substitui_o_vazio(&mut w2, a, c);
    assert!(
        w2.get_entity(a).is_ok(),
        "uma sprite com imagem nunca é apagada"
    );
}
