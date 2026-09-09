//! ⭐⭐⭐ **QUE OUTRAS JUNTAS EXISTEM** — a bancada do pedido do Enio de 2026-09-09.
//!
//! > *«Além das junções de tipo Organic, Fillet e Chamfer, faça pesquisa de outros tipos
//! > funcionais, de boa aparência e de boa performance e me relate quais existem»*
//!
//! ⚠️ **Ela é uma SONDA, não um gate:** nada aqui é produto. O que ela produz é a tabela com que a
//! escolha se faz — e sem ela «boa performance» e «boa aparência» seriam duas opiniões.
//!
//! # As QUATRO réguas, e por que são quatro
//!
//! | régua | responde a | quem morre sem ela |
//! |---|---|---|
//! | **recuo** / **mordida** | *que tamanho o artista vê?* | a fileira de caracteres, que tem de medir a mesma coisa |
//! | **`‖∇f‖`** | *a marcha pode andar o valor do campo?* | a peça — um campo que sobe mais que a distância **fura** |
//! | **curvatura** | *o brilho corre liso pela junta?* | «boa aparência», que sem isto é adjectivo |
//! | **nós / ns por ponto** | *quanto custa por avaliação?* | «boa performance», idem |
//!
//! ⭐⭐ **A régua da curvatura é a que faltava a esta casa.** O filete exacto é `G1`: a curvatura
//! salta de `0` na parede para `1/r` no arco, e é esse degrau que uma luz de estúdio desenha como
//! uma banda na peça. Uma junta `G2` não tem o degrau. *Nenhum gate deste módulo mede isso, e é a
//! diferença entre o filete de um CAD e o «blend» dele.*
//!
//! Corre com
//! `cargo test -p ph2d-field-eval --test probe_the_other_junctions -- --ignored --nocapture`.

use fidget::context::Tree;
use ph2d_field_eval::Field;
use std::f64::consts::{FRAC_1_SQRT_2, SQRT_2};

/// O piso da raiz — o mesmo da [`ph2d_field_eval::ops`], para a derivada não ser `NaN` na origem.
const PISO: f64 = 1.0e-30;
/// O tamanho de junta de toda a bancada. ⚠️ Um número só, senão as colunas não se comparam.
const R: f64 = 0.25;

fn k(v: f64) -> Tree {
    Tree::constant(v)
}

fn len2(x: &Tree, y: &Tree) -> Tree {
    (x.square() + y.square()).max(PISO).sqrt()
}

/// ⭐ **A BANCADA: duas paredes num canto CÔNCAVO de 90°** — `a = x`, `b = y`, sólido onde
/// `x ≤ 0` ou `y ≤ 0`.
///
/// ⚠️ **Os dois campos são distâncias exactas** (`‖∇‖ = 1`), então tudo o que a régua do gradiente
/// acusar é do OPERADOR e não das peças. É a mesma bancada do `the_four_characters`.
fn paredes() -> (Tree, Tree) {
    (Tree::x(), Tree::y())
}

// ─────────────────────────── as juntas que a casa JÁ TEM ───────────────────────────

fn arco(a: &Tree, b: &Tree, r: f64) -> Tree {
    let u = (k(r) - a.clone()).max(0.0);
    let v = (k(r) - b.clone()).max(0.0);
    a.min(b.clone()).max(r) - len2(&u, &v)
}

fn corte(a: &Tree, b: &Tree, r: f64) -> Tree {
    a.min(b.clone())
        .min((a.clone() + b.clone() - k(r)) * k(FRAC_1_SQRT_2))
}

/// O `Organic` de hoje — smooth-min POLINOMIAL de grau 2.
fn derretido(a: &Tree, b: &Tree, kk: f64) -> Tree {
    let h = (k(0.5) + k(0.5) * (b.clone() - a.clone()) / k(kk))
        .max(0.0)
        .min(1.0);
    let mixed = b.clone() + (a.clone() - b.clone()) * h.clone();
    mixed - k(kk) * h.clone() * (k(1.0) - h)
}

// ─────────────────────────── a família da PLENITUDE (norma-p) ───────────────────────────

