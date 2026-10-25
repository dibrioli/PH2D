//! **Os canais da pilha** (família [`CONSTRAINT`], vocabulário em `ph2d_ecs::canal_constraint`): o
//! que o documento vale para o Auto Key ([`publicar`]) e o que a curva manda escrever ([`aplicar`]).
//!
//! A curva escreve o campo do item e declara-o ao `PreviewDrive` (`fields`): *a corrida não é
//! documento* — gravar a meio do play grava o autorado, como nos campos do catálogo.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use bevy_ecs::entity::Entity;
use ph2d_constraints_ecs::{CONSTRAINTS_TYPE_NAME, Constraints, register_constraint_components};
use ph2d_ecs::canal::familia::CONSTRAINT;
use ph2d_ecs::canal_constraint as cc;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, FieldKey, SimWorld};
use ph2d_preview_drive::{FieldRef, PreviewDrive};
use serde_json::Value;

fn ler(c: &Constraints, k: FieldKey) -> Option<f32> {
    crate::campos::ler_item(c.item(cc::item_de(k))?, k.campo)
}

/// Publica, para cada entidade com pilha, cada campo de cada item ([`crate::campos`]).
pub fn publicar(sim: &mut SimWorld, entidades: &[u64]) {
    let lotes: Vec<(u64, FieldKey, f32)> = entidades
        .iter()
        .filter_map(|&b| {
            Some((
                b,
                sim.world().get::<Constraints>(Entity::try_from_bits(b)?)?,
            ))
        })
        .flat_map(|(b, c)| {
            c.items.iter().flat_map(move |it| {
                cc::CAMPOS
                    .map(|campo| cc::chave(campo, it.id))
                    .map(move |k| (b, k, ler(c, k)))
            })
        })
        .filter_map(|(b, k, v)| Some((b, k, v?)))
        .collect();
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for &b in entidades {
        lidos.esquecer(b, CONSTRAINT);
    }
    for (b, k, v) in lotes {
        lidos.publicar(b, k, v);
    }
}

/// As portas da vtable do componente, sem o registo da shell.
fn porta() -> FieldRef {
    static P: OnceLock<FieldRef> = OnceLock::new();
    *P.get_or_init(|| {
        let mut reg = ComponentRegistry::new();
        register_constraint_components(&mut reg);
        let e = reg
            .get_by_name(CONSTRAINTS_TYPE_NAME)
            .expect("o registo da própria crate");
        FieldRef::of(e, "items")
    })
}

/// A chave serde de um campo do item na posição `pos` (`"items[2]/kind/Limit/rot[0]"`). O ledger
/// pede uma chave `'static`; o cache cresce até ao maior índice já conduzido e nunca mais.
fn chave_serde(pos: usize, campo: &'static str) -> Option<&'static str> {
    static CACHE: OnceLock<Mutex<BTreeMap<(usize, &'static str), &'static str>>> = OnceLock::new();
    let mut c = CACHE.get_or_init(Mutex::default).lock().ok()?;
    Some(
        *c.entry((pos, campo))
            .or_insert_with(|| Box::leak(format!("items[{pos}]/{campo}").into_boxed_str())),
    )
}

/// O valor serde de um campo do item — o que o ledger guarda e repõe.
fn json(it: &ph2d_constraints_ecs::Item, campo: &str) -> Option<Value> {
    let mut v = serde_json::to_value(it).ok()?;
    ph2d_ecs::scene::property::slot_in(&mut v, campo).cloned()
}

/// Drena o que a curva mandou: escreve o campo do item e declara a escrita ao `PreviewDrive`.
/// Devolve se alguma coisa mudou.
pub fn aplicar(sim: &mut SimWorld, drive: &mut PreviewDrive) -> bool {
    let Some(escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(CONSTRAINT))
    else {
        return false;
    };
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    for (b, k, v) in escritas {
        let Some(e) = Entity::try_from_bits(b) else {
            continue;
        };
        let Some(c) = sim.world().get::<Constraints>(e) else {
            continue;
        };
        let item = cc::item_de(k);
        let Some(pos) = c.items.iter().position(|i| i.id == item) else {
            continue;
        };
        let antes = c.items[pos];
        let mut depois = antes;
        let Some(campo) = crate::campos::chave_no_item(&antes.kind, k.campo) else {
            continue;
        };
        if crate::campos::escrever_item(&mut depois, k.campo, v)
            && let (Some(velho), Some(novo), Some(key)) = (
                json(&antes, campo),
                json(&depois, campo),
                chave_serde(pos, campo),
            )
        {
            if let Some(mut c) = sim.world_mut().get_mut::<Constraints>(e)
                && let Some(it) = c.item_mut(item)
            {
                *it = depois;
            }
            if !ph2d_ecs::is_transient(sim.world(), e) {
                drive
                    .fields
                    .driven(e, FieldRef { key, ..porta() }, velho, novo, false);
            }
            mudou = true;
        }
        if let Some(ficou) = sim.world().get::<Constraints>(e).and_then(|c| ler(c, k)) {
            ecos.push((b, k, v, ficou));
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (b, k, v, f) in ecos {
        lidos.eco(b, k, v, f);
    }
    mudou
}
