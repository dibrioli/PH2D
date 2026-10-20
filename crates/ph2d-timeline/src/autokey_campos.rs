//! ⭐⭐⭐ **O auto-key dos CAMPOS de família** (plano 35 §3.0 — *«o auto key grava absolutamente
//! tudo»*): a mesma lei do [`crate::autokey`] — o mundo contra a curva no instante, pela MESMA porta
//! `curve_seen` —, sobre os campos que a família publicou em `ph2d_ecs::CanaisLidos`, e com a regra
//! que os 19 de sempre não precisavam: **o GRUPO grava-se inteiro**. Mexer um ponto grava a forma
//! inteira (decisão do dono, 20/10); mexer o vermelho grava a cor inteira — e por isso todas as
//! trilhas de um grupo têm as chaves nos mesmos tempos.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::FieldKey;

use crate::autokey::{AutokeyPlan, curve_seen_b, key_value_b};
use crate::doc::TimelineDoc;
use crate::prop::PropKind;

/// A identidade do grupo de um campo: a família, o grupo, e o `i` quando o grupo se parte por ele.
/// Um campo que nenhuma família declara é um grupo de um só.
fn grupo(k: FieldKey) -> (u16, u16, u16, u16) {
    match k.info() {
        Some(i) => (k.familia, i.grupo, if i.por_i { k.i } else { 0 }, u16::MAX),
        None => (k.familia, k.campo, k.i, k.j),
    }
}

/// **Que chaves os campos de `entity` pedem neste quadro.** `lidos` = o que a família publicou (o
/// documento, através do eco); `base` = o que ela publicou no quadro anterior — sem curva, um campo
/// só nasce canal se MUDOU (`allow_create`), como a pose. `solo` = a vista Keys (a trilha crua).
#[must_use]
pub fn autokey_campos(
    doc: &TimelineDoc,
    entity: u64,
    t_secs: f64,
    lidos: &[(FieldKey, f32)],
    base: &BTreeMap<FieldKey, f32>,
    allow_create: bool,
    solo: bool,
) -> AutokeyPlan {
    // Os bindings da entidade, resolvidos UMA vez: uma forma são centenas de campos.
    let meus: BTreeMap<PropKind, &crate::TargetBinding> = doc
        .bindings()
        .iter()
        .filter(|b| b.entity == entity && matches!(b.prop, PropKind::Field(_)))
        .map(|b| (b.prop, b))
        .collect();
    let mut mexidos = BTreeSet::new();
    for &(k, v) in lidos {
        let p = PropKind::Field(k);
        let moved = match curve_seen_b(doc, entity, p, meus.get(&p).copied(), t_secs, solo) {
            Some(curva) => v != curva,
            None => allow_create && base.get(&k).is_some_and(|&antes| antes != v),
        };
        if moved {
            mexidos.insert(grupo(k));
        }
    }
    let mut plan = AutokeyPlan::default();
    if mexidos.is_empty() {
        return plan;
    }
    for &(k, v) in lidos {
        if !mexidos.contains(&grupo(k)) {
            continue;
        }
        let p = PropKind::Field(k);
        if solo {
            plan.keys.push((p, v));
            continue;
        }
        match key_value_b(doc, entity, p, meus.get(&p).copied(), v, t_secs) {
            Ok(guardar) => plan.keys.push((p, guardar)),
            Err(_) => plan.refused.push(p),
        }
    }
    plan
}

#[cfg(test)]
#[path = "autokey_campos_tests.rs"]
mod tests;