/// ⭐⭐⭐ **UM SÓ NÚMERO QUE CONTÉM O ARCO, O CORTE RETO E TUDO ENTRE ELES.**
///
/// Com `u = (r−a)⁺` e `v = (r−b)⁺`, a superfície é `u^p + v^p = r^p`:
///
/// | `p` | a curva | o que o artista vê |
/// |---|---|---|
/// | `→ ∞` | `max(u,v) = r` | a quina VIVA |
/// | `4` | superelipse magra | arco «apertado» |
/// | `2` | **circunferência** | o `Fillet` de hoje, ao bit |
/// | `1` | **recta** | o `Chamfer` de hoje (mesmo nível, outra normalização) |
/// | `0,5` | astróide | o cordão CHEIO (convexo na união, escavado na intersecção) |
fn norma_p(u: &Tree, v: &Tree, p: f64) -> Tree {
    if (p - 1.0).abs() < 1e-12 {
        return u.clone() + v.clone();
    }
    if (p - 2.0).abs() < 1e-12 {
        return len2(u, v);
    }
    if p >= 64.0 {
        return u.max(v.clone());
    }
    let pot = |t: &Tree| (t.max(PISO).ln() * k(p)).exp();
    ((pot(u) + pot(v)).max(PISO).ln() * k(1.0 / p)).exp()
}

fn plenitude(a: &Tree, b: &Tree, r: f64, p: f64) -> Tree {
    let u = (k(r) - a.clone()).max(0.0);
    let v = (k(r) - b.clone()).max(0.0);
    a.min(b.clone()).max(r) - norma_p(&u, &v, p)
}

/// ⭐⭐⭐ **AS PARADAS BARATAS DA MESMA FAMÍLIA** — os expoentes que se escrevem só com `square` e
/// `sqrt`, sem passar pelo par `exp`/`ln`.
///
/// `p = 2^m` sai de `m` quadrados e `m` raízes; `p = 2^−m` sai das mesmas operações trocadas de
/// lado. ⚠️ **É a diferença entre uma fileira de chips e um slider** — o custo medido do caminho
/// geral está na tabela ao lado.
fn norma_p_barata(u: &Tree, v: &Tree, p: f64) -> Tree {
    let quadrados = |t: &Tree, m: u32| {
        let mut r = t.clone();
        for _ in 0..m {
            r = r.square();
        }
        r
    };
    let raizes = |t: Tree, m: u32| {
        let mut r = t;
        for _ in 0..m {
            r = r.max(PISO).sqrt();
        }
        r
    };
    if p >= 2.0 {
        let m = p.log2().round() as u32;
        raizes(quadrados(u, m) + quadrados(v, m), m)
    } else {
        // `p = 2^−m`: a soma das raízes, elevada de volta.
        let m = (-p.log2()).round() as u32;
        quadrados(&(raizes(u.clone(), m) + raizes(v.clone(), m)), m)
    }
}

fn plenitude_barata(a: &Tree, b: &Tree, r: f64, p: f64) -> Tree {
    let u = (k(r) - a.clone()).max(0.0);
    let v = (k(r) - b.clone()).max(0.0);
    a.min(b.clone()).max(r) - norma_p_barata(&u, &v, p)
}

