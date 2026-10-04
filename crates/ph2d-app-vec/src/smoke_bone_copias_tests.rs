use super::*;

/// ⭐⭐⭐ **A CENA ENSINA O QUE DIZ** — as duas barras prendem, a união do contacto NÃO corre nelas
/// (os contornos das cópias cruzam-se em repouso) e, dobradas, cada uma sai com a camada do traço.
/// ⛔ **O CONTROLO:** antes de dobrar (recta) nenhuma tem camada.
#[test]
fn as_duas_barras_saem_com_a_camada_do_traco() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut st = crate::state::VecState::default();
    build(&mut scene, &mut sim, &mut st);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut st.entities);
    let ids: Vec<VecPathId> = st.bone_smoke_pend.as_ref().expect("pendentes").iter().map(|(id, _)| *id).collect();
    let camadas = |sim: &SimWorld, scene: &VecScene| -> Vec<bool> {
        let d = ph2d_skeleton_live::skin_live::recook_desenhando(sim, &mut scene.clone());
        ids.iter().map(|id| d.get(id).is_some_and(|x| x.traco.is_some())).collect()
    };
    let mut recta = SimWorld::default();
    let mut scene_r = VecScene::new();
    let mut st_r = crate::state::VecState::default();
    build(&mut scene_r, &mut recta, &mut st_r);
    ph2d_vec_entities::entities::sync(&mut recta, &mut scene_r, &mut st_r.entities);
    for (id, raiz) in st_r.bone_smoke_pend.take().expect("pendentes") {
        assert_eq!(ph2d_skeleton_live::skin_live::bind(&mut recta, &mut scene_r, &st_r.entities, &[id], raiz), 1);
    }
    assert_eq!(camadas(&recta, &scene_r), [false, false], "o CONTROLO: recta já tem camada");
    bind(&mut scene, &mut sim, &mut st);
    assert_eq!(camadas(&sim, &scene), [true, true], "dobradas, cada barra sai com a camada do traço");
}
