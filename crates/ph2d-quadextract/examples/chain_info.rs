//! ⭐⭐⭐ **A CADEIA INTEIRA DA CASA**, com a **fase zero** honrada — e a única
//! medição em que a barra do oráculo (`4,8°`–`7,1°` de enviesamento) é comparável.
//!
//! ```text
//! cargo run --release -p ph2d-quadextract --example chain_info -- [peca] [densidade]
//! ```
//!
//! ⛔ **`--release`.** O solver contínuo gasta `160 000` rondas de Gauss–Seidel; em
//! `debug` isto é minutos por peça.

fn median_edge(mesh: &ph2d_mesh::Mesh) -> f32 {
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    e.sort_by(f32::total_cmp);
    e[e.len() / 2]
}

fn aspect(mesh: &ph2d_mesh::Mesh) -> (f32, f32) {
    let pos = mesh.positions();
    let mut a: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        let mut lo = f32::MAX;
        let mut hi = 0.0f32;
        for k in 0..v.len() {
            let (p, q) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
            let l = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
            lo = lo.min(l);
            hi = hi.max(l);
        }
        a.push(hi / lo.max(1.0e-20));
    }
    a.sort_by(f32::total_cmp);
    (a[a.len() / 2], a[(a.len() - 1) * 99 / 100])
}

