//! O lado `before` da TELA e do VIDRO pelos journals do passo — filho de `undo_planes.rs` (`#[path]`) pelo tecto de LOC.

use super::*;

impl PlaneDeltas {
    /// **A tela de um `before` ELIDIDO** (B17.4): o lado de antes é o journal da tela, válido quando ele descreve
    /// ESTE passo e a testemunha é o plano do `after` (o vivo). Senão `(_, true)`: o antes não existe em lugar
    /// nenhum, e quem chama descarta a história (o molde do relevo, `discard_if_relief_is_lost`).
    pub(super) fn canvas_from_journal(
        before: &mut crate::undo::ModelSnapshot,
        after: &mut crate::undo::ModelSnapshot,
        (stride, win): (usize, Option<crate::undo_delta::PlaneWindow>),
        tiles: Option<(&[crate::compositor::Region], u32)>,
        src: Option<&ReliefSource<'_>>,
    ) -> (StoredPlane<u8>, bool) {
        let ok = src.is_some_and(|src| {
            src.state.journal_describes_step_at(src.writes) && !src.state.on_foreign_plane()
        }) && (src.is_some_and(|s| !s.state.canvas_untouched())
            || before
                .canvas_elided
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .is_some_and(|p| std::sync::Arc::ptr_eq(&p, &after.canvas_rgba)));
        let built = ok
            .then(|| {
                src.and_then(|src| {
                    let j = src.state.canvas_journal();
                    // Captura INTEIRA (quem não sabia onde ia escrever): o antes já está completo, e a janela
                    // declarada + os tiles marcados guardam só o que mudou, como o `split` de um antes segurado.
                    if let Some(w) = j.whole().filter(|w| w.len() == after.canvas_rgba.len()) {
                        let mut b = std::sync::Arc::clone(w);
                        let mut a = std::sync::Arc::clone(&after.canvas_rgba);
                        return Some(StoredPlane::split_tiled(&mut b, &mut a, stride, win, tiles));
                    }
                    StoredPlane::from_journal(&after.canvas_rgba, &j, win)
                })
            })
            .flatten();
        before.canvas_elided = None;
        before.canvas_rgba = std::sync::Arc::new(Vec::new());
        after.canvas_rgba = std::sync::Arc::new(Vec::new());
        match built {
            Some(p) => (p, false),
            None => (StoredPlane::Unchanged, true),
        }
    }

    /// **O delta do vidro** (ADR-0205 D4): com o `before` a DESCREVER o vidro, pelo journal por tile
    /// ([`StoredMap::from_journal`]); a segurá-lo (a chave desligada, um snapshot estrutural), pelo `split`.
    /// Elidido e o journal a recusar = indescritível: o `split` leria toda chave como `OnlyAfter` e o undo
    /// APAGARIA o vidro (o mesmo braço do relevo, abaixo).
    pub(super) fn vidro_map(
        before: &mut crate::undo::ModelSnapshot,
        after: &mut crate::undo::ModelSnapshot,
        stride: usize,
        win: Option<crate::undo_delta::PlaneWindow>,
        relief: Option<&ReliefSource<'_>>,
    ) -> (StoredMap<crate::compositor::vidro::Vidro>, bool) {
        let el = std::mem::take(&mut before.relief_elided.vidros);
        if el.is_empty() {
            return (
                StoredMap::split(&mut before.vidros, &mut after.vidros, stride, win),
                false,
            );
        }
        let built = relief
            .and_then(|src| {
                src.state
                    .with_vidro_before(src.writes, src.layer, |l, absent, j| {
                        StoredMap::from_journal(
                            &before.vidros,
                            &el,
                            &after.vidros,
                            l,
                            absent,
                            j,
                            win,
                        )
                    })
            })
            .flatten();
        before.vidros.clear();
        after.vidros.clear();
        match built {
            Some(m) => {
                #[cfg(test)]
                VIDRO_FROM_JOURNAL.with(|c| c.set(c.get() + 1));
                (m, false)
            }
            None => (StoredMap::default(), true),
        }
    }
}
