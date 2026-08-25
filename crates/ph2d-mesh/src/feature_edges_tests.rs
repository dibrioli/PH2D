//! Os gates da ponte vértice → aresta.

use super::{FEATURE_EDGE_MIN_COS, feature_edges};
use crate::{FeatureDir, FeatureOptions, feature_dirs, shapes};

fn edge_mean(m: &crate::Mesh) -> f32 {
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

/// ⭐⭐⭐ **O «E» DAS DUAS PONTAS É A LEI, e este gate é o que o separa de um «OU».**
///
/// A fixtura marca **metade** da esfera com uma direcção constante. Uma aresta que
/// atravesse a fronteira tem **uma** ponta marcada — e a lei diz que ela **não é
/// feição**, por mais bem alinhada que esteja.
///
/// ⛔ **Sem este gate, trocar o `&&` por `||` passa despercebido**: a contagem total
/// sobe, e «mais feições» lê-se como sucesso. *A régua da espec é o contrário disso.*
#[test]
fn an_edge_needs_both_ends_marked_not_just_one() {
    let mut sphere = shapes::uv_sphere(32, 48, 1.0);
    sphere.triangulate();
    let pos = sphere.positions().to_vec();

    // Só o hemisfério `x > 0`, e toda a marca aponta para o mesmo lado.
    let dirs: Vec<FeatureDir> = (0..pos.len())
        .filter(|&v| pos[v][0] > 0.0)
        .map(|v| FeatureDir {
            vert: v as u32,
            dir: [1.0, 0.0, 0.0],
            radius: 0.1,
            anisotropy: 1.0,
        })
        .collect();
    assert!(
        dirs.len() > 100,
        "a fixtura tem de ter marcas: {}",
        dirs.len()
    );

    let (edges, rep) = feature_edges(&sphere, &dirs, FEATURE_EDGE_MIN_COS);
    for e in &edges {
        assert!(
            pos[e.verts[0] as usize][0] > 0.0 && pos[e.verts[1] as usize][0] > 0.0,
            "⛔ a aresta {e:?} tem uma ponta NAO marcada e ficou — o «E» virou «OU»"
        );
        let l = e.dir[0].mul_add(e.dir[0], e.dir[1].mul_add(e.dir[1], e.dir[2] * e.dir[2]));
        assert!(
            (l - 1.0).abs() < 1.0e-4,
            "⛔ a direccao que a aresta carrega tem de vir normalizada: {l}"
        );
    }
    assert_eq!(
        rep.kept + rep.rejected_angle,
        rep.candidates,
        "⛔ a escrituracao nao fecha: {} + {} != {}",
        rep.kept,
        rep.rejected_angle,
        rep.candidates
    );
    assert!(
        rep.candidates < rep.total_edges,
        "⛔ TODA aresta virou candidata ({} de {}) — a metade nao marcada nao existiu",
        rep.candidates,
        rep.total_edges
    );
    assert!(
        rep.kept > 0,
        "⛔ nenhuma aresta ficou: a fixtura nao contem o fenomeno"
    );
    eprintln!(
        "meia-esfera marcada: {} de {} arestas candidatas, {} ficaram ({:.2}% da peca)",
        rep.candidates,
        rep.total_edges,
        rep.kept,
        rep.sparsity_pct()
    );
}

/// ⭐⭐ **O CONTROLO: uma esfera lisa não tem uma única aresta de feição.**
///
/// ⚠️ Ele é o irmão do controlo da detecção — e é preciso os dois, porque a ponte pode
/// fabricar arestas a partir de marcas que a detecção não deu. *Um zero aqui só vale
/// alguma coisa ao lado de uma fixtura em que a ponte devolve muitas.*
#[test]
fn a_smooth_sphere_yields_no_feature_edge_at_all() {
    let mut sphere = shapes::uv_sphere(32, 48, 1.0);
    sphere.triangulate();
    let (dirs, _) = feature_dirs(&sphere, edge_mean(&sphere), FeatureOptions::default());
    let (edges, rep) = feature_edges(&sphere, &dirs, FEATURE_EDGE_MIN_COS);
    eprintln!(
        "esfera: {} marcas, {} arestas candidatas, {} ficaram ({:.2}%)",
        dirs.len(),
        rep.candidates,
        rep.kept,
        rep.sparsity_pct()
    );
    assert!(
        rep.sparsity_pct() < 0.5,
        "⛔ a esfera LISA produziu {:.2}% de arestas de feicao ({} de {}) — e' ruido",
        rep.sparsity_pct(),
        rep.kept,
        rep.total_edges
    );
    assert!(
        edges.windows(2).all(|w| w[0].verts < w[1].verts),
        "a lista tem de vir ordenada e sem repeticao"
    );
}
