//! ⭐⭐⭐ **`Delete` e `Ctrl/Cmd+D` existem, e são o MESMO verbo do menu** (report do Enio, 2026-08-30).
//!
//! *«temos um bug: delete não funciona na hierarquia. Avalie também duplicate»*
//!
//! # ⛔ O que a caça mediu
//!
//! Não havia tecla nenhuma: o `KEY_DELETE` do dispatcher vira `GraphKey::Delete` e o **único**
//! consumidor dele em toda a árvore é o painel do grafo de motion. Apagar ou duplicar um objeto da
//! cena só era possível pelo menu de contexto de uma linha — e **três doc-comments do shell
//! afirmavam o contrário**, cada um a invocar um *«caminho genérico do hero»* que nunca existiu.
//!
//! # Por que estes gates e não um de comportamento
//!
//! A cadeia vive num `impl App`, e um `App` pede janela, superfície e device. O que é alcançável de
//! um teste é a **costura**: que a cadeia é CHAMADA, que ela é gateada à área do painel, e que ela
//! empurra as MESMAS acções que o item de menu empurra. *Encolher o resíduo é o que se pode fazer
//! quando o arnês não existe; fingir que ele existe, não.*

use std::fs;

const CHAIN: &str = include_str!("../src/input_dispatch/keyboard_hierarchy.rs");

/// **A cadeia é CHAMADA** — sem este fio ela é código morto e o report volta inteiro.
#[test]
fn the_keyboard_calls_the_hierarchy_chain() {
    let kb = fs::read_to_string("src/input_dispatch/keyboard.rs").expect("keyboard.rs");
    // ⚠️ **A ASSERÇÃO É A INSTRUÇÃO INTEIRA, e a razão foi medida:** a 1.ª redacção procurava só a
    // chamada, e a mutação `if false && self.hierarchy_key_chain(...)` **SOBREVIVEU** — a cadeia
    // morta com o gate verde. *Um gate textual não distingue uma chamada viva de uma desactivada;
    // o que ele pode fixar é a FORMA da instrução.* (A lei em si tem gate a sério: ela mudou-se
    // para a `verb_for`, que é pura — ver `keyboard_hierarchy_tests`.)
    assert!(
        kb.contains(
            "if self.hierarchy_key_chain(state, repeat, physical_key) {\n            return;\n        }"
        ),
        "o `keyboard.rs` deixou de chamar a cadeia da Hierarquia (ou a chamada ganhou uma guarda \
         a' frente) — `Delete` e `Ctrl+D` voltam a nao existir, e o unico caminho para apagar um \
         objeto volta a ser o menu de contexto"
    );
    // ⚠️ **A ORDEM é a feature**: ela tem de vir DEPOIS de toda cadeia específica (o traço do Flip,
    // o nó de curva, a figura do Painter, a key da timeline) e ANTES do encaminhamento ao widget
    // focado, senão rouba o `Delete` de quem o reivindica com um alvo mais preciso.
    let minha = kb
        .find("self.hierarchy_key_chain(")
        .expect("a chamada sumiu");
    let painter = kb
        .find("self.painter_delete_chain(")
        .expect("a cadeia do Painter sumiu");
    let hero = kb
        .find("forward_key_to_hero(")
        .expect("o encaminhamento ao widget focado sumiu");
    assert!(
        painter < minha && minha < hero,
        "a cadeia da Hierarquia saiu do lugar (painter {painter}, hierarquia {minha}, hero \
         {hero}) — antes do Painter ela come o Delete da figura em maos; depois do hero ela corre \
         com o widget focado a ja' ter respondido"
    );
}

