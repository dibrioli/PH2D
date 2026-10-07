//! **As secções NAV REGION e NAV AGENT: o instantâneo e o dreno** (plano 30, W4) — e (W7) NAV COST
//! AREA e NAV LINK, no mesmo vocabulário.
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
    InspectorNavAgent, InspectorNavCostArea, InspectorNavInfo, InspectorNavLink,
    InspectorNavRegion, NAV_AREA_COST_MIN, NavAgora, NavAlvoModo, NavEstado, NavFieldEdit,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, Health, NavAgent, NavCostArea, NavCostAreaNow, NavLink, NavNow, NavRegion,
    NavStatus, NavTarget, PlatformPlayer, RigidBody, TopDownPlayer,
};

/// O estado da ponte, no vocabulário do painel.
/// ⭐ **Abaixo desta fracção da rapidez, travado por OUTRO corpo, ele DÁ PASSAGEM** — o `NavNow::avanco`.
///
/// Medido na cena `=2` (8 agentes cruzam a porta, 900 tiques), transições da leitura por limiar:
/// `0,99` → 40 · `0,9` → 39 · `0,75` → 40 · **`0,5` → 34** · `0,25` → 18 · `0,1` → 16. A metade lê-se
/// (≈ 4 trocas por agente numa travessia, nunca a piscar por tique) e só acusa quem perdeu mais do que
/// ganhou. ⚠️ O «por OUTRO corpo» é a metade que importa: só pela rapidez, o agente SOZINHO no
/// labirinto `=1` acusava 58 de 369 tiques (a quina também trava); com ela, zero.
pub const AVANCO_DE_QUEM_DA_PASSAGEM: f32 = 0.5;

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
        NavTarget::None | NavTarget::NearestTagged(_) => {
            let m = if matches!(a.target, NavTarget::None) {
                NavAlvoModo::Nenhum
            } else {
                NavAlvoModo::Tag
            };
            (m, String::new(), false, [0.0, 0.0])
        }
        NavTarget::Named(id) => {
            let (nome, perdido) = crate::projectile_inspector::nome_do_alvo(world, id);
            (NavAlvoModo::Objecto, nome, perdido, [0.0, 0.0])
        }
        NavTarget::Point(p) => (NavAlvoModo::Ponto, String::new(), false, p),
        // ⚠️ **Perdida = ninguém com esse nome OU um objecto que não é uma forma desenhada** — a
        // mesma pergunta que a rota faz (`nav_rota::forma_chamada`).
        NavTarget::Patrol(id) => {
            let (nome, perdido) = crate::projectile_inspector::nome_do_alvo(world, id);
            let forma = !perdido && forma_com_nome(world, id);
            (NavAlvoModo::Patrulha, nome, id != 0 && !forma, [0.0, 0.0])
        }
    };
    let alvo_tag = match a.target {
        NavTarget::NearestTagged(t) => t,
        _ => 0,
    };
    let mover = world.get::<TopDownPlayer>(e);
    let pos = world_transform(world, e).map_or([0.0, 0.0], |t| [t.translation.x, t.translation.y]);
    InspectorNavAgent {
        alvo_modo,
        alvo_nome,
        alvo_perdido,
        alvo_tag,
        alvo_ponto,
        radius: a.radius,
        arrive: a.arrive_distance,
        repath: a.repath_distance,
        stuck_after: a.stuck_after_s,
        active: a.active,
        avoidance: a.avoidance,
        avoid_harm: a.avoid_harm,
        // ⚠️ A MESMA pergunta da ponte (`zonas_que_evita`): sem `Health` nada o fere.
        has_health: world.get::<Health>(e).is_some(),
        on_arrived: a.on_arrived.clone(),
        on_no_path: a.on_no_path.clone(),
        on_stuck: a.on_stuck.clone(),
        has_body: world.get::<RigidBody>(e).is_some(),
        has_mover: mover.is_some(),
        mover_reads_keys: mover.is_some_and(|m| m.default_controls),
        has_platformer: world.get::<PlatformPlayer>(e).is_some(),
        in_region: dentro_de_alguma_regiao(world, pos),
        ordem: world.get::<NavNow>(e).and_then(|n| n.ordem),
        alvo_da_ordem: world
            .get::<NavNow>(e)
            .filter(|n| n.alvo_da_ordem != 0)
            .map(|n| crate::projectile_inspector::nome_do_alvo(world, n.alvo_da_ordem).0)
            .unwrap_or_default(),
        agora: world.get::<NavNow>(e).map(|n| NavAgora {
            estado: estado(n.status),
            restante: n.remaining,
            raio: n.radius,
            dando_passagem: n.avanco < AVANCO_DE_QUEM_DA_PASSAGEM,
        }),
    }
}

