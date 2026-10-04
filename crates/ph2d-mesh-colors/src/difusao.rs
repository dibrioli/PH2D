//! ⭐⭐⭐⭐ **O DESFOQUE NA SUPERFÍCIE** — o calor sobre a retícula
//! (`docs/3D/30` §2 e §14, a W6).
//!
//! Um Gaussiano de desvio `σ` É o calor ao fim de `t = σ²/2` (`∂u/∂t = −A u`).
//! Sobre a retícula, `A = M⁻¹L`: `L` é o laplaciano de COTANGENTES da
//! triangulação da retícula (um sub-triângulo de um triângulo é a face em ponto
//! pequeno; uma célula de quad parte-se em dois pela diagonal MAIS CURTA) e `M`
//! a área que cada amostra representa (um terço de cada triângulo). O desfoque
//! é `e^{−tA} u` — os elementos finitos lineares de sempre, sobre uma
//! triangulação CONFORME da superfície inteira.
//!
//! ⛔ A 1.ª redacção dava às células de quad um estêncil só nos eixos
//! (`A/|e|²` a cada lado): inconsistente numa célula trapezoidal, e a esfera UV
//! é feita delas — uma amostra de ARESTA da malha desviava `28×` mais que duas
//! do meio de faces à mesma distância de uma risca (média `0,195` contra
//! `0,007` degraus, pior `3`; gate `uma_aresta_da_malha_le_o_mesmo_que_o_meio_de_uma_face`).
//! A diagonal da célula entra no grafo (dentro da face: nunca há dono a decidir).
//!
//! **O raio é das UNIDADES DA PEÇA** porque pesos e massas vêm das posições; e
//! um par é um par NA SUPERFÍCIE — inclusive através de uma aresta da malha
//! ([`crate::vizinhanca`]) — logo não há costura nem atlas. ⭐ Numa retícula
//! regular o calor acrescenta `2t` de variância por eixo, qualquer que seja a
//! resolução: `σ²` EXACTO no mundo, o valor que o gate mede a `8x` e a `32x`.
//!
//! # ⭐⭐⭐ Porque um polinómio de Chebyshev e não passos explícitos
//!
//! O passo explícito estável é `dt ≤ ½ · min m/Σw ~ h²/8`: `t/dt = 4σ²/dt` dá
//! `8·(σ/h)²` passos — **2 048** a `σ = 16` amostras, o que um raio médio pede
//! a `32x`. O mesmo `e^{−tA}` aproximado por Chebyshev em `[0, λ_sup]` pede um
//! grau `~7·σ/h` (`~120`): o custo cresce com o raio, não com o quadrado dele.
//! O polinómio é o de `g(λ) = (1 − e^{−tλ})/λ` e o desfoque escreve-se
//! `u − g(A)·(A u)` ⇒ **um campo constante fica constante AO BIT** (`A u = 0`
//! exacto). O erro do polinómio é `< 1e-6` na cor (um 4 000-avos de degrau).
//!
//! ⚠️ Uma cotangente negativa (face obtusa) faria `A` deixar de preservar a
//! positividade: o peso TOTAL de um par (as metades das duas faces) corta-se em
//! `0` — numa face obtusa o desfoque perde um pouco de isotropia e não inventa
//! valores.

use crate::amostragem::{posicao_quad, posicao_tri};
use crate::{Tinta, cantos, sitio_quad, sitio_tri};

/// O erro máximo do polinómio sobre `e^{−tλ}` em `[0, λ_sup]` (na cor `0..1`).
const ERRO_DO_POLINOMIO: f64 = 1e-6;

/// O que o calor sabe somar: uma cor `[f32; 4]` (pré-multiplicada, quem chama
/// decide) ou um escalar.
pub trait Canal: Copy + Send + Sync {
    /// O zero.
    const ZERO: Self;
    /// `b − a`.
    fn menos(b: Self, a: Self) -> Self;
    /// `self += k · v`.
    fn soma(&mut self, k: f32, v: Self);
}

impl Canal for f32 {
    const ZERO: Self = 0.0;
    #[inline]
    fn menos(b: Self, a: Self) -> Self {
        b - a
    }
    #[inline]
    fn soma(&mut self, k: f32, v: Self) {
        *self += k * v;
    }
}

impl Canal for [f32; 4] {
    const ZERO: Self = [0.0; 4];
    #[inline]
    fn menos(b: Self, a: Self) -> Self {
        [b[0] - a[0], b[1] - a[1], b[2] - a[2], b[3] - a[3]]
    }
    #[inline]
    fn soma(&mut self, k: f32, v: Self) {
        for (s, x) in self.iter_mut().zip(v) {
            *s += k * x;
        }
    }
}

