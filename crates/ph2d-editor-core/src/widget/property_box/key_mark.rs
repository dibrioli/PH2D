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

/// O estado de animação de um campo, como o losango o mostra. A AUSÊNCIA no mapa é «não animado»
/// (o ponto de sempre).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyMark {
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
    static ALVOS: RefCell<Vec<(Rect, Vec<NodeId>)>> = const { RefCell::new(Vec::new()) };
}

/// Publica o estado de cada campo animado neste quadro (substitui o anterior). Chamado pela shell.
pub fn set_key_marks(marks: BTreeMap<NodeId, KeyMark>) {
    MARCAS.with(|m| *m.borrow_mut() = marks);
}

/// O losango de uma linha com estes campos: o estado mais forte entre eles (`Mudado` > `Chave` >
/// `Animado`); `None` = nenhum é animado.
#[must_use]
pub fn key_mark_of(fields: &[NodeId]) -> Option<KeyMark> {
    MARCAS.with(|m| {
        let m = m.borrow();
        fields.iter().filter_map(|id| m.get(id).copied()).max()
    })
}

/// Esvazia os alvos clicáveis — a shell chama-o antes de pintar os painéis do quadro.
pub fn begin_key_mark_targets() {
    ALVOS.with(|a| a.borrow_mut().clear());
}

/// O losango sob `(x, y)`, se algum foi pintado neste quadro: os campos da linha dele.
#[must_use]
pub fn key_mark_at(x: f32, y: f32) -> Option<KeyMarkClick> {
    ALVOS.with(|a| {
        a.borrow()
            .iter()
            .rev()
            .find(|(r, _)| r.contains(x, y))
            .map(|(_, f)| KeyMarkClick { fields: f.clone() })
    })
}

/// Pinta a coluna do decorator de uma linha com `fields`: o losango quando algum é animado (e o
/// regista como alvo do clique), o ponto de sempre quando não.
pub fn paint_decorator_for(scene: &mut VectorScene, theme: Theme, r: Rect, fields: &[NodeId]) {
    if !crate::paint::ui_is_redesign() {
        return;
    }
    let Some(mark) = key_mark_of(fields) else {
        super::row::paint_decorator(scene, theme, r, false);
        return;
    };
    paint_key_mark(scene, theme, r, mark);
    ALVOS.with(|a| a.borrow_mut().push((r, fields.to_vec())));
}

/// O losango no centro de `r`: a cor da chave da dope-sheet (`TimelineKey`) — o mesmo losango, o
/// mesmo significado —, vazio quando `Animado`; cheio na cor de aviso quando `Mudado`.
pub fn paint_key_mark(scene: &mut VectorScene, theme: Theme, r: Rect, mark: KeyMark) {
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
