//! **Arch-gate: o `ClearRun` da §14 descarta a FITA, e não passa pelo fan-out**
//! (W17).
//!
//! ## Por que um gate de TEXTO
//!
//! O `ClearRun` é o único verbo da §14 que não é uma escrita de componente: a
//! fita de entrada mora na shell (`App.player_tape`), não no `PlatformPlayer`.
//! Ele é honrado no laço de ações — o lugar onde `self` é mutável, o mesmo do
//! `Join` da §11 e do eyedropper da §12 —, e esse laço exige `hero`/`sim`/
//! `renderer` mais uma janela: **nenhum gate de unidade o alcança**.
//!
//! ## O modo de falha, e é silencioso
//!
//! Empurrá-lo para o `player_edits` compila e passa o seam (que só prova que o
//! clique levanta o verbo): o `apply_player_edit` o nomeia num braço INERTE, o
//! botão fica pintado, clicável, e não descarta corrida nenhuma.
//!
//! ⚠️ E a segunda asserção existe porque *descartar é idempotente*: espalhado
//! pela seleção ele não corromperia nada HOJE. É precisamente essa forma que
//! apodrece — o Ctrl+V do editor de nós colava duas vezes porque um dispatch
//! duplicado *"nunca tinha importado enquanto todos os verbos eram idempotentes"*.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// O braço do laço de ações que trata uma edição da §14.
fn player_edit_arm() -> &'static str {
    let at = SRC
        .find("EditorAction::InspectorPlayerEdit { entity_bits, edit } =>")
        .expect("o laco de acoes tem de tratar a edicao da §14");
    let rest = &SRC[at..];
    let end = rest
        .find("EditorAction::InspectorWheelEdit")
        .expect("o braco seguinte fecha este");
    &rest[..end]
}

/// **O verbo chega à fita.**
///
/// ⚠️ **Mutação medida:** trocar o corpo do `if` por `{}` deixa o botão pintado e
/// clicável, e **todos** os gates de seam e de comportamento continuam verdes —
/// o clique levanta o verbo, o verbo chega ao laço, e nada acontece.
#[test]
fn the_clear_run_arm_clears_the_tape() {
    let arm = player_edit_arm();
    assert!(
        arm.contains("self.player_tape.clear()"),
        "o `ClearRun` da §14 nao descarta a fita. Braco:\n{arm}"
    );
}

/// **E ele NÃO cai no fan-out por entidade** — o controle positivo do gate acima.
///
/// Sem esta metade, um `self.player_tape.clear()` escrito AO LADO do
/// `player_edits.push` (as duas coisas, no mesmo braço) passaria: a primeira
/// asserção só diz que o texto certo está lá, não que o errado não está.
#[test]
fn the_clear_run_arm_does_not_fan_out() {
    let arm = player_edit_arm();
    let at = arm
        .find("ClearRun")
        .expect("o braco tem de NOMEAR o verbo que intercepta");
    let after = &arm[at..];
    let else_at = after
        .find("} else {")
        .expect("a intercepcao tem de ter um ramo alternativo para o resto da secao");
    assert!(
        !after[..else_at].contains("player_edits.push"),
        "o `ClearRun` tambem foi empurrado para o fan-out por entidade. Braco:\n{arm}"
    );
    assert!(
        after[else_at..].contains("player_edits.push"),
        "o RESTO da §14 deixou de chegar ao fan-out -- a intercepcao engoliu a secao \
         inteira. Braco:\n{arm}"
    );
}

/// **E o scanner acha alguma coisa** — o controle que impede os dois gates acima
/// de ficarem verdes por não terem lido nada.
#[test]
fn the_scanner_finds_the_arm() {
    let arm = player_edit_arm();
    assert!(
        arm.len() > 200,
        "o scanner leu {} bytes: o braco mudou de forma e os gates acima deixaram de \
         olhar para o produto",
        arm.len()
    );
}
