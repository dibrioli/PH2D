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

/// **A RÉGUA DA PREGA** — a onda do deslocamento NORMAL, medida em arestas.
///
/// ⚠️ **Ela existe porque nenhuma das 33 réguas desta linha mede uma prega.** As
/// três do gate (`espinho`/`rasgo`/`estica`) são escalares de MAGNITUDE; uma
/// prega é uma **oscilação da componente fora do plano**, e a lei que lhe dá
/// barra é publicada: `λ ~ 2π(B/K)^{1/4}` (Cerda & Mahadevan, PRL 90 074302).
///
/// Devolve `(relevo, ondula, lambda)`, todos em **arestas**:
/// - `relevo` = RMS de `u·n⁰` na banda — o quanto a superfície LEVANTOU;
/// - `ondula` = RMS da parte que **oscila** (o resíduo contra a média do anel-1)
///   — é isto que é prega, e não o relevo;
/// - `lambda` = o comprimento de onda estimado. Para uma senóide de onda `λ`
///   amostrada com aresta `h`, a fração de arestas que **cruzam zero** é `≈2h/λ`
///   ⇒ `λ/h ≈ 2/fração`. ⭐ É um estimador sem FFT, sem ordenação e sem
///   parametrização — só precisa do sinal em cada ponta de aresta.
fn prega(antes: &Mesh, depois: &Mesh, centro: [f32; 3], r: f32) -> (f32, f32, f32) {
    let n0 = antes.normals();
    let (p0, p1) = (antes.positions(), depois.positions());
    let adj = antes.adjacency();
    // A banda LIVRE: entre o bordo da pegada e o anel pregado.
    let na_banda = |v: usize| {
        let p = p0[v];
        let d =
            ((p[0] - centro[0]).powi(2) + (p[1] - centro[1]).powi(2) + (p[2] - centro[2]).powi(2))
                .sqrt();
        d > r && d < 2.0 * r
    };
    // `s` = o deslocamento ao longo da normal de REPOUSO.
    let s: Vec<f32> = (0..antes.vert_count())
        .map(|v| {
            let (a, b, n) = (p0[v], p1[v], n0[v]);
            (b[0] - a[0]) * n[0] + (b[1] - a[1]) * n[1] + (b[2] - a[2]) * n[2]
        })
        .collect();
    // A parte que OSCILA: o resíduo contra a média do anel-1. ⚠️ Sem isto a
    // régua leria a bossa que o pincel faz (relevo) como se fosse prega.
    let osc: Vec<f32> = (0..antes.vert_count())
        .map(|v| {
            let viz = adj.vert_verts.neighbours(v);
            if viz.is_empty() {
                return 0.0;
            }
            s[v] - viz.iter().map(|n| s[*n as usize]).sum::<f32>() / viz.len() as f32
        })
        .collect();

    let mut h = (0.0f32, 0usize);
    let (mut ss, mut so, mut n) = (0.0f64, 0.0f64, 0usize);
    for v in 0..antes.vert_count() {
        if !na_banda(v) {
            continue;
        }
        ss += f64::from(s[v] * s[v]);
        so += f64::from(osc[v] * osc[v]);
        n += 1;
        for w in adj.vert_verts.neighbours(v) {
            let q = p0[*w as usize];
            let e =
                ((p0[v][0] - q[0]).powi(2) + (p0[v][1] - q[1]).powi(2) + (p0[v][2] - q[2]).powi(2))
                    .sqrt();
            h = (h.0 + e, h.1 + 1);
        }
    }
    if n == 0 || h.1 == 0 {
        return (0.0, 0.0, 0.0);
    }
    let aresta = h.0 / h.1 as f32;
    let (mut cruza, mut arestas) = (0usize, 0usize);
    for v in 0..antes.vert_count() {
        if !na_banda(v) {
            continue;
        }
        for w in adj.vert_verts.neighbours(v) {
            let u = *w as usize;
            if u <= v || !na_banda(u) {
                continue;
            }
            arestas += 1;
            if osc[v] * osc[u] < 0.0 {
                cruza += 1;
            }
        }
    }
    let frac = if arestas == 0 {
        0.0
    } else {
        cruza as f32 / arestas as f32
    };
    let lambda = if frac > 1e-6 {
        2.0 / frac
    } else {
        f32::INFINITY
    };
    (
        (ss / n as f64).sqrt() as f32 / aresta,
        (so / n as f64).sqrt() as f32 / aresta,
        lambda,
    )
}

