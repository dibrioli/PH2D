//! **DE QUE É FEITO O RASGO** — a sonda que atribui o report de 2026-08-16.
//!
//! O report (com foto): *"tanto hardness como falloffs apresenta problemas
//! graves"*. A foto mostra traços LISOS ao lado de traços **em escamas** — a
//! superfície partida em lascas com quinas duras, e o barro entre elas a mostrar
//! a esfera de baixo.
//!
//! Ela mede três coisas, e as três sobre o MESMO traço:
//!
//! * **o DIEDRO** — o ângulo entre triângulos vizinhos. É o que a luz desenha:
//!   uma superfície lisa fica em poucos graus, uma escama fica em dezenas.
//! * **o GUARDA-CHUVA** — o desvio de um vértice à média dos vizinhos, em
//!   arestas locais. É o irmão do `measure_dyntopo_spikes`, e separa *agulha*
//!   (um vértice sozinho) de *degrau* (uma fileira inteira deslocada).
//! * **o PERFIL** — a altura depositada por faixa de raio. É ele que diz se a
//!   curva desenhou o ombro que promete.
//!
//! ⚠️ **Sem dyntopo, de propósito.** A malha default do produto já tem 98 304
//! quads, e ligar o refino misturaria duas causas num número só. Se o rasgo
//! aparecer aqui, ele é da LEI do dab; se não aparecer, é do refino.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --test measure_hardness_and_falloff
//! --release -- --ignored --nocapture`

use ph2d_mesh::{Mesh, Ray, shapes::sculpt_sphere};
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};

/// **O CENTRO DO DAB, COMO O PRODUTO O ACHA** — um raio de fora para dentro,
/// re-picado a CADA passo (`sculpt3d_input.rs`: cada passo do `walk` chama
/// `sculpt_at(sx, sy)`, que é um pick).
///
/// ⚠️ **É esta linha que carrega o auto-limite, e a primeira versão desta sonda
/// não a tinha.** Com o centro pregado na esfera de PARTIDA ele fica ENTERRADO
/// sob o barro que sobe; as distâncias saem do `pre` congelado, nunca crescem, e
/// o depósito cresce sem fim — medido, dezasseis dabs no mesmo sítio davam
/// **dezasseis vezes** um. Com o centro a subir, o `pre` de cada vértice fica
/// cada vez mais LONGE dele, o peso cai e o pincel se esgota. *Uma fixture que
/// prende a câmara na superfície original não contém o gesto que ela mede.*
fn pick(mesh: &Mesh, dir: [f32; 3]) -> Option<[f32; 3]> {
    let origin = [dir[0] * 3.0, dir[1] * 3.0, dir[2] * 3.0];
    let ray = Ray::new(origin, [-dir[0], -dir[1], -dir[2]]);
    mesh.raycast(&ray).map(|h| h.point)
}

/// O raio e a força do traço — os do produto, não números escolhidos aqui.
const RADIUS: f32 = 0.30;
const STRENGTH: f32 = 0.5;
const DABS: usize = 24;

