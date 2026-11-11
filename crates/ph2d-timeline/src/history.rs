//! [`TimelineHistory`] — bounded undo/redo over [`TimelineDoc`] snapshots.
//!
//! Mirrors `ph2d-vec-edit`'s `History`: `begin` snapshots the doc at a gesture's
//! start, `commit_if_changed` at its end turns it into an undo step **only if
//! the doc actually changed** (content equality, so a click that moved nothing
//! never pollutes the stack). Atomic operations use [`TimelineHistory::push`]
//! directly. Panel-only state (selection, pan/zoom, flags) is **not** snapshotted
//! — it is never undoable (per plan B2/T4).

use crate::doc::TimelineDoc;
use crate::state::SelectedKey;
use ph2d_board_model::undo_tick;

/// Maximum number of undo steps kept (older steps drop off the bottom).
pub const HISTORY_CAP: usize = 128;

/// Bounded undo/redo stack of document snapshots.
#[derive(Debug, Default, Clone)]
pub struct TimelineHistory {
    /// Cada passo leva a hora do relógio único das pilhas (`undo_tick`): o Ctrl+Z vai à fila mais nova.
    /// O 3.º campo é a transação que liga o passo ao passo GLOBAL do mesmo gesto
    /// ([`Self::liga_gesto_da_cena`]): um Ctrl+Z desfaz os dois.
    undo: Vec<(u64, TimelineDoc, Option<u64>)>,
    redo: Vec<(u64, TimelineDoc, Option<u64>)>,
    pending: Option<TimelineDoc>,
    /// A hora do passo que um gesto da CENA escreveu (o Auto Key), à espera do passo global dele.
    gesto_da_cena: Option<u64>,
    /// Keys moved inside the open gesture ([`Self::note_moved`]).
    moved: Vec<SelectedKey>,
    /// ⭐ As covas das purgas de donos mortos ([`ph2d_ecs::graveyard`]) — o undo global que devolve
    /// o objecto devolve os clipes no mesmo quadro (`ph2d-timeline-persist`).
    pub graveyard: ph2d_ecs::Graveyard<TimelineDoc>,
}

impl TimelineHistory {
    /// An empty history.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot the current doc before a gesture that may mutate it.
    pub fn begin(&mut self, doc: &TimelineDoc) {
        self.pending = Some(doc.clone());
    }

    /// The snapshot of the gesture in flight (the document as it was at [`begin`](Self::begin)).
    #[must_use]
    pub fn pending(&self) -> Option<&TimelineDoc> {
        self.pending.as_ref()
    }

    /// The keys this gesture has moved so far: they absorb what lies under them only where the
    /// gesture closes (`intent_apply`), never at a frame they merely crossed.
    pub(crate) fn note_moved(&mut self, keys: &[SelectedKey]) {
        for k in keys {
            if !self.moved.contains(k) {
                self.moved.push(*k);
            }
        }
    }

    /// Hand over (and forget) the keys this gesture moved.
    pub(crate) fn take_moved(&mut self) -> Vec<SelectedKey> {
        std::mem::take(&mut self.moved)
    }

    /// Close a gesture: if `doc` differs from the [`begin`](Self::begin)
    /// snapshot, push an undo step (and clear redo); otherwise discard it.
    pub fn commit_if_changed(&mut self, doc: &TimelineDoc) -> bool {
        if let Some(pre) = self.pending.take()
            && &pre != doc
        {
            self.push(pre);
            return true;
        }
        false
    }

    /// [`Self::commit_if_changed`] de um passo que um gesto da CENA escreveu (o Auto Key): ele
    /// espera o passo global do mesmo gesto para se ligar a ele ([`Self::liga_gesto_da_cena`]).
    pub fn commit_da_cena(&mut self, doc: &TimelineDoc) -> bool {
        let fez = self.commit_if_changed(doc);
        if fez {
            self.gesto_da_cena = self.undo.last().map(|s| s.0);
        }
        fez
    }

    /// O passo global do gesto registou-se: se o passo que a cena escreveu ainda é o topo, os dois
    /// ficam UMA transação (devolvida, para o passo global a levar).
    pub fn liga_gesto_da_cena(&mut self) -> Option<u64> {
        let hora = self.gesto_da_cena.take()?;
        let topo = self.undo.last_mut().filter(|s| s.0 == hora)?;
        let t = *topo.2.get_or_insert_with(undo_tick);
        Some(t)
    }

