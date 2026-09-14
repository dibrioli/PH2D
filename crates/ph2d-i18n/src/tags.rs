//! **AS STRINGS DO PAINEL TAGS** (TOP-20 #9, W4) — o irmão de tabela do [`super`].
//!
//! ⚠️ **Um módulo por ASSUNTO, como o `sculpt3d.rs` e o `model3d.rs`**, e pela mesma razão de
//! isolamento (`CLAUDE.md` §0.2): enquanto todas as chaves moram num `match` só, duas linhas
//! paralelas que acrescentem uma chave cada colidem no mesmo punhado de linhas.
//!
//! ⛔ **As FRASES DE RECUSA não estão aqui**, e a ausência é a decisão: elas nascem em
//! `ph2d_tags::TagError::message`, ao lado da lei que as produz, e atravessam o painel sem serem
//! interpretadas. Duplicá-las numa tabela de i18n daria duas frases para a mesma recusa, e elas
//! divergiriam na primeira vez que alguém mexesse numa.

/// A tradução de uma chave `panel.tags.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.tags.title" => "Tags",
        // ⭐⭐ **Os TRÊS que estavam escritos em SEIS sítios.** O `Pick a tag…` vivia em três
        // pintores (a secção do Inspector · o passe diferido do popover · o alvo de uma *Signal
        // Action*) e o `Only for tag…` em dois — *uma palavra escrita em dois sítios ainda não é
        // uma palavra do app, só uma PORTA é* (a lei que o `chrome.rs` já escreve, com quatro
        // duplicados medidos). Foi o `hr15_no_hardcoded_ui_strings` que os apanhou.
        "panel.tags.pick" => "Pick a tag\u{2026}",
        "panel.tags.new_or_search" => "New tag or search\u{2026}",
        // ⚠️ **O `(any)` faz parte da frase**, e não é decoração: sem filtro escolhido a armadilha
        // dispara para TODOS, e o campo vazio tem de o dizer (a §7.3 do plano).
        "panel.tags.only_for" => "Only for tag\u{2026}  (any)",
        _ => return None,
    })
}
