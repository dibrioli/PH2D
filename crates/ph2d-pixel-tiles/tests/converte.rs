//! Convert To: o composto igual antes e depois, e desfazer ao byte.

use ph2d_pixel_core::frames::{NewFrame, new_frame};
use ph2d_pixel_core::{
    Background, Cel, ColorMode, FrameId, History, LayerId, LayerKind, NewSprite, Op, Patch, Rect,
    Sprite, TileRef, compose_rgba, derived_is_consistent,
};
use ph2d_pixel_tiles::converte::{para_imagem, para_tilemap, para_tilemap_em};

/// Imagem 16×16 com tiles repetidos de 4×4 e um pedaço fora da grelha; `mode` à escolha.
fn base(mode: ColorMode) -> (Sprite, History, LayerId, FrameId) {
    let mut s = Sprite::new(NewSprite {
        width: 16,
        height: 16,
        mode,
        background: Background::Transparent,
    })
    .unwrap();
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let bpp = s.mode().bpp();
    let rect = Rect::new(2, 1, 12, 12);
    let mut pixels = Vec::new();
    for y in rect.y..rect.y + 12 {
        for x in rect.x..rect.x + 12 {
            // Periódico de 4 em 4: as células do miolo repetem-se.
            let v = ((x & 3) + (y & 3) * 4 + 1) as u8;
            pixels.extend_from_slice(&[v, v, v, 255][4 - bpp.min(4)..]);
            if bpp == 2 {
                let n = pixels.len();
                pixels[n - 2..].copy_from_slice(&[v, 255]);
            }
        }
    }
    let mut h = History::default();
    let p = Patch {
        layer: l,
        frame: f,
        bbox: Some(rect),
        rect,
        pixels,
    };
    h.commit(&mut s, Op::Patch(p)).unwrap();
    (s, h, l, f)
}

fn compostos(s: &Sprite) -> Vec<Vec<u8>> {
    s.frames()
        .iter()
        .map(|f| {
            let mut px = Vec::new();
            compose_rgba(s, f.id, None, &mut px);
            px
        })
        .collect()
}

#[test]
fn imagem_para_tilemap_e_volta_em_todos_os_modos_de_cor() {
    for mode in [ColorMode::Rgba, ColorMode::Gray, ColorMode::Indexed] {
        let (mut s, mut h, l, _) = base(mode);
        let (antes, composto) = (s.clone(), compostos(&s));
        let (op, set) = para_tilemap(&mut s, l, 4, 4, false).unwrap();
        // Só o contador de ids anda ao preparar; o documento em si, não.
        let reservado = s.clone();
        h.commit(&mut s, op).unwrap();
        assert!(
            matches!(s.layer(l).unwrap().kind, LayerKind::Tilemap { .. }),
            "{mode:?}"
        );
        assert!(derived_is_consistent(&s), "{mode:?}");
        assert_eq!(compostos(&s), composto, "{mode:?}");
        let g = s
            .entry((l, s.frames()[0].id))
            .unwrap()
            .tiles
            .clone()
            .unwrap();
        assert_eq!((g.x, g.y, g.w, g.h), (0, 0, 4, 4));
        assert!(
            s.tileset(set).unwrap().tiles.len() < 16,
            "{mode:?}: deduplicou"
        );
        // Desfazer ao byte.
        let depois = s.clone();
        assert!(h.undo(&mut s));
        assert_eq!(s, reservado, "{mode:?}");
        assert_eq!(s.layers(), antes.layers());
        assert_eq!(
            s.entries().collect::<Vec<_>>(),
            antes.entries().collect::<Vec<_>>()
        );
        assert!(h.redo(&mut s));
        assert_eq!(s, depois);
        // E de volta.
        let op = para_imagem(&s, l).unwrap();
        h.commit(&mut s, op).unwrap();
        assert_eq!(s.layer(l).unwrap().kind, LayerKind::Image, "{mode:?}");
        assert_eq!(compostos(&s), composto, "{mode:?}");
        assert!(s.entries().all(|(_, c)| c.tiles.is_none()), "{mode:?}");
        assert!(h.undo(&mut s));
        assert_eq!(s, depois, "{mode:?}");
    }
}

