//! **A LEI DE UM PATCH** — `L_i = e_{i-1} + e_{i+1}`, resolvida para `e`.
//!
//! Este módulo é a **régua** da crate: ele decide, sozinho e sem o solver, se um
//! conjunto de comprimentos de lado admite ladrilhamento só com quadriláteros.
//! Ver a lei e a sua derivação no doc de [`crate`].
//!
//! # A estrutura que torna isto fácil
//!
//! A lei liga `e_j` a `e_{j+2}` (via `L_{j+1}`), nunca a `e_{j+1}`. Então os
//! índices se partem nos ciclos da permutação `j ↦ j+2`:
//!
//! - **`n` ímpar** — **um** ciclo com todos os índices;
//! - **`n` par** — **dois** ciclos, os pares e os ímpares, independentes.
//!
//! Dentro de um ciclo `(j_0, j_1, …)` a lei é a cadeia `e_{j_k} + e_{j_{k+1}} =
//! M_k`, que se resolve propagando um parâmetro `s` e fechando a volta:
//!
//! | comprimento do ciclo | ao fechar | consequência |
//! |---|---|---|
//! | **ímpar** | `s = A/2` | solução **única**; exige `A` par (a paridade) |
//! | **par** | `A = 0` | um **grau de liberdade**; exige a soma alternada zero |
//!
//! ⚠️ **É daí que sai, sem caso especial, o `L_0 = L_2` do patch de 4 lados**
//! (dois ciclos de comprimento 2 ⇒ soma alternada `L_0 − L_2 = 0`) e a paridade
//! do patch de 3 lados (um ciclo de comprimento 3).

/// O que impede um conjunto de comprimentos de fechar num patch.
///
/// ⚠️ **Cada variante nomeia um MECANISMO diferente**, e é isso que a torna útil
/// num gate: *"não fecha"* não distingue um lado curto demais de uma paridade
/// ímpar, e as duas curas são opostas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CornerError {
    /// Menos de 3 lados — ver [`crate::LayoutError::Valence`].
    Valence {
        /// Qual patch, quando conhecido.
        patch: Option<usize>,
        /// Quantos lados.
        sides: usize,
    },
    /// A soma alternada ao longo de um ciclo ímpar é **ímpar**: nenhum inteiro
    /// resolve. Curar = mudar **um** comprimento em `±1`.
    Parity {
        /// Qual patch, quando conhecido.
        patch: Option<usize>,
    },
    /// Um ciclo par não fecha: a soma alternada não é zero. Num patch de 4 lados
    /// isto é exatamente *"os lados opostos discordam"*.
    Inconsistent {
        /// Qual patch, quando conhecido.
        patch: Option<usize>,
        /// De quanto é o desacordo, com sinal.
        by: i64,
    },
    /// Existe solução racional, mas alguma aresta interior sairia `< 1` — o
    /// leque degenera. Curar = **aumentar** os lados, nunca diminuir.
    TooShort {
        /// Qual patch, quando conhecido.
        patch: Option<usize>,
        /// Qual aresta interior do leque.
        corner: usize,
    },
}

impl CornerError {
    /// Carimba o índice do patch num erro que ainda não o conhece.
    #[must_use]
    pub fn at_patch(self, p: usize) -> Self {
        match self {
            Self::Valence { sides, .. } => Self::Valence {
                patch: Some(p),
                sides,
            },
            Self::Parity { .. } => Self::Parity { patch: Some(p) },
            Self::Inconsistent { by, .. } => Self::Inconsistent { patch: Some(p), by },
            Self::TooShort { corner, .. } => Self::TooShort {
                patch: Some(p),
                corner,
            },
        }
    }
}

/// **RESOLVE UM PATCH** — dos comprimentos de lado para as arestas do leque.
///
/// Devolve `e_0..e_{n-1}`, todos `>= 1`, tais que `lens[i] = e[i-1] + e[i+1]`
/// para todo `i` (módulo `n`).
///
/// ⚠️ **Quando há grau de liberdade (ciclo par), a escolha é o MEIO da faixa
/// admissível.** Não é estética: pôr `e` no extremo deixa o leque com uma aresta
/// interior de comprimento 1 e outra enorme, que é onde o F5 produz o quad
/// esticado. E o meio é **determinístico** — não há empate a desempatar.
///
/// # Errors
/// Ver [`CornerError`]: valência, paridade, inconsistência ou lado curto demais.
pub fn solve_corners(lens: &[u32]) -> Result<Vec<u32>, CornerError> {
    let n = lens.len();
    if n < 3 {
        return Err(CornerError::Valence {
            patch: None,
            sides: n,
        });
    }
    let mut e = vec![0i64; n];
    // Os ciclos de `j ↦ j+2`: um só se `n` é ímpar, dois se é par.
    let starts: &[usize] = if n % 2 == 1 { &[0] } else { &[0, 1] };
    for &start in starts {
        let mut cycle = Vec::new();
        let mut j = start;
        loop {
            cycle.push(j);
            j = (j + 2) % n;
            if j == start {
                break;
            }
        }
        solve_cycle(lens, &cycle, n, &mut e)?;
    }
    for (j, v) in e.iter().enumerate() {
        if *v < 1 {
            return Err(CornerError::TooShort {
                patch: None,
                corner: j,
            });
        }
    }
    Ok(e.into_iter().map(|v| v as u32).collect())
}

/// Resolve UM ciclo da permutação `j ↦ j+2` e escreve os `e` dele.
fn solve_cycle(lens: &[u32], cycle: &[usize], n: usize, e: &mut [i64]) -> Result<(), CornerError> {
    let l = cycle.len();
    // `e[cycle[k]] = a[k] + (-1)^k · s`, com `a[0] = 0` e `a[k] = M_{k-1} - a[k-1]`,
    // onde `M_k = lens[(cycle[k] + 1) mod n]` liga `e_{cycle[k]}` a `e_{cycle[k+1]}`.
    let mut a = vec![0i64; l + 1];
    for k in 0..l {
        let m = i64::from(lens[(cycle[k] + 1) % n]);
        a[k + 1] = m - a[k];
    }
    let s = if l % 2 == 1 {
        // Fecha em `a[l] - s = s`.
        if a[l] % 2 != 0 {
            return Err(CornerError::Parity { patch: None });
        }
        a[l] / 2
    } else {
        // Fecha em `a[l] + s = s`: o parâmetro some, e sobra uma condição dura.
        if a[l] != 0 {
            return Err(CornerError::Inconsistent {
                patch: None,
                by: a[l],
            });
        }
        // ⚠️ `e_k = a[k] + (-1)^k·s >= 1` ⇒ uma faixa para `s`. O meio dela.
        let mut lo = i64::MIN;
        let mut hi = i64::MAX;
        for (k, ak) in a.iter().enumerate().take(l) {
            if k % 2 == 0 {
                lo = lo.max(1 - ak);
            } else {
                hi = hi.min(ak - 1);
            }
        }
        if lo > hi {
            // Alguma aresta interior é forçada abaixo de 1 — a faixa é vazia.
            // O `corner` culpado é o do limite que apertou.
            return Err(CornerError::TooShort {
                patch: None,
                corner: cycle[0],
            });
        }
        // Divisão que arredonda para baixo mesmo com negativos — determinismo.
        lo + (hi - lo) / 2
    };
    for (k, ak) in a.iter().enumerate().take(l) {
        e[cycle[k]] = ak + if k % 2 == 0 { s } else { -s };
    }
    Ok(())
}
