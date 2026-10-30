//! ⭐⭐ **O LOSANGO DO PAINEL DO ESQUELETO** — `PH2D_VEC_BONE_SMOKE=8` (F40).
//!
//! O que está na tela, e o que cada coisa é:
//!
//! | O quê | O que é |
//! |---|---|
//! | **A barra laranja** (o braço da cena `=1`, já presa a 3 ossos) | o desenho que os ossos dobram e esticam |
//! | **O osso do MEIO, já escolhido** | o «antebraço»: o painel *Skeleton* mostra os números DELE |
//! | **A coluna de losangos** à direita de *Length*, *Segments* e das 4 alças da curva | cada losango diz se aquele número tem animação: cheio = há chave neste quadro · contorno = animado entre chaves · vazio = sem animação |
//! | **A animação já gravada** | *Length* tem chave no quadro 0 (o comprimento desenhado), no 24 (1,5×) e no 48 (volta): no Play o osso azul do meio cresce e encolhe |
//!
//! ⭐ **A LEI (dono 30/10, como o Blender/Spine):** no Play o osso do meio cresce, EMPURRA o osso
//! seguinte (a raiz dele fica na ponta nova) e ESTICA a barra presa ao longo do eixo. A mão
//! (`ph2d_app_skeleton::comprimento::aplica`) e a curva (`canais_osso::aplicar`) correm a MESMA lei
//! (`ph2d_skeleton_live::comprimento::empurra_filhos`); a pele estica por `agora / bind`
//! (`Tendon::comprimento`). ⛔ Até F40 o filho e a barra ficavam — a cena ensinava a lei velha.
//!
//! ⚠️ O osso do meio nasce com 4 segmentos: com 1 a curva é inerte e o painel esconde as alças
//! (`section_campos`), e a cena não as mostraria.
//!
//! Monta em dois tempos como a `=1`: prender exige a entidade da forma (`vec_entities::sync`).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, FieldKey, SimWorld, canal::familia::OSSO, canal_osso};
use ph2d_skeleton_demo::{ARM_A, ARM_B, ARM_BONES, cadeia};
use ph2d_timeline::{PropKind, TimelineDoc};
use ph2d_vec_scene::{ShapeKind, VecScene, cook_tinted as shape};

/// Os segmentos do osso do meio — o bastante para as alças da curva aparecerem no painel.
pub(crate) const SEGMENTOS_DO_MEIO: u8 = 4;
/// Quanto o braço anda para a direita (m) em relação à `=1`: lá ele mora à esquerda, ao lado de
/// outras peças; sozinho ele fica ao centro da câmera de omissão (sem *Frame All*, que corta com a
/// timeline aberta).
pub(crate) const DESLOCA: f64 = 5.0;
/// Quanto o *Length* cresce na chave do meio.
pub(crate) const CRESCE: f64 = 1.5;
/// Os quadros das três chaves: autorado · crescido · autorado.
pub(crate) const QUADROS: [i64; 3] = [0, 24, 48];
/// O canal do *Length* (família OSSO).
#[must_use]
pub(crate) const fn canal_comprimento() -> PropKind {
    PropKind::Field(FieldKey::new(OSSO, canal_osso::COMPRIMENTO, 0, 0))
}

/// O 1.º tempo: a barra e a corrente de 3 ossos do braço; o do meio com segmentos.
pub(crate) fn build(scene: &mut VecScene, sim: &mut SimWorld, st: &mut crate::state::VecState) {
    let (a, b, r) = crate::smoke_bone::BARRA;
    let d = |p: [f64; 2]| [p[0] + DESLOCA, p[1]];
    let braco = scene.push_path(shape(
        ShapeKind::RoundRect,
        d(a),
        d(b),
        &[r],
        [230, 170, 90],
    ));
    let raiz = cadeia(sim, d(ARM_A), d(ARM_B), ARM_BONES);
    if let Some(m) = raiz.and_then(|r| crate::smoke_bone::osso_do_meio(sim, r))
        && let Some(mut o) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(m)
    {
        o.segments = SEGMENTOS_DO_MEIO;
    }
    st.bone_smoke_pend = Some(vec![(braco, raiz)]);
    st.bone_smoke_step = 1;
}

