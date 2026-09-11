//! **A curvatura do SSS pode sair de graça no FRAGMENT?** — a sonda que decide se
//! o `Mesh` ganha um plano por-vértice ou não.
//!
//! ```text
//! cargo test -p ph2d-mesh --release --test measure_curvature_estimators -- --ignored --nocapture
//! ```
//!
//! O estimador que a indústria usa quando **não** tem curvatura assada é
//! `length(fwidth(N)) / length(fwidth(P))` — derivadas de tela sobre a normal
//! interpolada. Ele custa **zero** na CPU: nenhum plano, nenhuma das quatro portas
//! (`rebuild` · `refresh_region` · `splice_topology` · `shrink_topology`), nenhum
//! byte no vértice. Se ele servir, a wave inteira do SSS fica muito menor.
//!
//! ⚠️ **Não se decide isso por raciocínio, e há um argumento fácil de errar:** numa
//! esfera lisa o estimador de tela é EXATO (mover um pixel muda a normal por
//! `Δs/R` e a posição por `Δs`, e a razão é `1/R` em qualquer ângulo de vista).
//! O que ele não é, é **CONTÍNUO**.
//!
//! # As duas perguntas que a sonda faz
//!
//! 1. **Quanto os dois erram** contra o `1/R` de uma esfera de raio conhecido.
//! 2. **Quanto o campo SALTA** ao atravessar uma aresta da malha. É aqui que os
//!    dois diferem por CONSTRUÇÃO: a curvatura por-vértice é interpolada, logo
//!    contínua; o `fwidth` da normal interpolada é **constante dentro de cada
//!    triângulo** (a normal é linear ali antes do `normalize`) e **degrau** na
//!    fronteira. Num app de ESCULTURA isso é o shader desenhando a topologia.
//!
//! # ⛔ A terceira diferença que eu ia alegar, e que a MATEMÁTICA refutou
//!
//! Eu ia escrever que `length(...)` é não-negativo, logo o estimador de tela
//! **não tem sinal**, logo ele não distingue uma narina de uma ponta de nariz —
//! e que isso o desqualificava.
//!
//! **É verdade sobre o estimador e IRRELEVANTE para esta LUT.** A pré-integração
//! de Penner é
//!
//! ```text
//! D(θ, r) = ∫ clamp(cos(θ + x), 0, 1) · R(2r·sin(x/2)) dx  /  ∫ R(2r·sin(x/2)) dx
//! ```
//!
//! e ela é **par em `x`** (a substituição `x → −x` leva o numerador nele mesmo,
//! inclusive com o clamp), enquanto `R` só depende de `|2r·sin(x/2)|`. Logo
//! `D(θ, r) = D(θ, |r|)`: **a difusão frontal não sabe o sinal da curvatura.**
//!
//! O sinal segue valendo — mas para o **Cavity**, que é outro canal e já o usa.
//! A sonda continua contando os côncavos porque o número é informação sobre a
//! fixture, não porque ele decide esta escolha.
//!
//! # A diferença que sobra, e que não é sobre número nenhum
//!
//! O canal por-vértice existe **na CPU**. O export, a doação ao 2D e um bake
//! futuro podem lê-lo; um `fwidth` existe apenas dentro de um fragment e não é
//! consultável por ninguém.

use ph2d_mesh::{Mesh, QueryScratch, RegionScratch, shapes};

/// A curvatura de MUNDO por-vértice, **pela porta do produto**.
///
/// ⚠️ A primeira versão desta sonda trazia uma cópia local da fórmula — e uma
/// sonda que reimplementa o que mede fica **cega à porta**: ela seguiria
/// imprimindo o número da cópia depois de o produto mudar de lei.
fn world_curvature_at(m: &Mesh, v: usize) -> f32 {
    ph2d_mesh::world_curvature_at(m.positions(), m.normals(), &m.adjacency().vert_verts, v)
}

