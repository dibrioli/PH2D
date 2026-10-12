//! ⭐ W5h — **o MIOLO dos opacos pela DIFUSÃO** (ordem do dono, 11/10: «espalhar suave» em todo corpo).
//!
//! Dentro da união dos corpos (W5k: todo oclusor de opacidade > 0) a luz é a solução de Dirichlet
//! `Δu − u/ℓ² = 0` com `u` = a luz de FORA na borda (as *diffusion curves*, Orzan et al. 2008; `ℓ = ∞`
//! é Laplace). A lei até a W5g — o exterior mais próximo, que desenha o cone com arestas radiais — é a
//! `suavidade = 0`; a resposta é a mistura linear das duas (e o translúcido mistura-a com a transmitida,
//! `crate::no_miolo`).
//!
//! O gabarito resolve-a numa grade própria, mais fina que a da placa, pelas diferenças finitas de
//! Gibou et al. (2002): a borda corta cada aresta da grade no ponto EXATO e entra pela fração `θ` da
//! aresta (2.ª ordem, matriz simétrica), e o sistema por gradientes conjugados. A régua dele é a fórmula
//! exata do disco (o núcleo de Poisson), no portão `o_miolo_do_disco_e_o_de_poisson`.

use crate::geometria::{self, Fronteira};
use crate::{Objeto, corpo};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpcoesDoMiolo {
    /// `0` = a luz do exterior mais próximo (a lei até a W5g) · `1` = a difusão inteira.
    pub suavidade: f64,
    /// O comprimento (m) em que a luz da borda cai a `1/e` para dentro; `f64::INFINITY` espalha até ao
    /// centro.
    pub penetracao_m: f64,
}

/// A escolha do dono (11/10): a difusão inteira, até ao centro.
pub const MIOLO_PADRAO: OpcoesDoMiolo = OpcoesDoMiolo {
    suavidade: 1.0,
    penetracao_m: f64::INFINITY,
};

/// Nós da grade ao longo do lado menor da caixa do MENOR corpo opaco. MEDIDO contra Poisson (o disco
/// de `o_miolo_do_disco`): 160 → erro 5,2e-3 em 0,12 s · 320 → 7,6e-4 em 0,95 s (`dev`, opt 2).
const NOS_NO_MENOR: f64 = 320.0;
/// O teto de nós da grade inteira (a memória e o tempo de um portão em `dev`).
const NOS_MAX: f64 = 4.0e6;
/// O resíduo relativo em que os gradientes conjugados param.
const TOLERANCIA: f64 = 1e-10;

/// A vizinha de um nó de dentro: outro nó de dentro, ou a borda a `θ` da aresta com a luz `g` lá.
#[derive(Clone, Copy)]
enum Viz {
    Dentro(u32),
    Borda { theta: f64, g: [f64; 3] },
}

/// O miolo resolvido de uma cena.
pub(crate) struct Miolo {
    origem: [f64; 2],
    h: f64,
    n: [usize; 2],
    /// O índice da incógnita de cada nó (`u32::MAX` = fora da união).
    indice: Vec<u32>,
    viz: Vec<[Viz; 4]>,
    u: Vec<[f64; 3]>,
}

