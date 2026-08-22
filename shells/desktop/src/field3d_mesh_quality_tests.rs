//! ⚠️ **A sonda da QUALIDADE da malha exportada** — não "saiu alguma coisa", mas *que* coisa saiu.
//!
//! ⭐ O smoke da W19 aprovou a exportação e **reprovou a malha**: em *shade smooth* no Blender
//! aparecem manchas escuras num reticulado regular, e de perto são triângulos que se sobrepõem.
//! Um relatório de artista (*"baixa qualidade, sobreposição de faces"*) não é um mecanismo — esta
//! sonda existe para o transformar num, porque **o remédio depende de qual defeito é**:
//!
//! | defeito | o que se vê | a cura |
//! |---|---|---|
//! | vértice do QEF fora da célula | faces dobradas, normal invertida | prender o vértice à célula |
//! | triângulo degenerado (área ~0) | mancha, normal indefinida | soldar/colapsar |
//! | aresta não-manifold | buraco, sombra dupla | topologia — o remesh não come isto |
//! | vértice fora da superfície | silhueta errada | projetar de volta pelo campo |
//!
//! ⚠️ **Um remesh de quads a jusante NÃO cura nenhuma das quatro** — ele *herda* a entrada. É por
//! isso que esta medição vem antes de qualquer decisão sobre esperar por ele.
//!
//! Módulo-filho: `use super::*` traz as fixtures do pai.

use super::*;

/// O que a sonda conta numa malha, tudo em unidades da **célula** do octree.
#[derive(Default)]
struct Report {
    tris: usize,
    verts: usize,
    /// Triângulos com área **exatamente** zero — normal indefinida.
    zero_area: usize,
    /// Qualidade de forma `q = 4√3·A / Σℓ²`: 1 = equilátero, 0 = degenerado.
    q_min: f64,
    q_lt_02: usize,
    q_lt_10: usize,
    /// Faces cuja normal **discorda do gradiente do campo** — a face está virada do avesso.
    inverted: usize,
    /// Área média das invertidas, em unidades de **célula²** — separa a lasca invisível da dobra.
    inv_area_mean: f64,
    /// A maior delas.
    inv_area_max: f64,
    /// Invertidas com área ≥ 5 % de uma célula² — as que a tela de facto mostra.
    inv_big: usize,
    /// A maior aresta de uma invertida, em células — um vértice que **fugiu da célula** a estica.
    inv_edge_max: f64,
    /// O pior alinhamento `n̂ · ĝ` encontrado.
    cos_worst: f64,
    /// Arestas com incidência ≠ 2 (borda ou não-manifold).
    bad_edges: usize,
    /// Vértices exatamente coincidentes (posição repetida).
    dup_verts: usize,
    /// Maior `|f|` no centróide, em frações de célula — quão longe da superfície a malha passa.
    off_surface: f64,
    /// **Onde** mora a pior invertida: raio ao eixo Y e altura. Um defeito no eixo de um torno e um
    /// espalhado pela parede pedem waves diferentes, e uma contagem não os distingue.
    worst_at: [f64; 2],
    /// O mesmo para o triângulo de pior forma.
    worst_q_at: [f64; 2],
}

/// `∇f` por diferença central.
fn grad(field: &ph2d_field_eval::Field, p: [f64; 3], eps: f64) -> [f64; 3] {
    let d = |i: usize| {
        let mut a = p;
        let mut b = p;
        a[i] += eps;
        b[i] -= eps;
        (field.at(a[0], a[1], a[2]) - field.at(b[0], b[1], b[2])) / (2.0 * eps)
    };
    [d(0), d(1), d(2)]
}

/// A normal contra a qual uma face é julgada: a **média das normais nos três vértices**.
///
/// ⚠️ **É a mesma regra do gate `the_exported_mesh_never_folds_a_face`**, e é de propósito que seja
/// uma só. Duas outras foram tentadas e as duas reprovam geometria correta: `∇f` no **baricentro**
/// (que numa parede fina cai dentro do material e aponta para a face do outro lado) e `∇f` na
/// superfície mais próxima do baricentro (que numa **quina de 90°** pousa numa das duas faces,
/// enquanto o quad que atravessa a quina tem a normal *entre* elas). Os vértices estão sobre a
/// superfície por construção, e a média deles é justamente essa direção do meio.
fn face_normal_oracle(field: &ph2d_field_eval::Field, v: &[[f64; 3]], eps: f64) -> [f64; 3] {
    let mut g = [0.0f64; 3];
    for p in v {
        let gv = grad(field, *p, eps);
        let l = (gv[0] * gv[0] + gv[1] * gv[1] + gv[2] * gv[2]).sqrt();
        if l > 0.0 && l.is_finite() {
            for k in 0..3 {
                g[k] += gv[k] / l;
            }
        }
    }
    g
}

