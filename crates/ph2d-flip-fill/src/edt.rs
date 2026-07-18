//! **Transformada de distância euclidiana EXATA**, ao quadrado — o primitivo que o
//! trapped-ball de fato quer.
//!
//! Port de **Felzenszwalb & Huttenlocher**, *"Distance Transforms of Sampled
//! Functions"* (Theory of Computing 8(19), 2012): `O(N)`, separável, **exata** (não é
//! a aproximação por chanfro). O mesmo algoritmo que a linha do Painter mediu e
//! validou para o fechamento morfológico do Inflate (`sculpt_close.rs`) — aqui ele é
//! re-portado do paper, e não importado de lá, porque o Flip não depende do Painter.
//!
//! ## Por que EDT, e não passes de morfologia
//!
//! O `Grid::grow` do W4 é um offset por passes (alternando 4- e 8-conexo para
//! acumular um octógono em vez de um quadrado — BUGS #13). Ele responde à pergunta do
//! usuário ("encolha a região N px") e responde bem.
//!
//! O trapped-ball pergunta outra coisa: **"cabe aqui uma bola de raio r?"** — que é
//! *literalmente* uma pergunta de distância. Com a EDT ela é respondida de uma vez, e
//! de graça **para todo r ao mesmo tempo** (`d[p] >= r²`), o que torna a varredura de
//! raios decrescentes do paper um simples re-threshold do MESMO buffer, em vez de uma
//! nova rodada de erosões por raio. É exata (nada de octógono) e é `O(área)`.
//!
//! ## Aritmética
//!
//! Distâncias ao quadrado são **inteiras**, e é assim que ficam guardadas (`u32`): num
//! grid de 4096 o máximo é `2·4095² ≈ 33,5 M`, que **não** é exatamente representável
//! em `f32` (mantissa de 24 bits ⇒ inteiros exatos só até 16,7 M). Guardar em `f32`
//! daria um resultado *quase* certo — a classe de bug que este projeto já pagou várias
//! vezes. A interseção de parábolas do envelope inferior roda em `f64`, onde os
//! valores envolvidos são exatos.

/// Sentinela de "infinito" para a passada 1D. Tem de ser **maior que qualquer
/// distância real** do grid e pequena o bastante para não estourar a aritmética.
/// `4·(max_dim² + 1)` cumpre os dois: a maior distância real ao quadrado num grid
/// `w × h` é `(w−1)² + (h−1)² < 2·max_dim²`.
fn infinity_for(max_dim: usize) -> u32 {
    // Cabe folgado em `u32`: no maior grid (4096) a sentinela é ~67 M, e a maior soma
    // que a passada 1D produz é `dx² + f` ≈ 16,7 M + 67 M = 84 M — bem abaixo de 4,29 G.
    // Guardar em `u32` em vez de `u64` METADE o tráfego de memória de uma passada que é
    // O(N) e limitada por banda, e a exatidão é a mesma (tudo inteiro).
    4 * (max_dim as u32 * max_dim as u32 + 1)
}

/// **Distância euclidiana ao quadrado, de cada pixel ao conjunto `S`.**
///
/// `in_set(i)` diz se o pixel de índice `i = y·w + x` pertence a `S`. Pixels de `S`
/// recebem `0`. Se `S` for vazio, todo pixel recebe [`u32::MAX`] (nada de `0`, que
/// significaria "está no conjunto" — a resposta errada mais perigosa aqui).
///
/// O resultado é **exato**: `d[i]` é o mínimo de `(x−x')² + (y−y')²` sobre `S`.
#[must_use]
pub fn sq_distance_to_set(w: usize, h: usize, in_set: impl Fn(usize) -> bool) -> Vec<u32> {
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let inf = infinity_for(w.max(h));
    // Passada 1: por COLUNA. `f` vira a distância vertical ao quadrado.
    let mut f: Vec<u32> = (0..w * h)
        .map(|i| if in_set(i) { 0 } else { inf })
        .collect();

    let mut scratch = Envelope::new(w.max(h));
    // Colunas.
    let mut col: Vec<u32> = vec![0; h];
    for x in 0..w {
        for y in 0..h {
            col[y] = f[y * w + x];
        }
        scratch.transform(&mut col);
        for y in 0..h {
            f[y * w + x] = col[y];
        }
    }
    // Passada 2: por LINHA, sobre o resultado da primeira. É a separabilidade do
    // paper: `min_q ((p−q)² + f(q))` aplicada nos dois eixos dá a EDT 2D exata.
    let mut row: Vec<u32> = vec![0; w];
    let mut out: Vec<u32> = vec![0; w * h];
    for y in 0..h {
        row.copy_from_slice(&f[y * w..y * w + w]);
        scratch.transform(&mut row);
        for x in 0..w {
            // ⚠️ A sentinela **cabe em `u32`** (`4·(4096²+1) ≈ 67 M`), então um
            // `try_from` a deixaria passar como se fosse uma distância de verdade.
            // Quem nunca viu o conjunto tem de sair como `u32::MAX` — "infinitamente
            // longe" —, nunca como um número que parece medido.
            out[y * w + x] = if row[x] >= inf { u32::MAX } else { row[x] };
        }
    }
    out
}