const DIRS: [[isize; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
const FORA_DA_UNIAO: u32 = u32::MAX;

pub(crate) fn dentro_da_uniao(objetos: &[Objeto], p: [f64; 2], buf: &mut Vec<f64>) -> bool {
    objetos
        .iter()
        .filter(|o| corpo(o))
        .any(|o| geometria::dentro(&o.fronteira, p, buf))
}

/// Os cruzamentos das arestas opacas com a reta `eixo = c` (`eixo` 1: horizontal, `y = c`; 0:
/// vertical), na outra coordenada.
fn cortes_da_reta(polis: &[&[[f64; 2]]], eixo: usize, c: f64, out: &mut Vec<f64>) {
    out.clear();
    let outro = 1 - eixo;
    for v in polis {
        for (k, a) in v.iter().enumerate() {
            let b = v[(k + 1) % v.len()];
            if (a[eixo] > c) != (b[eixo] > c) {
                out.push(a[outro] + (c - a[eixo]) * (b[outro] - a[outro]) / (b[eixo] - a[eixo]));
            }
        }
    }
}

impl Miolo {
    /// Resolve o miolo da união dos corpos de `objetos`, com `exterior(q)` a luz num ponto `q` FORA
    /// dela. `None` sem nenhum corpo.
    pub(crate) fn resolver(
        objetos: &[Objeto],
        op: OpcoesDoMiolo,
        exterior: &(dyn Fn([f64; 2]) -> [f64; 3] + Sync),
    ) -> Option<Self> {
        let polis: Vec<&[[f64; 2]]> = objetos
            .iter()
            .filter(|o| corpo(o))
            .filter_map(|o| match &o.fronteira {
                Fronteira::Poligono(v) if v.len() >= 3 => Some(v.as_slice()),
                _ => None,
            })
            .collect();
        if polis.is_empty() {
            return None;
        }
        let caixa = |v: &[[f64; 2]]| {
            let mut lo = [f64::INFINITY; 2];
            let mut hi = [f64::NEG_INFINITY; 2];
            for p in v {
                for k in 0..2 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
            (lo, hi)
        };
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        let mut menor = f64::INFINITY;
        for v in &polis {
            let (a, b) = caixa(v);
            menor = menor.min((b[0] - a[0]).min(b[1] - a[1]));
            for k in 0..2 {
                lo[k] = lo[k].min(a[k]);
                hi[k] = hi[k].max(b[k]);
            }
        }
        let area = (hi[0] - lo[0]) * (hi[1] - lo[1]);
        let h = (menor / NOS_NO_MENOR).max((area / NOS_MAX).sqrt());
        if !(h.is_finite() && h > 0.0) {
            return None;
        }
        // Dois nós de folga de cada lado: as vizinhas de todo nó de dentro existem.
        let origem = [lo[0] - 2.0 * h, lo[1] - 2.0 * h];
        let n = [
            ((hi[0] - lo[0]) / h).ceil() as usize + 5,
            ((hi[1] - lo[1]) / h).ceil() as usize + 5,
        ];
        let no = |i: usize, j: usize| [origem[0] + i as f64 * h, origem[1] + j as f64 * h];

        // Os nós de dentro, linha a linha (a paridade por polígono, a união entre eles).
        let mut indice = vec![FORA_DA_UNIAO; n[0] * n[1]];
        let mut nos = Vec::new();
        let mut cortes = Vec::new();
        let mut por_poli = Vec::new();
        for j in 0..n[1] {
            let y = no(0, j)[1];
            for v in &polis {
                cortes_da_reta(&[v], 1, y, &mut por_poli);
                por_poli.sort_by(f64::total_cmp);
                for par in por_poli.as_chunks::<2>().0 {
                    let i0 = ((par[0] - origem[0]) / h).ceil().max(0.0) as usize;
                    let i1 = (((par[1] - origem[0]) / h).floor() as usize).min(n[0] - 1);
                    for i in i0..=i1 {
                        let x = no(i, j)[0];
                        if x > par[0] && x < par[1] {
                            indice[j * n[0] + i] = 0;
                        }
                    }
                }
            }
        }
        for (k, ix) in indice.iter_mut().enumerate() {
            if *ix != FORA_DA_UNIAO {
                *ix = nos.len() as u32;
                nos.push((k % n[0], k / n[0]));
            }
        }
        if nos.is_empty() {
            return None;
        }

        // A borda em cada aresta que sai da união: o cruzamento mais perto do nó de dentro depois do
        // qual o ponto já está fora (a união pode ter várias fronteiras na mesma aresta).
        struct Corte {
            k: usize,
            d: usize,
            theta: f64,
            q: [f64; 2],
        }
        let mut pedidos = Vec::new();
        let mut buf = Vec::new();
        for (k, &(i, j)) in nos.iter().enumerate() {
            let p = no(i, j);
            for (d, dir) in DIRS.iter().enumerate() {
                let (vi, vj) = (i as isize + dir[0], j as isize + dir[1]);
                if indice[vj as usize * n[0] + vi as usize] != FORA_DA_UNIAO {
                    continue;
                }
                let eixo = if dir[1] == 0 { 1 } else { 0 };
                let ao_longo = 1 - eixo;
                cortes_da_reta(&polis, eixo, p[eixo], &mut cortes);
                let s = if dir[0] + dir[1] > 0 { 1.0 } else { -1.0 };
                let mut ts: Vec<f64> = cortes
                    .iter()
                    .map(|c| s * (c - p[ao_longo]))
                    .filter(|t| *t >= 0.0 && *t <= h)
                    .collect();
                ts.sort_by(f64::total_cmp);
                let u = [dir[0] as f64, dir[1] as f64];
                let mut theta = 1.0;
                for t in ts {
                    let alem = t + 1e-9 * h;
                    if !dentro_da_uniao(objetos, [p[0] + u[0] * alem, p[1] + u[1] * alem], &mut buf)
                    {
                        theta = t / h;
                        break;
                    }
                }
                let t = theta * h + 1e-9;
                pedidos.push(Corte {
                    k,
                    d,
                    theta: theta.max(1e-6),
                    q: [p[0] + u[0] * t, p[1] + u[1] * t],
                });
            }
        }
        let gs = crate::em_paralelo(&pedidos, |c| exterior(c.q));

        let mut viz = vec![[Viz::Dentro(0); 4]; nos.len()];
        for (k, &(i, j)) in nos.iter().enumerate() {
            for (d, dir) in DIRS.iter().enumerate() {
                let (vi, vj) = (
                    (i as isize + dir[0]) as usize,
                    (j as isize + dir[1]) as usize,
                );
                viz[k][d] = Viz::Dentro(indice[vj * n[0] + vi]);
            }
        }
        for (c, g) in pedidos.iter().zip(gs) {
            viz[c.k][c.d] = Viz::Borda { theta: c.theta, g };
        }

        let kh2 = (h / op.penetracao_m).powi(2);
        let u = gradientes_conjugados(&viz, if kh2.is_finite() { kh2 } else { 0.0 });
        Some(Self {
            origem,
            h,
            n,
            indice,
            viz,
            u,
        })
    }

    /// O miolo em `p` (dentro da união): a bilinear dos nós da célula, com a luz da borda no lugar de
    /// um canto de fora. `None` se a célula não tem nenhum nó de dentro.
    pub(crate) fn em(&self, p: [f64; 2]) -> Option<[f64; 3]> {
        let fx = (p[0] - self.origem[0]) / self.h;
        let fy = (p[1] - self.origem[1]) / self.h;
        if fx < 0.0 || fy < 0.0 {
            return None;
        }
        let (i, j) = (fx.floor() as usize, fy.floor() as usize);
        if i + 1 >= self.n[0] || j + 1 >= self.n[1] {
            return None;
        }
        let (tx, ty) = (fx - i as f64, fy - j as f64);
        let cantos = [(i, j), (i + 1, j), (i, j + 1), (i + 1, j + 1)];
        let k = |c: (usize, usize)| self.indice[c.1 * self.n[0] + c.0];
        if cantos.iter().all(|&c| k(c) == FORA_DA_UNIAO) {
            return None;
        }
        // A luz da borda nas arestas da célula que saem de um canto de dentro, por canto de fora.
        let mut valores = [[0.0; 3]; 4];
        let mut todas = ([0.0; 3], 0.0);
        let mut na_borda = [([0.0; 3], 0.0); 4];
        // (canto, vizinho na célula, direção de canto → vizinho)
        let arestas: [(usize, usize, usize); 8] = [
            (0, 1, 0),
            (1, 0, 1),
            (2, 3, 0),
            (3, 2, 1),
            (0, 2, 2),
            (2, 0, 3),
            (1, 3, 2),
            (3, 1, 3),
        ];
        for (a, b, d) in arestas {
            let ka = k(cantos[a]);
            if ka == FORA_DA_UNIAO {
                continue;
            }
            if let Viz::Borda { g, .. } = self.viz[ka as usize][d] {
                let s = &mut na_borda[b];
                for (c, gc) in g.iter().enumerate() {
                    s.0[c] += gc;
                    todas.0[c] += gc;
                }
                s.1 += 1.0;
                todas.1 += 1.0;
            }
        }
        for (c, v) in valores.iter_mut().enumerate() {
            let kc = k(cantos[c]);
            *v = if kc != FORA_DA_UNIAO {
                self.u[kc as usize]
            } else if na_borda[c].1 > 0.0 {
                na_borda[c].0.map(|x| x / na_borda[c].1)
            } else if todas.1 > 0.0 {
                todas.0.map(|x| x / todas.1)
            } else {
                return None;
            };
        }
        Some([0, 1, 2].map(|c| {
            let a = valores[0][c] + (valores[1][c] - valores[0][c]) * tx;
            let b = valores[2][c] + (valores[3][c] - valores[2][c]) * tx;
            a + (b - a) * ty
        }))
    }
}

/// `A u = b` com `A` a matriz de Gibou (simétrica, definida positiva) e o pré-condicionador de Jacobi,
/// por canal.
fn gradientes_conjugados(viz: &[[Viz; 4]], kh2: f64) -> Vec<[f64; 3]> {
    let n = viz.len();
    let diag: Vec<f64> = viz
        .iter()
        .map(|v| {
            kh2 + v
                .iter()
                .map(|x| match x {
                    Viz::Dentro(_) => 1.0,
                    Viz::Borda { theta, .. } => 1.0 / theta,
                })
                .sum::<f64>()
        })
        .collect();
    let aplicar = |x: &[f64], y: &mut [f64]| {
        for (k, v) in viz.iter().enumerate() {
            let mut s = diag[k] * x[k];
            for w in v {
                if let Viz::Dentro(m) = w {
                    s -= x[*m as usize];
                }
            }
            y[k] = s;
        }
    };
    let mut u = vec![[0.0; 3]; n];
    for c in 0..3 {
        let b: Vec<f64> = viz
            .iter()
            .map(|v| {
                v.iter()
                    .map(|x| match x {
                        Viz::Dentro(_) => 0.0,
                        Viz::Borda { theta, g } => g[c] / theta,
                    })
                    .sum()
            })
            .collect();
        let norma_b = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norma_b == 0.0 {
            continue;
        }
        // Começa no pré-condicionado (`b / diag`): a borda já está lá.
        let mut x: Vec<f64> = b.iter().zip(&diag).map(|(b, d)| b / d).collect();
        let mut ax = vec![0.0; n];
        aplicar(&x, &mut ax);
        let mut r: Vec<f64> = b.iter().zip(&ax).map(|(b, a)| b - a).collect();
        let mut z: Vec<f64> = r.iter().zip(&diag).map(|(r, d)| r / d).collect();
        let mut p = z.clone();
        let mut rz: f64 = r.iter().zip(&z).map(|(a, b)| a * b).sum();
        let mut ap = vec![0.0; n];
        for _ in 0..(20 * n).max(100) {
            let nr = r.iter().map(|x| x * x).sum::<f64>().sqrt();
            if nr <= TOLERANCIA * norma_b {
                break;
            }
            aplicar(&p, &mut ap);
            let alfa = rz / p.iter().zip(&ap).map(|(a, b)| a * b).sum::<f64>();
            for k in 0..n {
                x[k] += alfa * p[k];
                r[k] -= alfa * ap[k];
                z[k] = r[k] / diag[k];
            }
            let rz_novo: f64 = r.iter().zip(&z).map(|(a, b)| a * b).sum();
            let beta = rz_novo / rz;
            rz = rz_novo;
            for k in 0..n {
                p[k] = z[k] + beta * p[k];
            }
        }
        for k in 0..n {
            u[k][c] = x[k];
        }
    }
    u
}
