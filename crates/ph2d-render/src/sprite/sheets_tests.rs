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
        offsets: Default::default(),
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

/// ⭐ A caixa do gizmo e o encaixe medem o quad NA TELA: o da folha do quadro, e a folha `0` num vazio.
///
/// *Mutações: `drawn_geometry` ler sempre a folha 0 ⇒ o tamanho é 1 e reprova; a cópia herdar o `offset` da folha 0
/// ⇒ o Pivot resolvido anda e reprova.*
#[test]
fn a_caixa_do_gizmo_e_a_da_folha_na_tela() {
    let (mut w, e, sprite) = two_sheets();
    w.get_mut::<Sprite>(e).unwrap().offset = [8.0, 8.0];
    assert_eq!(drawn_geometry(&w, e).unwrap().0.size, sprite.size);
    show(&mut w, e, FrameRef::Cell { sheet: 1, cell: 3 });
    let (s, g) = drawn_geometry(&w, e).unwrap();
    assert_eq!(s.size, run_sheet().size);
    assert_eq!(
        s.resolve_anchor(32.0),
        run_sheet().anchor,
        "o Pivot da folha, sem o offset da 0"
    );
    let g = g.unwrap();
    assert_eq!((g.hframes, g.vframes, g.frame), (6, 2, 3));
    show(&mut w, e, FrameRef::Empty);
    assert_eq!(drawn_geometry(&w, e).unwrap().0.size, sprite.size);
}

/// ⭐ **A imagem aberta no Painter é a da folha VISÍVEL** (plano GEF 26 F13), seja qual for o quadro na tela, sobre a
/// geometria da folha `0` (a que o ponteiro do Painter mede). *Mutações: sem o ramo da imagem aberta ⇒ o quadro na tela
/// (folha 0) desenha `idle`; Pivot da folha ⇒ a âncora deixa de ser a do `Sprite`.*
#[test]
fn a_imagem_aberta_no_painter_e_a_da_folha_visivel() {
    let (mut w, e, sprite) = two_sheets();
    resolve_sheet_textures(&mut w, &BTreeMap::new(), &mut |_| Some(42));
    w.entity_mut(e).insert(ph2d_ecs::ShownSheet(1));
    assert_eq!(
        sheet_view(&w, e, &sprite),
        SheetView::Own,
        "fechada: o quadro na tela"
    );
    w.insert_resource(ph2d_ecs::ImagemAberta {
        entity: e,
        quad: [1.0, 1.0],
    });
    let SheetView::Other(s, g) = sheet_view(&w, e, &sprite) else {
        panic!("a folha visível");
    };
    assert_eq!(s.source, SpriteSource::Individual { texture_id: 42 });
    assert_eq!((s.anchor, s.size), (sprite.anchor, sprite.size));
    assert_eq!((g.hframes, g.vframes, g.frame), (4, 1, 0));
    // CONTROLO: com a folha 0 visível, aberta, é ela.
    w.entity_mut(e).insert(ph2d_ecs::ShownSheet(0));
    assert_eq!(sheet_view(&w, e, &sprite), SheetView::Own);
}

/// ⭐ Bug do smoke 08/11: a prévia da animação ao lado da `run` aberta no Painter cortava a imagem pela grelha da
/// `idle` (4 × 1), mostrando bonecos e meio. Corta-se pela folha visível, à célula na tela quando é dela.
/// *Mutações: devolver `(sprite, grid)` sempre ⇒ 4 × 1; ignorar a célula na tela ⇒ frame 0.*
#[test]
fn a_previa_ao_lado_da_folha_aberta_corta_se_pela_grelha_dela() {
    let (mut w, e, sprite) = two_sheets();
    let g0 = SpriteGrid::new(4, 1);
    w.entity_mut(e).insert(ph2d_ecs::ShownSheet(1));
    assert_eq!(
        open_preview_cell(&w, e, &sprite, g0),
        (sprite, g0),
        "fechada"
    );
    w.insert_resource(ph2d_ecs::ImagemAberta {
        entity: e,
        quad: [1.0, 1.0],
    });
    let (s, g) = open_preview_cell(&w, e, &sprite, g0);
    assert_eq!(s.size, [1.5, 1.5]);
    assert_eq!((g.hframes, g.vframes, g.frame), (6, 2, 0));
    let mut na_tela = g0;
    (na_tela.sheet, na_tela.frame) = (1, 7);
    w.entity_mut(e).insert(na_tela);
    assert_eq!(open_preview_cell(&w, e, &sprite, g0).1.frame, 7);
    // CONTROLO: a folha 0 aberta é a de sempre.
    w.entity_mut(e).insert(ph2d_ecs::ShownSheet(0));
    assert_eq!(open_preview_cell(&w, e, &sprite, g0), (sprite, g0));
}

/// ⭐ **O acerto do quadro desloca o quad que o render desenha** (GEF 28 A): na folha `0` sem acerto é o `Sprite` de
/// sempre (`Own`, byte a byte); com acerto, uma cópia com o quad deslocado e o resto igual; na folha `k` soma-se ao
/// Pivot dela. O `drawn_geometry` (caixa do gizmo, encaixe) diz o mesmo.
/// *Mutações: o render ignorar o acerto · somá-lo com o sinal trocado · `OwnNudged` mudar a grelha ou o tamanho.*
#[test]
fn o_render_desenha_o_quadro_com_o_acerto_dele() {
    let (mut w, e, sprite) = two_sheets();
    resolve_sheet_textures(&mut w, &BTreeMap::new(), &mut |_| Some(42));
    let d = [0.25, -0.125];
    w.entity_mut(e).insert(ph2d_ecs::CellOffsets {
        cut: [4, 1],
        cells: [(0, d)].into_iter().collect(),
    });
    let SheetView::OwnNudged(s) = sheet_view(&w, e, &sprite) else {
        panic!("a folha 0 com acerto");
    };
    assert_eq!(
        s,
        Sprite {
            anchor: [d[0], d[1]],
            ..sprite
        }
    );
    assert_eq!(drawn_geometry(&w, e).unwrap().0.anchor, s.anchor);
    show(&mut w, e, FrameRef::Cell { sheet: 0, cell: 1 });
    assert_eq!(
        sheet_view(&w, e, &sprite),
        SheetView::Own,
        "a célula 1 não tem acerto"
    );
    w.get_mut::<SpriteSheets>(e).unwrap().0[0]
        .offsets
        .set([6, 2], 7, d);
    show(&mut w, e, FrameRef::Cell { sheet: 1, cell: 7 });
    let SheetView::Other(s, _) = sheet_view(&w, e, &sprite) else {
        panic!("a folha 1");
    };
    let a = run_sheet().anchor;
    assert_eq!(s.anchor, [a[0] + d[0], a[1] + d[1]]);
    assert_eq!(drawn_geometry(&w, e).unwrap().0.anchor, s.anchor);
    assert_eq!(
        sheet_geometry(&w, e, 1, 7).unwrap().0.anchor,
        a,
        "a folha sem acerto"
    );
}