/// ⭐⭐⭐ **A JUNTA CÓNICA — o `Rho` do CAD**, e a única parametrização CONTÍNUA que não paga
/// transcendentais.
///
/// A cónica tangente às duas faces nos mesmos pontos do filete é `a·b = λ·(a + b − r)²` — **um
/// produto e um quadrado**. Com `ρ` a razão clássica (a distância do ombro à corda sobre a distância
/// da quina à corda), `λ = (1−ρ)²/(4ρ²)` e:
///
/// | `ρ` | a cónica | o que se vê |
/// |---|---|---|
/// | `→ 0` | a corda | o **Chamfer** |
/// | `0,4142` = `√2−1` | a **circunferência** | o **Fillet**, exacto |
/// | `0,5` | parábola | o «cheio» clássico de superfície classe-A |
/// | `→ 1` | a quina | **viva** |
///
/// ⚠️ **O campo cru é uma quadrática, não uma distância** — daí o divisor `L`, que é o majorante de
/// `‖∇F‖` medido nos dois extremos da região (a tangência e o ombro). Na circunferência ele vale
/// exactamente `r`, e é por isso que ali o campo volta a ser exacto.
fn conica(a: &Tree, b: &Tree, r: f64, rho: f64) -> Tree {
    let lam = (1.0 - rho) * (1.0 - rho) / (4.0 * rho * rho);
    let quadratica = a.clone() * b.clone() - (a.clone() + b.clone() - k(r)).square() * k(lam);
    // ⚠️ `‖∇F‖ = ‖(b − 2λ(a+b−r), a − 2λ(a+b−r))‖`, e sobre a caixa `[0, r]²` o pior está num
    // CANTO: `√2·r·max(2λ, |1 − 2λ|)`, nunca abaixo de `r` (que é o valor na tangência).
    // ⛔ A 1.ª redacção estimava-o em DOIS pontos e lia `1,91×` de gradiente na circunferência —
    // *um majorante amostrado erra sempre para baixo* (memória `a_sampled_maximum…`).
    let l = r.max(SQRT_2 * r * (2.0 * lam).max((1.0 - 2.0 * lam).abs()));
    // ⛔⛔ **O RECORTE DESTA JUNTA NÃO TEM SAÍDA BOA, e as duas foram MEDIDAS.**
    //
    // | recorte | forma | `‖∇f‖` |
    // |---|---|---|
    // | só o tecto, `max(a−r, b−r)` | **certa** | `1,70`–`1,96` |
    // | a caixa inteira, com `−a` e `−b` | ⛔ **parte a junta da parede** | `1,0000` |
    //
    // A 2.ª fecha o gradiente e **apaga o material que a junta acrescenta encostado à face** — em
    // `a = 0` o `max` levanta o campo a `0`, e a tangência deixa de existir. ⭐ *O material de um
    // filete TOCA as duas faces; um recorte que exclui `a < 0` exclui também `a = 0`.*
    //
    // ⇒ fica a 1.ª, e o `1,70` é o preço registado — ver o §145.3 do doc 06 para porque isso é
    // pior do que parece (o passo da marcha é do DOCUMENTO, logo uma junta destas atrasa a cena
    // inteira).
    let caixa = (a.clone() - k(r)).max(b.clone() - k(r));
    a.min(b.clone()).min((quadratica * k(1.0 / l)).max(caixa))
}

// ─────────────────────────── as SUAVES ───────────────────────────

/// ⭐⭐ **O smooth-min CÚBICO** — a mesma família do `Organic`, um grau acima: `G2` em vez de `G1`.
fn derretido_g2(a: &Tree, b: &Tree, kk: f64) -> Tree {
    let h = (k(kk) - (a.clone() - b.clone()).abs()).max(0.0) / k(kk);
    a.min(b.clone()) - h.square() * h * k(kk / 6.0)
}

// ─────────────────────────── as ASSIMÉTRICAS ───────────────────────────

/// ⭐ **O chanfro de DOIS recuos** — `ca` numa face, `cb` na outra. O *two-distance chamfer* do CAD.
fn corte_assimetrico(a: &Tree, b: &Tree, ca: f64, cb: f64) -> Tree {
    let norma = (1.0 / (ca * ca) + 1.0 / (cb * cb)).sqrt();
    let plano = (a.clone() * k(1.0 / ca) + b.clone() * k(1.0 / cb) - k(1.0)) * k(1.0 / norma);
    a.min(b.clone()).min(plano)
}

/// ⭐ **O filete ELÍPTICO** — raio `ra` numa face e `rb` na outra; o arco de sempre num espaço
/// esticado, com o valor reposto pelo MENOR dos dois raios (que o mantém minorante).
fn arco_eliptico(a: &Tree, b: &Tree, ra: f64, rb: f64) -> Tree {
    let (na, nb) = (a.clone() * k(1.0 / ra), b.clone() * k(1.0 / rb));
    let u = (k(1.0) - na.clone()).max(0.0);
    let v = (k(1.0) - nb.clone()).max(0.0);
    (na.min(nb).max(1.0) - len2(&u, &v)) * k(ra.min(rb))
}

