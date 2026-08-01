//! **O TERCEIRO estado do relevo num snapshot** — filho de [`super`] (`#[path]`, então ele enxerga os
//! privados do controller), o degrau 4 do S3 (doc 28 §5.60).
//!
//! # Por que TRÊS estados, e não dois
//!
//! Um [`ModelSnapshot`](super::ModelSnapshot) guardava o relevo em dois: a chave **está** no mapa (o
//! plano existia) ou **não está** (não existia). O degrau 4 precisa de um terceiro — *existia, e eu
//! não o seguro* — e a §5.60 mediu o que acontece sem ele. Com o `before` simplesmente esvaziado,
//! [`StoredMap::from_journal`](crate::undo_delta::StoredMap::from_journal) lê toda chave pelo braço
//! `(None, Some(a))`, que **significa `OnlyAfter` = "não existia antes"** ⇒ desfazer REMOVE a chave, e
//! **o undo apaga o relevo**. E o mesmo braço guarda um `Arc` **forte** do plano VIVO, então a elisão
//! nem removia um dono: ela **trocava o dono de lugar** (medido: 3 donos no gesto → 2 com o `before`
//! elidido → 3 de novo depois do commit).
//!
//! # Por que um `Weak`, e não um `BTreeSet` de chaves
//!
//! O conjunto de chaves basta para o *significado*; o `Weak` é a **TESTEMUNHA**. O lado `before` de um
//! passo elidido não é um buffer — ele é `journal.get(i).unwrap_or(vivo[i])`, a identidade da §5.28 —,
//! e essa identidade só vale enquanto **o plano vivo de agora É o plano que este snapshot descrevia**.
//!
//! Há DUAS escritas de produção que trocam o plano inteiro sem passar por porta de captura (o eraser
//! em `impasto.rs`, o reset do warp em `warp/relief.rs`), e um `BTreeSet` não teria como saber: o undo
//! reconstruiria o `before` sobre um fundo estranho e **nada no sistema piscaria**. A cura de projeto
//! é a captura naqueles sítios; a testemunha é a rede para o sítio que ninguém listou — *enumeração
//! apodrece, testemunha não* (§13.13).
//!
//! ⚠️ **E o `Weak` é o handle certo por um segundo motivo, que a §5.12 pagou no Wet Paint:** ele prende
//! a ALOCAÇÃO, então nenhum `Arc` futuro pode nascer naquele endereço ⇒ a comparação de ponteiro é sã.
//! Um ponteiro cru teria o ABA que o ADR-0124 documentou no editor de áudio.
//!
//! ⚠️ **E um `Weak` NÃO conta como dono:** `Arc::make_mut` só COPIA na presença de outro **strong**
//! (com só um `Weak` vivo ele *move* — 0,0000 ms medidos na §5.12), e a porta de fork pergunta
//! `strong_count > 1` justamente por isso (§5.15). É essa propriedade que torna a elisão um ganho em
//! vez de um rearranjo.

use std::collections::BTreeMap;
use std::sync::{Arc, Weak};

use crate::layers::LayerId as RtLayerId;
use ph2d_painter_brush::material::MaterialBytes;

/// As três famílias de relevo que um snapshot **descreve sem segurar**.
///
/// Vazio = o snapshot não elide nada (ele segura o que tem — o mundo pré-degrau-4). Não há um quarto
/// estado escondido: um snapshot que elidiu uma camada SEM relevo e um que não elidiu nada são a
/// mesma coisa, e as duas levam ao mesmo delta.
#[derive(Clone, Debug, Default)]
pub struct ElidedRelief {
    pub(crate) heights: BTreeMap<RtLayerId, Weak<Vec<f32>>>,
    pub(crate) covers: BTreeMap<RtLayerId, Weak<Vec<u8>>>,
    pub(crate) mats: BTreeMap<RtLayerId, Weak<Vec<MaterialBytes>>>,
}

fn downgrade<T>(m: &BTreeMap<RtLayerId, Arc<Vec<T>>>) -> BTreeMap<RtLayerId, Weak<Vec<T>>> {
    m.iter().map(|(k, v)| (*k, Arc::downgrade(v))).collect()
}

impl ElidedRelief {
    /// As testemunhas dos três mapas de um tool VIVO.
    ///
    /// ⚠️ Quem chama **esvazia os mapas fortes no MESMO passo** — um snapshot que elide e segura
    /// carrega o fato duas vezes, e as duas cópias divergem no dia em que uma delas é escrita.
    pub(crate) fn of(
        heights: &BTreeMap<RtLayerId, Arc<Vec<f32>>>,
        covers: &BTreeMap<RtLayerId, Arc<Vec<u8>>>,
        mats: &BTreeMap<RtLayerId, Arc<Vec<MaterialBytes>>>,
    ) -> Self {
        Self {
            heights: downgrade(heights),
            covers: downgrade(covers),
            mats: downgrade(mats),
        }
    }

    /// `true` quando este snapshot não elide nada.
    pub(crate) fn is_empty(&self) -> bool {
        self.heights.is_empty() && self.covers.is_empty() && self.mats.is_empty()
    }

    /// **Devolve os planos ao snapshot, pelas testemunhas** — `false` quando alguma não dá upgrade.
    ///
    /// ⚠️ **A pré-condição é do CHAMADOR e não é verificável daqui:** o `Weak` prova que o objeto
    /// ainda vive, **não** que o conteúdo dele é o de quando o snapshot foi tirado — a elisão faz as
    /// escritas serem NO LUGAR, então o mesmo objeto pode carregar bytes de depois. Quem chama tem de
    /// saber que ninguém escreveu no meio, e no produto quem sabe isso é o JOURNAL
    /// ([`UndoController::base_for_top`](super::UndoController::base_for_top)).
    pub(crate) fn rehydrate_into(&self, m: &mut super::ModelSnapshot) -> bool {
        fn up<T>(
            src: &BTreeMap<RtLayerId, Weak<Vec<T>>>,
            dst: &mut BTreeMap<RtLayerId, Arc<Vec<T>>>,
        ) -> bool {
            src.iter().all(|(k, w)| match w.upgrade() {
                Some(p) => {
                    dst.insert(*k, p);
                    true
                }
                None => false,
            })
        }
        up(&self.heights, &mut m.heights)
            && up(&self.covers, &mut m.covers)
            && up(&self.mats, &mut m.mats)
    }
}

/// **A testemunha, para UMA família e UMA camada.**
///
/// - `None` = a chave não está elidida ⇒ quem pergunta usa o mapa forte, como sempre.
/// - `Some(true)` = elidida, e o plano vivo **é** o objeto que o snapshot descrevia ⇒ o `before` é
///   reconstruível por `journal.get(i).unwrap_or(vivo[i])`.
/// - `Some(false)` = elidida, e o plano **mudou de identidade**. Alguém o trocou por inteiro, e o
///   `before` daquele passo não existe mais em lugar nenhum. A resposta é **recusar** — nunca
///   inventar um fundo.
pub(crate) fn witness<T>(
    elided: &BTreeMap<RtLayerId, Weak<Vec<T>>>,
    k: RtLayerId,
    live: &Arc<Vec<T>>,
) -> Option<bool> {
    let w = elided.get(&k)?;
    Some(w.upgrade().is_some_and(|p| Arc::ptr_eq(&p, live)))
}
