//! **A moeda da Marble (uma célula de folha, em Region) traça-se pela arte da célula** — sonda do «quadrado» visto na
//! foto de 09/11: a máscara que o caminho real (`picture_owner` → `traced_frames` → `frames_mask`) dá à forma é a
//! célula `COIN_CELL` da `COIN_SHEET`, píxel a píxel, e o traço é redondo.

use super::*;

fn mundo_com_a_marble_outline() -> (World, Entity) {
    let mut w = World::new();
    monta(&mut w);
    let marble = {
        let mut q = w.query::<(Entity, &Name)>();
        q.iter(&w)
            .find(|(_, n)| n.0 == "Marble")
            .map(|(e, _)| e)
            .expect("Marble")
    };
    let forma = ph2d_physics_ecs::main_shape(&w, marble).expect("forma");
    w.get_mut::<Collider>(forma).expect("collider").shape = ColliderShape::Outline {
        half_x: MARBLE_RADIUS,
        half_y: MARBLE_RADIUS,
    };
    (w, forma)
}

#[test]
fn a_moeda_da_marble_traca_se_pela_celula_da_folha() {
    let (mut w, forma) = mundo_com_a_marble_outline();
    let db = ph2d_asset::AssetDb::new();
    let mut atlas = std::collections::BTreeMap::new();
    ph2d_render::factory_images::register(&db, &mut atlas);
    let dono = ph2d_physics_outline::picture_owner(&w, forma).expect("a Marble tem imagem");
    let quadros = ph2d_physics_outline::traced_frames(&w, dono);
    let id = ph2d_render::texture_of_entity(&w, dono, &atlas).expect("textura");
    let asset = db.get(&id).expect("asset");
    let (iw, ih, px) = asset.image_rgba8().expect("rgba");
    let pic = |_k: u8| {
        Some(ph2d_physics_outline::Picture {
            w: iw,
            h: ih,
            rgba: px.clone().into_owned().into(),
        })
    };
    let (m, _) =
        ph2d_physics_outline::frames_mask(&w, dono, &quadros, pic, 128, 100.0).expect("máscara");
    // A célula real: COIN_CELL na folha.
    let [cx, cy, cw, ch] = COIN_CELL.map(|v| v as u32);
    let celula: usize = (cy..cy + ch)
        .flat_map(|y| (cx..cx + cw).map(move |x| (x, y)))
        .filter(|&(x, y)| px[((y * iw + x) * 4 + 3) as usize] >= 128)
        .count();
    let opacos = m.bits.iter().filter(|b| **b != 0).count();
    println!(
        "folha {iw}×{ih} · quadros {quadros:?} · célula {cw}×{ch}: {celula} opacos · máscara {}×{}: {opacos} opacos",
        m.w, m.h
    );
    assert_eq!([m.w, m.h], [cw, ch], "a máscara tem o tamanho da célula");
    assert_eq!(opacos, celula, "a máscara é a célula");
    let q = ph2d_ecs::scene::EditorCommandQueue::new();
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let n = ph2d_physics_outline::retrace_followed(&mut w, &db, &atlas, 100.0, &q, &reg);
    ph2d_ecs::scene::apply_editor_commands(&mut w, &q, &reg).expect("aplica");
    let o = w.get::<ColliderOutline>(forma).cloned().unwrap_or_default();
    let pontos: Vec<usize> = o.rings.iter().map(Vec::len).collect();
    println!(
        "retraçados {n} · anéis da Marble {pontos:?} · picture_px {:?}",
        o.picture_px
    );
    assert!(
        pontos.iter().any(|p| *p > 4),
        "a moeda traça-se redonda, não um quadrado: {pontos:?}"
    );
    // A lei para fora (dono 09/11): na moeda redonda, Detail 8 usa menos pontos que Detail 2.
    w.entity_mut(forma).insert(ph2d_physics_outline::asked_with(
        &o,
        ph2d_editor_core::OutlineEdit::Detail(8.0),
    ));
    ph2d_physics_outline::retrace_followed(&mut w, &db, &atlas, 100.0, &q, &reg);
    ph2d_ecs::scene::apply_editor_commands(&mut w, &q, &reg).expect("aplica");
    let o8 = w.get::<ColliderOutline>(forma).cloned().unwrap_or_default();
    let (n2, n8) = (
        o.rings.iter().map(Vec::len).sum::<usize>(),
        o8.rings.iter().map(Vec::len).sum::<usize>(),
    );
    println!("pontos da moeda: Detail 2 = {n2} · Detail 8 = {n8}");
    assert!(n8 < n2, "Detail 2 → 8: {n2} → {n8}");
}
