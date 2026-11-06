//! Com a camada de tilemap ativa, cada comando que recusa chega ao chamador sem pânico, e o
//! documento fica igual. É a lista que a tela da P10b deixa cinzenta.

use ph2d_app_pixel::PixelApp;
use ph2d_editor_core::pixel_panel::{
    PixelDialogChoice as D, PixelEditAction as E, PixelLayerAction as L, PixelRequest as R,
    PixelSizeMethod, PixelSpriteAction as S,
};
use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_core::{
    Background, Cel, ColorMode, NewSprite, Op, TileGrid, TileRef, TilesSet, derived_is_consistent,
    selection,
};

fn app_com_tilemap() -> (PixelApp, u32) {
    let mut app = PixelApp::default();
    let key = app
        .create(NewSprite {
            width: 32,
            height: 32,
            mode: ColorMode::Rgba,
            background: Background::Transparent,
        })
        .unwrap();
    let doc = app.foca(key).unwrap();
    let (acima, f) = (doc.sprite.layers()[0].id, doc.frame);
    let (op, l, set) = new_tilemap_layer(&mut doc.sprite, acima, 4, 4).unwrap();
    doc.history.commit(&mut doc.sprite, op).unwrap();
    let tile: Vec<u8> = (0..16_u8).flat_map(|i| [i + 1, 5, 5, 255]).collect();
    let g = TileGrid {
        x: 4,
        y: 4,
        w: 1,
        h: 1,
        tiles: vec![TileRef::new(1, false, false, false)],
    };
    let t = TilesSet {
        counts: vec![(set, 2)],
        tiles: vec![(set, 1, tile)],
        cels: vec![((l, f), Some(Cel::tilemap(g)))],
        ..TilesSet::default()
    };
    doc.history
        .commit(&mut doc.sprite, Op::Tiles(Box::new(t)))
        .unwrap();
    doc.layer = l;
    (app, key)
}

/// O que se pede, com e sem seleção: a tilemap fica intacta e o mapa continua a mandar.
#[test]
fn cada_comando_que_recusa_deixa_o_documento_igual() {
    let edits = [
        E::FlipH,
        E::FlipV,
        E::Rotate180,
        E::Rotate90Cw,
        E::Rotate90Ccw,
        E::ShiftLeft,
        E::ShiftRight,
        E::ShiftUp,
        E::ShiftDown,
        E::Clear,
        E::Cut,
        E::Fill,
        E::Stroke,
        E::Transform,
        E::Paste,
    ];
    let menus = [
        S::RotateCanvas180,
        S::RotateCanvas90Cw,
        S::RotateCanvas90Ccw,
        S::FlipCanvasH,
        S::FlipCanvasV,
    ];
    let camadas = [L::MergeDown, L::ViaCut];
    let canais = Default::default();
    let dialogos = [
        D::SpriteSize {
            width: 64,
            height: 64,
            method: PixelSizeMethod::default(),
        },
        D::CanvasSize {
            left: 1,
            top: 0,
            right: 0,
            bottom: 0,
        },
        D::Invert {
            channels: canais,
            all: false,
        },
        D::BrightnessContrast {
            brightness: 50,
            contrast: 0,
            channels: canais,
            all: false,
        },
    ];
    for com_selecao in [false, true] {
        let (mut app, key) = app_com_tilemap();
        if com_selecao {
            let doc = app.foca(key).unwrap();
            let op = selection::select_all(&doc.sprite).unwrap();
            doc.history.commit(&mut doc.sprite, op).unwrap();
        }
        // A prancheta cheia, para o Paste ser mesmo pedido.
        app.aplica(R::Edit(E::Copy));
        let antes = app.doc(key).unwrap().sprite.clone();
        let pedidos = edits
            .into_iter()
            .map(R::Edit)
            .chain(menus.into_iter().map(R::Sprite))
            .chain(camadas.into_iter().map(R::LayerAction))
            .chain([R::ReplaceColor]);
        for r in pedidos {
            let nome = format!("{r:?}");
            app.aplica(r);
            app.aplica(R::Edit(E::Commit));
            let s = &app.doc(key).unwrap().sprite;
            assert_eq!(*s, antes, "{nome}, seleção {com_selecao}");
            assert!(derived_is_consistent(s));
        }
        for c in dialogos.clone() {
            let nome = format!("{c:?}");
            app.aplica_dialogo(c);
            let s = &app.doc(key).unwrap().sprite;
            assert_eq!(*s, antes, "{nome}, seleção {com_selecao}");
        }
    }
}