/// ⭐⭐⭐ **O laplaciano da retícula** — o grafo de [`crate::vizinhanca`] com o
/// peso de cada par e a massa de cada amostra.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Difusao {
    /// Os vizinhos de `a`, em CSR simétrico: `viz[ini[a]..ini[a + 1]]`.
    ini: Vec<u32>,
    viz: Vec<u32>,
    /// O peso `w_ab ≥ 0` de cada entrada (igual nas duas direcções).
    peso: Vec<f32>,
    /// `1 / m_a` (`0` numa amostra sem área: ela não muda).
    inv_massa: Vec<f32>,
    /// O tecto do espectro de `A` (Gershgorin: `max_a 2·Σ_b w_ab / m_a`).
    lambda_sup: f32,
    /// Quantas threads um passo usa (`0` = as da máquina).
    threads: usize,
}

/// ⭐ **A célula `abcd` de um quad parte-se pela diagonal `ac`?** — a MAIS
/// CURTA (a que evita os ângulos obtusos; empate → `ac`). A mesma resposta para
/// o grafo e para os pesos.
fn diagonal_ac(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> bool {
    let (ac, bd) = (sub(c, a), sub(d, b));
    dot(ac, ac) <= dot(bd, bd)
}

/// As diagonais das células de um quad — os pares que o grafo da
/// [`crate::vizinhanca`] não tem (ele dá os dos eixos) e a triangulação pede.
fn diagonais(tinta: &Tinta, f: usize, c: &[u32], pos: &[[f32; 3]], mut par: impl FnMut(u32, u32)) {
    let l = tinta.lado_da_face(f);
    let q = [0, 1, 2, 3].map(|k| pos[c[k] as usize]);
    let at = |i: u32, j: u32| posicao_quad(q, l, (i, j));
    let idx = |i: u32, j: u32| tinta.indice_de(f, c, sitio_quad(l, i, j));
    for j in 0..l {
        for i in 0..l {
            if diagonal_ac(at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)) {
                par(idx(i, j), idx(i + 1, j + 1));
            } else {
                par(idx(i + 1, j), idx(i, j + 1));
            }
        }
    }
}

