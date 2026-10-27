//! **A ESPESSURA da tinta** ([`Grid::espessura`], Óleo Real W1b): quanto a célula tem de tinta EM PÉ, em
//! unidades de massa, sem o tecto de 3000 do depósito (`trail/transfer.rs`), que só governa a cor e a
//! massa. Guarda-se o COEFICIENTE `k` do EXCESSO sobre a massa `m`: `espessura = m + (k − 1) · max(m, MASSA_QUE_COBRE)`
//! ([`em`]). Com tinta que cobre (`m ≥` [`MASSA_QUE_COBRE`]) é o factor `m · k` de sempre: quem tira massa (pickup,
//! empurrão do molhado, levantar, borracha) leva a espessura na mesma proporção sem a conhecer; só o depósito (onde o
//! tecto apara), o arrasto (onde a tinta viaja) e a lâmina ([`nivela`]) a escrevem. Preguiçoso como
//! [`super::altura`]: vazio = `k = 1` em toda a parte, e só se aloca quando uma célula passa do tecto — quem não
//! acumula não paga.
//!
//! ⛔ **Abaixo da massa que cobre o excesso é ABSOLUTO** (BUG do smoke 27/10, `docs/Painter/Pinceis/W3_P_Knife.md`
//! §BUG): a lâmina que nivela leva o vale do grão (massa ~10) à altura da média (~2000); como FACTOR isso era `k ~
//! 10²`, e a massa que chegava DEPOIS (o assentar, a água) multiplicava-se por ele — a média do dab seguinte
//! apanhava-a e o factor ia a ∞ na 2.ª pincelada (medido: `kmax` 1,2·10² no 1.º traço, ∞ no 2.º; a foto em placas
//! negras). Como excesso sobre a massa que cobre, a tinta que chega SOMA-SE e o coeficiente fica `≤ média ÷ 500`.

use super::Grid;

/// A massa de uma camada que COBRE: a escala da opacidade do motor (`α(m) = 1 − 0,998^m`, `crate::opacity`) — a
/// massa de uma profundidade óptica, `1 ÷ −ln 0,998 = 499,5`. Abaixo dela a célula é papel com véu de tinta.
pub const MASSA_QUE_COBRE: f64 = 499.5;

fn massa(g: &Grid, i: usize) -> f64 {
    f64::from(g.sett[i]) + f64::from(g.susp[i])
}

/// O coeficiente guardado da célula `i` (`1` sem o campo).
fn coef(g: &Grid, i: usize) -> f64 {
    g.espessura.get(i).map_or(1.0, |&k| f64::from(k))
}

/// O coeficiente que dá a espessura `t` à massa `m > 0`.
fn coef_de(t: f64, m: f64) -> f64 {
    1.0 + (t - m) / m.max(MASSA_QUE_COBRE)
}

/// O factor EFECTIVO da célula `i`, `espessura ÷ massa` (`1` sem massa ou sem o campo; < 1 onde a faca raspou).
#[must_use]
pub fn factor(g: &Grid, i: usize) -> f64 {
    let m = massa(g, i);
    if m > 0.0 && !g.espessura.is_empty() {
        em(g, i) / m
    } else {
        1.0
    }
}

/// A espessura da célula `i`, em unidades de massa (`0` sem massa).
#[must_use]
pub fn em(g: &Grid, i: usize) -> f64 {
    let m = massa(g, i);
    if m <= 0.0 {
        return 0.0;
    }
    (m + (coef(g, i) - 1.0) * m.max(MASSA_QUE_COBRE)).max(0.0)
}

/// Fixa a espessura da célula `i` em `t` (unidades de massa) sobre a massa que ela tem AGORA. Sem massa a
/// célula não tem espessura (`k` 1); o campo só se aloca quando `t` passa da massa.
pub fn fixa(g: &mut Grid, i: usize, t: f64) {
    let m = massa(g, i);
    let k = if m > 0.0 { coef_de(t.max(m), m) } else { 1.0 };
    if g.espessura.is_empty() {
        // A folga é a do arredondamento `f32` da massa: abaixo dela nada foi aparado.
        if k <= 1.0 + 1e-4 {
            return;
        }
        g.espessura = vec![1.0; g.cells];
    }
    g.espessura[i] = k as f32;
}

/// Fixa a espessura da célula `i` em `t` SEM o piso da massa: a lâmina que NIVELA ([`crate::lamina`]) baixa a tinta
/// em pé abaixo do que a massa pediria (a cor e a massa não mudam, só a altura). Sem massa, nada; o campo aloca-se
/// quando o coeficiente sai de 1.
pub fn nivela(g: &mut Grid, i: usize, t: f64) {
    let m = massa(g, i);
    if m <= 0.0 {
        return;
    }
    let k = coef_de(t.max(0.0), m);
    if g.espessura.is_empty() {
        if k == 1.0 {
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
