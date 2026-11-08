use super::*;
use ph2d_ecs::{ExtraSheet, FrameRef};

fn img(b: u8) -> AssetId {
    AssetId::from_bytes(&[b])
}

fn run_sheet() -> ExtraSheet {
    ExtraSheet {
        asset: img(2),
        size: [1.5, 1.5],
        anchor: [0.0, -0.75],
        hframes: 6,
        vframes: 2,
    }
}

/// Um objeto de duas folhas: `idle` (4 × 1, no átlas) e `run` (6 × 2, por subir).
fn two_sheets() -> (World, Entity, Sprite) {
    let mut w = World::new();
    let sprite = Sprite::atlas(7, [1.0, 1.0], [1.0; 4]);
    let e = w
        .spawn((
            sprite,
            SpriteGrid::new(4, 1),
            SpriteSheets([run_sheet()].into_iter().collect()),
        ))
        .id();
    (w, e, sprite)
}

fn show(w: &mut World, e: Entity, f: FrameRef) {
    w.get_mut::<SpriteGrid>(e).unwrap().show(f);
}

#[test]
fn um_quadro_de_outra_folha_pinta_a_imagem_dela() {
    let (mut w, e, sprite) = two_sheets();
    let mut subidas = Vec::new();
    resolve_sheet_textures(&mut w, &BTreeMap::new(), &mut |id| {
        subidas.push(id);
        Some(42)
    });
    assert_eq!(subidas, [img(2)], "sobe uma vez, a imagem da folha 1");
    assert_eq!(sheet_view(&w, e, &sprite), SheetView::Own);
    show(&mut w, e, FrameRef::Cell { sheet: 1, cell: 7 });
    let SheetView::Other(s, g) = sheet_view(&w, e, &sprite) else {
        panic!("a folha 1 tem de pôr a imagem dela em cena");
    };
    assert_eq!(s.source, SpriteSource::Individual { texture_id: 42 });
    assert_eq!((s.size, s.anchor), (run_sheet().size, run_sheet().anchor));
    assert_eq!((g.hframes, g.vframes, g.frame, g.sheet), (6, 2, 7, 0));
    assert_eq!(s.tint, sprite.tint, "o resto do Sprite é o da folha 0");
    // Já resolvida: o quadro seguinte não sobe outra vez.
    resolve_sheet_textures(&mut w, &BTreeMap::new(), &mut |_| panic!("subiu de novo"));
}

#[test]
fn a_imagem_no_atlas_nao_sobe_copia() {
    let (mut w, e, sprite) = two_sheets();
    let atlas = BTreeMap::from([(3, img(2))]);
    resolve_sheet_textures(&mut w, &atlas, &mut |_| panic!("estava no átlas"));
    show(&mut w, e, FrameRef::Cell { sheet: 1, cell: 0 });
    let SheetView::Other(s, _) = sheet_view(&w, e, &sprite) else {
        panic!("folha 1");
    };
    assert_eq!(s.source, SpriteSource::Atlas { key: 3 });
}

#[test]
fn o_vazio_e_a_textura_por_subir_nao_pintam() {
    let (mut w, e, sprite) = two_sheets();
    show(&mut w, e, FrameRef::Cell { sheet: 1, cell: 0 });
    assert_eq!(sheet_view(&w, e, &sprite), SheetView::Nothing, "por subir");
    resolve_sheet_textures(&mut w, &BTreeMap::new(), &mut |_| Some(5));
    show(&mut w, e, FrameRef::Empty);
    assert_eq!(sheet_view(&w, e, &sprite), SheetView::Nothing, "vazio");
}

#[test]
fn trocar_a_imagem_de_uma_folha_resolve_de_novo_e_sem_folhas_o_cache_sai() {
    let (mut w, e, _) = two_sheets();
    resolve_sheet_textures(&mut w, &BTreeMap::new(), &mut |_| Some(5));
    w.get_mut::<SpriteSheets>(e).unwrap().0[0].asset = img(9);
    let mut subidas = Vec::new();
    resolve_sheet_textures(&mut w, &BTreeMap::new(), &mut |id| {
        subidas.push(id);
        Some(6)
    });
    assert_eq!(subidas, [img(9)]);
    w.entity_mut(e).remove::<SpriteSheets>();
    resolve_sheet_textures(&mut w, &BTreeMap::new(), &mut |_| None);
    assert!(w.get::<SheetTextures>(e).is_none());
}