/// A cotangente do ângulo em `r` do triângulo `pqr` e o dobro da área dele.
fn cot(p: [f32; 3], q: [f32; 3], r: [f32; 3]) -> (f32, f32) {
    let (u, v) = (sub(p, r), sub(q, r));
    let x = cross(u, v);
    let dupla = dot(x, x).sqrt();
    (dot(u, v) / dupla, dupla)
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// ⭐⭐ **Os coeficientes `d_k` de `g(λ) = (1 − e^{−tλ})/λ` na base de
/// Chebyshev de `[0, λ_sup]`** — interpolação nos nós de Chebyshev em `f64`,
/// cortada quando a cauda pesa menos que [`ERRO_DO_POLINOMIO`] em `e^{−tλ}`.
/// Vazio para `t ≤ 0`.
#[must_use]
pub fn coeficientes(t: f64, lambda_sup: f64) -> Vec<f32> {
    if !(t > 0.0 && lambda_sup > 0.0 && lambda_sup.is_finite()) {
        return Vec::new();
    }
    let alfa = 0.5 * t * lambda_sup;
    let k_est = (5.3 * alfa.sqrt()).ceil() as usize + 8;
    let n = 2 * k_est + 32;
    let g = |lam: f64| -(-t * lam).exp_m1() / lam;
    let nos: Vec<f64> = (0..n)
        .map(|j| {
            let x = (std::f64::consts::PI * (j as f64 + 0.5) / n as f64).cos();
            g(0.5 * lambda_sup * (x + 1.0))
        })
        .collect();
    let mut d: Vec<f64> = (0..n)
        .map(|k| {
            let s: f64 = nos
                .iter()
                .enumerate()
                .map(|(j, gj)| {
                    gj * (std::f64::consts::PI * k as f64 * (j as f64 + 0.5) / n as f64).cos()
                })
                .sum();
            2.0 * s / n as f64
        })
        .collect();
    d[0] *= 0.5;
    // O erro em `e^{−tλ} = 1 − λ·g(λ)` é `≤ λ_sup · Σ|cauda|`.
    let mut k = n;
    let mut cauda = 0.0;
    while k > 1 && (cauda + d[k - 1].abs()) * lambda_sup < ERRO_DO_POLINOMIO {
        cauda += d[k - 1].abs();
        k -= 1;
    }
    d.truncate(k);
    d.into_iter().map(|v| v as f32).collect()
}

impl Difusao {
    /// ⭐ **O laplaciano da retícula de `tinta`** — `cantos_de(f)` dá os cantos
    /// da face `f` (com ou sem o sentinela [`crate::TRI`]) e `pos` as posições
    /// dos vértices, nas unidades da peça.
    #[must_use]
    pub fn nova<'a>(
        tinta: &Tinta,
        cantos_de: impl Fn(usize) -> &'a [u32],
        pos: &[[f32; 3]],
    ) -> Self {
        Self::com_threads(tinta, cantos_de, pos, 0)
    }

    /// A [`Self::nova`] com o número de threads de cada passo fixo — `1` corre
    /// tudo na thread que chama (a régua de que o paralelo dá o mesmo ao bit).
    #[must_use]
    pub fn com_threads<'a>(
        tinta: &Tinta,
        cantos_de: impl Fn(usize) -> &'a [u32],
        pos: &[[f32; 3]],
        threads: usize,
    ) -> Self {
        let n = tinta.amostras().len();
        let faces = tinta.topologia().faces();
        let face = |f: usize| {
            let c = cantos_de(f);
            &c[..cantos(c)]
        };
        // 1. O grafo: cada par uma vez (a lei do `vizinhanca`), guardado nas
        //    duas direcções.
        let mut ini = vec![0u32; n + 1];
        for f in 0..faces {
            let c = face(f);
            let mut conta = |a: u32, b: u32| {
                ini[a as usize + 1] += 1;
                ini[b as usize + 1] += 1;
            };
            if c.len() == 3 {
                tinta.para_cada_par_tri(f, c, &mut conta);
            } else {
                tinta.para_cada_par_quad(f, c, &mut conta);
                diagonais(tinta, f, c, pos, &mut conta);
            }
        }
        for a in 0..n {
            ini[a + 1] += ini[a];
        }
        let mut cursor = ini.clone();
        let mut viz = vec![0u32; ini[n] as usize];
        for f in 0..faces {
            let c = face(f);
            let mut poe = |a: u32, b: u32| {
                for (x, y) in [(a, b), (b, a)] {
                    viz[cursor[x as usize] as usize] = y;
                    cursor[x as usize] += 1;
                }
            };
            if c.len() == 3 {
                tinta.para_cada_par_tri(f, c, &mut poe);
            } else {
                tinta.para_cada_par_quad(f, c, &mut poe);
                diagonais(tinta, f, c, pos, &mut poe);
            }
        }
        // 2. Os pesos e as massas, sub-elemento a sub-elemento — um par de
        //    aresta recebe a metade de CADA face (a dona emitiu-o, as duas
        //    somam-lhe).
        let mut peso = vec![0.0f32; viz.len()];
        let mut massa = vec![0.0f32; n];
        let mut soma = |a: u32, b: u32, w: f32| {
            for (x, y) in [(a, b), (b, a)] {
                let (i, j) = (ini[x as usize] as usize, ini[x as usize + 1] as usize);
                if let Some(e) = viz[i..j].iter().position(|&v| v == y) {
                    peso[i + e] += w;
                }
            }
        };
        // Um triângulo da retícula `pqr` (índices e posições): `½·cot` a cada
        // lado oposto, um terço da área a cada canto.
        let mut triangulo =
            |(ip, xp): (u32, [f32; 3]), (iq, xq): (u32, [f32; 3]), (ir, xr): (u32, [f32; 3])| {
                let (cot_r, dupla) = cot(xp, xq, xr);
                if !(dupla > 0.0 && dupla.is_finite()) {
                    return;
                }
                let (cot_p, _) = cot(xq, xr, xp);
                let (cot_q, _) = cot(xr, xp, xq);
                soma(ip, iq, 0.5 * cot_r);
                soma(iq, ir, 0.5 * cot_p);
                soma(ir, ip, 0.5 * cot_q);
                for i in [ip, iq, ir] {
                    massa[i as usize] += dupla / 6.0;
                }
            };
        for f in 0..faces {
            let c = face(f);
            let l = tinta.lado_da_face(f);
            if c.len() == 3 {
                let (pa, pb, pc) = (pos[c[0] as usize], pos[c[1] as usize], pos[c[2] as usize]);
                let no = |s: (u32, u32, u32)| {
                    (
                        tinta.indice_de(f, c, sitio_tri(l, s.0, s.1, s.2)),
                        posicao_tri(pa, pb, pc, l, s),
                    )
                };
                // Os sub-triângulos direitos (`i + j + k = L − 1`) e os
                // invertidos (`= L − 2`) — ver `amostragem::leitura_tri`.
                for i in 0..l {
                    for j in 0..(l - i) {
                        let k = l - 1 - i - j;
                        triangulo(no((i + 1, j, k)), no((i, j + 1, k)), no((i, j, k + 1)));
                    }
                }
                for i in 0..l.saturating_sub(1) {
                    for j in 0..(l - 1 - i) {
                        let k = l - 2 - i - j;
                        triangulo(
                            no((i, j + 1, k + 1)),
                            no((i + 1, j, k + 1)),
                            no((i + 1, j + 1, k)),
                        );
                    }
                }
            } else {
                let q = [0, 1, 2, 3].map(|k| pos[c[k] as usize]);
                let no = |i: u32, j: u32| {
                    (
                        tinta.indice_de(f, c, sitio_quad(l, i, j)),
                        posicao_quad(q, l, (i, j)),
                    )
                };
                for j in 0..l {
                    for i in 0..l {
                        let (a, b, cc, d) =
                            (no(i, j), no(i + 1, j), no(i + 1, j + 1), no(i, j + 1));
                        if diagonal_ac(a.1, b.1, cc.1, d.1) {
                            triangulo(a, b, cc);
                            triangulo(a, cc, d);
                        } else {
                            triangulo(a, b, d);
                            triangulo(b, cc, d);
                        }
                    }
                }
            }
        }
        for w in &mut peso {
            *w = w.max(0.0);
        }
        // 3. O tecto do espectro e as massas inversas.
        let inv_massa: Vec<f32> = massa
            .iter()
            .map(|&m| if m > 0.0 { 1.0 / m } else { 0.0 })
            .collect();
        let mut me = Self {
            ini,
            viz,
            peso,
            inv_massa,
            lambda_sup: 0.0,
            threads,
        };
        me.lambda_sup = (0..n)
            .map(|a| me.lambda_da_amostra(a))
            .fold(0.0f32, f32::max);
        me
    }

    /// O disco de Gershgorin da amostra `a`: `2·Σ_b w_ab / m_a` — o espectro
    /// de `A` cabe em `[0, max_a]` ([`Self::lambda_sup`]).
    #[must_use]
    pub fn lambda_da_amostra(&self, a: usize) -> f32 {
        let rigidez: f32 = self.peso[self.ini[a] as usize..self.ini[a + 1] as usize]
            .iter()
            .sum();
        2.0 * rigidez * self.inv_massa[a]
    }

    /// `N`, as amostras do plano a que este laplaciano serve.
    #[must_use]
    pub fn amostras(&self) -> usize {
        self.inv_massa.len()
    }

    /// O tecto do espectro de `A` (unidades da peça⁻²).
    #[must_use]
    pub fn lambda_sup(&self) -> f32 {
        self.lambda_sup
    }

    /// O grafo para a placa: `(ini, viz, peso, inv_massa)`.
    #[must_use]
    pub fn csr(&self) -> (&[u32], &[u32], &[f32], &[f32]) {
        (&self.ini, &self.viz, &self.peso, &self.inv_massa)
    }

    /// Quantos bytes o laplaciano segura.
    #[must_use]
    pub fn footprint_bytes(&self) -> usize {
        (self.ini.capacity()
            + self.viz.capacity()
            + self.peso.capacity()
            + self.inv_massa.capacity())
            * 4
    }

    /// ⭐ **O polinómio do Gaussiano de desvio `sigma`** (unidades da peça) —
    /// o que a CPU e a placa aplicam. Vazio para `sigma ≤ 0` (no-op).
    #[must_use]
    pub fn polinomio(&self, sigma: f32) -> Vec<f32> {
        if !(sigma > 0.0) {
            return Vec::new();
        }
        let s = f64::from(sigma);
        coeficientes(0.5 * s * s, f64::from(self.lambda_sup))
    }

    /// ⭐⭐⭐⭐ **O Gaussiano de desvio `sigma` NA SUPERFÍCIE**, nas primeiras
    /// [`Self::amostras`] entradas de `buf` (a cauda fica como estava).
    pub fn desfoca<T: Canal>(&self, sigma: f32, buf: &mut [T]) {
        let d = self.polinomio(sigma);
        let n = self.amostras();
        debug_assert!(buf.len() >= n, "o plano tem {n} amostras");
        if d.is_empty() || buf.len() < n {
            return;
        }
        let u = &mut buf[..n];
        // `T₀ = A u`; `y = u − d₀·T₀`.
        let mut atual = vec![T::ZERO; n];
        {
            let u: &[T] = u;
            self.por_linhas(&mut atual, |a, out: &mut T| *out = self.a_vezes(a, u));
        }
        for (y, t0) in u.iter_mut().zip(&atual) {
            y.soma(-d[0], *t0);
        }
        // `T₁ = X T₀` e depois `T_{k+1} = 2 X T_k − T_{k−1}`; `y −= d_k·T_k`.
        let mut anterior = vec![T::ZERO; n];
        let mut seguinte = vec![T::ZERO; n];
        let escala = 2.0 / self.lambda_sup;
        for (k, &dk) in d.iter().enumerate().skip(1) {
            let (tk, tk1) = (&atual, &anterior);
            let primeiro = k == 1;
            self.por_linhas_com(&mut seguinte, u, |a, out: &mut T, y: &mut T| {
                // `X z = escala·A z − z`.
                let mut x = T::ZERO;
                x.soma(escala, self.a_vezes(a, tk));
                x.soma(-1.0, tk[a]);
                let mut v = T::ZERO;
                if primeiro {
                    v = x;
                } else {
                    v.soma(2.0, x);
                    v.soma(-1.0, tk1[a]);
                }
                *out = v;
                y.soma(-dk, v);
            });
            std::mem::swap(&mut anterior, &mut atual);
            std::mem::swap(&mut atual, &mut seguinte);
        }
    }

    /// `(A z)_a = Σ_b w_ab (z_a − z_b) / m_a`.
    #[inline]
    fn a_vezes<T: Canal>(&self, a: usize, z: &[T]) -> T {
        let k = self.inv_massa[a];
        if k == 0.0 {
            return T::ZERO;
        }
        let za = z[a];
        let mut s = T::ZERO;
        for e in self.ini[a] as usize..self.ini[a + 1] as usize {
            s.soma(self.peso[e], T::menos(za, z[self.viz[e] as usize]));
        }
        let mut out = T::ZERO;
        out.soma(k, s);
        out
    }

    fn threads(&self, n: usize) -> usize {
        let t = match self.threads {
            0 => std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
            t => t,
        };
        if n < 1 << 14 { 1 } else { t }
    }

    /// `f(a, &mut out[a])` em todas as linhas, repartidas.
    fn por_linhas<T: Canal>(&self, out: &mut [T], f: impl Fn(usize, &mut T) + Sync) {
        let n = out.len();
        let threads = self.threads(n);
        if threads <= 1 {
            for (a, o) in out.iter_mut().enumerate() {
                f(a, o);
            }
            return;
        }
        let por = n.div_ceil(threads);
        std::thread::scope(|s| {
            for (bi, banda) in out.chunks_mut(por).enumerate() {
                let f = &f;
                s.spawn(move || {
                    for (i, o) in banda.iter_mut().enumerate() {
                        f(bi * por + i, o);
                    }
                });
            }
        });
    }

    /// A irmã de [`Self::por_linhas`] que também escreve `y[a]` (bandas iguais).
    fn por_linhas_com<T: Canal>(
        &self,
        out: &mut [T],
        y: &mut [T],
        f: impl Fn(usize, &mut T, &mut T) + Sync,
    ) {
        let n = out.len();
        let threads = self.threads(n);
        if threads <= 1 {
            for (a, (o, yy)) in out.iter_mut().zip(y.iter_mut()).enumerate() {
                f(a, o, yy);
            }
            return;
        }
        let por = n.div_ceil(threads);
        std::thread::scope(|s| {
            for (bi, (banda, ys)) in out.chunks_mut(por).zip(y.chunks_mut(por)).enumerate() {
                let f = &f;
                s.spawn(move || {
                    for (i, (o, yy)) in banda.iter_mut().zip(ys.iter_mut()).enumerate() {
                        f(bi * por + i, o, yy);
                    }
                });
            }
        });
    }

    /// 🔎 A RÉGUA dos testes: o mesmo calor por passos explícitos pequenos
    /// (`(I − dt·A)^n`, `dt ≤ 1/λ_sup`), que converge para `e^{−tA}`.
    #[cfg(test)]
    pub(crate) fn desfoca_por_passos<T: Canal>(
        &self,
        sigma: f32,
        passos_por_unidade: f32,
        buf: &mut [T],
    ) {
        let t = 0.5 * sigma * sigma;
        let n = (t * self.lambda_sup * passos_por_unidade).ceil().max(1.0) as usize;
        let dt = t / n as f32;
        let m = self.amostras();
        let mut a = buf[..m].to_vec();
        let mut b = a.clone();
        for _ in 0..n {
            for (i, o) in b.iter_mut().enumerate() {
                let mut v = a[i];
                v.soma(-dt, self.a_vezes(i, &a));
                *o = v;
            }
            std::mem::swap(&mut a, &mut b);
        }
        buf[..m].copy_from_slice(&a);
    }
}
