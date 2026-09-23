//! ⏱️⭐⭐⭐⭐ **O VASO POR FÓRMULA — quanto ele custa, e quanto do desenho ele consegue dizer.**
//!
//! # A pergunta do dono (2026-09-23)
//!
//! *«Não seria possível criar vasos com fórmulas para que tudo fique rápido?»* — e o mecanismo que
//! ele aponta está CERTO: uma fita paga `26,5` linhas por pedaço RECTO e `50,9` por pedaço CURVO
//! (medido, `profile::custo`), logo um contorno desenhado custa **`O(pedaços)`**; uma fórmula custa
//! **`O(grau)`**, independentemente de quantas curvas a silhueta parece ter.
//!
//! ⚠️ **Mas «possível» tem duas metades, e só uma é o relógio:**
//!
//! 1. **quanto custa** — as linhas da fita e o custo por amostra na placa;
//! 2. **quanto do DESENHO a fórmula consegue dizer** — porque se ela não reproduz o lábio e a
//!    parede interna do vaso, ela é rápida a desenhar outra peça.
//!
//! Esta sonda mede as duas.
//!
//! # A forma da fórmula, e porque é esta
//!
//! Um sólido de revolução é, no plano `(u, v)` com `u = √(x² + z²)`, a região entre **duas funções
//! da altura**: `dentro(v) ≤ u ≤ fora(v)`. ⭐ É essa a forma que torna o vaso expressável: a parede
//! externa sobe, a interna desce, e a base é sólida porque ali `dentro(v)` é **negativo** — *sem
//! degrau nenhum, que é o que uma fórmula não sabe fazer*.
//!
//! As duas funções são polinómios de **Chebyshev** avaliados por **Clenshaw** (`3` operações por
//! grau), e não monómios: a base de Chebyshev é bem condicionada no intervalo, e o Clenshaw é o
//! esquema que a casa usaria no dispositivo.
//!
//! ⚠️⚠️ **E o campo tem de ser NORMALIZADO ou a marcha atravessa a peça.** `u − fora(v)` **não** é a
//! distância à curva `u = fora(v)`: ela é maior do que a distância quando a curva é inclinada, e uma
//! esfera-marcha que acredite num valor MAIOR do que a distância dá um passo para dentro do sólido.
//! ⇒ divide-se por `√(1 + máx|fora′|²)`, um **majorante global da inclinação** — conservador em todo
//! ponto, uma multiplicação, e é a mesma lei que o [`ph2d_field_eval::field_shrink`] da casa aplica.

use fidget::context::Tree;
use ph2d_field::Profile;

/// Quantas alturas a silhueta é amostrada.
const ALTURAS: usize = 512;
/// Quantos passos em `u` para achar a fronteira em cada altura.
const PASSOS_U: usize = 4096;

