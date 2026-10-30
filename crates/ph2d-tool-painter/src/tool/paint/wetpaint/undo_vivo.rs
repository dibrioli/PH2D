//! ⭐ **O undo não mata a tinta viva** (smoke do dono 30/10) — filho de [`super`] por LOC.
//!
//! Antes, todo undo acabava a sessão (`restore_model_confined` → `wetpaint_end_session`): a tinta que ficava na tela
//! virava `base` seca de uma sessão nova, sem água, assentado nem óleo, e o traço seguinte já não a misturava, re-molhava
//! nem arrastava. Agora o pen-down de cada passo guarda as grades do motor pela profundidade do undo, e desfazer ESSE
//! passo devolve-as à sessão, que continua viva sobre a tela restaurada. Sem a fotografia (outro passo, o redo) a
//! sessão acaba como sempre.

use super::*;

/// A sessão tirada do caminho do restore, com as grades de antes do passo desfeito.
pub(crate) struct SessaoNoUndo(WetSession, Vec<ph2d_wet_paint::grid::GridSnapshot>);

impl PainterTool {
    /// Pen-down de um passo novo com a sessão viva: fotografa as grades à profundidade de agora.
    pub(crate) fn wet_guarda_o_passo(&mut self) {
        let d = self.undo.undo_depth();
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        sess.bring_home();
        let fotos = sess
            .engine
            .layers
            .iter()
            .map(|l| ph2d_wet_paint::grid::snapshot_grid(&l.grid))
            .collect();
        sess.antes_do_passo.retain(|(k, _)| *k < d);
        sess.antes_do_passo.push((d, fotos));
    }

    /// Antes de um undo: se a sessão tem as grades de antes do passo do topo, sai do caminho do restore com elas.
    pub(crate) fn wet_solta_para_o_undo(&mut self) -> Option<SessaoNoUndo> {
        let d = self.undo.undo_depth().checked_sub(1)?;
        self.wetpaint_guard();
        let sess = self.paint.wetpaint.session.as_mut()?;
        let i = sess.antes_do_passo.iter().position(|(k, _)| *k == d)?;
        let (_, fotos) = sess.antes_do_passo.remove(i);
        sess.antes_do_passo.retain(|(k, _)| *k < d);
        let sess = self.paint.wetpaint.session.take()?;
        Some(SessaoNoUndo(sess, fotos))
    }

    /// Depois do undo: a sessão volta com as grades de antes do passo, presa à tela restaurada.
    pub(crate) fn wet_volta_do_undo(&mut self, solta: SessaoNoUndo, desfez: bool) {
        let SessaoNoUndo(mut sess, fotos) = solta;
        if desfez {
            sess.bring_home();
            for (l, f) in sess.engine.layers.iter_mut().zip(&fotos) {
                ph2d_wet_paint::grid::restore_grid(&mut l.grid, f);
            }
            sess.engine.mark_dirty_full();
            sess.canvas = Arc::downgrade(&self.canvas_rgba);
            sess.stroke_open = false;
            sess.lanes.clear();
            sess.adiados.clear();
            sess.preview = None;
        }
        // Uma sessão que o restore fez nascer (não devia: o restore acaba-as) perde para a que estava viva.
        self.paint.wetpaint.session = Some(sess);
    }
}
