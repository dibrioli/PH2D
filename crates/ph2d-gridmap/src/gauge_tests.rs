//! ⭐⭐⭐ **OS GATES DO CALIBRE** — e a sonda que responde se a extracção é viável.

use ph2d_mesh::Mesh;

fn upto(
    mut mesh: Mesh,
) -> (
    crate::cut::CutMesh,
    crate::comb::Combed,
    crate::solve::GridMap,
) {
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    let dual = ph2d_crossfield::Dual::build(&mesh);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = crate::cut::cut_along_patches(&mesh, &layout);
    let (combed, _) = crate::comb::comb_patches(&mesh, &layout, &cut);
    let pos = mesh.positions();
    let mut edges: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            edges.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    edges.sort_by(f32::total_cmp);
    let h = edges[edges.len() / 2];
    let (map, _) = crate::solve::solve(&mesh, &cut, &combed, h);
    (cut, combed, map)
}

/// ⭐⭐ **A ÁRVORE COBRE TUDO, e a conta de ciclos fecha.**
///
/// `arestas − vértices + componentes` é o número de ciclos independentes de um grafo, e
/// é quantos inteiros a extracção tem de escolher. ⚠️ *Se esta conta não fechar, a
/// árvore não é uma árvore e as translações «invariantes» não o são.*
#[test]
fn the_spanning_tree_covers_every_patch_and_the_cycle_count_closes() {
    for (name, mesh) in [
        ("esfera lisa", ph2d_mesh::shapes::uv_sphere(24, 36, 1.0)),
        ("toro", ph2d_mesh::shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        let (cut, combed, map) = upto(mesh);
        let (g, rep) = super::fix(&cut, &combed, &map);
        assert_eq!(
            rep.reached, 1,
            "{name}: a semente por componente contou {} raizes -- mais que `1` significa \
             que o grafo de patches esta' PARTIDO, e cada pedaco tem calibre proprio",
            rep.reached
        );
        assert_eq!(
            g.offset.len(),
            rep.patches,
            "{name}: falta calibre a algum patch"
        );
        let seams = cut.seams.len() - rep.loose;
        assert_eq!(
            rep.tree + rep.cycles,
            seams,
            "{name}: {} de arvore + {} de ciclo != {seams} costuras com salto",
            rep.tree,
            rep.cycles
        );
        assert_eq!(
            rep.cycles,
            seams + rep.reached - rep.patches,
            "{name}: a conta `arestas - vertices + componentes` nao fecha"
        );
    }
}

/// ⭐⭐⭐ **SONDA — a extracção é viável? (a distância a inteiro INVARIANTE)**
///
/// ```text
/// cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-gridmap --release \
///     are_the_cycle_shifts_near_integers -- --ignored --nocapture
/// ```
///
/// ⛔⛔ **A primeira medição perguntou isto às translações CRUAS e respondeu `0,408`** —
/// mas a translação de uma costura é de **calibre**, e perguntar se um número de calibre
/// é inteiro é perguntar sobre a escolha de quem o escreveu. ⭐ O que é invariante é a
/// translação das costuras que **fecham ciclo**, depois de a árvore ser levada a zero.
#[test]
#[ignore = "sonda -- a distancia a inteiro invariante"]
fn are_the_cycle_shifts_near_integers() {
    for (name, mesh) in [
        ("ESFERA FINA", ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)),
        ("ESFERA LISA", ph2d_mesh::shapes::uv_sphere(24, 36, 1.0)),
        ("TORO", ph2d_mesh::shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        let (cut, combed, map) = upto(mesh);
        let (_, rep) = super::fix(&cut, &combed, &map);
        // ⭐ O controlo: a mesma pergunta feita à grandeza de CALIBRE.
        let mut raw: Vec<f32> = map
            .shift
            .iter()
            .map(|t| (t[0] - t[0].round()).abs().max((t[1] - t[1].round()).abs()))
            .collect();
        raw.sort_by(f32::total_cmp);
        let raw_p50 = raw.get(raw.len() / 2).copied().unwrap_or(0.0);
        eprintln!(
            "  {name}: {} patches · {} de arvore · ⭐{} CICLOS \
             | distancia a inteiro: ⭐INVARIANTE p50 {:.3} max {:.3} · (calibre, sem sentido: {:.3})",
            rep.patches, rep.tree, rep.cycles, rep.frac_p50, rep.frac_max, raw_p50
        );
    }
}
