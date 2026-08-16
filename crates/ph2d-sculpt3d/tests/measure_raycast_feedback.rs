//! **O RAIO QUE AFUNDA NA PRÓPRIA CRATERA** — o laço temporal do centro do dab.
//!
//! O centro de um dab sai de um PICK. A pergunta que esta sonda faz é *contra
//! qual superfície*, e as duas respostas possíveis levam a comportamentos
//! qualitativamente diferentes:
//!
//! - **VIVA** — o raio bate na malha que o traço está a deformar. O dab `n`
//!   mede a cratera que os dabs `1..n−1` cavaram, desce para dentro dela, e o
//!   dab `n+1` mede uma cratera mais funda. É realimentação POSITIVA.
//! - **CONGELADA** — o raio bate nas posições de antes do traço. A profundidade
//!   satura no valor nominal do pincel.
//!
//! ⚠️ **O Blender faz a CONGELADA por default**, e a linha está no
//! `sculpt.cc:4899`: `original = force_original || (cache ? !cache->accum :
//! false)` — com *Accumulate* desligado (o de fábrica) o `pbvh::raycast` é
//! testado contra as posições que o undo guardou (`sculpt.cc:4476-4499`). O
//! mesmo vale para a amostragem de ÁREA que orienta o dab (`sculpt.cc:1544`).
//!
//! ⚠️ **E o gesto que separa as duas NÃO é um traço rápido** — é o traço LENTO,
//! ou parado, sobre o mesmo ponto: ali o mesmo lugar recebe muitos dabs e o
//! laço tem tempo de correr. Num arrasto rápido cada ponto recebe um ou dois
//! dabs e as duas leis medem quase o mesmo, que é como isto atravessa uma
//! suíte inteira de fixtures de traço.
//!
//! O oráculo é a forma da curva `profundidade(n)`, não um número isolado:
//! saturação contra aceleração. Uma barra absoluta não distinguiria as duas
//! (a congelada TAMBÉM cava), e é a CURVATURA que carrega o defeito.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --test measure_raycast_feedback --release
//! -- --ignored --nocapture`

use ph2d_mesh::{Mesh, Ray, shapes::uv_sphere};
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};

/// Contra que superfície o pick é resolvido.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PickAgainst {
    /// O que este repo faz hoje: `stack.mesh().raycast(..)`, a malha viva.
    Live,
    /// O que o Blender faz com *Accumulate* desligado.
    Frozen,
}

/// O RELEVO do ponto mais deslocado, medido do raio unitário para fora.
///
/// ⚠️ **A primeira versão media `1 − r` (para DENTRO) e imprimia zeros em toda
/// a tabela** — o `Verb::Draw` empurra para FORA, então `1 − r` é negativo em
/// todo vértice tocado e o `max` caía nos vértices intocados, que valem
/// exactamente 0. *Uma régua com o sinal trocado não mede pouco: mede o
/// CONTROLE, e reporta que não há fenómeno nenhum.*
fn depth(mesh: &Mesh) -> f32 {
    mesh.positions()
        .iter()
        .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - 1.0)
        .fold(f32::NEG_INFINITY, f32::max)
}

/// `events` dabs no MESMO ponto de tela, com o pick resolvido contra `against`.
///
/// Devolve a profundidade depois de cada dab.
fn hold(against: PickAgainst, events: usize) -> Vec<f32> {
    hold_verb(against, events, Verb::Draw, Brush::default().accumulate)
}

