//! **O QUE O CAMPO ELÁSTICO CUSTA** — a sonda que o report de FPS pede
//! (Enio, 2026-08-13: *"o resultado ficou muito bom do modo L mas com um pouco
//! de queda de FPS"*).
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_field_cost -- --ignored --nocapture
//! ```
//!
//! ⚠️ **`--release` não é preferência.** O que se mede aqui é aritmética
//! por-vértice; em debug ela mede o `opt-level=0`, não o produto.
//!
//! # Como esta sonda decide
//!
//! **Pela porta do produto** (`SculptStroke::dab`), e a decomposição sai de
//! chamar `verts_in_sphere`/`refresh_region` — que são as portas que o traço
//! usa — em vez de cronometrar de dentro dele. Uma sonda com laço próprio fica
//! CEGA à porta e segue reportando o custo antigo depois de o produto parar de
//! pagá-lo (a lição que a `line/Painter` pagou três vezes).
//!
//! ⚠️ **E o CONTROLE vem primeiro:** `s-mode` e `l-mode`, MESMO verbo, MESMO
//! raio de pincel, MESMA malha. Sem ele, um número absoluto não distingue *"o
//! campo é caro"* de *"esta malha é grande"* — e a pergunta do report é
//! exatamente essa diferença.
//!
//! ⚠️ **A consulta isolada usa o `query_radius` de CADA modo**, não o raio do
//! pincel: é isso que o `dab` faz, e medir os dois com o mesmo raio esconderia
//! justamente a variável que a §7.11 moveu.

use std::time::Instant;

use ph2d_mesh::{Mesh, QueryScratch, RegionScratch, shapes};
use ph2d_sculpt3d::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb, kelvinlet};

