//! **A ESPESSURA da tinta** ([`Grid::espessura`], Óleo Real W1b): quanto a célula tem de tinta EM PÉ, em
//! unidades de massa, sem o tecto de 3000 do depósito (`trail/transfer.rs`), que só governa a cor e a
//! massa. Guarda-se como FACTOR `espessura ÷ (sett+susp)` (≥ 1): quem tira massa (pickup, empurrão do
//! molhado, levantar, borracha) leva a espessura na mesma proporção sem a conhecer; só o depósito (onde o
//! tecto apara) e o arrasto (onde a tinta viaja) a escrevem. Preguiçoso como [`super::altura`]: vazio =
//! factor 1 em toda a parte, e só se aloca quando uma célula passa do tecto — quem não acumula não paga.

use super::Grid;

/// O factor da célula `i` (`1` sem o campo).
#[must_use]
pub fn factor(g: &Grid, i: usize) -> f64 {
    g.espessura.get(i).map_or(1.0, |&k| f64::from(k))
}

/// A espessura da célula `i`, em unidades de massa.
#[must_use]
pub fn em(g: &Grid, i: usize) -> f64 {
    (f64::from(g.sett[i]) + f64::from(g.susp[i])) * factor(g, i)
}

/// Fixa a espessura da célula `i` em `t` (unidades de massa) sobre a massa que ela tem AGORA. Sem massa a
/// célula não tem espessura (factor 1); o campo só se aloca quando `t` passa da massa.
pub fn fixa(g: &mut Grid, i: usize, t: f64) {
    let m = f64::from(g.sett[i]) + f64::from(g.susp[i]);
    let k = if m > 0.0 { (t / m).max(1.0) } else { 1.0 };
    if g.espessura.is_empty() {
        // A folga é a do arredondamento `f32` da massa: abaixo dela nada foi aparado.
        if k <= 1.0 + 1e-4 {
            return;
        }
        g.espessura = vec![1.0; g.cells];
    }
    g.espessura[i] = k as f32;
}

/// Devolve a janela de um recorte (`linhas`); vazio = a grade não tinha o campo ⇒ a janela volta a 1.
pub fn restaura_janela(g: &mut Grid, janela: (i32, i32, i32, i32), linhas: &[f32]) {
    let (n, s) = (g.cells, g.s);
    super::altura::restaura_campo(&mut g.espessura, n, s, janela, linhas, 1.0);
}

#[cfg(test)]
#[path = "espessura_tests.rs"]
mod tests;
