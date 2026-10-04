//! ⭐⭐⭐ **O SOLVER EM FUNDO de uma forma presa com efeito** (F50-j) — num irmão do
//! [`super::efeitos_tests`] pelo tecto de LOC (2026-10-03).

use super::ouro_reguas_tests::*;
use crate::skin_desenho::Leis;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

fn caminho_mut(scene: &mut VecScene, id: VecPathId) -> &mut VecPath {
    scene.path_mut(id).expect("path")
}

/// ⭐⭐⭐ **GATE — mudar um efeito numa forma presa não paga o solver NO QUADRO, e o desenho chega
/// ao exacto** (F50-j: arrastar o *Twist* custava `59 ms` por quadro, pior `86`; em fundo `2,5`,
/// pior `3,3` — release, a mudar a pilha a cada quadro).
///
/// As metades: logo a seguir à mudança o desenho sai com o campo ANTERIOR (o quadro não esperou);
/// e alguns quadros depois é, ao bit, o que o solver síncrono dá. ⛔ O CONTROLO é o síncrono numa
/// thread própria.
#[test]
fn mudar_um_efeito_nao_paga_o_solver_no_quadro_e_chega_ao_exacto() {
    let twist = |angle| {
        vec![FxEntry::new(PathEffect::Twist(
            ph2d_vec_scene::fx_twist::TwistSpec { angle },
        ))]
    };
    let desenho = |p: &mut BPalco| {
        crate::skin_live::recook_leis(&p.sim, &mut p.scene.clone(), Leis::do_ambiente())
            .remove(&p.id)
            .expect("desenho")
    };
    // ⚠️ Numa THREAD própria: a gaveta é `thread_local` e indexada pela entidade, e duas fixturas
    // frescas dão a mesma — a referência deixava a resposta pronta na gaveta da outra.
    let exacto = std::thread::spawn(move || {
        let mut r = b_palco(false);
        r.dobra_em_s(60.0);
        caminho_mut(&mut r.scene, r.id).effects = twist(40.0);
        desenho(&mut r)
    })
    .join()
    .expect("a referência síncrona");

    crate::skin_desenho::solver_em_fundo_no_teste(true);
    let mut p = b_palco(false);
    p.dobra_em_s(60.0);
    caminho_mut(&mut p.scene, p.id).effects = twist(25.0);
    let _ = desenho(&mut p);
    caminho_mut(&mut p.scene, p.id).effects = twist(40.0);
    let logo = desenho(&mut p);
    let t = std::time::Instant::now();
    let mut depois = logo.clone();
    while depois.verts != exacto.verts && t.elapsed().as_secs() < 30 {
        std::thread::yield_now();
        depois = desenho(&mut p);
    }
    crate::skin_desenho::solver_em_fundo_no_teste(false);
    assert_ne!(
        logo.verts, exacto.verts,
        "o 1.º quadro depois da mudança já trazia o campo novo — o solver correu NO quadro"
    );
    assert_eq!(
        depois.verts, exacto.verts,
        "o campo resolvido em fundo nunca chegou ao desenho (ou chegou outro)"
    );
}