// ─────────────────────────── as DECORAÇÕES DA COSTURA ───────────────────────────
//
// ⭐⭐⭐ Todas partem do MESMO par de coordenadas, e é isso que as torna possíveis sem ninguém
// saber onde a costura está: `d = min(a,b)` é a superfície da união, e `s = (a−b)/√2` é a distância
// com sinal ao plano bissector — ou seja, **ao longo de quê** e **a que distância da costura**.

fn costura(a: &Tree, b: &Tree) -> (Tree, Tree) {
    (a.min(b.clone()), (a.clone() - b.clone()) * k(FRAC_1_SQRT_2))
}

/// **CORDÃO DE SOLDA** — um tubo de raio `r` a correr sobre a costura.
fn cordao(a: &Tree, b: &Tree, r: f64) -> Tree {
    a.min(b.clone()).min(len2(a, b) - k(r))
}

/// **SULCO** — um canal de profundidade `prof` e meia-largura `larg` escavado sobre a costura.
fn sulco(a: &Tree, b: &Tree, prof: f64, larg: f64) -> Tree {
    let (d, s) = costura(a, b);
    d.clone().max((d + k(prof)).min(k(larg) - s.abs()))
}

/// **FRISO** — o oposto: uma nervura de altura `alt` e meia-largura `larg` sobre a costura.
fn friso(a: &Tree, b: &Tree, alt: f64, larg: f64) -> Tree {
    let (d, s) = costura(a, b);
    d.clone().min((d - k(alt)).max(s.abs() - k(larg)))
}

/// **GRAVAÇÃO EM V** — uma incisão de profundidade `prof`, a 45°.
fn gravado(a: &Tree, b: &Tree, prof: f64) -> Tree {
    let (d, s) = costura(a, b);
    d.clone().max((d + k(prof) - s.abs()) * k(FRAC_1_SQRT_2))
}

/// **ESCADA** — a transição em `n` degraus (o `fOpUnionStairs` da família hg_sdf).
fn escada(a: &Tree, b: &Tree, r: f64, n: f64) -> Tree {
    let s = r / n;
    let u = b.clone() - k(r);
    let dente = ((u.clone() - a.clone() + k(s)).modulo(k(2.0 * s)) - k(s)).abs();
    a.min(b.clone()).min((u + a.clone() + dente) * k(0.5))
}

/// **COLUNATA** — `n` colunas redondas na diagonal da junta. ⚠️ A repetição é INFINITA, então ela
/// só existe recortada à caixa `a < r ∧ b < r`; fora dela a junta é a viva.
fn colunata(a: &Tree, b: &Tree, r: f64, n: f64) -> Tree {
    let raio = r * SQRT_2 / ((n - 1.0) * 2.0 + SQRT_2);
    // Roda 45°: o eixo da colunata é a bissectriz.
    let (d, s) = costura(a, b);
    let along = s;
    let out = (a.clone() + b.clone()) * k(FRAC_1_SQRT_2) - k(r * FRAC_1_SQRT_2) + k(raio * SQRT_2);
    let along = if (n % 2.0 - 1.0).abs() < 1e-9 {
        along + k(raio)
    } else {
        along
    };
    let cel = along.modulo(k(2.0 * raio)) - k(raio);
    let colunas = len2(&out, &cel) - k(raio);
    let dentro = a.max(b.clone()) - k(r);
    // Fora da caixa da junta a colunata não existe: some-a ao campo por um `max` com a caixa.
    let recortada = colunas.max(dentro.clone()).min(out.max(dentro));
    d.min(recortada)
}

// ─────────────────────────── as RÉGUAS ───────────────────────────

/// **O RECUO ao longo da face** — até onde a junta sobe a parede `x = 0`.
fn recuo(f: &Field) -> f64 {
    let superficie_x = |y: f64| {
        let (mut lo, mut hi) = (-1.0f64, 2.0f64);
        for _ in 0..70 {
            let m = f64::midpoint(lo, hi);
            if f.at(m, y, 0.0) < 0.0 {
                lo = m;
            } else {
                hi = m;
            }
        }
        f64::midpoint(lo, hi)
    };
    let (mut lo, mut hi) = (0.0f64, 3.0f64);
    for _ in 0..60 {
        let m = f64::midpoint(lo, hi);
        if superficie_x(m) > 1.0e-4 {
            lo = m;
        } else {
            hi = m;
        }
    }
    f64::midpoint(lo, hi)
}