/// O 2.º tempo: prende a barra, grava as chaves do *Length* no osso do meio e põe-no em foco.
pub(crate) fn bind(
    scene: &mut VecScene,
    sim: &mut SimWorld,
    doc: &mut TimelineDoc,
    st: &mut crate::state::VecState,
) {
    st.bone_smoke_step = 2;
    let Some(pecas) = st.bone_smoke_pend.take() else {
        return;
    };
    let Some(&(braco, Some(raiz))) = pecas.first() else {
        return;
    };
    ph2d_skeleton_live::skin_live::bind(sim, scene, &st.entities, &[braco], Some(raiz));
    let Some(meio) = semear(sim, doc, raiz) else {
        println!("[vec-bone-smoke] o osso do MEIO nao existe -- PARE, a cena nao montou");
        return;
    };
    st.bone_smoke_foco = Some(meio.to_bits());
    println!(
        "[vec-bone-smoke] O LOSANGO DO ESQUELETO: a barra laranja e' um braco preso a 3 ossos, e o \
         do MEIO ja' esta' escolhido -- o painel «Skeleton» a' direita mostra os numeros dele.\n\
         [vec-bone-smoke] 1) Ao lado de «Length» o losango esta' CHEIO: ha' chave neste quadro \
         (0). «Segments» e as 4 alcas da curva tem o losango VAZIO: sem animacao.\n\
         [vec-bone-smoke] 2) Carregue Play na Timeline: o osso azul do meio cresce ate' 1,5x no \
         quadro 24 e volta no 48 -- ele EMPURRA o osso seguinte e ESTICA a barra laranja junto \
         (igual a escrever o numero a' mao); entre chaves o losango de «Length» fica so' com o \
         contorno.\n\
         [vec-bone-smoke] 3) Pare num quadro sem chave, clique no losango vazio de «Segments»: ele \
         enche (chave nova)."
    );
}

/// ⭐ As chaves do *Length* no osso do MEIO; devolve-o.
pub(crate) fn semear(sim: &SimWorld, doc: &mut TimelineDoc, raiz: Entity) -> Option<Entity> {
    let meio = crate::smoke_bone::osso_do_meio(sim, raiz)?;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "fps da régua"
    )]
    let fps = doc.fps_display.round().max(1.0) as u32;
    let l0 = sim.world().get::<ph2d_skeleton_ecs::Bone>(meio)?.length;
    for (q, l) in QUADROS.into_iter().zip([l0, l0 * CRESCE, l0]) {
        #[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
        doc.insert_key(
            meio.to_bits(),
            canal_comprimento(),
            RationalTime::from_frame(q, fps),
            AnimValue::Float(l as f32),
            Interp::Linear,
        );
    }
    Some(meio)
}

/// ⭐ **O osso que o prólogo põe em foco** — o da cena (`=8`, lido 1×) ou, com a sonda da foto do
/// peso (`PH2D_VEC_WEIGHT_PROBE`), o «Bone 2» (o do meio da barra laranja, o único cuja rampa cabe
/// no enquadramento — medido por `diag_qual_osso_mostra_mais`) ou o «Bone 14» com `=2`.
pub fn osso_em_foco(sim: &ph2d_ecs::SimWorld, st: &mut crate::state::VecState) -> Option<u64> {
    if let Some(b) = st.bone_smoke_foco.take() {
        return Some(b);
    }
    let sonda = std::env::var("PH2D_VEC_WEIGHT_PROBE").ok()?;
    let alvo = if sonda == "2" { "Bone 14" } else { "Bone 2" };
    sim.world()
        .iter_entities()
        .find(|er| {
            er.get::<ph2d_skeleton_ecs::Bone>().is_some()
                && er.get::<ph2d_ecs::Name>().is_some_and(|n| n.0 == alvo)
        })
        .map(|er| er.id().to_bits())
}

/// A foto do meio do Play (`fotografa_cena.sh` não clica): `=8` com `PH2D_VEC_BONE_SMOKE_TOCA=1`
/// abre a tocar; sem ela abre parada no quadro 0.
#[must_use]
pub fn toca_no_arranque() -> bool {
    crate::smoke_bone::nivel() == 8
        && std::env::var("PH2D_VEC_BONE_SMOKE_TOCA").is_ok_and(|v| v == "1")
}

#[cfg(test)]
#[path = "smoke_bone_losango_tests.rs"]
mod tests;
