//! **A secção RAY SENSOR: o instantâneo e o dreno** (suplente #21).
//!
//! ⛔⛔ **Ele mora na CRATE DA FAMÍLIA e não na shell**, ao contrário dos irmãos mais velhos
//! (`inspector_topdown`, `inspector_projectile`): a catraca `the_shell_only_shrinks` reprovou na
//! primeira tentativa (`197 219` contra o tecto de `196 990`), e a cura que ela prescreve por
//! escrito é **MOVER para `crates/ph2d-app-<família>`, nunca subir o número**. ⭐ O molde já
//! existia ao lado — o gatilho, a vigia do contador, a cutscene, o emissor e o HUD já vivem aqui.
//!
//! ⚠️ **As duas metades vivem juntas de propósito** — o mesmo molde do
//! [`super::inspector_projectile`]: quem lê o mundo para o painel e quem escreve a edição de volta
//! fazem a MESMA tradução, e separá-los seria a porta pela qual as duas divergem.
//!
//! # ⭐⭐⭐ O que esta secção faz que as irmãs não fazem: ela lê a CORRIDA
//!
//! Os campos saem dos dois componentes, como sempre. **O que o raio VÊ** sai do mapa da ponte
//! (`PhysicsBridge::ray_sensor_hits`) — não há campo nenhum onde ele viva, e a lei do #11/#20 diz
//! porquê: *o que nasce numa corrida não é documento*.
//!
//! ⚠️ **E o alvo vira um NOME aqui**, nunca bits: o painel mostra texto, e os bits de uma entidade
//! não sobrevivem a um `Ctrl+Z` (o undo respawna o mundo inteiro com bits novos). *Guardar bits num
//! snapshot que dura um quadro seria inofensivo; traduzi-los aqui é o que mantém a regra numa porta
//! só.*

use ph2d_ecs::{Entity, Name, SimWorld, World};
use ph2d_editor_core::ray_edits::{InspectorRayInfo, RayFieldEdit, RaySee, RayShapeKind};
use ph2d_physics_ecs::{RaySensor, RayShape, RaySignals, SeeFilter};

