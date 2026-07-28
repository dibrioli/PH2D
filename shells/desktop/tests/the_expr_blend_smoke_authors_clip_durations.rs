//! **Arch-gate: o smoke do expr-blend AUTORA a duração dos seus clips — a veil aparece na aba Keys.**
//!
//! ## O defeito (Enio, três rodadas: 2026-07-27)
//!
//! *"o véu de duração ainda está invisível até mudar o valor na caixa de duração do clip. Deixe-o
//! sempre visível desde a abertura do app."*
//!
//! Medido pela porta real (`TimelineViewSnapshot::rebuild` + `beyond_end_shade`): **o produto já está
//! certo** — o boot usa `TimelineState::with_default_duration()` (o clip 0 e a cena nascem 4 s) e todo
//! clip feito pela UI ganha 4 s do intent `AddClip` (`intent_apply.rs`), então plain boot e clips do
//! artista mostram a veil nas DUAS abas. O que NÃO mostrava era o SMOKE: ele constrói os clips com o
//! `doc.add_clip` CRU (a camada de DADOS, DERIVADA de propósito — pinado por
//! `state::default_duration_tests`), então na aba Keys o clip ativo abria sem `length_override` e sem
//! veil. O artista testava pelo smoke e via o produto errado.
//!
//! ## Por que um gate de TEXTO
//!
//! A cena do smoke é código de shell atrás de `PH2D_EXPR_BLEND_SMOKE`, e roda dentro do `App` (precisa
//! de janela) — nenhum unit test a alcança. O comportamento do produto já é gateado noutro lugar
//! (`state::default_duration_tests::the_product_default_is_a_four_second_authored_composition` e
//! `timeline_persist::...opens_with_the_default_four_second_composition`); este lê a fonte do smoke e
//! afirma que ele autora as durações dos clips que cria — a única forma de o smoke representar o
//! produto em vez de um doc derivado. É a mesma disciplina do `the_smokes_open_the_painter_in_digital`.

const SMOKE: &str = include_str!("../src/expr_blend_smoke.rs");

/// A cena runtime do smoke estampa `length_override` nos clips que ela cria com `add_clip` cru — sem
/// isso a aba Keys abre o clip ativo SEM veil (o defeito reportado).
///
/// **Mutação que deve sangrar:** apagar o laço `for i in 0..clip_count {
/// doc.set_clip_length_override(i, Some(6.0)); }` da cena runtime — os clips voltam a derivados e a
/// veil some na aba Keys.
#[test]
fn the_runtime_scene_authors_its_clip_durations() {
    assert!(
        SMOKE.contains("set_clip_length_override(i, Some(6.0))"),
        "a cena runtime do expr-blend smoke não autora mais a duração dos seus clips — o `doc.add_clip` \
         cru deixa cada clip DERIVADO, e a aba Keys abre o clip ativo sem veil (o defeito das três \
         rodadas do Enio). Estampe `set_clip_length_override` em cada clip que a cena cria."
    );
}

/// Controle positivo: a cena runtime AINDA constrói os clips e a cena, para o gate acima não passar por
/// vacuidade se alguém esvaziar o arquivo ou remover a construção.
#[test]
fn the_runtime_scene_still_builds_clips_and_a_scene() {
    assert!(
        SMOKE.contains("doc.add_clip(") && SMOKE.contains("doc.set_scene_length(Some(6.0))"),
        "o expr-blend smoke perdeu a construção de clips/cena — o gate acima ficaria verde por \
         vacuidade; se o smoke foi reescrito, atualize este gate de propósito"
    );
}