/// **Ela é gateada à ÁREA do painel** — sem isto, uma tecla com cinco donos passa a ter seis.
#[test]
fn the_chain_is_gated_on_the_pointer_being_over_the_panel() {
    // ⚠️ **O que este ficheiro mede agora é o FIO, não a lei** — depois de a decisão se ter mudado
    // para a `verb_for` (pura), quem prova que cada guarda recusa sozinha é
    // `keyboard_hierarchy_tests`. Aqui afirma-se que os fatos CHEGAM lá: uma guarda perfeita
    // alimentada por `over_panel: true` constante não guarda nada.
    assert!(
        CHAIN.contains("over_panel: self.cursor_over_hierarchy()"),
        "a cadeia deixou de perguntar se o ponteiro esta' sobre a Hierarquia — ela passa a roubar \
         o Delete do traco do Flip, do no' de curva, da figura do Painter e da key da timeline"
    );
    assert!(
        CHAIN.contains("ph2d_editor::ids::HIER_PANEL"),
        "a area deixou de ser a do painel da Hierarquia"
    );
    // ⚠️ E o campo de texto FOCADO fica com as teclas: o rename de uma linha vive dentro deste
    // mesmo painel, e sem a guarda apagar uma letra do nome apagaria o objeto.
    assert!(
        CHAIN.contains("text_focused: self.text_entry_focused()"),
        "a guarda do campo de texto sumiu — apagar uma letra no rename de uma linha passa a apagar \
         o objeto"
    );
}

/// ⭐⭐ **As teclas são um segundo PRODUTOR do verbo do menu, nunca uma segunda LEI.**
///
/// ⛔ **O controle NEGATIVO é o que este gate compra:** a cadeia não pode conter `despawn`, nem
/// `deep_copy`, nem tocar no gizmo. No dia em que contiver, existem duas respostas para *«o que
/// apagar quer dizer»* — e a que o artista alcança pelo teclado deixa de ser a que o menu aplica
/// (a multi-selecção, a limpeza do gizmo, a promoção do novo primário, o undo).
#[test]
fn the_keys_push_the_same_actions_the_menu_pushes() {
    for acao in [
        // ⚠️ **A FORMA mudou na integração de 2026-09-04, a LEI não**: a `line/components`
        // partiu o `EditorAction` e as 33 variantes da Hierarquia desceram para o
        // `HierRequest`. Este gate afirma sobre a REDACÇÃO das duas portas, então ele tinha de
        // seguir — e é exactamente por isso que ele existe: se só uma das duas tivesse sido
        // reescrita, a tecla e o menu passariam a empurrar coisas diferentes em silêncio.
        "action_bus::EditorAction::Hierarchy(action_bus::HierRequest::Delete { row })",
        "action_bus::EditorAction::Hierarchy(action_bus::HierRequest::Duplicate { row })",
    ] {
        assert!(
            CHAIN.contains(acao),
            "a cadeia deixou de empurrar `{acao}` — ou a tecla morreu, ou ela ganhou lei propria"
        );
    }
    let menu = fs::read_to_string("../../crates/ph2d-panel-hierarchy/src/event.rs")
        .expect("o event.rs do painel");
    for acao in [
        "EditorAction::Hierarchy(HierRequest::Delete { row })",
        "EditorAction::Hierarchy(HierRequest::Duplicate { row })",
    ] {
        assert!(
            menu.contains(acao),
            "o item de MENU deixou de empurrar `{acao}` — as duas portas divergiram, e a que o \
             gate acima mede e' a do teclado"
        );
    }
    // ⚠️ **A 1.ª redacção deste controle proibia `gizmo.` e reprovou sobre codigo CERTO**: a cadeia
    // LÊ a selecção para saber a quem o verbo se aplica, e isso não é lei própria — é a pergunta.
    // O que ela não pode é ESCREVER: mutar o gizmo ou tocar no mundo é ter uma segunda resposta
    // para *«o que apagar quer dizer»*. *Uma cerca que proíbe a leitura junto com a escrita mede
    // outra coisa.*
    for proibido in [
        "despawn",
        "deep_copy",
        "gizmo.selection =",
        "replace_selection",
        "extra_selection",
        "world_mut",
    ] {
        assert!(
            !CHAIN.contains(proibido),
            "a cadeia do teclado passou a ESCREVER (`{proibido}`) — ela e' um PRODUTOR do verbo, e \
             uma lei propria aqui diverge da que o menu aplica"
        );
    }
}