/// **O instantâneo.** `None` para quem não tem o componente (ADR-0166).
///
/// `vistos` = o que ele vê AGORA, do mais perto para o mais longe (um só sem `all_hits`; vazio =
/// nada); `tree` = a árvore de tags do documento (o `only_see` da `camada_vazia`).
pub fn build_ray_info(
    world: &World,
    tree: &ph2d_tags::TagTree,
    bits: u64,
    selected_count: usize,
    clock_playing: bool,
    vistos: &[(Entity, f32, Entity)],
) -> Option<InspectorRayInfo> {
    let e = Entity::from_bits(bits);
    let r = world.get::<RaySensor>(e)?.clone();
    // ⚠️ **Os nomes são OPCIONAIS**, e a ausência lê-se como duas strings vazias — que é
    // exactamente *«ele vê e cala-se»*, a terceira queixa. ⛔ Não há aqui um `?`: um raio sem
    // `RaySignals` é um raio legítimo, e devolver `None` esconderia a secção inteira dele.
    let s = world.get::<RaySignals>(e);
    // ⭐ O NOME de quem ele vê — e `None` quando o alvo já não está no mundo, que é honesto: o
    // canal publica a entidade do tique, e um corpo apagado entre o tique e o quadro não tem nome.
    let nome = |alvo: Entity| {
        world
            .get::<Name>(alvo)
            .map(|n| n.as_str().to_string())
            .unwrap_or_default()
    };
    let (sees, sees_at) = vistos
        .first()
        .map_or((String::new(), 0.0), |&(alvo, d, _)| (nome(alvo), d));
    // RS-4: as tags da FORMA acertada são o «material» dela (o Surface Type do Unreal).
    let sees_tags = vistos
        .first()
        .map(|&(_, _, forma)| crate::gotas::tags_da_forma(world, tree, forma))
        .unwrap_or_default();
    let sees_all = if r.all_hits || r.cone_activo() > 0.0 {
        vistos.iter().map(|&(alvo, d, _)| (nome(alvo), d)).collect()
    } else {
        Vec::new()
    };
    let (ignore_up_to, ignore_up_to_off, ignore_up_to_unknown, ignore_up_to_auto) =
        crate::topo::leitura(world, e, &r);
    let aim = crate::gotas::leitura_da_mira(world, bits, r.aim_at);
    Some(InspectorRayInfo {
        entity_bits: bits,
        angle: r.angle,
        reach: r.reach,
        layer: u32::from(r.layer),
        turns_with_facing: r.turns_with_facing,
        ignore_body: r.ignore_body,
        ignore_parts: r.ignore_parts,
        ignore_up_to,
        ignore_up_to_off,
        ignore_up_to_unknown,
        ignore_up_to_auto,
        up_to_pick_armed: crate::topo::armado(world, Some(bits)),
        only_see: match r.only_see {
            SeeFilter::Everything => RaySee::Everything,
            SeeFilter::OnlyTag => RaySee::OnlyTag,
            SeeFilter::ExceptTag => RaySee::ExceptTag,
        },
        see_tag: r.see_tag,
        see_tag_exists: r.see_tag == 0 || tree.get(ph2d_tags::TagId(r.see_tag)).is_some(),
        walls_block: r.walls_block,
        hit_triggers: r.hit_triggers,
        starts_inside: r.starts_inside,
        all_hits: r.all_hits,
        enabled: r.enabled,
        thickness: r.thickness,
        cone: r.cone,
        shape: match r.shape {
            RayShape::Line => RayShapeKind::Line,
            RayShape::Thick => RayShapeKind::Thick,
            RayShape::Cone => RayShapeKind::Cone,
        },
        aim_at: aim.0,
        aim_at_gone: aim.1,
        aim_at_unknown: aim.2,
        aim_pick_armed: crate::gotas::armado(world, Some((bits, crate::gotas::Gotas::Mira))),
        also_ignore: crate::gotas::nomes(world, &r.also_ignore),
        ignore_pick_armed: crate::gotas::armado(world, Some((bits, crate::gotas::Gotas::Lista))),
        sees_tags,
        layer_empty: camada_vazia(world, tree, e, &r),
        on_enter: s.map(|s| s.on_enter.clone()).unwrap_or_default(),
        on_exit: s.map(|s| s.on_exit.clone()).unwrap_or_default(),
        sees,
        sees_at,
        sees_all,
        clock_playing,
        selected_count,
    })
}