/// O mesmo, com o VERBO e o Accumulate como parâmetros.
///
/// ⚠️ **A tabela 2×2 é a wave inteira**, e ela existe porque as duas colunas
/// não são independentes: o auto-limite da referência precisa que **exactamente
/// um** dos dois lados se mova.
fn hold_verb(against: PickAgainst, events: usize, verb: Verb, accumulate: bool) -> Vec<f32> {
    let mut mesh = uv_sphere(24, 36, 1.0);
    mesh.triangulate();
    mesh.rebuild();
    // A superfície CONGELADA no pen-down — o que o `!cache->accum` do Blender
    // consulta. Uma cópia, porque a viva vai ser deformada por baixo.
    let frozen = mesh.clone();

    let brush = Brush {
        verb,
        accumulate,
        falloff: Falloff::Smooth,
        radius: 0.25,
        strength: 0.5,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);

    let mut out = Vec::with_capacity(events);
    for _ in 0..events {
        // O ponteiro NÃO se move: o mesmo pixel, evento após evento.
        let ray = Ray::new([0.0, 0.0, 5.0], [0.0, 0.0, -1.0]);
        let hit = match against {
            PickAgainst::Live => mesh.raycast(&ray),
            PickAgainst::Frozen => frozen.raycast(&ray),
        };
        let Some(hit) = hit else { break };
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at(hit.point, brush.radius, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
        out.push(depth(&mesh));
    }
    out
}

/// **O que a CURA custa** — congelar a superfície do pick no pen-down.
///
/// A forma mais simples de o produto responder *"onde a superfície estava
/// quando o traço começou?"* é guardar uma cópia no pen-down. Isto mede o preço
/// dela, porque o §0 manda medir antes de escrever qualquer limite — e porque a
/// alternativa (testar triângulos com posições congeladas dentro do octree
/// VIVO) é mais barata em memória e mais cara em desenho.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_what_freezing_the_pick_surface_costs() {
    use std::time::Instant;
    println!("\n  o custo de UMA copia da malha, no pen-down:\n");
    for (rings, segs) in [(24usize, 36usize), (64, 96), (128, 192), (222, 333)] {
        let mut m = uv_sphere(rings, segs, 1.0);
        m.triangulate();
        m.rebuild();
        // Aquece: a primeira copia paga o first-touch das paginas.
        let _ = m.clone();
        let t = Instant::now();
        let c = m.clone();
        let ms = t.elapsed().as_secs_f64() * 1e3;
        // 12 B por posicao + 12 por normal -- o piso do que uma copia move.
        let mb = (c.vert_count() * 24) as f64 / (1024.0 * 1024.0);
        println!(
            "  {:>8} verts / {:>8} faces -> {:>7.3} ms, >= {:>6.2} MB",
            c.vert_count(),
            c.face_count(),
            ms,
            mb
        );
    }
    println!();
}

/// **A TABELA 2×2 — quem tem de se mover para o traço se auto-limitar.**
///
/// O peso de um dab é `f(|v − c| / r)`: `v` é a posição do vértice e `c` o
/// centro do dab. **Cada um deles pode ser lido VIVO ou CONGELADO**, e as duas
/// leituras são governadas por coisas diferentes:
///
/// - o `v` pelo `accumulate` do pincel (o `from_live` da [`ph2d_sculpt3d::GripLaw`],
///   que é o `vProxy` do `Brush.js:57` da referência);
/// - o `c` pela superfície contra a qual o PICK é resolvido — que hoje é sempre
///   a viva.
///
/// ⚠️ **O auto-limite precisa que EXACTAMENTE UM dos dois se mova.** Se os dois
/// sobem juntos, `|v − c|` fica constante e todo dab pesa o mesmo — o relevo
/// cresce para sempre. Se nenhum se move, idem. A saturação vive na DIAGONAL:
/// o vértice sai da pegada porque o centro ficou, ou porque ele andou.
///
/// *É por isso que congelar o pick não é uma cura que se possa aplicar sozinha:*
/// com o `accumulate` armado ela cura, com ele desarmado ela **destrói** o
/// auto-limite que já existe.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_which_of_the_two_has_to_move() {
    const N: usize = 40;
    println!("\n  40 dabs no MESMO ponto -- quem se move contra quem");
    println!("  (Draw, r=0.25, str=0.5, Smooth; relevo final e razao do ultimo incremento)\n");
    println!(
        "  {:<10} {:<12} {:>12} {:>14} {:>10}",
        "pick", "accumulate", "relevo(40)", "inc40/inc1", "veredito"
    );
    for against in [PickAgainst::Live, PickAgainst::Frozen] {
        for accum in [true, false] {
            let v = hold_verb(against, N, Verb::Draw, accum);
            if v.len() < N {
                continue;
            }
            let (i1, i40) = (v[0], v[N - 1] - v[N - 2]);
            let ratio = i40 / i1.max(1e-9);
            println!(
                "  {:<10} {:<12} {:>12.6} {:>14.4} {:>10}",
                format!("{against:?}"),
                accum,
                v[N - 1],
                ratio,
                if ratio < 0.5 { "satura" } else { "SEM TETO" }
            );
        }
    }

    // O CONTROLE que separa *a lei* de *o verbo*: o Clay da referência flatten-a
    // contra um plano, e um alvo que é um PLANO satura por construção -- ele não
    // depende da diagonal acima. É o que torna o `_clay = true` do `Brush.js:18`
    // load-bearing: a tool `Brush` do original NÃO é aditiva por default.
    println!("\n  CONTROLE -- o mesmo com Verb::Clay (alvo = PLANO, satura por construcao):");
    for accum in [true, false] {
        let v = hold_verb(PickAgainst::Live, N, Verb::Clay, accum);
        if v.len() < N {
            continue;
        }
        let (i1, i40) = (v[0], v[N - 1] - v[N - 2]);
        println!(
            "  {:<10} {:<12} {:>12.6} {:>14.4}",
            "Live",
            accum,
            v[N - 1],
            i40 / i1.max(1e-9)
        );
    }
    println!();
}

