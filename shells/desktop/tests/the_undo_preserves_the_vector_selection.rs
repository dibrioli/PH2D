//! **ARCH-GATE: o `apply_project` tem de PRESERVAR a seleção vetorial.**
//!
//! O `apply_project` não é alcançável por um teste headless (exige `gfx` = janela + GPU), então a
//! política vive numa função pura com gates próprios. Isso deixa um buraco: alguém pode voltar a
//! zerar o `vec_pen` dentro do `apply_project` e **todos** aqueles gates continuam verdes, sobre
//! uma política que já não é chamada.
//!
//! Este gate fecha o buraco lendo o FONTE — o mesmo padrão do arch-gate de precificação do áudio.
//! Ele existe por um bug reportado: *"o undo faz os pins sumirem, embora ainda funcionando"*. O
//! envelope segue a deformar (o recook varre por QUERY) e o overlay some com a seleção.

use std::fs;

#[test]
fn apply_project_restores_the_pen_selection_after_the_restore() {
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/undo.rs"))
        .expect("undo.rs legível");
    let body = src
        .split("pub(crate) fn apply_project")
        .nth(1)
        .expect("apply_project existe")
        .split("\n    pub(crate) fn ")
        .next()
        .expect("corpo de apply_project");

    assert!(
        body.contains("surviving_selection"),
        "o `apply_project` deixou de chamar `surviving_selection` — a seleção morre no undo e o \
         overlay do envelope (gaiola e pinos) fica invisível com a ferramenta funcionando"
    );
    assert!(
        body.contains("select_many"),
        "o `apply_project` calcula a seleção sobrevivente e não a devolve ao pen"
    );
    // E a ordem importa: a captura tem de vir ANTES do `restore`, senão ela lê o pen já zerado.
    let capture = body
        .find("selected_paths")
        .expect("captura a seleção prévia");
    let restore = body.find("state.restore").expect("chama o restore");
    assert!(
        capture < restore,
        "a seleção prévia é lida DEPOIS do restore — nesse ponto ela já não existe"
    );
}