/// O tile 0 é só o vazio; cópias exatas dão um só tile.
#[test]
fn a_deduplicacao_e_exata_por_omissao() {
    let (mut s, mut h, l, f) = base(ColorMode::Rgba);
    let (op, set) = para_tilemap(&mut s, l, 4, 4, false).unwrap();
    h.commit(&mut s, op).unwrap();
    let t = s.tileset(set).unwrap();
    let g = s.entry((l, f)).unwrap().tiles.clone().unwrap();
    let centro = g.tiles[(g.w + 1) as usize];
    assert_eq!(g.tiles[(g.w + 2) as usize], centro, "o miolo repete-se");
    assert!(
        g.tiles.iter().all(|r| r.0 & !TileRef::ID_MASK == 0),
        "sem flips"
    );
    assert_eq!(t.tiles[0], vec![0; 4 * 4 * 4]);
    let mut unicos = t.tiles.clone();
    unicos.sort();
    unicos.dedup();
    assert_eq!(unicos.len(), t.tiles.len(), "nenhum repetido");
}

/// Com flips pedidos e tile quadrado, um tile espelhado reaproveita o outro; sem o pedido, não.
#[test]
fn flips_so_a_pedido_e_com_tile_quadrado() {
    let monta = || {
        let mut s = Sprite::new(NewSprite {
            width: 8,
            height: 4,
            mode: ColorMode::Rgba,
            background: Background::Transparent,
        })
        .unwrap();
        let (l, f) = (s.layers()[0].id, s.frames()[0].id);
        let rect = Rect::new(0, 0, 8, 4);
        let mut pixels = Vec::new();
        for y in 0..4 {
            for x in 0..8 {
                // A segunda célula é a primeira ao espelho.
                let sx: u8 = if x < 4 { x } else { 7 - x };
                pixels.extend_from_slice(&[sx + y * 4 + 1, 9, 9, 255]);
            }
        }
        let p = Patch {
            layer: l,
            frame: f,
            bbox: Some(rect),
            rect,
            pixels,
        };
        let mut h = History::default();
        h.commit(&mut s, Op::Patch(p)).unwrap();
        (s, h, l, f)
    };
    let (mut s, mut h, l, f) = monta();
    let composto = compostos(&s);
    let (op, set) = para_tilemap(&mut s, l, 4, 4, true).unwrap();
    h.commit(&mut s, op).unwrap();
    assert_eq!(s.tileset(set).unwrap().tiles.len(), 2, "o vazio e um só");
    let g = s.entry((l, f)).unwrap().tiles.clone().unwrap();
    assert!(g.tiles[1].flip_x() && !g.tiles[1].flip_y() && !g.tiles[1].flip_d());
    assert_eq!(compostos(&s), composto);
    assert!(derived_is_consistent(&s));
    // Sem o pedido: dois tiles.
    let (mut s, mut h, l, _) = monta();
    let (op, set) = para_tilemap(&mut s, l, 4, 4, false).unwrap();
    h.commit(&mut s, op).unwrap();
    assert_eq!(s.tileset(set).unwrap().tiles.len(), 3);
    // Tile não quadrado: o pedido não vale.
    let (mut s, mut h, l, _) = monta();
    let (op, set) = para_tilemap(&mut s, l, 4, 2, true).unwrap();
    h.commit(&mut s, op).unwrap();
    assert!(s.tileset(set).unwrap().tiles.len() > 2);
    assert_eq!(compostos(&s), composto);
    assert!(derived_is_consistent(&s));
}

#[test]
fn os_ligados_continuam_ligados_e_a_opacidade_fica() {
    let (mut s, mut h, l, f) = base(ColorMode::Rgba);
    let (op, f2) = new_frame(&mut s, f, NewFrame::Linked).unwrap();
    h.commit(&mut s, op).unwrap();
    let (op, f3) = new_frame(&mut s, f, NewFrame::Copy).unwrap();
    h.commit(&mut s, op).unwrap();
    let op = ph2d_pixel_core::frames::set_cel(&s, (l, f3), Some(99), Some(1)).unwrap();
    h.commit(&mut s, op).unwrap();
    let composto = compostos(&s);
    let (op, _) = para_tilemap(&mut s, l, 4, 4, false).unwrap();
    h.commit(&mut s, op).unwrap();
    assert_eq!(s.entry((l, f2)).unwrap().link, Some(f));
    let c: &Cel = s.entry((l, f3)).unwrap();
    assert_eq!((c.opacity, c.z), (99, 1));
    assert_eq!(compostos(&s), composto);
    assert!(derived_is_consistent(&s));
    let op = para_imagem(&s, l).unwrap();
    h.commit(&mut s, op).unwrap();
    assert_eq!(s.entry((l, f2)).unwrap().link, Some(f));
    assert_eq!(compostos(&s), composto);
}

