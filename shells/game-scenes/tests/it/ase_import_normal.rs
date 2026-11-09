//! P14e: a normal do `.ase` importado entra na imagem pela porta da família (`normal_map`). Mora na
//! COMPOSIÇÃO: a crate do import (folha) não depende da família (`architecture_no_dependency_climbs_a_layer`).

use ph2d_sprite_ase_import::{grid_for, pack};

/// ⭐ P14e: a normal da camada Normal Map da `heroi_p14` ENTRA na imagem do Sprite importado e é a
/// do Pixel (`compose_normal`, na grelha da cor). *Mutação: `liga_normal` não chamada ⇒ sem normal.*
#[test]
fn a_normal_do_ase_entra_na_imagem_importada() {
    use ph2d_pixel_cena::fixture::heroi_p14;
    let s = heroi_p14();
    let (bytes, _) = ph2d_pixel_io::ase::escrever(&s).expect("grava");
    let imp = ph2d_pixel_cena::importa_ase(&bytes, "heroi", 1.0 / 16.0).expect("import");
    let (cols, rows) = grid_for(imp.cor.len(), 16, 16);
    ph2d_app_sprite_anim::normal_map::set_normais(&[]);
    let db = ph2d_asset::AssetDb::new();
    let cor = db.insert_image_rgba8(96, 16, pack(&imp.cor, 16, 16, cols, rows));
    let atlas = [(7, cor)].into_iter().collect();
    let mut w = ph2d_ecs::World::new();
    let e = w
        .spawn(ph2d_render::Sprite::atlas(7, [1.0; 2], [1.0; 4]))
        .id();
    let n = imp.normal.as_ref().expect("a fixture tem Normal Map");
    let id = db.insert_image_rgba8(96, 16, pack(n, 16, 16, cols, rows));
    let porta: ph2d_sprite_ase_import::EscolherNormal =
        ph2d_app_sprite_anim::normal_map::escolher_normal;
    porta(&w, &db, &atlas, e, Some(id)).expect("liga");
    let id = ph2d_app_sprite_anim::normal_map::normal_de(cor).expect("a imagem tem normal");
    let pixel: Vec<Vec<u8>> = s
        .frames()
        .iter()
        .map(|f| ph2d_pixel_core::compose_normal(&s, f.id))
        .collect();
    let px = db
        .get(&id)
        .and_then(|a| a.image_rgba8().map(|(_, _, p)| p.into_owned()));
    assert_eq!(
        px,
        Some(pack(&pixel, 16, 16, cols, rows)),
        "a normal é a do Pixel"
    );
}
