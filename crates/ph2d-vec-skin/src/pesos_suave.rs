//! ⭐⭐⭐ **O CAMPO DE PESO LIDO COM DERIVADA CONTÍNUA** — a cura da ondulação de 2026-09-20.
//!
//! # ⛔⛔⛔ O defeito, medido
//!
//! Report do dono, 3.ª foto, com o arco que ele esperava marcado a verde: *«a imagem vetorial
//! deforma mal, com várias curvas ao longo do caminho. Baixa qualidade para um app pro»*.
//!
//! Medido (`skinned_mesh_ondulacao_tests`): ao longo das arestas que em repouso são **rectas**, a
//! curvatura da forma deformada troca de sinal **68** vezes pela lei ideal e **12** depois de o
//! caminho vectorial lhe ajustar cúbicas por cima. O contorno atravessa **119 triângulos** do
//! lattice, e a razão `68/119 = 0,57` é a assinatura: **cerca de uma onda por cada duas
//! travessias**.
//!
//! # ⭐⭐ A causa está no PAPER, e não é nossa
//!
//! O *Bounded Biharmonic Weights* (Jacobson et al., SIGGRAPH 2011) diz que os pesos são
//! **`C¹` nas alças e `C∞` em todo o resto** — e, duas páginas à frente, que o problema é
//! discretizado **com elementos finitos LINEARES**, *«os pesos tornam-se funções lineares por
//! troço cujos valores nos vértices se procuram»*.
//!
//! ⇒ **o campo verdadeiro é liso; as facetas são da DISCRETIZAÇÃO.** Um campo linear por triângulo
//! tem gradiente CONSTANTE lá dentro e um SALTO em cada aresta; a deformação multiplica-o por uma
//! rotação, e o que se vê é a curvatura a saltar em cada travessia.
//!
//! # ⭐ O oráculo, corrido sobre os NOSSOS dados (§0.9)
//!
//! A pergunta é de **interpolação de dados dispersos**, e o interpolante `C¹` de referência desde
//! os anos 60 é o **Clough–Tocher**. A SciPy (**BSD-3**) implementa-o ao lado do
//! `LinearNDInterpolator`, que é o que fazíamos — logo os dois lados da comparação saem do MESMO
//! oráculo, sobre a nossa malha e o nosso contorno:
//!
//! | campo de peso | ondulações | Σw | pior peso |
//! |---|---:|---|---:|
//! | `LinearNDInterpolator` (o que fazíamos) | `68` | `[1,000000, 1,000000]` | `+0,000000` |
//! | **`CloughTocher2DInterpolator`** | **`16`** | `[1,000000, 1,000000]` | `−0,000249` |
//! | molificação (média num disco), raio `1×` a `4×` a aresta | `50`–`70` | `1,000000` | `0` |
//!
//! ⚠️ **A molificação foi medida porque preserva as duas invariantes do BBW por CONSTRUÇÃO** (é
//! uma combinação convexa) e **perdeu**: ela borra o campo sem tirar os vincos, e aumentar o raio
//! **piora** (`50 → 70`). *O que é preciso não é um campo mais liso em média, é um campo cuja
//! DERIVADA não salta.*
//!
//! ⚠️ E a não-negatividade, que é a razão de existir do BBW, **sobrevive por medição e não por
//! promessa**: o pior peso do `C¹` é `−0,000249` num universo de `3 264` valores, com massa
//! negativa total de `0,0095`. *É ruído, não um overshoot.*
//!
//! # ⭐⭐⭐ A LEI que se implementa: o método SIDE–VERTEX (Nielson, 1979)
//!
//! Ele é o mesmo `C¹` do Clough–Tocher e é **verificável a olho na álgebra**, que é a razão da
//! escolha: o Clough–Tocher parte cada triângulo em três e a continuidade através da aresta sai de
//! condições sobre dezanove ordenadas de Bézier — fácil de escrever quase certo, e um campo *quase*
//! `C¹` tem exactamente o aspecto de um campo `C¹`.
//!
//! Aqui, sobre a aresta `jk` a lei **reduz-se aos dados que os dois triângulos PARTILHAM** (os
//! valores e os gradientes de `j` e `k`), e é isso que faz a costura fechar. As três metades:
//!
//! 1. a recta que sai do vértice `i` e passa por `p` corta a aresta oposta em `Q`;
//! 2. ao longo dela corre-se uma **cúbica de Hermite** entre `(f_i, g_i)` e `(f_Q, g_Q)`, com
//!    `f_Q` e `g_Q` interpolados **linearmente** na aresta;
//! 3. as três respostas misturam-se com `W_i = (λ_j λ_k)² / Σ`, que vale **1** na aresta oposta a
//!    `i` e **0** nas outras duas.
//!
//! ⚠️ **A partição da unidade é exacta e não uma esperança:** a lei é LINEAR nos dados, e com
//! `f ≡ 1` e `g ≡ 0` cada Hermite devolve `1` e os `W` somam `1`. O mesmo argumento dá a
//! reprodução de funções lineares. Os dois estão gateados.

