//! ⭐⭐ **«Mostra o cartão desta família»** — o clique num ícone de componente da Hierarquia
//! (doc 01 §7.1) atendido aqui: rola o corpo até ao PRIMEIRO cartão pintado da família e abre-o.
//!
//! ⚠️ A família de um cartão é DERIVADA do catálogo pelo componente que ele edita
//! ([`SECTION_COMPONENT`]); é a mesma resposta que escolhe o ícone da linha, então o ícone e o
//! cartão não podem discordar.

use std::cell::RefCell;

use ph2d_a11y::NodeId;
use ph2d_component_desc::ComponentCategory;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;

/// O componente que cada secção EDITA (o principal, quando são vários) — um por [`ids::LIVE_SECTIONS`].
pub(crate) const SECTION_COMPONENT: [(NodeId, &str); 47] = [
    (ids::INSP_LIVE_NAME_SECTION, "ph2d::ecs::Name"),
    (ids::INSP_LIVE_VISIBILITY_SECTION, "ph2d::ecs::Visibility"),
    (ids::INSP_LIVE_TRANSFORM_SECTION, "ph2d::ecs::Transform"),
    (ids::INSP_LIVE_RENDER_SECTION, "ph2d::render::Sprite"),
    (ids::INSP_LIVE_COLOR_SECTION, "ph2d::ecs::SpriteCornerTint"),
    (ids::INSP_LIVE_SHEET_SECTION, "ph2d::ecs::SpriteGrid"),
    (ids::INSP_LIVE_ORDERING_SECTION, "ph2d::ecs::OrderInLayer"),
    (ids::INSP_LIVE_SAMPLING_SECTION, "ph2d::ecs::TextureFilter"),
    (ids::INSP_LIVE_SLICE_SECTION, "ph2d::ecs::SliceNine"),
    (ids::INSP_LIVE_ANCHOR_SECTION, "ph2d::ecs::NamedAnchorList"),
    (ids::INSP_LIVE_ANIM_SECTION, "ph2d::ecs::SpriteAnimator"),
    (ids::INSP_LIVE_BLEND_SECTION, "ph2d::ecs::BlendMode"),
    (ids::INSP_LIVE_PHYSICS_SECTION, "ph2d::physics::RigidBody"),
    (ids::INSP_LIVE_JOINT_SECTION, "ph2d::physics::PhysicsJoint"),
    (ids::INSP_LIVE_WHEEL_SECTION, "ph2d::physics::PulleyWheel"),
    (
        ids::INSP_LIVE_PLAYER_SECTION,
        "ph2d::physics::PlatformPlayer",
    ),
    (ids::INSP_LIVE_TIMER_SECTION, "ph2d::ecs::Timers"),
    (ids::INSP_LIVE_ACTION_SECTION, "ph2d::ecs::SignalActions"),
    (ids::INSP_LIVE_AUDIO_SECTION, "ph2d::ecs::AudioSource2D"),
    (ids::INSP_LIVE_CAMERA_SECTION, "ph2d::ecs::GameCamera"),
    (ids::INSP_LIVE_TAGS_SECTION, "ph2d::ecs::Tags"),
    (ids::INSP_LIVE_FACTORY_SECTION, "ph2d::ecs::Factory"),
    (ids::INSP_LIVE_LIFECYCLE_SECTION, "ph2d::ecs::Lifetime"),
    (
        ids::INSP_LIVE_TOPDOWN_SECTION,
        "ph2d::physics::TopDownPlayer",
    ),
    (
        ids::INSP_LIVE_PROJECTILE_SECTION,
        "ph2d::physics::ProjectileMotion",
    ),
    (ids::INSP_LIVE_RAY_SECTION, "ph2d::physics::RaySensor"),
    (ids::INSP_LIVE_SM_SECTION, "ph2d::ecs::StateMachine"),
    (ids::INSP_LIVE_SCRIPT_SECTION, "ph2d::script::LuauScript"),
    (
        ids::INSP_LIVE_PARTICLES_SECTION,
        "ph2d::ecs::ParticleEmitter",
    ),
    (ids::INSP_LIVE_HUD_SECTION, "ph2d::ecs::UiCanvas"),
    (ids::INSP_LIVE_SEQ_SECTION, "ph2d::ecs::SequencePlayer"),
    (ids::INSP_LIVE_WATCH_SECTION, "ph2d::ecs::CounterWatch"),
    (ids::INSP_LIVE_TRIGGER_SECTION, "ph2d::ecs::SignalOnAction"),
    (ids::INSP_LIVE_TWEEN_SECTION, "ph2d::ecs::Tweens"),
    (ids::INSP_LIVE_PATHFOLLOW_SECTION, "ph2d::ecs::PathFollow"),
    (ids::INSP_LIVE_SHAKE_SECTION, "ph2d::ecs::CameraShake"),
    (ids::INSP_LIVE_EMITTER_SECTION, "ph2d::ecs::ShakeEmitter"),
    (ids::INSP_LIVE_WEAPON_SECTION, "ph2d::ecs::WeaponFire"),
    (ids::INSP_LIVE_PARALLAX_SECTION, "ph2d::ecs::ScrollFactor"),
    (ids::INSP_LIVE_HEALTH_SECTION, "ph2d::physics::Health"),
    (ids::INSP_LIVE_DAMAGE_SECTION, "ph2d::physics::Damage"),
    (
        ids::INSP_LIVE_HEALTH_BAR_SECTION,
        "ph2d::physics::HealthBar",
    ),
    (
        ids::INSP_LIVE_NAV_REGION_SECTION,
        "ph2d::physics::NavRegion",
    ),
    (ids::INSP_LIVE_NAV_AGENT_SECTION, "ph2d::physics::NavAgent"),
    (
        ids::INSP_LIVE_NAV_COST_AREA_SECTION,
        "ph2d::physics::NavCostArea",
    ),
    (ids::INSP_LIVE_NAV_LINK_SECTION, "ph2d::physics::NavLink"),
    (ids::INSP_LIVE_LUZ_SECTION, "ph2d::light2d::Luz"),
];

