//! **A tela dos tilemaps, do lado da família** (`docs/PixelArt/P10_plano.md` §5): liga o painel
//! `ph2d-panel-pixel-tiles` e o gesto às leis de `ph2d-pixel-tiles`.

use std::sync::Arc;

use ph2d_editor_core::pixel_panel::{PixelLayerAction as A, PixelRequest};
use ph2d_editor_core::screens::hero::{HeroScreen, PANEL_PIXEL_TILES};
use ph2d_panel_pixel_tiles::canal::{self, Alvo, Contexto, Eixo, Mao, Pedido, Tela};
use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_core::{LayerId, Op, Sprite, TileRef, Tileset, TilesetId};
use ph2d_pixel_tiles::mao::{Carimbo, imagens, tileset_de};
use ph2d_pixel_tiles::mapa::Mapa;
use ph2d_pixel_tiles::{converte, tileset as lei};

use crate::input::KeyAnswer;
use crate::{Doc, PixelApp, Tool};

#[derive(Default)]
pub(crate) struct Estado {
    pub(crate) mao: Mao,
    pub(crate) carimbo: Option<(u64, u32, Carimbo)>,
    janela: bool,
    visto: Option<bool>,
    publicado: Option<(u32, u64, Arc<Vec<Vec<u8>>>)>,
}

/// O tileset da camada activa.
fn activo(doc: &Doc) -> Option<&Tileset> {
    tileset_de(&doc.sprite, doc.layer)
}

fn tem_tilemap(s: &Sprite) -> bool {
    s.layers().iter().any(|l| l.tileset().is_some())
}

/// A recusa de um comando do menu Sprite ou da linha do tempo, com o motivo certo: `None` no `op` e
/// tilemap no sprite. `false`: o motivo é outro.
pub(crate) fn recusou(app: &mut PixelApp, key: u32, op: Option<&Op>, chave: &str) -> bool {
    let tem = app.docs.get(&key).is_some_and(|d| tem_tilemap(&d.sprite));
    if op.is_none() && tem {
        app.recusa(chave);
    }
    op.is_none() && tem
}

/// O motivo de recusar o traço: em tilemap só as ferramentas que passam pelo funil desenham.
pub(crate) fn recusa_traco(doc: &Doc, tool: Tool) -> Option<&'static str> {
    let m = crate::camadas::recusa_pintar(doc)?;
    let funil = !matches!(
        tool,
        Tool::Gradient | Tool::Blur | Tool::Jumble | Tool::Text
    );
    (m != "pixel.layer.refuse.tilemap" || !funil).then_some(m)
}

/// Premir em Draw Tiles. `false`: o gesto não é dele.
pub(crate) fn press(
    app: &mut PixelApp,
    bits: u64,
    key: u32,
    at: (i32, i32),
    tool: Tool,
    secundario: bool,
) -> bool {
    let Some(doc) = app.docs.get(&key) else {
        return false;
    };
    let Some(set) = activo(doc).filter(|_| app.tiles.mao.alvo == Alvo::Tiles) else {
        return false;
    };
    let (layer, frame) = (doc.layer, doc.frame);
    let m = Mapa::de(&doc.sprite, layer, frame, set);
    let (c, r) = m.celula(at.0, at.1);
    if tool == Tool::Eyedropper {
        app.tiles.mao.pega(m.get(c, r));
        return true;
    }
    if !matches!(tool, Tool::Pencil | Tool::Eraser) {
        app.recusa("pixel.tiles.only_pencil");
        return true;
    }
    if let Some(motivo) = recusa_traco(doc, tool) {
        app.recusa(motivo);
        return true;
    }
    let tile = if tool == Tool::Eraser || secundario {
        TileRef::EMPTY
    } else {
        app.tiles.mao.tile_ref()
    };
    let g = Carimbo {
        layer,
        frame,
        tile,
        celulas: Vec::new(),
        passo: false,
    };
    app.tiles.carimbo = Some((bits, key, g));
    anda(app, at);
    true
}

