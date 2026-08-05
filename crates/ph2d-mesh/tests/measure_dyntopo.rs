//! **A sonda que decide o desenho da W9** — quanto custa MUDAR a topologia.
//!
//! O plano diz que a topologia dinâmica precisa de dois pré-requisitos que ela
//! *"não pode herdar"*: anéis mutáveis e um octree de folhas mutáveis. Isso é
//! uma afirmação sobre um NÚMERO — *reconstruir é caro demais para caber num
//! dab* —, e o §0 do `CLAUDE.md` manda medir antes de escrever o limite.
//!
//! As três perguntas, nesta ordem:
//!
//! 1. **Reconstruir cabe num dab?** O K1 é 8 ms; se `Adjacency::build` +
//!    `Octree::build` couberem nele na malha que um artista de fato esculpe, a
//!    estrutura mutável é otimização prematura e o dyntopo pode nascer sobre o
//!    `rebuild` que já existe.
//! 2. **Qual é o TAMANHO do trabalho local?** Um split toca a vizinhança do
//!    dab, não a malha — a razão entre as duas é o ganho que a estrutura
//!    mutável compra, e ela é o argumento inteiro.
//! 3. **Onde está o joelho?** A malha em que reconstruir passa do orçamento é
//!    o número que decide se a W9 é uma wave ou três.
//!
//! ```text
//! cargo test -p ph2d-mesh --release --test measure_dyntopo -- --ignored --nocapture
//! ```

use ph2d_mesh::{Adjacency, Octree, QueryScratch, edge_target, refine_in_sphere, shapes};
use std::time::Instant;

fn ms(f: impl FnOnce()) -> f64 {
    let t = Instant::now();
    f();
    t.elapsed().as_secs_f64() * 1000.0
}

