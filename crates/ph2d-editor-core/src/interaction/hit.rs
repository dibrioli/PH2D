//! [`HitIndex`] — `(NodeId, Rect)` slots indexed by paint order.
//!
//! Paint pass calls [`HitIndex::register`] as it emits each
//! interactive widget's geometry. Pointer dispatch consults
//! [`HitIndex::hit`], which walks back-to-front so the most-recently
//! painted (top-most z) widget wins for overlapping rects.
//!
//! Backing store is [`smallvec::SmallVec`] inline up to 128 entries
//! (covers the ~31 widgets in the current hero with room for several
//! more screens). Beyond 128, it spills to heap — visible in
//! profiler. [`HitIndex::clear_for_frame`] drops length to 0 without
//! freeing capacity.

use crate::zones::Rect;
use ph2d_a11y::NodeId;
use smallvec::SmallVec;

const INLINE_CAPACITY: usize = 128;

#[derive(Debug, Default)]
pub struct HitIndex {
    /// Painted in z-order (back→front). Iteration for `hit` reverses
    /// so first match is the top-most widget.
    rects: SmallVec<[(NodeId, Rect); INLINE_CAPACITY]>,
    /// Pilha de RECORTES abertos. Ver [`HitIndex::push_clip`].
    clips: SmallVec<[Rect; 4]>,
}