/// O arrasto chegou ao píxel `at`: reescreve o MESMO passo do desfazer com as células novas.
pub(crate) fn anda(app: &mut PixelApp, at: (i32, i32)) {
    let Some((_, key, g)) = app.tiles.carimbo.as_ref() else {
        return;
    };
    let (key, g) = (*key, g.clone());
    let Some(celulas) = app.docs.get(&key).and_then(|d| g.ate(&d.sprite, at)) else {
        return;
    };
    if g.passo {
        app.undo(key);
    }
    let op = app.docs.get(&key).and_then(|d| g.op(&d.sprite, &celulas));
    let feito = op.is_some_and(|op| app.commit(key, op).is_ok());
    if let Some((_, _, g)) = app.tiles.carimbo.as_mut() {
        g.celulas = celulas;
        g.passo = feito;
    }
}

/// `Esc` a meio do arrasto: o passo sai.
pub(crate) fn cancela(app: &mut PixelApp) -> bool {
    let Some((_, key, g)) = app.tiles.carimbo.take() else {
        return false;
    };
    if g.passo {
        app.undo(key);
    }
    true
}

/// `Space+Tab` e `Space+1/2/3`.
pub(crate) fn tecla(app: &mut PixelApp, f: impl FnOnce(&mut Mao)) -> KeyAnswer {
    if app.chave.and_then(|k| activo(app.docs.get(&k)?)).is_none() {
        return KeyAnswer::Toast(ph2d_i18n::tr("pixel.tiles.not_tilemap").to_owned());
    }
    f(&mut app.tiles.mao);
    KeyAnswer::Quiet
}

/// Os verbos do menu Layer. `false`: o pedido não é daqui.
pub(crate) fn aplica_pedido(app: &mut PixelApp, r: &PixelRequest) -> bool {
    let PixelRequest::LayerAction(a @ (A::NewTilemap | A::ConvertToTilemap | A::ConvertToImage)) =
        r
    else {
        return false;
    };
    let Some(key) = app.chave else { return true };
    if *a == A::ConvertToImage {
        let op = app
            .docs
            .get(&key)
            .and_then(|d| converte::para_imagem(&d.sprite, d.layer));
        app.passo(key, op);
    } else {
        canal::abrir(if *a == A::NewTilemap {
            Tela::NovaCamada
        } else {
            Tela::Converter
        });
        app.tiles.janela = true;
    }
    true
}

fn cria(app: &mut PixelApp, key: u32, w: u16, h: u16, converter: Option<bool>) {
    let Some(d) = app.docs.get_mut(&key) else {
        return;
    };
    let layer = d.layer;
    let feito: Option<(Op, LayerId)> = match converter {
        Some(flips) => {
            converte::para_tilemap(&mut d.sprite, layer, w, h, flips).map(|(op, _)| (op, layer))
        }
        None => new_tilemap_layer(&mut d.sprite, layer, w, h).map(|(op, l, _)| (op, l)),
    };
    let Some((op, nova)) = feito else {
        return app.recusa("pixel.tiles.refuse.convert");
    };
    if app.commit(key, op).is_ok()
        && let Some(d) = app.docs.get_mut(&key)
    {
        d.layer = nova;
    }
}

fn pedido(app: &mut PixelApp, key: u32, p: Pedido) {
    let set = app.docs.get(&key).and_then(activo).map(|t| t.id);
    let mao = &mut app.tiles.mao;
    match p {
        Pedido::Alvo(a) => mao.alvo = a,
        Pedido::Modo(m) => mao.modo = m,
        Pedido::Tile(i) => mao.tile = i,
        Pedido::Flip(Eixo::X) => mao.flip_x = !mao.flip_x,
        Pedido::Flip(Eixo::Y) => mao.flip_y = !mao.flip_y,
        Pedido::Flip(Eixo::D) => mao.flip_d = !mao.flip_d,
        Pedido::NovoTile => novo(app, key, set),
        Pedido::ApagaTile => apaga(app, key, set),
        Pedido::NovaCamada { w, h } => cria(app, key, w, h, None),
        Pedido::Converte { w, h, flips } => cria(app, key, w, h, Some(flips)),
        Pedido::Fechou => {}
    }
}

