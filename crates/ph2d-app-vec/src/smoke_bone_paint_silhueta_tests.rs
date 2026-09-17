//! ⭐⭐⭐ **A SILHUETA DESENHADA VAI E VOLTA?** — a régua que o olho usa, e o gate que a pôs no
//! produto (report do dono, 2026-09-16, foto com cinco setas: *«Smooth parece ter resultado
//! discretamente inferior, gerando micro irregularidades»*).
//!
//! Filho do [`super`] por RESPONSABILIDADE: *o `Smooth` refina onde a dobra pede* é a pergunta
//! dele; *o que ele desenha é mais liso que o `Fast`* é esta.
//!
//! ⛔⛔ **O gate do irmão estava VERDE sobre o defeito, e a razão é que a régua dele partilhava a
//! lei do produto:** ele mede o desvio da malha contra o CAMPO que ela segue, e o campo P1 tinha um
//! vinco em cada aresta do bind — o `Smooth` seguia os vincos fielmente e a régua aprovava. *Um
//! espelho não acusa.* A régua daqui não sabe que campo existe: ela olha só a polilinha desenhada.
//!
//! ⚠️ **A arte desta cena é um rectângulo OPACO**, logo a silhueta é a fronteira da malha e os
//! quatro lados dela são os quatro lados do rectângulo de repouso.

use super::*;

/// Os vértices de um LADO do rectângulo da arte, ordenados ao longo dele — `(índice, repouso)`.
///
/// ⚠️ **A arte desta cena é um rectângulo OPACO**, logo a malha cobre-o de ponta a ponta e a
/// silhueta desenhada **é** a fronteira da malha: os vértices com `repouso[eixo] == valor` são
/// exactamente os que o contorno visível atravessa, e a ordem ao longo do lado é a ordem deles.
fn lado(m: &ph2d_poly2d::Mesh2d, eixo: usize, valor: f64) -> Vec<usize> {
    let mut v: Vec<usize> = (0..m.rest.len())
        .filter(|&i| (m.rest[i][eixo] - valor).abs() < 1e-9)
        .collect();
    v.sort_by(|&a, &b| m.rest[a][1 - eixo].total_cmp(&m.rest[b][1 - eixo]));
    v
}

/// ⭐⭐⭐ **QUANTO UMA POLILINHA VAI E VOLTA** — `(rotação líquida, rotação total, maior canto)`,
/// em graus.
///
/// ⚠️ **A régua é a rotação que se CANCELA**: numa curva que dobra sempre para o mesmo lado a
/// rotação total é igual à líquida; toda a diferença é tangente que foi para um lado e voltou —
/// que é, literalmente, o *«micro-irregularidade»* do report. Ela não depende de onde os vértices
/// caem nem de quantos são.
fn vai_e_volta(pts: &[[f64; 2]]) -> (f64, f64, f64) {
    let (mut liquida, mut total, mut maior) = (0.0_f64, 0.0_f64, 0.0_f64);
    for w in pts.windows(3) {
        let (a, b) = (
            [w[1][0] - w[0][0], w[1][1] - w[0][1]],
            [w[2][0] - w[1][0], w[2][1] - w[1][1]],
        );
        let ang = (a[0] * b[1] - a[1] * b[0])
            .atan2(a[0] * b[0] + a[1] * b[1])
            .to_degrees();
        liquida += ang;
        total += ang.abs();
        maior = maior.max(ang.abs());
    }
    (liquida, total, maior)
}

/// A silhueta de uma malha posada, lado a lado — `(nome, pontos em px de ecrã)`.
fn silhueta(
    m: &ph2d_poly2d::Mesh2d,
    posed: &[[f64; 2]],
    zoom: f64,
) -> Vec<(&'static str, Vec<[f64; 2]>)> {
    let (w, h) = (f64::from(m.size[0]), f64::from(m.size[1]));
    [
        ("topo", 1, 0.0),
        ("base", 1, h),
        ("esq", 0, 0.0),
        ("dir", 0, w),
    ]
    .into_iter()
    .map(|(nome, eixo, valor)| {
        let pts = lado(m, eixo, valor)
            .into_iter()
            .map(|i| {
                [
                    posed[i][0] * PX_POR_METRO * zoom,
                    posed[i][1] * PX_POR_METRO * zoom,
                ]
            })
            .collect();
        (nome, pts)
    })
    .collect()
}

