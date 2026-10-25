//! ⭐ **A chave POSICIONAL do Vetor passa ao id** (F31 §A.5) — a migração semântica da v23.
//!
//! Uma chave antiga reconhece-se PELA FORMA (VETOR, `item` `NONE`, e um campo de lista cujo `i`
//! endereça uma posição: entrada da pilha com `i ≥ 1`, efeito, filtro), sem bandeira gravada: um
//! documento meio-migrado que se grave cura-se no load seguinte, e o undo que devolva um documento
//! antigo também. A posição traduz-se pelo espelho [`IdsDoVetor`] que o `ph2d-app-vec` publica;
//! sem o espelho da entidade espera, índice fora da lista fica órfã, chave nova já ligada não
//! funde (avisa uma vez).

use std::collections::BTreeSet;
use std::sync::Mutex;

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor::lista;
use ph2d_ecs::canal_vetor_ids::{IdsDaForma, IdsDoVetor};
use ph2d_ecs::{FieldKey, World};
use ph2d_timeline::{PropKind, TimelineDoc};

/// Os `(entidade, target)` cuja chave nova já estava ligada — avisados uma vez por processo.
static AVISADOS: Mutex<BTreeSet<(u64, u64)>> = Mutex::new(BTreeSet::new());

/// A posição que a chave antiga `k` endereça — `None` = não é da forma antiga.
fn forma_antiga(k: FieldKey) -> Option<(ph2d_ecs::canal_vetor::Lista, usize)> {
    if k.familia != VETOR || !k.item.is_none() {
        return None;
    }
    let l = lista(k.campo);
    Some((l, l.posicao_antiga(k.i)?))
}

/// A chave durável de `k` na forma `ids` — `None` = órfã (índice fora da lista) ou não antiga.
fn duravel(k: FieldKey, ids: &IdsDaForma) -> Option<FieldKey> {
    let (l, pos) = forma_antiga(k)?;
    let id = *ids.de(l)?.get(pos)?;
    (!id.is_none()).then(|| FieldKey::do_item(VETOR, k.campo, id, k.j))
}

/// Corre DEPOIS da cura (os bindings vivos têm a entidade deste quadro). Não aloca (só o aviso).
pub(crate) fn migra_chaves_do_vetor(doc: &mut TimelineDoc, world: &World) {
    let Some(espelho) = world.get_resource::<IdsDoVetor>() else {
        return;
    };
    let candidata = |b: &ph2d_timeline::TargetBinding| match b.prop {
        PropKind::Field(k) if !b.missing => {
            duravel(k, espelho.0.get(&b.entity)?).map(|n| (b.target, PropKind::Field(n)))
        }
        _ => None,
    };
    // Por índice: o re-chavear não muda o nº nem a ordem dos bindings.
    for n in 0..doc.bindings().len() {
        let b = &doc.bindings()[n];
        let entidade = b.entity;
        let Some((target, nova)) = candidata(b) else {
            continue;
        };
        if doc.rechaveia(target, nova) {
            continue;
        }
        let mut avisados = AVISADOS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if avisados.insert((entidade, target.get())) {
            eprintln!(
                "[timeline] a chave posicional do alvo {} não passa a {nova:?}: já ligada (não funde)",
                target.get()
            );
        }
    }
}

/// O `i` que o nome da linha de `k` (uma chave de item) mostrava antes do id — `None` = não é de
/// lista, ou o espelho não tem o item.
pub(crate) fn i_antigo(k: FieldKey, ids: &IdsDaForma) -> Option<usize> {
    if k.familia != VETOR || k.item.is_none() {
        return None;
    }
    let l = lista(k.campo);
    Some(l.i_antigo(ids.posicao(l, k.item)?))
}

/// O nome da linha de cada chave de item do Vetor em `tracks`: o rótulo do grupo e o `i` antigo
/// (o 0 sem número) — o texto que o dope-sheet dava com a chave posicional.
pub(crate) fn publica_nomes(
    tracks: &[ph2d_timeline::TrackView],
    channel_names: &mut std::collections::BTreeMap<FieldKey, String>,
    world: &World,
) {
    let Some(ids) = world.get_resource::<IdsDoVetor>() else {
        return;
    };
    for t in tracks {
        if let PropKind::Field(k) = t.prop
            && let Some(info) = k.info()
            && let Some(n) = ids.0.get(&t.entity).and_then(|f| i_antigo(k, f))
        {
            let rotulo = ph2d_i18n::tr(info.rotulo);
            let nome = if n > 0 {
                format!("{rotulo} {n}")
            } else {
                rotulo.to_owned()
            };
            channel_names.insert(k, nome);
        }
    }
}
