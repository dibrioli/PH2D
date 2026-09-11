//! **A LEI DA ÂNCORA quando um passo ERRA — e o que ela faz à superfície.**
//!
//! O shell caminha o gesto em passos de espaçamento e carimba cada um por um
//! PICK. Um passo pode não achar superfície (a mão sai do modelo o tempo todo),
//! e aí três leis são possíveis para onde a âncora vai:
//!
//! - `Pointer` — a âncora salta para o PONTEIRO (o `SculptBase.js:151-152` do
//!   SculptGL, pareado com o `break` de `:141-146`).
//! - `WalkEnd` — a âncora vai para o fim do WALK (o que este repo shipou até
//!   2026-08-16).
//! - `LastLanded` — a âncora fica no último dab APLICADO (o que eu escrevi em
//!   2026-08-16, e que esta sonda existe para julgar).
//!
//! ⚠️ **A diferença só existe DEPOIS de um miss**, e é por isso que nenhuma
//! fixture de traço limpo a separa: sem miss as três leis dão o MESMO número.
//! O gesto que as separa é o que atravessa a SILHUETA — sair do modelo e
//! voltar —, e é exactamente o que um escultor faz o tempo todo.
//!
//! O oráculo é o **desvio de guarda-chuva** (a distância de um vértice à média
//! dos vizinhos, dividida pela aresta local): numa superfície lisa vale ~0,0x e
//! numa AGULHA vale ~1. É o irmão do `measure_dyntopo_spikes`, e a razão de ser
//! uma RAZÃO por aresta é a mesma: uma distância absoluta encolheria sozinha
//! quando as arestas encolhem.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --test measure_anchor_law --release
//! -- --ignored --nocapture`

use ph2d_mesh::{Mesh, Ray, shapes::uv_sphere};
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};

/// A lei que decide para onde a âncora vai quando o evento acaba.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AnchorLaw {
    /// SculptGL: o resíduo é DESCARTADO, a âncora é o ponteiro.
    Pointer,
    /// O que este repo shipou até 2026-08-16.
    WalkEnd,
    /// O que eu escrevi em 2026-08-16.
    LastLanded,
}

/// O desvio de guarda-chuva: `(pior, p99, média)`.
///
/// Cópia local de propósito — o irmão `measure_dyntopo_spikes` tem a sua, e as
/// duas sondas são `#[ignore]`: partilhar exigiria uma crate de teste que
/// nenhuma das duas paga.
fn umbrella(mesh: &Mesh) -> (f32, f32, f32) {
    let pos = mesh.positions();
    let adj = mesh.adjacency();
    let mut all: Vec<f32> = Vec::with_capacity(pos.len());
    for (i, p) in pos.iter().enumerate() {
        let nb = adj.vert_verts.neighbours(i);
        if nb.len() < 3 {
            continue;
        }
        let mut mid = [0.0f32; 3];
        let mut len = 0.0f32;
        for &j in nb {
            let q = pos[j as usize];
            for k in 0..3 {
                mid[k] += q[k];
            }
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            len += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        }
        let n = nb.len() as f32;
        let d = [p[0] - mid[0] / n, p[1] - mid[1] / n, p[2] - mid[2] / n];
        let dev = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let edge = len / n;
        if edge > 1e-9 {
            all.push(dev / edge);
        }
    }
    all.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    (
        all.last().copied().unwrap_or(0.0),
        all.get((all.len() as f32 * 0.99) as usize)
            .copied()
            .unwrap_or(0.0),
        all.iter().sum::<f32>() / all.len().max(1) as f32,
    )
}

/// O que a sonda observa de uma corrida.
struct Run {
    dabs: usize,
    /// Quantos passos do walk foram PEDIDOS (carimbados ou não).
    steps: usize,
    /// O maior número de passos que UM evento pediu.
    worst_burst: usize,
    umbrella: (f32, f32, f32),
}