/// O envelope inferior de parábolas da passada 1D (a máquina do paper), com os
/// buffers reusados entre linhas/colunas — alocar por linha num grid de milhões de
/// pixels é o custo dominante, e ele é evitável.
struct Envelope {
    /// Índice do vértice da `k`-ésima parábola do envelope.
    v: Vec<usize>,
    /// Fronteiras entre parábolas vizinhas (`z[k]`..`z[k+1]` é o domínio de `v[k]`).
    z: Vec<f64>,
    /// Cópia da entrada (o transform escreve na saída enquanto lê a entrada).
    src: Vec<u32>,
}

impl Envelope {
    fn new(n: usize) -> Self {
        Self {
            v: vec![0; n + 1],
            z: vec![0.0; n + 2],
            src: vec![0; n],
        }
    }

    /// `d[p] ← min_q ( (p−q)² + d[q] )`, in-place. É a transformada 1D exata.
    fn transform(&mut self, d: &mut [u32]) {
        let n = d.len();
        if n == 0 {
            return;
        }
        self.src[..n].copy_from_slice(d);
        let f = &self.src[..n];

        // Construção do envelope inferior.
        let mut k = 0usize;
        self.v[0] = 0;
        self.z[0] = f64::NEG_INFINITY;
        self.z[1] = f64::INFINITY;
        for q in 1..n {
            // Interseção da parábola de `q` com a de `v[k]`. Em `f64` porque `s` é
            // racional; os inteiros que entram nela são todos exatos neste tamanho.
            let mut s = intersect(f, q, self.v[k]);
            // Enquanto a nova parábola cobrir a última, a última sai do envelope.
            while k > 0 && s <= self.z[k] {
                k -= 1;
                s = intersect(f, q, self.v[k]);
            }
            if k == 0 && s <= self.z[0] {
                // A nova parábola cobre TODAS as anteriores: ela vira o envelope.
                self.v[0] = q;
                self.z[0] = f64::NEG_INFINITY;
                self.z[1] = f64::INFINITY;
            } else {
                k += 1;
                self.v[k] = q;
                self.z[k] = s;
                self.z[k + 1] = f64::INFINITY;
            }
        }

        // Leitura: para cada `p`, a parábola cujo domínio o contém.
        let mut k = 0usize;
        for (p, out) in d.iter_mut().enumerate().take(n) {
            let pf = p as f64;
            while self.z[k + 1] < pf {
                k += 1;
            }
            let vk = self.v[k];
            let dx = p.abs_diff(vk) as u32;
            *out = dx * dx + f[vk];
        }
    }
}

/// Abscissa onde as parábolas centradas em `q` e em `r` se cruzam.
///
/// `s = ((f[q] + q²) − (f[r] + r²)) / (2q − 2r)`. Chamada só com `q > r`, então o
/// denominador nunca é zero.
fn intersect(f: &[u32], q: usize, r: usize) -> f64 {
    let (fq, fr) = (f[q] as f64, f[r] as f64);
    let (qf, rf) = (q as f64, r as f64);
    ((fq + qf * qf) - (fr + rf * rf)) / (2.0 * qf - 2.0 * rf)
}

#[cfg(test)]
#[path = "edt_tests.rs"]
mod tests;
