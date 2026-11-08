//! ⭐ O sublinhado da cor da camada de colisão (GEF02 onda 9e, dono 07/11: «melhor um sublinhado colorido que o
//! pequeno quadrado»). Uma língua só: a fila «Collides with», o seletor Layer e o Rename do Inspector, a lista
//! «Layers» e as caixinhas «Bumps into» da aba pintam a MESMA marca, debaixo do nome e da largura dele.
//!
//! Cada sublinhado fica registado pelo `NodeId` do controlo (o último pintado vence; mapa limitado pelos ids):
//! é o que os testes leem para saber de que camada e onde ele está.

use super::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// O que um teste lê de um sublinhado pintado.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sublinhado {
    /// A camada cuja cor se pintou.
    pub camada: usize,
    /// O traço: x e largura do texto, y logo abaixo da caixa do texto.
    pub rect: Rect,
    /// A cor resolvida no tema da pintura (`ColorToken::layer(camada)`).
    pub cor: [f32; 4],
}

thread_local! {
    static PINTADOS: RefCell<BTreeMap<ph2d_a11y::NodeId, Sublinhado>> = const { RefCell::new(BTreeMap::new()) };
}

/// O traço debaixo de um texto pintado em `(x, topo)` com `fonte` px e `largura` medida: espessura
/// `StrokeToken::Thick`, recuo `Spacing::Xxs` abaixo da caixa do texto (o pé das descendentes).
#[must_use]
pub fn rect_do_sublinhado(x: f32, topo: f32, fonte: f32, largura: f32) -> Rect {
    let alto = StrokeToken::Thick.px();
    Rect::new(
        x,
        topo + fonte + Spacing::Xxs.px() - alto * 0.5,
        largura.max(0.0),
        alto,
    )
}

/// Pinta o sublinhado da camada `camada` debaixo do texto e regista-o por `id`.
#[allow(clippy::too_many_arguments)]
pub fn paint_layer_underline(
    scene: &mut VectorScene,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    camada: usize,
    x: f32,
    topo: f32,
    fonte: f32,
    largura: f32,
) -> Rect {
    let r = rect_do_sublinhado(x, topo, fonte, largura);
    let cor = resolve(ColorToken::layer(camada), theme);
    fill_rounded_rect(scene, r, r.h * 0.5, cor);
    let marca = Sublinhado {
        camada,
        rect: r,
        cor: cor.components,
    };
    PINTADOS.with(|m| m.borrow_mut().insert(id, marca));
    r
}

/// O último sublinhado pintado para o controlo `id` (os testes; `None` se nunca se pintou).
#[must_use]
pub fn layer_underline_of(id: ph2d_a11y::NodeId) -> Option<Sublinhado> {
    PINTADOS.with(|m| m.borrow().get(&id).copied())
}

/// Esquece todos (um teste que quer ver só a pintura que vem a seguir).
pub fn forget_layer_underlines() {
    PINTADOS.with(|m| m.borrow_mut().clear());
}