impl HitIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drop registered rects without freeing capacity. Called once
    /// per frame before the paint pass re-registers everything.
    pub fn clear_for_frame(&mut self) {
        self.rects.clear();
        // ⚠️ **Um recorte por fechar não pode atravessar o quadro.** Se um pintor entrar em
        // pânico ou sair por um caminho que salta o `pop_clip`, o quadro SEGUINTE herdaria a
        // janela e o painel inteiro ficaria mudo sob o rato — um defeito que se cura sozinho
        // por um quadro e nunca se reproduz. O limpar é aqui porque é aqui que o quadro começa.
        self.clips.clear();
    }

    /// Register a widget's rect. Paint pass calls this for each
    /// interactive widget as it emits geometry. Multiple registrations
    /// for the same `NodeId` are allowed — the latest wins for hit
    /// (used when a popover / tooltip overlays an existing widget).
    ///
    /// **Pitfall**: hit() walks back-to-front, so registering a parent
    /// container's rect AFTER its children's rects shadows them
    /// (see `docs/UI_Bugs/README.md` §1.1). When a parent only acts
    /// through its children, don't register the parent at all.
    pub fn register(&mut self, id: NodeId, rect: Rect) {
        let Some(visible) = self.clipped(rect) else {
            return;
        };
        self.rects.push((id, visible));
    }

    /// **Abre um RECORTE: daqui até ao [`Self::pop_clip`], o que for registado fora deste
    /// rectângulo não é clicável.**
    ///
    /// ⚠️ **O NEUTRO é não chamar** — sem recorte aberto o `register` é byte a byte o de sempre,
    /// e é isso que torna esta porta segura de adoptar num sítio de cada vez.
    ///
    /// ⚠️ **Existe porque um corpo a dobrar-se é pintado inteiro e mostrado em parte.** Sem ele,
    /// as rows que o recorte da CENA esconde continuavam registadas na posição natural — e como
    /// a secção seguinte já subiu por cima delas, uma row invisível continuava a responder ao
    /// rato nos vãos entre os widgets de baixo. O recorte da cena e o do hit têm de dizer a
    /// mesma coisa, senão *o que se vê* e *o que se clica* discordam durante a transição.
    pub fn push_clip(&mut self, rect: Rect) {
        let clipped = self
            .clipped(rect)
            .unwrap_or(Rect::new(rect.x, rect.y, 0.0, 0.0));
        self.clips.push(clipped);
    }

    /// Fecha o recorte mais recente. Sem par aberto é um no-op.
    pub fn pop_clip(&mut self) {
        self.clips.pop();
    }

    /// A parte de `rect` que sobrevive ao recorte vigente. `None` quando nada sobrevive.
    ///
    /// ⚠️ **Uma intersecção VAZIA devolve `None`, e não um rectângulo de área zero:** um rect
    /// degenerado ainda passa no `contains` de quem está exactamente na sua borda, o que é a
    /// diferença entre *não clicável* e *clicável numa linha de um pixel*.
    fn clipped(&self, rect: Rect) -> Option<Rect> {
        let Some(clip) = self.clips.last() else {
            return Some(rect);
        };
        let x0 = rect.x.max(clip.x);
        let y0 = rect.y.max(clip.y);
        let x1 = (rect.x + rect.w).min(clip.x + clip.w);
        let y1 = (rect.y + rect.h).min(clip.y + clip.h);
        (x1 > x0 && y1 > y0).then(|| Rect::new(x0, y0, x1 - x0, y1 - y0))
    }

    /// Find the topmost widget under the cursor. Returns `None` if
    /// the cursor is outside every registered rect.
    pub fn hit(&self, x: f32, y: f32) -> Option<NodeId> {
        for (id, rect) in self.rects.iter().rev() {
            if rect.contains(x, y) {
                return Some(*id);
            }
        }
        None
    }

    /// Like [`Self::hit`] but returns the rect alongside the id —
    /// drag dispatchers (Slider) need the geometry to compute new
    /// values from pointer position.
    pub fn hit_with_rect(&self, x: f32, y: f32) -> Option<(NodeId, Rect)> {
        for (id, rect) in self.rects.iter().rev() {
            if rect.contains(x, y) {
                return Some((*id, *rect));
            }
        }
        None
    }

    /// Like [`Self::hit_with_rect`] but only counting rects whose id passes `keep`. Used while a
    /// full-screen modal (the command palette) is open: the docked panels register their full-canvas
    /// graph/timeline/flip **surface** hit AFTER the hero chrome (last-wins), so it would otherwise
    /// shadow every modal widget — resolving while skipping those surfaces lets the modal's pills /
    /// close-X / scrim (registered under them) win the click.
    pub fn hit_with_rect_where(
        &self,
        x: f32,
        y: f32,
        keep: impl Fn(NodeId) -> bool,
    ) -> Option<(NodeId, Rect)> {
        for (id, rect) in self.rects.iter().rev() {
            if rect.contains(x, y) && keep(*id) {
                return Some((*id, *rect));
            }
        }
        None
    }

    /// Look up the most recent registered rect for `id`, if any.
    /// Used to refresh `WidgetStore::active_rect` when geometry
    /// changes between Down and Up.
    pub fn rect_for(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find_map(|(rid, r)| if *rid == id { Some(*r) } else { None })
    }

    /// Number of registered rects this frame.
    pub fn len(&self) -> usize {
        self.rects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }

    /// Inline capacity — used by HR-3 tests to assert no heap spill.
    pub fn inline_capacity(&self) -> usize {
        INLINE_CAPACITY
    }

    /// True iff the rect storage is still inline (no heap spill).
    pub fn is_inline(&self) -> bool {
        !self.rects.spilled()
    }

    /// Iterate every (id, rect) registered this frame in
    /// registration order. Used by the hierarchy drag-and-drop
    /// dispatcher to find a drop target by scanning all rows
    /// without rebuilding state from the store.
    pub fn iter_registrations(&self) -> impl Iterator<Item = (NodeId, Rect)> + '_ {
        self.rects.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(x, y, w, h)
    }

    #[test]
    fn empty_index_no_hit() {
        let idx = HitIndex::new();
        assert!(idx.hit(50.0, 50.0).is_none());
    }

    #[test]
    fn hit_returns_topmost_overlap() {
        let mut idx = HitIndex::new();
        idx.register(NodeId(1), r(0.0, 0.0, 100.0, 100.0));
        idx.register(NodeId(2), r(20.0, 20.0, 60.0, 60.0));
        // (50, 50) sits inside both — newer registration wins.
        assert_eq!(idx.hit(50.0, 50.0), Some(NodeId(2)));
        // (10, 10) only inside the first.
        assert_eq!(idx.hit(10.0, 10.0), Some(NodeId(1)));
    }

    #[test]
    fn hit_outside_all_rects_returns_none() {
        let mut idx = HitIndex::new();
        idx.register(NodeId(1), r(0.0, 0.0, 50.0, 50.0));
        assert!(idx.hit(100.0, 100.0).is_none());
    }

    #[test]
    fn clear_for_frame_keeps_capacity() {
        let mut idx = HitIndex::new();
        for i in 0..30 {
            idx.register(NodeId(i as u64), r(i as f32 * 10.0, 0.0, 5.0, 5.0));
        }
        let cap_before = idx.rects.capacity();
        idx.clear_for_frame();
        assert_eq!(idx.len(), 0);
        assert!(idx.rects.capacity() >= cap_before);
    }

    #[test]
    fn stays_inline_under_capacity() {
        let mut idx = HitIndex::new();
        for i in 0..INLINE_CAPACITY {
            idx.register(NodeId(i as u64), r(0.0, 0.0, 1.0, 1.0));
        }
        assert!(
            idx.is_inline(),
            "should not spill at exactly INLINE_CAPACITY"
        );
    }

    #[test]
    fn spills_past_inline_capacity() {
        let mut idx = HitIndex::new();
        for i in 0..(INLINE_CAPACITY + 1) {
            idx.register(NodeId(i as u64), r(0.0, 0.0, 1.0, 1.0));
        }
        assert!(!idx.is_inline());
    }
}
