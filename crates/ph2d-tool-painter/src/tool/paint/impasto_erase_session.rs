//! A **SESSÃO da borracha de impasto** — o `pre` da massa e a devolução que um re-carimbo faz.
//!
//! O irmão exato do [`super::sculpt_session`], e pelo mesmo motivo, escrito no
//! `stamp_preview::stamp_drag_preview` desde 2026-07-13: o depósito constrói o relevo num envelope
//! por-traço e só o funde no commit, então limpar o envelope basta — mas quem **REESCREVE o plano da
//! camada ao vivo** tem de restaurá-lo, senão *"uma Curve cavaria mais fundo a cada movimento do ponteiro
//! enquanto o artista apenas OLHAVA"*.
//!
//! A borracha reescreve o mesmo plano (não há envelope dela para descascar: ela esfrega o que já está
//! commitado) e **não tinha restauração nenhuma** — Enio, 2026-08-07: *"no modo eraser de impasto, as
//! shapes vivas do stroke estão marcando (reduzindo o relevo) da massa antes de apertar apply/enter"*.
//! Medido: a MESMA linha, pousada e depois arrastada por um desvio, deixava **1158 texels** com a massa
//! comida até o ZERO onde a linha desenhada direto a deixava intacta — a mordida de toda figura por onde
//! a mão passou, permanente antes do Apply.

use crate::tool::PainterTool;

impl PainterTool {
    /// Devolve a massa que a borracha mordeu desde o congelamento — a metade `erasing` do
    /// [`super::impasto::PainterTool::stamp_dabs_height`] vista pelo avesso.
    ///
    /// Chamada ao lado do [`Self::restamp_reset_sculpt`] nos MESMOS sítios: os dois canais reescrevem o
    /// plano da camada e por isso têm de ter exatamente o mesmo tempo de vida — uma terceira regra a
    /// manter em dia é como um deles nasce sem ela.
    ///
    /// A janela é a que a sessão acumulou, e ela é zerada aqui: depois da devolução não há mais mordida
    /// a desfazer, e o próximo carimbo constrói a dele do zero.
    pub(super) fn restamp_reset_erase(&mut self) {
        let Some(pre) = self.paint.relief.erase_pre.as_ref() else {
            return;
        };
        let Some(rect) = pre.bbox else {
            return; // nada mordido desde o último carimbo — e então nem um byte é lido
        };
        let layer = pre.layer;
        let heights = std::sync::Arc::clone(&pre.heights);
        let covers = std::sync::Arc::clone(&pre.covers);
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if heights.len() != n {
            // Uma sessão que não descreve mais esta tela não é uma sessão (o guard de forma que a
            // varredura do rebind ensinou a escrever).
            self.paint.relief.erase_pre = None;
            return;
        }
        if let Some(entry) = self.heights.get_mut(&layer)
            && entry.len() == n
        {
            let dst = super::plane_fork::fork_heights(
                entry,
                &self.undo.write_state,
                layer,
                (w, h),
                Some(rect),
            );
            super::impasto_settle::for_each_in(rect, w, |i| dst[i] = heights[i]);
        }
        if covers.len() == n
            && let Some(entry) = self.covers.get_mut(&layer)
            && entry.len() == n
        {
            let dst = super::plane_fork::fork_covers(
                entry,
                &self.undo.write_state,
                layer,
                (w, h),
                Some(rect),
            );
            super::impasto_settle::for_each_in(rect, w, |i| dst[i] = covers[i]);
        }
        if let Some(p) = self.paint.relief.erase_pre.as_mut() {
            p.bbox = None;
        }
        self.sync_relief_flags();
        // A massa voltou, logo a tela mudou. O `peel_drag_preview` acima devolveu os PIXELS e marcou o
        // retângulo deles, que não é este: a mordida da borracha alcança onde os dabs passaram, e a luz
        // lê o plano, não o pigmento.
        self.mark_dirty(rect);
    }

    /// A sessão morre quando o gesto morre — e daí em diante a mordida é permanente, como a tinta.
    ///
    /// Chamada do [`Self::commit_drag_preview`] e do `close_stroke`, os dois sítios onde o
    /// `end_sculpt_session` já mora: o segundo é no-op depois do primeiro, *uma morte, não duas*.
    /// Deixá-la viva seria a arma carregada que o `commit_drag_preview` descreve para o sculpt — o
    /// primeiro quadro de preview do gesto SEGUINTE devolveria massa que este gesto apagou de verdade.
    pub(super) fn drop_erase_session(&mut self) {
        self.paint.relief.erase_pre = None;
    }
}
