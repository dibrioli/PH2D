//! **A sonda que decide o K2** (`docs/3D/03.5`): *recomputar as normais da
//! vizinhança tocada passa de 2 ms?*
//!
//! ```text
//! cargo test -p ph2d-mesh --release --test measure_normals -- --nocapture
//! ```
//!
//! Ela também responde a pergunta que a sonda do dab levantou: **o `rayon` está
//! ajudando?** O paralelismo rendeu só 1,35× num tier de 32 threads, e um número
//! desses ou é overhead de distribuição ou é limite de LARGURA DE BANDA. As duas
//! hipóteses levam a decisões opostas — a primeira diz "ajuste o limiar", a
//! segunda diz "a CPU não tem mais o que dar" —, então ela é medida, não
//! escolhida.

use std::time::Instant;

use ph2d_mesh::{Mesh, QueryScratch, RegionScratch, shapes};

const K2_BUDGET_MS: f64 = 2.0;

fn median(mut v: Vec<f64>) -> f64 {
    if v.len() > 1 {
        v.remove(0);
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Os vértices sob uma esfera de raio `frac` do modelo, num ponto da malha.
fn touched(mesh: &Mesh, frac: f32, seed: usize) -> Vec<u32> {
    let radius = mesh.bounds().longest_edge() * frac;
    let center = mesh.positions()[(seed * 7919) % mesh.vert_count()];
    let mut q = QueryScratch::default();
    let mut out = Vec::new();
    mesh.verts_in_sphere(center, radius, &mut q, &mut out);
    out
}

#[test]
fn measure_normals() {
    println!(
        "\n=== ph2d-mesh :: normais da região tocada (K2) — {} threads de rayon ===\n",
        rayon::current_num_threads()
    );
    println!(
        "{:>10} {:>7} {:>10} {:>10} {:>10} {:>10} {:>12}",
        "triangulos", "raio", "vertices", "total ms", "n.vertice", "n.face", "descobrir"
    );

    let mut worst = 0.0f64;
    let mut worst_desc = String::new();
    let mut per_vert = Vec::new();
    for target in [1_000_000usize, 5_000_000] {
        let mut mesh = shapes::sphere_with_triangles(target, 1.0);
        for frac in [0.02f32, 0.10, 0.30] {
            let mut scratch = RegionScratch::default();
            let mut samples = Vec::new();
            let mut verts = 0usize;
            for seed in 0..12 {
                let hits = touched(&mesh, frac, seed);
                verts = verts.max(hits.len());
                let t = Instant::now();
                mesh.refresh_region(&hits, &mut scratch);
                samples.push(t.elapsed().as_secs_f64() * 1e3);
            }
            let ms = median(samples);

            // Ablação pelas PORTAS DO PRODUTO: as duas metades de matemática do
            // `refresh_region` são funções públicas, então dá para cronometrá-las
            // sem instrumentar por dentro. O que sobra é DESCOBRIR a vizinhança
            // (os dois passes de dedup), que não tem porta própria — e é
            // exatamente por isso que ele precisa aparecer como resíduo.
            let hits = touched(&mesh, frac, 3);
            let faces: Vec<u32> = {
                let mut seen = vec![false; mesh.face_count()];
                let mut out = Vec::new();
                for &v in &hits {
                    for &f in mesh.adjacency().vert_faces.neighbours(v as usize) {
                        if !seen[f as usize] {
                            seen[f as usize] = true;
                            out.push(f);
                        }
                    }
                }
                out
            };
            let mut tmp = Vec::new();
            let mut vn = Vec::new();
            let mut fnm = Vec::new();
            for _ in 0..9 {
                let t = Instant::now();
                ph2d_mesh::vertex_normals_of(
                    mesh.face_normals(),
                    &mesh.adjacency().vert_faces,
                    &hits,
                    &mut tmp,
                );
                vn.push(t.elapsed().as_secs_f64() * 1e3);
                let t = Instant::now();
                ph2d_mesh::face_normals_of(mesh.positions(), mesh.faces(), &faces, &mut tmp);
                fnm.push(t.elapsed().as_secs_f64() * 1e3);
            }
            let (vn, fnm) = (median(vn), median(fnm));

            println!(
                "{:>10} {:>6.0}% {:>10} {:>10.3} {:>10.3} {:>10.3} {:>12.3}",
                mesh.triangle_count(),
                frac * 100.0,
                verts,
                ms,
                vn,
                fnm,
                (ms - vn - fnm).max(0.0)
            );
            per_vert.push(ms / verts as f64);
            if ms > worst {
                worst = ms;
                worst_desc = format!(
                    "{} triangulos, raio {:.0}%, {verts} vertices",
                    mesh.triangle_count(),
                    frac * 100.0
                );
            }
        }
    }

    println!("\nVEREDITO K2: pior {worst:.3} ms ({worst_desc}) contra o teto de {K2_BUDGET_MS} ms");
    if worst >= K2_BUDGET_MS {
        println!("  >> O K2 DISPARA neste regime. Decisão de arquitetura do Enio (docs/3D/03.5).");
    }

    let _ = per_vert;
}

/// ⚠️ **O gate é a FORMA, não o relógio — e a fixture teve de ser refeita para
/// perguntar isso de verdade.**
///
/// O valor absoluto de milissegundos é decisão de produto em aberto (que pincel
/// o artista usa em que densidade), e travar um teto que eu mesmo escrevi ontem,
/// sem dado, seria deixar um número não-medido governar. O que **não** é
/// negociável é a forma: o custo tem de ser função da PEGADA, não da malha.
///
/// ⚠️ A primeira versão media *custo por vértice tocado* com o raio fixo em
/// FRAÇÃO do modelo — e o número de vértices tocados cresce junto com a
/// densidade, então ela comparava footprints diferentes e reportava 8,1× de
/// variação com o diagnóstico errado (*"varredura de malha"*; era cache, porque
/// os vetores de dedup deixam de caber em L2). O experimento certo **fixa a
/// pegada** e varia só a malha: mesma quantidade de vértices tocados em malhas
/// de tamanhos diferentes.
#[test]
fn the_region_refresh_is_bound_by_the_footprint_not_by_the_mesh() {
    const TARGET_VERTS: usize = 30_000;
    let mut costs = Vec::new();
    for target in [500_000usize, 5_000_000] {
        let mut mesh = shapes::sphere_with_triangles(target, 1.0);
        // Acha o raio que toca ~TARGET_VERTS vértices NESTA densidade.
        let mut best = (f32::MAX, 0.02f32, 0usize);
        for k in 1..=60 {
            let frac = k as f32 * 0.005;
            let n = touched(&mesh, frac, 3).len();
            let err = (n as f32 - TARGET_VERTS as f32).abs();
            if err < best.0 {
                best = (err, frac, n);
            }
        }
        let (_, frac, n) = best;
        let mut scratch = RegionScratch::default();
        let mut samples = Vec::new();
        for seed in 0..12 {
            let hits = touched(&mesh, frac, seed);
            let t = Instant::now();
            mesh.refresh_region(&hits, &mut scratch);
            samples.push(t.elapsed().as_secs_f64() * 1e3);
        }
        let ms = median(samples);
        println!(
            "malha {:>8} triangulos · raio {:.1}% · {n} vertices tocados · {ms:.3} ms",
            mesh.triangle_count(),
            frac * 100.0
        );
        costs.push(ms);
    }
    let growth = costs[1] / costs[0];
    println!(
        "10x a malha, MESMA pegada: {growth:.2}x de custo\n\
         (>1 é esperado e é CACHE — os vetores de dedup crescem com a malha e \
         saem do L2; o que este gate recusa é crescimento PROPORCIONAL à malha)"
    );
    assert!(
        growth < 3.0,
        "10x a malha custou {growth:.2}x com a MESMA pegada — isso é assinatura \
         de varredura de malha, e é o que mata a malha grande"
    );
}

/// ⚠️ **A pergunta que o número de 1,35× levantou: o paralelismo rende, ou a
/// memória é o teto?** Esta sonda compara as DUAS portas do produto sobre o
/// MESMO conjunto — nenhuma delas é reimplementada aqui.
///
/// Se o ganho for próximo do número de threads, o kernel é limitado por CPU e o
/// caminho da CPU ainda tem folga. Se for próximo de 1, ele é limitado por
/// LARGURA DE BANDA — e aí acrescentar threads não muda nada, o que é um
/// argumento A FAVOR do device, que tem uma ordem de grandeza mais de banda.
#[test]
fn measure_normals_parallel_speedup() {
    let mesh = shapes::sphere_with_triangles(5_000_000, 1.0);
    let hits = touched(&mesh, 0.30, 3);
    let fnorm = mesh.face_normals();
    let ring = &mesh.adjacency().vert_faces;

    let mut out = Vec::new();
    let mut par = Vec::new();
    let mut ser = Vec::new();
    for _ in 0..9 {
        let t = Instant::now();
        ph2d_mesh::vertex_normals_of(fnorm, ring, &hits, &mut out);
        par.push(t.elapsed().as_secs_f64() * 1e3);

        // O caminho serial pela porta que o produto usa para lista esparsa.
        let mut normals = vec![[0.0f32; 3]; mesh.vert_count()];
        let t = Instant::now();
        ph2d_mesh::recompute_vertex_normals(fnorm, ring, &mut normals, Some(&hits));
        ser.push(t.elapsed().as_secs_f64() * 1e3);
    }
    let (p, s) = (median(par), median(ser));
    println!(
        "\n=== paralelismo das normais ({} vertices, {} threads) ===\n\
         serial {s:.3} ms · paralelo {p:.3} ms · ganho {:.2}x\n",
        hits.len(),
        rayon::current_num_threads(),
        s / p
    );

    // ⚠️ **Paridade BYTE-A-BYTE.** É ela que torna o paralelismo uma escolha de
    // velocidade e não de resultado: o `rayon` muda qual thread avalia qual
    // vértice, nunca a ordem da soma DENTRO de um vértice.
    let mut normals = vec![[0.0f32; 3]; mesh.vert_count()];
    ph2d_mesh::recompute_vertex_normals(fnorm, ring, &mut normals, Some(&hits));
    for (i, &v) in hits.iter().enumerate() {
        assert_eq!(
            normals[v as usize], out[i],
            "o caminho paralelo divergiu do serial no vértice {v}"
        );
    }

    // ⚠️ **E a metade que só um RELÓGIO pode ver — o gate que uma mutação
    // sobrevivente pediu.** Pôr `PAR_MIN = usize::MAX` torna o `rayon` código
    // morto, e a paridade acima fica **trivialmente verde**: os dois caminhos
    // passam a ser o mesmo. É a armadilha exata que a `line/Painter` documentou
    // no ADR-0120 (*"usar clone() no scratch tornaria o caminho rápido código
    // morto que nunca roda, com todos os gates verdes"*).
    //
    // O piso é 2× contra os 6,4× medidos: folga larga o bastante para uma
    // máquina carregada não silenciar o gate, apertada o bastante para que
    // "nunca paraleliza" (razão ~1,0) não passe.
    assert!(
        s / p > 2.0,
        "o caminho paralelo rendeu só {:.2}x sobre o serial — o `rayon` não está \
         de fato correndo (PAR_MIN alto demais? limiar invertido?)",
        s / p
    );
}
