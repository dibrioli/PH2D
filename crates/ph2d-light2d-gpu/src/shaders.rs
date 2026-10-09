//! As fontes WGSL. Não há `#include` no repo: cada módulo é a CONCATENAÇÃO dos ficheiros que usa
//! (`concat!(include_str!…)`), nunca uma segunda cópia do texto.

use crate::config::Precisao;

/// A gêmea WGSL da `ph2d_light2d_ref::queda::queda` (gate G12).
pub const QUEDA_WGSL: &str = include_str!("wgsl/queda.wgsl");

pub(crate) const C0: &str = concat!(
    include_str!("wgsl/c0_dono.wgsl"),
    include_str!("wgsl/c0_captura.wgsl")
);
/// A C0 dos sprites por texel (W3): a MESMA lei do dono à frente.
pub(crate) const C0_SPRITE: &str = concat!(
    include_str!("wgsl/c0_dono.wgsl"),
    include_str!("wgsl/c0_sprite.wgsl")
);
/// O texto da lei do dono — o censo (um texto, dois consumidores) lê-o.
pub const C0_DONO_WGSL: &str = include_str!("wgsl/c0_dono.wgsl");
pub(crate) const C1: &str = include_str!("wgsl/c1_jfa.wgsl");

macro_rules! rc {
    ($real:literal, $saida:literal) => {
        concat!(
            include_str!($real),
            include_str!($saida),
            include_str!("wgsl/queda.wgsl"),
            include_str!("wgsl/c2_rc.wgsl")
        )
    };
}

const RC_F16_F16: &str = rc!("wgsl/real_f16.wgsl", "wgsl/saida_f16.wgsl");
const RC_F16_F32: &str = rc!("wgsl/real_f16.wgsl", "wgsl/saida_f32.wgsl");
const RC_F32_F16: &str = rc!("wgsl/real_f32.wgsl", "wgsl/saida_f16.wgsl");
const RC_F32_F32: &str = rc!("wgsl/real_f32.wgsl", "wgsl/saida_f32.wgsl");

/// O módulo das cascatas para `(aritmética, armazenamento)`.
#[must_use]
pub fn rc(aritmetica: Precisao, armazenamento: Precisao) -> &'static str {
    match (aritmetica, armazenamento) {
        (Precisao::F16, Precisao::F16) => RC_F16_F16,
        (Precisao::F16, Precisao::F32) => RC_F16_F32,
        (Precisao::F32, Precisao::F16) => RC_F32_F16,
        (Precisao::F32, Precisao::F32) => RC_F32_F32,
    }
}