fn median(mut v: Vec<f64>) -> f64 {
    if v.len() > 1 {
        v.remove(0);
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn eye_towards(c: [f32; 3]) -> [f32; 3] {
    let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    if l > 1e-6 {
        [-c[0] / l, -c[1] / l, -c[2] / l]
    } else {
        [0.0, 0.0, -1.0]
    }
}

/// Um caminho de dabs sobre a superfície — o pincel ANDA, como a mão faz. Bater
/// no mesmo ponto mediria o cache quente daquela vizinhança.
fn stroke_centers(mesh: &Mesh, count: usize) -> Vec<[f32; 3]> {
    let n = mesh.vert_count();
    (0..count)
        .map(|i| mesh.positions()[(i * 7919) % n])
        .collect()
}

struct Row {
    mode: RefMode,
    radius_frac: f32,
    /// Quantos vértices a CONSULTA devolve — a pegada, o que o `query_radius`
    /// de fato pede à malha.
    footprint: usize,
    /// Quantos o dab MOVE. ⚠️ Não é a mesma pergunta, e confundi-las foi o 1º
    /// corte desta sonda: um verbo de carimbo tem curva que zera em `t = 1`, e
    /// aí os dois números coincidem; um CAMPO move tudo o que consulta.
    verts: usize,
    dab_ms: f64,
    query_ms: f64,
    normals_ms: f64,
}

impl Row {
    fn rest_ms(&self) -> f64 {
        (self.dab_ms - self.query_ms - self.normals_ms).max(0.0)
    }
}

fn measure(mesh: &mut Mesh, mode: RefMode, radius_frac: f32, dabs: usize) -> Row {
    let radius = mesh.bounds().longest_edge() * radius_frac;
    let centers = stroke_centers(mesh, dabs);
    // Força minúscula: a sonda mede CUSTO, e deformar a malha ao longo da
    // varredura mudaria a vizinhança de um dab para o seguinte.
    let brush = Brush {
        verb: Verb::Move,
        mode,
        radius,
        strength: 1e-4,
        ..Brush::default()
    };
    // A porta que decide o suporte. Para o `l-mode` ela devolve `3 · raio`.
    let query_r = brush.query_radius(radius);

    let mut stroke = SculptStroke::default();
    let mut full = Vec::with_capacity(dabs);
    let mut verts = 0usize;
    for c in &centers {
        stroke.begin(mesh);
        let dab = Dab::at(*c, radius, eye_towards(*c));
        let t = Instant::now();
        let moved = stroke.dab(mesh, &brush, &dab, Symmetry::default());
        full.push(t.elapsed().as_secs_f64() * 1e3);
        verts = verts.max(moved);
    }

    let mut q = QueryScratch::default();
    let mut hits = Vec::new();
    let mut query = Vec::with_capacity(dabs);
    let mut footprint = 0usize;
    for c in &centers {
        let t = Instant::now();
        mesh.verts_in_sphere(*c, query_r, &mut q, &mut hits);
        query.push(t.elapsed().as_secs_f64() * 1e3);
        footprint = footprint.max(hits.len());
    }

    let mut region = RegionScratch::default();
    let mut normals = Vec::with_capacity(dabs);
    for c in &centers {
        mesh.verts_in_sphere(*c, query_r, &mut q, &mut hits);
        let t = Instant::now();
        mesh.refresh_region(&hits, &mut region);
        normals.push(t.elapsed().as_secs_f64() * 1e3);
    }

    Row {
        mode,
        radius_frac,
        footprint,
        verts,
        dab_ms: median(full),
        query_ms: median(query),
        normals_ms: median(normals),
    }
}

#[test]
#[ignore = "sonda de medição; roda sob demanda com --ignored --nocapture"]
fn measure_what_the_l_mode_costs_against_its_own_control() {
    println!("\n=== o custo de um dab: s-mode (CONTROLE) contra l-mode ===\n");
    println!(
        "{:>6} {:>7} {:>10} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}",
        "modo", "raio", "pegada", "movidos", "dab ms", "consulta", "normais", "resto", "ns/vert"
    );

    // ⚠️ **A MALHA DO PRODUTO, e as duas razões são independentes.** A cena que
    // o artista abre é a `sculpt_sphere` — um cubo subdividido, ~196 k
    // triângulos —, e o 1º corte desta sonda usava uma `sphere_with_triangles`
    // de 1 M, que é uma **UV-sphere**: vértices amontoados nos polos. Ali a
    // pegada cresce LINEARMENTE com o raio (medido: 3,00x para 3x o raio),
    // porque um disco perto do polo engole anéis inteiros — e isso escondia a
    // lei que vale numa malha uniforme, onde a pegada é `r²` e o `l-mode` pede
    // **9x** os vértices, não 3x. *Uma fixture com densidade errada responde a
    // outra pergunta com confiança.*
    let mut mesh = shapes::sculpt_sphere(1.0);
    println!(
        "malha: {} triangulos, {} vertices (a que o modulo abre)\n",
        mesh.triangle_count(),
        mesh.vert_count()
    );
    let mut rows = Vec::new();
    for frac in [0.02f32, 0.10, 0.30] {
        for mode in [RefMode::S, RefMode::L] {
            let r = measure(&mut mesh, mode, frac, 12);
            println!(
                "{:>6?} {:>6.0}% {:>10} {:>9} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.1}",
                r.mode,
                r.radius_frac * 100.0,
                r.footprint,
                r.verts,
                r.dab_ms,
                r.query_ms,
                r.normals_ms,
                r.rest_ms(),
                r.dab_ms * 1e6 / r.footprint.max(1) as f64
            );
            rows.push(r);
        }
        println!();
    }

    println!("razao l/s, por fração de raio:");
    for frac in [0.02f32, 0.10, 0.30] {
        let s = rows
            .iter()
            .find(|r| r.mode == RefMode::S && r.radius_frac == frac)
            .expect("controle s-mode");
        let l = rows
            .iter()
            .find(|r| r.mode == RefMode::L && r.radius_frac == frac)
            .expect("l-mode");
        println!(
            "  raio {:>3.0}%  pegada {:>5.2}x  dab {:>5.2}x  consulta {:>5.2}x  normais {:>5.2}x  resto {:>5.2}x",
            frac * 100.0,
            l.footprint as f64 / s.footprint.max(1) as f64,
            l.dab_ms / s.dab_ms,
            l.query_ms / s.query_ms,
            l.normals_ms / s.normals_ms,
            l.rest_ms() / s.rest_ms().max(1e-9),
        );
    }
}

/// **De que o "resto" é feito** — o kernel por-vértice, isolado.
///
/// ⚠️ Isto NÃO é a porta do produto e a sonda diz isso: serve só para responder
/// *"quanto do por-vértice é a família de três escalas?"*, que é a única
/// pergunta cuja resposta o produto não consegue dar por ablação (não há knob
/// que troque a família).
#[test]
#[ignore = "sonda de medição; roda sob demanda com --ignored --nocapture"]
fn measure_what_a_field_evaluation_is_made_of() {
    const N: usize = 4_000_000;
    let f = [0.3f32, 0.2, 0.1];
    let eps = 0.25f32;

    let bench = |label: &str, mut body: Box<dyn FnMut(usize) -> f32>| {
        let t = Instant::now();
        let mut sink = 0.0f32;
        for i in 0..N {
            sink += body(i);
        }
        let ms = t.elapsed().as_secs_f64() * 1e3;
        println!(
            "  {label:<34} {ms:>8.2} ms   {:>6.2} ns/avaliacao   (sink {:.3})",
            ms * 1e6 / N as f64,
            sink
        );
        ms
    };

    // Pontos espalhados pela pegada, para não medir um raio só.
    let pt = |i: usize| {
        let t = i as f32 * 1e-6;
        [0.31 + t, -0.22 + t * 0.5, 0.17 - t * 0.3]
    };

    println!("\n=== de que uma avaliação de campo é feita ({N} avaliações) ===\n");
    let tri = bench(
        "grab, familia Tri (3 escalas)",
        Box::new(move |i| kelvinlet::grab(pt(i), eps, f, kelvinlet::Scales::Tri)[0]),
    );
    let bi = bench(
        "grab, familia Bi (2 escalas)",
        Box::new(move |i| kelvinlet::grab(pt(i), eps, f, kelvinlet::Scales::Bi)[0]),
    );
    let mono = bench(
        "grab, familia Mono (1 escala)",
        Box::new(move |i| kelvinlet::grab(pt(i), eps, f, kelvinlet::Scales::Mono)[0]),
    );
    let land = bench(
        "rim_landing (a janela da §7.13)",
        Box::new(move |i| kelvinlet::rim_landing(i as f32 * 2.5e-7)),
    );

    println!(
        "\n  Tri custa {:.2}x o Mono e {:.2}x o Bi.",
        tri / mono,
        tri / bi
    );
    println!(
        "  A JANELA custa {:.1}% de uma avaliação Tri — o controle da §7.13.",
        100.0 * land / tri
    );
}

/// **O QUE SE PERDE COALESCENDO OS DABS DE UM `Grip::Hold`** — a medição que
/// tem de vir ANTES da linha de código (Enio, 2026-08-13: *"ambos"*).
///
/// O `walk`, que espaça dabs por DISTÂNCIA percorrida, cobre só
/// `Grip::Stamp | Paint`. O `Grip::Hold` — o Move/Grab, o modo cujo FPS foi
/// reportado — faz **um dab por EVENTO de ponteiro**: a ~1000 Hz de mouse e
/// 60 fps são ~16 dabs por quadro.
///
/// ⚠️ **O argumento para coalescer já está escrito no braço vizinho do
/// `sculpt3d_input.rs`**, no `Grip::Turn`: *"daria N dabs com o mesmo total
/// acumulado no mesmo lugar — trabalho idêntico repetido, porque o alvo é
/// função do `pre` congelado e do gesto TOTAL"*. O `Hold` tem a mesma lei.
///
/// ⚠️ **E a diferença honesta que esta sonda mede:** a PEGADA é consultada nas
/// posições **VIVAS**, então descartar dabs intermediários pode mudar *quais*
/// vértices entram no conjunto `touched` — que só cresce. Se o barro final
/// diverge, coalescer é decisão de produto; se não diverge, é trabalho repetido
/// a menos.
#[test]
#[ignore = "sonda de medição; roda sob demanda com --ignored --nocapture"]
fn measure_what_coalescing_a_hold_gesture_changes() {
    let radius_frac = 0.10f32;
    // O puxão, em fração do raio do pincel: um gesto curto (dentro da pegada) e
    // um longo (que arrasta barro para fora dela) medem coisas diferentes.
    for pull_frac in [0.5f32, 1.0, 2.0] {
        println!("\n=== puxão de {:.0}% do raio ===", pull_frac * 100.0);
        println!(
            "{:>8} {:>10} {:>12} {:>14} {:>12}",
            "eventos", "movidos", "dab total ms", "desvio max", "desvio rel"
        );

        let mut reference: Option<Vec<[f32; 3]>> = None;
        for events in [16usize, 8, 4, 2, 1] {
            let mut mesh = shapes::sculpt_sphere(1.0);
            let radius = mesh.bounds().longest_edge() * radius_frac;
            let pull = radius * pull_frac;
            let start = mesh.positions()[0];
            let eye = eye_towards(start);
            let brush = Brush {
                verb: Verb::Move,
                mode: RefMode::L,
                radius,
                strength: 1.0,
                ..Brush::default()
            };

            let mut stroke = SculptStroke::default();
            stroke.begin(&mesh);
            let t = Instant::now();
            let mut moved = 0usize;
            for k in 1..=events {
                // O ponteiro anda em passos iguais até o MESMO destino: é a
                // mesma mão, amostrada mais fino ou mais grosso.
                let f = k as f32 / events as f32;
                let c = [start[0] + pull * f, start[1], start[2]];
                let dab = Dab::at(c, radius, eye);
                moved = moved.max(stroke.dab(&mut mesh, &brush, &dab, Symmetry::default()));
            }
            let ms = t.elapsed().as_secs_f64() * 1e3;

            let now = mesh.positions().to_vec();
            let (dev, rel) = match &reference {
                // O CONTROLE é a rota de 16 eventos — o que o app faz hoje.
                None => (0.0f32, 0.0f32),
                Some(r) => {
                    let d = r
                        .iter()
                        .zip(&now)
                        .map(|(a, b)| {
                            let dx = a[0] - b[0];
                            let dy = a[1] - b[1];
                            let dz = a[2] - b[2];
                            (dx * dx + dy * dy + dz * dz).sqrt()
                        })
                        .fold(0.0f32, f32::max);
                    (d, d / pull)
                }
            };
            println!(
                "{events:>8} {moved:>10} {ms:>12.3} {dev:>14.6} {:>11.2}%",
                rel * 100.0
            );
            if reference.is_none() {
                reference = Some(now);
            }
        }
    }
    println!(
        "\n⚠️  O desvio é medido contra a rota de 16 eventos — o que o produto\n\
           faz hoje. `desvio rel` é a fração do PUXÃO que o artista pediu."
    );
}
