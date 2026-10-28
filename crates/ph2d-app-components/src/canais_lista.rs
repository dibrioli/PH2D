//! **O funil da família LISTA** — a curva de um campo de um ITEM (um timer, um estado, uma seta…)
//! chega ao item com aquele `ItemId`, esteja ele na posição que estiver (`ph2d_ecs::canal_lista`;
//! plano `docs/UI_New_and_Simple/F30_plano.md` §A1).
//!
//! Lê e escreve o valor GUARDADO pela chave serde do item (`property::slot_in`, a lei do funil
//! COMPONENTE), passa pela lei do componente (`property_laws::sanitize`) e declara a escrita ao
//! `FieldDrive` sobre a LISTA inteira — o que a curva escreve vê-se, não se guarda.

use ph2d_component_desc::FieldKind;
use ph2d_component_desc::lista::{self, ListaDesc};
use ph2d_ecs::canal::familia::LISTA;
use ph2d_ecs::canal_componente as cc;
use ph2d_ecs::canal_lista as cl;
use ph2d_ecs::scene::property::slot_in;
use ph2d_ecs::scene::registry::{ComponentRegistry, ComponentTypeEntry};
use ph2d_ecs::{
    CanaisEscritos, CanaisLidos, Entity, FieldKey, ItemId, SimWorld, TextosDosCanais, World,
};
use ph2d_preview_drive::{FieldRef, PreviewDrive};
use serde_json::Value;

/// O id de um item no valor serde dele (`0` = ainda por atribuir: sem canal).
fn id_de(item: &Value) -> ItemId {
    let n = item.get("item").and_then(Value::as_u64).unwrap_or(0);
    ItemId(u32::try_from(n).unwrap_or(0))
}

/// Os itens da lista `l` no valor do componente.
fn itens<'v>(l: &ListaDesc, inteiro: &'v mut Value) -> Option<&'v mut Vec<Value>> {
    slot_in(inteiro, l.caminho)?.as_array_mut()
}

/// O maior id que o valor `f32` de um canal guarda EXACTO (a mantissa tem 24 bits). Um id acima
/// não publica: o canal arredondaria para OUTRO item.
const ID_EXACTO_EM_F32: u32 = 1 << 24;

/// Os ids dos itens da lista `l`, pela posição.
fn ids_da_lista(l: &ListaDesc, inteiro: &mut Value) -> Vec<ItemId> {
    itens(l, inteiro).map_or_else(Vec::new, |v| v.iter().map(id_de).collect())
}

/// `(field_id, ids da lista apontada)` de cada campo de `l` que aponta para um item ([`ListaDesc::aponta`]).
fn alvos(l: &ListaDesc, inteiro: &mut Value) -> Vec<(u16, Vec<ItemId>)> {
    l.aponta
        .iter()
        .filter_map(|&(f, _)| Some((f, ids_da_lista(l.alvo(f)?, inteiro))))
        .collect()
}

/// Os ids da lista para onde o campo `f` aponta, se aponta.
fn alvo_de(alvos: &[(u16, Vec<ItemId>)], f: u16) -> Option<&[ItemId]> {
    alvos
        .iter()
        .find(|(x, _)| *x == f)
        .map(|(_, v)| v.as_slice())
}

/// O valor de uma faixa de um campo de item; um texto entra na tabela; uma posição de item apontado
/// sai como o `ItemId` dele.
fn ler_campo(
    f: &ph2d_component_desc::FieldDesc,
    faixa: u16,
    item: &mut Value,
    textos: &mut TextosDosCanais,
    alvo: Option<&[ItemId]>,
) -> Option<f32> {
    let s = slot_in(item, f.serde_key())?;
    let v = cc::ler_texto(f, s, textos).or_else(|| cc::ler_valor(f, faixa, s))?;
    let Some(ids) = alvo else {
        return Some(v);
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "uma posição"
    )]
    let id = ids
        .get(v.max(0.0) as usize)
        .filter(|i| !i.is_none() && i.0 <= ID_EXACTO_EM_F32)?;
    #[expect(clippy::cast_precision_loss, reason = "≤ ID_EXACTO_EM_F32")]
    Some(id.0 as f32)
}

/// Os canais de cada item das listas que a entidade tem, com o valor de agora.
pub fn campos(
    reg: &ComponentRegistry,
    w: &World,
    e: Entity,
    textos: &mut TextosDosCanais,
) -> Vec<(FieldKey, f32)> {
    campos_e_posicoes(reg, w, e, textos).0
}

/// Os campos lidos e as posições dos itens de uma entidade.
type CamposEPosicoes = (Vec<(FieldKey, f32)>, Vec<(u32, u16)>);

/// [`campos`] e a posição (1, 2, …) de cada item na lista dele — só para o nome da linha.
fn campos_e_posicoes(
    reg: &ComponentRegistry,
    w: &World,
    e: Entity,
    textos: &mut TextosDosCanais,
) -> CamposEPosicoes {
    let mut out = Vec::new();
    let mut posicoes = Vec::new();
    for l in lista::todas() {
        let Some(mut inteiro) = reg
            .get_by_name(l.componente)
            .and_then(|entry| (entry.value_of)(w, e).ok().flatten())
        else {
            continue;
        };
        let alvos = alvos(l, &mut inteiro);
        let Some(itens) = itens(l, &mut inteiro) else {
            continue;
        };
        for (n, item) in itens.iter_mut().enumerate() {
            let id = id_de(item);
            if id.is_none() {
                continue;
            }
            posicoes.push((id.0, u16::try_from(n + 1).unwrap_or(u16::MAX)));
            for f in l.campos {
                for x in 0..f.faixas_de_canal() {
                    let Some(k) = cl::chave(l.codigo, f.field_id, x, id) else {
                        continue;
                    };
                    if let Some(v) = ler_campo(f, x, item, textos, alvo_de(&alvos, f.field_id)) {
                        out.push((k, v));
                    }
                }
            }
        }
    }
    (out, posicoes)
}

