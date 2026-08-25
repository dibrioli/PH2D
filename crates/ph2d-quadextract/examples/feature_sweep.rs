//! ⭐⭐⭐ **A SONDA QUE FIXA OS QUATRO COEFICIENTES DA LEI DA FEIÇÃO** (`CLAUDE.md` §0.0).
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
//! ⛔ **O que ela mede NÃO é «quantas feições achámos».** A espec é explícita: marcar
//! feição a mais é pior que a menos, porque cada restrição força uma singularidade. As
//! colunas que decidem são a **esparsidade** e o que a **janela** de facto recusa.

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

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| String::from("esfera"));
    let mesh = load(&name);
    let h = edge_mean(&mesh);
    println!(
        "{name}: {} vertices, {} faces (F1 honrado) · aresta media {h:.4}",
        mesh.positions().len(),
        mesh.face_count()
    );
    println!(
        "{:>6} {:>6} {:>10} {:>8} | {:>10} {:>8} {:>8} {:>8} | {:>8}",
        "r0/e", "r1/h", "anisotrop", "janela", "MARCADOS", "%", "planos", "janela", "raio p50"
    );
    for r0 in [1.0f32, 2.0] {
        for r1 in [2.0f32, 4.0, 8.0] {
            for a in [0.5f32, 0.7, 0.85] {
                for hw in [0.25f32, 0.5, 1.0] {
                    let opts = ph2d_mesh::FeatureOptions {
                        r0_in_edges: r0,
                        r1_in_h: r1,
                        samples: 6,
                        half_window_in_h: hw,
                        min_anisotropy: a,
                        min_curvature_in_bbox: 0.05,
                    };
                    let (_, r) = ph2d_mesh::feature_dirs(&mesh, h, opts);
                    #[allow(clippy::cast_precision_loss)]
                    let pct = 100.0 * r.marked as f64 / r.points.max(1) as f64;
                    println!(
                        "{r0:>6.1} {r1:>6.1} {a:>10.2} {hw:>8.2} | {:>10} {pct:>7.2}% {:>8} {:>8} | {:>8.3}",
                        r.marked, r.rejected_flat, r.rejected_window, r.radius_p50
                    );
                }
            }
        }
    }
}
