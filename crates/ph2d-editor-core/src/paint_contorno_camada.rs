//! ⭐ O contorno da cor da camada de colisão na MOLDURA do elemento (GEF02 onda 9f, dono 07/11: «não gostei do
//! sublinhado. vamos tentar uma outline colorida na moldura»). Uma língua só: os quadros «Collides with», o seletor
//! Layer e o Rename do Inspector, a lista «Layers», as caixinhas «Bumps into» e a lista do Remove da aba.
//!
//! O traço fica DENTRO da caixa (recuo de meia espessura, raio do elemento menos meia espessura): a medida e o
//! alinhamento do elemento não mudam. Desligado (não colide) = a mesma cor a [`LAYER_OUTLINE_OFF_ALPHA`].
//!
//! Cada contorno fica registado pelo `NodeId` do controlo (o último pintado vence): é o que os testes leem.

use super::{resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, LAYER_OUTLINE_OFF_ALPHA, StrokeToken, Theme};
use ph2d_vector::VectorScene;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// O que um teste lê de um contorno pintado.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ContornoDeCamada {
    /// A camada cuja cor se pintou.
    pub camada: usize,
    /// A caixa do ELEMENTO (o traço vive dentro dela).
    pub rect: Rect,
    /// A linha do traço (a caixa recuada de meia espessura).
    pub traco: Rect,
    /// A cor resolvida no tema da pintura, já com a opacidade do estado.
    pub cor: [f32; 4],
    /// `false` = o quadro desligado (mais apagado).
    pub ligado: bool,
}

thread_local! {
    static PINTADOS: RefCell<BTreeMap<ph2d_a11y::NodeId, ContornoDeCamada>> = const { RefCell::new(BTreeMap::new()) };
}

/// A espessura do contorno: `StrokeToken::Thick`.
#[must_use]
pub fn espessura_do_contorno() -> f32 {
    StrokeToken::Thick.px()
}

/// Pinta o contorno da camada `camada` dentro de `caixa` (raio `raio`, o do elemento) e regista-o por `id`.
#[allow(clippy::too_many_arguments)]
pub fn paint_layer_outline(
    scene: &mut VectorScene,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    camada: usize,
    caixa: Rect,
    raio: f32,
    ligado: bool,
) {
    let e = espessura_do_contorno();
    let meio = e * 0.5;
    let traco = Rect::new(
        caixa.x + meio,
        caixa.y + meio,
        (caixa.w - e).max(0.0),
        (caixa.h - e).max(0.0),
    );
    let base = resolve(ColorToken::layer(camada), theme);
    let cor = if ligado {
        base
    } else {
        base.multiply_alpha(LAYER_OUTLINE_OFF_ALPHA)
    };
    // FRAME-RAW-OK: o contorno da camada é cor de DADO (a camada), não a moldura de estado do tema
    stroke_rounded_rect(scene, traco, (raio - meio).max(0.0), e, cor);
    let marca = ContornoDeCamada {
        camada,
        rect: caixa,
        traco,
        cor: cor.components,
        ligado,
    };
    PINTADOS.with(|m| m.borrow_mut().insert(id, marca));
}

/// O último contorno pintado para o controlo `id` (os testes; `None` se nunca se pintou).
#[must_use]
pub fn layer_outline_of(id: ph2d_a11y::NodeId) -> Option<ContornoDeCamada> {
    PINTADOS.with(|m| m.borrow().get(&id).copied())
}

/// Esquece todos (um teste que quer ver só a pintura que vem a seguir).
pub fn forget_layer_outlines() {
    PINTADOS.with(|m| m.borrow_mut().clear());
}
