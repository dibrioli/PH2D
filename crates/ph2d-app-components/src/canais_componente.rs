//! **O funil da família COMPONENTE** — a curva de QUALQUER campo do catálogo chega ao componente
//! (`ph2d_ecs::canal_componente`; plano `docs/UI_New_and_Simple/F27_bloco_B_plano.md`).
//!
//! Lê e escreve o valor GUARDADO pela chave serde (`property::slot_in`, a mesma lei dos scripts e
//! do ledger), passa pela lei do componente (`property_laws::sanitize`) e declara a escrita ao
//! `FieldDrive` — o que a curva escreve vê-se, não se guarda, como o que um script escreve.

use ph2d_component_desc::{FieldDesc, FieldKind, Target, View};
use ph2d_ecs::canal::familia::COMPONENTE;
use ph2d_ecs::canal_componente as cc;
use ph2d_ecs::scene::property::slot_in;
use ph2d_ecs::scene::property_ref::{self, RefNames};
use ph2d_ecs::scene::registry::{ComponentRegistry, ComponentTypeEntry};
use ph2d_ecs::{CanaisEscritos, CanaisLidos, Entity, FieldKey, SimWorld, TextosDosCanais, World};
use ph2d_preview_drive::{FieldRef, PreviewDrive};
use ph2d_tags::TagTree;
use serde_json::Value;

/// A entrada do registo e o campo que a chave nomeia.
fn alvo(
    reg: &ComponentRegistry,
    k: FieldKey,
) -> Option<(
    &ComponentTypeEntry,
    &'static ph2d_component_desc::FieldDesc,
    u16,
)> {
    let (d, f, faixa) = cc::resolver(k)?;
    Some((reg.get_by_name(d.canonical_name)?, f, faixa))
}

