//! O **laplaciano de cotangentes** e a **massa agrupada** da malha — as duas peças de que a energia
//! bilaplaciana `Q = L · M⁻¹ · L` é feita.
//!
//! ⚠️ **A fórmula da cotangente é a mesma em 2D e em 3D**, e não por acaso: ela sai do gradiente das
//! funções de base lineares sobre um triângulo, que não sabe em que dimensão o triângulo está
//! mergulhado. ⛔ O que muda é o que se faz com ela; a matriz é a mesma.

/// `L` em CSR, mais a massa agrupada e a diagonal de `Q` (para o pré-condicionador).
pub(crate) struct Laplacian {
    /// Início de cada linha em [`Self::col`]/[`Self::val`].
    pub(crate) fila: Vec<usize>,
    pub(crate) col: Vec<u32>,
    pub(crate) val: Vec<f64>,
    /// A massa agrupada por vértice (a área baricêntrica). ⚠️ Nunca zero — ver a guarda abaixo.
    pub(crate) massa: Vec<f64>,
    /// `diag(Q)_i = Σ_k L_ik² / m_k` — o pré-condicionador de Jacobi, sem montar `Q`.
    pub(crate) diag_q: Vec<f64>,
}

/// A cotangente do ângulo em `a`, no triângulo `(a, b, c)`.
///
/// ⚠️ **`cot = (e1·e2) / |e1 × e2|`**, e o denominador é o dobro da área. ⛔ Um triângulo degenerado
/// dá área zero e cotangente infinita: a porta recusa a malha em vez de a deixar envenenar a
/// matriz — *um `inf` numa matriz esparsa vira `NaN` três operações depois, longe da causa*.
fn cot(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Option<f64> {
    let e1 = [b[0] - a[0], b[1] - a[1]];
    let e2 = [c[0] - a[0], c[1] - a[1]];
    let cruz = e1[0] * e2[1] - e1[1] * e2[0];
    (cruz.abs() > f64::EPSILON).then(|| (e1[0] * e2[0] + e1[1] * e2[1]) / cruz.abs())
}

/// Monta `L` (rigidez, semi-definida positiva) e a massa. `None` numa malha degenerada.
pub(crate) fn build(mesh: &ph2d_poly2d::Mesh2d) -> Option<Laplacian> {
    let n = mesh.rest.len();
    // Acumulador denso por linha → CSR no fim. ⚠️ Um mapa por aresta seria `O(n log n)` e a malha
    // de uma arte tem `~7` vizinhos por vértice: a lista por linha é mais barata e mais simples.
    let mut linhas: Vec<Vec<(u32, f64)>> = vec![Vec::new(); n];
    let mut massa = vec![0.0_f64; n];
    for t in &mesh.tris {
        let (i, j, k) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let (Some(&pi), Some(&pj), Some(&pk)) =
            (mesh.rest.get(i), mesh.rest.get(j), mesh.rest.get(k))
        else {
            return None;
        };
        let area =
            ((pj[0] - pi[0]) * (pk[1] - pi[1]) - (pj[1] - pi[1]) * (pk[0] - pi[0])).abs() / 2.0;
        if area <= f64::EPSILON {
            return None;
        }
        for v in [i, j, k] {
            massa[v] += area / 3.0;
        }
        // ⭐ A aresta OPOSTA a cada canto leva metade da cotangente daquele canto — é essa a
        // fórmula, e é por isso que o laço percorre `(canto, aresta oposta)`.
        for (canto, (u, v)) in [(pi, (j, k)), (pj, (k, i)), (pk, (i, j))] {
            let w = cot(canto, mesh.rest[u], mesh.rest[v])? / 2.0;
            empurra(&mut linhas[u], u as u32, w);
            empurra(&mut linhas[v], v as u32, w);
            empurra(&mut linhas[u], v as u32, -w);
            empurra(&mut linhas[v], u as u32, -w);
        }
    }
    // ⚠️ Um vértice sem triângulo teria massa zero e `M⁻¹` infinita. A malha vem de um traçador que
    // não os produz, e a guarda fica porque o custo dela é uma comparação.
    if massa.iter().any(|&m| m <= f64::EPSILON) {
        return None;
    }

    let mut fila = Vec::with_capacity(n + 1);
    let (mut col, mut val) = (Vec::new(), Vec::new());
    fila.push(0);
    for linha in &mut linhas {
        linha.sort_unstable_by_key(|&(c, _)| c);
        let mut anterior: Option<u32> = None;
        for &(c, v) in linha.iter() {
            if anterior == Some(c) {
                *val.last_mut().expect("acabou de ser empurrado") += v;
            } else {
                col.push(c);
                val.push(v);
                anterior = Some(c);
            }
        }
        fila.push(col.len());
    }

    // `diag(Q)_i = Σ_k L_ik² / m_k`.
    let mut diag_q = vec![0.0_f64; n];
    for i in 0..n {
        let mut s = 0.0;
        for idx in fila[i]..fila[i + 1] {
            let k = col[idx] as usize;
            s += val[idx] * val[idx] / massa[k];
        }
        diag_q[i] = s;
    }
    Some(Laplacian {
        fila,
        col,
        val,
        massa,
        diag_q,
    })
}

fn empurra(linha: &mut Vec<(u32, f64)>, c: u32, v: f64) {
    linha.push((c, v));
}

impl Laplacian {
    /// `y = L · x`.
    pub(crate) fn mul(&self, x: &[f64], y: &mut [f64]) {
        for (i, saida) in y.iter_mut().enumerate() {
            let mut s = 0.0;
            for idx in self.fila[i]..self.fila[i + 1] {
                s += self.val[idx] * x[self.col[idx] as usize];
            }
            *saida = s;
        }
    }

    /// `y = Q · x = L · M⁻¹ · L · x`, com um rascunho emprestado.
    ///
    /// ⭐ **O `Q` nunca existe** — ver o cabeçalho do [`crate`].
    pub(crate) fn mul_q(&self, x: &[f64], rascunho: &mut [f64], y: &mut [f64]) {
        self.mul(x, rascunho);
        for (r, &m) in rascunho.iter_mut().zip(self.massa.iter()) {
            *r /= m;
        }
        self.mul(rascunho, y);
    }

    pub(crate) fn n(&self) -> usize {
        self.massa.len()
    }
}
