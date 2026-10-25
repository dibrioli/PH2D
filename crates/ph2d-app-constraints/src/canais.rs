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
use ph2d_ecs::canal_constraint::{self as cc, FORCA, LIGADA};
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, FieldKey, SimWorld};
use ph2d_preview_drive::{FieldRef, PreviewDrive};
use serde_json::Value;

fn bit(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

#[expect(clippy::cast_possible_truncation, reason = "a força é 0..=1")]
fn ler(c: &Constraints, k: FieldKey) -> Option<f32> {
    let it = c.item(cc::item_de(k))?;
    match k.campo {
        FORCA => Some(it.influence as f32),
        LIGADA => Some(bit(it.enabled)),
        _ => None,
    }
}

/// Publica, para cada entidade com pilha, a força e o liga/desliga de cada item.
pub fn publicar(sim: &mut SimWorld, entidades: &[u64]) {
    let lotes: Vec<(u64, FieldKey, f32)> = entidades
        .iter()
        .filter_map(|&b| Some((b, sim.world().get::<Constraints>(Entity::try_from_bits(b)?)?)))
        .flat_map(|(b, c)| {
            c.items.iter().flat_map(move |it| {
                cc::CAMPOS.map(|campo| cc::chave(campo, it.id)).map(move |k| (b, k, ler(c, k)))
            })
        })
        .filter_map(|(b, k, v)| Some((b, k, v?)))
        .collect();
    let mut lidos = sim.world_mut().get_resource_or_insert_with(CanaisLidos::default);
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

/// A chave serde de um campo do item na posição `pos` (`"items[2]/influence"`). O ledger pede uma
/// chave `'static`; o cache cresce até ao maior índice já conduzido e nunca mais.
fn chave_serde(pos: usize, campo: u16) -> Option<&'static str> {
    let nome = match campo {
        FORCA => "influence",
        LIGADA => "enabled",
        _ => return None,
    };
    static CACHE: OnceLock<Mutex<BTreeMap<(usize, u16), &'static str>>> = OnceLock::new();
    let mut c = CACHE.get_or_init(Mutex::default).lock().ok()?;
    Some(
        *c.entry((pos, campo))
            .or_insert_with(|| Box::leak(format!("items[{pos}]/{nome}").into_boxed_str())),
    )
}

fn json(campo: u16, v: f32) -> Value {
    if campo == LIGADA {
        Value::Bool(v >= 0.5)
    } else {
        let f = if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 };
        Value::from(f64::from(f))
    }
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
        let (Some(antes), Some(key)) = (ler(c, k), chave_serde(pos, k.campo)) else {
            continue;
        };
        let novo = json(k.campo, v);
        let velho = json(k.campo, antes);
        if novo != velho {
            if let Some(mut c) = sim.world_mut().get_mut::<Constraints>(e)
                && let Some(it) = c.item_mut(item)
            {
                match k.campo {
                    FORCA => it.influence = novo.as_f64().unwrap_or(it.influence),
                    _ => it.enabled = novo.as_bool().unwrap_or(it.enabled),
                }
            }
            if !ph2d_ecs::is_transient(sim.world(), e) {
                drive.fields.driven(e, FieldRef { key, ..porta() }, velho, novo, false);
            }
            mudou = true;
        }
        if let Some(ficou) = sim.world().get::<Constraints>(e).and_then(|c| ler(c, k)) {
            ecos.push((b, k, v, ficou));
        }
    }
    let mut lidos = sim.world_mut().get_resource_or_insert_with(CanaisLidos::default);
    for (b, k, v, f) in ecos {
        lidos.eco(b, k, v, f);
    }
    mudou
}
