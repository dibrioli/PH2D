//! **As secções NAV REGION e NAV AGENT: o instantâneo e o dreno** (plano 30, W4).
//!
//! ⚠️ **As duas metades vivem juntas de propósito** — o molde do [`super::ray_inspector`]: quem lê
//! o mundo para o painel e quem escreve a edição de volta fazem a MESMA tradução.
//!
//! # ⭐⭐ O que o instantâneo pergunta além dos campos
//!
//! As quatro perguntas que a PONTE faz para decidir se conduz o agente (há corpo · há mover · o
//! mover ouve o teclado · há um `PlatformPlayer`) — escritas aqui como colunas, para a queixa
//! (`InspectorNavAgent::queixa`) as poder ordenar. ⚠️ São as MESMAS perguntas, e não parecidas: se a
//! ponte passar a perguntar outra coisa, a queixa tem de a acompanhar, senão o painel diz «está
//! tudo bem» sobre um agente que a ponte salta.
//!
//! E a leitura VIVA vem do [`NavNow`] que a ponte publica no mundo — o Inspector não alcança a ponte.
//!
//! ⚠️ **O alvo é um NOME aqui**, nunca bits, pela lei do alvo do projéctil: os bits não sobrevivem a
//! um `Ctrl+Z`. ⭐ A tradução nome↔id é a do [`super::projectile_inspector::nome_do_alvo`] — uma
//! porta, dois leitores.

use ph2d_ecs::{Entity, SimWorld, World, stable_name_id, world_transform};
use ph2d_editor_core::nav_edits::{
    InspectorNavAgent, InspectorNavInfo, InspectorNavRegion, NavAgora, NavAlvoModo, NavEstado,
    NavFieldEdit,
};
use ph2d_physics_ecs::{
    NavAgent, NavNow, NavRegion, NavStatus, NavTarget, PlatformPlayer, RigidBody, TopDownPlayer,
};

/// O estado da ponte, no vocabulário do painel.
fn estado(s: NavStatus) -> NavEstado {
    match s {
        NavStatus::Idle => NavEstado::Parado,
        NavStatus::Moving => NavEstado::AAndar,
        NavStatus::MovingPartial => NavEstado::Parcial,
        NavStatus::Arrived => NavEstado::Chegou,
        NavStatus::NoPath => NavEstado::SemCaminho,
    }
}

/// **Alguma `NavRegion` contém este ponto?** — a mesma pergunta que a ponte faz (o rectângulo de
/// mundo centrado no `Transform` da região).
fn dentro_de_alguma_regiao(world: &World, p: [f32; 2]) -> bool {
    let Some(mut q) = world.try_query::<(Entity, &NavRegion)>() else {
        return false;
    };
    q.iter(world).any(|(e, r)| {
        let Some(t) = world_transform(world, e) else {
            return false;
        };
        let [a, b] = r.rect([t.translation.x, t.translation.y]);
        p[0] >= a[0] && p[0] <= b[0] && p[1] >= a[1] && p[1] <= b[1]
    })
}

fn info_do_agente(world: &World, e: Entity, a: &NavAgent) -> InspectorNavAgent {
    let (alvo_modo, alvo_nome, alvo_perdido, alvo_ponto) = match a.target {
        NavTarget::None => (NavAlvoModo::Nenhum, String::new(), false, [0.0, 0.0]),
        NavTarget::Named(id) => {
            let (nome, perdido) = crate::projectile_inspector::nome_do_alvo(world, id);
            (NavAlvoModo::Objecto, nome, perdido, [0.0, 0.0])
        }
        NavTarget::Point(p) => (NavAlvoModo::Ponto, String::new(), false, p),
    };
    let mover = world.get::<TopDownPlayer>(e);
    let pos = world_transform(world, e).map_or([0.0, 0.0], |t| [t.translation.x, t.translation.y]);
    InspectorNavAgent {
        alvo_modo,
        alvo_nome,
        alvo_perdido,
        alvo_ponto,
        radius: a.radius,
        arrive: a.arrive_distance,
        repath: a.repath_distance,
        stuck_after: a.stuck_after_s,
        active: a.active,
        avoidance: a.avoidance,
        on_arrived: a.on_arrived.clone(),
        on_no_path: a.on_no_path.clone(),
        on_stuck: a.on_stuck.clone(),
        has_body: world.get::<RigidBody>(e).is_some(),
        has_mover: mover.is_some(),
        mover_reads_keys: mover.is_some_and(|m| m.default_controls),
        has_platformer: world.get::<PlatformPlayer>(e).is_some(),
        in_region: dentro_de_alguma_regiao(world, pos),
        agora: world.get::<NavNow>(e).map(|n| NavAgora {
            estado: estado(n.status),
            restante: n.remaining,
            raio: n.radius,
        }),
    }
}