fn measure(mesh: &ph2d_mesh::Mesh, field: &ph2d_field_eval::Field, cell: f64) -> Report {
    let pos = mesh.positions();
    let mut r = Report {
        tris: mesh.faces().len(),
        verts: pos.len(),
        q_min: f64::INFINITY,
        cos_worst: f64::INFINITY,
        ..Report::default()
    };

    // Vértices coincidentes: chave em bits, que é a única igualdade honesta para f32.
    let mut seen = std::collections::BTreeSet::new();
    for p in pos {
        if !seen.insert([p[0].to_bits(), p[1].to_bits(), p[2].to_bits()]) {
            r.dup_verts += 1;
        }
    }

    // Incidência por aresta não-dirigida.
    let mut inc: std::collections::BTreeMap<(u32, u32), u32> = std::collections::BTreeMap::new();
    let mut tri = Vec::new();
    for f in mesh.faces() {
        tri.clear();
        f.triangles(&mut tri);
        for t in &tri {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                *inc.entry((a.min(b), a.max(b))).or_default() += 1;
            }
        }
    }
    r.bad_edges = inc.values().filter(|&&n| n != 2).count();

    let eps = cell / 8.0;
    for f in mesh.faces() {
        tri.clear();
        f.triangles(&mut tri);
        for t in &tri {
            let v: Vec<[f64; 3]> = t
                .iter()
                .map(|&i| {
                    let p = pos[i as usize];
                    [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]
                })
                .collect();
            let e1 = [v[1][0] - v[0][0], v[1][1] - v[0][1], v[1][2] - v[0][2]];
            let e2 = [v[2][0] - v[0][0], v[2][1] - v[0][1], v[2][2] - v[0][2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let two_a = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if two_a == 0.0 {
                r.zero_area += 1;
                r.q_min = 0.0;
                continue;
            }
            let sum_l2: f64 = (0..3)
                .map(|k| {
                    let a = v[k];
                    let b = v[(k + 1) % 3];
                    (b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)
                })
                .sum();
            let q = 4.0 * 3.0f64.sqrt() * (two_a / 2.0) / sum_l2;
            if q < r.q_min {
                r.q_min = q;
                r.worst_q_at = [(v[0][0].powi(2) + v[0][2].powi(2)).sqrt(), v[0][1]];
            }
            if q < 0.02 {
                r.q_lt_02 += 1;
            }
            if q < 0.10 {
                r.q_lt_10 += 1;
            }

            let c = [
                (v[0][0] + v[1][0] + v[2][0]) / 3.0,
                (v[0][1] + v[1][1] + v[2][1]) / 3.0,
                (v[0][2] + v[1][2] + v[2][2]) / 3.0,
            ];
            r.off_surface = r.off_surface.max(field.at(c[0], c[1], c[2]).abs() / cell);
            let g = face_normal_oracle(field, &v, eps);
            let gl = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
            if gl > 0.0 {
                let cos = (n[0] * g[0] + n[1] * g[1] + n[2] * g[2]) / (two_a * gl);
                if cos < r.cos_worst {
                    r.cos_worst = cos;
                    r.worst_at = [(c[0].powi(2) + c[2].powi(2)).sqrt(), c[1]];
                }
                if cos < 0.0 {
                    r.inverted += 1;
                    let area = (two_a / 2.0) / (cell * cell);
                    r.inv_area_mean += area;
                    r.inv_area_max = r.inv_area_max.max(area);
                    if area >= 0.05 {
                        r.inv_big += 1;
                    }
                    r.inv_edge_max = r.inv_edge_max.max(sum_l2.sqrt() / cell);
                }
            }
        }
    }
    if r.inverted > 0 {
        r.inv_area_mean /= r.inverted as f64;
    }
    r
}

/// ⭐ **O relatório de qualidade da malha, por profundidade e por extrator** — o mecanismo por trás
/// do *"sobreposição de faces"* do smoke da W19, e a prova de que o extrator da casa o apaga.
#[test]
#[ignore = "medição, não gate — corre com --ignored --nocapture"]
fn measure_export_mesh_quality() {
    println!(
        "cena | prof |    tris |   vért | quads | área0 | q<0,10 |   q_min | invert |     % | \
         grandes | ℓ_máx | cos_pior | (r,y) da pior | arestas | dups | fora(cél) | (r,y) do pior q |      ms"
    );
    for n in [1u32, 2, 4, 5] {
        let doc = scene(n);
        let field = ph2d_field_eval::Field::new(&doc);
        for depth in [5u8, 6, 7, 8] {
            // A grade cobre [-1, 1] em cada eixo: a célula é 2 / 2^prof.
            let cell = 2.0 / f64::from(1u32 << depth);
            let t0 = std::time::Instant::now();
            let m = ph2d_field_eval::extract::extract(
                &doc,
                &crate::field3d_smoke::sampled_registry(),
                depth,
            )
            .expect("a cena malha");
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let quads = m.faces().iter().filter(|f| !f.is_tri()).count();
            let r = measure(&m, &field, cell);
            println!(
                "{n:4} | {depth:4} | {:7} | {:6} | {:5.0} | {:5} | {:6} | {:7.4} | {:6} | {:5.1} \
                 | {:7} | {:5.2} | {:8.3} | ({:5.2},{:6.2}) | {:7} | {:4} | {:9.3} | ({:5.2},{:6.2}) | {ms:7.1}",
                r.tris,
                r.verts,
                100.0 * quads as f64 / m.faces().len().max(1) as f64,
                r.zero_area,
                r.q_lt_10,
                r.q_min,
                r.inverted,
                100.0 * r.inverted as f64 / r.tris.max(1) as f64,
                r.inv_big,
                r.inv_edge_max,
                r.cos_worst,
                r.worst_at[0],
                r.worst_at[1],
                r.bad_edges,
                r.dup_verts,
                r.off_surface,
                r.worst_q_at[0],
                r.worst_q_at[1]
            );
            let _ = r.q_lt_02;
            let _ = r.inv_area_mean;
            let _ = r.inv_area_max;
        }
    }
}

/// ⚠️ **A sonda que decide a W5**: *uma escultura pode entrar na booleana do campo?*
///
/// ⭐ **O plano dizia "malha esculpida → campo (via `ph2d-sdf`) → entra na booleana", e metade disso
/// está BLOQUEADA por uma propriedade do motor.** A `fidget::context::TreeOp` é uma álgebra
/// **fechada** — `Input(Var)` · `Const` · `Binary` · `Unary` · remapeamentos — e **não tem operação
/// de consulta a dados**. Um campo em voxels não vira um nó de árvore, e portanto não entra no
/// `compile` de hoje.
///
/// Sobram três caminhos, e esta sonda mede os números que escolhem entre eles:
///
/// | caminho | o que custa |
/// |---|---|
/// | a malha vira EXPRESSÃO (um termo por triângulo) | árvore de ~10 nós por triângulo — a sonda mede quantos triângulos uma escultura tem |
/// | a booleana acontece na MALHA | ⛔ é exatamente o que falha, e é a tese do módulo que não falha |
/// | ⭐ avaliador **híbrido**: folha analítica (JIT) ou folha **amostrada**, e a booleana é `min/max` nos dois | precisa que amostrar não seja muito mais caro que avaliar |
///
/// A pergunta decisiva é a última linha: **quanto custa uma amostra trilinear contra uma avaliação
/// da árvore com JIT** — porque o traçado avalia milhões por quadro, em lote.
#[test]
#[ignore = "medição, não gate — corre com --ignored --nocapture"]
fn measure_sculpt_to_field_bridge() {
    println!("--- voxelizar uma malha (ph2d_sdf::VoxelField)");
    println!("triângulos |  res | células | MB | ms (voxelizar+flood)");
    for tris in [2_000usize, 20_000, 100_000] {
        let mesh = ph2d_mesh::shapes::sphere_with_triangles(tris, 0.6);
        for res in [64u32, 128, 256] {
            let t0 = std::time::Instant::now();
            let mut f = ph2d_sdf::VoxelField::for_bounds(mesh.bounds(), res);
            f.voxelize(&mesh);
            f.flood_fill();
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            // 4 B de distância + 3 B de travessia por célula.
            let mb = f.cell_count() as f64 * 7.0 / (1024.0 * 1024.0);
            println!(
                "{:10} | {res:4} | {:7} | {mb:5.1} | {ms:8.1}",
                mesh.faces().len(),
                f.cell_count()
            );
        }
    }

    println!("\n--- o custo POR PONTO: amostra trilinear contra a árvore com JIT");
    let mesh = ph2d_mesh::shapes::sphere_with_triangles(100_000, 0.6);
    let mut vf = ph2d_sdf::VoxelField::for_bounds(mesh.bounds(), 256);
    vf.voxelize(&mesh);
    vf.flood_fill();

    let n = 1_000_000usize;
    // Um caminho determinístico que atravessa a caixa — sem `rand`, e reprodutível.
    let pts: Vec<[f32; 3]> = (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            [
                (t * 37.0).sin() * 0.7,
                (t * 41.0).cos() * 0.7,
                (t * 43.0).sin() * 0.7,
            ]
        })
        .collect();

    let t0 = std::time::Instant::now();
    let mut acc = 0.0f32;
    for p in &pts {
        acc += vf.sample(*p);
    }
    let ms_sample = t0.elapsed().as_secs_f64() * 1000.0;

    let doc = scene(1);
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let mut batch = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
    let (xs, ys, zs): (Vec<f32>, Vec<f32>, Vec<f32>) = (
        pts.iter().map(|p| p[0]).collect(),
        pts.iter().map(|p| p[1]).collect(),
        pts.iter().map(|p| p[2]).collect(),
    );
    let t0 = std::time::Instant::now();
    let acc2: f32 = batch.eval(&xs, &ys, &zs).expect("lote").iter().sum();
    let ms_tree = t0.elapsed().as_secs_f64() * 1000.0;

    println!("amostra trilinear : {ms_sample:8.1} ms por 1 M pontos  ({acc:.3})");
    println!("árvore com JIT    : {ms_tree:8.1} ms por 1 M pontos  ({acc2:.3})");
    println!("razão             : {:8.2}x", ms_sample / ms_tree);
}
