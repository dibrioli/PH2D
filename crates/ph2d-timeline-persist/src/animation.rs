//! A metade da ANIMAÇÃO EM VISTA que precisa do mundo (doc ObjectsAndComponents 01 §6): a cadeia
//! de ancestrais da selecção e os nomes do cabeçalho *«Animação de: X ▾»*. A lei vive em
//! `ph2d_timeline::animation_view`; aqui só se pergunta ao mundo.

use ph2d_core::Playhead;
use ph2d_ecs::{ChildOf, Entity, Name, World};
use ph2d_timeline::{SCENE, TimelineState, TimelineViewSnapshot};

/// Os ancestrais de `bits`, do pai para a raiz. Pára num ciclo (um pai que já passou) em vez de
/// num número escolhido.
fn ancestors(world: &World, bits: u64) -> Vec<u64> {
    let mut out: Vec<u64> = Vec::new();
    let mut cur = Entity::try_from_bits(bits);
    while let Some(parent) = cur
        .and_then(|e| world.get::<ChildOf>(e))
        .map(ChildOf::parent)
    {
        let p = parent.to_bits();
        if p == bits || out.contains(&p) {
            break;
        }
        out.push(p);
        cur = Some(parent);
    }
    out
}

/// **O passo de cada quadro, antes dos pedidos do painel:** a Timeline segue a selecção.
pub fn follow_selection(
    timeline: &mut TimelineState,
    playhead: &mut Playhead,
    world: &World,
    selected: Option<u64>,
) {
    let chain = selected.map(|s| ancestors(world, s)).unwrap_or_default();
    ph2d_timeline::animation_view::follow(timeline, playhead, selected, &chain);
}

fn name_of(world: &World, bits: u64) -> String {
    Entity::try_from_bits(bits)
        .and_then(|e| world.get::<Name>(e))
        .map(|n| n.as_str().to_string())
        .unwrap_or_default()
}

/// **Enche o cabeçalho** depois do `rebuild`: o dono em vista, as escolhas do ▾ (a Cena, os
/// ancestrais com Animação de fora para dentro, o seleccionado se tiver uma) e a oferta de
/// criar a Animação no seleccionado que não a tem.
pub fn publish_animation_header(
    view: &mut TimelineViewSnapshot,
    timeline: &TimelineState,
    world: &World,
) {
    let header = &mut view.animation;
    // Os nomes dos donos das fontes do Arrange da Cena (onda 4, item 5) — o `rebuild` deixou-os
    // vazios: o documento não conhece nomes.
    for g in header.sources.iter_mut().filter(|g| g.owner != SCENE) {
        g.name = name_of(world, g.owner);
    }
    header.owner = timeline.view.owner;
    header.choices.clear();
    header.offer = None;
    header.choices.push((SCENE, String::new()));
    let sel = timeline.view.selected;
    if sel == SCENE {
        return;
    }
    let mut chain = ancestors(world, sel);
    chain.reverse();
    chain.push(sel);
    for e in chain {
        if timeline.doc.has_animation(e) {
            header.choices.push((e, name_of(world, e)));
        }
    }
    if !timeline.doc.has_animation(sel) {
        header.offer = Some((sel, name_of(world, sel)));
    }
}

/// ⭐ **A cópia de um objecto leva a Animação PRÓPRIA** (escolha do dono, 08/10): cada original da
/// sub-árvore copiada que tem Animação dá a sua à cópia, com as tracks reapontadas para os objectos
/// da cópia (`pairs` = `(original, cópia)`). UM passo do undo da timeline. Devolve quantos clips.
pub fn copy_animations(timeline: &mut TimelineState, pairs: &[(u64, u64)]) -> usize {
    let owners: Vec<(u64, u64)> = pairs
        .iter()
        .copied()
        .filter(|(src, _)| timeline.doc.has_animation(*src))
        .collect();
    if owners.is_empty() {
        return 0;
    }
    timeline.history.push(timeline.doc.clone());
    let remap = |e: u64| pairs.iter().find(|(s, _)| *s == e).map(|(_, d)| *d);
    owners
        .iter()
        .map(|(src, dst)| timeline.doc.copy_animation(*src, *dst, remap))
        .sum()
}