use crate::pesos::CampoDoDominio;

/// **O campo com os gradientes por vértice já derivados** — a leitura `C¹`.
///
/// ⚠️⚠️ **Ele é DERIVADO e não guardado**, e isso é uma decisão: os gradientes saem dos pesos e da
/// malha por uma conta determinista, logo escrevê-los no ficheiro seria um *vector paralelo* que
/// pode divergir do que descreve — o defeito que o `corner_radius` deste repo proíbe por escrito.
/// ⇒ o `SkinBind::source` não muda, e um bind gravado ontem lê-se `C¹` hoje.
///
/// ⚠️ **Construído UMA vez por forma por quadro** e consultado por amostra de curva: derivar por
/// consulta seria `O(V·B)` dentro do laço do desenho.
pub struct CampoSuave<'a> {
    campo: &'a CampoDoDominio,
    /// `grad[(v * ossos + j)]` — o gradiente do peso do tendão `j` no vértice `v`, no espaço da
    /// MALHA (o mesmo em que `malha.rest` vive).
    grad: Vec<[f64; 2]>,
    ossos: usize,
}

impl<'a> CampoSuave<'a> {
    /// `None` quando o campo não fecha — a mesma recusa da [`CampoDoDominio::valida`].
    #[must_use]
    pub fn novo(campo: &'a CampoDoDominio) -> Option<Self> {
        if !campo.valida() {
            return None;
        }
        let ossos = campo.ossos();
        let grad = gradientes(campo, ossos);
        Some(Self { campo, grad, ossos })
    }

    /// ⭐⭐⭐ **A linha de pesos em `p`, com derivada contínua** — a irmã `C¹` da
    /// [`CampoDoDominio::linha`].
    ///
    /// `None` fora da malha, exactamente como ela.
    #[must_use]
    pub fn linha(&self, p_local: [f64; 2]) -> Option<Vec<f64>> {
        let p = self.campo.para_malha_pub(p_local);
        let m = &self.campo.malha;
        let n = self.ossos;
        for t in &m.tris {
            let (ia, ib, ic) = (t[0] as usize, t[1] as usize, t[2] as usize);
            let (a, b, c) = (m.rest[ia], m.rest[ib], m.rest[ic]);
            let den = (b[0] - a[0]).mul_add(c[1] - a[1], -((c[0] - a[0]) * (b[1] - a[1])));
            if den.abs() < 1e-12 {
                continue;
            }
            let l1 = (b[0] - p[0]).mul_add(c[1] - p[1], -((c[0] - p[0]) * (b[1] - p[1]))) / den;
            let l2 = (c[0] - p[0]).mul_add(a[1] - p[1], -((a[0] - p[0]) * (c[1] - p[1]))) / den;
            let l3 = 1.0 - l1 - l2;
            if l1 < -1e-9 || l2 < -1e-9 || l3 < -1e-9 {
                continue;
            }
            return Some(self.side_vertex([ia, ib, ic], [a, b, c], [l1, l2, l3], n));
        }
        None
    }

    /// A lei, num triângulo, para os `n` ossos de uma vez.
    fn side_vertex(&self, idx: [usize; 3], pv: [[f64; 2]; 3], lam: [f64; 3], n: usize) -> Vec<f64> {
        // ⚠️⚠️ **Nos vértices o denominador dos pesos ANULA-SE** (dois `λ` a zero) — e a
        // resposta ali é EXACTA na mesma, porque o ramo de `soma == 0` abaixo cai na mistura
        // linear, que num vértice devolve o próprio valor dele.
        //
        // ⛔ Havia aqui uma cerca explícita para esse caso e ela foi **APAGADA**: a mutação que a
        // desliga **sobreviveu aos três gates**, e a lei desta casa é que *uma linha que a mutação
        // não consegue matar é comentário com sintaxe de código*. O que a cobre está gateado pelo
        // `a_leitura_c1_e_exacta_nos_vertices`.
        // ⚠️ **O expoente `2` é o clássico e NÃO está discriminado por estes gates** (medido: com
        // `powi(1)` a derivada continua a desaparecer com a sonda). A razão é que na aresta os
        // três raios devolvem o MESMO valor, logo o termo `Σ (∂W_i/∂n)·P_i` colapsa em
        // `f_aresta · Σ ∂W_i/∂n = 0` seja qual for o expoente. *Fica o `2`, que é o da
        // literatura, e a não-discriminação fica escrita em vez de escondida.*
        let q = [
            (lam[1] * lam[2]).powi(2),
            (lam[2] * lam[0]).powi(2),
            (lam[0] * lam[1]).powi(2),
        ];
        let soma: f64 = q.iter().sum();
        if soma <= 0.0 {
            // Numa aresta exacta dois `λ` são zero só nos vértices, já tratados acima; aqui isto
            // é a degenerescência numérica, e a leitura honesta é a linear.
            return (0..n)
                .map(|j| {
                    lam[0].mul_add(
                        self.f(idx[0], j),
                        lam[1].mul_add(self.f(idx[1], j), lam[2] * self.f(idx[2], j)),
                    )
                })
                .collect();
        }
        (0..n)
            .map(|j| {
                let mut acc = 0.0;
                for (i, &qi) in q.iter().enumerate() {
                    if qi == 0.0 {
                        continue;
                    }
                    acc += qi / soma * self.raio(idx, pv, lam, i, j);
                }
                acc
            })
            .collect()
    }

