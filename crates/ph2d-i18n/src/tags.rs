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
        // ⭐⭐ **OS VERBOS DA BARRA** (2026-09-17) — o painel TAGS nunca teve régua de HR-15, e estes
        // oito eram os literais que ela não via. ⚠️ O `+` faz parte do rótulo: ele é o que diz que o
        // botão CRIA, e a barra não tem ícone.
        "panel.tags.verb.new" => "+ New",
        "panel.tags.verb.child" => "+ Child",
        "panel.tags.verb.rename" => "Rename",
        "panel.tags.verb.unparent" => "Move to root",
        // ⚠️ **Frases com peças do código, lidas por [`crate::tr_with`] com marcadores NOMEADOS** —
        // colar o número no pintor fixaria a ordem das palavras, e há línguas em que a contagem vem
        // à frente. ⭐⭐ E o rótulo de apagar CARREGA O ESTRAGO: apagar `Enemy` leva `Flying` e
        // `Boss` junto, e o artista só vê isso se o botão o disser ANTES de ser carregado — são
        // duas frases porque uma tag-folha não leva tags nenhumas.
        "panel.tags.verb.select" => "Select ({members})",
        "panel.tags.verb.delete_subtree" => "Delete ({tags} tags, {objects} objects)",
        "panel.tags.verb.delete" => "Delete ({objects} objects)",
        // ⚠️ A frase do vazio NOMEIA o botão que a cura (`+ New`), e por isso ela é uma frase
        // inteira e não um fragmento: um *«Nenhuma tag»* solto não diz ao artista o que fazer.
        "panel.tags.empty" => "No tags yet. Press + New to make the first one.",
        // ⭐⭐⭐ **OS NOMES das linhas de TEXTO** — report do dono, 2026-09-22: *«campos de texto
        //    difíceis de saber para que servem»*. A chave irmã sem `_label` é o EXEMPLO que a caixa
        //    mostra enquanto está VAZIA; esta é o NOME, que fica na coluna da esquerda para sempre.
        "panel.tags.new_label" => "New Tag",
        _ => return None,
    })
}
