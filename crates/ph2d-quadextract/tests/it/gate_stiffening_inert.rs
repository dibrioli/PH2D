//! ⛔ **O ENDURECIMENTO LOCAL foi construído, medido e REJEITADO** — este gate é o que
//! garante que ele fica **inerte** no caminho que shipa.
//!
//! A tabela da rejeição, o mecanismo e as duas hipóteses que a fecham vivem em
//! [`ph2d_gridmap::STIFFEN_PASSES`]. ⭐ *O que sobrevive dela é a RÉGUA* — a contagem de
//! triângulos virados no contínuo, que não existia.

use ph2d_crossfield::{Dual, solve_miq, vertex_index};
use ph2d_gridmap::{RoundOptions, STIFFEN_PASSES, comb_patches, cut_along_patches, round_welded};
use ph2d_mesh::{Mesh, shapes};

fn piece() -> Mesh {
    let mut mesh = shapes::uv_sphere(40, 60, 1.0);
    for p in mesh.positions_mut() {
        let bump = 0.12 * (1.0 - (p[1].abs() / 0.10)).max(0.0);
        let k = 1.0 + bump;
        *p = [p[0] * k, p[1], p[2] * k];
    }
    mesh.rebuild();
    mesh.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
    mesh.triangulate();
    mesh
}

/// ⭐⭐ **A ZERO PASSAGENS o caminho é o de sempre — e a régua das dobras funciona.**
///
/// ⛔ **As duas metades são precisas.** Sem a primeira, uma rejeição pode ir a ship a
/// mexer no produto; sem a segunda, o contador podia estar sempre a `0` e o gate passaria
/// por não medir nada. *Um gate de inércia sozinho aprova um instrumento morto.*
#[test]
fn stiffening_at_zero_passes_is_the_old_path() {
    assert_eq!(
        STIFFEN_PASSES, 0,
        "⛔ o endurecimento local esta' REJEITADO por medicao — ver a tabela no doc dele"
    );

    let mesh = piece();
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    let singular: Vec<u32> = vertex_index(&mesh, &dual, &field)
        .into_iter()
        .enumerate()
        .filter(|(_, k)| *k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    let layout = ph2d_trace::trace_patches(&mesh, &dual, &field);
    let (cut, _) = cut_along_patches(&mesh, &layout);
    let (combed, _) = comb_patches(&mesh, &layout, &cut);

    let pos = mesh.positions();
    let mut v: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let t = f.verts();
        for k in 0..t.len() {
            let (a, b) = (pos[t[k] as usize], pos[t[(k + 1) % t.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            v.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    v.sort_by(f32::total_cmp);
    let h = v[v.len() / 2];

    let (_, rep) = round_welded(
        &mesh,
        &cut,
        &combed,
        ph2d_gridmap::Step::uniform(h),
        RoundOptions::default(),
        &singular,
    );
    eprintln!(
        "esfera com vinco: {} passagens de endurecimento · virados {} ⇒ {}",
        rep.stiffen_passes, rep.folded_before, rep.folded_after
    );
    assert_eq!(
        rep.stiffen_passes, 0,
        "⛔ o produto correu {} passagens de uma cura REJEITADA",
        rep.stiffen_passes
    );
    assert_eq!(
        rep.folded_before, rep.folded_after,
        "⛔ sem endurecer, as duas contagens sao a MESMA leitura"
    );
    assert!(
        rep.folded_before > 0,
        "⛔ a fixtura tem de conter DOBRAS, senao este gate aprova um contador morto"
    );
}