fn novo(app: &mut PixelApp, key: u32, set: Option<TilesetId>) {
    let Some((op, n)) = app.docs.get(&key).and_then(|d| {
        let id = set?;
        Some((
            lei::novo_tile(&d.sprite, id)?,
            d.sprite.tileset(id)?.tiles.len(),
        ))
    }) else {
        return;
    };
    if app.commit(key, op).is_ok() {
        app.tiles.mao.tile = n as u32;
    }
}

fn apaga(app: &mut PixelApp, key: u32, set: Option<TilesetId>) {
    let tile = app.tiles.mao.tile;
    let op = app
        .docs
        .get(&key)
        .and_then(|d| lei::apaga_tile(&d.sprite, set?, tile));
    if op.is_none() {
        app.recusa("pixel.tiles.refuse.delete");
    }
    app.passo(key, op);
}

/// O quadro: os pedidos do painel, o que ele mostra e quando abre.
pub(crate) fn frame(app: &mut PixelApp, hero: &mut HeroScreen) {
    let pedidos = canal::take_pedidos();
    for p in pedidos {
        if matches!(
            p,
            Pedido::Fechou | Pedido::NovaCamada { .. } | Pedido::Converte { .. }
        ) {
            app.tiles.janela = false;
            app.tiles.visto = None;
        }
        if let Some(key) = app.chave {
            app.assenta();
            crate::viva::assenta(app);
            pedido(app, key, p);
        }
    }
    let doc = app.chave.and_then(|k| Some((k, app.docs.get(&k)?)));
    let set = doc.and_then(|(k, d)| Some((k, d, activo(d)?)));
    let t = &mut app.tiles;
    let mut x = Contexto::default();
    if let Some((k, d, set)) = set {
        t.mao.tile = t.mao.tile.min(set.tiles.len() as u32 - 1);
        t.mao.flip_d &= set.tile_w == set.tile_h;
        let novo = !t
            .publicado
            .as_ref()
            .is_some_and(|p| (p.0, p.1) == (k, d.stamp));
        if novo {
            t.publicado = Some((k, d.stamp, Arc::new(imagens(&d.sprite, set))));
        }
        x.mao = Some(t.mao);
        (x.tile_w, x.tile_h) = (set.tile_w, set.tile_h);
        x.imagens = t
            .publicado
            .as_ref()
            .map(|p| p.2.clone())
            .unwrap_or_default();
    }
    let tem = x.mao.is_some();
    canal::publish_contexto(x);
    if t.janela {
        hero.panel_visibility.insert(PANEL_PIXEL_TILES, true);
    } else if t.visto != Some(tem) {
        hero.panel_visibility.insert(PANEL_PIXEL_TILES, tem);
        t.visto = Some(tem);
    }
}

/// A grelha da camada activa e, em Draw Tiles, a célula sob o cursor.
pub(crate) fn desenha_grelha(
    app: &PixelApp,
    key: u32,
    bits: u64,
    (w, h): (u32, u32),
    m: ph2d_vector::Affine,
    (linha, minimo): (f64, f64),
    (grelha, acento): (ph2d_vector::Color, ph2d_vector::Color),
    scene: &mut ph2d_vector::VectorScene,
) {
    let Some(doc) = app.docs.get(&key) else {
        return;
    };
    let Some(set) = activo(doc) else { return };
    let (tw, th) = (u32::from(set.tile_w), u32::from(set.tile_h));
    let mapa = Mapa::de(&doc.sprite, doc.layer, doc.frame, set);
    let (fx, fy) = mapa.origem(0, 0);
    if ph2d_pixel_render::px_per_cell(m) * f64::from(tw.min(th)) >= minimo {
        ph2d_pixel_render::draw_tile_grid(scene, (w, h), (tw, th), (fx, fy), m, grelha, linha);
    }
    if app.tiles.mao.alvo == Alvo::Tiles
        && let Some((hb, cell)) = app.hover
        && hb == bits
    {
        let (c, r) = mapa.celula(cell.0, cell.1);
        let (x, y) = mapa.origem(c, r);
        let em = m * ph2d_vector::Affine::translate((f64::from(x), f64::from(y)));
        ph2d_pixel_render::draw_border(scene, tw, th, em, acento, linha);
    }
}
