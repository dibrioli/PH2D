//! A régua dos pontinhos escuros soltos (§BUG-bordas-W3b; partida de `oleo_real.rs`, 29/10).

/// §BUG-bordas-W3b — **a régua dos pontinhos escuros SOLTOS** (a de `oraculo/pontinhos_regua.py`, para os gates
/// das duas crates; `px` = `(luminância, cobertura)` por célula, `(w + 2)` por linha, borda de 1): célula a ≤ 3 do
/// contorno da tinta (cobertura ≥ ½), mais escura que o p5 e que METADE da mediana da tinta interior (a > 4 do
/// contorno) E ≥ 20 níveis abaixo da mediana das 8 vizinhas; conta as componentes 8-conexas de ≤ 4 células com uma
/// célula a ≤ 4 do PAPEL ABERTO (papel com < 20 % de tinta na janela 9×9) — a orla EXTERNA, a das setas do dono; os
/// salpicos do interior da textura contam à parte (o Rebelle tem 81 no Dabber inteiro, 1 na orla). Devolve
/// `(soltos na orla, soltos em toda a parte)` — a ≤ 3 de qualquer contorno, também o de um furo no meio da tinta.
pub fn pontinhos_escuros(px: &[[f64; 2]], (w, h): (usize, usize)) -> (usize, usize) {
    let s = w + 2;
    let at = |x: usize, y: usize| px[x + y * s];
    let tinta = |x: usize, y: usize| at(x, y)[1] >= 0.5;
    let viz = |x: usize, y: usize| {
        [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ]
        .map(|(dx, dy)| ((x as isize + dx) as usize, (y as isize + dy) as usize))
    };
    // A distância (Chebyshev) ao contorno, até 5.
    let mut dist = vec![99usize; s * (h + 2)];
    for y in 2..h {
        for x in 2..w {
            if tinta(x, y) && viz(x, y).iter().any(|&(a, b)| !tinta(a, b)) {
                dist[x + y * s] = 0;
            }
        }
    }
    for k in 1..=5 {
        for y in 2..h {
            for x in 2..w {
                if dist[x + y * s] == 99 && viz(x, y).iter().any(|&(a, b)| dist[a + b * s] == k - 1)
                {
                    dist[x + y * s] = k;
                }
            }
        }
    }
    let mut interior: Vec<f64> = (2..h)
        .flat_map(|y| (2..w).map(move |x| (x, y)))
        .filter(|&(x, y)| tinta(x, y) && dist[x + y * s] > 4)
        .map(|(x, y)| at(x, y)[0])
        .collect();
    interior.sort_by(f64::total_cmp);
    let p5 = interior[interior.len() / 20].min(0.5 * interior[interior.len() / 2]);
    let mut escuro = vec![false; s * (h + 2)];
    for y in 2..h {
        for x in 2..w {
            if dist[x + y * s] > 3 {
                continue;
            }
            let mut v = viz(x, y).map(|(a, b)| at(a, b)[0]);
            v.sort_by(f64::total_cmp);
            let mediana = 0.5 * (v[3] + v[4]);
            let l = at(x, y)[0];
            escuro[x + y * s] = l < p5 && l <= mediana - 20.0;
        }
    }
    // O papel aberto, e a faixa a ≤ 4 dele.
    let mut perto = vec![false; s * (h + 2)];
    for y in 1..=h {
        for x in 1..=w {
            if tinta(x, y) {
                continue;
            }
            let mut n = 0;
            for yy in y.saturating_sub(4).max(1)..=(y + 4).min(h) {
                for xx in x.saturating_sub(4).max(1)..=(x + 4).min(w) {
                    n += usize::from(tinta(xx, yy));
                }
            }
            if n * 5 < 81 {
                perto[x + y * s] = true;
            }
        }
    }
    for _ in 0..4 {
        let antes = perto.clone();
        for y in 2..h {
            for x in 2..w {
                if !antes[x + y * s] && viz(x, y).iter().any(|&(a, b)| antes[a + b * s]) {
                    perto[x + y * s] = true;
                }
            }
        }
    }
    let mut visto = vec![false; s * (h + 2)];
    let (mut soltos, mut todos) = (0, 0);
    for y in 2..h {
        for x in 2..w {
            if !escuro[x + y * s] || visto[x + y * s] {
                continue;
            }
            let (mut pilha, mut tam, mut na_orla) = (vec![(x, y)], 0, false);
            visto[x + y * s] = true;
            while let Some((a, b)) = pilha.pop() {
                tam += 1;
                na_orla |= perto[a + b * s];
                for (c, d) in viz(a, b) {
                    if escuro[c + d * s] && !visto[c + d * s] {
                        visto[c + d * s] = true;
                        pilha.push((c, d));
                    }
                }
            }
            soltos += usize::from(tam <= 4 && na_orla);
            todos += usize::from(tam <= 4);
        }
    }
    (soltos, todos)
}