/// **O dreno.** `true` = tocou no mundo.
pub fn apply_ray_edit(world: &mut World, bits: u64, edit: &RayFieldEdit) -> bool {
    let e = Entity::from_bits(bits);
    if world.get_entity(e).is_err() {
        return false;
    }
    // ⚠️ **Os dois NOMES vivem noutro componente**, e ele pode não estar lá — um raio que ganha voz
    // pela primeira vez recebe o componente aqui. ⛔ Fazê-lo no painel seria o painel a escrever no
    // mundo, que é o que esta porta existe para impedir.
    match edit {
        RayFieldEdit::OnEnter(nome) => {
            let mut s = world.get_mut::<RaySignals>(e).map(|s| s.clone());
            if let Some(v) = s.as_mut() {
                v.on_enter.clone_from(nome);
            } else {
                s = Some(RaySignals {
                    on_enter: nome.clone(),
                    on_exit: String::new(),
                });
            }
            world.entity_mut(e).insert(s.unwrap_or_default());
            return true;
        }
        RayFieldEdit::OnExit(nome) => {
            let mut s = world.get_mut::<RaySignals>(e).map(|s| s.clone());
            if let Some(v) = s.as_mut() {
                v.on_exit.clone_from(nome);
            } else {
                s = Some(RaySignals {
                    on_enter: String::new(),
                    on_exit: nome.clone(),
                });
            }
            world.entity_mut(e).insert(s.unwrap_or_default());
            return true;
        }
        RayFieldEdit::PickUpTo(arma) => {
            crate::topo::arma(world, arma.then_some(bits));
            return false;
        }
        RayFieldEdit::PickAimAt(arma) => {
            crate::gotas::arma(world, arma.then_some((bits, crate::gotas::Gotas::Mira)));
            return false;
        }
        RayFieldEdit::PickIgnore(arma) => {
            crate::gotas::arma(world, arma.then_some((bits, crate::gotas::Gotas::Lista)));
            return false;
        }
        _ => {}
    }
    // O topo resolve-se ANTES do empréstimo mutável: ele varre a cadeia e a cena.
    let topo = match edit {
        RayFieldEdit::IgnoreUpTo(nome) => {
            let id = crate::topo::resolve(world, e, nome);
            crate::topo::escrito(world, bits, id.is_none().then_some(nome.as_str()));
            match id {
                Some(id) => Some(id),
                None => return false,
            }
        }
        RayFieldEdit::AimAt(nome) => {
            let id = crate::gotas::resolve(world, nome);
            crate::gotas::escrita(world, bits, id.is_none().then_some(nome.as_str()));
            match id {
                Some(id) => Some(id),
                None => return false,
            }
        }
        RayFieldEdit::IgnoreAdd(nome) => match crate::gotas::resolve(world, nome) {
            Some(id) if id != 0 => Some(id),
            _ => return false,
        },
        _ => None,
    };
    let Some(mut r) = world.get_mut::<RaySensor>(e) else {
        return false;
    };
    match edit {
        RayFieldEdit::Angle(v) => r.angle = *v,
        RayFieldEdit::TurnsWithFacing(v) => r.turns_with_facing = *v,
        RayFieldEdit::Reach(v) => r.reach = *v,
        // ⚠️ **A camada é um ÍNDICE**, e o tecto é o do mundo (`MAX_LAYERS`): um número maior não
        // tem linha na matriz, e a porta do motor leria uma coluna que não existe.
        RayFieldEdit::Layer(v) => r.layer = u8::try_from(*v).unwrap_or(0),
        RayFieldEdit::IgnoreBody(v) => r.ignore_body = *v,
        RayFieldEdit::IgnoreParts(v) => r.ignore_parts = *v,
        RayFieldEdit::OnlySee(v) => {
            r.only_see = match v {
                RaySee::Everything => SeeFilter::Everything,
                RaySee::OnlyTag => SeeFilter::OnlyTag,
                RaySee::ExceptTag => SeeFilter::ExceptTag,
            }
        }
        RayFieldEdit::SeeTag(v) => r.see_tag = *v,
        RayFieldEdit::IgnoreUpTo(_) => r.ignore_up_to = topo.unwrap_or(0),
        RayFieldEdit::WallsBlock(v) => r.walls_block = *v,
        RayFieldEdit::HitTriggers(v) => r.hit_triggers = *v,
        RayFieldEdit::StartsInside(v) => r.starts_inside = *v,
        RayFieldEdit::AllHits(v) => r.all_hits = *v,
        RayFieldEdit::Enabled(v) => r.enabled = *v,
        RayFieldEdit::Thickness(v) => r.thickness = *v,
        RayFieldEdit::Cone(v) => r.cone = *v,
        RayFieldEdit::Shape(v) => {
            r.shape = match v {
                RayShapeKind::Line => RayShape::Line,
                RayShapeKind::Thick => RayShape::Thick,
                RayShapeKind::Cone => RayShape::Cone,
            }
        }
        RayFieldEdit::AimAt(_) => r.aim_at = topo.unwrap_or(0),
        RayFieldEdit::IgnoreAdd(_) => {
            let id = topo.unwrap_or(0);
            if r.also_ignore.contains(&id) {
                return false;
            }
            r.also_ignore.push(id);
        }
        RayFieldEdit::IgnoreRemove(n) => {
            if *n >= r.also_ignore.len() {
                return false;
            }
            r.also_ignore.remove(*n);
        }
        // Os dois nomes e os conta-gotas já saíram acima.
        RayFieldEdit::OnEnter(_)
        | RayFieldEdit::OnExit(_)
        | RayFieldEdit::PickUpTo(_)
        | RayFieldEdit::PickAimAt(_)
        | RayFieldEdit::PickIgnore(_) => {
            return false;
        }
    }
    sanitize_ray_sensor(&mut r);
    true
}

