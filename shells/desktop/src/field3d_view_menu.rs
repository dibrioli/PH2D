//! ⭐⭐⭐ **O CABEÇALHO DE CADA VISTA É CLICÁVEL** (W109) — o último ⏳ do canvas de primeira classe.
//!
//! # O que a W90d deixou aberto, com as palavras dela
//!
//! > *«É um **MOSTRADOR**, não um controlo … ⏳ Um cabeçalho **clicável** (com menu por vista) fica
//! > aberto, e é ele que pede a faixa reservada.»*
//!
//! ⛔ **A segunda metade daquela frase estava errada, e é a razão de esta wave ser barata.** Uma
//! faixa reservada seria precisa para uma *barra* de cabeçalho — mas um menu precisa só de **um
//! alvo de clique**, e o rótulo já tem posição e tamanho. *Uma dependência afirmada sem a desmontar
//! é uma feature adiada com cara de arquitectura* — a terceira vez que esta linha o escreve (a
//! primeira foi o divisor «a precisar» do cabeçalho, a segunda a divisão «a precisar» das quatro
//! câmeras).
//!
//! # ⭐ Ele não é um `ContextMenuKind`
//!
//! O [`ph2d_editor::widget::paint_context_menu`] é **autónomo**: recebe um modelo e um
//! rectângulo. Registar uma variante no `ContextMenuKind` foundational obrigaria a tocar o `enum`, a
//! lista de amostras, a tabela de rows e o ficheiro de ids — quatro sítios de outra crate, com
//! colisão textual garantida — para um menu que vive **dentro** do canvas 3D e que este módulo já
//! sabe pintar e apanhar. *A porta genérica que serve é a do PINTOR, não a do enum.*
//!
//! # ⚠️ Os dois rectângulos são PUBLICADOS por quem pinta, nunca estimados
//!
//! O chip do rótulo depende da largura do texto, e o menu da largura da linha mais comprida. A casa
//! já tem a lei: *«quem empilha texto de comprimento variável tem de perguntar ao pintor quanto ele
//! gastou, não estimar»* ([`ph2d_editor::paint::paint_text_block`]). ⇒ o pintor mede
//! ([`ph2d_text::TextSystem::prefix_width`]) e guarda — `Viewport::label` e `Smoke::view_menu_rect`
//! —, exactamente como o `Viewport::area` já responde *«este clique é meu?»*. Enquanto nada foi
//! pintado os dois são `None`, e *«ainda não desenhei» e «o ponto não é meu» são a mesma resposta*.

use crate::field3d_views::Standard;
use ph2d_editor::NodeId;
use ph2d_editor::widget::{ContextMenu, ContextMenuEntry, ListItem};
use ph2d_editor::zones::Rect;

/// A altura de uma linha do menu — a mesma que o resto do chrome usa.
pub(crate) const ROW_H_PX: f32 = 26.0; // LITERAL-PX-OK: overlay metric (menu row height)

/// A folga que o chip do rótulo deixa à volta do texto, para o alvo do clique não ser o próprio
/// glifo. ⚠️ **Ela é o que separa «o rótulo é legível» de «o rótulo é agarrável»**: o texto de
/// *«Top»* tem `~22 px` de largura, e um alvo desse tamanho reprova qualquer régua de toque.
pub(crate) fn chip_pad() -> f32 {
    ph2d_tokens::Spacing::Sm.px()
}

/// O recuo interior do menu, igual ao que o [`ContextMenu`] pinta.
fn menu_pad() -> f32 {
    ph2d_tokens::Spacing::Md.px()
}

/// ⭐ **O CHIP CLICÁVEL do rótulo**, derivado da área da vista e da largura MEDIDA do texto.
///
/// ⚠️ `inset` é o mesmo recuo com que o rótulo é pintado — ele entra por argumento em vez de ser
/// uma segunda cópia da constante do pintor. *Uma lei escrita em dois sítios ainda não é uma lei.*
pub(crate) fn chip(view: Rect, inset: f32, text_w: f32, font_px: f32) -> Rect {
    let pad = chip_pad();
    Rect::new(
        view.x + inset - pad,
        view.y + inset - pad * 0.5,
        text_w + pad * 2.0,
        font_px + pad,
    )
}