    /// A cúbica de Hermite ao longo da recta `vértice i → aresta oposta`, no ponto `p`.
    fn raio(&self, idx: [usize; 3], pv: [[f64; 2]; 3], lam: [f64; 3], i: usize, j: usize) -> f64 {
        let (a, b) = ((i + 1) % 3, (i + 2) % 3);
        // Onde a recta corta a aresta oposta: a fracção sai dos próprios `λ`.
        let dl = lam[a] + lam[b];
        if dl <= 1e-12 {
            return self.f(idx[i], j);
        }
        let t = lam[b] / dl;
        let qp = [
            (1.0 - t).mul_add(pv[a][0], t * pv[b][0]),
            (1.0 - t).mul_add(pv[a][1], t * pv[b][1]),
        ];
        // ⭐⭐⭐ **O VALOR NA ARESTA É A CÚBICA DE HERMITE DOS DADOS DELA, e não a recta.**
        //
        // ⛔⛔ A 1.ª redacção pôs aqui a interpolação LINEAR de `f_a` e `f_b`, e o gate da derivada
        // apanhou-a: o salto **não desaparecia** quando a sonda estreitava (`2,4e−2` no limite,
        // contra `1,1e−1` da lei linear) — *o que é `C¹` tem de tender para zero, e o que só é
        // «mais liso» estabiliza*. A restrição de um interpolante `C¹` a uma aresta **é** a
        // cúbica de Hermite dos dados dessa aresta; pondo a recta, os dois triângulos concordam
        // no VALOR e discordam na derivada, que é precisamente o que se queria curar.
        let ea = [pv[b][0] - pv[a][0], pv[b][1] - pv[a][1]];
        let (ga, gb) = (self.g(idx[a], j), self.g(idx[b], j));
        let (fa, fb) = (self.f(idx[a], j), self.f(idx[b], j));
        let (ma, mb) = (
            ga[0].mul_add(ea[0], ga[1] * ea[1]),
            gb[0].mul_add(ea[0], gb[1] * ea[1]),
        );
        let (t2, t3) = (t * t, t * t * t);
        let (h00, h10) = (2.0 * t3 - 3.0 * t2 + 1.0, t3 - 2.0 * t2 + t);
        let (h01, h11) = (-2.0 * t3 + 3.0 * t2, t3 - t2);
        let fq = h00 * fa + h10 * ma + h01 * fb + h11 * mb;
        // ⚠️ A DERIVADA da cúbica ao longo da aresta, e a componente NORMAL vinda linearmente —
        // as duas são função só dos dados da aresta, logo os dois triângulos vêem o mesmo.
        //
        // ⛔⛔ **Escrita em claro de propósito.** A 1.ª redacção comprimiu-a em `mul_add`
        // encadeados e TROCOU O SINAL do termo dos valores (`6(t²−t)(f_b−f_a)` em vez de
        // `(f_a−f_b)`), e o resultado não foi um erro pequeno: o salto da derivada foi de
        // `2,4e−2` para `4,8e−1`, **quatro vezes pior que a lei linear que ela vinha curar**.
        // *Uma base polinomial escreve-se onde se pode ler.*
        let dfe = 6.0 * (t2 - t) * (fa - fb)
            + (3.0 * t2 - 4.0 * t + 1.0) * ma
            + (3.0 * t2 - 2.0 * t) * mb;
        let e2 = ea[0].mul_add(ea[0], ea[1] * ea[1]);
        let gq = if e2 > 1e-18 {
            // Separa a parte tangencial (a da cúbica) da normal (linear entre os extremos).
            let tang = dfe / e2;
            let nrm = [-ea[1], ea[0]];
            let gl = [
                (1.0 - t).mul_add(ga[0], t * gb[0]),
                (1.0 - t).mul_add(ga[1], t * gb[1]),
            ];
            let cn = gl[0].mul_add(nrm[0], gl[1] * nrm[1]) / e2;
            [
                tang.mul_add(ea[0], cn * nrm[0]),
                tang.mul_add(ea[1], cn * nrm[1]),
            ]
        } else {
            [
                (1.0 - t).mul_add(ga[0], t * gb[0]),
                (1.0 - t).mul_add(ga[1], t * gb[1]),
            ]
        };
        let d = [qp[0] - pv[i][0], qp[1] - pv[i][1]];
        let gi = self.g(idx[i], j);
        let (m0, m1) = (
            gi[0].mul_add(d[0], gi[1] * d[1]),
            gq[0].mul_add(d[0], gq[1] * d[1]),
        );
        // `s = 0` no vértice `i`, `s = 1` na aresta.
        let s = 1.0 - lam[i];
        let (s2, s3) = (s * s, s * s * s);
        let h00 = 2.0f64.mul_add(s3, -3.0 * s2) + 1.0;
        let h10 = s3 - 2.0 * s2 + s;
        let h01 = (-2.0f64).mul_add(s3, 3.0 * s2);
        let h11 = s3 - s2;
        h11.mul_add(
            m1,
            h01.mul_add(fq, h10.mul_add(m0, h00 * self.f(idx[i], j))),
        )
    }