/// ⭐⭐⭐⭐ **O `Smooth` NÃO DESENHA OS VINCOS DOS PESOS** — a silhueta dele vai e volta tanto quanto
/// a do `Fast`, em todo zoom e com orçamento de sobra.
///
/// A tabela que esta cena imprime (lado de cima, zoom `8×`; os outros três lados empatam):
///
/// | desenho | nós | vai-e-volta | trocas de sinal | maior canto |
/// |---|---:|---:|---:|---:|
/// | `Fast` | `46` | `26,60°` | `2` | `3,57°` |
/// | `Smooth`, pesos em linha recta (a lei de ANTES), zoom `4×` | `60` | **`46,61°`** | **`20`** | `3,10°` |
/// | o mesmo, zoom `8×` | `66` | **`47,00°`** | **`24`** | `3,10°` |
/// | o mesmo, orçamento `×16` | `74` | **`57,95°`** | **`36`** | `3,10°` |
/// | `Smooth`, pesos de Hermite, zoom `8×` | `57` | `26,62°` | `2` | `3,22°` |
/// | o mesmo, orçamento `×16` | `73` | `26,71°` | `2` | `2,62°` |
///
/// ⭐⭐ **As duas trocas de sinal são a curva em S real** da corrente de ossos; as outras eram um
/// vinco por aresta do bind, e crescem com o orçamento — *quanto mais o botão trabalhava, pior
/// ficava*. ⚠️ **A barra sai do vale MEDIDO entre o lado aprovado (o `Fast`, que o dono pôs como
/// referência) e o defeito reproduzido**: o `Smooth` novo fica a `+0,11°` e `+0` trocas no pior
/// caso, a lei de antes a `+20,0°` e `+18` no melhor — a barra é `+2°` **ou** `+2` trocas.
///
/// ⛔ **O CONTROLO é a lei de antes a reprovar na MESMA corrida** — se ela deixar de reprovar, a
/// cena deixou de ter o defeito e o gate deixou de o medir.
///
/// (Red-first: `PH2D_SKIN_WEIGHTS=linear` põe o produto na lei de antes ⇒ RED. Mutação: a porta
/// do produto ler os pesos em linha recta ⇒ RED.)
#[test]
fn o_smooth_nao_desenha_os_vincos_dos_pesos() {
    const FOLGA_GRAUS: f64 = 2.0;
    const FOLGA_TROCAS: usize = 2;
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let mut ws = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);
    let fast: Vec<[f64; 2]> = sm
        .mesh
        .rest
        .iter()
        .enumerate()
        .map(|(v, &q)| campo(q, sm.pesos_de(v)))
        .collect();
    let mut controlo_reprovou = 0usize;
    for (zoom, mult) in [(1.0_f64, 1usize), (4.0, 1), (8.0, 1), (8.0, 16)] {
        let mut o = opcoes(true, zoom);
        o.max_pieces *= mult;
        let produto = ph2d_skeleton_live::skin_refine::refine_skinned(
            &sm.mesh, &sm.pesos, ossos, &mut campo, o,
        );
        let (m1, p1, _, _) =
            ph2d_poly2d::refine_posed_attrs(&sm.mesh, &sm.pesos, ossos, &mut campo, o);
        let lados_f = silhueta(&sm.mesh, &fast, zoom);
        let lados_s = silhueta(&produto.mesh, &produto.posed, zoom);
        let lados_p1 = silhueta(&m1, &p1, zoom);
        for ((nome, f), ((_, s), (_, antes))) in lados_f.iter().zip(lados_s.iter().zip(&lados_p1)) {
            let vv = |p: &[[f64; 2]]| {
                let (n, a, _) = vai_e_volta(p);
                (a - n.abs(), trocas_de_sinal(p))
            };
            let ((vf, tf), (vs, ts), (va, ta)) = (vv(f), vv(s), vv(antes));
            println!(
                "zoom {zoom}x orcamento x{mult:>2} {nome:>4} | Fast {vf:>6.2} ({tf:>2}) | Smooth \
                 {vs:>6.2} ({ts:>2}, {} nos) | lei de antes {va:>6.2} ({ta:>2})",
                s.len()
            );
            assert!(
                vs <= vf + FOLGA_GRAUS && ts <= tf + FOLGA_TROCAS,
                "zoom {zoom}x, {nome}: o Smooth vai e volta {vs:.2} graus com {ts} trocas de \
                 sinal, contra {vf:.2} e {tf} do Fast — os vincos dos pesos voltaram"
            );
            if va > vf + FOLGA_GRAUS || ta > tf + FOLGA_TROCAS {
                controlo_reprovou += 1;
            }
        }
    }
    // ⛔ A lei de antes tem de reprovar a MESMA barra onde ela refina o lado da dobra — zoom `4×`,
    // `8×` e `8×` com orçamento de sobra, no lado de cima.
    assert!(
        controlo_reprovou >= 3,
        "a lei de antes so' reprovou {controlo_reprovou} vez(es) — a cena deixou de ter o defeito"
    );
}

