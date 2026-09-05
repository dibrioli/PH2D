//! **SONDA — até onde a deformação do tecido CHEGA, e o que ela deixa na frente.**
//!
//! ⚠️ Não é um gate: ela imprime a tabela. A pergunta que ela responde é a que
//! nenhuma régua do tecido faz — *o perfil do deslocamento* — porque todas
//! medem o MÁXIMO, e o máximo é o vértice que a mão conduz diretamente.

use ph2d_mesh::{Face, Mesh};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// Grade plana de `n × n` células sobre `[-1, 1]`.
fn plano(n: usize) -> Mesh {
    let s = 2.0 / n as f32;
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            pos.push([i as f32 * s - 1.0, j as f32 * s - 1.0, 0.0]);
        }
    }
    let id = |i: usize, j: usize| u32::try_from(j * (n + 1) + i).unwrap();
    let mut faces = Vec::new();
    for j in 0..n {
        for i in 0..n {
            faces.push(Face::quad(
                id(i, j),
                id(i + 1, j),
                id(i + 1, j + 1),
                id(i, j + 1),
            ));
        }
    }
    Mesh::from_parts(pos, faces).expect("grade")
}

fn desloc(a: &Mesh, b: &Mesh, v: usize) -> f32 {
    let (p, q) = (a.positions()[v], b.positions()[v]);
    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
}

/// `(células da grade, eventos, passo por evento)` — o PERCURSO é sempre `0,68`.
/// `(células da grade, eventos, passo por evento)`.
///
/// ⚠️ **Os eventos são 35 em TODAS as células, de propósito:** assim o número de
/// varreduras do solver é constante e a única coisa que varia é o passo da mão
/// medido em ARESTAS. ⛔ As células com 70 eventos foram tentadas e **descartadas
/// por contaminação** — a `0,02 × 70` o pincel sai da grade de `[-1, 1]` e a
/// banda medida deixa de estar sob o dedo (as colunas do perfil liam `0,00`).
const CELULAS: [(usize, usize, f32); 7] = [
    (24, 35, 0.02),
    (96, 35, 0.02),
    (144, 35, 0.02),
    (144, 35, 0.01),
    (192, 35, 0.02),
    (192, 35, 0.01),
    (192, 35, 0.005),
];

