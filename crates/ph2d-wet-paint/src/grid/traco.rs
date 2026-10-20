//! O campo PREGUIÇOSO [`Grid::traco`] (J2, 03_MOTORES §7 d): o número do traço que pousou a tinta de cada célula —
//! vazio até o primeiro pincel com [`crate::tuning::Knob::LevelingOwnStroke`] pintar na grade (a 4096² seriam 67 MB
//! por camada para quem nunca o usa). `0` = nenhum traço (papel, ou tinta de um pincel sem a lei).

use super::Grid;

/// O campo, alocado a zeros se ainda não existia.
pub fn garante(g: &mut Grid) -> &mut [u32] {
    if g.traco.is_empty() {
        g.traco = vec![0; g.cells];
    }
    &mut g.traco
}

/// ⭐ **Duas células de traços DIFERENTES, as duas com tinta MOLHADA** (`susp > 0`), e pelo menos uma de um traço que
/// isola (o número só é posto pelo pincel com a lei — a lei é da TINTA; `0` = tinta sem a lei, ou papel): o fluxo não
/// passa entre elas ([`crate::solver::parede_do_traco`]) — a tinta com a lei não escoa para a molhada de outro traço,
/// nem a outra para dentro dela. A tinta seca não é parede. Sem o campo, nunca.
#[inline]
#[must_use]
pub fn separa(t: &[u32], susp: &[f32], a: usize, b: usize) -> bool {
    match (t.get(a), t.get(b)) {
        (Some(&x), Some(&y)) => x != y && susp[a] > 0.0 && susp[b] > 0.0,
        _ => false,
    }
}

/// Devolve a janela `[x0..=x1] × [y0..=y1]` de um recorte (`linhas`, linha-a-linha): vazio = a grade não tinha
/// o campo quando o recorte foi tirado ⇒ a janela volta a zero (sem alocar, se o campo também não existe).
pub fn restaura_janela(g: &mut Grid, (x0, y0, x1, y1): (i32, i32, i32, i32), linhas: &[u32]) {
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

#[cfg(test)]
mod tests {
    use super::separa;

    /// ⭐ A parede é entre tintas MOLHADAS de traços diferentes, uma pelo menos com a lei: a seca (`susp` 0) não é parede,
    /// o mesmo traço também não, duas tintas sem a lei (`0`) também não; a com a lei e a sem ela, sim.
    #[test]
    fn a_parede_e_so_entre_tintas_molhadas_de_tracos_diferentes() {
        let t = [1u32, 2, 2, 0, 0];
        let m = [1.0f32; 5];
        assert!(separa(&t, &m, 0, 1));
        assert!(!separa(&t, &[1.0, 0.0, 1.0, 1.0, 1.0], 0, 1), "a tinta seca virou parede");
        assert!(!separa(&t, &m, 1, 2));
        assert!(separa(&t, &m, 2, 3), "a tinta com a lei escoou para a sem ela");
        assert!(!separa(&t, &m, 3, 4));
    }

    /// ⭐ Secar a tela leva o campo do traço com a água (a tinta seca não é parede de ninguém).
    #[test]
    fn secar_a_tela_tira_as_paredes() {
        let mut g = crate::grid::Grid::new(16, 16);
        super::garante(&mut g)[20] = 7;
        crate::grid::dry_canvas(
            &mut g,
            crate::colorops::ColorMix::Plain,
            0.0,
            crate::assenta::Ecra::Porto,
        );
        assert!(g.traco.is_empty());
    }
}