/// ⏱️ **SONDA (`--ignored`) — a silhueta desenhada, lado a lado, nas três leis.**
///
/// ```text
/// cargo test -p ph2d-app-vec --lib -- --ignored --nocapture sonda_a_silhueta_vai_e_volta
/// ```
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_a_silhueta_vai_e_volta() {
    let altura = super::super::super::ALTURA_PX;
    let (sim, e) = cena(altura, None);
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let (w, h) = (f64::from(sm.mesh.size[0]), f64::from(sm.mesh.size[1]));
    let mut ws = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);
    let lados = [
        ("topo", 1, 0.0),
        ("base", 1, h),
        ("esq", 0, 0.0),
        ("dir", 0, w),
    ];

    for zoom in [1.0_f64, 4.0, 8.0] {
        println!("\n=== zoom {zoom}x ===");
        let o = opcoes(true, zoom);
        let r = ph2d_skeleton_live::skin_refine::refine_skinned(
            &sm.mesh, &sm.pesos, ossos, &mut campo, o,
        );
        let (mr, pr) = (r.mesh, r.posed);
        println!(
            "smooth: {} pecas ({:?}, {:?})",
            mr.tris.len(),
            r.report.lei,
            r.law
        );
        let (m1, p1, _, _) =
            ph2d_poly2d::refine_posed_attrs(&sm.mesh, &sm.pesos, ossos, &mut campo, o);
        let pf: Vec<[f64; 2]> = sm
            .mesh
            .rest
            .iter()
            .enumerate()
            .map(|(v, &q)| campo(q, sm.pesos_de(v)))
            .collect();
        for (nome, eixo, valor) in lados {
            let px = |p: [f64; 2]| [p[0] * PX_POR_METRO * zoom, p[1] * PX_POR_METRO * zoom];
            let fast: Vec<[f64; 2]> = lado(&sm.mesh, eixo, valor)
                .into_iter()
                .map(|i| px(pf[i]))
                .collect();
            let suave: Vec<[f64; 2]> = lado(&mr, eixo, valor)
                .into_iter()
                .map(|i| px(pr[i]))
                .collect();
            let suave_p1: Vec<[f64; 2]> = lado(&m1, eixo, valor)
                .into_iter()
                .map(|i| px(p1[i]))
                .collect();
            // O CAMPO-ALVO, amostrado denso: pesos lineares entre os vértices do bind do lado.
            let bind = lado(&sm.mesh, eixo, valor);
            let mut alvo = Vec::new();
            for par in bind.windows(2) {
                let (a, b) = (par[0], par[1]);
                for s in 0..32 {
                    let t = f64::from(s) / 32.0;
                    let q = [
                        sm.mesh.rest[a][0] + (sm.mesh.rest[b][0] - sm.mesh.rest[a][0]) * t,
                        sm.mesh.rest[a][1] + (sm.mesh.rest[b][1] - sm.mesh.rest[a][1]) * t,
                    ];
                    let wq: Vec<f64> = sm
                        .pesos_de(a)
                        .iter()
                        .zip(sm.pesos_de(b))
                        .map(|(x, y)| x + (y - x) * t)
                        .collect();
                    alvo.push(px(campo(q, &wq)));
                }
            }
            if let Some(&u) = bind.last() {
                alvo.push(px(pf[u]));
            }
            for (lei, pts) in [
                ("fast", &fast),
                ("smooth", &suave),
                ("smoothP1", &suave_p1),
                ("alvo P1", &alvo),
            ] {
                let (n, a, m) = vai_e_volta(pts);
                println!(
                    "{nome:>5} {lei:>8}: {:>5} nos | liquida {n:>7.2} | total {a:>7.2} | \
                     VAI-E-VOLTA {:>6.2} | maior canto {m:>5.2}",
                    pts.len(),
                    a - n.abs()
                );
            }
        }
    }
}

