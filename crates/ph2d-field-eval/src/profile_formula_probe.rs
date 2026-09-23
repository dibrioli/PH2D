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
///
/// ⚠️ **Número de estrutura, não de gosto**: é a densidade com que o ajuste vê as paredes, e
/// `24` primitivas com `128` alturas dão `~5` amostras por primitiva. Movê-lo pede re-correr a
/// [`crate::profile_formula::probe_formula_do_perfil`].
const ALTURAS: usize = 128;

/// ⭐⭐⭐ **A SILHUETA DE UM TORNO — as duas paredes como funções da altura.**
///
/// Devolve, por altura, `(v, dentro, fora)`, com `dentro` a ser **`NaN` onde a parede interna não
/// existe** (a base sólida do vaso).
///
/// # ⭐ Como ela é achada: pelas TRAVESSIAS, não por varredura
///
/// Uma recta horizontal a atravessar um contorno fechado cruza-o um número **par** de vezes, e o
/// sólido é o intervalo entre a 1.ª e a 2.ª travessia. ⇒ a silhueta sai de `O(primitivas)` contas
/// por altura, exactas. ⛔ A 1.ª redacção varria `u` em `4 096` passos e perguntava o sinal do
/// campo: `2` milhões de avaliações, e — pior — **um limiar mais pequeno que o próprio passo**, que
/// fabricou um degrau na parede interna e um erro de ajuste que não convergia em grau nenhum.
///
/// # ⚠️ A régua de «isto é o eixo» é a do PERFIL, e é uma porta só
///
/// A 1.ª travessia pode ser a **costura do eixo** (o segmento que fecha o contorno sobre `u = 0`),
/// e aí não há parede interna nenhuma. A régua é a `Profile::tolerance`, que é a MESMA com que o
/// [`crate::profile::sd_profile`] decide o que assenta no eixo. *Um número próprio aqui seria uma
/// segunda resposta a «o que encosta no eixo».*
///
/// # ⛔ A CERCA: mais de duas travessias ⇒ RECUSA
///
/// Três ou mais travessias querem dizer que o perfil tem um **sobressaliente** (a secção àquela
/// altura é mais de um anel), e aí ele **não** é a região entre duas funções da altura. `None` é a
/// resposta certa, e o chamador fica com o contorno desenhado — que sabe desenhar qualquer coisa.
pub(crate) fn silhueta(profile: &Profile) -> Option<Vec<(f64, f64, f64)>> {
    // ⛔ Um torno tem UM contorno. Dois são outra topologia, e a lei das duas funções não a diz.
    let [contorno] = profile.contours() else {
        return None;
    };
    let tol = f64::from(profile.tolerance());
    let (plo, phi) = profile.bounds();
    let (vb, vt) = (f64::from(plo[1]), f64::from(phi[1]));
    if !(vt - vb).is_finite() || vt - vb <= tol {
        return None;
    }
    let pts: Vec<[f64; 2]> = contorno
        .iter()
        .map(|p| [f64::from(p[0]), f64::from(p[1])])
        .collect();
    let mut out = Vec::with_capacity(ALTURAS);
    for i in 0..ALTURAS {
        #[allow(clippy::cast_precision_loss)]
        let v = vb + (vt - vb) * (i as f64 + 0.5) / ALTURAS as f64;
        let mut cruzes: Vec<f64> = Vec::with_capacity(4);
        for j in 0..pts.len() {
            let (a, b) = (pts[j], pts[(j + 1) % pts.len()]);
            // ⚠️ **A regra semi-aberta** (`[a.y, b.y)`) é a do enrolamento desta casa: ela conta
            // cada travessia UMA vez quando a recta passa exactamente por um vértice.
            let sobe = a[1] <= v && b[1] > v;
            let desce = b[1] <= v && a[1] > v;
            if sobe || desce {
                let t = (v - a[1]) / (b[1] - a[1]);
                cruzes.push(a[0] + t * (b[0] - a[0]));
            }
        }
        if cruzes.len() != 2 {
            // ⛔ Zero é uma altura fora da peça (acontece nas pontas); mais de duas é o
            // sobressaliente que esta lei não diz.
            if cruzes.len() > 2 {
                return None;
            }
            continue;
        }
        cruzes.sort_by(f64::total_cmp);
        let (d, f) = (cruzes[0], cruzes[1]);
        out.push((v, if d > tol { d } else { f64::NAN }, f));
    }
    (out.len() > ALTURAS / 2).then_some(out)
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

/// Os coeficientes de Chebyshev da DERIVADA, exactos — a recorrência clássica.
fn derivada(c: &[f64]) -> Vec<f64> {
    let n = c.len();
    let mut d = vec![0.0f64; n];
    for k in (1..n).rev() {
        #[allow(clippy::cast_precision_loss)]
        let dois_k = 2.0 * k as f64;
        d[k - 1] = if k + 1 < n { d[k + 1] } else { 0.0 } + dois_k * c[k];
    }
    if n > 0 {
        d[0] *= 0.5;
    }
    d
}

/// ⭐⭐⭐ **UM MAJORANTE VERDADEIRO da inclinação da parede** — `Σ|d_k|`, porque `|T_k| ≤ 1`.
///
/// ⛔⛔ **E ele não pode ser um máximo AMOSTRADO.** Um máximo amostrado erra sempre **para baixo**,
/// e aqui um majorante pequeno demais faz o factor de normalização ficar **grande** demais ⇒ o campo
/// devolve um valor **maior** do que a distância e a esfera-marcha dá um passo **para dentro do
/// sólido**. *Este é o número que decide se a peça fura.*
fn majorante_da_inclinacao(c: &[f64], lo: f64, hi: f64) -> f64 {
    // A regra da cadeia do mapa `v ↦ t = 2(v − lo)/(hi − lo) − 1`.
    derivada(c).iter().map(|x| x.abs()).sum::<f64>() * 2.0 / (hi - lo)
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
    campo_com(fora, dom_f, Some((dentro, dom_d)), lip)
}

/// O campo, com a cavidade OPCIONAL — um torno sólido não a tem.
fn campo_com(
    fora: &[f64],
    dom_f: (f64, f64),
    cavidade: Option<(&[f64], (f64, f64))>,
    lip: f64,
) -> Tree {
    let (x, y, z) = (Tree::x(), Tree::y(), Tree::z());
    let u = crate::ops::safe_sqrt(x.square() + z.square());
    // ⚠️ O majorante da inclinação — conservador em todo ponto, uma multiplicação.
    let k = Tree::constant(1.0 / (1.0 + lip * lip).sqrt());
    let solido = ((u.clone() - clenshaw_tree(fora, &y, dom_f.0, dom_f.1)) * k.clone())
        .max(y.clone() - Tree::constant(dom_f.1))
        .max(Tree::constant(dom_f.0) - y.clone());
    match cavidade {
        None => solido,
        Some((cd, dom_d)) => {
            let c = ((u - clenshaw_tree(cd, &y, dom_d.0, dom_d.1)) * k)
                .max(Tree::constant(dom_d.0) - y);
            solido.max(-c)
        }
    }
}

fn linhas(t: &Tree) -> usize {
    crate::Field::from_tree(t)
        .tape_shape()
        .map_or(0, |s| s.guardados)
}

/// ⭐⭐⭐⭐ **O GRAU dos dois polinómios que dizem as paredes.**
///
/// ⚠️ **Medido, não escolhido** (`probe_formula_do_perfil` no vaso da cena `5`, `24` primitivas):
///
/// | grau | erro da parede externa | da interna | linhas de WGSL | quadro previsto |
/// |---:|---:|---:|---:|---:|
/// | `8` | `0,0128` | `0,0062` | `73` | `6,5 ms` |
/// | `12` | `0,0072` | `0,0036` | `97` | `7,2 ms` |
/// | **`16`** | **`0,0026`** | **`0,0025`** | `121` | **`7,9 ms`** |
/// | `24` | `0,0014` | `0,0011` | `169` | `9,4 ms` |
///
/// ⭐ `16` é o joelho: de `12` para `16` o erro cai `2,8×` por `+25 %` de linhas; de `16` para `24`
/// cai `1,8×` por `+40 %`. A `16` as duas paredes ficam a `0,003` da peça, que é `0,9 %` do raio do
/// vaso — abaixo de um pixel no uso normal.
pub const GRAU: usize = 16;

/// ⏱️⭐⭐⭐⭐ **Quantas vezes a fórmula foi AJUSTADA** — só sob teste.
///
/// # Porque um contador, e não um relógio
///
/// A economia desta cura é **invisível a toda régua de valor**: a árvore que a região devolve é a
/// MESMA, ao bit, venha ela de um ajuste novo ou do mapa do [`crate::RegionCompiler`]. ⇒ o que se
/// mede é a CONTA.
///
/// ⛔⛔ **E o número que a justifica:** ajustar custa `0,0748 ms` e o `specialised_profile` corria
/// **por ladrilho × fatia** — `750` regiões a `1920×1080`, `39 406` com ladrilho `8` ⇒ `56` a
/// `2 948 ms` por quadro. *O A/B de CPU não o viu porque a poupança da marcha e o gasto da montagem
/// se cancelavam.*
///
/// ⚠️ **É um átomo GLOBAL, e isso é são sob `nextest`** (um processo por teste) e **poluído** sob
/// `cargo test`, que corre vários testes no mesmo processo — a lei do `CLAUDE.md` §5.0.
#[cfg(test)]
pub static AJUSTES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// ⭐⭐⭐⭐ **A FIDELIDADE que a fórmula tem de alcançar para ser usada** — fracção do raio da peça.
///
/// ⚠️⚠️ **Este é um limite de PRODUTO e o recurso dele é o OLHO, e é honesto dizê-lo.** A rota da
/// fórmula é aproximada por construção, e o dono aprovou-a nesses termos (2026-09-23) com o número
/// ao lado: no vaso da cena `5` as duas paredes ficam a `0,0026` de um raio de `0,326`, que é
/// **`0,8 %`** — abaixo de um pixel no uso normal.
///
/// ⛔ **Ela NÃO pode ser a `Profile::tolerance`**, e isso está medido: a tolerância do vaso é `1e-4`
/// e o ajuste ao grau `16` erra `2,6e-3` — `26×` mais. *Com aquela barra a peça do dono seria
/// recusada e a wave não compraria nada.*
///
/// ⭐ **E é esta cerca que devolve o perfil DEGRAU ao contorno desenhado:** uma parede quase
/// vertical (uma roldana com escalões) não é dizível por um polinómio da altura, e o erro do ajuste
/// diz isso em números — *a recusa sai de uma medição e não de uma lista de formas*.
pub const FIDELIDADE: f64 = 0.01;

/// ⭐⭐⭐⭐ **O TORNO POR FÓRMULA** — as duas paredes como polinómios da altura.
///
/// `None` quando a silhueta não é a região entre duas funções da altura (ver [`silhueta`]), e aí o
/// chamador fica com o contorno desenhado, que sabe desenhar qualquer coisa.
///
/// # A forma, e porque cada termo existe
///
/// ```text
/// sólido   = max( (u − fora(v))·k , v − v_topo , v_base − v )
/// cavidade = max( (u − dentro(v))·k , v_fundo − v )
/// campo    = max( sólido , −cavidade )
/// ```
///
/// ⭐ **A cavidade é uma INTERSECÇÃO** (*«dentro do raio interno **E** acima do fundo»*), e é isso
/// que faz a base sólida do vaso sair sem pedir ao ajuste que extrapole para baixo do arranque da
/// parede — uma extrapolação que numa base de Chebyshev **sobe** (Runge) e abriria um buraco.
///
/// ⚠️⚠️ **E o `k` é o que impede a peça de furar:** `u − fora(v)` **não** é a distância à curva
/// `u = fora(v)` — ela é MAIOR quando a curva é inclinada, e uma esfera-marcha que acredite num
/// valor maior do que a distância dá um passo para dentro do sólido.
#[must_use]
pub fn sd_revolve_por_formula(profile: &Profile) -> Option<Tree> {
    // ⏱️⭐⭐⭐ **O CONTADOR que mede a CONTA e não o valor** — ver [`AJUSTES`].
    #[cfg(test)]
    AJUSTES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let s = silhueta(profile)?;
    let vs: Vec<f64> = s.iter().map(|(v, _, _)| *v).collect();
    let ds: Vec<f64> = s.iter().map(|(_, d, _)| *d).collect();
    let fs: Vec<f64> = s.iter().map(|(_, _, f)| *f).collect();
    let dom_f = (vs[0], vs[vs.len() - 1]);
    let cf = ajusta(&vs, &fs, dom_f.0, dom_f.1, GRAU);
    let dentro_vs: Vec<f64> = vs
        .iter()
        .zip(&ds)
        .filter(|(_, d)| d.is_finite())
        .map(|(v, _)| *v)
        .collect();
    // ⚠️ **Menos de `GRAU + 2` amostras é um ajuste sem sujeito** — ali não há parede interna que
    // valha, e a peça é um torno SÓLIDO.
    let cavidade = (dentro_vs.len() >= GRAU + 2).then(|| {
        let dom_d = (dentro_vs[0], dentro_vs[dentro_vs.len() - 1]);
        (ajusta(&vs, &ds, dom_d.0, dom_d.1, GRAU), dom_d)
    });
    // ⭐⭐⭐⭐ **A CERCA DA FIDELIDADE** — ver [`FIDELIDADE`]. Ela mede o ajuste contra a silhueta que
    // a própria peça produziu, nas duas paredes, e RECUSA quando o desenho diz algo que um polinómio
    // da altura não diz (um escalão, uma parede vertical).
    let barra = {
        let (plo, phi) = profile.bounds();
        FIDELIDADE * f64::from(phi[0].max(plo[0].abs())).max(f64::EPSILON)
    };
    let pior = |c: &[f64], ys: &[f64], d: (f64, f64)| {
        vs.iter()
            .zip(ys)
            .filter(|(_, y)| y.is_finite())
            .map(|(v, y)| (clenshaw(c, *v, d.0, d.1) - y).abs())
            .fold(0.0f64, f64::max)
    };
    if pior(&cf, &fs, dom_f) > barra {
        return None;
    }
    if let Some((cd, dom_d)) = cavidade.as_ref()
        && pior(cd, &ds, *dom_d) > barra
    {
        return None;
    }
    let lip = majorante_da_inclinacao(&cf, dom_f.0, dom_f.1).max(
        cavidade
            .as_ref()
            .map_or(0.0, |(cd, d)| majorante_da_inclinacao(cd, d.0, d.1)),
    );
    if !lip.is_finite() {
        return None;
    }
    Some(campo_com(
        &cf,
        dom_f,
        cavidade.as_ref().map(|(c, d)| (&c[..], *d)),
        lip,
    ))
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
    /// A inclinação máxima AMOSTRADA — ⚠️ ela erra sempre para BAIXO e não serve de majorante.
    pub inclinacao: f64,
    /// ⭐ O MAJORANTE verdadeiro (`Σ|d_k|`) — é este que o campo usa, e é ele que decide quantos
    /// passos a marcha dá.
    pub majorante: f64,
    pub linhas: usize,
}

/// ⏱️⭐⭐⭐⭐ **A porta da sonda: o vaso por fórmula, grau a grau.**
///
/// ⚠️ `None` quando a régua não acha a peça (a silhueta saiu com menos de metade das alturas) — que
/// é o único jeito de esta tabela mentir sem se ver.
#[doc(hidden)]
#[must_use]
pub fn probe_formula_do_perfil(profile: &Profile, graus: &[usize]) -> Option<Vec<LinhaDaFormula>> {
    let s = silhueta(profile)?;
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
                let majorante = majorante_da_inclinacao(&cf, dom_f.0, dom_f.1)
                    .max(majorante_da_inclinacao(&cd, dom_d.0, dom_d.1));
                LinhaDaFormula {
                    grau,
                    majorante,
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
pub fn probe_silhueta_do_perfil(profile: &Profile) -> Option<Vec<(f64, f64, f64)>> {
    silhueta(profile)
}
