use super::*;

#[test]
fn keep_size_devolve_o_tamanho_de_antes() {
    let mut w = World::new();
    let a = w
        .spawn(Sprite::atlas(
            ph2d_render::WHITE_TILE_KEY,
            [3.0, 3.0],
            [1.0; 4],
        ))
        .id();
    assert!(keep_size(&mut w, a, [8.0, 2.0]));
    assert_eq!(w.get::<Sprite>(a).unwrap().size, [8.0, 2.0]);
    assert!(!keep_size(&mut w, a, [8.0, 2.0]));
}