/// ⏱️ **SONDA (`--ignored`) — o lado de CIMA vértice a vértice: onde o vai-e-volta nasce.**
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_o_lado_de_cima_vertice_a_vertice() {
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let mut ws = pele.scratch();
    let bind = lado(&sm.mesh, 1, 0.0);
    let pts: Vec<[f64; 2]> = bind
        .iter()
        .map(|&i| {
            let p = ponto_do_produto(&pele, p2l.apply(sm.mesh.rest[i]), sm.pesos_de(i), &mut ws);
            [p[0] * PX_POR_METRO, p[1] * PX_POR_METRO]
        })
        .collect();
    println!("   x(rest)  passo |  canto(graus) | pesos");
    for k in 0..bind.len() {
        let i = bind[k];
        let passo = if k > 0 {
            sm.mesh.rest[i][0] - sm.mesh.rest[bind[k - 1]][0]
        } else {
            0.0
        };
        let canto = if k > 0 && k + 1 < bind.len() {
            let (a, b) = (
                [pts[k][0] - pts[k - 1][0], pts[k][1] - pts[k - 1][1]],
                [pts[k + 1][0] - pts[k][0], pts[k + 1][1] - pts[k][1]],
            );
            (a[0] * b[1] - a[1] * b[0])
                .atan2(a[0] * b[0] + a[1] * b[1])
                .to_degrees()
        } else {
            0.0
        };
        let w: Vec<String> = sm.pesos_de(i).iter().map(|x| format!("{x:.4}")).collect();
        println!(
            "{:>9.2} {passo:>6.2} | {canto:>+12.4} | {}",
            sm.mesh.rest[i][0],
            w.join(" ")
        );
    }
}

/// Quantas vezes a rotação troca de sinal ao longo da polilinha (ignora ângulos abaixo de `1e-6°`).
fn trocas_de_sinal(pts: &[[f64; 2]]) -> usize {
    let mut sinal = 0.0_f64;
    let mut trocas = 0;
    for w in pts.windows(3) {
        let (a, b) = (
            [w[1][0] - w[0][0], w[1][1] - w[0][1]],
            [w[2][0] - w[1][0], w[2][1] - w[1][1]],
        );
        let ang = (a[0] * b[1] - a[1] * b[0]).atan2(a[0] * b[0] + a[1] * b[1]);
        if ang.abs().to_degrees() < 1e-6 {
            continue;
        }
        if sinal != 0.0 && ang.signum() != sinal {
            trocas += 1;
        }
        sinal = ang.signum();
    }
    trocas
}