/// O estimador de TELA, avaliado onde ele de fato vive: **dentro de um
/// triângulo**.
///
/// A normal interpolada é linear nos baricêntricos (antes do `normalize`), então
/// `|∂n/∂s|` é constante no triângulo e vale, ao longo de cada aresta,
/// `|n_b − n_a| / |p_b − p_a|`. Tomamos a média das três arestas — que é o que
/// `length(fwidth(N))/length(fwidth(P))` mede quando o quad de 2×2 pixels cai no
/// meio da face.
fn screen_style_curvature(m: &Mesh, tri: [u32; 3]) -> f32 {
    let mut sum = 0.0f32;
    for k in 0..3 {
        let (a, b) = (tri[k] as usize, tri[(k + 1) % 3] as usize);
        let (na, nb) = (m.normals()[a], m.normals()[b]);
        let (pa, pb) = (m.positions()[a], m.positions()[b]);
        let dn = [nb[0] - na[0], nb[1] - na[1], nb[2] - na[2]];
        let dp = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ln = (dn[0] * dn[0] + dn[1] * dn[1] + dn[2] * dn[2]).sqrt();
        let lp = (dp[0] * dp[0] + dp[1] * dp[1] + dp[2] * dp[2]).sqrt();
        if lp > f32::MIN_POSITIVE {
            sum += ln / lp;
        }
    }
    sum / 3.0
}

fn triangles(m: &Mesh) -> Vec<[u32; 3]> {
    m.faces()
        .iter()
        .filter(|f| f.is_tri())
        .map(|f| [f.0[0], f.0[1], f.0[2]])
        .collect()
}

fn median(mut v: Vec<f32>) -> f32 {
    v.sort_by(f32::total_cmp);
    v[v.len() / 2]
}

