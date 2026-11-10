//! **A tela dos tilemaps, do lado da família** (`docs/PixelArt/P10_plano.md` §5): liga o painel
//! `ph2d-panel-pixel-tiles` e o gesto às leis de `ph2d-pixel-tiles`.

use std::sync::Arc;

use ph2d_editor_core::pixel_panel::{PixelLayerAction as A, PixelRequest};
use ph2d_editor_core::screens::hero::{HeroScreen, JanelaDaMao, PANEL_PIXEL_TILES};
use ph2d_panel_pixel_tiles::canal::{self, Alvo, Contexto, Mao, Pedido, Tela};
use ph2d_panel_pixel_tiles::familia;
use ph2d_pixel_core::{FrameId, Onion, Op, Sprite, Stroke, Tileset, TilesetId, apply};
use ph2d_pixel_tiles::converte;
use ph2d_pixel_tiles::mao::{self, Carimbo, conjuntos, imagens, op_do_traco, tileset_de};
use ph2d_pixel_tiles::mapa::Mapa;

use crate::input::KeyAnswer;
use crate::{Doc, PixelApp, Tool};

/// As imagens publicadas ao painel, e de que documento, tileset e versão são.
type Publicado = (u32, TilesetId, u64, Arc<Vec<Vec<u8>>>);