/// O desvio de guarda-chuva **sobre os vértices TOCADOS**: `(pior, p99)`.
///
/// ⚠️ **A primeira versão varria a malha INTEIRA e media a FIXTURE.** Os polos
/// de uma `uv_sphere` são vértices de valência alta cujo guarda-chuva vale
/// `0,3157` sem ninguém tocar neles, e o `max` pousava sempre lá: as seis linhas
/// da tabela saíam com o MESMO número, com e sem o fenómeno. *Uma régua que
/// inclui uma singularidade da fixture reporta a singularidade.*
///
/// Cópia local, irmã da do `measure_anchor_law`: as duas sondas são `#[ignore]`
/// e partilhar exigiria uma crate de teste que nenhuma paga.
fn umbrella_of(mesh: &Mesh, verts: &[u32]) -> (f32, f32) {
    let (pos, adj) = (mesh.positions(), mesh.adjacency());
    let mut all: Vec<f32> = Vec::with_capacity(verts.len());
    for &vi in verts {
        let i = vi as usize;
        let p = pos[i];
        let nb = adj.vert_verts.neighbours(i);
        if nb.len() < 3 {
            continue;
        }
        let (mut mid, mut len) = ([0.0f32; 3], 0.0f32);
        for &j in nb {
            let q = pos[j as usize];
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            len += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            for k in 0..3 {
                mid[k] += q[k];
            }
        }
        let n = nb.len() as f32;
        let d = [p[0] - mid[0] / n, p[1] - mid[1] / n, p[2] - mid[2] / n];
        let edge = len / n;
        if edge > 1e-9 {
            all.push((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() / edge);
        }
    }
    all.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    (
        all.last().copied().unwrap_or(0.0),
        all.get((all.len() as f32 * 0.99) as usize)
            .copied()
            .unwrap_or(0.0),
    )
}

/// Um traço do produto, com o pincel dado. Devolve `(dabs, relevo, pior, p99)`.
fn stroke_over(path: &[[f32; 2]], spacing: f32, brush: &Brush) -> (usize, f32, f32, f32) {
    let mut mesh = uv_sphere(48, 72, 1.0);
    mesh.triangulate();
    mesh.rebuild();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let mut anchor = path[0];
    let mut dabs = 0usize;
    for &p in &path[1..] {
        let Some(walk) = ph2d_sculpt3d::walk(anchor, p, spacing) else {
            continue;
        };
        let end = walk.anchor();
        for [sx, sy] in walk {
            let ray = Ray::new([sx, sy, 5.0], [0.0, 0.0, -1.0]);
            let Some(hit) = mesh.raycast(&ray) else { break };
            stroke.dab(
                &mut mesh,
                brush,
                &Dab::at(hit.point, brush.radius, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
            dabs += 1;
        }
        anchor = end;
    }
    let touched = stroke.touched().to_vec();
    let (worst, p99) = umbrella_of(&mesh, &touched);
    (dabs, depth(&mesh), worst, p99)
}

/// **O que o Accumulate faz a um TRAÇO** — a régua que casa com a foto.
///
/// A tabela acima mede o pincel PARADO, e o produto nunca o deixa parado: sem
/// deslocamento o `walk` não emite dab nenhum. O regime em que o defeito
/// alcança o artista é outro, e são dois:
///
/// - o traço **LENTO**, em que o espaçamento põe muitos dabs sobrepostos sobre
///   o mesmo vértice;
/// - o traço que **CRUZA A SI MESMO**, que é literalmente o que o checkbox
///   promete somar duas vezes.
///
/// O oráculo é o **desvio de guarda-chuva** — a distância de um vértice à média
/// dos vizinhos sobre a aresta local. Numa superfície lisa vale ~0,0x; numa
/// AGULHA vale ~1. É a régua da foto, e não a profundidade: um relevo alto e
/// LISO é o pincel a funcionar.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_what_accumulate_does_to_a_stroke() {
    // Um traco em X: a segunda perna cruza a primeira no centro.
    let cross: Vec<[f32; 2]> = (0..2)
        .flat_map(|leg| {
            (0..40).map(move |k| {
                let t = -0.5 + k as f32 / 39.0;
                if leg == 0 {
                    [t, t * 0.6]
                } else {
                    [t, -t * 0.6]
                }
            })
        })
        .collect();
    let line: Vec<[f32; 2]> = (0..80).map(|k| [-0.5 + k as f32 / 79.0, 0.0]).collect();

    for (name, path, spacing) in [
        ("reta LENTA  (spacing 0.02)", &line, 0.02f32),
        ("reta normal (spacing 0.10)", &line, 0.10),
        ("X que cruza (spacing 0.10)", &cross, 0.10),
    ] {
        println!("\n  {name}");
        println!(
            "  {:<12} {:>7} {:>12} {:>12} {:>12}",
            "accumulate", "dabs", "relevo", "umb pior", "umb p99"
        );
        for accum in [true, false] {
            let brush = Brush {
                verb: Verb::Draw,
                accumulate: accum,
                falloff: Falloff::Smooth,
                radius: 0.22,
                strength: 0.5,
                ..Brush::default()
            };
            let (dabs, relief, worst, p99) = stroke_over(path, spacing, &brush);
            println!(
                "  {:<12} {:>7} {:>12.6} {:>12.4} {:>12.4}",
                accum, dabs, relief, worst, p99
            );
        }
    }
    println!();
}

/// **O PINCEL DE FÁBRICA, e o que cada default que esta linha mexeu faz a ele.**
///
/// A pergunta do reporte — *"piorou"* — é sobre o pincel que o artista pega sem
/// tocar em nada. Esta varredura parte do `Brush::default()` de HOJE e devolve
/// **um default de cada vez** ao valor anterior, para atribuir a mudança a um
/// canal em vez de a uma wave inteira.
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_which_factory_default_moves_the_stroke() {
    let line: Vec<[f32; 2]> = (0..80).map(|k| [-0.5 + k as f32 / 79.0, 0.0]).collect();
    let base = Brush {
        radius: 0.22,
        ..Brush::default()
    };
    println!("\n  o pincel de FABRICA de hoje, e um canal de cada vez de volta");
    println!("  (Draw, r=0.22, reta de 1.0 unidade, spacing 0.10 -- o traco normal)\n");
    println!(
        "  {:<34} {:>7} {:>12} {:>12} {:>12}",
        "pincel", "dabs", "relevo", "umb pior", "umb p99"
    );
    let row = |label: &str, b: &Brush| {
        let (dabs, relief, worst, p99) = stroke_over(&line, 0.10, b);
        println!(
            "  {:<34} {:>7} {:>12.6} {:>12.4} {:>12.4}",
            label, dabs, relief, worst, p99
        );
    };
    row("FABRICA (hoje)", &base);
    row(
        "  accumulate = false",
        &Brush {
            accumulate: false,
            ..base.clone()
        },
    );
    row(
        "  falloff = Smooth (o antigo)",
        &Brush {
            falloff: Falloff::Smooth,
            ..base.clone()
        },
    );
    row(
        "  strength = 0.5 (o antigo)",
        &Brush {
            strength: 0.5,
            ..base.clone()
        },
    );
    row(
        "  auto_smooth = 0",
        &Brush {
            auto_smooth: 0.0,
            ..base.clone()
        },
    );
    println!("\n  forca de fabrica de hoje: {:.4}", base.strength);
    println!("  curva  de fabrica de hoje: {:?}", base.falloff);
    println!("  accum  de fabrica de hoje: {}", base.accumulate);
    println!("  auto_smooth de fabrica:    {:.4}", base.auto_smooth);
    println!();
}

/// **Segurar o pincel no mesmo ponto: satura ou acelera?**
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_whether_the_pick_sinks_into_its_own_crater() {
    const N: usize = 40;
    let live = hold(PickAgainst::Live, N);
    let frozen = hold(PickAgainst::Frozen, N);

    println!("\n  40 dabs no MESMO ponto (Draw, r=0.25, str=0.5, Smooth)");
    println!("  relevo = quanto o ponto mais deslocado saiu do raio unitario\n");
    println!(
        "  {:>4} {:>12} {:>12}  |  {:>12} {:>12}",
        "dab", "VIVA", "d(VIVA)", "CONGELADA", "d(CONG)"
    );
    for k in [0usize, 1, 2, 4, 9, 19, 29, 39] {
        let (a, b) = (live.get(k).copied(), frozen.get(k).copied());
        let (da, db) = (
            a.zip(k.checked_sub(1).and_then(|j| live.get(j).copied()))
                .map(|(x, y)| x - y),
            b.zip(k.checked_sub(1).and_then(|j| frozen.get(j).copied()))
                .map(|(x, y)| x - y),
        );
        let f = |v: Option<f32>| v.map_or("  --".into(), |x| format!("{x:.6}"));
        println!(
            "  {:>4} {:>12} {:>12}  |  {:>12} {:>12}",
            k + 1,
            f(a),
            f(da),
            f(b),
            f(db)
        );
    }

    // A leitura que decide: o INCREMENTO do ultimo dab contra o do primeiro.
    // Saturacao => o incremento ENCOLHE. Aceleracao (ou teimosia) => nao.
    let inc = |v: &[f32], k: usize| {
        if k == 0 { v[0] } else { v[k] - v[k - 1] }
    };
    if live.len() >= N && frozen.len() >= N {
        println!(
            "\n  incremento do 1o dab -> do 40o:\n    VIVA      {:.6} -> {:.6}  ({:.3}x)\n    CONGELADA {:.6} -> {:.6}  ({:.3}x)",
            inc(&live, 0),
            inc(&live, N - 1),
            inc(&live, N - 1) / inc(&live, 0).max(1e-9),
            inc(&frozen, 0),
            inc(&frozen, N - 1),
            inc(&frozen, N - 1) / inc(&frozen, 0).max(1e-9),
        );
        println!(
            "\n  profundidade final: VIVA {:.6}  CONGELADA {:.6}  ({:.2}x)",
            live[N - 1],
            frozen[N - 1],
            live[N - 1] / frozen[N - 1].max(1e-9)
        );
    }
    println!();
}
