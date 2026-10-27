//! O Smear leva a **marca do nivelado** ([`crate::grid::nivelado`], K7/K8) como leva o material
//! ([`crate::grid::material::borra`]): a célula vai para a média PESADA PELA MASSA da amostra bilinear, na fracção da
//! cor — a lei do arrasto e do Blend do rasto ([`crate::trail`]). Sem o campo (a sessão nunca nivelou), nada se lê
//! nem se aloca: ao bit.

use crate::grid::Grid;

/// A marca da caixa `[x0..=x1] × [y0..=y1]`, linha a linha (a amostra do Smear); VAZIA sem o campo.
pub(super) fn recorte(g: &Grid, x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<f32> {
    if g.nivelado.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(((x1 - x0 + 1) * (y1 - y0 + 1)) as usize);
    for y in y0..=y1 {
        let o = y as usize * g.s;
        out.extend_from_slice(&g.nivelado[o + x0 as usize..=o + x1 as usize]);
    }
    out
}

/// Borra a marca da célula `i` para a média da `amostra` (texels `ls`, pesos de massa `ws`) na fracção `t`.
pub(super) fn borra(
    campo: &mut [f32],
    amostra: &[f32],
    i: usize,
    ls: [usize; 4],
    ws: [f64; 4],
    t: f64,
) {
    if amostra.is_empty() {
        return;
    }
    let (mut soma, mut peso) = (0.0f64, 0.0f64);
    for (l, w) in ls.into_iter().zip(ws) {
        if w > 0.0 {
            soma += f64::from(amostra[l]) * w;
            peso += w;
        }
    }
    if peso > 0.0 {
        let a = f64::from(campo[i]);
        campo[i] = (a + (soma / peso - a) * t) as f32;
    }
}

#[cfg(test)]
#[path = "marca_tests.rs"]
mod tests;
