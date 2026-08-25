//! ⭐⭐⭐ **A SONDA DO GATE Nº7 — as restrições ficam ESPARSAS, e a régua traz a
//! contagem de SINGULARIDADES ao lado** (`SPEC_restricoes_por_eliminacao.md` §5).
//!
//! ```text
//! cargo run --release -p ph2d-quadextract --example feature_sweep -- <peça|ficheiro.obj>
//! ```
//!
//! ⛔⛔ **Ela vive aqui, e não na `ph2d-mesh`, por uma razão MEDIDA:** a lei tem de ser
//! medida sobre a malha que o pipeline de facto alimenta — a **remalhada pelo F1**, cujas
//! arestas são uniformes e da ordem de `h`. Medida sobre um cilindro analítico cru, a
//! aresta média saía `0,68` numa peça de raio `0,5` (as arestas verticais são a altura
//! inteira), a vizinhança de raio `r₀` engolia a peça toda, e a anisotropia da parede —
//! que é `1,0` por construção — lia-se `0,00`. *Uma lei medida sobre a malha errada
//! devolve zero e lê-se como «a lei não funciona».*
//!
//! ⛔⛔ **O que ela mede NÃO é «quantas feições achámos».** A espec é explícita: marcar
//! feição a mais é pior que a menos, **porque cada restrição força uma singularidade** —
//! e é por isso que a coluna `SING` está aqui. ⭐ *Sem ela, a varredura não tem como
//! distinguir «apanhou o vinco» de «encheu a peça de restrições duvidosas»: as duas
//! sobem a percentagem de marcados exactamente da mesma maneira.*
//!
//! ⚠️ **A linha `BASE` é o controlo** — o mesmo campo sem restrição nenhuma. Uma
//! contagem de singularidades só é boa ou má **contra ela**.

fn edge_mean(m: &ph2d_mesh::Mesh) -> f32 {
    let pos = m.positions();
    let (mut s, mut n) = (0.0f32, 0usize);
    for f in m.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            s += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
            n += 1;
        }
    }
    s / n.max(1) as f32
}

fn load(name: &str) -> ph2d_mesh::Mesh {
    let mut mesh = if name.ends_with(".obj") {
        let text = std::fs::read_to_string(name).unwrap_or_else(|e| panic!("{name}: {e}"));
        ph2d_mesh::import_obj(&text)
            .unwrap_or_else(|e| panic!("{name} nao e' um OBJ deste leitor: {e:?}"))
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{name} nao tem peca dentro"))
            .mesh
    } else {
        match name {
            "cilindro" => ph2d_mesh::shapes::cylinder(64, 0.5, 1.5),
            "toro" => ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35),
            _ => ph2d_mesh::shapes::uv_sphere(48, 72, 1.0),
        }
    };
    mesh.triangulate();
    // ⛔ A FASE ZERO, como no `chain_info`: é ela que torna as arestas uniformes, e a
    // lei da feição é medida em múltiplos delas.
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    mesh
}

/// Quantas singularidades o campo planta, com as restrições que lhe derem — **e quantas
/// resoluções saíram sem convergir**.
///
/// ⛔ **A segunda coluna não é enfeite:** o doc do [`ph2d_crossfield::SolveReport`] regista
/// que uma malha mais fina passou de `8` para `194` singularidades **por teto de CG**, e não
/// por defeito de algoritmo. *Sem ela, uma restrição que só torna o sistema mais duro de
/// resolver lê-se como uma restrição errada.*
fn singular_count(mesh: &ph2d_mesh::Mesh, dual: &ph2d_crossfield::Dual) -> (usize, usize) {
    let (field, rep) = ph2d_crossfield::solve_miq(dual);
    (
        ph2d_crossfield::singularities(mesh, dual, &field).0,
        rep.cg_capped,
    )
}

