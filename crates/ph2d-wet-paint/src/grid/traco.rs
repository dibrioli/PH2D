//! O campo PREGUIÇOSO [`Grid::traco`] (J2, 03_MOTORES §7 d): o número do traço que pousou a tinta de cada célula —
//! vazio até o primeiro pincel com [`crate::tuning::Knob::LevelingOwnStroke`] pintar na grade (a 4096² seriam 33 MB
//! por camada para quem nunca o usa). `0` = nenhum traço (papel, ou tinta de um pincel sem a lei).

use super::Grid;

/// O campo, alocado a zeros se ainda não existia.
pub fn garante(g: &mut Grid) -> &mut [u16] {
    if g.traco.is_empty() {
        g.traco = vec![0; g.cells];
    }
    &mut g.traco
}

/// ⭐ **Duas células de traços DIFERENTES** (as duas pintadas): o fluxo não passa entre elas
/// ([`crate::solver::parede_do_traco`]). Sem o campo, nunca.
#[inline]
#[must_use]
pub fn separa(t: &[u16], a: usize, b: usize) -> bool {
    match (t.get(a), t.get(b)) {
        (Some(&x), Some(&y)) => x != 0 && y != 0 && x != y,
        _ => false,
    }
}

/// Devolve a janela `[x0..=x1] × [y0..=y1]` de um recorte (`linhas`, linha-a-linha): vazio = a grade não tinha
/// o campo quando o recorte foi tirado ⇒ a janela volta a zero (sem alocar, se o campo também não existe).
pub fn restaura_janela(g: &mut Grid, (x0, y0, x1, y1): (i32, i32, i32, i32), linhas: &[u16]) {
    if linhas.is_empty() && g.traco.is_empty() {
        return;
    }
    let (s, w) = (g.s, (x1 - x0 + 1) as usize);
    let campo = garante(g);
    for (r, y) in (y0..=y1).enumerate() {
        let o = y as usize * s + x0 as usize;
        match linhas.get(r * w..(r + 1) * w) {
            Some(l) => campo[o..o + w].copy_from_slice(l),
            None => campo[o..o + w].fill(0),
        }
    }
}
