use super::*;
use ph2d_ecs::{Entity, ExtraSheet, FrameRef, SpriteSheets, World};

fn img(b: u8) -> AssetId {
    AssetId::from_bytes(&[b])
}

/// `idle` (folha 0, célula 7 do átlas, 4 × 1) e `run` (folha 1, 6 × 2, maior e com outro Pivot).
fn duas_folhas() -> (World, Entity, BTreeMap<u32, AssetId>) {
    let mut w = World::new();
    let run = ExtraSheet {
        asset: img(2),
        size: [1.5, 1.5],
        anchor: [0.0, -0.75],
        hframes: 6,
        vframes: 2,
        offsets: Default::default(),
    };
    let e = w
        .spawn((
            crate::Sprite::atlas(7, [1.0, 1.0], [1.0; 4]),
            SpriteGrid::new(4, 1),
            SpriteSheets([run].into_iter().collect()),
        ))
        .id();
    (w, e, [(7, img(1))].into_iter().collect())
}

/// ⭐ GEF 26 F2/F4: a imagem de uma folha extra conta como em uso (Select users, Remove from library, o índice, gravar).
/// Mutação que sangra: `images_of` sem as `sheet_assets` (em uso ignora as extras).
#[test]
fn a_imagem_de_outra_folha_conta_como_em_uso() {
    let (mut w, e, atlas) = duas_folhas();
    assert_eq!(images_of(&w, e, &atlas), vec![img(1), img(2)]);
    let so_extra = w.spawn(SpriteSheets::default()).id();
    assert!(images_of(&w, so_extra, &atlas).is_empty());
    w.get_mut::<SpriteSheets>(e).unwrap().0[0].asset = img(1);
    assert_eq!(images_of(&w, e, &atlas), vec![img(1)], "sem repetidas");
}

/// ⭐ GEF 26 F6: a miniatura mostra o quadro na tela com a imagem E a geometria da folha dele; o vazio, nada.
/// Mutação que sangra: `drawn_texture` sempre pela folha `0` (o retrato com a textura da folha 0).
#[test]
fn a_miniatura_mostra_o_quadro_da_sua_folha() {
    let (mut w, e, atlas) = duas_folhas();
    assert_eq!(drawn_texture(&w, e, &atlas), Some(img(1)));
    w.get_mut::<SpriteGrid>(e)
        .unwrap()
        .show(FrameRef::Cell { sheet: 1, cell: 7 });
    assert_eq!(drawn_texture(&w, e, &atlas), Some(img(2)));
    let (s, g) = crate::drawn_geometry(&w, e).unwrap();
    assert_eq!(
        (s.size, g.map(|g| (g.hframes, g.frame))),
        ([1.5, 1.5], Some((6, 7)))
    );
    w.get_mut::<SpriteGrid>(e).unwrap().show(FrameRef::Empty);
    assert_eq!(
        drawn_texture(&w, e, &atlas),
        None,
        "o quadro vazio não mostra imagem"
    );
}
