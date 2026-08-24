//! Gates dos **QUATRO ESTADOS DO CLAMP** (doc 89, folha 10).
//!
//! ⚠️ **A cura foi um ENUM no param que já existia, e não um `clamp_max`
//! apendado** — com um param novo, `clamp` teria de significar *o piso*, e toda
//! cena salva com `Clamp` DESLIGADO passaria a ter o teto ligado. A escada
//! (`0 = Off`, `1 = Both`) é o que faz todo documento já autorado ler o que
//! escreveu.
//!
//! Separado do `tests.rs` pelo tecto de LOC (HR-18).

use super::*;

/// ⭐ **A CÉLULA, medida:** *"segure o piso, deixe o teto voar"* era inexprimível
/// com um `bool`, e é exactamente o que a lei de espaço linear do `field.combine`
/// pede — somar dois campos passa de `1`, e cortar ali destrói o que a soma
/// carregava.
#[test]
fn min_only_holds_the_floor_and_lets_the_ceiling_fly() {
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "multiplier", 2.0);
    // Com `min = −0.5` e `max = 1`, a entrada `0.8` mapeia em `(−0.5 + 0.8·1.5)·2
    // = 1.4`, e a entrada `0` em `−1.0`.
    g.set_param(rm, "min", -0.5);
    g.set_param(rm, "clamp", 2.0); // Min Only
    let got = falloff_of(&g, &Ops::falloff(vec![0.0, 0.8]), rm);
    assert!(approx(got[0], 0.0), "o PISO tem de segurar: {}", got[0]);
    assert!(approx(got[1], 1.4), "o TETO tem de voar: {}", got[1]);
}

/// E o simétrico, que a mesma escada dá de graça.
#[test]
fn max_only_holds_the_ceiling_and_lets_the_floor_fly() {
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "multiplier", 2.0);
    g.set_param(rm, "min", -0.5);
    g.set_param(rm, "clamp", 3.0); // Max Only
    let got = falloff_of(&g, &Ops::falloff(vec![0.0, 0.8]), rm);
    assert!(got[0] < -1e-3, "o PISO tem de voar: {}", got[0]);
    assert!(approx(got[1], 1.0), "o TETO tem de segurar: {}", got[1]);
}

/// ⭐⭐ **A ESCADA PRESERVA TODO DOCUMENTO JÁ AUTORADO**, e isto é a razão de a
/// cura ser um enum no param que já existia em vez de um `clamp_max` apendado.
///
/// ⚠️ Com um param novo, `clamp` teria de significar *o piso* — e toda cena salva
/// com `Clamp` DESLIGADO passaria a ter o teto ligado, cortando em silêncio o
/// excesso que ela desenhava. Aqui `0` continua a ser *nenhum* e `1` continua a
/// ser *os dois*.
#[test]
fn the_two_states_a_saved_document_can_hold_mean_what_they_always_meant() {
    let (mut g, rm) = chain();
    linear(&mut g, rm);
    g.set_param(rm, "multiplier", 2.0);
    g.set_param(rm, "min", -0.5);

    g.set_param(rm, "clamp", 1.0); // o default de sempre
    let both = falloff_of(&g, &Ops::falloff(vec![0.0, 0.8]), rm);
    assert!(approx(both[0], 0.0) && approx(both[1], 1.0), "{both:?}");

    g.set_param(rm, "clamp", 0.0); // o «desligado» de sempre
    let off = falloff_of(&g, &Ops::falloff(vec![0.0, 0.8]), rm);
    assert!(
        off[0] < -1e-3,
        "desligado tinha de deixar o piso voar: {off:?}"
    );
    assert!(approx(off[1], 1.4), "e o teto tambem: {off:?}");
}
