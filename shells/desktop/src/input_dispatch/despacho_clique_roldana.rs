//! **A roldana sob o cursor vira a seleção** — a porta que o `ramo_gizmo_pivo_e_ancora` (`despacho_clique_gizmo.rs`)
//! chama, mudada VERBATIM do índice (`line/input-dispatch`, 2026-09-13). ⚠️ Não mora ao lado do ramo porque o
//! `despacho_clique_gizmo.rs` passaria o tecto de 600 linhas (582 + 36 = 618, medido).

/// **A roldana sob o cursor vira a SELEÇÃO** (W-RopeStop) — o pedido do Enio
/// *"permita selecionar as polias com mouse no canvas"*.
///
/// Até aqui uma roldana só era alcançável pela Hierarquia: ela não tem sprite,
/// então o `pick_sprites_at_world` não a vê, e as alças dela (centro/aro) só
/// nascem DEPOIS de ela estar selecionada — o laço em que a única porta de
/// entrada era uma lista de nomes.
///
/// ⚠️ **A tolerância é a MESMA `SNAP_PX` do ímã de âncora e do conta-gotas de
/// corda**, convertida em mundo pelo zoom: um app onde dois alvos de canvas
/// respondem a distâncias diferentes é um app que se aprende duas vezes.
///
/// Devolve se alguma roldana foi de fato selecionada — o chamador usa isso para
/// consumir o Down, como faz com o pivô e com a âncora.
/// ⚠️ Toma os TRÊS pedaços do `AppGfx` de que precisa, e não `&AppGfx`: quem
/// chama já segura um `&mut` em `gfx.hero_screen` — empréstimos por CAMPO são
/// disjuntos, um reborrow da struct inteira não é. A mesma assinatura, pelo mesmo
/// motivo, que o `joint_anchor_drag::open_drag`.
pub(super) fn select_wheel_at(
    physics: &ph2d_physics_ecs::PhysicsBridge,
    camera: &ph2d_render::Camera2d,
    win: ph2d_host::WindowSize,
    hero: &mut ph2d_editor_core::HeroScreen,
    at: (f32, f32),
) -> bool {
    let w = camera.screen_to_world(at, win);
    let tol =
        ph2d_app_physics::joint_anchor_drag::SNAP_PX * camera.height_world / win.height as f32;
    let Some(wheel) = physics.wheel_at_world(w, tol) else {
        return false;
    };
    hero.gizmo.selection = Some(wheel.to_bits());
    hero.gizmo.extra_selection.clear();
    true
}
