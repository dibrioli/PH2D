//! O canal entre a família e o painel: o que ela sabe, o que ele pede.

use std::cell::RefCell;
use std::sync::Arc;

use ph2d_pixel_core::Forma;
use ph2d_pixel_tiles::desenho::Modo;
pub use ph2d_pixel_tiles::mao::{Alvo, Conjunto, Mao};

/// O que o painel mostra do tileset da camada activa.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Contexto {
    pub mao: Option<Mao>,
    pub tile_w: u16,
    pub tile_h: u16,
    /// RGBA de cada tile, na ordem; o 0 é o vazio.
    pub imagens: Arc<Vec<Vec<u8>>>,
    /// Os tilesets do sprite: qualquer camada pode usá-los.
    pub conjuntos: Vec<Conjunto>,
    /// O que a janela de criar ou converter traz escolhido.
    pub padrao: Option<u32>,
    /// A forma da grelha e os terrenos do tileset activo.
    pub forma: Forma,
    pub terrenos: Vec<String>,
    /// O user data do tileset activo (P12 U), na vista do cartão.
    pub user: Option<ph2d_editor_core::panel::cards::user_data::UserDataView>,
}

/// A janela de criar ou converter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tela {
    #[default]
    Tileset,
    NovaCamada,
    Converter,
}

/// Um eixo de flip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eixo {
    X,
    Y,
    D,
}

/// Um gesto do painel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pedido {
    Alvo(Alvo),
    Modo(Modo),
    Tile(u32),
    Flip(Eixo),
    NovoTile,
    ApagaTile,
    Terreno(u32),
    NovoTerreno,
    ApagaTerreno,
    /// As 47 formas do terreno da mão, numa camada para pintar.
    Gabarito,
    /// A forma da grelha do tileset activo.
    Forma(Forma),
    /// `conjunto`: um tileset que já existe; `None`, um novo de `w × h` nesta forma.
    NovaCamada {
        conjunto: Option<u32>,
        w: u16,
        h: u16,
        forma: Forma,
    },
    Converte {
        conjunto: Option<u32>,
        w: u16,
        h: u16,
        flips: bool,
    },
    /// A janela fechou sem criar nada.
    Fechou,
    /// Um gesto do cartão User Data do tileset activo: um passo «User Data».
    UserData(ph2d_editor_core::panel::cards::user_data::CardEdit),
}

#[derive(Default)]
struct Canal {
    contexto: Contexto,
    abrir: Option<Tela>,
    pedidos: Vec<Pedido>,
}

thread_local! {
    static CANAL: RefCell<Canal> = RefCell::new(Canal::default());
}

fn com<R>(f: impl FnOnce(&mut Canal) -> R) -> R {
    CANAL.with(|c| f(&mut c.borrow_mut()))
}

pub fn publish_contexto(x: Contexto) {
    com(|c| c.contexto = x);
}

#[must_use]
pub fn contexto() -> Contexto {
    com(|c| c.contexto.clone())
}

/// A família pede esta janela.
pub fn abrir(t: Tela) {
    com(|c| c.abrir = Some(t));
}

pub(crate) fn take_abrir() -> Option<Tela> {
    com(|c| c.abrir.take())
}

pub fn pede(p: Pedido) {
    com(|c| c.pedidos.push(p));
}

/// Toma (e limpa) os gestos deste quadro.
#[must_use]
pub fn take_pedidos() -> Vec<Pedido> {
    com(|c| std::mem::take(&mut c.pedidos))
}