fn main() {
    let mut args = std::env::args().skip(1);
    let piece = args.next().unwrap_or_else(|| String::from("esfera"));
    let scale: f32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);
    // ⭐⭐ **UM CAMINHO DE FICHEIRO É UMA PEÇA.** ⚠️ *Uma régua só prova o que a
    // fixtura contém* — as formas analíticas acima não têm relevo, e a queixa do
    // artista («não é fiel à curvatura») é **sobre relevo**. Medi-la nas peças que
    // ele de facto viu exige carregar as peças que ele de facto viu.
    let mut mesh = if piece.ends_with(".obj") {
        let text = std::fs::read_to_string(&piece)
            .unwrap_or_else(|e| panic!("nao consegui ler {piece}: {e}"));
        let pieces = ph2d_mesh::import_obj(&text)
            .unwrap_or_else(|e| panic!("{piece} nao e' um OBJ que este leitor entenda: {e:?}"));
        let first = pieces
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{piece} nao tem uma peca dentro"));
        first.mesh
    } else {
        match piece.as_str() {
            "toro" => ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35),
            "esfera-fina" => ph2d_mesh::shapes::uv_sphere(96, 144, 1.0),
            "esfera-irregular" => ph2d_mesh::shapes::uv_sphere_shuffled(48, 72, 1.0),
            _ => ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
        }
    };
    mesh.triangulate();
    let (a50, a99) = aspect(&mesh);
    println!(
        "{piece}: {} faces cruas, aspecto p50 {a50:.2} p99 {a99:.2}",
        mesh.face_count()
    );

    // ── ⛔⛔ FASE ZERO. Sem ela a mesma cadeia dá `10-12°`, e o defeito e' a entrada.
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    let (a50, a99) = aspect(&mesh);
    println!(
        "  F1 (fase zero): {} faces, aspecto p50 {a50:.2} p99 {a99:.2}  (a assinatura do F1 e' 1,16 / 1,58)",
        mesh.face_count()
    );

    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let singular: Vec<u32> = ph2d_crossfield::vertex_index(&mesh, &dual, &field)
        .into_iter()
        .enumerate()
        .filter(|(_, k)| *k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, cr) = ph2d_gridmap::cut_along_patches(&mesh, &layout);
    let (combed, comb) = ph2d_gridmap::comb_patches(&mesh, &layout, &cut);
    println!(
        "  F2+F3+G1+G2: {} patches ({} discos), {} costuras ({} com salto)",
        cr.patches, cr.discs, comb.seams, comb.jumps
    );

    let h = median_edge(&mesh) * scale;
    let t = std::time::Instant::now();
    let pin = std::env::args().nth(3).as_deref() != Some("sem-singularidades");
    let (map, r) = ph2d_gridmap::round_to_integers(
        &mesh,
        &cut,
        &combed,
        h,
        ph2d_gridmap::RoundOptions {
            pin_singularities: pin,
            ..ph2d_gridmap::RoundOptions::default()
        },
        &singular,
    );
    println!("  modalidade das singularidades: {pin}");
    println!(
        "  G3+G5 ({:.1} s): {} costuras de arvore + {} de CICLO + {} singularidades (de {}, {} copias, {} ambiguas) ⇒ {} inteiros \
         | degrau1 {} degrau2 {} | {} visitas | passo pior {:.4} soma {:.3} | passou-as-costuras {}",
        t.elapsed().as_secs_f64(),
        r.tree_seams,
        r.cycle_seams,
        r.singular_pinned,
        singular.len(),
        r.singular_copies,
        r.ambiguous_seams,
        r.pinned,
        r.level1,
        r.level2,
        r.visits,
        r.worst_step,
        r.sum_step,
        r.switched_to_seams
    );
    println!(
        "  ⭐ distancia a inteiro DEPOIS: {:.3e}  (tem de ser 0) | costura p50 {:.4}→{:.4} max {:.4}→{:.4} | angulo p50 {:.1}°",
        r.shift_frac_max,
        r.seam_before.0,
        r.seam_after.0,
        r.seam_before.1,
        r.seam_after.1,
        r.solve.angle_p50
    );
    println!(
        "  REGUAS do mapa: alinhamento p50 {:.3} p95 {:.3} | angulo p50 {:.1}° p95 {:.1}° | ESCALA p50 {:.3} p95 {:.3} | {} triangulos, {} saltados, {} pares",
        r.solve.align_p50,
        r.solve.align_p95,
        r.solve.angle_p50,
        r.solve.angle_p95,
        r.solve.scale_p50,
        r.solve.scale_p95,
        r.solve.triangles,
        r.solve.skipped,
        r.solve.pairs
    );

    let (tris, uv) = ph2d_gridmap::corner_map(&cut, &map);
    // ⭐ **CONTROLO INDEPENDENTE da ponte**: contar as dobras aqui, sem passar pela
    // extraccao. Se os dois numeros discordarem, o defeito e' do `corner_map` (uma
    // ordem de cantos trocada inverteria metade das areas sem erro nenhum a acusar).
    let folded = uv
        .iter()
        .filter(|t| {
            let d = (t[1][0] - t[0][0]) * (t[2][1] - t[0][1])
                - (t[1][1] - t[0][1]) * (t[2][0] - t[0][0]);
            d < 0.0
        })
        .count();
    println!(
        "  ponte: {} triangulos, {} DOBRADOS no dominio ({:.1}%)  <- controlo independente",
        tris.len(),
        folded,
        100.0 * folded as f64 / tris.len().max(1) as f64
    );
    let cm = ph2d_quadextract::CornerMap {
        pos: mesh.positions(),
        tris: &tris,
        uv: &uv,
    };
    match ph2d_quadextract::extract(&cm, None) {
        Ok((out, e)) => {
            let shape = ph2d_quadfill::quad_shape(&out);
            println!(
                "  EXTRACCAO: residuo de translacao p50 {:.3e} max {:.3e} | {} nos ({} vertice, {} aresta, {} face) | {} dobras",
                e.shift_residual_p50,
                e.shift_residual,
                e.vertex_nodes + e.edge_nodes + e.face_nodes,
                e.vertex_nodes,
                e.edge_nodes,
                e.face_nodes,
                e.folded_faces
            );
            let mut cnt: std::collections::BTreeMap<(u32, u32), usize> =
                std::collections::BTreeMap::new();
            for f in out.faces() {
                let v = f.verts();
                for k in 0..v.len() {
                    let (a, b) = (v[k], v[(k + 1) % v.len()]);
                    *cnt.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
                }
            }
            println!(
                "  degeneradas {} | celulas: {} fechadas {} abandonadas {} nao-fechadas | anel {:?} | cantos {:?} | arestas: {} bordo, {} nao-manifold",
                e.degenerate_faces,
                e.cells_closed,
                e.cells_abandoned,
                e.cells_unclosed,
                e.ring_len
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| **n > 0)
                    .map(|(i, n)| (i, *n))
                    .collect::<Vec<_>>(),
                e.ring_distinct
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| **n > 0)
                    .map(|(i, n)| (i, *n))
                    .collect::<Vec<_>>(),
                cnt.values().filter(|c| **c == 1).count(),
                cnt.values().filter(|c| **c >= 3).count()
            );
            println!(
                "  ⭐ SAIDA: {} verts, {} quads, X = {} | ordem limpa {:?} | orfas {} disputadas {} fugidas {}",
                out.vert_count(),
                e.quads,
                ph2d_quadextract::euler_characteristic(&out),
                e.port_step,
                e.orphan,
                e.contested,
                e.runaway
            );
            println!(
                "  ⭐⭐ FORMA: aspecto p50 {:.2} p99 {:.2} (>4x: {}) | ENVIESAMENTO p50 {:.1}° p99 {:.1}° (>60: {}) | area spread {:.2}",
                shape.aspect_p50,
                shape.aspect_p99,
                shape.aspect_over_4,
                shape.skew_p50,
                shape.skew_p99,
                shape.skew_over_60,
                shape.area_spread
            );
            println!(
                "     (a barra do oraculo: enviesamento p50 4,8-7,1 | aspecto p50 1,08-1,22 | >60: 0-4)"
            );
        }
        Err(err) => println!("  EXTRACCAO recusada: {err}"),
    }
}