/// ⭐ **A lei do RAIO** (doc CodeEditor 08 §3.5): a camada é um índice da matriz do mundo, e acima
/// do tecto (`MAX_LAYERS`) não há coluna. O resto aceita tudo (o alcance zero é um aviso do cartão).
/// ⭐ **Nenhuma forma numa camada que o raio vê** (GEF 07, B·3): fora as que ele exclui — o corpo dono
/// (`ignore_body`) e a família (`ignore_parts`). Um sensor só conta com `hit_triggers`; com `only_see`
/// só contam as formas cujo corpo passa no filtro. Uma forma com exceção própria (`CollisionOverride`)
/// conta como visível — o aviso só fala quando é CERTO que o raio nunca verá nada.
fn camada_vazia(world: &World, tree: &ph2d_tags::TagTree, raio: Entity, r: &RaySensor) -> bool {
    use ph2d_physics_ecs::{Collider, CollisionOverride, RigidBody};
    let linha = ph2d_physics_ecs::excecao::LiveLayerRows::row(world, r.layer);
    let dono = |e: Entity| {
        if world.get::<RigidBody>(e).is_some() {
            Some(e)
        } else {
            ph2d_physics_ecs::owner_body(world, e)
        }
    };
    let meu = dono(raio);
    let topo = if r.ignore_parts {
        ph2d_physics_ecs::topo_da_familia(world, raio, r.ignore_up_to)
    } else {
        None
    };
    // ⚠️ Só `&Collider` na consulta: um `Option<&CollisionOverride>` nela devolve `None` enquanto
    // nenhum objeto da cena tiver exceção (o tipo nem está registado) — e o aviso mentia sempre.
    let Some(mut q) = world.try_query::<(Entity, &Collider)>() else {
        return true;
    };
    !q.iter(world).any(|(e, c)| {
        let excecao = world.get::<CollisionOverride>(e).is_some();
        let ve = excecao || linha & (1u32 << u32::from(c.layer).min(31)) != 0;
        let corpo = dono(e);
        let excluido = (r.ignore_body && meu.is_some() && corpo == meu)
            || (corpo != meu
                && topo
                    .zip(corpo)
                    .is_some_and(|(t, b)| ph2d_physics_ecs::na_familia(world, t, b)));
        let passa = r.only_see.passes(
            r.see_tag,
            tree,
            corpo.and_then(|b| world.get::<ph2d_ecs::tags::Tags>(b)),
        );
        ve && (r.hit_triggers || !c.is_sensor) && !excluido && passa
    })
}

/// ⭐ **A lei do RAIO**: a camada é um índice da matriz (acima de `MAX_LAYERS` não há coluna); a
/// espessura não é negativa e o leque vai de `0` a uma volta (`360`). `NaN` = a forma de fábrica.
pub fn sanitize_ray_sensor(r: &mut ph2d_physics_ecs::RaySensor) {
    r.layer = r.layer.min(ph2d_physics_ecs::MAX_LAYERS as u8 - 1);
    r.thickness = if r.thickness.is_finite() {
        r.thickness.max(0.0)
    } else {
        0.0
    };
    r.cone = if r.cone.is_finite() {
        r.cone.clamp(0.0, 360.0)
    } else {
        0.0
    };
    r.also_ignore.retain(|&id| id != 0);
}

/// **Aplica as edições que o painel emitiu neste quadro.** `true` = o documento mudou.
///
/// ⚠️ Irmã das `apply_all` das outras secções desta crate — a shell chama UMA função por secção, e
/// o laço vive com quem sabe o que cada edição significa.
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, RayFieldEdit)]) -> bool {
    let mut mexeu = false;
    for (bits, edit) in edits {
        if apply_ray_edit(sim.world_mut(), *bits, edit) {
            mexeu = true;
        }
    }
    mexeu
}

#[cfg(test)]
#[path = "inspector_tests.rs"]
mod tests;
