//! **O corte do dab por LINHA** (B26.1, o «RLE da máscara» do `libmypaint` ISC no nosso idioma): numa pegada plana,
//! sem cápsula, sem película com AA e sem Shape, a silhueta é `0` onde o `t` exacto passa de `1` (a queda dá
//! `peso(1)` a todo `t ≥ 1`). O `t` é convexo ao longo da linha ([`crate::FootprintDeform::falloff_t_abaixo_na_linha`]),
//! logo as colunas que pintam cabem num intervalo; fora dele (folga do `f32` no limiar e um píxel a mais de cada lado)
//! o laço saltava-as pelo `w <= 0.0` — o salto é ao bit. A orla interior não tem buracos (pegada convexa): o RLE
//! do `libmypaint` só tem as duas pontas para cortar aqui.

use super::{DabCtx, Invariantes};
use std::sync::OnceLock;

/// O `t` exacto a partir do qual o `f32` dá com certeza `t ≥ 1`, com a folga relativa e absoluta dos arredondamentos
/// (a mesma do corte do Wet, `corte_das_linhas.rs`).
const LIMIAR: f64 = 1.0 * (1.0 + 1e-3) + 1e-3;

/// `PH2D_CORTE_DO_DAB=0` desliga o corte (A×B no mesmo binário); lida uma vez.
pub(super) fn ligado() -> bool {
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var("PH2D_CORTE_DO_DAB").map_or(true, |v| v != "0"))
}

/// O dab cuja silhueta é nula fora do `t < 1` da pegada plana — `None` = o laço percorre `x0..x1` inteiro.
pub(super) fn do_dab(ctx: &DabCtx, inv: &Invariantes, pedido: bool) -> Option<Corte> {
    let elegivel = pedido
        && ctx.capsula.is_none()
        && ctx.film_aa.is_none()
        && ctx.shape.is_none()
        && ctx.footprint.falloff_t_lipschitz().is_some()
        && inv.queda.peso(ctx.spec, 1.0) == 0.0;
    elegivel.then(|| Corte {
        cx: f64::from(ctx.cx),
        inv_r: f64::from(ctx.inv_radius),
    })
}

#[derive(Clone, Copy)]
pub(super) struct Corte {
    cx: f64,
    inv_r: f64,
}

impl Corte {
    /// As colunas `[a, b)` da linha cujo `dy` (em píxeis, o mesmo `f32` do laço) pode pintar, dentro de `[x0, x1)`.
    #[inline]
    pub(super) fn linha(self, ctx: &DabCtx, dy: f32) -> (i64, i64) {
        let v = f64::from(dy * ctx.inv_radius);
        let Some([lo, hi]) = ctx.footprint.falloff_t_abaixo_na_linha(v, LIMIAR) else {
            return (ctx.x0, ctx.x0);
        };
        // `dx = px + 0,5 − cx` ⇒ `px = cx + u/inv_r − 0,5`; um píxel a mais de cada lado.
        let a = ((self.cx + lo / self.inv_r - 0.5).floor() as i64).saturating_sub(1);
        let b = ((self.cx + hi / self.inv_r - 0.5).ceil() as i64).saturating_add(2);
        let a = a.clamp(ctx.x0, ctx.x1);
        (a, b.clamp(a, ctx.x1))
    }
}

#[cfg(test)]
thread_local! {
    /// Texéis cuja silhueta o laço avaliou (só sob teste): o gate do corte conta-os.
    pub(super) static AVALIADOS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