    /// O gesto da cena não deu passo global (nada mudou no documento): o passo fica sozinho.
    pub fn esquece_gesto_da_cena(&mut self) {
        self.gesto_da_cena = None;
    }

    /// A transação do passo que o próximo desfazer (`redo = false`) ou refazer tiraria.
    #[must_use]
    pub fn transacao_do_topo(&self, redo: bool) -> Option<u64> {
        if redo { &self.redo } else { &self.undo }
            .last()
            .and_then(|s| s.2)
    }

    /// Discard the pending snapshot without making a step (cancelled gesture).
    pub fn cancel(&mut self) {
        self.pending = None;
        self.moved.clear();
    }

    /// O `Esc` a meio de um gesto: fecha-o sem passo e devolve o documento do [`Self::begin`].
    pub fn desiste(&mut self) -> Option<TimelineDoc> {
        self.moved.clear();
        self.pending.take()
    }

    /// Whether a gesture is currently bracketed (a `begin` awaits its commit).
    /// Atomic edits check this so an edit *inside* a gesture does not close it —
    /// a handle drag emits one edit per frame and must undo as a single step.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.pending.is_some()
    }

    /// Push a known-changed pre-state directly onto undo (atomic op).
    pub fn push(&mut self, pre: TimelineDoc) {
        if self.undo.len() >= HISTORY_CAP {
            self.undo.remove(0);
        }
        self.undo.push((undo_tick(), pre, None));
        self.redo.clear();
    }

    /// Whether an undo step is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// O passo que o próximo desfazer (`redo = false`) ou refazer tiraria é mais NOVO que `outro`
    /// (a hora do topo da outra fila)? O Ctrl+Z desfaz o passo mais recente entre as filas.
    #[must_use]
    pub fn mais_novo_que(&self, redo: bool, outro: Option<u64>) -> bool {
        let lado = if redo { &self.redo } else { &self.undo };
        lado.last().is_some_and(|s| Some(s.0) > outro)
    }

    /// Whether a redo step is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Toda fotografia guardada, mutável — para uma mudança de IDENTIDADE que não é um passo
    /// (a binding que muda de dono, [`crate::place_owner`]): ela tem de valer em todo passado a
    /// que o Ctrl+Z pode voltar.
    pub fn docs_mut(&mut self) -> impl Iterator<Item = &mut TimelineDoc> {
        self.undo
            .iter_mut()
            .chain(self.redo.iter_mut())
            .map(|s| &mut s.1)
            .chain(self.pending.iter_mut())
    }

    /// ⭐ O passo do topo é uma purga enterrada ([`ph2d_ecs::Graveyard::holds`])? Então o Ctrl+Z é do
    /// undo global (ele devolve o objecto, e a cova devolve os clipes) — a fila não o consome.
    #[must_use]
    pub fn top_is_buried(&self) -> bool {
        self.undo.last().is_some_and(|t| self.graveyard.holds(&t.1))
    }

    /// Tira o passo do topo se ele for `pre` — o cemitério a desfazer a purga sozinho, quando o undo
    /// global devolveu o dono. Não vai para o redo: o redo global volta a apagar, e a purga a enterrar.
    pub fn take_top_if(&mut self, pre: &TimelineDoc) -> Option<TimelineDoc> {
        (self.undo.last().map(|s| &s.1) == Some(pre)).then(|| self.undo.pop().map(|s| s.1))?
    }

    /// Undo: return the previous doc; push `current` onto redo.
    pub fn undo(&mut self, current: &TimelineDoc) -> Option<TimelineDoc> {
        let (_, prev, t) = self.undo.pop()?;
        self.redo.push((undo_tick(), current.clone(), t));
        Some(prev)
    }

    /// Redo: return the next doc; push `current` back onto undo.
    pub fn redo(&mut self, current: &TimelineDoc) -> Option<TimelineDoc> {
        let (_, next, t) = self.redo.pop()?;
        self.undo.push((undo_tick(), current.clone(), t));
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prop::PropKind;
    use ph2d_anim::{AnimValue, Interp, RationalTime};

    fn secs(t: f64) -> RationalTime {
        RationalTime::from_seconds(t)
    }

    #[test]
    fn commit_only_pushes_on_real_change() {
        let mut doc = TimelineDoc::new();
        let mut h = TimelineHistory::new();
        // A no-op gesture leaves the stack empty.
        h.begin(&doc);
        h.commit_if_changed(&doc);
        assert!(!h.can_undo());
        // A real edit becomes one step.
        h.begin(&doc);
        doc.insert_key(
            1,
            PropKind::TranslationX,
            secs(0.0),
            AnimValue::Float(5.0),
            Interp::Linear,
        );
        h.commit_if_changed(&doc);
        assert!(h.can_undo());
    }

    fn chave(doc: &mut TimelineDoc, v: f32) {
        doc.upsert_key(
            1,
            PropKind::Rotation,
            secs(0.0),
            AnimValue::Float(v),
            Interp::Hold,
        );
    }

    /// ⭐ A chave que o Auto Key escreveu num gesto da cena liga-se ao passo global desse gesto
    /// (smoke do dono 30/10: Gravity Y + Ctrl+Z «não acontece nada»), e a ligação viaja com o passo.
    #[test]
    fn a_chave_do_gesto_da_cena_liga_se_ao_passo_global_e_viaja() {
        let mut doc = TimelineDoc::new();
        let mut h = TimelineHistory::new();
        h.begin(&doc);
        chave(&mut doc, 1.0);
        assert!(h.commit_da_cena(&doc));
        let t = h.liga_gesto_da_cena().expect("ligou");
        assert_eq!(h.transacao_do_topo(false), Some(t));
        assert_eq!(h.liga_gesto_da_cena(), None, "uma ligação por gesto");
        let back = h.undo(&doc).unwrap();
        assert_eq!(
            (h.transacao_do_topo(false), h.transacao_do_topo(true)),
            (None, Some(t))
        );
        h.redo(&back).unwrap();
        assert_eq!(
            h.transacao_do_topo(false),
            Some(t),
            "o refazer volta ligado"
        );
    }

    /// Um passo da timeline por cima do da cena (editou-se na dope sheet) ou um gesto sem passo
    /// global deixam a chave sozinha — nunca se liga a um passo global de OUTRO gesto.
    #[test]
    fn a_chave_da_cena_nao_se_liga_a_outro_gesto() {
        let mut doc = TimelineDoc::new();
        let mut h = TimelineHistory::new();
        h.begin(&doc);
        chave(&mut doc, 1.0);
        h.commit_da_cena(&doc);
        h.begin(&doc);
        chave(&mut doc, 2.0);
        h.commit_if_changed(&doc);
        assert_eq!(h.liga_gesto_da_cena(), None, "o topo já é outro passo");
        h.begin(&doc);
        chave(&mut doc, 3.0);
        h.commit_da_cena(&doc);
        h.esquece_gesto_da_cena();
        assert_eq!(h.liga_gesto_da_cena(), None, "o gesto não deu passo global");
        assert_eq!(h.transacao_do_topo(false), None);
    }

    #[test]
    fn undo_redo_restores_snapshots() {
        let mut doc = TimelineDoc::new();
        let mut h = TimelineHistory::new();
        let empty = doc.clone();
        h.begin(&doc);
        doc.insert_key(
            1,
            PropKind::Rotation,
            secs(0.0),
            AnimValue::Float(1.0),
            Interp::Hold,
        );
        h.commit_if_changed(&doc);
        let edited = doc.clone();

        let back = h.undo(&doc).expect("undo");
        assert_eq!(back, empty);
        assert!(h.can_redo());
        let fwd = h.redo(&back).expect("redo");
        assert_eq!(fwd, edited);
    }

    /// Auditoria F31: o Ctrl+Z desfaz o passo MAIS NOVO entre a timeline e a fila global.
    #[test]
    fn ctrl_z_desfaz_o_passo_mais_novo_entre_timeline_e_global() {
        let doc = TimelineDoc::new();
        let mut h = TimelineHistory::new();
        h.push(doc.clone());
        let global_depois = undo_tick();
        assert!(
            !h.mais_novo_que(false, Some(global_depois)),
            "o global é mais novo"
        );
        assert!(
            h.mais_novo_que(false, None),
            "sem passo global, é da timeline"
        );
        let global_antes = global_depois;
        h.push(doc.clone());
        assert!(
            h.mais_novo_que(false, Some(global_antes)),
            "a timeline é mais nova"
        );
        let _ = h.undo(&doc);
        assert!(
            h.mais_novo_que(true, Some(global_antes)),
            "refazer o mais novo"
        );
        assert!(!h.mais_novo_que(true, Some(undo_tick())));
    }
}
