//! **A shell estampa a DIREÇÃO do relógio antes do apply** (Enio, 2026-08-01: *"a direção do
//! easing é invertida se a playhead está voltando a zero"*).
//!
//! ⚠️ **Arch-gate porque nenhum teste de unidade alcança esta costura:** o `Playhead` sabe a
//! direção, o avaliador não; o doc a carrega como transiente e é a `timeline_bridge` que faz
//! a ponte, dentro de uma função que exige janela. Os gates de motor podem ficar todos verdes
//! com a estampa DELETADA — eles chamam `set_reverse_play` à mão —, e o produto nunca
//! inverteria nada.

const SRC: &str = include_str!("../src/render_loop/timeline_bridge.rs");

/// A estampa existe, sai do `Playhead`, e vem ANTES das três rotas de apply.
#[test]
fn the_bridge_stamps_the_direction_from_the_playhead_before_applying() {
    let stamp = SRC
        .find("set_reverse_play(")
        .expect("a bridge tem de estampar a direção do relógio no doc");
    let arg_end = SRC[stamp..]
        .find(");")
        .map(|e| stamp + e)
        .expect("a chamada fecha");
    let arg = &SRC[stamp..arg_end];
    assert!(
        arg.contains("is_advancing_forward"),
        "a direção sai do Playhead, não de um palpite: {arg}"
    );
    assert!(
        arg.contains("is_playing"),
        "…e PAUSADO é para a frente: um scrub para trás é leitura, não reprodução: {arg}"
    );
    // As três rotas de apply — a estampa precede TODAS: o doc já tem de saber a direção
    // quando o avaliador pergunta.
    for apply in [
        "apply_container(",
        "apply_active_clip(",
        "apply_scene(world",
    ] {
        let at = SRC.find(apply).unwrap_or_else(|| panic!("{apply} sumiu"));
        assert!(
            stamp < at,
            "a estampa tem de vir antes de {apply} (estampa {stamp}, apply {at})"
        );
    }
}
