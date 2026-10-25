//! **A família LISTA** — o canal de um campo de um ITEM de uma lista de componente (F30 §A1,
//! `docs/UI_New_and_Simple/F30_plano.md` §A1): o timer, o tween, o estado, a seta…
//!
//! A chave: `campo = codigo << 10 | field_id << 2 | faixa` (o vocabulário é o do catálogo,
//! `ph2d_component_desc::lista`), `i:j` = o [`ItemId`] do item — a IDENTIDADE, nunca a posição:
//! apagar ou reordenar um item não desloca a chave de outro. Item apagado ⇒ chave órfã.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use ph2d_component_desc::FieldDesc;
use ph2d_component_desc::lista::{self, ListaDesc};

use crate::ItemId;
use crate::canal::familia::LISTA;
use crate::canal::{CampoInfo, FieldKey};

/// A chave da faixa `faixa` do campo `field_id` do item `item` da lista `codigo`.
#[must_use]
pub fn chave(codigo: u16, field_id: u16, faixa: u16, item: ItemId) -> Option<FieldKey> {
    let campo = lista::campo(codigo, field_id, faixa)?;
    #[expect(clippy::cast_possible_truncation, reason = "as duas metades do id")]
    let (i, j) = ((item.0 >> 16) as u16, item.0 as u16);
    Some(FieldKey::new(LISTA, campo, i, j))
}

/// O item que a chave endereça.
#[must_use]
pub fn item(k: FieldKey) -> ItemId {
    ItemId(u32::from(k.i) << 16 | u32::from(k.j))
}

/// A lista, o campo do item e a faixa que a chave nomeia.
#[must_use]
pub fn resolver(k: FieldKey) -> Option<(&'static ListaDesc, &'static FieldDesc, u16)> {
    if k.familia != LISTA {
        return None;
    }
    let (codigo, field_id, faixa) = lista::decompor(k.campo);
    let l = lista::lista(codigo)?;
    let f = l.campo(field_id)?;
    (faixa < f.faixas_de_canal()).then_some((l, f, faixa))
}

/// Todas as chaves de um item da lista `l`, campo a campo e faixa a faixa.
pub fn chaves_do_item(l: &'static ListaDesc, item: ItemId) -> impl Iterator<Item = FieldKey> {
    l.campos.iter().flat_map(move |f| {
        (0..f.faixas_de_canal()).filter_map(move |x| chave(l.codigo, f.field_id, x, item))
    })
}

/// O grupo (o losango único de x/y, a cor inteira) — índice denso de `(codigo, field_id)`, só em
/// sessão; parte-se pelo item ([`FieldKey::grupo`]).
fn grupos() -> &'static BTreeMap<(u16, u16), u16> {
    static G: OnceLock<BTreeMap<(u16, u16), u16>> = OnceLock::new();
    G.get_or_init(|| {
        let mut g = BTreeMap::new();
        for l in lista::todas() {
            for f in l.campos {
                let n = u16::try_from(g.len()).unwrap_or(u16::MAX);
                g.insert((l.codigo, f.field_id), n);
            }
        }
        g
    })
}

#[must_use]
pub fn info(k: FieldKey) -> Option<CampoInfo> {
    let (l, f, _) = resolver(k)?;
    Some(CampoInfo {
        rotulo: f.label_key,
        especie: crate::canal_componente::especie(f),
        grupo: *grupos().get(&(l.codigo, f.field_id))?,
        por_i: true,
    })
}

#[must_use]
pub fn membro(k: FieldKey) -> Option<&'static str> {
    let (_, f, faixa) = resolver(k)?;
    crate::canal_componente::nome_da_faixa(f, faixa)
}

#[cfg(test)]
#[path = "canal_lista_tests.rs"]
mod tests;
