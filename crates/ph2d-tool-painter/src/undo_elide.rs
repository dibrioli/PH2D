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
    /// A 4.ª família (ADR-0205 D2): o vidro, sob a chave `VIDRO_PELO_JOURNAL`.
    pub(crate) vidros: BTreeMap<RtLayerId, Weak<Vec<crate::compositor::vidro::Vidro>>>,
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
        vidros: Option<&BTreeMap<RtLayerId, Arc<Vec<crate::compositor::vidro::Vidro>>>>,
    ) -> Self {
        Self {
            heights: downgrade(heights),
            covers: downgrade(covers),
            mats: downgrade(mats),
            vidros: vidros.map(downgrade).unwrap_or_default(),
        }
    }

    /// `true` quando este snapshot não elide nada.
    pub(crate) fn is_empty(&self) -> bool {
        self.heights.is_empty()
            && self.covers.is_empty()
            && self.mats.is_empty()
            && self.vidros.is_empty()
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
            && up(&self.vidros, &mut m.vidros)
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

/// `PH2D_UNDO_AUDIT`: a rede de verificação do journal ([`crate::tool::PainterTool::audit_journal_matches_the_before`]).
#[cfg(any(test, debug_assertions))]
pub(crate) fn ligada() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_UNDO_AUDIT").is_some())
}

impl super::ModelSnapshot {
    /// **A tela passa a ser DESCRITA** (B17.4): fica a testemunha, o `Arc` forte sai — no mesmo passo, pela razão do
    /// [`ElidedRelief::of`]. O lado de antes é o journal da tela sobre o plano vivo.
    pub(crate) fn elide_canvas(&mut self) {
        if self.canvas_elided.is_none() && !self.canvas_rgba.is_empty() {
            #[cfg(any(test, debug_assertions))]
            if ligada() {
                self.canvas_audit = Some(Arc::new(self.canvas_rgba.as_ref().clone()));
            }
            self.canvas_elided = Some(Arc::downgrade(&self.canvas_rgba));
            self.canvas_rgba = Arc::new(Vec::new());
        }
    }
}

impl super::UndoController {
    /// **A tela que `s` descreve, SEGURADA**: a dele quando a segura; elidida, o plano vivo devolvido pelo journal ao
    /// último reset (o próprio `Arc` vivo quando nada foi capturado). `None` = a testemunha não é o plano vivo (alguém o
    /// trocou por inteiro): o antes não existe, e quem pergunta recusa.
    pub(crate) fn canvas_held(
        &self,
        s: &super::ModelSnapshot,
        live: &Arc<Vec<u8>>,
    ) -> Option<Arc<Vec<u8>>> {
        let Some(w) = &s.canvas_elided else {
            return Some(Arc::clone(&s.canvas_rgba));
        };
        // A 1.ª escrita depois da elisão MUDA o plano de alocação (`make_mut` com só um `Weak` move, sem copiar) e a
        // testemunha morre: com o journal a falar, quem responde é ele; calado, a testemunha tem de ser o vivo.
        if self.write_state.canvas_untouched() {
            let w_live = w.upgrade().is_some_and(|p| Arc::ptr_eq(&p, live));
            return w_live.then(|| Arc::clone(live));
        }
        let mut v = live.as_ref().clone();
        self.write_state.rewind_canvas(&mut v).then(|| Arc::new(v))
    }

    /// `s` com a tela segurada ([`Self::canvas_held`]).
    pub(crate) fn with_canvas_held(
        &self,
        s: &super::ModelSnapshot,
        live: &Arc<Vec<u8>>,
    ) -> Option<super::ModelSnapshot> {
        let mut m = s.clone();
        if m.canvas_elided.is_some() {
            m.canvas_rgba = self.canvas_held(s, live)?;
            m.canvas_elided = None;
        }
        Some(m)
    }

    /// **Um passo abre — e é AQUI que o cursor larga a tela** (B17.4): só quando a tela dele É o objeto vivo e nada a
    /// escreveu desde o reset (a identidade prova os bytes). Elidir no `set_cursor` erraria: os editores de forma
    /// reinstalam outra tela depois do commit (sem captura), e o cursor tem de segurar a de antes. Se o journal tem
    /// bytes que a absorção não levou, o cursor elidido segura-os ANTES de o reset os esquecer.
    pub(crate) fn begin_step(&mut self, writes: u64, live: &Arc<Vec<u8>>) {
        #[cfg(test)]
        let elide = self.elide_cursor;
        #[cfg(not(test))]
        let elide = true;
        if elide
            && self.write_state.canvas_untouched()
            && !self.write_state.on_foreign_plane()
            && let Some(c) = self.cursor.as_deref_mut()
            && Arc::ptr_eq(&c.canvas_rgba, live)
        {
            c.elide_canvas();
        } else if !self.write_state.canvas_untouched()
            && let Some(c) = self.cursor.as_deref()
            && c.canvas_elided.is_some()
        {
            match self.with_canvas_held(c, live) {
                Some(held) => self.cursor = Some(Box::new(held)),
                None => {
                    eprintln!(
                        "[painter-undo] o cursor elidido perdeu a tela: historico descartado"
                    );
                    self.clear();
                }
            }
        }
        self.write_state.begin_step(writes);
    }
}