#[test]
fn quem_nao_se_converte_recusa() {
    let (mut s, _, l, _) = base(ColorMode::Rgba);
    assert!(para_tilemap(&mut s, l, 0, 4, false).is_none());
    assert!(para_imagem(&s, l).is_none(), "já é imagem");
    let antes = s.clone();
    let (g, _) = {
        let id = antes.layers()[0].id;
        ph2d_pixel_core::layers::new_group(&mut s, id).unwrap()
    };
    ph2d_pixel_core::apply(&mut s, g).unwrap();
    let grupo = s.layers().iter().find(|x| x.is_group()).unwrap().id;
    assert!(para_tilemap(&mut s, grupo, 4, 4, false).is_none());
}

/// Uma camada Background não vira tilemap: ela é a opaca do fundo.
#[test]
fn a_background_recusa() {
    let mut s = Sprite::new(NewSprite {
        width: 8,
        height: 8,
        mode: ColorMode::Rgba,
        background: Background::White,
    })
    .unwrap();
    let l = s.layers()[0].id;
    assert!(s.layer(l).unwrap().background);
    assert!(para_tilemap(&mut s, l, 4, 4, false).is_none());
}

/// Sobre um tileset que já existe: o pedaço igual a um tile usa-o, o resto acrescenta-se, e um tile
/// do gabarito nunca serve (quem entra nele vira terreno).
#[test]
fn o_tileset_existente_reaproveita_mas_nao_entra_no_gabarito() {
    let (mut s, mut h, l, f) = base(ColorMode::Rgba);
    let (op, _, set) = ph2d_pixel_core::tiles::new_tilemap_layer(&mut s, l, 4, 4).unwrap();
    h.commit(&mut s, op).unwrap();
    let px = s.read_rect((l, f), Rect::new(4, 4, 4, 4));
    let terreno = ph2d_pixel_tiles::terreno::novo_terreno(&s, set, "T").unwrap();
    h.commit(&mut s, terreno).unwrap();
    let primeiro = s.tileset(set).unwrap().terrains[0].first;
    let no_gabarito = ph2d_pixel_core::TilesSet {
        tiles: vec![(set, primeiro, px.clone())],
        ..Default::default()
    };
    h.commit(&mut s, Op::Tiles(Box::new(no_gabarito))).unwrap();
    let a_rever = s.tileset(set).unwrap().tiles.len() as u32;
    let antes = compostos(&s);
    let op = para_tilemap_em(&s, l, set, false).unwrap();
    h.commit(&mut s, op).unwrap();
    assert_eq!(compostos(&s), antes);
    assert!(derived_is_consistent(&s));
    let g = s.entry((l, f)).unwrap().tiles.as_deref().unwrap();
    let ids: Vec<u32> = g.tiles.iter().map(|t| t.id()).collect();
    assert!(ids.iter().all(|&i| i == 0 || i >= a_rever), "{ids:?}");
    let com_repetidos = ids.iter().filter(|&&i| i != 0).count();
    let novos = s.tileset(set).unwrap().tiles.len() as u32 - a_rever;
    assert!(
        novos > 0 && (novos as usize) < com_repetidos,
        "dedupe entre si"
    );

    // Sem o gabarito no caminho, o mesmo pedaço reaproveita o tile normal.
    let (mut s2, mut h2, l2, f2) = base(ColorMode::Rgba);
    let (op, _, set2) = ph2d_pixel_core::tiles::new_tilemap_layer(&mut s2, l2, 4, 4).unwrap();
    h2.commit(&mut s2, op).unwrap();
    let px = s2.read_rect((l2, f2), Rect::new(4, 4, 4, 4));
    let mais = ph2d_pixel_core::TilesSet {
        counts: vec![(set2, 2)],
        tiles: vec![(set2, 1, px)],
        ..Default::default()
    };
    h2.commit(&mut s2, Op::Tiles(Box::new(mais))).unwrap();
    let op = para_tilemap_em(&s2, l2, set2, false).unwrap();
    h2.commit(&mut s2, op).unwrap();
    let g = s2.entry((l2, f2)).unwrap().tiles.as_deref().unwrap();
    assert!(g.tiles.iter().any(|t| t.id() == 1));
    assert!(h2.undo(&mut s2));
    assert_eq!(s2.tileset(set2).unwrap().tiles.len(), 2);
}
