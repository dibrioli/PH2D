//! ⛔⛔ **O GESTO DA PEÇA ACRESCENTADA TEM BRAÇO NO DRENO** (ADR-0164 / F5.11).
//!
//! # Porque este gate é textual, e porque ele é preciso
//!
//! O `match` que consome o `EditorAction` na `render_loop` termina num `_ => {}`: uma acção **nova
//! sem braço compila, corre e não faz nada**. É a primeira das duas espécies de controlo morto que
//! a caça de 2026-08-30 nomeou, e **nenhum gate de registo a apanha** — um seam de painel prova que
//! o clique chega ao *barramento*, nunca que alguém do outro lado o lê. Os irmãos deste ficheiro
//! são o `the_unused_override_gestures_reach_the_verb` e o `the_apply_ladder_has_one_door`.
//!
//! ⚠️ **Ele descasca comentários antes de varrer** — um censo textual que não separa prosa de
//! código mente nos dois sentidos, e esta linha já o pagou.

use std::path::Path;

fn code_of(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    let body = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O botão do cartão chega à porta que põe a peça na receita.**
///
/// **Mutação que deve sangrar:** apagar o braço do `InspectorApplyAddedPiece`, ou fazê-lo chamar
/// outra porta.
#[test]
fn the_apply_added_gesture_has_an_arm_and_its_own_door() {
    let body = code_of("render_loop/mod.rs");
    assert!(
        body.contains("EditorAction::InspectorApplyAddedPiece"),
        "a accao nao tem braco no dreno — o `_ => {{}}` do fim do match come-a em silencio"
    );
    assert!(
        body.contains("instance_added::promote("),
        "o braco nao chama a porta que promove a peca — o clique morre a um passo do efeito"
    );
}

/// ⛔⛔ **A CHAVE atravessa: o dreno resolve o `StableId`, e nunca os bits que o cartão viu.**
///
/// Entre o clique e o ponto de aplicação pode ter corrido um Ctrl+Z, que **respawna tudo com bits
/// novos**. Um braço que fizesse `Entity::from_bits` sobre o que o painel mandou apontaria para uma
/// entidade morta — ou, pior, para outra que nasceu no lugar dela.
///
/// **Mutação que deve sangrar:** trocar o `entity_for_stable_id` por um `Entity::from_bits` do
/// campo.
#[test]
fn the_arm_resolves_the_piece_by_identity_not_by_bits() {
    let body = code_of("render_loop/mod.rs");
    let arm = body
        .find("if let Some(piece) = apply_added")
        .expect("o braco adiado");
    let tail = &body[arm..arm + 1200.min(body.len() - arm)];
    assert!(
        tail.contains("entity_for_stable_id("),
        "o braco resolve a peca por outra via que nao a identidade — um Ctrl+Z entre o clique e \
         este ponto troca todos os bits"
    );
}

/// ⛔ **Todo caminho negativo FALA.** Um botão que come o clique em silêncio é pior que um ausente
/// — a lei que o menu dos verbos já paga, e que o report de 2026-09-05 cobrou.
#[test]
fn every_refusal_of_the_apply_added_gesture_has_a_voice() {
    let body = code_of("render_loop/mod.rs");
    let arm = body
        .find("if let Some(piece) = apply_added")
        .expect("o braco adiado");
    // ⛔⛔ **Era uma janela de 1600 BYTES, e um número de bytes é uma agulha cujo sentido depende
    // do COMPRIMENTO DOS NOMES.** Ao mudar `crate::instance_added::` para
    // `ph2d_app_components::instance_added::` (+17 caracteres por ocorrência) os dois toasts deste
    // braço passaram de dentro para fora da janela, e o gate reprovou sobre produto **correcto**.
    // ⛔ Alargar o número seria pior do que deixá-lo: medido, o braço tem `1816` bytes e os toasts
    // dele estão a `1498` e `1727` — mas há outro par a `3573`/`3816`, de um braço VIZINHO. Uma
    // janela de `4000` ficaria verde a contar os toasts de outra pessoa.
    // ⇒ a fatia passa a ser **o braço**, fechado na chaveta à indentação dele.
    let fim = body[arm..]
        .find("\n            }")
        .expect("o braco deixou de fechar a esta indentacao — reancore este censo");
    let tail = &body[arm..arm + fim];
    assert!(
        tail.matches("Toast::warning").count() >= 2,
        "as recusas do gesto nao falam — `NotAdded` e o resto tem de dizer coisas diferentes"
    );
}