/// A silhueta do vaso como **duas funções da altura** — `(v, dentro, fora)`, só onde há peça.
///
/// ⚠️⚠️ **`dentro` é `NaN` onde a parede interna NÃO EXISTE, e isso é a correcção de uma régua que
/// fabricava a resposta.** A 1.ª redacção registava `0` na base do vaso (onde `u = 0` está DENTRO
/// do sólido), o que faz a função ter um **DEGRAU** de `0` para `0,19` à altura em que a parede
/// interna começa — e um polinómio a seguir um degrau erra metade dele. Medido: o erro máximo lia
/// `0,0885` ao grau `4` e `0,0754` ao grau `24`, *sem melhorar*, que é exactamente a assinatura de
/// uma descontinuidade. ⭐ No CAMPO não há degrau nenhum: ali a base é sólida porque `dentro(v)` é
/// **negativo**, e é isso que o ajuste tem de poder fazer.
///
/// ⚠️ **A régua é o SINAL do campo do perfil** ([`ph2d_field_eval::profile_index::ProfileIndex::sd`]),
/// que é a mesma lei que a peça usa. *Ler os vértices do desenho daria a silhueta do polígono e não
/// a da peça, que tem arcos.*
fn silhueta(profile: &Profile) -> Vec<(f64, f64, f64)> {
    let idx = crate::profile_index::ProfileIndex::build(profile);
    let (plo, phi) = profile.bounds();
    let u_max = phi[0].max(plo[0].abs()) * 1.05;
    let mut out = Vec::with_capacity(ALTURAS);
    for i in 0..ALTURAS {
        #[allow(clippy::cast_precision_loss)]
        let v = f64::from(plo[1])
            + (f64::from(phi[1]) - f64::from(plo[1])) * (i as f64 + 0.5) / ALTURAS as f64;
        let mut dentro: Option<f64> = None;
        let mut fora: Option<f64> = None;
        let mut estava = false;
        for j in 0..=PASSOS_U {
            #[allow(clippy::cast_precision_loss)]
            let u = f64::from(u_max) * j as f64 / PASSOS_U as f64;
            #[allow(clippy::cast_possible_truncation)]
            let esta = idx.sd(u as f32, v as f32) < 0.0;
            if esta && !estava {
                dentro = Some(u);
            }
            if !esta && estava {
                fora = Some(u);
            }
            estava = esta;
        }
        if let Some(b) = fora {
            // ⭐⭐⭐ **`dentro` só existe quando a fronteira de entrada NÃO é o próprio eixo — e o
            // limiar tem de ser maior que o PASSO da varredura.**
            //
            // ⛔⛔ Medido: com um limiar de `1e-6` (menor que o passo de `8,4e-5`) o fundo do vaso
            // registava parede interna em `~1e-4`, porque o ponto `u = 0` está **em cima** da
            // costura do eixo e lê `sd = 0`, que não é `< 0` ⇒ o passo seguinte é o primeiro
            // «dentro». ⇒ `dentro(v)` ganhava um **DEGRAU** de `~0` para `0,153` à altura da base, e
            // o ajuste polinomial lia `0,09` de erro *em todos os graus* — a assinatura de uma
            // descontinuidade. *Uma régua cujo limiar é menor que o próprio passo de amostragem não
            // distingue «em zero» de «perto de zero».*
            let piso = 4.0 * f64::from(u_max) / PASSOS_U as f64;
            out.push((v, dentro.filter(|d| *d > piso).unwrap_or(f64::NAN), b));
        }
    }
    out
}

/// Um ajuste de mínimos quadrados na base de **Chebyshev**, no intervalo `[lo, hi]` de `x`.
fn ajusta(xs: &[f64], ys: &[f64], lo: f64, hi: f64, grau: usize) -> Vec<f64> {
    let n = grau + 1;
    let base = |x: f64| {
        let t = (2.0 * (x - lo) / (hi - lo) - 1.0).clamp(-1.0, 1.0);
        let mut b = vec![0.0; n];
        for (k, s) in b.iter_mut().enumerate() {
            *s = (k as f64 * t.acos()).cos();
        }
        b
    };
    let mut a = vec![vec![0.0f64; n + 1]; n];
    for (x, y) in xs.iter().zip(ys).filter(|(_, y)| y.is_finite()) {
        let b = base(*x);
        for i in 0..n {
            for j in 0..n {
                a[i][j] += b[i] * b[j];
            }
            a[i][n] += b[i] * y;
        }
    }
    // Eliminação de Gauss com pivô parcial — `n ≤ 13`.
    for i in 0..n {
        let p = (i..n)
            .max_by(|x, y| a[*x][i].abs().total_cmp(&a[*y][i].abs()))
            .unwrap();
        a.swap(i, p);
        if a[i][i].abs() < 1e-14 {
            continue;
        }
        let (pivo, resto) = a[i..].split_first_mut().expect("i < n");
        for linha in resto {
            let f = linha[i] / pivo[i];
            for (alvo, p) in linha[i..].iter_mut().zip(&pivo[i..]) {
                *alvo -= f * p;
            }
        }
    }
    let mut c = vec![0.0f64; n];
    for i in (0..n).rev() {
        if a[i][i].abs() < 1e-14 {
            continue;
        }
        let mut s = a[i][n];
        for j in (i + 1)..n {
            s -= a[i][j] * c[j];
        }
        c[i] = s / a[i][i];
    }
    c
}

/// Clenshaw em números — para medir o erro do ajuste.
fn clenshaw(c: &[f64], x: f64, lo: f64, hi: f64) -> f64 {
    let t = (2.0 * (x - lo) / (hi - lo) - 1.0).clamp(-1.0, 1.0);
    let (mut b1, mut b2) = (0.0, 0.0);
    for k in (1..c.len()).rev() {
        let b = 2.0 * t * b1 - b2 + c[k];
        b2 = b1;
        b1 = b;
    }
    t * b1 - b2 + c[0]
}