/// Os canais que a entidade tem: cada campo animável de cada componente do catálogo que ela traz.
#[must_use]
pub fn campos(reg: &ComponentRegistry, w: &World, e: Entity) -> Vec<FieldKey> {
    reg.iter()
        .filter_map(|entry| Some((entry.desc?, (entry.value_of)(w, e).ok()??)))
        .flat_map(|(d, mut v)| {
            cc::chaves_de(d)
                .filter(|&k| {
                    cc::resolver(k).is_some_and(|(_, f, faixa)| {
                        slot_in(&mut v, f.serde_key()).is_some_and(|s| {
                            cc::ler_valor(f, faixa, s).is_some()
                                || (f.kind == FieldKind::Text && (s.is_string() || s.is_null()))
                                || (referencia(f) == Some(Target::StableSet) && s.is_array())
                        })
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// O valor do canal; um texto entra na tabela da sessão (`textos`) e lê-se pelo índice.
pub fn ler(
    reg: &ComponentRegistry,
    w: &World,
    e: Entity,
    k: FieldKey,
    textos: &mut TextosDosCanais,
    names: RefNames<'_>,
) -> Option<f32> {
    let (entry, f, faixa) = alvo(reg, k)?;
    let mut v = (entry.value_of)(w, e).ok()??;
    let s = slot_in(&mut v, f.serde_key())?;
    if let Some(t) = referencia(f) {
        let nome = lista_em_texto(property_ref::name_of(w, names, t, s));
        return Some(textos.indice(nome.as_str()));
    }
    cc::ler_texto(f, s, textos).or_else(|| cc::ler_valor(f, faixa, s))
}

/// Uma escrita que mudou o componente: o que o `FieldDrive` guarda (o valor GUARDADO inteiro do
/// campo, antes e depois da lei).
pub struct Escrita {
    pub campo: FieldRef,
    pub antes: Value,
    pub depois: Value,
}

/// Escreve `x` na faixa e passa a lei do componente; `None` = nada mudou (ou a forma recusou).
pub fn escrever(
    reg: &ComponentRegistry,
    w: &mut World,
    e: Entity,
    k: FieldKey,
    x: f32,
    textos: &TextosDosCanais,
    names: RefNames<'_>,
) -> Option<Escrita> {
    let (entry, f, faixa) = alvo(reg, k)?;
    if f.view == View::Named(Target::Script) && !script_existe(w, x) {
        return None;
    }
    let mut inteiro = (entry.value_of)(w, e).ok()??;
    let guardado = |v: &mut Value| slot_in(v, f.stored_key()).cloned();
    let antes = guardado(&mut inteiro)?;
    let slot = slot_in(&mut inteiro, f.serde_key())?;
    let mudou = if let Some(t) = referencia(f) {
        escrever_ref(w, names, (t, f.name), slot, x, textos)
    } else if f.kind == FieldKind::Text {
        cc::escrever_texto(f, slot, x, textos)
    } else {
        cc::escrever_valor(f, faixa, slot, x)
    };
    if !mudou {
        return None;
    }
    (entry.set_value)(w, e, inteiro).ok()?;
    crate::property_laws::sanitize(entry.canonical_name, w, e);
    let depois = guardado(&mut (entry.value_of)(w, e).ok()??)?;
    (antes != depois).then(|| Escrita {
        campo: FieldRef::of(entry, f.stored_key()),
        antes,
        depois,
    })
}

/// Publica o valor de cada canal das `entidades` (o que o Auto Key e o losango comparam).
pub fn publicar(sim: &mut SimWorld, reg: &ComponentRegistry, entidades: &[u64], tags: &TagTree) {
    let names = RefNames { tags: Some(tags) };
    let mut textos = tirar_textos(sim);
    let lotes: Vec<(u64, Vec<(FieldKey, f32)>)> = entidades
        .iter()
        .filter_map(|&b| {
            let e = Entity::try_from_bits(b)?;
            let w = sim.world();
            w.get_entity(e).ok()?;
            let v = campos(reg, w, e)
                .into_iter()
                .filter_map(|k| Some((k, ler(reg, w, e, k, &mut textos, names)?)))
                .collect();
            Some((b, v))
        })
        .collect();
    crate::canais_script::publicar(sim, entidades, &mut textos);
    crate::canais_lista::publicar(sim, reg, entidades, &mut textos);
    sim.world_mut().insert_resource(textos);
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for &b in entidades {
        lidos.esquecer(b, COMPONENTE);
    }
    for (b, valores) in lotes {
        for (k, v) in valores {
            lidos.publicar(b, k, v);
        }
    }
}

/// ⭐ **Aplica o que a curva mandou neste quadro**, regista o eco e declara a escrita à
/// pré-visualização. Devolve `true` se algum componente mudou.
pub fn aplicar(
    sim: &mut SimWorld,
    reg: &ComponentRegistry,
    drive: &mut PreviewDrive,
    tags: &TagTree,
) -> bool {
    let names = RefNames { tags: Some(tags) };
    let Some(mut escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(COMPONENTE))
    else {
        return false;
    };
    escritas.sort_by_key(|&(b, k, _)| (b, k));
    let mut textos = tirar_textos(sim);
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    for (b, k, x) in escritas {
        let Some(e) = Entity::try_from_bits(b) else {
            continue;
        };
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        if let Some(esc) = escrever(reg, sim.world_mut(), e, k, x, &textos, names) {
            // Uma cópia da corrida não é documento: não há autorado a guardar por baixo dela.
            if !ph2d_ecs::is_transient(sim.world(), e) {
                drive
                    .fields
                    .conduzido_pela_curva(e, esc.campo, esc.antes, esc.depois);
            }
            mudou = true;
        }
        if let Some(ficou) = ler(reg, sim.world(), e, k, &mut textos, names) {
            ecos.push((b, k, x, ficou));
        }
    }
    // As propriedades que o script declara: a mesma tabela, o mesmo quadro.
    mudou |= crate::canais_script::aplicar(sim, reg, drive, &textos);
    // Os itens das listas (F30 §A1): a mesma tabela, o mesmo quadro.
    mudou |= crate::canais_lista::aplicar(sim, reg, drive, &mut textos);
    sim.world_mut().insert_resource(textos);
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (b, k, x, f) in ecos {
        lidos.eco(b, k, x, f);
    }
    mudou
}

/// O alvo de um campo que guarda uma IDENTIDADE (o hash do nome, o `StableId`, a `TagId`): o canal
/// lê e escreve o NOME, pela conversão do script (`property_ref`) — nunca o número cru.
fn referencia(f: &FieldDesc) -> Option<Target> {
    match f.view {
        View::Named(t @ (Target::NameHash | Target::Stable | Target::Tag | Target::StableSet)) => {
            Some(t)
        }
        _ => None,
    }
}

/// O separador do texto de uma lista de objectos no canal («Crate, Box»).
const LISTA_SEP: &str = ", ";

/// O valor nomeado como TEXTO do canal: um nome fica; uma lista junta-se por [`LISTA_SEP`].
fn lista_em_texto(v: Value) -> Value {
    match v {
        Value::Array(a) => Value::String(
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(LISTA_SEP),
        ),
        v => v,
    }
}

/// Guarda a identidade do alvo que o índice `x` nomeia (`0` = ninguém); um índice que a tabela
/// não tem, ou um nome sem alvo, não escreve.
fn escrever_ref(
    w: &World,
    names: RefNames<'_>,
    (t, campo): (Target, &str),
    slot: &mut Value,
    x: f32,
    textos: &TextosDosCanais,
) -> bool {
    let nome = match textos.texto(x) {
        // Uma LISTA de objectos é o texto dos nomes juntos (o `lista_em_texto`), e volta a ser a lista.
        Some(n) if t == Target::StableSet => Value::Array(
            n.split(LISTA_SEP)
                .filter(|p| !p.trim().is_empty())
                .map(|p| Value::String(p.trim().to_owned()))
                .collect(),
        ),
        Some(n) => Value::String(n.to_owned()),
        None if x.round() == 0.0 => Value::Null,
        None => return false,
    };
    match property_ref::id_of(w, names, t, campo, nome) {
        Ok(id) if *slot != id => {
            *slot = id;
            true
        }
        _ => false,
    }
}

/// O script `x` existe na biblioteca do projeto (`0` = nenhum, que existe sempre) — a curva não
/// aponta um objecto para um script que o projeto não tem.
fn script_existe(w: &World, x: f32) -> bool {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "arredondado e positivo"
    )]
    let k = x.round().max(0.0) as u64;
    k == 0
        || w.get_resource::<ph2d_script::ProjectScripts>()
            .is_some_and(|l| l.get(ph2d_script::ScriptId(k)).is_some())
}

/// A tabela dos textos da sessão, fora do mundo enquanto o funil o lê e escreve (volta no fim).
fn tirar_textos(sim: &mut SimWorld) -> TextosDosCanais {
    sim.world_mut()
        .remove_resource::<TextosDosCanais>()
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "canais_componente_tests.rs"]
mod tests;