/// Um traço reto pelo topo da esfera, com o pincel dado. Devolve a malha, as
/// posições de ANTES e o conjunto de vértices que o traço de facto moveu.
///
/// ⚠️ **O `before` viaja com o resultado, e a primeira versão desta sonda não o
/// devolvia.** Ela binava pela posição FINAL, que já carrega o deslocamento: um
/// vértice no eixo, empurrado `0,10` para fora, media `0,10` de distância ao
/// eixo e caía na terceira faixa. O perfil saía com as duas primeiras faixas
/// VAZIAS e o pico no meio — a assinatura de uma curva oca, que nenhuma delas
/// tem. *Uma coordenada derivada do estado que o experimento move não é uma
/// coordenada.*
fn stroke_maybe_refined(brush: &Brush, refine: bool) -> (Mesh, Vec<[f32; 3]>, Vec<u32>) {
    let mut mesh = sculpt_sphere(1.0);
    mesh.triangulate();
    mesh.rebuild();
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();

    let mut stroke = SculptStroke::default();
    let mut births: Vec<ph2d_mesh::Birth> = Vec::new();
    let mut scratch = ph2d_mesh::RegionScratch::default();
    stroke.begin(&mesh);
    for k in 0..DABS {
        let t = k as f32 / (DABS - 1) as f32;
        let x = -0.5 + 1.0 * t;
        let y = (1.0 - x * x).max(0.0).sqrt();
        let dir = [x, y, 0.0];
        let Some(center) = pick(&mesh, dir) else {
            continue;
        };
        if refine {
            let target = ph2d_mesh::edge_target(brush.radius, 0.5);
            let _ = ph2d_mesh::refine_in_sphere(
                &mut mesh,
                center,
                brush.radius,
                target,
                &mut births,
                &mut scratch,
            );
            stroke.grow_with(&mesh, &births);
        }
        let eye = [-dir[0], -dir[1], -dir[2]];
        stroke.dab(
            &mut mesh,
            brush,
            &Dab::at(center, brush.radius, eye),
            Symmetry::default(),
        );
    }

    // ⚠️ O refino APENDA vértices, então o `before` é mais curto que a malha —
    // as estatísticas percorrem só o prefixo comum, que são os vértices que
    // existiam quando o traço começou.
    let after = mesh.positions();
    let moved: Vec<u32> = (0..before.len().min(after.len()))
        .filter(|&i| {
            let (a, b) = (before[i], after[i]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() > 1e-6
        })
        .map(|i| i as u32)
        .collect();
    (mesh, before, moved)
}

/// A distância de um ponto da esfera de partida ao EIXO do traço (o arco de
/// `x = -0,5` a `x = +0,5` no plano `z = 0`).
fn axis_dist(p: [f32; 3]) -> f32 {
    let x = p[0].clamp(-0.5, 0.5);
    let y = (1.0 - x * x).max(0.0).sqrt();
    let d = [p[0] - x, p[1] - y, p[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// `(pior, p99, média)` do diedro em GRAUS, contado só nas arestas que tocam a
/// região trabalhada — o resto da esfera é liso e diluiria a estatística.
fn dihedral(mesh: &Mesh, moved: &[u32]) -> (f32, f32, f32) {
    let pos = mesh.positions();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    mesh.triangle_indices(&mut tris);
    let mut is_moved = vec![false; pos.len()];
    for &v in moved {
        is_moved[v as usize] = true;
    }
    let normal = |t: &[u32; 3]| {
        let (a, b, c) = (pos[t[0] as usize], pos[t[1] as usize], pos[t[2] as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l < 1e-20 {
            None
        } else {
            Some([n[0] / l, n[1] / l, n[2] / l])
        }
    };
    // aresta (min,max) -> triângulos que a compartilham
    let mut edges: std::collections::BTreeMap<(u32, u32), Vec<usize>> =
        std::collections::BTreeMap::new();
    for (ti, t) in tris.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            let key = if a < b { (a, b) } else { (b, a) };
            edges.entry(key).or_default().push(ti);
        }
    }
    let mut all: Vec<f32> = Vec::new();
    for ((a, b), ts) in &edges {
        if ts.len() != 2 || !(is_moved[*a as usize] || is_moved[*b as usize]) {
            continue;
        }
        let (Some(n0), Some(n1)) = (normal(&tris[ts[0]]), normal(&tris[ts[1]])) else {
            continue;
        };
        let c = (n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]).clamp(-1.0, 1.0);
        all.push(c.acos().to_degrees());
    }
    all.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let worst = all.last().copied().unwrap_or(0.0);
    let p99 = all
        .get((all.len() as f32 * 0.99) as usize)
        .copied()
        .unwrap_or(0.0);
    let mean = all.iter().sum::<f32>() / all.len().max(1) as f32;
    (worst, p99, mean)
}

/// O perfil ATRAVÉS do traço: deslocamento médio por faixa de distância ao
/// eixo, em oito faixas de `raio/8` — **binado pela posição de ANTES**.
fn profile(before: &[[f32; 3]], mesh: &Mesh) -> [f32; 8] {
    let pos = mesh.positions();
    let mut sum = [0.0f64; 8];
    let mut cnt = [0u32; 8];
    for (i, b) in before.iter().enumerate() {
        let k = ((axis_dist(*b) / RADIUS) * 8.0) as usize;
        if k < 8 {
            let a = pos[i];
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            sum[k] += f64::from((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
            cnt[k] += 1;
        }
    }
    std::array::from_fn(|k| {
        if cnt[k] == 0 {
            0.0
        } else {
            (sum[k] / f64::from(cnt[k])) as f32
        }
    })
}

/// **A ONDULAÇÃO AO LONGO DO TRAÇO** — a escama da foto.
///
/// Amostra a crista (os vértices a menos de `raio/4` do eixo) em dezasseis
/// fatias de `x`, e devolve `(pico-a-pico ÷ altura média, altura média)`. Um
/// traço liso dá alguns por cento; uma fileira de escamas dá dezenas.
///
/// ⚠️ As duas fatias das PONTAS ficam de fora: ali a altura cai a zero por
/// desenho, e incluí-las mediria o começo e o fim do traço em vez da crista.
fn scallop(before: &[[f32; 3]], mesh: &Mesh) -> (f32, f32) {
    const N: usize = 16;
    let pos = mesh.positions();
    let mut sum = [0.0f64; N];
    let mut cnt = [0u32; N];
    for (i, b) in before.iter().enumerate() {
        if axis_dist(*b) > RADIUS * 0.25 || b[0] < -0.5 || b[0] > 0.5 || b[2].abs() > 0.5 {
            continue;
        }
        let k = (((b[0] + 0.5) / 1.0) * N as f32) as usize;
        if k < N {
            let a = pos[i];
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            sum[k] += f64::from((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
            cnt[k] += 1;
        }
    }
    let h: Vec<f32> = (1..N - 1)
        .filter(|k| cnt[*k] > 0)
        .map(|k| (sum[k] / f64::from(cnt[k])) as f32)
        .collect();
    if h.len() < 3 {
        return (0.0, 0.0);
    }
    let mean = h.iter().sum::<f32>() / h.len() as f32;
    let lo = h.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = h.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    (if mean > 1e-9 { (hi - lo) / mean } else { 0.0 }, mean)
}

fn run(name: &str, brush: &Brush) {
    run_maybe(name, brush, false);
}

fn run_maybe(name: &str, brush: &Brush, refine: bool) {
    let (mesh, before, moved) = stroke_maybe_refined(brush, refine);
    let (dw, dp, _median) = dihedral(&mesh, &moved);
    let prof = profile(&before, &mesh);
    let (ripple, crest) = scallop(&before, &mesh);
    let cols: Vec<String> = prof.iter().map(|h| format!("{h:6.4}")).collect();
    println!(
        "  {name:<26} diedro pior {dw:6.2} p99 {dp:6.2}  crista {crest:6.4} ondul {:5.1}%  perfil [{}]",
        ripple * 100.0,
        cols.join(" ")
    );
}

/// **A VARREDURA DAS DOZE CURVAS**, com o `hardness` no neutro.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_every_falloff() {
    println!("\nAS DOZE CURVAS  (Draw, raio {RADIUS}, forca {STRENGTH}, hardness 0)");
    for f in Falloff::ALL {
        let brush = Brush {
            verb: Verb::Draw,
            falloff: f,
            radius: RADIUS,
            strength: STRENGTH,
            ..Brush::default()
        };
        run(f.label(), &brush);
    }
}

/// **A VARREDURA DA DUREZA**, com a curva de fábrica.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_every_hardness() {
    println!("\nA DUREZA  (Draw, raio {RADIUS}, forca {STRENGTH}, curva Smooth)");
    for h in [0.0f32, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let brush = Brush {
            verb: Verb::Draw,
            falloff: Falloff::Smooth,
            radius: RADIUS,
            strength: STRENGTH,
            hardness: h,
            ..Brush::default()
        };
        run(&format!("hardness {h:.2}"), &brush);
    }
}

/// **UM DAB SÓ** — a curva contra o barro, sem caminho, sem envelope e sem
/// vizinho. É a única forma de separar *a curva está errada* de *o traço a
/// lava*.
///
/// Imprime o deslocamento medido em dezasseis faixas de `t`, **normalizado pela
/// primeira**, ao lado do que a curva analítica manda. Se as duas colunas
/// concordam, a lei do dab está certa e o defeito é do que vem depois.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_one_dab_against_the_curve() {
    const N: usize = 16;
    for f in [
        Falloff::Smooth,
        Falloff::Constant,
        Falloff::Sphere,
        Falloff::Pow4,
    ] {
        let brush = Brush {
            verb: Verb::Draw,
            falloff: f,
            radius: RADIUS,
            strength: STRENGTH,
            ..Brush::default()
        };
        let mut mesh = sculpt_sphere(1.0);
        mesh.triangulate();
        mesh.rebuild();
        let before: Vec<[f32; 3]> = mesh.positions().to_vec();
        let center = [0.0, 1.0, 0.0];
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at(center, brush.radius, [0.0, -1.0, 0.0]),
            Symmetry::default(),
        );
        let pos = mesh.positions();
        let mut sum = [0.0f64; N];
        let mut cnt = [0u32; N];
        for (i, b) in before.iter().enumerate() {
            let d = [b[0] - center[0], b[1] - center[1], b[2] - center[2]];
            let t = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() / RADIUS;
            let k = (t * N as f32) as usize;
            if k < N {
                let a = pos[i];
                let m = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                sum[k] += f64::from((m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt());
                cnt[k] += 1;
            }
        }
        let h: Vec<f32> = (0..N)
            .map(|k| {
                if cnt[k] == 0 {
                    0.0
                } else {
                    (sum[k] / f64::from(cnt[k])) as f32
                }
            })
            .collect();
        let peak = h[0].max(1e-9);
        let med: Vec<String> = h.iter().map(|v| format!("{:5.3}", v / peak)).collect();
        let want: Vec<String> = (0..N)
            .map(|k| {
                let t = (k as f32 + 0.5) / N as f32;
                format!("{:5.3}", f.weight(t))
            })
            .collect();
        println!("\n  {} (pico {peak:.5})", f.label());
        println!("    medido [{}]", med.join(" "));
        println!("    curva  [{}]", want.join(" "));
    }
}

/// **APERTAR NO MESMO LUGAR** — a saturação.
///
/// Um envelope satura: `n` dabs no mesmo sítio dão o mesmo que um. Uma soma
/// cresce com `n`. É a diferença entre um traço lento e um traço rápido
/// deixarem a MESMA marca ou não, e ela decide se o rasgo é da curva ou da
/// composição.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_pressing_in_the_same_place() {
    println!(
        "\n  default accumulate do Draw: {}",
        Brush::default().accumulate
    );
    for (f, acc) in [
        (Falloff::Smooth, false),
        (Falloff::Smooth, true),
        (Falloff::Constant, false),
        (Falloff::Constant, true),
    ] {
        println!("\n  {} accumulate={acc}", f.label());
        for n in [1usize, 2, 4, 8, 16] {
            let brush = Brush {
                verb: Verb::Draw,
                falloff: f,
                radius: RADIUS,
                strength: STRENGTH,
                accumulate: acc,
                ..Brush::default()
            };
            let mut mesh = sculpt_sphere(1.0);
            mesh.triangulate();
            mesh.rebuild();
            let before: Vec<[f32; 3]> = mesh.positions().to_vec();
            let mut stroke = SculptStroke::default();
            stroke.begin(&mesh);
            for _ in 0..n {
                // ⚠️ RE-PICA a cada dab, como o `sculpt_at` do produto.
                let Some(center) = pick(&mesh, [0.0, 1.0, 0.0]) else {
                    break;
                };
                stroke.dab(
                    &mut mesh,
                    &brush,
                    &Dab::at(center, brush.radius, [0.0, -1.0, 0.0]),
                    Symmetry::default(),
                );
            }
            let pos = mesh.positions();
            let peak = before
                .iter()
                .enumerate()
                .map(|(i, b)| {
                    let a = pos[i];
                    let m = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                    (m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt()
                })
                .fold(0.0f32, f32::max);
            println!("    {n:2} dab(s) no mesmo sitio: pico {peak:.5}");
        }
    }
}

/// **O MESMO TRAÇO COM O REFINO LIGADO** — o cruzamento que separa *o rasgo é
/// da CURVA* de *o rasgo é da TOPOLOGIA*.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_with_dyntopo() {
    println!("\nCOM REFINO  (Draw, raio {RADIUS}, forca {STRENGTH})");
    for f in [
        Falloff::Plateau,
        Falloff::Smooth,
        Falloff::Sphere,
        Falloff::Constant,
    ] {
        let brush = Brush {
            verb: Verb::Draw,
            falloff: f,
            radius: RADIUS,
            strength: STRENGTH,
            ..Brush::default()
        };
        run_maybe(&format!("{} +refino", f.label()), &brush, true);
        run_maybe(&format!("{} sem refino", f.label()), &brush, false);
    }
}

/// **O SEGUNDO PASSE CONTRA O DEGRAU** — quanto de `auto_smooth` é preciso para
/// que uma curva de platô deixe uma superfície que a malha carrega.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_auto_smooth_against_the_cliff() {
    println!("\nO SEGUNDO PASSE  (Draw, raio {RADIUS}, forca {STRENGTH})");
    for (name, f, h) in [
        ("Plateau (fabrica)", Falloff::Plateau, 0.0f32),
        ("Constant", Falloff::Constant, 0.0),
        ("Sphere", Falloff::Sphere, 0.0),
        ("Smooth + hardness 0,9", Falloff::Smooth, 0.9),
    ] {
        println!("\n  {name}");
        for a in [0.0f32, 0.10, 0.25, 0.50, 1.0] {
            let brush = Brush {
                verb: Verb::Draw,
                falloff: f,
                radius: RADIUS,
                strength: STRENGTH,
                hardness: h,
                auto_smooth: a,
                ..Brush::default()
            };
            run(&format!("auto_smooth {a:.2}"), &brush);
        }
    }
}

/// **A TOPOLOGIA DINÂMICA COMO O PRODUTO A RODA** — colapso ANTES do refino,
/// com o alvo do slider, exactamente o `refine_for_dab` da shell.
///
/// ⚠️ A sonda anterior (`measure_with_dyntopo`) só REFINAVA, e por isso mediu
/// zero: a esfera de fábrica já é mais fina que o alvo. O produto também
/// **COLAPSA**, e é a metade que faltava.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_the_dyntopo_the_product_runs() {
    let base = sculpt_sphere(1.0);
    let pos = base.positions();
    let adj = base.adjacency();
    let mut len = 0.0f64;
    let mut n = 0u32;
    for (i, p) in pos.iter().enumerate() {
        for &j in adj.vert_verts.neighbours(i) {
            let q = pos[j as usize];
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            len += f64::from((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
            n += 1;
        }
    }
    let edge = (len / f64::from(n.max(1))) as f32;
    println!(
        "\n  a esfera de fabrica: {} verts, aresta media {edge:.5}",
        pos.len()
    );
    for detail in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
        let target = ph2d_mesh::edge_target(RADIUS, detail);
        println!(
            "  detail {detail:.2}: alvo de refino {target:.5} ({:.1}x a aresta), alvo de colapso {:.5}",
            target / edge,
            ph2d_mesh::collapse_target(target)
        );
    }

    for detail in [0.5f32, 1.0] {
        let brush = Brush {
            verb: Verb::Draw,
            falloff: Falloff::Plateau,
            radius: RADIUS,
            strength: STRENGTH,
            ..Brush::default()
        };
        let mut mesh = sculpt_sphere(1.0);
        mesh.triangulate();
        mesh.rebuild();
        let before: Vec<[f32; 3]> = mesh.positions().to_vec();
        let mut stroke = SculptStroke::default();
        let mut births = Vec::new();
        let mut remap = ph2d_mesh::Remap::default();
        let mut region = ph2d_mesh::RegionScratch::default();
        stroke.begin(&mesh);
        for k in 0..DABS {
            let t = k as f32 / (DABS - 1) as f32;
            let x = -0.5 + 1.0 * t;
            let dir = [x, (1.0 - x * x).max(0.0).sqrt(), 0.0];
            let Some(center) = pick(&mesh, dir) else {
                continue;
            };
            let target = ph2d_mesh::edge_target(RADIUS, detail);
            let shrunk = ph2d_mesh::collapse_in_sphere(
                &mut mesh,
                center,
                RADIUS,
                ph2d_mesh::collapse_target(target),
                &mut remap,
                &mut region,
            );
            if matches!(shrunk, ph2d_mesh::Collapse::Done { .. }) {
                stroke.shrink_with(&remap);
            }
            let out = ph2d_mesh::refine_in_sphere(
                &mut mesh,
                center,
                RADIUS,
                target,
                &mut births,
                &mut region,
            );
            if matches!(out, ph2d_mesh::Refine::Done { .. }) {
                stroke.grow_with(&mesh, &births);
            }
            let eye = [-dir[0], -dir[1], -dir[2]];
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::at(center, brush.radius, eye),
                Symmetry::default(),
            );
        }
        let all: Vec<u32> = (0..mesh.vert_count() as u32).collect();
        let (w, p, m) = dihedral(&mesh, &all);
        println!(
            "\n  detail {detail:.2} COM colapso+refino: {} verts (era {}), diedro pior {w:.2} p99 {p:.2} media {m:.2}",
            mesh.vert_count(),
            before.len()
        );
    }
}

/// **O COLAPSO SOZINHO** — sem dab, sem refino, sem pincel. Se a dobra aparecer
/// aqui, ela é do OPERADOR e não tem nada a ver com curva nem com dureza.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_the_collapse_alone() {
    for detail in [0.5f32, 1.0] {
        let mut mesh = sculpt_sphere(1.0);
        mesh.triangulate();
        mesh.rebuild();
        let before = mesh.vert_count();
        let all: Vec<u32> = (0..before as u32).collect();
        let (w0, p0, _) = dihedral(&mesh, &all);
        let target = ph2d_mesh::collapse_target(ph2d_mesh::edge_target(RADIUS, detail));
        let mut remap = ph2d_mesh::Remap::default();
        let mut region = ph2d_mesh::RegionScratch::default();
        let out = ph2d_mesh::collapse_in_sphere(
            &mut mesh,
            [0.0, 1.0, 0.0],
            RADIUS,
            target,
            &mut remap,
            &mut region,
        );
        let all: Vec<u32> = (0..mesh.vert_count() as u32).collect();
        let (w, p, m) = dihedral(&mesh, &all);
        println!(
            "\n  detail {detail:.2} (alvo {target:.5}) {out:?}\n    antes  {before} verts, diedro pior {w0:.2} p99 {p0:.2}\n    depois {} verts, diedro pior {w:.2} p99 {p:.2} media {m:.2}",
            mesh.vert_count()
        );
    }
}

