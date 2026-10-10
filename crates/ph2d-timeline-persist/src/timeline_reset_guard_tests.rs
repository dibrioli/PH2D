//! A guarda do reset (`purge_the_dead`): um osso inteligente que ainda ENCONTRA o seu clip segura a
//! composição; um homónimo de outro dono não (doc ObjectsAndComponents 01 §15, onda 5 item d).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_skeleton_ecs::SmartBone;
use ph2d_timeline::{PropKind, SCENE, TimelineIntent as I, apply_intent};

/// O Herói animado (uma pista + o clip `andar` dele) e um osso inteligente solto que nomeia `clip`.
/// Apaga o Herói e devolve se o `upkeep` reiniciou a timeline.
fn apaga_o_heroi_com_um_osso_a_nomear(clip: &str, clip_da_cena: Option<&str>) -> bool {
    let mut sim = SimWorld::new();
    let heroi = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("Herói")))
        .id();
    sim.world_mut().spawn((
        ph2d_ecs::Transform::IDENTITY,
        Name::new("osso"),
        SmartBone {
            clip: clip.into(),
            ..SmartBone::default()
        },
    ));
    let mut timeline = TimelineState::with_default_duration();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut timeline,
        &mut ph,
        I::AddKey {
            entity: heroi.to_bits(),
            prop: PropKind::TranslationX,
            t: ph2d_anim::RationalTime::from_seconds(0.0),
            value: ph2d_anim::AnimValue::Float(1.0),
            interp: ph2d_anim::Interp::Linear,
        },
    );
    timeline
        .doc
        .add_clip_for("andar".into(), heroi.to_bits())
        .unwrap();
    if let Some(nome) = clip_da_cena {
        timeline.doc.add_clip_for(nome.into(), SCENE).unwrap();
    }
    upkeep(&mut timeline, sim.world_mut());

    sim.world_mut().despawn(heroi);
    ph2d_timeline::apply_from_doc(sim.world_mut(), &mut timeline.doc, 0.0);
    upkeep(&mut timeline, sim.world_mut())
}

/// ⛔ O defeito: o osso solto vive na animação da CENA, que não tem `andar`; o `andar` era do
/// Herói, que saiu. Antes a guarda via o NOME no mundo e conservava a composição rançosa.
#[test]
fn um_homonimo_de_outro_dono_nao_segura_o_reset() {
    assert!(
        apaga_o_heroi_com_um_osso_a_nomear("andar", None),
        "o osso não encontra «andar» na animação onde vive — nada a conservar"
    );
}

/// A lei de 2026-09-08 continua: o osso que AINDA encontra o seu clip segura o documento.
#[test]
fn o_osso_que_encontra_o_clip_da_cena_segura_o_reset() {
    assert!(
        !apaga_o_heroi_com_um_osso_a_nomear("acenar", Some("acenar")),
        "o clip da Cena que o osso percorre não pode sair com o reset"
    );
}

/// Controlo: sem osso a nomear nada, o último objecto animado reinicia (o reset vive).
#[test]
fn sem_dependencia_o_reset_acontece() {
    assert!(apaga_o_heroi_com_um_osso_a_nomear("", None));
}