#[derive(Default)]
pub(crate) struct Estado {
    pub(crate) mao: Mao,
    pub(crate) carimbo: Option<(u64, u32, Carimbo)>,
    janela: bool,
    visto: JanelaDaMao,
    publicado: Option<Publicado>,
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
        Tool::Gradient | Tool::Blur | Tool::Jumble | Tool::Push | Tool::Text
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
    let Some(set) = activo(doc).filter(|_| app.tiles.mao.alvo != Alvo::Pixels) else {
        return false;
    };
    let (layer, frame) = (doc.layer, doc.frame);
    let m = Mapa::de(&doc.sprite, layer, frame, set);
    let (c, r) = m.celula(at.0, at.1);
    if tool == Tool::Eyedropper {
        mao::pega(&mut app.tiles.mao, set, m.get(c, r));
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
    let marca = match mao::marca_de(&app.tiles.mao, set, tool == Tool::Eraser || secundario) {
        Ok(marca) => marca,
        Err(motivo) => {
            app.recusa(motivo);
            return true;
        }
    };
    let g = Carimbo {
        layer,
        frame,
        marca,
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
    let feito = app.passo_nomeado(key, op, "pixel.history.tiles");
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
        app.passo_nomeado(key, op, "pixel.history.convert_to_image");
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

fn pedido(app: &mut PixelApp, key: u32, p: Pedido) {
    let fg = app.fg;
    let Some(d) = app.docs.get_mut(&key) else {
        return;
    };
    let onde = (d.layer, d.frame);
    if let Pedido::UserData(e) = &p {
        let set = d.sprite.layer(d.layer).and_then(|l| l.tileset());
        let o = set.map(|t| ph2d_editor_core::pixel_panel::PixelOwner::Tileset(t.0));
        let op = o.and_then(|o| ph2d_pixel_ponte::user::op(&d.sprite, o, e));
        app.passo_nomeado(key, op, "pixel.history.user_data");
        return;
    }
    let cor = d.sprite.value_of_rgba(fg);
    let nome = nome_tiles(&p);
    let r = familia::responde(p, &mut d.sprite, onde, &mut app.tiles.mao, cor);
    if let Some(motivo) = r.recusa {
        app.recusa(motivo);
    }
    if app.passo_nomeado(key, r.op, nome) {
        let mao = &mut app.tiles.mao;
        mao.tile = r.tile.unwrap_or(mao.tile);
        mao.terreno = r.terreno.unwrap_or(mao.terreno);
        if let (Some(l), Some(d)) = (r.camada, app.docs.get_mut(&key)) {
            d.layer = l;
        }
    }
}

/// Compõe a imagem do canvas. O traço em curso numa camada de tilemap mostra-se como o RESULTADO:
/// o mesmo op que largar grava, aplicado só enquanto compõe; o cartão acompanha.
pub(crate) fn compoe(
    app: &mut PixelApp,
    key: u32,
    quadro: FrameId,
    st: Option<&Stroke>,
    onion: Option<&Onion>,
) {
    let modo = app.tiles.mao.modo;
    let Some(d) = app.docs.get_mut(&key) else {
        return;
    };
    let em_tile = st.is_some_and(|st| tileset_de(&d.sprite, st.layer).is_some());
    let inverso = st
        .filter(|_| em_tile && !d.view.is_current(quadro, d.stamp, onion))
        .and_then(|st| op_do_traco(st, &d.sprite, modo))
        .and_then(|op| apply(&mut d.sprite, op).ok());
    let por_cima = st.filter(|_| !em_tile);
    d.view
        .refresh_with(&d.sprite, quadro, por_cima, d.stamp, onion);
    let Some(inverso) = inverso else { return };
    if let Some(set) = activo(d) {
        let img = Arc::new(imagens(&d.sprite, set));
        app.tiles.publicado = Some((key, set.id, d.stamp, img));
    }
    let _ = apply(&mut d.sprite, inverso);
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
            app.tiles.visto.esquece();
        }
        if let Some(key) = app.chave {
            app.assenta();
            crate::viva::assenta(app);
            pedido(app, key, p);
        }
    }
    let doc = app.chave.and_then(|k| Some((k, app.docs.get(&k)?)));
    let set = doc.and_then(|(k, d)| Some((k, d, activo(d)?)));
    let em_traco = app.gesture.is_some();
    let t = &mut app.tiles;
    let mut x = Contexto::default();
    if let Some((_, d)) = doc {
        x.com_sprite = true;
        (x.conjuntos, x.padrao) = conjuntos(&d.sprite, d.layer);
    }
    if let Some((k, d, set)) = set {
        familia::acerta_mao(&mut t.mao, set);
        // A meio do traço quem publica é a pré-visualização: o documento ainda não o tem.
        let dele = |p: &Publicado| (p.0, p.1) == (k, set.id);
        let novo = !t
            .publicado
            .as_ref()
            .is_some_and(|p| dele(p) && (em_traco || p.2 == d.stamp));
        if novo {
            t.publicado = Some((k, set.id, d.stamp, Arc::new(imagens(&d.sprite, set))));
        }
        x.mao = Some(t.mao);
        (x.tile_w, x.tile_h, x.forma) = (set.tile_w, set.tile_h, set.forma);
        x.terrenos = set.terrains.iter().map(|k| k.name.clone()).collect();
        x.user = Some(ph2d_pixel_ponte::user::vista(&set.user));
        x.imagens = t
            .publicado
            .as_ref()
            .map(|p| p.3.clone())
            .unwrap_or_default();
    }
    let tem = x.mao.is_some();
    canal::publish_contexto(x);
    if t.janela {
        t.visto.abre_sozinha(hero, PANEL_PIXEL_TILES);
    } else {
        t.visto.segue(hero, PANEL_PIXEL_TILES, tem);
    }
}

/// A célula a contornar: a sob o cursor, em Draw Tiles.
pub(crate) fn realce(app: &PixelApp, bits: u64) -> Option<(i32, i32)> {
    let tiles = app.tiles.mao.alvo != Alvo::Pixels;
    app.hover.filter(|h| tiles && h.0 == bits).map(|h| h.1)
}

/// O nome do passo de um pedido da janela dos tiles.
const fn nome_tiles(p: &Pedido) -> &'static str {
    match p {
        Pedido::NovoTile => "pixel.history.new_tile",
        Pedido::ApagaTile => "pixel.history.delete_tile",
        Pedido::NovoTerreno => "pixel.history.new_terrain",
        Pedido::ApagaTerreno => "pixel.history.delete_terrain",
        Pedido::NovaCamada { .. } => "pixel.keys.layer.new_tilemap",
        Pedido::Converte { .. } => "pixel.history.convert_to_tilemap",
        _ => "pixel.history.tiles",
    }
}