/// Uma esfera com sete toques — o que uma mão faz nos primeiros segundos, e a
/// única fixture em que o salto de faceta pode aparecer (numa esfera CRUA os
/// triângulos são todos iguais e os dois estimadores concordam por simetria).
fn sculpted(rings: usize, segs: usize) -> Mesh {
    let mut m = shapes::uv_sphere(rings, segs, 1.0);
    m.triangulate();
    let mut q = QueryScratch::default();
    let mut scratch = RegionScratch::default();
    let mut moved = Vec::new();
    for i in 0..7usize {
        let seed = (i * 7919) % m.vert_count();
        let center = m.positions()[seed];
        let radius = 0.10 + 0.06 * (i % 3) as f32;
        let push = if i % 2 == 0 { 0.06 } else { -0.05 };
        m.verts_in_sphere(center, radius, &mut q, &mut moved);
        let hits: Vec<u32> = moved.clone();
        for &v in &hits {
            let n = m.normals()[v as usize];
            let p = m.positions()[v as usize];
            let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
            let t = 1.0 - (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() / radius;
            let w = t.max(0.0).powi(2);
            let q = &mut m.positions_mut()[v as usize];
            q[0] += n[0] * push * w;
            q[1] += n[1] * push * w;
            q[2] += n[2] * push * w;
        }
        m.refresh_region(&hits, &mut scratch);
    }
    m
}

#[test]
#[ignore = "sonda de medição; roda sob demanda"]
fn measure_the_two_curvature_estimators() {
    println!("\n=== (1) EXATIDAO numa esfera de raio conhecido (48x72) ===");
    println!(
        "{:>8} | {:>16} | {:>16} | {:>8}",
        "raio R", "|kappa| vertice", "fwidth (face)", "1/R"
    );
    for r in [1.0f32, 2.0, 4.0] {
        let mut m = shapes::uv_sphere(48, 72, r);
        m.triangulate();
        let vert = median(
            (0..m.vert_count())
                .map(|v| world_curvature_at(&m, v).abs())
                .collect(),
        );
        let face = median(
            triangles(&m)
                .iter()
                .map(|t| screen_style_curvature(&m, *t))
                .collect(),
        );
        println!("{r:>8.1} | {vert:>16.4} | {face:>16.4} | {:>8.4}", 1.0 / r);
    }

    // (2) A DESCONTINUIDADE — e não o "salto", que foi a minha primeira tentativa
    // e não separa nada.
    //
    // ⚠️ Comparar *quanto o campo varia ao longo de uma aresta* nos dois
    // estimadores mede a GEOMETRIA (um vinco varia muito nos dois, e a primeira
    // versão desta sonda reportou 32,5 contra 29,0 — empate, e conclusão
    // nenhuma). A pergunta que os separa é outra: **quanto o campo pula em
    // distância ZERO** ao cruzar a aresta.
    //
    // Para o campo por-vértice a resposta é **exatamente 0 por construção** — no
    // ponto médio da aresta os dois triângulos interpolam o mesmo
    // `(κ_a + κ_b)/2`, então nem vale medir: vale AFIRMAR. Para o estimador de
    // tela é `|κ_face1 − κ_face2|`, porque ele é constante dentro de cada face.
    println!("\n=== (2) A DESCONTINUIDADE ao cruzar uma aresta (esfera esculpida 48x72) ===");
    let m = sculpted(48, 72);
    let tris = triangles(&m);
    let face_k: Vec<f32> = tris
        .iter()
        .map(|t| screen_style_curvature(&m, *t))
        .collect();
    let vert_k: Vec<f32> = (0..m.vert_count())
        .map(|v| world_curvature_at(&m, v))
        .collect();

    // Mapa aresta -> faces que a compartilham.
    let mut by_edge: std::collections::BTreeMap<(u32, u32), Vec<usize>> =
        std::collections::BTreeMap::new();
    for (fi, t) in tris.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            by_edge.entry((a.min(b), a.max(b))).or_default().push(fi);
        }
    }

    let mut face_gap = Vec::new();
    let mut vert_gap = Vec::new();
    for ((a, b), fs) in &by_edge {
        if fs.len() != 2 {
            continue;
        }
        face_gap.push((face_k[fs[0]] - face_k[fs[1]]).abs());
        // O campo interpolado, amostrado no meio da aresta pelos DOIS lados.
        let mid = 0.5 * (vert_k[*a as usize] + vert_k[*b as usize]);
        vert_gap.push((mid - mid).abs());
    }
    let scale = median(vert_k.iter().map(|k| k.abs()).collect()).max(1e-6);
    let pct = |x: f32| 100.0 * x / scale;
    let fg = median(face_gap.clone());
    let fg_max = face_gap.iter().copied().fold(0.0f32, f32::max);
    let vg_max = vert_gap.iter().copied().fold(0.0f32, f32::max);
    let loud = face_gap.iter().filter(|g| **g > scale).count();
    println!("  escala do campo (|kappa| mediano): {scale:.4}");
    println!("{:>28} | {:>10} | {:>10}", "", "MEDIANA", "MAXIMO");
    println!(
        "{:>28} | {fg:>10.4} | {fg_max:>10.4}   ({:.0}% / {:.0}% da escala)",
        "estimador de TELA (por face)",
        pct(fg),
        pct(fg_max)
    );
    println!(
        "{:>28} | {:>10.4} | {vg_max:>10.4}   (contínuo por construção)",
        "campo por VERTICE", 0.0
    );
    println!(
        "  arestas em que o degrau de tela passa a PROPRIA escala do campo: {loud} de {}",
        face_gap.len()
    );

    // (3) O SINAL. Estrutural, mas vale contar quantos vértices o estimador de
    // tela seria incapaz de distinguir.
    let concave = vert_k.iter().filter(|k| **k > 0.0).count();
    println!(
        "\n=== (3) O SINAL — e a REFUTACAO do meu proprio argumento ===\n  \
         {concave} de {} vertices sao CONCAVOS (kappa > 0), e o estimador de tela\n  \
         (`length`) nao os distingue dos convexos. ISSO NAO DECIDE NADA AQUI: a\n  \
         pre-integracao de Penner e' PAR em x, entao D(theta, r) = D(theta, |r|)\n  \
         e a difusao frontal nao sabe o sinal. Quem usa o sinal e' o Cavity.\n\n  \
         O que decide e' a secao (2) — e o fato de o canal por-vertice existir na\n  \
         CPU, onde o export e a doacao ao 2D podem le-lo.\n",
        vert_k.len()
    );
}