    fn f(&self, v: usize, j: usize) -> f64 {
        self.campo
            .pesos
            .get(v * self.ossos + j)
            .copied()
            .unwrap_or(0.0)
    }

    fn g(&self, v: usize, j: usize) -> [f64; 2] {
        self.grad
            .get(v * self.ossos + j)
            .copied()
            .unwrap_or([0.0; 2])
    }
}

/// ⭐⭐ **O GRADIENTE DE CADA PESO EM CADA VÉRTICE**, por mínimos quadrados sobre o anel dele.
///
/// ⚠️ **A partição da unidade sobrevive por construção:** a estimativa é **LINEAR** nos valores,
/// logo `Σ_j g_j` é a estimativa aplicada a `Σ_j f_j ≡ 1`, que é um campo constante — e os mínimos
/// quadrados de um campo constante dão o gradiente **ZERO** exacto.
///
/// ⚠️ **Um anel degenerado** (vizinhos colineares, ou nenhum) devolve gradiente nulo, e a lei cai
/// numa Hermite de derivadas nulas — que ainda é `C¹`, só mais chata. *Devolver lixo ali daria um
/// campo que oscila exactamente onde a malha é pior.*
fn gradientes(campo: &CampoDoDominio, ossos: usize) -> Vec<[f64; 2]> {
    let m = &campo.malha;
    let nv = m.rest.len();
    // O anel de cada vértice, colhido das faces.
    let mut anel: Vec<Vec<u32>> = vec![Vec::new(); nv];
    for t in &m.tris {
        for k in 0..3 {
            let (v, a, b) = (t[k] as usize, t[(k + 1) % 3], t[(k + 2) % 3]);
            for u in [a, b] {
                if !anel[v].contains(&u) {
                    anel[v].push(u);
                }
            }
        }
    }
    let mut out = vec![[0.0_f64; 2]; nv * ossos];
    for v in 0..nv {
        let p = m.rest[v];
        // A matriz normal `Σ d dᵀ` é a MESMA para todos os ossos — inverte-se uma vez por vértice.
        let (mut sxx, mut sxy, mut syy) = (0.0_f64, 0.0_f64, 0.0_f64);
        for &u in &anel[v] {
            let q = m.rest[u as usize];
            let d = [q[0] - p[0], q[1] - p[1]];
            sxx = d[0].mul_add(d[0], sxx);
            sxy = d[0].mul_add(d[1], sxy);
            syy = d[1].mul_add(d[1], syy);
        }
        let det = sxx.mul_add(syy, -(sxy * sxy));
        if det.abs() < 1e-18 {
            continue;
        }
        for j in 0..ossos {
            let fv = campo.pesos.get(v * ossos + j).copied().unwrap_or(0.0);
            let (mut bx, mut by) = (0.0_f64, 0.0_f64);
            for &u in &anel[v] {
                let q = m.rest[u as usize];
                let d = [q[0] - p[0], q[1] - p[1]];
                let df = campo
                    .pesos
                    .get(u as usize * ossos + j)
                    .copied()
                    .unwrap_or(0.0)
                    - fv;
                bx = d[0].mul_add(df, bx);
                by = d[1].mul_add(df, by);
            }
            out[v * ossos + j] = [
                syy.mul_add(bx, -(sxy * by)) / det,
                sxx.mul_add(by, -(sxy * bx)) / det,
            ];
        }
    }
    out
}

#[cfg(test)]
#[path = "pesos_suave_tests.rs"]
mod tests;
