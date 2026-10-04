//! A6 — os contornos FECHADOS de trás (duas cópias sobrepostas): a emenda e o avesso. Irmão de
//! [`super`] pelo tecto de LOC.

use crate::skin_live::tests::palco;
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_scene::VecPath;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};

/// Duas cópias da barra `40 × 10` sobrepostas (a 2.ª girada `5°`), presas a dois ossos e a ponta a
/// `graus`: a fonte guardada, a tabela, o campo, a pele e a profundidade.
pub(super) fn copias_dobradas(
    graus: f32,
) -> (
    VecPath,
    Vec<f64>,
    ph2d_vec_skin::pesos::CampoDoDominio,
    ph2d_skeleton::Skin,
    Vec<f64>,
) {
    let (mut sim, mut scene, map, id, [_, ponta]) = palco();
    scene.path_mut(id).expect("path").effects = vec![FxEntry::new(PathEffect::Repeat(
        ph2d_vec_scene::fx_repeat::RepeatSpec {
            copies_x: 1.0,
            move_x: 0.0,
            copies_y: 2.0,
            move_y: 60.0,
            spin: 5.0,
            orbit: 0.0,
        },
    ))];
    assert_eq!(
        crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], None),
        1
    );
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ponta)
        .expect("Transform")
        .rotation = graus.to_radians();
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let index = crate::skin_live::bone_index(&sim);
    let pele = crate::skin_live::resolve(&sim, &skin, e, &index).expect("pele");
    let prof = crate::esqueletos::profundidades(&sim, &skin, &index);
    (g.path, g.pesos, g.campo.expect("campo"), pele, prof)
}

/// ⭐⭐ **GATE — o trecho de um contorno FECHADO que passa pela EMENDA é um só** (o início NEGATIVO) —
/// nenhum trecho começa no nó `0` nem acaba no fim da volta enquanto outro do mesmo contorno toca a
/// outra ponta. ⛔ **O CONTROLO:** algum trecho passa mesmo pela emenda.
#[test]
fn o_trecho_que_passa_pela_emenda_e_um_so() {
    let (fonte, _, campo, pele, prof) = copias_dobradas(110.0);
    let cortes =
        super::super::cortes_dos_fechados(&fonte, (&campo, None), (&pele, &[], true), &prof)
            .expect("a dobra tapa um fechado");
    let mut pela_emenda = 0;
    for (c, trechos) in cortes.iter().enumerate() {
        let Some(t) = trechos else { continue };
        #[expect(clippy::cast_precision_loss, reason = "contagem de nós")]
        let m = fonte.contour(c).expect("contorno").0.len() as f64;
        pela_emenda += t.iter().filter(|((a, ..), _)| *a < 0.0).count();
        let comeca = t.iter().any(|((a, ..), _)| *a == 0.0);
        let acaba = t.iter().any(|(_, (b, ..))| *b >= m);
        assert!(
            !(comeca && acaba),
            "o contorno {c} partiu na emenda um trecho que é um só"
        );
    }
    assert!(
        pela_emenda > 0,
        "o CONTROLO: nenhum trecho passa pela emenda"
    );
}

/// ⭐⭐ **GATE — o AVESSO não tapa o traço de um contorno FECHADO** (ali ele É a borda da dobra; sem
/// ele a frente abria um vão sem desenhar a própria borda — FOTOGRAFADO em SVG a `130°`). Cada ponto
/// do contorno que SÓ o avesso taparia cai dentro de um trecho à vista. ⛔ **O CONTROLO:** há pontos
/// assim.
#[test]
fn o_avesso_nao_tapa_o_traco_de_um_contorno_fechado() {
    let (fonte, _, campo, pele, prof) = copias_dobradas(130.0);
    let mut f = super::super::Posada::nova(&campo, None, &pele, &[], true, &prof)
        .expect("posada")
        .com_a_arte(&fonte);
    let cortes =
        super::super::cortes_dos_fechados(&fonte, (&campo, None), (&pele, &[], true), &prof)
            .expect("a dobra tapa um fechado");
    let (mut so_dele, mut faltam) = (0, 0);
    for (c, trechos) in cortes.iter().enumerate() {
        let Some(t) = trechos else { continue };
        let v = fonte.contour(c).expect("contorno").0;
        let mut w = v.to_vec();
        w.push(v[0]);
        #[expect(clippy::cast_precision_loss, reason = "contagem de nós")]
        let m = v.len() as f64;
        for i in 0..(200 * v.len()) {
            #[expect(clippy::cast_precision_loss, reason = "amostra")]
            let u = i as f64 / 200.0;
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "segmento"
            )]
            let k = u.floor() as usize;
            #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
            let q = super::super::avalia(&super::super::cubica(&w, k), u - k as f64);
            f.avesso = true;
            let com = f.tapado(q);
            f.avesso = false;
            if com && !f.tapado(q) {
                so_dele += 1;
                let dentro_ = t
                    .iter()
                    .any(|((a, ..), (b, ..))| (u > *a && u < *b) || (u - m > *a && u - m < *b));
                faltam += usize::from(!dentro_);
            }
        }
    }
    assert!(
        so_dele > 0,
        "o CONTROLO: nenhum ponto do contorno só o avesso tapa"
    );
    assert_eq!(
        faltam, 0,
        "{faltam} de {so_dele} pontos do contorno sumiram pelo avesso"
    );
}
