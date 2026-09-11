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
    // ⚠️⚠️ **E o nome da função também mudou** (2026-09-04): o `apply_project` passou a ser um
    // invólucro de uma linha sobre o `apply_project_with`, que é quem faz o trabalho — porque o
    // passo passou a poder TRAZER a selecção (ver o 2.º gate deste ficheiro). Um `split` pelo nome
    // antigo apanhava o invólucro e afirmava sobre **três linhas**, todas verdes, sobre nada.
    // *É a segunda vez que este gate lê o sítio errado, e as duas por o alvo se ter mudado.*
    let body = src
        .split("pub(crate) fn apply_project_with")
        .nth(1)
        .expect("apply_project_with existe")
        .split("\n    pub(crate) fn ")
        .next()
        .expect("corpo de apply_project_with");

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

/// ⭐⭐⭐ **ARCH-GATE: o PASSO traz a selecção, e o restauro PREFERE-A ao transporte.**
///
/// # ⛔⛔ O report que ele fecha (Enio, 2026-09-04: *«o undo/redo está completamente destruído»*)
///
/// O irmão acima defende o **transporte** — ler a selecção antes do restauro e devolver o que
/// sobreviveu —, e isso é correcto para uma EDIÇÃO e falso para uma CRIAÇÃO: desfazer apaga a
/// selecção porque o objecto deixou de existir, e **refazer transporta a de agora, que está
/// vazia**. Medido pela sonda `PH2D_FIELD_UNDO_PROBE=1`: a forma voltava e o gizmo **não**
/// (`sel=None setas=0`), e daí em diante todo `Ctrl+Z` parecia não fazer nada.
///
/// ⚠️ **Este gate é sobre o FONTE pela mesma razão do irmão** — o `apply_undo` exige `gfx`.
#[test]
fn the_step_carries_the_selection_and_the_restore_prefers_it() {
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/undo_app.rs"))
        .expect("undo_app.rs legível");
    let apply_undo = src
        .split("pub(crate) fn apply_undo")
        .nth(1)
        .expect("apply_undo existe")
        .split("\n    pub(crate) fn ")
        .next()
        .expect("corpo de apply_undo");
    assert!(
        apply_undo.contains("field_selection_mark()"),
        "o `apply_undo` deixou de ler a selecção de AGORA — sem ela o lado que vai para a outra \
         fila fica sem mão, e refazer devolve a peça sem gizmo"
    );
    assert!(
        apply_undo.contains("Some(&mark)"),
        "o `apply_undo` restaura sem passar a marca do passo — volta ao transporte, e refazer uma \
         criação devolve o objecto e não devolve a selecção"
    );

    let corpo = src
        .split("pub(crate) fn apply_project_with")
        .nth(1)
        .expect("apply_project_with existe")
        .split("\n    pub(crate) fn ")
        .next()
        .expect("corpo de apply_project_with");
    assert!(
        corpo.contains("Some(m) => m.field.clone()"),
        "o `apply_project_with` recebe a marca do passo e não a usa — o argumento fica decorativo"
    );

    // ⚠️ **E a marca do baseline é re-armada no restauro, como o próprio baseline** — sem isto o
    // passo SEGUINTE é empurrado com a selecção de antes do restauro.
    assert!(
        corpo.contains("undo_baseline_selection"),
        "o restauro re-arma o `undo_baseline` e não a marca dele — o Ctrl+Z a seguir devolve a mão \
         ao objecto errado"
    );

    let post = src
        .split("pub(crate) fn post_frame_undo")
        .nth(1)
        .expect("post_frame_undo existe");
    assert!(
        post.contains("push_undo(base, mark)"),
        "o passo nasce sem a selecção que lhe pertence — a fila volta a ser só estado"
    );
}