/// O valor de UM canal (o eco depois da escrita).
fn ler(
    reg: &ComponentRegistry,
    w: &World,
    e: Entity,
    k: FieldKey,
    textos: &mut TextosDosCanais,
) -> Option<f32> {
    let (l, f, faixa) = cl::resolver(k)?;
    let mut inteiro = (reg.get_by_name(l.componente)?.value_of)(w, e).ok()??;
    let alvo = l.alvo(f.field_id).map(|a| ids_da_lista(a, &mut inteiro));
    let item = itens(l, &mut inteiro)?
        .iter_mut()
        .find(|i| id_de(i) == cl::item(k))?;
    ler_campo(f, faixa, item, textos, alvo.as_deref())
}

/// Publica os canais dos itens das `entidades` (o que o Auto Key e o losango comparam).
pub fn publicar(
    sim: &mut SimWorld,
    reg: &ComponentRegistry,
    entidades: &[u64],
    textos: &mut TextosDosCanais,
) {
    let lotes: Vec<(u64, CamposEPosicoes)> = entidades
        .iter()
        .filter_map(|&b| {
            let e = Entity::try_from_bits(b)?;
            sim.world().get_entity(e).ok()?;
            Some((b, campos_e_posicoes(reg, sim.world(), e, textos)))
        })
        .collect();
    let mut pos = sim
        .world_mut()
        .get_resource_or_insert_with(cl::PosicoesDosItens::default);
    pos.0.retain(|(b, _), _| !entidades.contains(b));
    pos.1.retain(|b| !entidades.contains(b));
    for (b, (_, ps)) in &lotes {
        pos.0.extend(ps.iter().map(|&(id, n)| ((*b, id), n)));
        pos.1.insert(*b);
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for &b in entidades {
        lidos.esquecer(b, LISTA);
    }
    for (b, (valores, _)) in lotes {
        for (k, v) in valores {
            lidos.publicar(b, k, v);
        }
    }
}

/// A escrita de `x` no item da chave; `Some((lista, antes, depois))` se a lista mudou.
fn escrever(
    reg: &ComponentRegistry,
    w: &mut World,
    e: Entity,
    k: FieldKey,
    x: f32,
    textos: &TextosDosCanais,
) -> Option<(FieldRef, Value, Value)> {
    let (l, f, faixa) = cl::resolver(k)?;
    let entry: &ComponentTypeEntry = reg.get_by_name(l.componente)?;
    let mut inteiro = (entry.value_of)(w, e).ok()??;
    let antes = slot_in(&mut inteiro, l.caminho)?.clone();
    // A curva de um campo que aponta guarda o `ItemId`: volta à POSIÇÃO de agora desse item (o
    // item apagado não escreve nada).
    let x = match l.alvo(f.field_id) {
        #[expect(clippy::cast_precision_loss, reason = "uma posição numa lista")]
        Some(a) => ids_da_lista(a, &mut inteiro)
            .iter()
            .position(|i| !i.is_none() && f64::from(i.0) == f64::from(x).round())?
            as f32,
        None => x,
    };
    let item = itens(l, &mut inteiro)?
        .iter_mut()
        .find(|i| id_de(i) == cl::item(k))?;
    let slot = slot_in(item, f.serde_key())?;
    let mudou = if f.kind == FieldKind::Text {
        cc::escrever_texto(f, slot, x, textos)
    } else {
        cc::escrever_valor(f, faixa, slot, x)
    };
    if !mudou {
        return None;
    }
    (entry.set_value)(w, e, inteiro).ok()?;
    crate::property_laws::sanitize(entry.canonical_name, w, e);
    let depois = slot_in(&mut (entry.value_of)(w, e).ok()??, l.caminho)?.clone();
    (antes != depois).then(|| (FieldRef::of(entry, l.caminho), antes, depois))
}

/// ⭐ **Aplica o que a curva mandou neste quadro** aos itens, regista o eco e declara a escrita à
/// pré-visualização. Devolve `true` se algum item mudou.
pub fn aplicar(
    sim: &mut SimWorld,
    reg: &ComponentRegistry,
    drive: &mut PreviewDrive,
    textos: &mut TextosDosCanais,
) -> bool {
    let Some(mut escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(LISTA))
    else {
        return false;
    };
    escritas.sort_by_key(|&(b, k, _)| (b, k));
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    for (b, k, x) in escritas {
        let Some(e) = Entity::try_from_bits(b).filter(|&e| sim.world().get_entity(e).is_ok())
        else {
            continue;
        };
        if let Some((campo, antes, depois)) = escrever(reg, sim.world_mut(), e, k, x, textos) {
            if !ph2d_ecs::is_transient(sim.world(), e) {
                drive.fields.conduzido_pela_curva(e, campo, antes, depois);
            }
            mudou = true;
        }
        if let Some(ficou) = ler(reg, sim.world(), e, k, textos) {
            ecos.push((b, k, x, ficou));
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (b, k, x, f) in ecos {
        lidos.eco(b, k, x, f);
    }
    mudou
}

#[cfg(test)]
#[path = "canais_lista_tests.rs"]
mod tests;
