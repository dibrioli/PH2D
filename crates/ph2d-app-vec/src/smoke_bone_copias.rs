//! ⭐⭐ **AS CÓPIAS NA DOBRA FORTE** — `PH2D_VEC_BONE_SMOKE=6` (A6, 2026-10-04): duas barras presas
//! feitas de DUAS cópias que se sobrepõem (um *Repeater* com giro — a união do contacto não corre),
//! a 2.ª com *Hatch* por cima, dobradas em S a [`DOBRA`]. Numa dobra forte o contorno da parte de
//! trás deixava de riscar por cima da parte da frente; as riscas também.
//!
//! ⚠️ **A dobra é aplicada DEPOIS de prender** — prender fotografa a pose de repouso.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::{ShapeKind, VecPathId, VecScene, cook_tinted as shape};

/// A dobra por junta, em graus — a forte em que os membros já se sobrepõem.
pub(crate) const DOBRA: f32 = 110.0;

/// O centro (recto) de cada barra.
pub(crate) const ORIGENS: [[f64; 2]; 2] = [[-2.6, -0.6], [2.6, -0.6]];

/// O nome de cada barra (e dos ossos dela) na lista Hierarchy.
const NOMES: [&str; 2] = ["Copias", "Copias com Hatch"];

/// As duas pilhas: só as cópias, e as cópias com *Hatch*.
fn pilha(com_hatch: bool) -> Vec<FxEntry> {
    let mut p = vec![FxEntry::new(PathEffect::Repeat(
        ph2d_vec_scene::fx_repeat::RepeatSpec {
            copies_x: 1.0,
            move_x: 0.0,
            copies_y: 2.0,
            move_y: 60.0,
            spin: 5.0,
            orbit: 0.0,
        },
    ))];
    if com_hatch {
        p.push(FxEntry::new(PathEffect::Hatch(
            ph2d_vec_scene::fx_hatch::HatchSpec {
                angle: 45.0,
                spacing: 8.0,
                cross: false,
            },
        )));
    }
    p
}

/// O 1.º tempo: as duas barras e os dois esqueletos, rectos.
pub(crate) fn build(scene: &mut VecScene, sim: &mut SimWorld, st: &mut crate::state::VecState) {
    let (l, t) = crate::smoke_bone_efeitos::PECA;
    let mut pend: Vec<(VecPathId, Option<Entity>)> = Vec::new();
    for (k, o) in ORIGENS.iter().enumerate() {
        let mut p = shape(
            ShapeKind::RoundRect,
            [o[0] - l / 2.0, o[1] - t / 2.0],
            [o[0] + l / 2.0, o[1] + t / 2.0],
            &[t / 2.0],
            crate::smoke_bone_par::COR,
        );
        let c = crate::smoke_bone_par::CONTORNO;
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(c[0], c[1], c[2], 255),
            t * crate::smoke_bone_par::ESPESSURA_DO_CONTORNO,
        ));
        p.effects = pilha(k == 1);
        let id = scene.push_path(p);
        pend.push((id, crate::smoke_bone_efeitos::esqueleto(sim, *o, NOMES[k])));
    }
    st.bone_smoke_pend = Some(pend);
    st.bone_smoke_step = 1;
}

/// O 2.º tempo: prende cada barra ao seu esqueleto RECTO e dobra.
pub(crate) fn bind(scene: &mut VecScene, sim: &mut SimWorld, st: &mut crate::state::VecState) {
    st.bone_smoke_step = 2;
    let Some(pecas) = st.bone_smoke_pend.take() else {
        return;
    };
    let mut presas = 0;
    for ((id, raiz), nome) in pecas.iter().zip(NOMES) {
        presas += ph2d_skeleton_live::skin_live::bind(sim, scene, &st.entities, &[*id], *raiz);
        if let Some(e) = st.entities.get(id).and_then(|b| Entity::try_from_bits(*b)) {
            sim.world_mut()
                .entity_mut(e)
                .insert(ph2d_ecs::Name::new(nome));
        }
        if let Some(r) = raiz {
            crate::smoke_bone_par::dobra_duas(sim, *r, DOBRA, DOBRA);
        }
    }
    if presas != pecas.len() {
        eprintln!("[vec-bone-smoke] PARE: so' {presas} de {} barras prenderam", pecas.len());
    }
    println!(
        "[vec-bone-smoke] AS COPIAS NA DOBRA FORTE: duas barras feitas de duas copias que se \
         sobrepoem, presas e dobradas em S ({DOBRA}° por junta); a da direita tem riscas.\n\
         [vec-bone-smoke] 1) Em cada dobra, a parte que vem por cima TAPA a de baixo: o contorno e \
         as riscas da parte de baixo param onde a de cima passa -- nada de linhas a atravessar por \
         cima."
    );
}

#[cfg(test)]
#[path = "smoke_bone_copias_tests.rs"]
mod tests;
