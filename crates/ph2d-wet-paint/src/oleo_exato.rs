//! As operações EXACTAS do IEEE 754 (arredondar, chão, tecto e a raiz, correctamente arredondada): o mesmo bit em
//! todo o SO pela própria norma — a lei do porte só pede o `libm` para as TRANSCENDENTES. As do `libm` são em
//! software e eram ~20 % do tempo do óleo por quadro (perf, J6: `round` 11 %, `from_parts` 7 %).

#[inline(always)]
pub(crate) fn round(v: f64) -> f64 {
    v.round()
}
#[inline(always)]
pub(crate) fn floor(v: f64) -> f64 {
    v.floor()
}
#[inline(always)]
pub(crate) fn ceil(v: f64) -> f64 {
    v.ceil()
}
#[inline(always)]
pub(crate) fn sqrt(v: f64) -> f64 {
    v.sqrt()
}