/// **O instantâneo.** `None` para quem não tem nenhum dos dois componentes (ADR-0166).
pub fn build_info(
    world: &World,
    bits: u64,
    selected_count: usize,
    clock_playing: bool,
) -> Option<InspectorNavInfo> {
    let e = Entity::from_bits(bits);
    let region = world.get::<NavRegion>(e).map(|r| InspectorNavRegion {
        half_w: r.half_extents[0],
        half_h: r.half_extents[1],
        obstacle_layers: r.obstacle_layers,
    });
    let agent = world
        .get::<NavAgent>(e)
        .map(|a| info_do_agente(world, e, a));
    if region.is_none() && agent.is_none() {
        return None;
    }
    Some(InspectorNavInfo {
        entity_bits: bits,
        region,
        agent,
        clock_playing,
        selected_count,
    })
}

/// **O dreno.** `true` = tocou no mundo.
pub fn apply_nav_edit(world: &mut World, bits: u64, edit: &NavFieldEdit) -> bool {
    let e = Entity::from_bits(bits);
    if world.get_entity(e).is_err() {
        return false;
    }
    match edit {
        NavFieldEdit::HalfW(_) | NavFieldEdit::HalfH(_) | NavFieldEdit::ObstacleLayers(_) => {
            let Some(mut r) = world.get_mut::<NavRegion>(e) else {
                return false;
            };
            match edit {
                // ⚠️ **Meias-extensões negativas não existem** — o `rect` toma o valor absoluto, e um
                // número negativo no painel seria um campo que mente sobre o que corre.
                NavFieldEdit::HalfW(v) => r.half_extents[0] = v.max(0.0),
                NavFieldEdit::HalfH(v) => r.half_extents[1] = v.max(0.0),
                NavFieldEdit::ObstacleLayers(m) => r.obstacle_layers = *m,
                _ => return false,
            }
            return true;
        }
        _ => {}
    }
    // ⚠️ **O nome resolve-se ANTES do empréstimo mutável** (ele varre o mundo).
    let novo_alvo = match edit {
        NavFieldEdit::AlvoNome(nome) => {
            let t = nome.trim();
            Some(NavTarget::Named(if t.is_empty() {
                0
            } else {
                stable_name_id(t)
            }))
        }
        _ => None,
    };
    let Some(mut a) = world.get_mut::<NavAgent>(e) else {
        return false;
    };
    match edit {
        NavFieldEdit::AlvoModo(m) => {
            // ⚠️ **Trocar de modo PRESERVA o que o modo de destino já tinha**, quando tinha: voltar
            // a `Objecto` depois de espreitar `Ponto` não deve apagar o nome — mas o componente só
            // guarda UM alvo, logo o que se preserva é o do modo em que ele já está.
            a.target = match (m, a.target) {
                (NavAlvoModo::Nenhum, _) => NavTarget::None,
                (NavAlvoModo::Objecto, t @ NavTarget::Named(_)) => t,
                (NavAlvoModo::Objecto, _) => NavTarget::Named(0),
                (NavAlvoModo::Ponto, t @ NavTarget::Point(_)) => t,
                (NavAlvoModo::Ponto, _) => NavTarget::Point([0.0, 0.0]),
            };
        }
        NavFieldEdit::AlvoNome(_) => {
            if let Some(t) = novo_alvo {
                a.target = t;
            }
        }
        NavFieldEdit::AlvoX(v) | NavFieldEdit::AlvoY(v) => {
            let mut p = match a.target {
                NavTarget::Point(p) => p,
                _ => [0.0, 0.0],
            };
            if matches!(edit, NavFieldEdit::AlvoX(_)) {
                p[0] = *v;
            } else {
                p[1] = *v;
            }
            a.target = NavTarget::Point(p);
        }
        NavFieldEdit::Radius(v) => a.radius = v.max(0.0),
        NavFieldEdit::Arrive(v) => a.arrive_distance = v.max(0.0),
        NavFieldEdit::Repath(v) => a.repath_distance = v.max(0.0),
        NavFieldEdit::StuckAfter(v) => a.stuck_after_s = v.max(0.0),
        NavFieldEdit::Active(b) => a.active = *b,
        NavFieldEdit::Avoidance(b) => a.avoidance = *b,
        NavFieldEdit::OnArrived(s) => a.on_arrived.clone_from(s),
        NavFieldEdit::OnNoPath(s) => a.on_no_path.clone_from(s),
        NavFieldEdit::OnStuck(s) => a.on_stuck.clone_from(s),
        NavFieldEdit::HalfW(_) | NavFieldEdit::HalfH(_) | NavFieldEdit::ObstacleLayers(_) => {
            return false;
        }
    }
    true
}

/// **Aplica as edições que o painel emitiu neste quadro.** `true` = o documento mudou.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, NavFieldEdit)]) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        if apply_nav_edit(sim.world_mut(), *bits, edit) {
            mexeu = true;
        }
    }
    mexeu
}

#[cfg(test)]
#[path = "nav_inspector_tests.rs"]
mod tests;