/// ⭐⭐⭐ **SONDA — a previsão de Cerda, testada antes de escrever produto.**
///
/// A lei `λ ~ 2π(B/K)^{1/4}` com o `K` do NOSSO gesto (a mola do `ClothDrive` é
/// uma fundação de Winkler, e ela é isotrópica ⇒ penaliza também a direção
/// normal, que é exatamente o que uma prega é) prevê `λ = 1,4`–`3,1` arestas
/// dentro da pegada: **abaixo do piso de Nyquist da malha**. A prega não existe
/// no espaço discreto.
///
/// **Previsão falsificável:** subir `bending` `44×` (ou baixar `CLOTH_GRIP` `44×`)
/// leva `λ` para `~8` arestas ⇒ **`ondula` tem de subir e `lambda` tem de crescer**.
/// Se não subir, a lei de Cerda não descreve este sistema e a fila inteira de
/// curas está ordenada sobre a coisa errada.
#[test]
#[ignore = "sonda"]
fn sonda_da_prega() {
    let antes = ph2d_mesh::shapes::uv_sphere(128, 192, 1.0);
    let mut mesh = antes.clone();
    let brush = Brush {
        verb: Verb::Cloth,
        radius: 0.30,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    let d = 0.03;
    let mut fim = [0.0f32; 3];
    for k in 0..35 {
        let x = -0.5 + d * k as f32;
        let z = (1.0 - x * x).max(0.0).sqrt();
        fim = [x, 0.0, z];
        let passo = if k == 0 { [0.0; 3] } else { [d, 0.0, 0.0] };
        s.dab(
            &mut mesh,
            &brush,
            &Dab::hooking(fim, brush.radius, [0.0, 0.0, -1.0], passo),
            Symmetry::default(),
        );
    }
    let (relevo, ondula, lambda) = prega(&antes, &mesh, fim, brush.radius);
    // ⭐ **AGULHA ou PREGA?** A mesma régua local do gate, mas com a POPULAÇÃO ao
    // lado do máximo: um vértice sozinho acima da barra é agulha; dezenas
    // espalhadas pelo traço são a estrutura da dobra. *«1 de 5 está mau» não diz
    // QUAL nem QUANTOS.*
    let adj = antes.adjacency();
    let (p0, p1) = (antes.positions(), mesh.positions());
    let d = |v: usize| {
        let (a, b) = (p0[v], p1[v]);
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    let max = (0..antes.vert_count()).map(d).fold(0.0f32, f32::max);
    let mut res: Vec<f32> = Vec::new();
    for v in 0..antes.vert_count() {
        let viz = adj.vert_verts.neighbours(v);
        if viz.len() < 3 {
            continue;
        }
        let h: f32 = viz
            .iter()
            .map(|w| {
                let q = p0[*w as usize];
                ((p0[v][0] - q[0]).powi(2) + (p0[v][1] - q[1]).powi(2) + (p0[v][2] - q[2]).powi(2))
                    .sqrt()
            })
            .sum::<f32>()
            / viz.len() as f32;
        let piso = (h / brush.radius).powi(2);
        let m = viz.iter().map(|w| d(*w as usize)).sum::<f32>() / viz.len() as f32;
        res.push((d(v) - m).abs() / max.max(1e-9) / piso);
    }
    res.sort_by(|a, b| b.partial_cmp(a).unwrap_or(core::cmp::Ordering::Equal));
    let acima = res.iter().filter(|r| **r > 20.0).count();
    println!(
        "PREGA vertices={} relevo={relevo:.3} ondula={ondula:.4} lambda={lambda:.2} \
         | residuo p0={:.1} p1={:.1} p2={:.1} p10={:.1} acima_de_20={acima}",
        antes.vert_count(),
        res[0],
        res[1],
        res[2],
        res[10.min(res.len() - 1)]
    );
}