/// O gesto do produto, com a lei da âncora como PARÂMETRO.
///
/// A câmera é ortográfica olhando por `-Z`; o "pixel" é a coordenada de mundo
/// em `x`/`y`, então a silhueta da esfera unitária está em `|p| = 1` e um
/// ponteiro em `x > 1` **erra** — que é o fenómeno inteiro.
fn sweep(law: AnchorLaw, events: &[[f32; 2]], spacing: f32) -> Run {
    let mut mesh = uv_sphere(24, 36, 1.0);
    mesh.triangulate();
    mesh.rebuild();

    let brush = Brush {
        verb: Verb::Draw,
        falloff: Falloff::Smooth,
        radius: 0.22,
        strength: 0.5,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);

    let mut anchor = events[0];
    let (mut dabs, mut steps, mut worst_burst) = (0usize, 0usize, 0usize);

    for &p in &events[1..] {
        let Some(walk) = ph2d_sculpt3d::walk(anchor, p, spacing) else {
            continue; // carry: o resíduo acumula, a âncora não anda
        };
        let end = walk.anchor();
        let mut landed: Option<[f32; 2]> = None;
        let mut burst = 0usize;
        for [sx, sy] in walk {
            burst += 1;
            steps += 1;
            // O pick do produto: raio ao longo de -Z a partir de fora.
            let ray = Ray::new([sx, sy, 5.0], [0.0, 0.0, -1.0]);
            let Some(hit) = mesh.raycast(&ray) else {
                break; // o `break` do SculptBase.js:141-146
            };
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::at(hit.point, brush.radius, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
            dabs += 1;
            landed = Some([sx, sy]);
        }
        worst_burst = worst_burst.max(burst);
        anchor = match law {
            AnchorLaw::Pointer => p,
            AnchorLaw::WalkEnd => end,
            AnchorLaw::LastLanded => landed.unwrap_or(anchor),
        };
    }

    Run {
        dabs,
        steps,
        worst_burst,
        umbrella: umbrella(&mesh),
    }
}

/// **O gesto que atravessa a silhueta, com as três leis lado a lado.**
#[test]
#[ignore = "sonda: roda sob demanda"]
fn measure_what_the_anchor_law_does_when_a_step_misses() {
    // Um vaivém que SAI do modelo pela direita e volta: `x = 1.4` está fora da
    // esfera unitária, logo o pick erra ali.
    let mut events: Vec<[f32; 2]> = Vec::new();
    for k in 0..60 {
        let t = k as f32 / 59.0;
        // -0.6 .. 1.4 .. -0.6 (uma ida e uma volta atravessando a silhueta)
        let x = if t < 0.5 {
            -0.6 + 4.0 * t
        } else {
            1.4 - 4.0 * (t - 0.5)
        };
        events.push([x, 0.15]);
    }
    let spacing = 0.05f32;

    println!("\n  gesto: 60 eventos, vaivem atravessando a silhueta (x ate 1.4)");
    println!("  esfera 24x36 unitaria, pincel r=0.22 str=0.5, spacing {spacing}\n");
    println!(
        "  {:<12} {:>6} {:>7} {:>7} {:>10} {:>10} {:>10}",
        "lei", "dabs", "passos", "rajada", "umb pior", "umb p99", "umb media"
    );
    for law in [
        AnchorLaw::Pointer,
        AnchorLaw::WalkEnd,
        AnchorLaw::LastLanded,
    ] {
        let r = sweep(law, &events, spacing);
        println!(
            "  {:<12} {:>6} {:>7} {:>7} {:>10.4} {:>10.4} {:>10.4}",
            format!("{law:?}"),
            r.dabs,
            r.steps,
            r.worst_burst,
            r.umbrella.0,
            r.umbrella.1,
            r.umbrella.2
        );
    }

    // O CONTROLE: o mesmo gesto que NUNCA sai do modelo.
    //
    // ⚠️ **Sem miss coincidem DUAS das tres, nao as tres** -- e a primeira
    // versao deste comentario afirmava as tres, o que a propria corrida negou
    // (18 contra 22). `WalkEnd` e `LastLanded` TEM de dar o mesmo numero, porque
    // sem miss o fim do walk E o ultimo dab aplicado sao o mesmo passo; o
    // `Pointer` diverge por DESENHO, e nao por defeito -- ele descarta o residuo
    // a cada evento, entao emite menos dabs mesmo sem um so miss.
    //
    // *Um controle que afirma demais reprova produto correto*, e o que ele tem
    // de discriminar aqui e' so' a metade que a lei do miss decide.
    let inside: Vec<[f32; 2]> = (0..60)
        .map(|k| {
            let t = k as f32 / 59.0;
            let x = if t < 0.5 {
                -0.6 + 1.2 * t
            } else {
                0.0 - 1.2 * (t - 0.5)
            };
            [x, 0.15]
        })
        .collect();
    // ⚠️ **A RAJADA só nasce com o ponteiro RÁPIDO, e a minha primeira fixture
    // não continha o fenómeno.** Com 60 eventos densos cada um anda ~1 passo,
    // então a ancora atrasada nunca fica longe e as tres leis empatam. O gesto
    // que as separa é o mouse a saltar: poucos eventos, cada um andando MUITO.
    // *Uma lei sobre o que acontece depois de um miss só é medível numa fixture
    // em que o ponteiro viaja enquanto está fora.*
    for events_n in [8usize, 5, 3] {
        let fast: Vec<[f32; 2]> = (0..events_n)
            .map(|k| {
                let t = k as f32 / (events_n - 1) as f32;
                let x = if t < 0.5 {
                    -0.6 + 4.0 * t
                } else {
                    1.4 - 4.0 * (t - 0.5)
                };
                [x, 0.15]
            })
            .collect();
        println!("\n  MOUSE RAPIDO -- o MESMO caminho em {events_n} eventos:");
        println!(
            "  {:<12} {:>6} {:>7} {:>7} {:>10} {:>10} {:>10}",
            "lei", "dabs", "passos", "rajada", "umb pior", "umb p99", "umb media"
        );
        for law in [
            AnchorLaw::Pointer,
            AnchorLaw::WalkEnd,
            AnchorLaw::LastLanded,
        ] {
            let r = sweep(law, &fast, spacing);
            println!(
                "  {:<12} {:>6} {:>7} {:>7} {:>10.4} {:>10.4} {:>10.4}",
                format!("{law:?}"),
                r.dabs,
                r.steps,
                r.worst_burst,
                r.umbrella.0,
                r.umbrella.1,
                r.umbrella.2
            );
        }
    }

    println!("\n  CONTROLE (o mesmo gesto SEM sair do modelo -- as tres tem de coincidir):");
    println!(
        "  {:<12} {:>6} {:>7} {:>7} {:>10} {:>10} {:>10}",
        "lei", "dabs", "passos", "rajada", "umb pior", "umb p99", "umb media"
    );
    for law in [
        AnchorLaw::Pointer,
        AnchorLaw::WalkEnd,
        AnchorLaw::LastLanded,
    ] {
        let r = sweep(law, &inside, spacing);
        println!(
            "  {:<12} {:>6} {:>7} {:>7} {:>10.4} {:>10.4} {:>10.4}",
            format!("{law:?}"),
            r.dabs,
            r.steps,
            r.worst_burst,
            r.umbrella.0,
            r.umbrella.1,
            r.umbrella.2
        );
    }
    println!();
}
