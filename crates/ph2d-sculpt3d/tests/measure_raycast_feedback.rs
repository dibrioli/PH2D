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
    let mut mesh = uv_sphere(24, 36, 1.0);
    mesh.triangulate();
    mesh.rebuild();
    // A superfície CONGELADA no pen-down — o que o `!cache->accum` do Blender
    // consulta. Uma cópia, porque a viva vai ser deformada por baixo.
    let frozen = mesh.clone();

    let brush = Brush {
        verb: Verb::Draw,
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
