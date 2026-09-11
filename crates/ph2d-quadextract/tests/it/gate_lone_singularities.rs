//! ⭐⭐⭐ **OS SINGULARES SOLTOS** — o vértice que o corte não duplicou e que ninguém
//! pregava num ponto inteiro.
//!
//! A cadeia causal que isto defende está medida em
//! [`ph2d_gridmap::RoundOptions::pin_lone_singularities`]: singular não pregado ⇒ imagem
//! fraccionária ⇒ transições fraccionárias ⇒ o extractor arredonda-as para células
//! inteiras ⇒ o traçado cai células ao lado ⇒ órfã ⇒ célula abandonada ⇒ **furo na ponta**.

use ph2d_crossfield::{Dual, solve_miq, vertex_index};
use ph2d_gridmap::{RoundOptions, comb_patches, cut_along_patches, round_welded};
use ph2d_mesh::{Mesh, shapes};

fn f1(mut mesh: Mesh) -> Mesh {
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    mesh
}

/// Uma esfera lisa — não há vinco nenhum, e as singularidades que ela tem são as **oito**
/// que Poincaré–Hopf obriga.
fn smooth() -> Mesh {
    f1(shapes::uv_sphere(48, 72, 1.0))
}

/// A mesma esfera com um **vinco** no equador. ⚠️ É a fixtura que contém o fenómeno: é
/// numa quina que o campo planta singularidades a mais, e é lá que aparecem as soltas.
fn creased() -> Mesh {
    let mut mesh = shapes::uv_sphere(48, 72, 1.0);
    for p in mesh.positions_mut() {
        let bump = 0.12 * (1.0 - (p[1].abs() / 0.10)).max(0.0);
        let k = 1.0 + bump;
        *p = [p[0] * k, p[1], p[2] * k];
    }
    mesh.rebuild();
    f1(mesh)
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

/// Corre o G3+G5 soldado com o interruptor na mão, e devolve `(mapa, quantos soltos)`.
fn run(mesh: &Mesh, pin_lone: bool) -> (ph2d_gridmap::GridMap, usize) {
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
    let opts = RoundOptions {
        pin_lone_singularities: pin_lone,
        ..RoundOptions::default()
    };
    let (map, rep) = round_welded(
        mesh,
        &cut,
        &combed,
        ph2d_gridmap::Step::uniform(median_edge(mesh)),
        opts,
        &singular,
    );
    (map, rep.singular_loose_pinned)
}

/// ⭐⭐⭐ **A INÉRCIA: onde não há solto nenhum, o mapa é BYTE-IDÊNTICO.**
///
/// ⛔ É este gate que impede a cura de se alargar. *Uma cura que também mexe onde não há
/// defeito deixa de ser uma cura e passa a ser uma mudança de produto* — e foi exactamente
/// o que aconteceu na 1.ª redacção, que pregava `19` classes onde a medição contava `6` e
/// trocava um defeito por outro.
#[test]
fn where_no_singular_is_loose_the_map_is_untouched() {
    let mesh = smooth();
    let (off, n_off) = run(&mesh, false);
    let (on, n_on) = run(&mesh, true);
    eprintln!("esfera lisa: {n_off} soltos com o interruptor em baixo, {n_on} em cima");
    assert_eq!(n_off, 0, "com o interruptor em baixo ninguem e' pregado");
    assert_eq!(
        n_on, 0,
        "⛔ a esfera LISA nao tem singular solto — se ela prega {n_on}, o criterio alargou"
    );
    assert_eq!(
        off.uv, on.uv,
        "⛔ o mapa mudou numa peca em que a cura nao tinha nada para fazer"
    );
    assert_eq!(off.shift, on.shift, "⛔ as translacoes mudaram sem motivo");
}

/// ⭐⭐ **O ALCANCE: numa peça com vinco, a cura tem quem pregar.**
///
/// ⛔ Sem este gate o da inércia é **vazio** — uma cura que nunca dispara passa nele por
/// não fazer nada em lado nenhum. *Um par de gates em que só um pode falhar não mede.*
#[test]
fn a_creased_piece_has_loose_singulars_and_they_get_pinned() {
    let mesh = creased();
    let (off, n_off) = run(&mesh, false);
    let (on, n_on) = run(&mesh, true);
    eprintln!("esfera com vinco: {n_off} soltos em baixo, {n_on} em cima");
    assert_eq!(n_off, 0);
    assert!(
        n_on > 0,
        "⛔ a fixtura nao contem o fenomeno: nenhum singular solto para pregar"
    );
    assert_ne!(
        off.uv, on.uv,
        "⛔ pregou {n_on} e o mapa nao se mexeu — a pregagem nao esta' a chegar ao mapa"
    );
}
