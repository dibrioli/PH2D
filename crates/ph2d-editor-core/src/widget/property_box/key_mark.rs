//! ⭐⭐ **O LOSANGO DE ANIMAÇÃO na coluna do decorator** (plano 34 §3) — o que fecha o «trabalho a
//! seguir» que o [`super::row::paint_decorator`] declarava: o ponto vira losango quando a linha do
//! tempo conduz o campo.
//!
//! *A shell publica, a folha lê* (o idioma do `published.rs`): a shell sabe que canal da linha do
//! tempo cada campo mostra e em que estado ele está; o pintor da linha só sabe os ids dos campos
//! que desenha. O mapa é o encontro dos dois, publicado uma vez por quadro.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::{Affine, BezPath, Fill, VectorScene};

use crate::paint::resolve;
use crate::zones::Rect;

/// O estado de animação de um campo, como o losango o mostra. A AUSÊNCIA no mapa é «este campo não
/// tem canal» (o ponto de sempre, inerte).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyMark {
    /// O campo tem canal e nenhuma curva o conduz — o ponto de sempre, mas CLICÁVEL: o 1.º clique
    /// grava a 1.ª chave (o cronómetro do After Effects).
    Livre,
    /// Uma curva conduz o campo, sem chave neste quadro — losango vazio.
    Animado,
    /// Há chave neste quadro — losango cheio.
    Chave,
    /// O valor saiu da curva e ninguém o gravou — losango cheio na cor de aviso: perde-se quando o
    /// tempo mudar (o laranja do Blender).
    Mudado,
}

/// O que um clique no losango de uma linha faz — a shell lê-o e escreve na linha do tempo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyMarkClick {
    /// Os campos da linha clicada (um losango por linha: `X`/`Y` partilham o seu).
    pub fields: Vec<NodeId>,
}

thread_local! {
    static MARCAS: RefCell<BTreeMap<NodeId, KeyMark>> = const { RefCell::new(BTreeMap::new()) };
    static ALVOS: RefCell<Vec<(NodeId, Vec<NodeId>)>> = const { RefCell::new(Vec::new()) };
    static PEDIDO: RefCell<Option<KeyMarkClick>> = const { RefCell::new(None) };
}

/// O sal do id do losango: um campo `f` regista o seu losango como `f ^ SAL` no índice de alvos.
const SAL: u64 = 0x6c6f_7361_6e67_6f00; // "losango"

/// O id com que o losango da linha cujo primeiro campo é `campo` entra no índice de alvos — o rato
/// acha-o pela MESMA ordem de empilhamento que os outros widgets (um menu aberto por cima ganha).
#[must_use]
pub const fn key_mark_id(campo: NodeId) -> NodeId {
    NodeId(campo.0 ^ SAL)
}

/// Publica o estado de cada campo animado neste quadro (substitui o anterior) e esvazia os alvos do
/// quadro anterior — a pintura que se segue regista os de agora. Chamado pela shell, uma vez por
/// quadro, antes de pintar os painéis.
pub fn set_key_marks(marks: BTreeMap<NodeId, KeyMark>) {
    MARCAS.with(|m| *m.borrow_mut() = marks);
    ALVOS.with(|a| a.borrow_mut().clear());
}

/// O losango de uma linha com estes campos: o estado mais forte entre eles (`Mudado` > `Chave` >
/// `Animado` > `Livre`); `None` = nenhum tem canal.
#[must_use]
pub fn key_mark_of(fields: &[NodeId]) -> Option<KeyMark> {
    MARCAS.with(|m| {
        let m = m.borrow();
        fields.iter().filter_map(|id| m.get(id).copied()).max()
    })
}

/// O losango cujo id de alvo é `id`, se foi pintado neste quadro: os campos da linha dele.
#[must_use]
pub fn key_mark_target(id: NodeId) -> Option<KeyMarkClick> {
    ALVOS.with(|a| {
        a.borrow()
            .iter()
            .rev()
            .find(|(alvo, _)| *alvo == id)
            .map(|(_, f)| KeyMarkClick { fields: f.clone() })
    })
}

/// O despacho do rato regista o clique num losango; a shell lê-o com [`take_key_mark_click`].
pub(crate) fn pedir(c: KeyMarkClick) {
    PEDIDO.with(|p| *p.borrow_mut() = Some(c));
}

/// O clique num losango desde a última leitura (a shell lê-o uma vez por quadro).
#[must_use]
pub fn take_key_mark_click() -> Option<KeyMarkClick> {
    PEDIDO.with(|p| p.borrow_mut().take())
}

/// Pinta a coluna do decorator de uma linha com `fields`: o losango quando algum é animado, o ponto
/// quando não. Quando algum tem canal devolve o id do alvo do clique ([`key_mark_id`]); quem pinta
/// regista-o em `r` no índice de alvos (o `widget` não fala com a `interaction`).
#[must_use]
pub fn paint_decorator_for(
    scene: &mut VectorScene,
    theme: Theme,
    r: Rect,
    fields: &[NodeId],
) -> Option<NodeId> {
    if !crate::paint::ui_is_redesign() {
        return None;
    }
    let (Some(mark), Some(&primeiro)) = (key_mark_of(fields), fields.first()) else {
        super::row::paint_decorator(scene, theme, r, false);
        return None;
    };
    paint_key_mark(scene, theme, r, mark);
    ALVOS.with(|a| {
        a.borrow_mut()
            .push((key_mark_id(primeiro), fields.to_vec()))
    });
    Some(key_mark_id(primeiro))
}

/// O losango no centro de `r`: a cor da chave da dope-sheet (`TimelineKey`) — o mesmo losango, o
/// mesmo significado —, vazio quando `Animado`; cheio na cor de aviso quando `Mudado`; o ponto de
/// sempre quando `Livre`.
pub fn paint_key_mark(scene: &mut VectorScene, theme: Theme, r: Rect, mark: KeyMark) {
    if mark == KeyMark::Livre {
        super::row::paint_decorator(scene, theme, r, false);
        return;
    }
    let h = f64::from(Spacing::Xs.px());
    let (cx, cy) = (f64::from(r.x + r.w * 0.5), f64::from(r.y + r.h * 0.5));
    let mut p = BezPath::new();
    p.move_to((cx, cy - h));
    p.line_to((cx + h, cy));
    p.line_to((cx, cy + h));
    p.line_to((cx - h, cy));
    p.close_path();
    let cor = resolve(
        if mark == KeyMark::Mudado {
            ColorToken::Warn
        } else {
            ColorToken::TimelineKey
        },
        theme,
    );
    if mark == KeyMark::Animado {
        scene.inner_mut().stroke(
            &ph2d_vector::Stroke::new(1.0), // LITERAL-PX-OK: o traço de um losango vazio
            Affine::IDENTITY,
            cor,
            None,
            &p,
        );
    } else {
        scene
            .inner_mut()
            .fill(Fill::NonZero, Affine::IDENTITY, cor, None, &p);
    }
}

#[cfg(test)]
#[path = "key_mark_tests.rs"]
mod tests;
