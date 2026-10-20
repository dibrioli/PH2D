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
        pose[i] = sample_prop_value(src, entity, prop).and_then(|v| match v {
            AnimValue::Float(f) => Some(f),
            _ => None,
        });
    }
    pose
}

/// ⭐⭐⭐ **DE QUEM ESTE PASSE OLHA A POSE: a SELECÇÃO mais o que a MÃO segura.**
///
/// ⛔⛔ **A selecção sozinha perdia a pose que o artista acabou de fazer** (report do dono,
/// 2026-09-14). Puxar a PONTA de uma corrente é cinemática inversa: a corrente **inteira** dobra —
/// e agarrar um osso **não** o selecciona (o `despacho_clique_select` escreve `bone_pose` e
/// devolve). Com a selecção sozinha, a animação ficava com **um osso** da pose e o resto perdia-se
/// no primeiro instante em que o apply voltasse a escrever pelas curvas.
///
/// ⭐ **A população certa já era CALCULADA** — a [`super::super::timeline_bridge::maos_do_quadro`] nasceu
/// na W8 para o apply *não* escrever por cima da mão, e é exactamente a mesma pergunta. *Uma porta
/// com um consumidor só estava a metade do trabalho que sabia fazer.*
///
/// ⚠️ **Isto NÃO espalha chaves pelo esqueleto:** quem filtra é o DIFF do [`super::apply_samples`] — um
/// osso cuja pose é a da curva não cunha nada. A mão diz *«olha também para estes»*, nunca
/// *«cunha estes»*.
///
/// ⚠️ **A ordem é a da SELECÇÃO primeiro** (é o que o diff e a primeira recusa do quadro esperam), a
/// mão a seguir, **sem repetidos** — o osso agarrado costuma estar nas duas listas.
///
/// ⛔ **Menos quem um MOTOR está a conduzir** (auditoria de 2026-09-08): a invariante do cabeçalho
/// deste ficheiro exige que ninguém escreva pose **entre** o apply e este passe, e os passes do
/// esqueleto (o osso inteligente, a âncora de IK) escrevem exactamente aí — a chave nasceria feita
/// da saída do motor. *Pré-visualização não é autoria.*
#[must_use]
pub(crate) fn populacao(
    hero: &HeroScreen,
    skeleton: &ph2d_app_skeleton::state::SkeletonState,
    sim: &ph2d_ecs::SimWorld,
    preview: &ph2d_preview_drive::PreviewDrive,
) -> Vec<u64> {
    let maos = super::super::timeline_bridge::maos_do_quadro(Some(hero), skeleton, sim);
    let mut fora: Vec<u64> = Vec::new();
    for e in hero.gizmo.iter_selected().chain(maos.iter().copied()) {
        if !preview.drives(e) && !fora.contains(&e) {
            fora.push(e);
        }
    }
    fora
}

/// ⭐ **Os campos de família que `entity` publicou neste quadro** (`ph2d_ecs::CanaisLidos`) — o que
/// o autokey compara com a curva, ao lado da pose.
#[must_use]
pub(crate) fn campos_lidos(
    sim: &ph2d_ecs::SimWorld,
    entity: u64,
) -> Vec<(ph2d_ecs::FieldKey, f32)> {
    sim.world()
        .get_resource::<ph2d_ecs::CanaisLidos>()
        .map(|l| l.campos(entity).collect())
        .unwrap_or_default()
}
