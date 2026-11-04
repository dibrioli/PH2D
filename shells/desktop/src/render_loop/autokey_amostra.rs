//! **O QUE o auto-key amostra, e DE QUEM** — irmão de [`super`] por tecto de LOC (a shell, 600): a
//! pose dos 11 canais de sempre, os campos que as famílias publicaram (plano 35 da UI/UX) e a
//! população. O passe lê daqui; o corte é por responsabilidade («o que se lê» × «o que se grava»).

use ph2d_anim::AnimValue;
use ph2d_editor_core::HeroScreen;
use ph2d_timeline::{PoseSample, PropKind};

use super::super::timeline_bridge::sample_prop_value;

/// Sample a sprite's six animatable values into a [`PoseSample`], in
/// `PropKind::ALL` order. `None` for a property whose backing component is absent
/// (e.g. opacity on an entity with no `Sprite`).
pub(super) fn sample_pose<'a>(
    src: impl Into<super::super::timeline_bridge::Mundo<'a>>,
    entity: u64,
) -> PoseSample {
    let src = src.into();
    let mut pose: PoseSample = [None; PropKind::AUTOKEYED.len()];
    for (i, &prop) in PropKind::AUTOKEYED.iter().enumerate() {
        // ⛔ O comprimento que a ponte semeou da rota de uma POLIA não é da mão (F26); o que a mão
        // digita depois é (F30 §D).
        if prop == PropKind::JointMaxLength
            && ph2d_app_physics::canais_junta::comprimento_da_rota(src.world, entity)
        {
            continue;
        }
        pose[i] = sample_prop_value(src, entity, prop).and_then(|v| match v {
            AnimValue::Float(f) => Some(f),
            _ => None,
        });
    }
    pose
}

/// ⭐⭐⭐ **DE QUEM ESTE PASSE OLHA A POSE:** a SELECÇÃO, o que a MÃO segura (`maos_do_quadro`: puxar a
/// ponta de uma corrente dobra-a inteira, e agarrar um osso não o selecciona; report do dono, 14/09),
/// o que a tool aponta (`AlvosDasFamilias`) e cada objecto PRESO por um cartão solto, sem repetidos.
///
/// ⚠️ Isto não espalha chaves: quem filtra é o DIFF do [`super::apply_samples`]. A ordem é a da selecção
/// primeiro (a primeira recusa do quadro). ⛔ Menos quem um MOTOR conduz: os passes do esqueleto
/// escrevem entre o apply e este passe, e a chave nasceria da saída do motor (auditoria de 08/09).
#[must_use]
pub(crate) fn populacao(
    hero: &HeroScreen,
    skeleton: &ph2d_app_skeleton::state::SkeletonState,
    sim: &ph2d_ecs::SimWorld,
    preview: &ph2d_preview_drive::PreviewDrive,
) -> Vec<u64> {
    let maos = super::super::timeline_bridge::maos_do_quadro(Some(hero), skeleton, sim);
    let mut fora: Vec<u64> = Vec::new();
    let alvos = sim.world().get_resource::<ph2d_ecs::AlvosDasFamilias>();
    let alvos = alvos
        .into_iter()
        .flat_map(ph2d_ecs::AlvosDasFamilias::todas);
    let pinos = ph2d_host_presos::losango::pinos(&hero.store);
    for e in hero
        .gizmo
        .iter_selected()
        .chain(maos.iter().copied())
        .chain(alvos)
        .chain(pinos)
    {
        if !preview.drives(e) && !fora.contains(&e) {
            fora.push(e);
        }
    }
    fora.extend(ph2d_ecs::bits_da_cena(sim.world())); // ⭐ e a CENA, sempre (F30 §B)
    fora
}
