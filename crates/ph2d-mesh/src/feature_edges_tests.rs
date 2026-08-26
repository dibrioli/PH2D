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

/// Um tubo ABERTO — o cilindro da casa sem as duas tampas.
///
/// ⚠️ Fixtura com resposta CONHECIDA: os dois laços de bordo são círculos em `y = ±h`, e
/// toda tangente deles é perpendicular ao eixo. *Uma fixtura sem resposta conhecida só sabe
/// dizer que o código não entrou em pânico.*
fn open_tube(segments: usize) -> crate::Mesh {
    let full = shapes::cylinder(segments, 1.0, 2.0);
    let faces: Vec<crate::Face> = full
        .faces()
        .iter()
        .filter(|f| !f.verts().iter().any(|&v| v < 2))
        .copied()
        .collect();
    crate::Mesh::from_parts(full.positions().to_vec(), faces).expect("o tubo e' construido aqui")
}

/// ⭐⭐⭐ **O BORDO É ACHADO, CONTADO E A TANGENTE CORRE À VOLTA DELE.**
#[test]
fn an_open_tube_has_two_boundary_loops_and_the_tangent_runs_around_them() {
    const SEG: usize = 24;
    let mesh = open_tube(SEG);
    let (edges, loops) = crate::boundary_feature_edges(&mesh);
    assert_eq!(loops, 2, "⛔ um tubo aberto tem DOIS lacos de bordo");
    assert_eq!(edges.len(), 2 * SEG, "⛔ um segmento de bordo por sector");
    let worst = edges
        .iter()
        .map(|e| e.dir[1].abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 0.02,
        "⛔ a tangente do bordo tem de correr A' VOLTA do tubo, nao ao longo dele: {worst}"
    );
}

/// ⭐⭐ **INÉRCIA: uma peça fechada não oferece feição de bordo nenhuma.**
///
/// ⛔ É este gate que deixa a restrição ficar LIGADA sem tocar em peça fechada nenhuma.
#[test]
fn a_closed_piece_offers_no_boundary_feature() {
    let mut mesh = shapes::uv_sphere(12, 18, 1.0);
    mesh.triangulate();
    let (edges, loops) = crate::boundary_feature_edges(&mesh);
    assert_eq!(loops, 0);
    assert!(edges.is_empty(), "⛔ uma esfera nao tem bordo");
}

/// ⭐⭐⭐ **A TANGENTE ALISA O SERRILHADO — e a aresta crua NÃO.**
///
/// ⛔⛔ **Sem este gate, `dir` podia ser a direcção crua da aresta e os outros dois gates
/// passariam na mesma** (num círculo perfeito a aresta já é tangente). A fixtura mete o
/// serrilhado de propósito: vértices alternados do bordo sobem e descem, que é o que uma
/// triangulação faz à volta de um buraco de verdade.
///
/// ⚠️ **A régua é a COMPARAÇÃO com a aresta crua**, não um limiar absoluto: um limiar
/// escolhido à mão mediria a amplitude que eu pus na fixtura.
#[test]
fn the_tangent_smooths_the_zigzag_that_the_raw_edge_keeps() {
    const SEG: usize = 24;
    let tube = open_tube(SEG);
    let mut pos = tube.positions().to_vec();
    // O serrilhado: só o anel de cima, alternando.
    for (i, p) in pos.iter_mut().enumerate() {
        if p[1] > 0.5 && i % 2 == 0 {
            p[1] -= 0.25;
        }
    }
    let mesh = crate::Mesh::from_parts(pos, tube.faces().to_vec()).expect("fixtura");
    let (edges, _) = crate::boundary_feature_edges(&mesh);

    let p = mesh.positions();
    let raw = edges
        .iter()
        .map(|e| {
            let (a, b) = (p[e.verts[0] as usize], p[e.verts[1] as usize]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let l = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
            (d[1] / l).abs()
        })
        .fold(0.0f32, f32::max);
    let smooth = edges.iter().map(|e| e.dir[1].abs()).fold(0.0f32, f32::max);
    eprintln!("bordo serrilhado: aresta crua {raw:.4} · tangente alisada {smooth:.4}");
    assert!(
        raw > 0.3,
        "⛔ a fixtura tem de CONTER o serrilhado: {raw}"
    );
    assert!(
        smooth < raw * 0.5,
        "⛔ a tangente tem de alisar o serrilhado, e nao herda-lo: {smooth} contra {raw}"
    );
}