/// Há uma forma DESENHADA com este nome?
fn forma_com_nome(world: &World, id: u64) -> bool {
    let Some(mut q) = world.try_query::<(Entity, &ph2d_ecs::Name)>() else {
        return false;
    };
    q.iter(world)
        .find(|(_, n)| stable_name_id(n.as_str()) == id)
        .is_some_and(|(e, _)| world.get::<ph2d_ecs::VecPathRef>(e).is_some())
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
    let cost_area = world.get::<NavCostArea>(e).map(|c| {
        let corpo = world.get::<RigidBody>(e);
        InspectorNavCostArea {
            cost: c.cost,
            forbidden: c.forbidden,
            // ⚠️ As MESMAS perguntas da ponte (`custos_deste_tique`): só um corpo com colisor tem
            // forma, e só um corpo PARADO a recorta (um `Dynamic` nunca é obstáculo).
            has_shape: corpo.is_some() && world.get::<Collider>(e).is_some(),
            body_moves: corpo.is_some_and(|b| b.kind == BodyKind::Dynamic),
            too_narrow_for: world.get::<NavCostAreaNow>(e).map(|n| n.too_narrow_for),
        }
    });
    let link = world.get::<NavLink>(e).map(|l| {
        let (to_nome, to_perdido) = crate::projectile_inspector::nome_do_alvo(world, l.to);
        InspectorNavLink {
            to_nome,
            to_perdido,
            two_way: l.two_way,
            teleport: l.teleport,
            cost: l.cost,
            on_crossed: l.on_crossed.clone(),
        }
    });
    if region.is_none() && agent.is_none() && cost_area.is_none() && link.is_none() {
        return None;
    }
    Some(InspectorNavInfo {
        entity_bits: bits,
        region,
        agent,
        cost_area,
        link,
        clock_playing,
        selected_count,
    })
}

/// ⭐ (W7) **O dreno da ÁREA e do ATALHO** — `None` = a edição não é destes dois.
fn apply_custo_e_atalho(world: &mut World, e: Entity, edit: &NavFieldEdit) -> Option<bool> {
    match edit {
        NavFieldEdit::CostAreaCost(_) | NavFieldEdit::CostAreaForbidden(_) => {
            let Some(mut c) = world.get_mut::<NavCostArea>(e) else {
                return Some(false);
            };
            match edit {
                // ⚠️ A lei só pede `> 0` (o custo multiplica um comprimento) — o piso é o do painel.
                NavFieldEdit::CostAreaCost(v) => c.cost = v.max(NAV_AREA_COST_MIN),
                NavFieldEdit::CostAreaForbidden(b) => c.forbidden = *b,
                _ => return Some(false),
            }
            Some(true)
        }
        NavFieldEdit::LinkTo(_)
        | NavFieldEdit::LinkTwoWay(_)
        | NavFieldEdit::LinkTeleport(_)
        | NavFieldEdit::LinkCost(_)
        | NavFieldEdit::LinkOnCrossed(_) => {
            let Some(mut l) = world.get_mut::<NavLink>(e) else {
                return Some(false);
            };
            match edit {
                // ⚠️ **O NOME cru vira o `stable_name_id`** — a regra do `AlvoNome` (vazio = `0`).
                NavFieldEdit::LinkTo(nome) => {
                    let t = nome.trim();
                    l.to = if t.is_empty() { 0 } else { stable_name_id(t) };
                }
                NavFieldEdit::LinkTwoWay(b) => l.two_way = *b,
                NavFieldEdit::LinkTeleport(b) => l.teleport = *b,
                NavFieldEdit::LinkCost(v) => l.cost = v.max(0.0),
                NavFieldEdit::LinkOnCrossed(s) => l.on_crossed.clone_from(s),
                _ => return Some(false),
            }
            Some(true)
        }
        _ => None,
    }
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
    if let Some(mexeu) = apply_custo_e_atalho(world, e, edit) {
        return mexeu;
    }
    // ⚠️ **O nome resolve-se ANTES do empréstimo mutável** (ele varre o mundo). ⭐ (W6) E ele
    // escreve no alvo que o agente JÁ tem: na patrulha é o nome da FORMA (o modo vem primeiro).
    let em_patrulha = matches!(
        world.get::<NavAgent>(e).map(|a| a.target),
        Some(NavTarget::Patrol(_))
    );
    let novo_alvo = match edit {
        NavFieldEdit::AlvoNome(nome) => {
            let t = nome.trim();
            let id = if t.is_empty() { 0 } else { stable_name_id(t) };
            Some(if em_patrulha {
                NavTarget::Patrol(id)
            } else {
                NavTarget::Named(id)
            })
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
                (NavAlvoModo::Tag, t @ NavTarget::NearestTagged(_)) => t,
                (NavAlvoModo::Tag, _) => NavTarget::NearestTagged(0),
                (NavAlvoModo::Patrulha, t @ NavTarget::Patrol(_)) => t,
                (NavAlvoModo::Patrulha, _) => NavTarget::Patrol(0),
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
        NavFieldEdit::AvoidHarm(b) => a.avoid_harm = *b,
        NavFieldEdit::AlvoTag(t) => a.target = NavTarget::NearestTagged(*t),
        NavFieldEdit::OnArrived(s) => a.on_arrived.clone_from(s),
        NavFieldEdit::OnNoPath(s) => a.on_no_path.clone_from(s),
        NavFieldEdit::OnStuck(s) => a.on_stuck.clone_from(s),
        NavFieldEdit::HalfW(_)
        | NavFieldEdit::HalfH(_)
        | NavFieldEdit::ObstacleLayers(_)
        | NavFieldEdit::CostAreaCost(_)
        | NavFieldEdit::CostAreaForbidden(_)
        | NavFieldEdit::LinkTo(_)
        | NavFieldEdit::LinkTwoWay(_)
        | NavFieldEdit::LinkTeleport(_)
        | NavFieldEdit::LinkCost(_)
        | NavFieldEdit::LinkOnCrossed(_) => {
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
