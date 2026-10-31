//! **Caminho B (rodada 10, `docs/Painter/49_realshader/MEDIDA_31_10_RODADA10.md`): a RÉGUA de tolerância
//! dos planos estreitos.** Antes de trocar o tipo de `vel_x/y`, `film`, `susp`, `susp_rgb` em todo o
//! crate, mede-se o que essa troca faria à cor final: o override EM EXECUÇÃO [`ESTREITO`] arredonda
//! esses planos ao formato estreito (guardar estreito e reler = arredondar) depois da advecção e no fim
//! de cada passo. `0` (produto) = nada muda, ao bit. Os bits escolhem o formato por grupo de planos.

use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;

use crate::grid::Grid;

/// `vel_x`/`vel_y` em `f16` (IEEE binário16, arredondamento ao par, subnormais incluídos).
pub const VEL_F16: usize = 1;
/// `vel_x`/`vel_y` em `bf16` (8 bits de expoente, 7 de mantissa).
pub const VEL_BF16: usize = 2;
/// `film`/`susp` em `f16`.
pub const MASSA_F16: usize = 4;
/// `susp_rgb` (`0..255`) em `u16` de ponto fixo, passo `1/256`.
pub const RGB_U16: usize = 8;
/// `film`/`susp` em `u16` de ponto fixo, passo `1/`[`ESC_MASSA_U16`] (satura em `65535/ESC`).
pub const MASSA_U16: usize = 16;
/// `susp_rgb` em `f16` (relativo: passo de `0,125` perto de 255).
pub const RGB_F16: usize = 32;
/// CONTROLO: `vel_x`/`vel_y` com 1 bit de mantissa a menos (22) — quanto vale UM ulp de perturbação.
pub const VEL_ULP: usize = 64;
/// Escala do `u16` de `film`/`susp`: `2^14` ⇒ teto `4,0`, passo `6,1e-5`.
pub const ESC_MASSA_U16: f32 = 16384.0;

/// Override da régua; `0` = produto (nenhum arredondamento).
pub static ESTREITO: AtomicUsize = AtomicUsize::new(0);

/// Arredonda `v` a `f32` com `keep` bits de mantissa (ao par). NaN/inf não ocorrem no estado.
#[inline]
fn mant(v: f32, keep: u32) -> f32 {
    let drop = 23 - keep;
    let b = v.to_bits();
    let lsb = (b >> drop) & 1;
    let r = b.wrapping_add((1u32 << (drop - 1)) - 1 + lsb);
    f32::from_bits(r & !((1u32 << drop) - 1))
}

/// O valor que um `f16` guardaria.
#[inline]
pub fn f16(v: f32) -> f32 {
    let a = v.abs();
    if a == 0.0 {
        return v;
    }
    if a >= 65504.0 {
        return 65504.0f32.copysign(v);
    }
    if a < 6.103_515_6e-5 {
        // Subnormal do f16: passo fixo 2^-24 (divisão exacta por potência de 2).
        let q = 5.960_464_5e-8_f32;
        return (v / q).round_ties_even() * q;
    }
    mant(v, 10)
}

/// O valor que um `bf16` guardaria.
#[inline]
pub fn bf16(v: f32) -> f32 {
    mant(v, 7)
}

/// O valor que um `u16` de ponto fixo com escala `esc` guardaria (satura em `0..65535`).
#[inline]
pub fn u16_fixo(v: f32, esc: f32) -> f32 {
    (v * esc).round().clamp(0.0, 65535.0) / esc
}

fn plano(a: &mut [f32], f: impl Fn(f32) -> f32 + Sync) {
    a.par_iter_mut().for_each(|v| *v = f(*v));
}

/// Aplica o formato estreito pedido por [`ESTREITO`] aos planos do grid. `0` = no-op.
pub fn arredonda(g: &mut Grid) {
    let m = ESTREITO.load(Ordering::Relaxed);
    if m == 0 {
        return;
    }
    if m & VEL_F16 != 0 {
        plano(&mut g.vel_x, f16);
        plano(&mut g.vel_y, f16);
    } else if m & VEL_ULP != 0 {
        plano(&mut g.vel_x, |v| mant(v, 22));
        plano(&mut g.vel_y, |v| mant(v, 22));
    } else if m & VEL_BF16 != 0 {
        plano(&mut g.vel_x, bf16);
        plano(&mut g.vel_y, bf16);
    }
    if m & MASSA_F16 != 0 {
        plano(&mut g.film, f16);
        plano(&mut g.susp, f16);
    } else if m & MASSA_U16 != 0 {
        plano(&mut g.film, |v| u16_fixo(v, ESC_MASSA_U16));
        plano(&mut g.susp, |v| u16_fixo(v, ESC_MASSA_U16));
    }
    if m & (RGB_U16 | RGB_F16) != 0 {
        let u = m & RGB_U16 != 0;
        g.susp_rgb.par_iter_mut().for_each(|c| {
            for v in c.iter_mut() {
                *v = if u { u16_fixo(*v, 256.0) } else { f16(*v) };
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_formatos_estreitos_guardam_o_que_o_formato_guarda() {
        assert_eq!(f16(1.0), 1.0);
        assert_eq!(f16(1.0 + 1.0 / 2048.0), 1.0); // empate → par
        assert_eq!(f16(1.0 + 3.0 / 2048.0), 1.0 + 2.0 / 1024.0);
        assert_eq!(f16(1e-7), 5.960_464_5e-8 * 2.0);
        assert_eq!(f16(1e6), 65504.0);
        assert_eq!(bf16(1.0 + 1.0 / 256.0), 1.0);
        assert_eq!(u16_fixo(254.999, 256.0), 255.0);
        assert_eq!(u16_fixo(-1.0, 256.0), 0.0);
    }
}
