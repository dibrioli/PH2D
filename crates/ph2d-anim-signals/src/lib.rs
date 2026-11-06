//! **Os sinais do cartão «Animation»**: os marcadores em quadro (SFX Lab W1) que o cartão lê e edita, e os
//! nomes de sinal que o projecto já autorou (o menu «Signal» das regras). Tudo passa pela fila de
//! comandos (`queue_set`/`queue_remove`), então o undo regista por diff como o resto do Inspector.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SpriteAnimMarkers, SpriteAnimations, World};
use ph2d_editor_core::{AnimFieldEdit, InspectorAnimMarker};
use ph2d_inspector_ordering::{queue_remove, queue_set};
use ph2d_physics_ecs::{AnimRules, AnimWhen, Health, RaySignals, SignalOnHit, SignalOnLeave};
use std::collections::BTreeSet;

/// O nome do componente dos marcadores no registo.
pub const MARKERS: &str = "ph2d::ecs::SpriteAnimMarkers";

/// Os nomes de sinal que o projecto já autorou (animações, marcadores, vida, contactos, raios) e os das
/// regras, ordenados — as opções «Signal» do menu.
#[must_use]
pub fn authored_signals(world: &World, rules: Option<&AnimRules>) -> Vec<String> {
    let mut names = BTreeSet::new();
    let mut put = |s: &str| {
        if !s.is_empty() {
            names.insert(s.to_owned());
        }
    };
    if let Some(mut q) = world.try_query::<&SpriteAnimations>() {
        for t in q.iter(world).flat_map(|lib| lib.iter()) {
            put(&t.signal_on_finish);
            put(&t.signal_on_loop);
        }
    }
    if let Some(mut q) = world.try_query::<&SpriteAnimMarkers>() {
        for m in q.iter(world).flat_map(|m| m.0.iter()) {
            put(&m.signal);
        }
    }
    if let Some(mut q) = world.try_query::<&Health>() {
        for h in q.iter(world) {
            put(&h.on_damage);
            put(&h.on_heal);
            put(&h.on_death);
        }
    }
    if let Some(mut q) = world.try_query::<&SignalOnHit>() {
        q.iter(world).for_each(|s| put(&s.0));
    }
    if let Some(mut q) = world.try_query::<&SignalOnLeave>() {
        q.iter(world).for_each(|s| put(&s.0));
    }
    if let Some(mut q) = world.try_query::<&RaySignals>() {
        for r in q.iter(world) {
            put(&r.on_enter);
            put(&r.on_exit);
        }
    }
    for r in rules.iter().flat_map(|r| r.0.iter()) {
        if let AnimWhen::Signal(n) = &r.when {
            put(n);
        }
    }
    names.into_iter().collect()
}

/// Os marcadores de uma linha da biblioteca, na ordem da lista.
#[must_use]
pub fn row_markers(world: &World, entity: Entity, name: &str) -> Vec<InspectorAnimMarker> {
    world
        .get::<SpriteAnimMarkers>(entity)
        .map(|m| {
            m.of_tag(name)
                .map(|m| InspectorAnimMarker {
                    step: m.step,
                    signal: m.signal.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Grava os marcadores `m`; vazios, o componente sai.
fn put_markers(
    m: &SpriteAnimMarkers,
    entity_bits: u64,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    if m.0.is_empty() {
        queue_remove(queue, registry, entity_bits, MARKERS);
    } else {
        queue_set(queue, registry, entity_bits, MARKERS, m);
    }
}

/// Uma edição dos marcadores da animação `row`. `None` = não é edição de marcador, ou linha ou marcador
/// fora (nada se grava).
pub fn apply_marker_edit(
    world: &World,
    entity_bits: u64,
    edit: &AnimFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> Option<()> {
    let entity = Entity::from_bits(entity_bits);
    let row = match edit {
        AnimFieldEdit::MarkerAdd(r)
        | AnimFieldEdit::MarkerStep(r, ..)
        | AnimFieldEdit::MarkerSignal(r, ..)
        | AnimFieldEdit::MarkerRemove(r, _) => usize::from(*r),
        _ => return None,
    };
    let name = world
        .get::<SpriteAnimations>(entity)?
        .iter()
        .nth(row)?
        .name
        .clone();
    let steps = ph2d_ecs::anim_steps(world, entity, &name);
    let mut m = world
        .get::<SpriteAnimMarkers>(entity)
        .cloned()
        .unwrap_or_default();
    match edit {
        AnimFieldEdit::MarkerAdd(_) => m.add(&name, steps),
        AnimFieldEdit::MarkerStep(_, i, step) => m.set_step(&name, (*i).into(), *step, steps)?,
        AnimFieldEdit::MarkerSignal(_, i, text) => {
            m.set_signal(&name, (*i).into(), clean_signal_name(text))?;
        }
        AnimFieldEdit::MarkerRemove(_, i) => m.remove_at(&name, (*i).into())?,
        _ => return None,
    }
    put_markers(&m, entity_bits, queue, registry);
    Some(())
}

/// Uma animação saiu da biblioteca: os marcadores saem com ela (inertes, voltariam sozinhos numa
/// animação nova de mesmo nome).
pub fn forget_tag(
    world: &World,
    entity_bits: u64,
    name: &str,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    if let Some(mut m) = world.get::<SpriteAnimMarkers>(entity).cloned()
        && m.remove_tag(name)
    {
        put_markers(&m, entity_bits, queue, registry);
    }
}

/// Uma animação mudou de nome: os marcadores vão com ela.
pub fn follow_rename(
    world: &World,
    entity_bits: u64,
    old: &str,
    new: &str,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    if let Some(mut m) = world.get::<SpriteAnimMarkers>(entity).cloned()
        && m.of_tag(old).next().is_some()
    {
        m.rename_tag(old, new);
        queue_set(queue, registry, entity_bits, MARKERS, &m);
    }
}

/// **O nome de um sinal, como ele fica guardado.** Aparado e capado no mesmo teto do nome de uma
/// tag (`ph2d_ecs::ANIM_NAME_MAX_BYTES`).
///
/// ⚠️ **Aparar não é higiene:** o contrato de um sinal é a **igualdade de string** (ADR-0143), e
/// um espaço à direita é invisível no campo e faz o consumidor não casar — o pior tipo de defeito,
/// porque tudo parece certo dos dois lados.
///
/// ⚠️ **O corte é por BYTES e respeita o caractere**: `String::truncate` entra em pânico a meio de
/// um multibyte, e um nome com acento é normal.
#[must_use]
pub fn clean_signal_name(raw: &str) -> String {
    let t = raw.trim();
    match t.char_indices().nth_back(0) {
        _ if t.len() <= ph2d_ecs::ANIM_NAME_MAX_BYTES => t.to_owned(),
        _ => {
            let mut end = ph2d_ecs::ANIM_NAME_MAX_BYTES;
            while end > 0 && !t.is_char_boundary(end) {
                end -= 1;
            }
            t[..end].to_owned()
        }
    }
}
