//! **Os conta-gotas do Inspector que escolhem um objecto no palco** — o Target da bala (GEF 06) e o
//! «Ignore up to», o «Aim at» e o «Also ignore» do Ray Sensor (GEF 07 C8, RS-3, RS-12), e o From do Motion Input (GEF 08). A shell compõe UMA porta: um clique, um `Esc`, um cursor.

use ph2d_ecs::World;
use ph2d_editor_core::action_bus::ActionBus;

/// Algum está armado (o cursor de mira).
#[must_use]
pub fn armado(world: &World) -> bool {
    crate::projectile_inspector::armado(world)
        || crate::ray_topo::armado(world, None)
        || ph2d_ray_sensor::gotas::armado(world, None)
        || ph2d_ray_sensor::gotas_from::armado(world, None)
}

/// Desarma todos (o `Esc`); `true` = algum estava armado.
pub fn desarma(world: &mut World) -> bool {
    crate::projectile_inspector::desarma(world).is_some()
        | crate::ray_topo::desarma(world).is_some()
        | ph2d_ray_sensor::gotas::desarma(world).is_some()
        | ph2d_ray_sensor::gotas_from::desarma(world).is_some()
}

/// ⭐ **O clique na Hierarchy com um conta-gotas armado** (dono 15/11): enquanto `clique` corre, a linha clicada vai
/// para o conta-gotas e NÃO selecciona — o dono dele fica no Inspector. Devolve o objecto da linha (`Entity::to_bits`).
/// Arma só durante o clique: a aba do som (`ph2d-app-sfxlab`) re-arma a dela a cada quadro.
pub fn pela_hierarquia(armado: bool, clique: impl FnOnce()) -> Option<u64> {
    if !armado {
        clique();
        return None;
    }
    ph2d_panel_hierarchy::arma_escolha(true);
    clique();
    let linha = ph2d_panel_hierarchy::toma_escolha();
    ph2d_panel_hierarchy::arma_escolha(false);
    linha
}

/// O press do rato visto pelos conta-gotas do canvas (os campos são as guardas do despacho da shell).
#[derive(Clone, Copy, Debug)]
pub struct PressNoCanvas {
    /// Algum conta-gotas de OBJECTO está armado (`App::conta_gotas_armado`).
    pub armado: bool,
    /// Botão primário a descer (`Down`).
    pub primario_desce: bool,
    /// Um menu (ou modal) estava aberto antes do clique: é dele.
    pub menu_aberto: bool,
    /// O cursor está sobre o canvas (aceita o gizmo por cima), nunca sobre um painel.
    pub sobre_canvas: bool,
    /// O cursor em píxeis da janela.
    pub cursor: (f32, f32),
}

/// ⭐⭐ **O press no canvas com um conta-gotas armado é DELE** (dono 15/11): `Some((ponto, debaixo))` = consumido —
/// o despacho entrega-o ao conta-gotas e pára, logo o pick de selecção (e o Up que o fecharia) nunca o vê. `debaixo` é
/// o que está sob o cursor, de cima para baixo (`pick_sprites_at_world` no mundo de apresentação). `None` = segue.
pub fn premir_no_canvas(
    p: PressNoCanvas,
    window: ph2d_host::WindowSize,
    camera: &ph2d_render::Camera2d,
    present: &mut World,
) -> Option<([f32; 2], Vec<u64>)> {
    if !(p.armado && p.primario_desce && !p.menu_aberto && p.sobre_canvas) {
        return None;
    }
    let ponto = camera.screen_to_world(p.cursor, window);
    Some((ponto, ph2d_render::pick_sprites_at_world(present, ponto)))
}

/// O ponto de mundo do objecto `bits` — o «onde» de um clique que veio da Hierarchy (a cola prende ali).
#[must_use]
pub fn ponto_de(world: &World, bits: u64) -> [f32; 2] {
    ph2d_ecs::Entity::try_from_bits(bits)
        .and_then(|e| ph2d_ecs::world_transform(world, e))
        .map_or([0.0; 2], |t| [t.translation.x, t.translation.y])
}

/// O clique no palco com `debaixo` (de cima para baixo); `true` = um conta-gotas consumiu-o.
pub fn escolher(world: &mut World, bus: &mut ActionBus, debaixo: &[u64]) -> bool {
    crate::projectile_inspector::escolher_alvo(world, bus, debaixo)
        || crate::ray_topo::escolher_topo(world, bus, debaixo)
        || ph2d_ray_sensor::gotas::escolher(world, bus, debaixo)
        || ph2d_ray_sensor::gotas_from::escolher(world, bus, debaixo)
}
