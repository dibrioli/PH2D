//! ⭐ **O campo Figma** (`docs/CartoesUI/02_especificacao.md` §3) — o registo de QUEM é campo de
//! cartão e o pedido do cursor preso que daí sai.
//!
//! Um campo de cartão é uma `NumberInput` como as outras (valor, buffer, foco, Mixed, Tab); o que
//! muda é o GESTO: o arrasto nasce só na célula do ícone, Shift = ×10 e Alt = ×0,1 (o Figma), o
//! eixo é sempre o horizontal, as setas andam pelo passo, o texto aceita contas e o duplo clique no
//! ícone repõe o valor de fábrica. O painel marca o campo antes de o pintar (como semeia o valor).

use super::WidgetStore;
use ph2d_a11y::NodeId;

/// O que o painel declara de um campo de cartão.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CardFieldSpec {
    /// Unidades por toque de seta (↑ / ↓), e as casas que o valor mostra.
    pub step: f64,
    /// Unidades por pixel de arrasto sem modificador (campos sem intervalo conhecido).
    pub scrub: f64,
    /// O valor de FÁBRICA que o duplo clique no ícone repõe (`None` = sem reposição).
    pub default: Option<f64>,
}

impl WidgetStore {
    /// Marca `id` como campo de cartão (idempotente; o painel chama-o a cada quadro).
    pub fn mark_card_field(&mut self, id: NodeId, spec: CardFieldSpec) {
        self.card_fields.get_mut().insert(id, spec);
    }

    /// T29: o campo de um ajudante de linha pintado dentro de um cartão — só se o painel não o
    /// declarou (a declaração do painel manda). `card_fields` é `RefCell` por isto (o precedente
    /// do `fold_body_h`: o passe de pintura só tem a loja partilhada).
    pub fn mark_card_field_at_paint(&self, id: NodeId, spec: CardFieldSpec) {
        self.card_fields.borrow_mut().entry(id).or_insert(spec);
    }

    #[must_use]
    pub fn card_field(&self, id: NodeId) -> Option<CardFieldSpec> {
        self.card_fields.borrow().get(&id).copied()
    }

    /// ⭐ **O cursor tem de ficar PRESO?** — sim enquanto um campo de cartão está a ser arrastado
    /// (depois do limiar). A shell lê-o a cada quadro: prende e esconde o cursor na subida, solta e
    /// devolve-o na descida (o arrasto sem fim, D7).
    #[must_use]
    pub fn pointer_lock_wanted(&self) -> bool {
        self.number_input_drag
            .is_some_and(|d| d.crossed_threshold && self.card_fields.borrow().contains_key(&d.id))
    }

    /// O multiplicador do passo pelos modificadores do Figma — Shift ×10, Alt ×0,1, nada ×1.
    /// ⚠️ Lido a CADA movimento, e o modelo do scrub é incremental: trocar a meio muda a taxa a
    /// partir daquele ponto, sem salto no valor.
    #[must_use]
    pub fn card_step_mul(&self) -> f64 {
        use ph2d_card_stack::metrics::{SCRUB_ALT_MUL, SCRUB_SHIFT_MUL};
        if self.shift_held() {
            SCRUB_SHIFT_MUL
        } else if self.alt_held() {
            SCRUB_ALT_MUL
        } else {
            1.0
        }
    }
}