/// **A MORDIDA** — onde a superfície cruza a diagonal, medida da quina.
fn mordida(f: &Field) -> f64 {
    let (mut lo, mut hi) = (-0.5f64, 2.0f64);
    for _ in 0..70 {
        let m = f64::midpoint(lo, hi);
        if f.at(m, m, 0.0) < 0.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    f64::midpoint(lo, hi) * SQRT_2
}

/// O pior `‖∇f‖` numa caixa à volta da junta. ⚠️ **É o recíproco do passo da marcha.**
fn pior_gradiente(f: &Field) -> f64 {
    let mut pior = 0.0f64;
    const N: usize = 41;
    for i in 0..N {
        for j in 0..N {
            let c = |t: usize| -0.6 + 1.2 * (t as f64 + 0.5) / N as f64;
            let g = f.gradient_norm(c(i), c(j), 0.0, 1.0e-4);
            if g.is_finite() {
                pior = pior.max(g);
            }
        }
    }
    pior
}

/// ⭐⭐⭐ **A CURVATURA DO PERFIL, e o SALTO dela** — a régua de «boa aparência».
///
/// O perfil inteiro (parede · junta · parede) é um gráfico `n(s)` nas coordenadas rodadas 45°:
/// `s = (x−y)/√2` corre ao longo e `n = (x+y)/√2` sai da quina. Um filete exacto tem `κ = 0` nas
/// duas paredes e `κ = 1/r` no arco — o **degrau** que a luz desenha.
fn curvatura(f: &Field) -> (f64, f64) {
    const PASSO: f64 = 0.01;
    let altura = |s: f64| {
        let (mut lo, mut hi) = (-0.9f64, 1.2f64);
        for _ in 0..60 {
            let m = f64::midpoint(lo, hi);
            let (x, y) = ((m + s) * FRAC_1_SQRT_2, (m - s) * FRAC_1_SQRT_2);
            if f.at(x, y, 0.0) < 0.0 {
                lo = m;
            } else {
                hi = m;
            }
        }
        f64::midpoint(lo, hi)
    };
    let n: Vec<f64> = (0..=120)
        .map(|i| altura(-0.6 + PASSO * f64::from(i)))
        .collect();
    let mut kappa = Vec::new();
    for w in n.windows(3) {
        let d1 = (w[2] - w[0]) / (2.0 * PASSO);
        let d2 = (w[2] - 2.0 * w[1] + w[0]) / (PASSO * PASSO);
        kappa.push(d2.abs() / (1.0 + d1 * d1).powf(1.5));
    }
    let maxima = kappa.iter().copied().fold(0.0f64, f64::max);
    let salto = kappa
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0f64, f64::max);
    (maxima, salto)
}

/// Os nós da árvore e os nanossegundos por ponto avaliado.
fn custo(tree: &Tree) -> (usize, f64) {
    use fidget::shape::EzShape;
    const N: usize = 1 << 17;
    let coord =
        |i: usize, c: usize| -0.9 + 1.8 * (((i * 7919 + c * 104_729) % 1024) as f32) / 1024.0;
    let xs: Vec<f32> = (0..N).map(|i| coord(i, 0)).collect();
    let ys: Vec<f32> = (0..N).map(|i| coord(i, 1)).collect();
    let zs: Vec<f32> = (0..N).map(|i| coord(i, 2)).collect();

    let mut ctx = fidget::context::Context::new();
    let _ = ctx.import(tree);
    let nos = ctx.len();

    let shape = ph2d_field_eval::Engine::from(tree.clone());
    let tape = shape.ez_float_slice_tape();
    let mut eval = ph2d_field_eval::Engine::new_float_slice_eval();
    let _ = eval.eval(&tape, &xs, &ys, &zs).expect("avalia");
    let mut a: Vec<f64> = (0..7)
        .map(|_| {
            let t0 = std::time::Instant::now();
            let _ = eval.eval(&tape, &xs, &ys, &zs).expect("avalia");
            t0.elapsed().as_secs_f64() * 1.0e9 / N as f64
        })
        .collect();
    a.sort_by(f64::total_cmp);
    (nos, a[3])
}