/// Clenshaw em ÁRVORE — `3` operações por grau, que é o custo que esta sonda existe para medir.
fn clenshaw_tree(c: &[f64], v: &Tree, lo: f64, hi: f64) -> Tree {
    let t = (v.clone() * Tree::constant(2.0 / (hi - lo))
        - Tree::constant(2.0 * lo / (hi - lo) + 1.0))
    .max(-1.0)
    .min(1.0);
    let mut b1 = Tree::constant(0.0);
    let mut b2 = Tree::constant(0.0);
    for k in (1..c.len()).rev() {
        let b = Tree::constant(2.0) * t.clone() * b1.clone() - b2 + Tree::constant(c[k]);
        b2 = b1;
        b1 = b;
    }
    t * b1 - b2 + Tree::constant(c[0])
}

/// A peça por fórmula: o sólido entre a parede externa e a altura, MENOS a cavidade.
///
/// ⚠️⚠️ **A cavidade é uma INTERSECÇÃO e não uma extrapolação, e isso foi a segunda correcção da
/// régua.** A 1.ª redacção punha as duas paredes no mesmo `max` e pedia ao polinómio de DENTRO que
/// ficasse negativo abaixo do arranque da parede interna — uma extrapolação livre, que numa base de
/// Chebyshev sobre o domínio INTEIRO sobe (Runge) e **abre um buraco na base sólida** (medido:
/// `+0,086` a `+0,104`). ⭐ A forma certa diz a mesma coisa sem pedir nada ao ajuste: a cavidade é
/// *«dentro do raio interno **E** acima do arranque»*, e o sólido é a peça **menos** ela.
///
/// ⭐ E cada parede é ajustada no **seu** domínio, com o Clenshaw a prender `t` em `[−1, 1]` ⇒ abaixo
/// do arranque o raio interno fica **preso ao valor da ponta**, que é a resposta sensata e não uma
/// extrapolação.
fn campo(dentro: &[f64], fora: &[f64], dom_d: (f64, f64), dom_f: (f64, f64), lip: f64) -> Tree {
    let (x, y, z) = (Tree::x(), Tree::y(), Tree::z());
    let u = crate::ops::safe_sqrt(x.square() + z.square());
    // ⚠️ O majorante global da inclinação — conservador em todo ponto, uma multiplicação.
    let k = Tree::constant(1.0 / (1.0 + lip * lip).sqrt());
    let solido = ((u.clone() - clenshaw_tree(fora, &y, dom_f.0, dom_f.1)) * k.clone())
        .max(y.clone() - Tree::constant(dom_f.1))
        .max(Tree::constant(dom_f.0) - y.clone());
    let cavidade =
        ((u - clenshaw_tree(dentro, &y, dom_d.0, dom_d.1)) * k).max(Tree::constant(dom_d.0) - y);
    solido.max(-cavidade)
}

fn linhas(t: &Tree) -> usize {
    crate::Field::from_tree(t)
        .tape_shape()
        .map_or(0, |s| s.guardados)
}

/// Uma linha da tabela: o grau, o pior erro do ajuste, a inclinação máxima e as linhas de WGSL.
#[derive(Clone, Copy, Debug)]
pub struct LinhaDaFormula {
    pub grau: usize,
    /// O pior desvio da parede EXTERNA, em unidades da peça.
    pub erro_fora: f64,
    /// O pior desvio da parede INTERNA, medido **só onde ela existe**.
    pub erro_dentro: f64,
    /// A altura em que o pior erro da parede interna está.
    pub onde_dentro: f64,
    pub inclinacao: f64,
    pub linhas: usize,
}

