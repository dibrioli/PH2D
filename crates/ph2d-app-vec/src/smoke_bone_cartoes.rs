//! ⭐ **O PAINEL BONES EM CARTÕES, CHEIO** — `PH2D_VEC_BONE_SMOKE=9`: a barra presa ao esqueleto
//! ([`ph2d_skeleton_demo::cartoes`]), o osso da ponta em foco e o *Transform* na mão. O cartão Skin
//! não aparece: osso + forma escolhidos juntos a selecção vira o Skeleton. Smart Bone nasce vazio
//! (`+`), salvo com `PH2D_VEC_BONE_SMOKE_SMART=1`.

use ph2d_ecs::SimWorld;
use ph2d_skeleton_demo::cartoes;
use ph2d_vec_scene::VecScene;

/// O 1.º tempo: monta a cena e guarda o que prender no 2.º.
pub(crate) fn build(scene: &mut VecScene, sim: &mut SimWorld, st: &mut crate::state::VecState) {
    let smart = std::env::var_os("PH2D_VEC_BONE_SMOKE_SMART").is_some();
    let Some(m) = cartoes::monta(scene, sim, crate::smoke_bone_par::COR, smart) else {
        eprintln!("[vec-bone-smoke] =9: a cena NAO montou -- PARE");
        return;
    };
    st.bone_smoke_foco = Some(m.ponta.to_bits());
    st.bone_smoke_pend = Some(vec![(m.barra, Some(m.raiz))]);
    st.bone_smoke_step = 1;
}

/// O 2.º tempo: prende a barra ao esqueleto.
pub(crate) fn bind(scene: &mut VecScene, sim: &mut SimWorld, st: &mut crate::state::VecState) {
    st.bone_smoke_step = 2;
    let Some(pecas) = st.bone_smoke_pend.take() else {
        return;
    };
    let mut presas = 0;
    for (id, raiz) in &pecas {
        presas += ph2d_skeleton_live::skin_live::bind(sim, scene, &st.entities, &[*id], *raiz);
    }
    println!("[vec-bone-smoke] =9 OS CARTOES DO PAINEL BONES: {presas} forma presa a 3 ossos.");
}