/// Uma secção `z = 0` em ASCII — a única régua que mostra a FORMA em vez de a resumir.
fn seccao(f: &Field, nome: &str) {
    const COLS: usize = 49;
    const LINHAS: usize = 25;
    const EXT: f64 = 0.45;
    println!("  ── {nome} ──");
    for j in 0..LINHAS {
        let y = EXT - 2.0 * EXT * (j as f64 + 0.5) / LINHAS as f64;
        let linha: String = (0..COLS)
            .map(|i| {
                let x = -EXT + 2.0 * EXT * (i as f64 + 0.5) / COLS as f64;
                if f.at(x, y, 0.0) < 0.0 { '#' } else { '.' }
            })
            .collect();
        println!("  {linha}");
    }
}

fn cabecalho() {
    println!(
        "  {:<26} | {:>6} | {:>7} | {:>7} | {:>7} | {:>6} | {:>4} | {:>7}",
        "junta", "recuo", "mordida", "‖∇f‖", "κ máx", "salto", "nós", "ns/pto"
    );
    println!("  {}", "-".repeat(96));
}

fn linha(nome: &str, tree: &Tree) {
    let f = Field::from_tree(tree);
    let (kmax, salto) = curvatura(&f);
    let (nos, ns) = custo(tree);
    println!(
        "  {:<26} | {:6.4} | {:7.4} | {:7.4} | {:7.3} | {:6.3} | {nos:4} | {ns:6.2}",
        nome,
        recuo(&f),
        mordida(&f),
        pior_gradiente(&f),
        kmax,
        salto
    );
}

// ─────────────────────────── as SONDAS ───────────────────────────

#[test]
#[ignore = "sonda: um só número contém o arco, o corte reto e tudo entre eles?"]
fn probe_the_fullness_axis() {
    let (a, b) = paredes();
    println!("\n⭐ A FAMÍLIA DA PLENITUDE — a mesma fórmula, o expoente a variar (r = {R})\n");
    cabecalho();
    linha("Fillet (a casa)", &arco(&a, &b, R));
    linha("Chamfer (a casa)", &corte(&a, &b, R));
    for p in [0.5, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0, 8.0, 64.0] {
        linha(&format!("plenitude p = {p}"), &plenitude(&a, &b, R, p));
    }
    println!("\n  ⭐ as MESMAS formas pelas paradas baratas (só `square` e `sqrt`):\n");
    cabecalho();
    for p in [0.25, 0.5, 2.0, 4.0, 8.0, 16.0] {
        linha(&format!("barata p = {p}"), &plenitude_barata(&a, &b, R, p));
    }
    println!("\n  ⭐ a CÓNICA — contínua em `ρ`, sem transcendental nenhum:\n");
    cabecalho();
    for rho in [0.15, 0.3, 0.414_213_562, 0.5, 0.65, 0.85] {
        linha(&format!("cónica ρ = {rho:.3}"), &conica(&a, &b, R, rho));
    }
    println!(
        "\n  ⚠️ `p = 2` tem de bater o `Fillet` e `p = 1` o `Chamfer` — se não bater, a família\n     não contém os dois e a leitura acaba aqui.\n"
    );
    for p in [0.5, 1.0, 2.0, 4.0, 64.0] {
        seccao(
            &Field::from_tree(&plenitude(&a, &b, R, p)),
            &format!("plenitude p = {p}"),
        );
    }
    for rho in [0.15, 0.414_213_562, 0.65] {
        seccao(
            &Field::from_tree(&conica(&a, &b, R, rho)),
            &format!("cónica ρ = {rho:.3}"),
        );
    }
}