/// ⏱️ **SONDA (`--ignored`) — o alvo com pesos de DERIVADA CONTÍNUA (Hermite) contra o P1.**
///
/// ⚠️ Os gradientes são os da PORTA do produto ([`ph2d_poly2d::recover_gradients`]) — uma segunda
/// cópia da lei aqui mediria outra coisa no primeiro ajuste.
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_o_alvo_hermite_contra_o_p1() {
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let g = ph2d_poly2d::recover_gradients(&sm.mesh, &sm.pesos, ossos);
    let grad = |v: usize, k: usize| [g[(v * ossos + k) * 2], g[(v * ossos + k) * 2 + 1]];
    let (w, h) = (f64::from(sm.mesh.size[0]), f64::from(sm.mesh.size[1]));
    let mut ws = pele.scratch();
    for (nome, eixo, valor) in [
        ("topo", 1, 0.0),
        ("base", 1, h),
        ("esq", 0, 0.0),
        ("dir", 0, w),
    ] {
        let bind = lado(&sm.mesh, eixo, valor);
        let mut p1 = Vec::new();
        let mut herm = Vec::new();
        let (mut pior_neg, mut soma_max) = (0.0_f64, 0.0_f64);
        for par in bind.windows(2) {
            let (a, b) = (par[0], par[1]);
            let (ra, rb) = (sm.mesh.rest[a], sm.mesh.rest[b]);
            let dx = [rb[0] - ra[0], rb[1] - ra[1]];
            for s in 0..32 {
                let t = f64::from(s) / 32.0;
                let q = [ra[0] + dx[0] * t, ra[1] + dx[1] * t];
                let (wa, wb) = (sm.pesos_de(a), sm.pesos_de(b));
                let lin: Vec<f64> = (0..ossos).map(|k| wa[k] + (wb[k] - wa[k]) * t).collect();
                let (t2, t3) = (t * t, t * t * t);
                let (h00, h10, h01, h11) = (
                    2.0 * t3 - 3.0 * t2 + 1.0,
                    t3 - 2.0 * t2 + t,
                    -2.0 * t3 + 3.0 * t2,
                    t3 - t2,
                );
                let her: Vec<f64> = (0..ossos)
                    .map(|k| {
                        let (ga, gb) = (grad(a, k), grad(b, k));
                        let (da, db) =
                            (ga[0] * dx[0] + ga[1] * dx[1], gb[0] * dx[0] + gb[1] * dx[1]);
                        h00 * wa[k] + h10 * da + h01 * wb[k] + h11 * db
                    })
                    .collect();
                pior_neg = pior_neg.min(her.iter().copied().fold(0.0_f64, f64::min));
                soma_max = soma_max.max((her.iter().sum::<f64>() - 1.0).abs());
                let px = |p: [f64; 2]| [p[0] * PX_POR_METRO, p[1] * PX_POR_METRO];
                p1.push(px(ponto_do_produto(&pele, p2l.apply(q), &lin, &mut ws)));
                herm.push(px(ponto_do_produto(&pele, p2l.apply(q), &her, &mut ws)));
            }
        }
        let desvio = p1
            .iter()
            .zip(&herm)
            .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
            .fold(0.0_f64, f64::max);
        for (lei, pts) in [("alvo P1", &p1), ("alvo Hermite", &herm)] {
            let (n, a, m) = vai_e_volta(pts);
            println!(
                "{nome:>5} {lei:>13}: liquida {n:>7.2} | total {a:>7.2} | VAI-E-VOLTA {:>6.2} | \
                 maior canto {m:>5.2} | trocas de sinal {}",
                a - n.abs(),
                trocas_de_sinal(pts)
            );
        }
        println!(
            "{nome:>5}  distancia maxima entre os dois alvos: {desvio:.3} px (zoom 1) | peso mais \
             negativo {pior_neg:.2e} | |soma - 1| {soma_max:.2e}"
        );
    }
}

/// ⏱️ **SONDA (`--ignored`) — o lado de cima com MAIS orçamento: para onde cada lei converge.**
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_o_topo_com_mais_orcamento() {
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let mut ws = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);
    for mult in [1usize, 4, 16, 64] {
        let mut o = opcoes(true, 8.0);
        o.max_pieces *= mult;
        let r = ph2d_skeleton_live::skin_refine::refine_skinned(
            &sm.mesh, &sm.pesos, ossos, &mut campo, o,
        );
        let (m1, p1, _, _) =
            ph2d_poly2d::refine_posed_attrs(&sm.mesh, &sm.pesos, ossos, &mut campo, o);
        for (lei, m, p) in [("Hermite", &r.mesh, &r.posed), ("P1", &m1, &p1)] {
            let pts: Vec<[f64; 2]> = lado(m, 1, 0.0)
                .into_iter()
                .map(|i| [p[i][0] * PX_POR_METRO * 8.0, p[i][1] * PX_POR_METRO * 8.0])
                .collect();
            let (n, a, c) = vai_e_volta(&pts);
            println!(
                "orcamento x{mult:>2} {lei:>7}: {:>6} pecas | topo {:>5} nos | VAI-E-VOLTA {:>6.2} | \
                 maior canto {c:>5.2} | trocas {}",
                m.tris.len(),
                pts.len(),
                a - n.abs(),
                trocas_de_sinal(&pts)
            );
        }
    }
}

