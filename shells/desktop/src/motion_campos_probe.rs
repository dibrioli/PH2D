//! ⭐⭐ **A AUDITORIA DO GRUPO DO FOCO — os CAMPOS** (ciclo 4, passo 2 — doc 103 §5).
//!
//! *«Nem todos ao mesmo tempo»*: este grupo é o que decide **quem** um animador ou um
//! deformador afecta, e **quanto**. Um `motion.falloff` sem campo é um interruptor; com campo é
//! um pincel.
//!
//! ⚠️ A régua é a mesma dos ciclos 1–3 e vive numa porta só
//! ([`crate::motion_ciclo_probe`]) — o que muda entre ciclos é a lista de nós, nunca o
//! instrumento.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture audit_the_field_group
//! ```

/// Os sete do ciclo 4 (doc 103 §5).
pub(crate) const GRUPO: [&str; 7] = [
    "motion.falloff",
    "field.box",
    "field.radial_sweep",
    "field.index_range",
    "field.remap",
    "field.combine",
    "field.shape",
];

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_field_group() {
    crate::motion_ciclo_probe::retrato(&GRUPO);
}

/// **OS PARAMS DE CADA NÓ DO GRUPO, um a um** — o que a auditoria compara contra as
/// referências. ⚠️ Sem esta lista, «falta X» é um palpite.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_each_field_offers
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_field_offers() {
    crate::motion_ciclo_probe::params_de(&GRUPO);
}

/// ⭐⭐ **AS ROWS QUE O CARTÃO DE FACTO PINTA** — ver a porta.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture what_the_field_card_shows
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_field_card_shows() {
    crate::motion_ciclo_probe::cartao(&GRUPO);
}
