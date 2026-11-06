//! O canal entre a família e o painel: o que ela sabe, o que ele pede.

use std::cell::RefCell;
use std::sync::Arc;

use ph2d_pixel_tiles::desenho::Modo;
pub use ph2d_pixel_tiles::mao::{Alvo, Mao};

/// O que o painel mostra do tileset da camada activa.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Contexto {
    pub mao: Option<Mao>,
    pub tile_w: u16,
    pub tile_h: u16,
    /// RGBA de cada tile, na ordem; o 0 é o vazio.
    pub imagens: Arc<Vec<Vec<u8>>>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pedido {
    Alvo(Alvo),
    Modo(Modo),
    Tile(u32),
    Flip(Eixo),
    NovoTile,
    ApagaTile,
    NovaCamada {
        w: u16,
        h: u16,
    },
    Converte {
        w: u16,
        h: u16,
        flips: bool,
    },
    /// A janela fechou sem criar nada.
    Fechou,
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