/// ⭐ **O RECTÂNGULO DO MENU**, ancorado por baixo do chip e **preso ao canvas**.
///
/// ⚠️ A prisão não é enfeite: o chip da vista de baixo-direita está a poucos pixels do canto, e um
/// menu que descesse dali sairia da janela — *o mesmo defeito que o gizmo de navegação já resolve
/// fugindo à moldura* (`panel_ops::panel_rects`).
pub(crate) fn menu_rect(chip: Rect, canvas: Rect, widest: f32) -> Rect {
    let w = widest + menu_pad() * 2.0;
    let h = ROW_H_PX * Standard::ALL.len() as f32 + menu_pad() * 2.0;
    let x = chip.x.min(canvas.x + canvas.w - w).max(canvas.x);
    // Por baixo do chip; se não couber, por cima dele.
    let abaixo = chip.y + chip.h;
    let y = if abaixo + h <= canvas.y + canvas.h {
        abaixo
    } else {
        (chip.y - h).max(canvas.y)
    };
    Rect::new(x, y, w, h)
}

/// ⭐ **Em que vista o ponto cai** — a mesma escada que o pintor percorre (`rect.y + pad`, uma linha
/// de `ROW_H_PX` por entrada).
///
/// ⚠️ **Fora do rectângulo devolve `None`, e o chamador fecha o menu** — clicar ao lado de um menu
/// aberto fecha-o em todo o chrome desta casa, e um menu 3D que ficasse aberto seria o único que
/// não obedece.
pub(crate) fn row_at(menu: Rect, p: [f32; 2]) -> Option<Standard> {
    if p[0] < menu.x || p[1] < menu.y || p[0] >= menu.x + menu.w || p[1] >= menu.y + menu.h {
        return None;
    }
    let dentro = p[1] - menu.y - menu_pad();
    if dentro < 0.0 {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let i = (dentro / ROW_H_PX) as usize;
    Standard::ALL.get(i).copied()
}

/// O id de uma linha — o hash da chave i18n dela, como no resto do módulo.
fn row_id(v: Standard) -> NodeId {
    ph2d_tool_registry::hash_node_id(v.key())
}

/// ⭐⭐ **O MODELO, derivado de [`Standard::ALL`]** — acrescentar uma vista nomeada põe-na aqui sem
/// uma linha de código.
///
/// ⚠️ **As chaves são as do PAINEL, e não as do rótulo do viewport**, e a escolha é a inversa da
/// que a W90d fez: ali o `(7)` seria a promessa de um controlo que não existia na quina; aqui o
/// controlo **é** o menu, e o atalho ao lado do nome é a única forma de a tecla ser descoberta por
/// quem não sabe que ela existe. *A mesma palavra em dois sítios pode ter de dizer coisas
/// diferentes — e a razão muda quando um mostrador vira controlo.*
pub(crate) fn model() -> ContextMenu {
    ContextMenu::new(
        ph2d_tool_registry::hash_node_id("viewport.model3d.view.menu"),
        ph2d_i18n::tr("viewport.model3d.view.menu"),
        Standard::ALL
            .into_iter()
            .map(|v| ContextMenuEntry::Item(ListItem::new(row_id(v), ph2d_i18n::tr(v.key()))))
            .collect(),
    )
}

/// A largura da linha mais comprida do menu, medida pelo pintor.
pub(crate) fn widest_row(text: &mut ph2d_text::TextSystem, font_px: f32) -> f32 {
    Standard::ALL
        .into_iter()
        .map(|v| text.prefix_width(ph2d_i18n::tr(v.key()), font_px))
        .fold(0.0_f32, f32::max)
}

#[cfg(test)]
#[path = "field3d_view_menu_tests.rs"]
mod tests;
