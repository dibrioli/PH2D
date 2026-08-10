//! **O que acontece quando o remesh é aplicado à própria saída, várias vezes.**
//!
//! `cargo test -p ph2d-sdf --release --test probe_repeat_remesh -- --ignored --nocapture --test-threads=1`
//!
//! Report do Enio (smoke da cena `=6`, 2026-08-10): *"após algumas vezes fazendo
//! remesh a 512, a esfera sumiu do nada"*. A sequência é o fenômeno — um remesh
//! só sai bem —, então a fixture tem de ser a sequência, não um remesh isolado.
//!
//! ⚠️ **A malha esticada não é decoração.** A cena `=6` é uma esfera PUXADA por
//! um snake hook, e `for_bounds` divide o MAIOR lado: uma peça comprida gasta
//! um passo MAIOR na mesma resolução, então ela é mais BARATA que a redonda —
//! quem chega primeiro a qualquer teto é a peça de lados iguais.

use ph2d_mesh::{Mesh, shapes};
use ph2d_sdf::remesh;

/// A memória residente do processo, em MB. `statm` é páginas; a segunda coluna
/// é a residente.
fn rss_mb() -> f64 {
    let s = std::fs::read_to_string("/proc/self/statm").unwrap_or_default();
    let pages: f64 = s
        .split_whitespace()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    pages * 4096.0 / (1024.0 * 1024.0)
}

/// Uma esfera esticada — a proporção da cena `=6` sem o motor de escultura
/// (esta crate não depende dele, e um bico escrito à mão seria uma segunda
/// resposta a *"o que um SnakeHook faz"*).
fn stretched(sy: f32) -> Mesh {
    let m = shapes::uv_sphere(48, 72, 1.0);
    let mut p = m.positions().to_vec();
    for v in &mut p {
        v[1] *= sy;
    }
    Mesh::from_parts(p, m.faces().to_vec()).unwrap()
}

fn walk(label: &str, mut mesh: Mesh, res: u32, rounds: usize) {
    eprintln!("\n=== {label} @ res {res} ===");
    let b = mesh.bounds();
    eprintln!(
        "entrada: {:>8} v / {:>8} f | caixa {:.3} x {:.3} x {:.3}",
        mesh.vert_count(),
        mesh.face_count(),
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    );
    for round in 1..=rounds {
        match remesh(&mesh, res) {
            Ok((out, report)) => {
                let b = out.bounds();
                let mb = report.cells as f64 * 7.0 / (1024.0 * 1024.0);
                eprintln!(
                    "  #{round}: {:>9} células ({mb:>6.1} MB) -> {:>8} v / {:>8} f | caixa {:.3} x {:.3} x {:.3}",
                    report.cells,
                    out.vert_count(),
                    out.face_count(),
                    b.max[0] - b.min[0],
                    b.max[1] - b.min[1],
                    b.max[2] - b.min[2],
                );
                if out.vert_count() == 0 {
                    eprintln!("  #{round}: >>> MALHA VAZIA com Ok <<<");
                    return;
                }
                mesh = out;
            }
            Err(e) => {
                eprintln!("  #{round}: RECUSA -- {e}");
                return;
            }
        }
    }
}

#[test]
#[ignore = "sonda de medição"]
fn what_repeating_a_remesh_does() {
    walk("esfera", shapes::uv_sphere(48, 72, 1.0), 512, 5);
    walk("esticada 3x", stretched(3.0), 512, 5);
}

/// Uma esfera com um BICO fino — a feature da cena `=6` que uma esfera lisa não
/// tem. O bico é o candidato natural a vazamento: ele é fino, e o campo só sabe
/// segurar a onda onde a superfície de fato atravessa uma aresta da grade.
fn spiked(len: f32, sharp: f32) -> Mesh {
    let m = shapes::uv_sphere(48, 72, 1.0);
    let mut p = m.positions().to_vec();
    for v in &mut p {
        // Quanto mais perto do polo +y, mais o vértice é puxado para fora e
        // apertado contra o eixo — um cone que sai da esfera.
        let t = ((v[1] - sharp).max(0.0) / (1.0 - sharp)).clamp(0.0, 1.0);
        let pull = t * t;
        v[1] += pull * len;
        v[0] *= 1.0 - 0.92 * pull;
        v[2] *= 1.0 - 0.92 * pull;
    }
    Mesh::from_parts(p, m.faces().to_vec()).unwrap()
}

/// **A malha COLAPSA em alguma resolução?** — o vazamento PARCIAL.
///
/// O guard do motor recusa `inside == 0`. Um vazamento que deixe POUCAS células
/// dentro passa por ele: o remesh devolve `Ok` com um caco, o shell o instala, e
/// a peça some da tela com log de sucesso. Esta sonda varre uma banda de
/// resoluções e procura a contagem que sai fora da tendência.
#[test]
#[ignore = "sonda de medição"]
fn does_any_resolution_collapse_the_mesh() {
    for (label, mesh) in [
        ("esfera", shapes::uv_sphere(48, 72, 1.0)),
        ("bico fino", spiked(1.6, 0.55)),
    ] {
        eprintln!("\n=== {label} ===");
        let mut worst: Option<(u32, usize, f64)> = None;
        for res in 480u32..=512 {
            match remesh(&mesh, res) {
                Ok((out, _)) => {
                    let v = out.vert_count();
                    // A contagem esperada cresce com o quadrado da resolução (é
                    // uma ÁREA de superfície dividida pelo passo ao quadrado).
                    let expect = f64::from(res).powi(2);
                    let ratio = v as f64 / expect;
                    if worst.is_none_or(|(_, _, w)| ratio < w) {
                        worst = Some((res, v, ratio));
                    }
                    if ratio < 1.0 {
                        eprintln!("  res {res}: {v} v -- COLAPSO (razão {ratio:.3})");
                    }
                }
                Err(e) => eprintln!("  res {res}: RECUSA -- {e}"),
            }
        }
        if let Some((res, v, ratio)) = worst {
            eprintln!("  menor razão: res {res} -> {v} v (razão {ratio:.3})");
        }
    }
}

/// **O que a HISTÓRIA cobra por remesh.**
///
/// A pilha de undo do escultor guarda `StrokeUndo::Remeshed(Box<Mesh>)` — a
/// malha ANTERIOR inteira — e não tem teto nenhum. Esta sonda mede o que cada
/// entrada dessas custa de verdade, pela residência do processo, que é o que o
/// app paga (a soma dos `Vec` públicos ignora adjacência e octree).
#[test]
#[ignore = "sonda de medição"]
fn what_the_history_costs_per_remesh() {
    for res in [150u32, 512] {
        let base = shapes::uv_sphere(48, 72, 1.0);
        let (out, _) = remesh(&base, res).expect("a esfera não recusa");
        eprintln!(
            "\n=== resolução {res}: malha de {} v / {} f ===",
            out.vert_count(),
            out.face_count()
        );

        eprintln!(
            "  footprint_bytes diz {:.1} MB",
            out.footprint_bytes() as f64 / (1024.0 * 1024.0)
        );

        let mut held: Vec<Box<Mesh>> = Vec::new();
        let start = rss_mb();
        for k in 1..=5 {
            let (m, _) = remesh(&base, res).expect("a esfera não recusa");
            held.push(Box::new(m));
            let now = rss_mb();
            eprintln!(
                "  {k} malha(s) na história: RSS {now:>8.1} MB (+{:>7.1} desde o início, {:>6.1} por malha)",
                now - start,
                (now - start) / f64::from(k),
            );
        }
        drop(held);
    }
}