/// A mediana de `n` corridas — a primeira aquece o alocador, e uma média sobre
/// amostra pequena é refém dela.
fn ms_median(n: usize, mut f: impl FnMut()) -> f64 {
    let mut v: Vec<f64> = (0..n).map(|_| ms(&mut f)).collect();
    v.sort_by(f64::total_cmp);
    v[n / 2]
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_what_a_topology_change_costs_today() {
    println!("\n  RECONSTRUIR — o preço de mudar a topologia sem estrutura mutável");
    println!("   vértices    faces      anéis ms   octree ms   normais ms    TOTAL ms");
    for (rings, segs) in [(16, 24), (32, 48), (64, 96), (128, 192), (256, 384)] {
        let mesh = shapes::uv_sphere(rings, segs, 1.0);
        let (v, f) = (mesh.vert_count(), mesh.face_count());
        let faces = mesh.faces().to_vec();
        let pos = mesh.positions().to_vec();

        let adj = ms_median(5, || {
            std::hint::black_box(Adjacency::build(v, &faces));
        });
        let oct = ms_median(5, || {
            std::hint::black_box(Octree::build(&pos, &faces));
        });
        // O `rebuild` inteiro é o que um dab pagaria hoje para acrescentar UM
        // vértice: ele refaz anéis, octree, normais de face e de vértice.
        let mut m2 = mesh.clone();
        let total = ms_median(5, || m2.rebuild());
        let normals = (total - adj - oct).max(0.0);

        println!("  {v:9}  {f:9}   {adj:9.3}   {oct:9.3}   {normals:10.3}   {total:9.3}");
    }
    println!(
        "\n  ⚠️ O K1 do ADR-0150 e' 8 ms POR DAB, e um traco emite dezenas deles.\n     \
         Reconstruir cabe no orcamento so' enquanto a malha for pequena -- e a\n     \
         topologia dinamica existe precisamente para a malha DEIXAR de ser."
    );
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_how_local_a_dab_really_is() {
    println!("\n  O TRABALHO LOCAL — quantas faces um dab de fato alcanca");
    println!("   vértices     faces    raio    faces no dab   fracao   razao global/local");
    let mut scratch = QueryScratch::default();
    for (rings, segs) in [(32, 48), (64, 96), (128, 192), (256, 384)] {
        let mesh = shapes::uv_sphere(rings, segs, 1.0);
        let (v, f) = (mesh.vert_count(), mesh.face_count());
        for radius in [0.10f32, 0.25] {
            let mut hits = Vec::new();
            mesh.octree()
                .faces_in_sphere([0.0, 0.0, 1.0], radius, &mut hits);
            let frac = hits.len() as f64 / f as f64;
            println!(
                "  {v:9} {f:9}   {radius:5.2}   {:12}   {:6.2}%   {:14.0}x",
                hits.len(),
                frac * 100.0,
                1.0 / frac.max(1e-9)
            );
        }
        // O mesmo caminho que o produto usa (a consulta de vértices, que o
        // pincel chama por dab) — para a sonda medir a porta, não um atalho.
        let mut verts = Vec::new();
        mesh.verts_in_sphere([0.0, 0.0, 1.0], 0.25, &mut scratch, &mut verts);
    }
    println!(
        "\n  ⚠️ Esta razao E' o argumento da estrutura mutavel: e' quanto trabalho\n     \
         um `rebuild` faz a mais do que a mudanca de fato pede."
    );
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_where_the_rebuild_leaves_the_budget() {
    println!("\n  O JOELHO — em que malha reconstruir passa do orcamento de um dab");
    println!("   vértices     faces     rebuild ms   cabe em 8 ms?");
    for (rings, segs) in [
        (16, 24),
        (32, 48),
        (48, 72),
        (64, 96),
        (96, 144),
        (128, 192),
    ] {
        let mesh = shapes::uv_sphere(rings, segs, 1.0);
        let (v, f) = (mesh.vert_count(), mesh.face_count());
        let mut m = mesh.clone();
        let t = ms_median(5, || m.rebuild());
        let verdict = if t <= 8.0 { "sim" } else { "NAO" };
        println!("  {v:9} {f:9}    {t:10.3}   {verdict}");
    }
}

/// **O QUE UM DAB DE FATO PAGA HOJE** — o `refine_in_sphere` inteiro, não o
/// `rebuild` sozinho.
///
/// ⚠️ **Esta sonda existe porque a 2ª rodada da W9.1 MOVEU o número que decide a
/// W9.2** (`CLAUDE.md` §0: *quem move o número que tornava algo inalcançável tem
/// de reconferir a nota*). O refino deixou de ser um passe de corte: ele é agora
/// até três passes de corte **mais** até três rodadas de flip, e **cada uma
/// termina numa troca de topologia** — ou seja, o custo por dab é um múltiplo do
/// `rebuild`, não um `rebuild`.
///
/// As duas colunas separam os dois regimes que o artista de fato encontra:
///
/// - **1º dab** — a malha ainda está grossa ali, então o corte roda até o teto
///   de passes. É o pior caso, e ele acontece uma vez por região.
/// - **em regime** — a região já tem a densidade pedida, o corte não acha nada e
///   o refino sai por `Enough`. É o que domina um traço.
#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_what_a_dab_pays_for_refinement_today() {
    println!("\n  O DAB INTEIRO — `refine_in_sphere`, corte + flip, não só o `rebuild`");
    println!("   vértices    1o dab ms   em regime ms   cabe em 8 ms?");
    let radius = 0.25f32;
    let centre = [0.0, 1.0, 0.0];
    for (rings, segs) in [(16, 24), (32, 48), (64, 96), (128, 192), (256, 384)] {
        let mut m = shapes::uv_sphere(rings, segs, 1.0);
        m.triangulate();
        m.rebuild();
        let v = m.vert_count();
        // ⚠️ **O alvo acompanha a MALHA, e sem isso a sonda não contém o
        // fenômeno:** com um alvo fixo, uma esfera fina já está abaixo dele e o
        // refino sai por `Enough` — a coluna do "1º dab" MEDIA MENOS na malha
        // grande (0,140 ms a 6k contra 1,034 a 1,5k), que é o oposto do que a
        // W9.2 precisa saber. Aqui ele é uma fração da aresta local, então o
        // refino de fato ACONTECE em toda linha.
        let target = 0.6 * mean_edge(&m);
        let mut births = Vec::new();
        let first = ms(|| {
            refine_in_sphere(&mut m, centre, radius, target, &mut births);
        });
        // Agora a região já está na densidade pedida: é o que um traço paga a
        // cada evento depois do primeiro.
        let steady = ms_median(5, || {
            refine_in_sphere(&mut m, centre, radius, target, &mut births);
        });
        let verdict = if first <= 8.0 { "sim" } else { "NAO" };
        println!("  {v:9}   {first:9.3}   {steady:12.3}   {verdict}");
    }
}

/// O comprimento médio de aresta da malha — a régua que faz a sonda do dab
/// refinar de verdade em toda densidade.
fn mean_edge(m: &ph2d_mesh::Mesh) -> f32 {
    let pos = m.positions();
    let mut tris = Vec::new();
    m.triangle_indices(&mut tris);
    let mut sum = 0.0f32;
    for t in &tris {
        for k in 0..3 {
            let (a, b) = (pos[t[k] as usize], pos[t[(k + 1) % 3] as usize]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            sum += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        }
    }
    sum / (tris.len() * 3).max(1) as f32
}
