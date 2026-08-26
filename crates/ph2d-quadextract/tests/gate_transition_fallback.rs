//! ⭐⭐⭐ **O RECURSO DE TRANSIÇÃO TEM DE APROXIMAR.**
//!
//! Quando nenhuma das quatro rotações leva uma aresta na outra **exactamente**, o
//! saneamento cai num recurso. ⛔ Esse recurso era `topo.xf[f][k]` — o valor derivado na
//! fase anterior —, e a [`late_collapse`] renumera as faces e repõe o `xf` inteiro a
//! **identidade** antes disto correr. Nas peças em que ela mexe, o recurso era portanto a
//! identidade: uma transição que diz *«as duas cartas coincidem»* entre duas cartas que não
//! coincidem, e que manda o traçado da isolinha para outra parte da peça.
//!
//! ⚠️ **Nenhum gate de integralidade a apanhava** — a identidade é perfeitamente inteira.
//!
//! ⚠️ **O que este gate consegue e o que não consegue.** Ele corre o ramo de recurso **8
//! vezes na esfera e 10 no toro**, e exige que todos aproximem; uma mudança que estrague o
//! recurso *nestas* peças reprova. ⛔ **Não** reproduz a combinação exacta do defeito
//! original (recurso + `late_collapse` a correr): nenhuma fixtura do repositório colapsa
//! tarde. *Dizer isto é mais barato que descobri-lo mais tarde.*

use ph2d_crossfield::{Dual, solve_miq, vertex_index};
use ph2d_gridmap::{RoundOptions, comb_patches, corner_map, cut_along_patches, round_welded};
use ph2d_mesh::{Mesh, shapes};

fn f1(mut mesh: Mesh) -> Mesh {
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    mesh
}

fn median_edge(m: &Mesh) -> f32 {
    let pos = m.positions();
    let mut v: Vec<f32> = Vec::new();
    for f in m.faces() {
        let t = f.verts();
        for k in 0..t.len() {
            let (a, b) = (pos[t[k] as usize], pos[t[(k + 1) % t.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            v.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    v.sort_by(f32::total_cmp);
    v[v.len() / 2]
}

fn run(mesh: &Mesh) -> ph2d_quadextract::ExtractReport {
    let dual = Dual::build(mesh);
    let (field, _) = solve_miq(&dual);
    let singular: Vec<u32> = vertex_index(mesh, &dual, &field)
        .into_iter()
        .enumerate()
        .filter(|(_, k)| *k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    let layout = ph2d_trace::trace_patches(mesh, &dual, &field);
    let (cut, _) = cut_along_patches(mesh, &layout);
    let (combed, _) = comb_patches(mesh, &layout, &cut);
    let (map, _) = round_welded(
        mesh,
        &cut,
        &combed,
        median_edge(mesh),
        RoundOptions::default(),
        &singular,
    );
    let (tris, uv) = corner_map(&cut, &map);
    let cm = ph2d_quadextract::CornerMap {
        pos: mesh.positions(),
        tris: &tris,
        uv: &uv,
    };
    ph2d_quadextract::extract(&cm, None)
        .expect("a extraccao corre nestas duas fixturas")
        .1
}

/// ⭐⭐ **Todo recurso aproxima — e o ramo de recurso É exercido.**
///
/// ⛔ **As duas metades são precisas.** Só a primeira passaria numa peça onde o recurso
/// nunca corre, que é a forma canónica de um gate vazio nesta linha.
#[test]
fn every_transition_fallback_lands_within_half_a_cell() {
    for (nome, mesh) in [
        ("esfera", f1(shapes::uv_sphere(24, 36, 1.0))),
        ("toro", f1(shapes::torus(64, 32, 1.0, 0.35))),
    ] {
        let rep = run(&mesh);
        eprintln!(
            "{nome}: {} faces F1 · {} transicoes inexactas, {} sem aproximar · {} nos",
            mesh.face_count(),
            rep.inexact_transitions,
            rep.far_fallbacks,
            rep.vertex_nodes + rep.edge_nodes + rep.face_nodes
        );
        assert!(
            rep.inexact_transitions > 0,
            "⛔ {nome} nao exerce o ramo de recurso — este gate aprovaria qualquer coisa"
        );
        assert_eq!(
            rep.far_fallbacks, 0,
            "⛔ {nome}: {} recursos caem a mais de meia celula do alvo — uma transicao \
             dessas manda o tracado para outra parte da peca",
            rep.far_fallbacks
        );
    }
}