/// ⏱️⭐⭐⭐⭐ **A porta da sonda: o vaso por fórmula, grau a grau.**
///
/// ⚠️ `None` quando a régua não acha a peça (a silhueta saiu com menos de metade das alturas) — que
/// é o único jeito de esta tabela mentir sem se ver.
#[doc(hidden)]
#[must_use]
pub fn probe_formula_do_perfil(profile: &Profile, graus: &[usize]) -> Option<Vec<LinhaDaFormula>> {
    let s = silhueta(profile);
    if s.len() <= ALTURAS / 2 {
        return None;
    }
    let (lo, hi) = (s[0].0, s[s.len() - 1].0);
    let vs: Vec<f64> = s.iter().map(|(v, _, _)| *v).collect();
    let ds: Vec<f64> = s.iter().map(|(_, d, _)| *d).collect();
    let fs: Vec<f64> = s.iter().map(|(_, _, f)| *f).collect();
    // ⭐ **Cada parede no SEU domínio** — a interna só existe de onde ela arranca para cima.
    let dentro_vs: Vec<f64> = vs
        .iter()
        .zip(&ds)
        .filter(|(_, d)| d.is_finite())
        .map(|(v, _)| *v)
        .collect();
    if dentro_vs.len() < 8 {
        return None;
    }
    let dom_d = (dentro_vs[0], dentro_vs[dentro_vs.len() - 1]);
    let dom_f = (lo, hi);
    Some(
        graus
            .iter()
            .map(|&grau| {
                let cd = ajusta(&vs, &ds, dom_d.0, dom_d.1, grau);
                let cf = ajusta(&vs, &fs, dom_f.0, dom_f.1, grau);
                let pior = |c: &[f64], ys: &[f64], d: (f64, f64)| {
                    vs.iter()
                        .zip(ys)
                        .filter(|(_, y)| y.is_finite())
                        .map(|(v, y)| (clenshaw(c, *v, d.0, d.1) - y).abs())
                        .fold(0.0f64, f64::max)
                };
                let (erro_fora, erro_dentro) = (pior(&cf, &fs, dom_f), pior(&cd, &ds, dom_d));
                // ⭐⭐⭐ **O MESMO erro, sem a banda em que o FUNDO da cavidade manda.**
                //
                // ⚠️ Onde a parede interna encontra o fundo, o desenho tem um segmento
                // **horizontal** — e em `u = dentro(v)` isso é uma tangente **VERTICAL**, que
                // polinómio nenhum em `v` tem. *Mas o fundo não é a parede: ele é o outro termo do
                // `max` da cavidade*, e a quina entre os dois sai da intersecção. ⇒ medir a parede
                // com a banda do fundo dentro é medir o termo errado.
                // ⭐⭐ **ONDE** o pior erro está — e foi esta coluna que achou o defeito da régua.
                //
                // ⚠️ Sem ela, um erro que não converge com o grau lê-se como *«o desenho tem uma
                // feição que polinómio nenhum diz»*. A 1.ª hipótese — que ele morava no FUNDO da
                // cavidade, onde o desenho tem um segmento horizontal (tangente VERTICAL em
                // `u = dentro(v)`) — foi medida e **REFUTADA**: excluir aquela banda não movia o
                // número. ⇒ o erro estava **na régua**, no limiar que distingue «a parede começa
                // aqui» de «o intervalo começa no eixo» (ver [`silhueta`]).
                let (_, onde_dentro) = vs
                    .iter()
                    .zip(&ds)
                    .filter(|(_, d)| d.is_finite())
                    .map(|(v, d)| ((clenshaw(&cd, *v, dom_d.0, dom_d.1) - d).abs(), *v))
                    .fold((0.0f64, f64::NAN), |a, b| if b.0 > a.0 { b } else { a });
                let inclinacao = vs
                    .windows(2)
                    .map(|w| {
                        let g = |c: &[f64], d: (f64, f64)| {
                            (clenshaw(c, w[1], d.0, d.1) - clenshaw(c, w[0], d.0, d.1))
                                / (w[1] - w[0])
                        };
                        g(&cd, dom_d).abs().max(g(&cf, dom_f).abs())
                    })
                    .fold(0.0f64, f64::max);
                LinhaDaFormula {
                    grau,
                    erro_fora,
                    erro_dentro,
                    onde_dentro,
                    inclinacao,
                    linhas: linhas(&campo(&cd, &cf, dom_d, dom_f, inclinacao)),
                }
            })
            .collect(),
    )
}

/// ⚠️ Só para a sonda: a faixa de alturas e a parede externa, para o cabeçalho da tabela.
#[doc(hidden)]
#[must_use]
pub fn probe_silhueta_do_perfil(profile: &Profile) -> Vec<(f64, f64, f64)> {
    silhueta(profile)
}
