use super::*;

/// O convite: o quadrado branco sai e a imagem pintada fica no lugar dele. Quem já tem imagem não sai.
#[test]
fn a_imagem_pintada_toma_o_lugar_do_vazio() {
    let mut w = World::new();
    let mut t = ph2d_ecs::Transform::default();
    t.translation = ph2d_core::Vec2::new(4.0, 5.0);
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
    let db = AssetDb::default();
    let (mut w2, _, a, _, c) = mundo(&db);
    substitui_o_vazio(&mut w2, a, c);
    assert!(
        w2.get_entity(a).is_ok(),
        "uma sprite com imagem nunca é apagada"
    );
}
