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
    // ⚠️⚠️ **O PAR, não um ficheiro.** A `App` que opera a fila mudou-se para o irmão
    // `undo_app.rs` na integração de 2026-09-04 (tecto de LOC estourado pela SOMA de duas
    // linhas), e todo gate que lia só `undo.rs` ficou a afirmar sobre o ficheiro errado — em
    // silêncio no dia seguinte, se a lei ainda lá estivesse. ⇒ *um gate que PARSEIA o fonte lê
    // a família inteira, nunca um nome de ficheiro.*
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/undo_app.rs"))
        .expect("undo_app.rs legível");
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
    // ⭐⭐⭐ **E a METADE 3D** (W113): o `apply_project` limpava a seleção inteira e devolvia só a
    // vetorial, então **todo `Ctrl+Z` apagava a seleção do modelador** e o gizmo desaparecia — o
    // report do Enio de 2026-09-03. ⚠️ Sem esta metade, os gates da lei pura ficam verdes sobre uma
    // política que ninguém chama, que é exactamente o buraco que este arquivo existe para fechar.
    assert!(
        body.contains("field_selection_ids"),
        "o `apply_project` deixou de guardar a seleção 3D em identidade durável — todo undo volta \
         a apagá-la, e o gizmo do modelador desaparece a cada Ctrl+Z"
    );
    assert!(
        body.contains("field_selection_back"),
        "o `apply_project` guarda a seleção 3D e não a devolve"
    );
    let captura_3d = body
        .find("field_selection_ids")
        .expect("captura a seleção 3D");
    let devolve_3d = body
        .find("field_selection_back")
        .expect("devolve a seleção 3D");

    // E a ordem importa: a captura tem de vir ANTES do `restore`, senão ela lê o pen já zerado.
    let capture = body
        .find("selected_paths")
        .expect("captura a seleção prévia");
    let restore = body.find("state.restore").expect("chama o restore");
    assert!(
        capture < restore,
        "a seleção prévia é lida DEPOIS do restore — nesse ponto ela já não existe"
    );
    // ⚠️ A mesma lei para a 3D, e ela é mais apertada: a captura tem de ler os bits ANTES de o
    // `restore` despawnar as entidades, e a devolução tem de correr DEPOIS.
    assert!(
        captura_3d < restore && restore < devolve_3d,
        "a seleção 3D é guardada ou devolvida do lado errado do `restore` — capturar depois lê \
         entidades mortas, devolver antes escreve bits que o respawn vai invalidar"
    );
}
