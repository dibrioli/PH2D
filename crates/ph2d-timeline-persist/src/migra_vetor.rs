//! ⭐ **O nome da linha de uma chave de item do Vetor** (F31 §A): o dope-sheet diz a POSIÇÃO do
//! item («Fill 2»), traduzida pelo espelho [`IdsDoVetor`] que o `ph2d-app-vec` publica.

use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor::lista;
use ph2d_ecs::canal_vetor_ids::{IdsDaForma, IdsDoVetor};
use ph2d_ecs::{FieldKey, World};
use ph2d_timeline::PropKind;

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