/// ⏱️ **SONDA (`--ignored`) — o que a lei de Hermite custa ONDE ela trabalha: o refinamento da cena.**
///
/// ⚠️ O mínimo de 40 corridas, com a mediana ao lado e a carga impressa — acima de `load ~5` um
/// relógio desta workstation não vale nada sozinho, e por isso as duas leis correm INTERCALADAS.
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_o_custo_da_lei_dos_pesos() {
    use std::time::Instant;
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "carga: {}",
        carga
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );
    let (sim, e) = cena(super::super::super::ALTURA_PX, None);
    let (sm, p2l, pele) = campo_da_cena(&sim, e);
    let ossos = sm.ossos();
    let mut ws = pele.scratch();
    let mut campo =
        |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);
    for zoom in [4.0_f64, 8.0] {
        let o = opcoes(true, zoom);
        let (mut th, mut tl) = (Vec::new(), Vec::new());
        let (mut nh, mut nl) = (0, 0);
        for _ in 0..40 {
            let t = Instant::now();
            let r = ph2d_skeleton_live::skin_refine::refine_skinned(
                &sm.mesh, &sm.pesos, ossos, &mut campo, o,
            );
            th.push(t.elapsed().as_secs_f64() * 1e3);
            nh = r.mesh.tris.len();
            let t = Instant::now();
            let (m, _, _, _) =
                ph2d_poly2d::refine_posed_attrs(&sm.mesh, &sm.pesos, ossos, &mut campo, o);
            tl.push(t.elapsed().as_secs_f64() * 1e3);
            nl = m.tris.len();
        }
        th.sort_by(f64::total_cmp);
        tl.sort_by(f64::total_cmp);
        #[expect(clippy::cast_precision_loss, reason = "contagens de peças")]
        let pp = |ms: f64, n: usize| ms * 1e3 / n as f64;
        println!(
            "zoom {zoom}x | Hermite {:.3}/{:.3} ms ({nh} pecas, {:.3} us/p) | linear {:.3}/{:.3} ms \
             ({nl} pecas, {:.3} us/p)",
            th[0],
            th[20],
            pp(th[0], nh),
            tl[0],
            tl[20],
            pp(tl[0], nl)
        );
    }
}

/// ⏱️ **SONDA (`--ignored`) — a faceta do `Fast` numa dobra FORTE, com a malha de bind de hoje.**
#[test]
#[ignore = "sonda: imprime a tabela, sem barra"]
fn sonda_a_faceta_do_fast_na_dobra_forte() {
    for graus in [25.0_f32, 60.0, 90.0, 120.0, 150.0] {
        let (sim, e) = cena_dobrada(
            super::super::super::ALTURA_PX,
            None,
            ph2d_poly2d::GridOptions::default(),
            graus,
        );
        let (sm, p2l, pele) = campo_da_cena(&sim, e);
        let ossos = sm.ossos();
        let mut ws = pele.scratch();
        let mut campo =
            |q: [f64; 2], pesos: &[f64]| ponto_do_produto(&pele, p2l.apply(q), pesos, &mut ws);
        let fast: Vec<[f64; 2]> = sm
            .mesh
            .rest
            .iter()
            .enumerate()
            .map(|(v, &q)| campo(q, sm.pesos_de(v)))
            .collect();
        let lei = ph2d_skeleton_live::skin_refine::weight_law(ossos, true);
        let attrs = ph2d_skeleton_live::skin_refine::weight_attrs(&sm.mesh, &sm.pesos, lei);
        let faceta = ph2d_skeleton_live::skin_refine::skinned_deviation(
            &sm.mesh, &fast, &attrs, lei, &mut campo,
        ) * PX_POR_METRO;
        let lados = silhueta(&sm.mesh, &fast, 1.0);
        let pior_canto = lados
            .iter()
            .map(|(_, p)| vai_e_volta(p).2)
            .fold(0.0_f64, f64::max);
        let mut linha = format!(
            "{graus:>5} graus | {} pecas | Fast: faceta {faceta:.2} px (z1) {:.2} (z4) | maior canto \
             da silhueta {pior_canto:.2}",
            sm.mesh.tris.len(),
            faceta * 4.0
        );
        for zoom in [1.0_f64, 4.0] {
            let r = ph2d_skeleton_live::skin_refine::refine_skinned(
                &sm.mesh,
                &sm.pesos,
                ossos,
                &mut campo,
                opcoes(true, zoom),
            );
            let f = ph2d_skeleton_live::skin_refine::skinned_deviation(
                &r.mesh, &r.posed, &r.attrs, r.law, &mut campo,
            ) * PX_POR_METRO
                * zoom;
            let canto = silhueta(&r.mesh, &r.posed, 1.0)
                .iter()
                .map(|(_, p)| vai_e_volta(p).2)
                .fold(0.0_f64, f64::max);
            linha += &format!(
                " || Smooth z{zoom}: {} pecas, faceta {f:.2} px, maior canto {canto:.2}",
                r.mesh.tris.len()
            );
        }
        println!("{linha}");
    }
}

#[path = "smoke_bone_paint_pincel_tests.rs"]
mod pincel;

#[path = "smoke_bone_paint_assada_tests.rs"]
mod assada;
