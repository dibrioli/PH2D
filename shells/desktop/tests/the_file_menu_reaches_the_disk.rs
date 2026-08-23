//! **A outra metade do menu Ficheiro** — o painel levanta a bandeira, e ELE tem de a drenar.
//!
//! ⚠️ Os gates do lado do painel vivem em
//! `crates/ph2d-editor-core/tests/the_file_menu_items_are_not_mute.rs`, e afirmam o que o clique
//! PROMETE. Este afirma que a promessa chega ao disco — e ele é um arch-gate de FONTE porque a
//! travessia real não é alcançável: o `drain_project_io` sai no `gfx.is_none()` de um `App` sem
//! janela, e o seletor nativo abre uma janela modal do sistema.
//!
//! ⛔ Sem ele, os três itens voltariam a ser mudos com um `git revert` de uma linha, e nada
//! reprovaria — foi exactamente esse o estado até 2026-08-23.

mod sculpt_source;
use sculpt_source::{function_body, source};

/// **As três bandeiras são DRENADAS** — e cada uma leva ao seu gesto.
///
/// ⚠️ A asserção é sobre as três juntas: drenar duas e esquecer a terceira deixa uma bandeira
/// **presa a `true`**, e a partir daí o app grava (ou abre) uma vez por quadro.
#[test]
fn every_file_menu_flag_is_drained() {
    let body = function_body(&source("project_io.rs"), "drain_project_io");
    for flag in ["asked.save", "asked.save_as", "asked.open"] {
        assert!(
            body.contains(flag),
            "o `drain_project_io` nao le' `{flag}` — a bandeira fica presa e o gesto repete uma vez \
             por quadro"
        );
    }
    assert!(
        body.contains("mem::take"),
        "as bandeiras tem de ser CONSUMIDAS (mem::take), nao so' lidas"
    );
    assert!(
        body.contains("project_save_gesture") && body.contains("project_open_gesture"),
        "…e cada uma tem de chamar o gesto, nao so' baixar a bandeira"
    );
}

/// **O TECLADO e o MENU chamam a mesma função.**
///
/// ⚠️ Duas portas para o mesmo gesto é como o `.ase` ficou invisível no diálogo de import no mesmo
/// dia (`crate::import_router`), e aqui a divergência seria pior: um `Save` do menu a gravar
/// noutro sítio que o `Ctrl+S`. O gate lê o handler de teclado e exige que ele passe pelas mesmas
/// duas portas — não pelo `project_save`/`project_load` crus, que não perguntam.
#[test]
fn the_keyboard_goes_through_the_same_door_as_the_menu() {
    let src = source("input_dispatch/keyboard_files.rs");
    let body = function_body(&src, "file_chords");
    assert!(
        body.contains("project_save_gesture"),
        "o Ctrl+S tem de passar pela porta que decide ONDE gravar"
    );
    assert!(
        body.contains("project_open_gesture"),
        "e o Ctrl+O pela que PERGUNTA — abrir deita fora o trabalho nao gravado"
    );
    assert!(
        body.contains("shift_key"),
        "o Ctrl+Shift+S (Save As) tem de perguntar o modificador, senao os dois gestos ficam \
         indistinguiveis"
    );
}
