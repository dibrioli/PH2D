//! ⭐ **As linhas que o corpo de um cartão pinta para quem as pede de fora** (T29/T64): o
//! `property_row` pergunta [`card_rows`] e, dentro de um cartão, pinta pela linha do cartão. O
//! painel que pinta o corpo injecta-as ([`com`]) — a fundação não chega ao painel (A10: uma tabela
//! injectada por quem chama, nunca um re-export).

use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::CheckboxValue;
use crate::widget::card_ink::CardInk;
use crate::widget::key_mark::{Eixos, Rotulo};
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Uma linha de campos Figma pedida de fora do cartão: rótulo, eixos, unidade, ids e passo.
pub type FieldsRow = fn(
    &mut VectorScene,
    &mut TextSystem,
    &WidgetStore,
    &mut HitIndex,
    CardInk,
    f32,
    f32,
    f32,
    &str,
    Eixos,
    Option<&str>,
    &[NodeId],
    f64,
) -> f32;
/// Uma escolha (segmentos `(texto, aceso, id)`) pedida de fora do cartão, com o rótulo em cima.
pub type ChoicesRow = fn(
    &mut VectorScene,
    &mut TextSystem,
    &WidgetStore,
    &mut HitIndex,
    CardInk,
    f32,
    f32,
    f32,
    &str,
    &[(&str, bool, NodeId)],
) -> f32;
/// Uma caixa de marcar à Figma pedida de fora do cartão: `(id, rótulo, valor, parada)`.
pub type CheckRow = fn(
    &mut VectorScene,
    &mut TextSystem,
    &WidgetStore,
    &mut HitIndex,
    CardInk,
    f32,
    f32,
    f32,
    NodeId,
    Rotulo<'_>,
    CheckboxValue,
    bool,
) -> f32;
/// Uma linha de texto pedida de fora do cartão: rótulo em cima, a caixa do cartão por baixo.
pub type TextRow = fn(
    &mut VectorScene,
    &mut TextSystem,
    &WidgetStore,
    &mut HitIndex,
    CardInk,
    f32,
    f32,
    f32,
    Rotulo<'_>,
    NodeId,
    &str,
) -> f32;
/// O rótulo em cima de um controlo que o chamador constrói: devolve as colunas da linha.
pub type LabelRow = fn(
    &mut VectorScene,
    &mut TextSystem,
    CardInk,
    f32,
    f32,
    f32,
    &str,
) -> crate::widget::PropertyRow;

/// As linhas que o corpo de um cartão sabe pintar para o `property_row`.
#[derive(Copy, Clone)]
pub struct CardRows {
    pub fields: FieldsRow,
    pub choices: ChoicesRow,
    pub check: CheckRow,
    pub text: TextRow,
    pub label: LabelRow,
}

thread_local! {
    static LINHAS: std::cell::Cell<Option<CardRows>> = const { std::cell::Cell::new(None) };
}

/// Corre `f` com as linhas do cartão publicadas; repõe as anteriores no fim (também num pânico).
pub fn com<R>(rows: CardRows, f: impl FnOnce() -> R) -> R {
    struct Repoe(Option<CardRows>);
    impl Drop for Repoe {
        fn drop(&mut self) {
            LINHAS.with(|c| c.set(self.0));
        }
    }
    let _repoe = Repoe(LINHAS.with(|c| c.replace(Some(rows))));
    f()
}

/// A tinta e as linhas do cartão cujo corpo se está a pintar — `None` fora de um cartão.
#[must_use]
pub fn card_rows() -> Option<(CardInk, CardRows)> {
    Some((
        crate::widget::card_ink::card_body()?,
        LINHAS.with(std::cell::Cell::get)?,
    ))
}