/// A família de uma secção, pelo catálogo.
pub(crate) fn category_of(section: NodeId) -> Option<ComponentCategory> {
    let (_, name) = SECTION_COMPONENT.iter().find(|(id, _)| *id == section)?;
    Some(ph2d_component_desc::desc_for(name)?.category)
}

thread_local! {
    /// As secções pintadas neste quadro: `(id, y do topo no ecrã)`, pela ordem do corpo.
    static BANDS: RefCell<Vec<(NodeId, f32)>> = const { RefCell::new(Vec::new()) };
    /// O `y` em que o conteúdo começa (o topo do corpo menos a rolagem).
    static CONTENT_TOP: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

/// O plano publica onde pintou cada secção ([`crate::plano::Plano::run`]).
pub(crate) fn record_bands(bands: impl Iterator<Item = (NodeId, f32)>) {
    BANDS.with(|b| {
        let mut b = b.borrow_mut();
        b.clear();
        b.extend(bands);
    });
}

/// O corpo publica onde o conteúdo começa.
pub(crate) fn record_content_top(top: f32) {
    CONTENT_TOP.with(|c| c.set(top));
}

/// ⭐ **Atende o pedido**, se ele é da entidade que o corpo acabou de pintar (`painted`, a dos
/// retratos do quadro — `InspectorState::last_entity`): rola até ao primeiro cartão da família e
/// abre-o. Um pedido de OUTRA entidade — mesmo com o mesmo nome — espera o quadro em que a selecção
/// chega; um da mesma sem cartão daquela família morre aqui.
pub(crate) fn serve(store: &mut WidgetStore, painted: Option<u64>) {
    let Some(req) = store.inspector_reveal().copied() else {
        return;
    };
    if painted != Some(req.entity) {
        return;
    }
    let category = req.category;
    let top = CONTENT_TOP.with(std::cell::Cell::get);
    // ⭐ O cartão pára onde o PRIMEIRO pára sem rolagem — com a mesma margem de cima (medido: no
    // topo cru do corpo ele ficava 17 px acima do sítio de qualquer cartão).
    let (hit, lead) = BANDS.with(|b| {
        let b = b.borrow();
        let hit = b
            .iter()
            .find(|(id, _)| category_of(*id) == Some(category))
            .copied();
        (hit, b.first().map_or(0.0, |(_, y0)| *y0 - top))
    });
    let Some((section, y)) = hit else {
        let _ = store.take_inspector_reveal();
        return;
    };
    // ⚠️ FECHADO, abre-se primeiro e rola-se no quadro seguinte: o clamp da rolagem
    // (`Pending::publish`) mede a altura DESTE quadro, e com o cartão fechado ela não chega para o
    // pôr no topo — medido, o cabeçalho ficava no fundo do corpo.
    if store.is_collapsed(section) {
        store.set_collapsed(section, false);
        return;
    }
    let _ = store.take_inspector_reveal();
    store.set_panel_scroll(ids::INSP_PANEL, (y - top - lead).max(0.0));
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::hier_rows::component_icon;

    #[test]
    fn toda_secao_tem_o_seu_componente_no_catalogo() {
        let ours: std::collections::BTreeSet<NodeId> =
            SECTION_COMPONENT.iter().map(|(id, _)| *id).collect();
        let live: std::collections::BTreeSet<NodeId> = ids::LIVE_SECTION_IDS.into_iter().collect();
        assert_eq!(ours, live, "a tabela e a LIVE_SECTIONS divergem");
        for (_, name) in SECTION_COMPONENT {
            assert!(
                ph2d_component_desc::desc_for(name).is_some(),
                "`{name}` não está no catálogo"
            );
        }
    }

    #[test]
    fn todo_icone_da_linha_tem_um_cartao_para_onde_ir() {
        for c in ComponentCategory::ALL {
            if component_icon(c).is_some() {
                assert!(
                    SECTION_COMPONENT
                        .iter()
                        .any(|(id, _)| category_of(*id) == Some(c)),
                    "a linha mostra um ícone de {c:?} e nenhum cartão do Inspector é dessa família"
                );
            }
        }
    }
}
