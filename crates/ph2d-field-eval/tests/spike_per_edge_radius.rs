//! ⭐⭐⭐ **SPIKE (2026-08-28): dá para ter um raio POR ARESTA num campo implícito?**
//!
//! Pergunta do Enio: *«avalie a possibilidade de chamfer e fillet por edge e por vertex, em vez do
//! objeto todo»*.
//!
//! ⚠️ **Isto é uma SONDA, não produto.** Nada aqui é chamado pelo módulo; ela existe para pôr
//! números debaixo do relatório, e é `#[ignore]` onde só mede. O que ela responde:
//!
//! 1. a construção por **grupo de arestas** (3 raios numa caixa) é **exacta**? — contra o oráculo
//!    que já é gate desta crate (`sd_box` com raio uniforme, medido a 0,00 %);
//! 2. quanto ela **custa** em nós de árvore contra a caixa de sempre;
//! 3. a construção por **aresta individual** (12 raios) mantém o campo contínuo?
//!
//! ⛔ O que ela **não** responde, e é a parte cara: como o artista **aponta** uma aresta e como o
//! documento a **nomeia** de forma durável. Ver o relatório.

use fidget::context::Tree;
use ph2d_field_eval::Field;

/// A caixa de sempre, com o raio uniforme — o **oráculo**, já medido a `0,00 %` pelos gates da
/// crate.
fn uniform(h: [f64; 3], r: f64) -> Tree {
    ph2d_field_eval::ops::sd_box(h, r)
}

/// **A caixa com um raio por GRUPO de arestas** — `rx` para as 4 paralelas a X, e assim por diante.
///
/// ⭐ A construção é a intersecção de **três barras infinitas de secção arredondada**:
/// - a barra em X limita `|y| ≤ hy, |z| ≤ hz` com os cantos YZ arredondados em `rx` ⇒ ela arredonda
///   exactamente as 4 arestas **paralelas a X**;
/// - idem para Y e Z.
///
/// A intersecção das três é a caixa com os 12 cantos arredondados, cada grupo pelo seu raio.
///
/// ⚠️ **Nada aqui é inventado**: a secção é a `rounded rect` 2D canónica
/// (`length(max(|p| − h + r, 0)) − r`, com o termo interior), que é a mesma forma que o
/// `cylinder_raw` desta crate já usa — exacta dentro e fora.
fn per_edge_group(h: [f64; 3], r: [f64; 3]) -> Tree {
    // Uma barra de secção arredondada nos eixos `(u, v)`, infinita no terceiro.
    let bar = |u: Tree, v: Tree, hu: f64, hv: f64, rr: f64| -> Tree {
        let qu = u.abs() - Tree::constant(hu - rr);
        let qv = v.abs() - Tree::constant(hv - rr);
        let fora = (qu.clone().max(0.0).square() + qv.clone().max(0.0).square())
            .max(1.0e-30)
            .sqrt();
        let dentro = qu.max(qv).min(0.0);
        fora + dentro - Tree::constant(rr)
    };
    let ax = bar(Tree::y(), Tree::z(), h[1], h[2], r[0]);
    let ay = bar(Tree::z(), Tree::x(), h[2], h[0], r[1]);
    let az = bar(Tree::x(), Tree::y(), h[0], h[1], r[2]);
    ax.max(ay).max(az)
}

fn field(t: Tree) -> Field {
    Field::from_tree(&t)
}

/// Uma grelha densa da caixa `[-e, e]³`.
fn grid(e: f64, steps: usize) -> Vec<[f64; 3]> {
    let p = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / steps as f64;
    let mut v = Vec::new();
    for i in 0..steps {
        for j in 0..steps {
            for k in 0..steps {
                v.push([p(i), p(j), p(k)]);
            }
        }
    }
    v
}