/// **O PERCURSO INTEIRO DE TOPOLOGIA, SEM UM ÚNICO DAB** — o controle que
/// separa *a dobra é da TOPOLOGIA* de *a dobra é do TRAÇO em voo*.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_the_topology_walk_without_any_dab() {
    let target = ph2d_mesh::edge_target(RADIUS, 0.5);
    let mut mesh = sculpt_sphere(1.0);
    mesh.triangulate();
    mesh.rebuild();
    let before = mesh.vert_count();
    let mut births = Vec::new();
    let mut remap = ph2d_mesh::Remap::default();
    let mut region = ph2d_mesh::RegionScratch::default();
    for k in 0..DABS {
        let t = k as f32 / (DABS - 1) as f32;
        let x = -0.5 + 1.0 * t;
        let dir = [x, (1.0 - x * x).max(0.0).sqrt(), 0.0];
        let Some(center) = pick(&mesh, dir) else {
            continue;
        };
        let _ = ph2d_mesh::collapse_in_sphere(
            &mut mesh,
            center,
            RADIUS,
            ph2d_mesh::collapse_target(target),
            &mut remap,
            &mut region,
        );
        let _ = ph2d_mesh::refine_in_sphere(
            &mut mesh,
            center,
            RADIUS,
            target,
            &mut births,
            &mut region,
        );
    }
    let all: Vec<u32> = (0..mesh.vert_count() as u32).collect();
    let (w, p, m) = dihedral(&mesh, &all);
    println!(
        "\n  SEM DAB: {} verts (era {before}), diedro pior {w:.2} p99 {p:.2} media {m:.2}",
        mesh.vert_count()
    );
}

/// **O CONTROLE**: a mesma esfera SEM traço nenhum. Se o diedro dela já for
/// grande, a régua não fala sobre o pincel.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_the_untouched_sphere() {
    let mut mesh = sculpt_sphere(1.0);
    mesh.triangulate();
    mesh.rebuild();
    let all: Vec<u32> = (0..mesh.vert_count() as u32).collect();
    let (w, p, m) = dihedral(&mesh, &all);
    println!(
        "\nCONTROLE: esfera intocada, {} verts, diedro pior {w:.2} p99 {p:.2} media {m:.2}",
        mesh.vert_count()
    );
}