#[test]
#[ignore = "sonda: as decorações da costura, que nenhuma composição exprime"]
fn probe_the_seam_decorations() {
    let (a, b) = paredes();
    println!("\n⭐ AS DECORAÇÕES DA COSTURA — elas ACHAM a costura sem ninguém a calcular\n");
    cabecalho();
    linha("cordão de solda", &cordao(&a, &b, R * 0.7));
    linha("sulco (panel line)", &sulco(&a, &b, R * 0.5, R * 0.35));
    linha("friso (nervura)", &friso(&a, &b, R * 0.5, R * 0.35));
    linha("gravação em V", &gravado(&a, &b, R * 0.6));
    linha("escada n = 3", &escada(&a, &b, R, 3.0));
    linha("escada n = 6", &escada(&a, &b, R, 6.0));
    linha("colunata n = 3", &colunata(&a, &b, R, 3.0));
    println!();
    seccao(&Field::from_tree(&cordao(&a, &b, R * 0.7)), "cordão");
    seccao(
        &Field::from_tree(&sulco(&a, &b, R * 0.5, R * 0.35)),
        "sulco",
    );
    seccao(
        &Field::from_tree(&friso(&a, &b, R * 0.5, R * 0.35)),
        "friso",
    );
    seccao(&Field::from_tree(&gravado(&a, &b, R * 0.6)), "gravado");
    seccao(&Field::from_tree(&escada(&a, &b, R, 3.0)), "escada n=3");
    seccao(&Field::from_tree(&colunata(&a, &b, R, 3.0)), "colunata n=3");
}

#[test]
#[ignore = "sonda: as assimétricas e as suaves de segunda ordem"]
fn probe_the_asymmetric_and_the_smooth() {
    let (a, b) = paredes();
    println!("\n⭐ ASSIMÉTRICAS E SUAVES — o que muda quando as duas faces não são iguais\n");
    cabecalho();
    linha("Fillet (referência)", &arco(&a, &b, R));
    linha(
        "Organic G1 (a casa)",
        &derretido(&a, &b, R * (4.0 - 2.0 * SQRT_2)),
    );
    linha("Organic G2 (cúbico)", &derretido_g2(&a, &b, R * 1.6));
    linha(
        "chanfro 2 recuos 1:3",
        &corte_assimetrico(&a, &b, R, R * 3.0),
    );
    linha("filete elíptico 1:3", &arco_eliptico(&a, &b, R, R * 3.0));
    println!();
    seccao(
        &Field::from_tree(&derretido(&a, &b, R * (4.0 - 2.0 * SQRT_2))),
        "Organic G1",
    );
    seccao(
        &Field::from_tree(&derretido_g2(&a, &b, R * 1.6)),
        "Organic G2",
    );
    seccao(
        &Field::from_tree(&corte_assimetrico(&a, &b, R, R * 3.0)),
        "chanfro 1:3",
    );
    seccao(
        &Field::from_tree(&arco_eliptico(&a, &b, R, R * 3.0)),
        "elíptico 1:3",
    );
}

/// ⭐⭐⭐ A pergunta do §5.0: **a composição já exprime isto?** — a costura de uma esfera com uma
/// chapa é um CÍRCULO, e o sulco tem de o seguir sem ninguém o calcular.
#[test]
#[ignore = "sonda: a decoração segue uma costura CURVA sem ninguém a conhecer"]
fn probe_the_seam_is_found_not_given() {
    let esfera = len2(&len2(&Tree::x(), &Tree::y()), &Tree::z()) - k(0.42);
    let chapa = Tree::z().abs() - k(0.18);
    let f = Field::from_tree(&sulco(&esfera, &chapa, 0.10, 0.07));
    println!("\n⭐ A COSTURA CURVA — esfera ∪ chapa, com um sulco a segui-la (secção x–z)\n");
    const COLS: usize = 61;
    const LINHAS: usize = 25;
    for j in 0..LINHAS {
        let z = 0.6 - 1.2 * (j as f64 + 0.5) / LINHAS as f64;
        let linha: String = (0..COLS)
            .map(|i| {
                let x = -0.75 + 1.5 * (i as f64 + 0.5) / COLS as f64;
                if f.at(x, 0.0, z) < 0.0 { '#' } else { '.' }
            })
            .collect();
        println!("  {linha}");
    }
    let (nos, ns) = custo(&sulco(&esfera, &chapa, 0.10, 0.07));
    println!("\n  sulco sobre costura curva: {nos} nós, {ns:.2} ns/ponto");
}