#[test]
#[ignore = "sonda"]
fn sonda_da_frente_do_tecido() {
    println!(
        "{:>4} {:>6} {:>6} {:>7} | {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} | {:>7} {:>6} {:>8} {:>5}",
        "n",
        "passo",
        "vert",
        "max/R",
        "0.25",
        "0.50",
        "0.75",
        "1.00",
        "1.25",
        "1.50",
        "1.75",
        "2.00",
        "degrau",
        "onde",
        "residuo",
        "fora"
    );
    for (n, eventos, passo_ev) in CELULAS {
        let antes = plano(n);
        let mut mesh = antes.clone();
        let brush = Brush {
            verb: Verb::Cloth,
            radius: 0.30,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        // O MESMO percurso em todas as células: só muda em quantos eventos ele é
        // entregue, que e' o que muda o passo por evento em ARESTAS.
        for k in 0..eventos {
            let c = [passo_ev * k as f32, 0.0, 0.0];
            let passo = if k == 0 {
                [0.0; 3]
            } else {
                [passo_ev, 0.0, 0.0]
            };
            s.dab(
                &mut mesh,
                &brush,
                &Dab::hooking(c, brush.radius, [0.0, 0.0, -1.0], passo),
                Symmetry::default(),
            );
        }
        // O perfil é medido em torno do FIM do traço, que é onde a mão está.
        let fim = [passo_ev * (eventos - 1) as f32, 0.0f32];
        let r = brush.radius;
        const ALVOS: [f32; 8] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
        let mut max = 0.0f32;
        let mut banda = [0.0f32; 8];
        for v in 0..antes.vert_count() {
            let p = antes.positions()[v];
            let d = desloc(&antes, &mesh, v);
            max = max.max(d);
            let dist = ((p[0] - fim[0]).powi(2) + (p[1] - fim[1]).powi(2)).sqrt() / r;
            for (b, alvo) in ALVOS.iter().enumerate() {
                if (dist - alvo).abs() < 0.13 {
                    banda[b] = banda[b].max(d);
                }
            }
        }
        // **O DEGRAU** — o maior salto de deslocamento entre vizinhos da grade,
        // em fração do maior deslocamento. Um arrasto rígido com frente abrupta
        // dá um degrau grande; um pano dá um perfil suave.
        let id = |i: usize, j: usize| j * (n + 1) + i;
        let mut degrau = 0.0f32;
        let mut onde = 0.0f32;
        let par = |a: usize, b: usize, degrau: &mut f32, onde: &mut f32| {
            let salto = (desloc(&antes, &mesh, a) - desloc(&antes, &mesh, b)).abs();
            if salto > *degrau {
                *degrau = salto;
                let p = antes.positions()[a];
                *onde = ((p[0] - fim[0]).powi(2) + (p[1] - fim[1]).powi(2)).sqrt() / r;
            }
        };
        for j in 0..=n {
            for i in 0..n {
                par(id(i, j), id(i + 1, j), &mut degrau, &mut onde);
            }
        }
        for j in 0..n {
            for i in 0..=n {
                par(id(i, j), id(i, j + 1), &mut degrau, &mut onde);
            }
        }
        // ⭐ **A RÉGUA LOCAL** — o resíduo de cada vértice contra a MÉDIA dos
        // quatro vizinhos da grade, em fração do maior deslocamento. Um perfil
        // liso, por mais inclinado que seja, dá resíduo ~0; um vértice que voou
        // sozinho dá resíduo grande. É a grandeza que uma mediana não vê.
        let mut pior_res = 0.0f32;
        let mut fora = 0usize;
        let piso = ((2.0 / n as f32) / r).powi(2);
        for j in 1..n {
            for i in 1..n {
                let d = desloc(&antes, &mesh, id(i, j));
                let viz = [id(i - 1, j), id(i + 1, j), id(i, j - 1), id(i, j + 1)];
                let m: f32 = viz.iter().map(|v| desloc(&antes, &mesh, *v)).sum::<f32>() / 4.0;
                // ⭐⭐ **A NORMALIZAÇÃO É O CHÃO DA DISCRETIZAÇÃO, e sem ela a
                // régua acusa a malha grossa.** Num perfil LISO o resíduo de
                // um vértice contra a média dos vizinhos vale `~(h²/4)·∇²f`,
                // logo ele cai com `(h/R)²` — uma barra fixa leria `0,093` numa
                // grade de 24 e `0,002` numa de 144 sobre o MESMO produto.
                // Dividindo por `(h/R)²` a grandeza fica adimensional e
                // comparável entre densidades.
                let res = (d - m).abs() / max.max(1e-9) / piso;
                pior_res = pior_res.max(res);
                if res > 20.0 {
                    fora += 1;
                }
            }
        }
        let aresta = 2.0 / n as f32;
        let f = |x: f32| x / max.max(1e-9);
        println!(
            "{:>4} {:>6.1} {:>6} {:>7.3} | {:>5.2} {:>5.2} {:>5.2} {:>5.2} {:>5.2} {:>5.2} {:>5.2} {:>5.2} | {:>7.3} {:>6.2} {:>8.1} {:>5}",
            n,
            passo_ev / aresta,
            antes.vert_count(),
            max / r,
            f(banda[0]),
            f(banda[1]),
            f(banda[2]),
            f(banda[3]),
            f(banda[4]),
            f(banda[5]),
            f(banda[6]),
            f(banda[7]),
            f(degrau),
            onde,
            pior_res,
            fora
        );
    }
}