/// ⭐⭐⭐ **O CONTROLO QUE SEPARA «a feição está errada» de «a eliminação está errada».**
///
/// Ele restringe as MESMAS faces à direcção que o campo **livre** já tinha ali. A resposta
/// certa é conhecida de antemão: forçar alguém a fazer o que ele já ia fazer não pode mudar
/// nada. ⇒ se a contagem explodir **aqui**, a causa não é a lei da feição — é a maquinaria
/// que elimina a variável.
fn control_pin_to_itself(
    mesh: &ph2d_mesh::Mesh,
    base: &ph2d_crossfield::Dual,
    edges: &[ph2d_mesh::FeatureEdge],
) -> (usize, usize) {
    let (field, _) = ph2d_crossfield::solve_miq(base);
    let mut owner: std::collections::BTreeMap<[u32; 2], [f32; 3]> =
        std::collections::BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            let key = [a.min(b), a.max(b)];
            if edges.binary_search_by(|e| e.verts.cmp(&key)).is_ok() {
                owner
                    .entry(key)
                    .or_insert_with(|| field.direction(base, fi));
            }
        }
    }
    let same: Vec<ph2d_mesh::FeatureEdge> = edges
        .iter()
        .filter_map(|e| {
            owner.get(&e.verts).map(|d| ph2d_mesh::FeatureEdge {
                verts: e.verts,
                dir: *d,
            })
        })
        .collect();
    let mut dual = base.clone();
    dual.constrain(mesh, &same);
    // ⭐ Quatro cantos: livre/preso × com/sem alinhamento. A ENERGIA ao lado da contagem
    // separa «o solver foi para outro sítio» de «o mesmo campo lido de outra maneira».
    for (rotulo, d) in [("livre", base), ("preso", &dual)] {
        for w in [ph2d_crossfield::ALIGN_WEIGHT, 0.0] {
            let (field, rep) =
                ph2d_crossfield::solve_miq_aligned(d, ph2d_crossfield::Rounding::default(), w);
            let (sing, _) = ph2d_crossfield::singularities(mesh, d, &field);
            println!(
                "     {rotulo:>5} align {w:.2}: {sing:>4} sing · energia {:>12.4} · \
                 {} inteiros livres · {} CG no teto · {} recentragens",
                ph2d_crossfield::energy(d, &field),
                rep.free_integers,
                rep.cg_capped,
                rep.recentres
            );
        }
    }
    singular_count(mesh, &dual)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| String::from("esfera"));
    let mesh = load(&name);
    let h = edge_mean(&mesh);
    let base = ph2d_crossfield::Dual::build(&mesh);
    let (base_sing, base_cap) = singular_count(&mesh, &base);
    println!(
        "{name}: {} vertices, {} faces (F1 honrado) · aresta media {h:.4}",
        mesh.positions().len(),
        mesh.face_count()
    );
    println!("  BASE (sem restricao nenhuma): {base_sing} singularidades, {base_cap} CG no teto");
    println!(
        "{:>6} {:>6} {:>8} {:>8} | {:>8} {:>7} {:>9} {:>7} | {:>6} {:>6} | {:>6} {:>6}",
        "r1/h",
        "anisot",
        "janela",
        "min_cos",
        "vert",
        "jan",
        "ARESTAS",
        "%peca",
        "faces",
        "confl",
        "SING",
        "capCG"
    );
    {
        let opts = ph2d_mesh::FeatureOptions {
            r1_in_h: 2.0,
            min_anisotropy: 0.92,
            ..ph2d_mesh::FeatureOptions::default()
        };
        let (dirs, _) = ph2d_mesh::feature_dirs(&mesh, h, opts);
        let (edges, _) = ph2d_mesh::feature_edges(&mesh, &dirs, 0.966);
        let (s, c) = control_pin_to_itself(&mesh, &base, &edges);
        println!(
            "  CONTROLO (as MESMAS {} arestas, fixas ao que o campo livre ja' fazia): \
             {s} singularidades, {c} CG no teto  <- tem de dar {base_sing}",
            edges.len()
        );
    }
    for r1 in [2.0f32, 4.0] {
        for a in [0.85f32, 0.92, 0.96] {
            for hw in [1.0f32, 2.0] {
                let opts = ph2d_mesh::FeatureOptions {
                    r0_in_edges: 1.0,
                    r1_in_h: r1,
                    samples: 6,
                    half_window_in_h: hw,
                    min_anisotropy: a,
                    min_curvature_in_bbox: 0.05,
                };
                let (dirs, vr) = ph2d_mesh::feature_dirs(&mesh, h, opts);
                for min_cos in [0.866f32, 0.966] {
                    let (edges, er) = ph2d_mesh::feature_edges(&mesh, &dirs, min_cos);
                    let mut dual = base.clone();
                    let cr = dual.constrain(&mesh, &edges);
                    let (sing, cap) = singular_count(&mesh, &dual);
                    println!(
                        "{r1:>6.1} {a:>8.2} {hw:>8.2} {min_cos:>8.3} | {:>8} {:>7} {:>9} \
                         {:>6.2}% | {:>6} {:>6} | {:>6} {:>6}",
                        vr.marked,
                        vr.rejected_window,
                        er.kept,
                        er.sparsity_pct(),
                        cr.faces,
                        cr.conflicts,
                        sing,
                        cap
                    );
                }
            }
        }
    }
}
