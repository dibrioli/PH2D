//! ⭐⭐⭐ **A CENA DOS TRÊS MODOS** — `PH2D_VEC_BONE_SMOKE=7` (A14, o esqueleto é um objecto).
//!
//! Uma barra SOLTA e um esqueleto-OBJECTO de dois ossos ao longo dela, nada preso e nada
//! seleccionado: o artista aprende os modos fazendo — em Object clica no osso (selecciona o
//! esqueleto), junta a barra e prende (`Ctrl+P` ou *Bind to Skeleton*); `Tab` entra no Edit (arrastar
//! da ponta cria um osso); o seletor *Mode* ▸ *Pose* posa (arrastar um osso dobra a barra); `Tab`
//! volta a Object, onde o gizmo move o esqueleto inteiro com a barra.

use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_vec_scene::{ShapeKind, VecScene, cook_tinted as shape};

/// Quantos ossos a corrente tem.
pub(crate) const OSSOS: u32 = 2;

/// O 1.º tempo: a barra e o esqueleto.
pub(crate) fn build(scene: &mut VecScene, sim: &mut SimWorld, st: &mut crate::state::VecState) {
    let (l, t) = crate::smoke_bone_efeitos::PECA;
    let mut p = shape(
        ShapeKind::RoundRect,
        [-l / 2.0, -t / 2.0],
        [l / 2.0, t / 2.0],
        &[t / 2.0],
        crate::smoke_bone_par::COR,
    );
    let c = crate::smoke_bone_par::CONTORNO;
    p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
        ph2d_vec_scene::Rgba8::new(c[0], c[1], c[2], 255),
        t * crate::smoke_bone_par::ESPESSURA_DO_CONTORNO,
    ));
    let barra = scene.push_path(p);
    let esq = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new(ph2d_i18n::tr("object_add.skeleton")),
            ph2d_skeleton_ecs::Skeleton,
        ))
        .id();
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    let x0 = -l / 2.0 + t / 2.0;
    let passo = (l - t) / f64::from(OSSOS);
    let mut pai = esq;
    for k in 0..OSSOS {
        let a = [x0 + passo * f64::from(k), 0.0];
        let b = [x0 + passo * f64::from(k + 1), 0.0];
        let Some(e) = ph2d_skeleton_live::bone::create(sim, Some(pai), a, b) else {
            return;
        };
        pai = Entity::from_bits(e);
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // ⚠️ A barra entra na espera do 2.º tempo (a shell só avança com a entidade dela): sem isto o
    // prólogo — o painel de ossos, o enquadramento — nunca corria.
    st.bone_smoke_pend = Some(vec![(barra, None)]);
    st.bone_smoke_step = 1;
}

/// O 2.º tempo: nada a prender (é o gesto que a cena ensina) — só o roteiro no terminal.
pub(crate) fn bind(st: &mut crate::state::VecState) {
    st.bone_smoke_step = 2;
    st.bone_smoke_pend = None;
    println!(
        "[vec-bone-smoke] OS TRES MODOS DO ESQUELETO: uma barra solta e um esqueleto de dois \
         ossos.\n[vec-bone-smoke] 1) Object: clique num osso (o esqueleto fica seleccionado), \
         Shift+clique na barra, Ctrl+P (prende).\n[vec-bone-smoke] 2) Tab: Edit -- arraste da \
         ponta do ultimo osso e nasce outro.\n[vec-bone-smoke] 3) Mode > Pose Mode: arraste um \
         osso e a barra dobra.\n[vec-bone-smoke] 4) Tab: Object -- o gizmo move o esqueleto \
         inteiro, e a barra vai junto."
    );
}

#[cfg(test)]
#[path = "smoke_bone_modos_tests.rs"]
mod tests;