/// Onde a superfície cruza um raio que sai da origem na direcção `d`.
fn surface_along(f: &Field, d: [f64; 3]) -> f64 {
    let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let u = [d[0] / n, d[1] / n, d[2] / n];
    let (mut lo, mut hi) = (0.0f64, 3.0f64);
    for _ in 0..90 {
        let m = 0.5 * (lo + hi);
        if f.at(u[0] * m, u[1] * m, u[2] * m) < 0.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    0.5 * (lo + hi)
}

/// ⭐⭐⭐ **ONDE A CONSTRUÇÃO POR GRUPO CONCORDA COM A CAIXA DE SEMPRE, E ONDE NÃO** — e o sítio em
/// que ela não concorda **é a resposta à segunda metade da pergunta do Enio**.
///
/// # ⚠️ Ela NÃO é a caixa arredondada de sempre, e não é um erro
///
/// O `sd_box` arredonda por **deslocamento da superfície** — a soma de Minkowski com uma **bola** —,
/// então os 8 cantos saem **esféricos**. A intersecção de três barras de secção arredondada dá
/// cilindros nas arestas e um canto de **Steinmetz**. As duas superfícies coincidem na face e na
/// aresta, e **divergem no vértice**.
///
/// ⇒ ⭐⭐⭐ **Assim que as arestas ganham raios independentes, o vértice passa a precisar de resposta
/// PRÓPRIA** — que é exactamente o *vertex blend* que os kernels de CAD tratam como operação
/// separada. *A pergunta «por aresta» e a pergunta «por vértice» são a mesma pergunta, e a segunda é
/// consequência da primeira.*
#[test]
fn the_per_group_box_agrees_on_faces_and_edges_and_diverges_at_the_vertex() {
    let h = [0.5, 0.5, 0.5];
    const R: f64 = 0.15;
    let caixa = field(uniform(h, R));
    let grupo = field(per_edge_group(h, [R; 3]));
    println!("  direcção        | sd_box  | por grupo | Δ");
    let medir = |nome: &str, d: [f64; 3]| -> f64 {
        let (a, b) = (surface_along(&caixa, d), surface_along(&grupo, d));
        println!("{nome} | {a:7.5} | {b:9.5} | {:+.5}", b - a);
        b - a
    };
    let na_face = medir("face   (1,0,0)", [1.0, 0.0, 0.0]);
    let na_aresta = medir("aresta (1,1,0)", [1.0, 1.0, 0.0]);
    let no_vertice = medir("vértice(1,1,1)", [1.0, 1.0, 1.0]);
    assert!(
        na_face.abs() < 1.0e-5,
        "a FACE tinha de coincidir: Δ = {na_face:.3e}"
    );
    assert!(
        na_aresta.abs() < 1.0e-5,
        "a ARESTA tinha de coincidir — é o mesmo cilindro de raio {R}: Δ = {na_aresta:.3e}"
    );
    // ⭐ E o VÉRTICE diverge, por construção. A barra é dos dois lados: se ele coincidisse, as duas
    // construções seriam a mesma e este spike não teria assunto.
    assert!(
        no_vertice > 1.0e-3,
        "o VÉRTICE tinha de divergir (Steinmetz contra esfera), e leu Δ = {no_vertice:.3e}"
    );
}

/// ⭐⭐ **E COM RAIOS DIFERENTES ELA DE FACTO ARREDONDA SÓ UM GRUPO.**
///
/// ⚠️ O controlo é o que dá sentido ao gate: numa aresta **paralela a Z** (que pediu raio) o campo
/// muda; numa **paralela a X** (que não pediu) ele tem de ser **igual** ao da caixa viva. Sem a
/// segunda metade, «arredondar tudo» passaria aqui.
///
/// ⚠️ **O sinal é POSITIVO**, e a primeira redacção deste gate tinha-o ao contrário: arredondar uma
/// aresta **convexa** RETIRA material, então um ponto que estava na quina passa a estar **fora**.
#[test]
fn only_the_group_that_asked_gets_rounded() {
    let h = [0.5, 0.5, 0.5];
    let viva = field(uniform(h, 0.0));
    let so_z = field(per_edge_group(h, [0.0, 0.0, 0.2]));

    let na_aresta_z = so_z.at(0.5, 0.5, 0.0) - viva.at(0.5, 0.5, 0.0);
    assert!(
        na_aresta_z > 0.02,
        "a aresta paralela a Z pediu raio e não foi arredondada (Δ = {na_aresta_z:.4})"
    );

    let na_aresta_x = (so_z.at(0.0, 0.5, 0.5) - viva.at(0.0, 0.5, 0.5)).abs();
    assert!(
        na_aresta_x < 1.0e-6,
        "a aresta paralela a X NÃO pediu raio e mudou na mesma (Δ = {na_aresta_x:.3e}) — o raio \
         vazou de um grupo para o outro"
    );
}

/// ⚠️ **O CAMPO CONTINUA A SER UMA DISTÂNCIA** — `‖∇f‖ ≤ 1` nas regiões lisas.
///
/// É o que a marcha consome, e é a pergunta que decide se isto pode entrar no produto ou fica na
/// gaveta: um construtor que suba o gradiente obriga a marcha inteira a andar mais devagar, para
/// toda peça, por causa de uma caixa.
#[test]
fn the_per_group_box_is_still_a_distance() {
    let h = [0.5, 0.35, 0.4];
    let f = field(per_edge_group(h, [0.06, 0.14, 0.22]));
    let mut pior = 0.0f64;
    for p in grid(1.2, 30) {
        let g = f.gradient_norm(p[0], p[1], p[2], 1.0e-4);
        if g.is_finite() {
            pior = pior.max(g);
        }
    }
    println!("  ‖∇f‖ pior da caixa por grupo: {pior:.4}");
    assert!(
        pior < 1.02,
        "a caixa por grupo lê ‖∇f‖ = {pior:.4} — ela obrigaria a marcha inteira a abrandar"
    );
}

/// **O PREÇO, em nós de árvore.** `#[ignore]`: é medição, não afirmação.
#[test]
#[ignore]
fn measure_the_price_of_a_radius_per_edge_group() {
    let h = [0.5, 0.35, 0.4];
    println!("  construção            | nós da árvore");
    for (nome, t) in [
        ("caixa viva", uniform(h, 0.0)),
        ("caixa, raio uniforme", uniform(h, 0.1)),
        (
            "caixa, 3 raios (grupo)",
            per_edge_group(h, [0.06, 0.14, 0.22]),
        ),
    ] {
        let mut ctx = fidget::context::Context::new();
        let _ = ctx.import(&t);
        println!("{nome:>22} | {:13}", ctx.len());
    }
}
