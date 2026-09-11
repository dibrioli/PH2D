//! ⛔ **A 2.ª TENTATIVA do arredondamento** (experimentar o inteiro do outro lado quando o
//! mais próximo dobra o mapa) foi construída, medida e **REJEITADA** — e ela **cumpre o que
//! promete**: apaga todas as dobras que o arredondamento cria, e os furos pioram.
//!
//! A tabela e as duas coisas que a medição derruba vivem no doc de `RETRY_ON_FOLD`, em
//! `ph2d-gridmap/src/weld_round.rs`.
//!
//! ⭐ **O que sobrevive dela é a RÉGUA por prego** — quantos pregos criam uma dobra —, e é
//! ela que separa *«um punhado de pregos maus»* de *«o custo espalhado de todos»*.

use ph2d_crossfield::{Dual, solve_miq, vertex_index};
use ph2d_gridmap::{RoundOptions, comb_patches, cut_along_patches, round_welded};
use ph2d_mesh::{Mesh, shapes};

/// Uma esfera com um vinco no equador — é numa quina que o arredondamento dobra o mapa.
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

/// ⭐⭐ **A 2.ª tentativa está DESLIGADA — e a régua que ela deixou está VIVA.**
///
/// ⛔ **As duas metades são precisas.** Só a primeira deixaria passar um contador morto:
/// uma régua sempre a `0` satisfaz «nenhuma segunda tentativa correu» sem medir nada.
/// ⚠️ *Um gate de inércia sozinho aprova um instrumento morto* — a mesma lei do irmão do
/// endurecimento local.
#[test]
fn the_second_try_is_off_and_the_ruler_is_alive() {
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
    let (_, rep) = round_welded(
        &mesh,
        &cut,
        &combed,
        ph2d_gridmap::Step::uniform(median_edge(&mesh)),
        RoundOptions::default(),
        &singular,
    );
    eprintln!(
        "esfera com vinco: {} pregos, {} criaram dobra · dobras {} ⇒ {} · 2a tentativa {} \
         (ganhou {}) · passo pior {:.4}",
        rep.pinned,
        rep.pins_that_folded,
        rep.folded_before_rounding,
        rep.folded_after_rounding,
        rep.second_tries,
        rep.second_tries_won,
        rep.worst_step
    );
    assert_eq!(
        rep.second_tries, 0,
        "⛔ a 2a tentativa esta' REJEITADA por medicao e correu {} vezes",
        rep.second_tries
    );
    assert!(
        rep.worst_step <= 0.5,
        "⛔ o passo pior foi {:.4}: sem a 2a tentativa o guloso NUNCA anda mais de meia \
         celula, e e' isso que mantem o mapa inteiro perto do continuo",
        rep.worst_step
    );
    assert!(
        rep.pins_that_folded > 0,
        "⛔ a fixtura nao tem prego nenhum a dobrar o mapa — este gate aprovaria um \
         contador morto"
    );
}
