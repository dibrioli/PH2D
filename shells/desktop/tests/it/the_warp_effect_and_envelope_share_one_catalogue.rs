//! **Gate de PARIDADE: o efeito Warp e o Envelope leem UM catálogo** (Enio 2026-07-25).
//!
//! Os estilos de warp viviam em duas listas que divergiram (o "Wave" de um era o "Flag" do outro;
//! um tinha Fisheye/Rise, o outro ArcUpper/ArcLower/Flag/Squeeze). Agora os dois são apelidos do
//! MESMO enum, [`ph2d_warp_style::WarpStyle`] — então não podem mais divergir.
//!
//! Este gate prova isso em DOIS níveis:
//!
//! 1. **Compilação** — a anotação `&[WarpStyle]` (o tipo do EFEITO) sobre `EnvelopeWarp::ALL` (o
//!    tipo do ENVELOPE) só compila se forem o MESMO tipo. Re-definir um como enum próprio quebra
//!    aqui, na hora, sem depender de ninguém rodar o teste.
//! 2. **Valor** — as listas e os rótulos batem, e o menu Add do efeito Warp lista exatamente os
//!    rótulos do catálogo (a porção warp).

use ph2d_vec_scene::effect::PathEffect;
use ph2d_vec_scene::fx_warp_presets::WarpStyle;

/// **As duas seções são o MESMO catálogo, com os MESMOS 9 estilos na MESMA ordem.**
#[test]
fn the_two_sections_read_the_same_catalogue() {
    // ⚠️ A prova de COMPILAÇÃO: `WarpStyle` é o tipo do efeito; `EnvelopeWarp::ALL` é do envelope.
    // Anotar um como o outro só passa o borrow-check se forem o mesmo tipo (os dois são apelidos de
    // `ph2d_warp_style::WarpStyle`). Um re-fork de qualquer lado vira erro de tipo aqui.
    let effect: &[WarpStyle] = WarpStyle::ALL;
    let envelope: &[WarpStyle] = ph2d_ecs::EnvelopeWarp::ALL;
    assert_eq!(
        effect, envelope,
        "as duas seções não oferecem a mesma lista de estilos"
    );
    assert_eq!(effect.len(), 9, "o catálogo devia ter 9 estilos");
}

/// **O menu Add do efeito Warp lista exatamente os rótulos do catálogo** — os rótulos de
/// `WarpStyle::ALL` aparecem em `PathEffect::KINDS` **contíguos, na ordem, exatamente uma vez**.
/// Um typo em `KINDS` ou uma reordenação de `WarpStyle` sangra aqui.
///
/// ⚠️ **A janela é LOCALIZADA, nunca posicional.** A versão anterior lia a CAUDA de `KINDS`
/// (`&kinds[kinds.len() - labels.len()..]`) — verdade só enquanto o warp fosse a ÚLTIMA família,
/// e essa premissa expirou no instante em que Falloff/Twist/Knot foram apendados depois dele,
/// na MESMA linha. O produto estava certo (o bloco segue contíguo em `WARP_BASE = 4`); quem
/// envelheceu foi o proxy — e ele já contradizia o próprio docstring, que dizia *"depois dos 4
/// base"* enquanto o código media do fim. Achar o bloco em vez de assumir onde ele está torna o
/// gate imune ao próximo apêndice, sem afrouxar nada ([[reference_topic_gate_discipline]]).
#[test]
fn the_add_menu_matches_the_catalogue() {
    let labels: Vec<&'static str> = WarpStyle::ALL.iter().map(|s| s.label()).collect();
    let kinds = PathEffect::KINDS;
    assert!(
        kinds.len() >= labels.len(),
        "KINDS tem menos entradas que estilos de warp"
    );
    let occurrences = kinds
        .windows(labels.len())
        .filter(|w| *w == labels.as_slice())
        .count();
    assert_eq!(
        occurrences, 1,
        "o catálogo do Warp tem de aparecer em KINDS contíguo, na ordem de WarpStyle::ALL e \
         EXATAMENTE uma vez — achei {occurrences}.\n  KINDS     = {kinds:?}\n  catálogo  = {labels:?}"
    );
}
