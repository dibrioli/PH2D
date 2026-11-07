//! **A resposta a cada gesto do painel**: a lei sobre o sprite, sem estado da família. A família só
//! grava o op e aplica o resto ([`Resposta`]).

use ph2d_i18n::tr_with;
use ph2d_pixel_core::tiles::{new_tilemap_layer_forma, tilemap_layer_using};
use ph2d_pixel_core::{Forma, FrameId, LayerId, Op, Sprite, Tileset, TilesetId};
use ph2d_pixel_tiles::mao::{Mao, tileset_de};
use ph2d_pixel_tiles::{converte, terreno, tileset as lei};

use crate::canal::{Eixo, Pedido};

/// O que a família faz depois de um gesto.
#[derive(Debug, Default)]
pub struct Resposta {
    pub op: Option<Op>,
    /// A chave da frase de recusa.
    pub recusa: Option<&'static str>,
    /// A camada que fica activa, se o op entrar.
    pub camada: Option<LayerId>,
    /// O tile que fica na mão, se o op entrar.
    pub tile: Option<u32>,
    /// O terreno que fica na mão, se o op entrar.
    pub terreno: Option<u32>,
}

impl Resposta {
    fn de(op: Option<Op>, recusa: &'static str) -> Self {
        Self {
            recusa: op.is_none().then_some(recusa),
            op,
            ..Self::default()
        }
    }
}

fn cria(
    s: &mut Sprite,
    layer: LayerId,
    conjunto: Option<u32>,
    (w, h): (u16, u16),
    forma: Forma,
) -> Resposta {
    let feito = match conjunto.map(TilesetId) {
        Some(set) => tilemap_layer_using(s, layer, set),
        None => new_tilemap_layer_forma(s, layer, w, h, forma).map(|(op, l, _)| (op, l)),
    };
    feito.map_or_else(
        || Resposta::de(None, "pixel.tiles.refuse.convert"),
        |(op, l)| Resposta {
            op: Some(op),
            camada: Some(l),
            ..Resposta::default()
        },
    )
}

fn converte_para(
    s: &mut Sprite,
    layer: LayerId,
    conjunto: Option<u32>,
    (w, h): (u16, u16),
    flips: bool,
) -> Resposta {
    let op = match conjunto.map(TilesetId) {
        Some(set) => converte::para_tilemap_em(s, layer, set, flips),
        None => converte::para_tilemap(s, layer, w, h, flips).map(|(op, _)| op),
    };
    Resposta {
        camada: op.is_some().then_some(layer),
        ..Resposta::de(op, "pixel.tiles.refuse.convert")
    }
}

/// O gabarito do terreno da mão, numa camada nova.
fn gabarito(s: &mut Sprite, layer: LayerId, frame: FrameId, set: TilesetId, m: Mao) -> Resposta {
    let qual = m.terreno as usize;
    let Some(nome) = s.tileset(set).and_then(|t| t.terrains.get(qual)) else {
        return Resposta::de(None, "pixel.tiles.no_terrain");
    };
    let nome = tr_with("pixel.tiles.template_layer", &[("name", &nome.name)]);
    match terreno::gabarito(s, layer, frame, set, qual, &nome) {
        Some((op, l)) => Resposta {
            op: Some(op),
            camada: Some(l),
            ..Resposta::default()
        },
        None => Resposta::de(None, "pixel.tiles.no_terrain"),
    }
}

/// O gesto `p` sobre o sprite; `fg` é o valor da cor da frente no modo dele.
pub fn responde(
    p: Pedido,
    s: &mut Sprite,
    (layer, frame): (LayerId, FrameId),
    mao: &mut Mao,
    fg: u32,
) -> Resposta {
    let set = tileset_de(s, layer).map(|t| t.id);
    match p {
        Pedido::Alvo(a) => mao.alvo = a,
        Pedido::Modo(m) => mao.modo = m,
        Pedido::Tile(i) => mao.tile = i,
        Pedido::Terreno(i) => mao.terreno = i,
        Pedido::Flip(Eixo::X) => mao.flip_x = !mao.flip_x,
        Pedido::Flip(Eixo::Y) => mao.flip_y = !mao.flip_y,
        Pedido::Flip(Eixo::D) => mao.flip_d = !mao.flip_d,
        Pedido::NovaCamada {
            conjunto,
            w,
            h,
            forma,
        } => return cria(s, layer, conjunto, (w, h), forma),
        Pedido::Converte {
            conjunto,
            w,
            h,
            flips,
        } => return converte_para(s, layer, conjunto, (w, h), flips),
        Pedido::Fechou => {}
        Pedido::NovoTile | Pedido::ApagaTile | Pedido::Forma(_) | Pedido::ApagaTerreno => {
            return set.map_or_else(Resposta::default, |id| do_tileset(p, s, id, mao));
        }
        Pedido::NovoTerreno => {
            return set.map_or_else(Resposta::default, |id| novo_terreno(s, id, fg));
        }
        Pedido::Gabarito => {
            return set.map_or_else(Resposta::default, |id| gabarito(s, layer, frame, id, *mao));
        }
    }
    Resposta::default()
}

fn novo_terreno(s: &Sprite, id: TilesetId, fg: u32) -> Resposta {
    let n = s.tileset(id).map_or(0, |t| t.terrains.len());
    let nome = tr_with("pixel.tiles.terrain_name", &[("n", &(n + 1).to_string())]);
    Resposta {
        terreno: Some(n as u32),
        ..Resposta::de(
            terreno::novo_terreno_com_guia(s, id, &nome, fg),
            "pixel.tiles.refuse.terrain_shape",
        )
    }
}

fn do_tileset(p: Pedido, s: &Sprite, id: TilesetId, m: &Mao) -> Resposta {
    match p {
        Pedido::NovoTile => Resposta {
            op: lei::novo_tile(s, id),
            tile: s.tileset(id).map(|t| t.tiles.len() as u32),
            ..Resposta::default()
        },
        Pedido::ApagaTile => {
            Resposta::de(lei::apaga_tile(s, id, m.tile), "pixel.tiles.refuse.delete")
        }
        Pedido::ApagaTerreno => Resposta::de(
            lei::apaga_terreno(s, id, m.terreno as usize),
            "pixel.tiles.no_terrain",
        ),
        Pedido::Forma(f) => {
            if s.tileset(id).is_some_and(|t| t.forma == f) {
                return Resposta::default();
            }
            Resposta::de(
                lei::muda_forma(s, id, f),
                "pixel.tiles.refuse.shape_terrain",
            )
        }
        _ => Resposta::default(),
    }
}

/// A mão depois de o tileset activo ter mudado: nada aponta para fora dele.
pub fn acerta_mao(m: &mut Mao, set: &Tileset) {
    use ph2d_pixel_tiles::mao::Alvo;
    m.tile = m.tile.min(set.tiles.len() as u32 - 1);
    m.flip_d &= set.tile_w == set.tile_h;
    m.terreno = m.terreno.min(set.terrains.len().saturating_sub(1) as u32);
    if m.alvo == Alvo::Terreno && !set.forma.tem_terreno() {
        m.alvo = Alvo::Tiles;
    }
}
