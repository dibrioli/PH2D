//! **O roteador das famílias de estágio das cenas 21..=37** — irmão do [`crate::vector_mode`] pelo
//! teto de 700 LOC. Cada família declara a sua faixa (`FIRST..=LAST`); `h` é a cena dentro da
//! família das alças (`scenes::handles_first`).

use crate::state::VecState;
use crate::{
    vector_mode_alinhar as alinhar, vector_mode_controle as controle, vector_mode_cor as cor,
    vector_mode_gesto as gesto, vector_mode_interface as interface,
    vector_mode_le_a_forma as le_a_forma, vector_mode_misto as misto, vector_mode_olho as olho,
    vector_mode_opaco as opaco, vector_mode_quinas as quinas, vector_mode_tracejado as tracejado,
    vector_mode_vestidos as vestidos,
};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::screens::hero::HeroScreen;

/// As faixas de estágio destas famílias — o teste das faixas lê a MESMA lista.
pub(crate) const FAIXAS: [(u8, u8); 12] = [
    (alinhar::FIRST, alinhar::LAST),
    (tracejado::FIRST, tracejado::LAST),
    (interface::FIRST, interface::LAST),
    (le_a_forma::FIRST, le_a_forma::LAST),
    (misto::FIRST, misto::LAST),
    (cor::FIRST, cor::LAST),
    (vestidos::FIRST, vestidos::LAST),
    (controle::FIRST, controle::LAST),
    (quinas::FIRST, quinas::LAST),
    (gesto::FIRST, gesto::LAST),
    (olho::FIRST, olho::LAST),
    (opaco::FIRST, opaco::LAST),
];

/// O estágio `s` é de uma destas famílias?
pub(super) fn owns(s: u8) -> bool {
    FAIXAS.iter().any(|&(a, b)| (a..=b).contains(&s))
}

/// Um passo da família dona de `s`; devolve o próximo estágio, se mudou.
pub(super) fn step(
    s: u8,
    vec: &mut VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    h: u8,
) -> Option<u8> {
    match s {
        alinhar::FIRST..=alinhar::LAST => alinhar::step(s, vec, scene, hero),
        tracejado::FIRST..=tracejado::LAST => tracejado::step(s, vec, scene, hero),
        interface::FIRST..=interface::LAST => {
            // As 27/28 continuam pela fila vestida; as 29/30 pelo Hover regravado; a 35 pelo gesto; a 37
            // pelo Hover a 30 %.
            let end = match h {
                21 => gesto::FIRST,
                23 => opaco::FIRST,
                15.. => le_a_forma::FIRST,
                13.. => vestidos::FIRST,
                _ => 9,
            };
            interface::step(s, vec, scene, sim, hero, h == 12, end)
        }
        le_a_forma::FIRST..=le_a_forma::LAST => le_a_forma::step(s, vec, scene, hero, h == 16),
        misto::FIRST..=misto::LAST => misto::step(s, vec, scene, hero, h == 18),
        cor::FIRST..=cor::LAST => cor::step(s, vec, scene, hero, h == 20),
        vestidos::FIRST..=vestidos::LAST => vestidos::step(s, vec, scene, sim, hero, h == 14),
        controle::FIRST..=controle::LAST => controle::step(s, vec, scene, hero),
        quinas::FIRST..=quinas::LAST => quinas::step(s, vec, scene, sim, hero),
        gesto::FIRST..=gesto::LAST => gesto::step(s, vec, scene, hero),
        olho::FIRST..=olho::LAST => olho::step(s, vec, scene, hero),
        opaco::FIRST..=opaco::LAST => opaco::step(s, vec, scene, hero),
        _ => None,
    }
}
