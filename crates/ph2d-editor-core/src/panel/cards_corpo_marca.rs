//! ⭐ **A linha de marcar do cartão numa MOLDURA, à Godot** (dono 13/11,
//! `project-memory/project_checkbox_in_a_frame_like_godot_2026_11_13.md`): o nome EM CIMA, como o de
//! qualquer campo, com o losango na linha do rótulo acima do canto superior direito da moldura; por
//! baixo a moldura de campo com o ✓ e a palavra «On» (`chrome.checkbox.on`, a mesma em ambos os estados).
//! O clique na moldura E no nome marca; o losango tem o alvo dele.

use super::rotulo_em_cima;
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::CheckboxValue;
use crate::widget::card_ink::CardInk;
use crate::widget::key_mark::Rotulo;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;

/// A palavra da moldura: nomeia o que a marca liga; quem diz o estado é o ✓.
fn palavra() -> &'static str {
    ph2d_i18n::tr("chrome.checkbox.on")
}

/// A altura da faixa do rótulo por cima da moldura (`LABEL_TOP` fica a cargo de quem arruma a linha).
pub(in crate::panel) const FAIXA: f32 = m::LABEL_PX + m::LABEL_BOTTOM;

/// A largura em que a linha mostra tudo: o maior entre o nome com o losango e a moldura com ✓ e «On».
pub(crate) fn pede(text_system: &mut TextSystem, label: Rotulo<'_>) -> f32 {
    let nome = text_system.prefix_width(label.texto, m::LABEL_PX).ceil() + m::KEY_MARK_BOX;
    nome.max(dentro(text_system))
}

/// A moldura com ✓ e «On» inteiros.
fn dentro(text_system: &mut TextSystem) -> f32 {
    let on = text_system.prefix_width(palavra(), m::VALUE_PX).ceil();
    m::FIELD_PAD_R + m::CHECK_BOX + m::CHECK_GAP + on + m::FIELD_PAD_R
}

/// ⭐ A linha de marcar com a MOLDURA no rect `r` (o do controlo, à altura de `FIELD_H`); o nome e o
/// losango ocupam a faixa [`FAIXA`] logo acima — a mesma conta de [`rotulo_em_cima`].
#[allow(clippy::too_many_arguments)]
pub(in crate::panel) fn check_em(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    r: Rect,
    (id, label): (NodeId, Rotulo<'_>),
    (valor, parada): (CheckboxValue, bool),
) {
    let topo = r.y - FAIXA - m::LABEL_TOP;
    let linha = rotulo_em_cima(scene, text_system, ink, r.x, r.w, topo, label.texto);
    debug_assert_eq!(linha.control.y, r.y);
    // O nome primeiro: o último registo é o da moldura, que `rect_for` devolve.
    hit_index.register_label(id, linha.label);
    hit_index.register(id, r);
    if super::censo::ligado() {
        let precisa = dentro(text_system);
        super::censo::texto(palavra(), precisa, r.w);
    }
    let hover = !parada && store.hover_live(id) > 0.0;
    crate::widget::card_field::paint_field_frame(scene, ink, r, hover, false);
    let caixa = crate::widget::card_check_rect(r.x + m::FIELD_PAD_R, r.y, r.h);
    crate::widget::paint_card_check_value(scene, ink, caixa, valor, hover);
    let tx = caixa.x + m::CHECK_BOX + m::CHECK_GAP;
    crate::paint::paint_text_elided(
        text_system,
        scene,
        palavra(),
        tx,
        r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
        m::VALUE_PX,
        (r.x + r.w - m::FIELD_PAD_R - tx).max(0.0),
        ink.color(if parada {
            ColorToken::CardLabel
        } else {
            ColorToken::KanbanCardFg
        }),
    );
    if let Some(alvo) =
        crate::widget::key_mark::paint_decorator_campo(scene, ink.theme, linha.dot, &[id], label)
    {
        hit_index.register(alvo, linha.dot);
    }
}

/// A largura de [`check_em_fila`]: a moldura com ✓ e o nome dentro, e o losango ao lado.
pub(crate) fn pede_em_fila(text_system: &mut TextSystem, nome: &str) -> f32 {
    let tw = text_system.prefix_width(nome, m::VALUE_PX).ceil();
    m::FIELD_PAD_R + m::CHECK_BOX + m::CHECK_GAP + tw + m::FIELD_PAD_R + m::KEY_MARK_BOX
}

/// ⭐ A linha de marcar numa BARRA de uma linha só (a banda do Flip, da Timeline): a mesma moldura de
/// campo, mas o nome mora DENTRO dela ao lado do ✓ (o `CheckBox` da Godot numa toolbar) e o losango
/// fica à direita, na altura dos vizinhos — o nome em cima abriria uma segunda linha numa barra de uma.
#[allow(clippy::too_many_arguments)]
pub(in crate::panel) fn check_em_fila(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    r: Rect,
    (id, label): (NodeId, Rotulo<'_>),
    (valor, parada): (CheckboxValue, bool),
) {
    let k = m::KEY_MARK_BOX;
    let moldura = Rect::new(r.x, r.y, (r.w - k).max(0.0), r.h);
    hit_index.register(id, moldura);
    let hover = !parada && store.hover_live(id) > 0.0;
    crate::widget::card_field::paint_field_frame(scene, ink, moldura, hover, false);
    let caixa = crate::widget::card_check_rect(moldura.x + m::FIELD_PAD_R, r.y, r.h);
    crate::widget::paint_card_check_value(scene, ink, caixa, valor, hover);
    let tx = caixa.x + m::CHECK_BOX + m::CHECK_GAP;
    let livre = (moldura.x + moldura.w - m::FIELD_PAD_R - tx).max(0.0);
    if super::censo::ligado() {
        let tw = text_system.prefix_width(label.texto, m::VALUE_PX);
        super::censo::texto(label.texto, tw, livre);
    }
    crate::paint::paint_text_elided(
        text_system,
        scene,
        label.texto,
        tx,
        r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
        m::VALUE_PX,
        livre,
        ink.color(if parada {
            ColorToken::CardLabel
        } else {
            ColorToken::KanbanCardFg
        }),
    );
    let dot = Rect::new(moldura.x + moldura.w, r.y + (r.h - k) * 0.5, k, k);
    if let Some(alvo) =
        crate::widget::key_mark::paint_decorator_campo(scene, ink.theme, dot, &[id], label)
    {
        hit_index.register(alvo, dot);
    }
}

/// A caixa SOLTA num rect já arrumado (o item de uma lista, não uma propriedade: a caixa É o item):
/// o quadradinho e o nome à direita, numa linha; a linha inteira é o alvo.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_solto(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    ink: CardInk,
    r: Rect,
    (id, nome): (NodeId, &str),
    (valor, parada): (CheckboxValue, bool),
) {
    hit_index.register(id, r);
    let caixa = crate::widget::card_check_rect(r.x, r.y, r.h);
    let hover = !parada && store.hover_live(id) > 0.0;
    crate::widget::paint_card_check_value(scene, ink, caixa, valor, hover);
    let tx = r.x + m::CHECK_BOX + m::CHECK_GAP;
    crate::paint::paint_text_elided(
        text_system,
        scene,
        nome,
        tx,
        r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
        m::VALUE_PX,
        (r.x + r.w - tx).max(0.0),
        ink.color(if parada {
            ColorToken::CardLabel
        } else {
            ColorToken::KanbanCardFg
        }),
    );
}
