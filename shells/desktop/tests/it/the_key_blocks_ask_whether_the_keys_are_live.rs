//! **ARCH-GATE: um bloco de TECLA pergunta se as teclas estão VIVAS, nunca só se a
//! ferramenta está em mãos** (BUGS_vector #25).
//!
//! Enio, 2026-08-04: *"ao tentar renomear objetos na Hierarchy, há conflitos com atalhos
//! do módulo vector."* Digitar `Update` no campo de rename disparava **U**nion,
//! **D**ifference e o modo **T**exto; `Backspace` apagava um vértice em vez de uma letra;
//! as setas moviam a forma em vez do cursor de texto.
//!
//! ⚠️ **A guarda existia e estava CERTA** (`text_entry_focused` pergunta ao *store* quem
//! tem o foco do teclado, e o campo de rename da Hierarquia é um `TextInput` do mesmo
//! store). O que apodreceu foi o modo de aplicá-la: ela era composta **à mão** em três dos
//! oito blocos, e os outros cinco nasceram sem — *uma condição que enumera os seus
//! leitores apodrece*.
//!
//! Por isso a pergunta virou PORTA (`App::vector_keys_live` / `App::motion_keys_live`) e
//! este gate recusa o predicado CRU num arquivo de teclado. O 9º bloco nasce coberto, que
//! é exatamente como os cinco nasceram descobertos.
//!
//! ⚠️ **Por que arch-gate e não teste de comportamento:** `vector_tool_active` lê
//! `self.gfx`, que exige janela + GPU, então headless ele devolve `false` e **nenhum**
//! bloco dispara — um teste headless não distingue a correção do bug. É o mesmo muro que
//! o irmão `the_hovered_area_owns_the_clipboard_chord.rs` documenta.

use std::fs;

/// A FAMÍLIA `keyboard*.rs`, não um arquivo — o `keyboard.rs` já cedeu blocos para
/// `keyboard_timeline.rs` e `keyboard_files.rs` quando cruzou o cap de LOC do HR-18, e um
/// gate que nomeia um endereço morre no próximo corte por responsabilidade.
fn keyboard_family() -> Vec<(String, String)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/input_dispatch");
    let mut out: Vec<(String, String)> = fs::read_dir(dir)
        .expect("src/input_dispatch legível")
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let is_kb = name.starts_with("keyboard") && name.ends_with(".rs");
            is_kb.then(|| (name, fs::read_to_string(e.path()).expect("arquivo legível")))
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// **CONTROLE POSITIVO.** Uma varredura vazia — pasta renomeada, prefixo trocado — deixaria
/// todo gate abaixo verde por vácuo, dizendo que zero arquivos não têm o defeito.
#[test]
fn the_sweep_finds_the_keyboard_family() {
    let fam = keyboard_family();
    assert!(
        fam.len() >= 3,
        "a varredura achou {} arquivo(s) de teclado; ela deveria alcançar pelo menos \
         keyboard.rs + keyboard_timeline.rs + keyboard_files.rs. Um gate que varre nada \
         passa por vácuo.",
        fam.len()
    );
    let names: Vec<&str> = fam.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        names.contains(&"keyboard.rs"),
        "keyboard.rs saiu da varredura: {names:?}"
    );
}

/// Nenhum bloco de tecla pergunta `vector_tool_active()` cru.
#[test]
fn the_vector_key_blocks_ask_whether_the_keys_are_live() {
    for (name, src) in keyboard_family() {
        assert!(
            !src.contains("self.vector_tool_active()"),
            "`{name}` pergunta `self.vector_tool_active()` — *a ferramenta está em mãos?* — \
             onde a pergunta de uma TECLA é `self.vector_keys_live()`, que também exige que \
             nenhum campo de texto tenha o foco. Com o rename da Hierarquia aberto, este \
             bloco rouba a tecla de quem está digitando (BUGS_vector #25). \
             O PONTEIRO continua usando `vector_tool_active` — clicar o canvas com um campo \
             focado é justamente como se sai dele —, e é por isso que a porta é só do teclado."
        );
    }
}

/// O espelho do Motion (o acorde Ctrl+Z/Y do grafo) segue a MESMA lei.
#[test]
fn the_motion_key_block_asks_whether_the_keys_are_live() {
    for (name, src) in keyboard_family() {
        assert!(
            !src.contains("self.motion_tool_active()"),
            "`{name}` pergunta `self.motion_tool_active()` cru: o Ctrl+Z do grafo desfaz o \
             MOTION enquanto o artista digita num campo focado. Use `self.motion_keys_live()`."
        );
    }
}

/// **A porta é UMA, e a metade do foco é COMPARTILHADA.** As duas portas diferem só em
/// *qual ferramenta*; se alguém escrever uma 3ª que re-derive a pergunta do foco por conta
/// própria, o dia em que `text_entry_focused` ganhar um 4º tipo de widget deixa uma delas
/// para trás.
#[test]
fn both_doors_are_built_from_the_one_focus_question() {
    let src = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/input_dispatch.rs"
    ))
    .expect("input_dispatch.rs legível");
    for door in ["vector_keys_live", "motion_keys_live"] {
        let at = src
            .find(&format!("fn {door}(&self) -> bool {{"))
            .unwrap_or_else(|| panic!("a porta `{door}` existe"));
        let body = &src[at..(at + 200).min(src.len())];
        assert!(
            body.contains("!self.text_entry_focused()"),
            "`{door}` não consulta `text_entry_focused()`: a porta existe e não faz a \
             pergunta que ela existe para fazer."
        );
    }
}
