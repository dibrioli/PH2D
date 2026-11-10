//! O emboss da tinta sem óleo pela LUZ do cartão (B14.4): os pesos do gradiente da massa rodam com o azimute.
//!
//! Os pesos de sempre `(0,5, 1)` (x direita, y baixo) são a omissão do cartão (`225°`, topo-esquerda); outro
//! azimute roda-os do mesmo ângulo, na moldura do [`crate::oleo_real::OleoReal::azimute`] — o MESMO comprimento.

/// O azimute de fábrica do cartão ([`crate::oleo_real::OleoReal`]), em radianos.
pub const AZIMUTE_DE_FABRICA: f64 = 225f64.to_radians();

/// Os pesos `(x, y)` do emboss com a luz em `azimute` (radianos, x direita, y baixo). Na omissão, os literais.
#[must_use]
pub fn pesos_do_relevo(azimute: f64) -> [f64; 2] {
    if azimute.to_bits() == AZIMUTE_DE_FABRICA.to_bits() {
        return [0.5, 1.0];
    }
    let theta = libm::atan2(1.0, 0.5) + (azimute - AZIMUTE_DE_FABRICA);
    let g = libm::sqrt(1.25);
    [g * libm::cos(theta), g * libm::sin(theta)]
}

#[cfg(test)]
#[path = "relevo_da_luz_tests.rs"]
mod tests;
