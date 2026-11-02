//! **O pincel de padrão do lado da família** (Ctrl+B, P7): a selecção do cel activo vira o pincel e
//! a selecção cai (no Trial ela deixa de recortar o traço); sem selecção, o recorte por arrasto
//! («Select brush bounds | Right-click to cut»). Leis em `docs/PixelArt/handoffs/HANDOFF_CONTINUACAO_line_PixelArt_2026-11-02_P7a.md` §3 e §6.

use std::sync::Arc;

use ph2d_pixel_core::pattern::PatternBrush;
use ph2d_pixel_core::raster::Cell;
use ph2d_pixel_core::selection::{self, Mask, SelectMode};

use crate::selecao::Laco;
use crate::{PixelApp, Tool};

impl PixelApp {
    /// **Edit ▸ New Brush** sobre o documento `key`.
    pub(crate) fn novo_pincel(&mut self, key: u32) {
        let Some(doc) = self.docs.get(&key) else {
            return;
        };
        let s = &doc.sprite;
        let Some(m) = s.mask().cloned() else {
            self.recorte = true;
            self.avisos
                .push((false, ph2d_i18n::tr("pixel.brush.select_bounds").to_owned()));
            return;
        };
        let Some(p) = PatternBrush::from_selection(s, (doc.layer, doc.frame), &m) else {
            return;
        };
        let op = selection::deselect(s);
        self.pega_pincel(p);
        self.passo(key, op);
    }

    /// O pincel `p` passa a ser o da mão, com as cores da imagem (decisão do dono 02/11, §6.1).
    pub(crate) fn pega_pincel(&mut self, p: PatternBrush) {
        self.pincel = Some(Arc::new(p));
        self.pincel_imagem = true;
    }

    /// **Escolher a cor de frente**: com o pincel de padrão, ele passa a pintar INTEIRO nessa cor
    /// (o Trial muda até com a MESMA cor, `rb2b_frente_paint_cores`; o pixel (0,0) que o Aseprite
    /// guarda é defeito dele e não se copia — dono 02/11).
    pub(crate) fn escolhe_fg(&mut self, c: [u8; 4]) {
        self.fg = c;
        self.pincel_imagem = false;
    }

    /// Largar o recorte: o rectângulo `a`–`b` vira o pincel; `corta` = também o apaga (UM passo).
    pub(crate) fn recorta_pincel(&mut self, key: u32, a: Cell, b: Cell, corta: bool) {
        self.recorte = false;
        let Some(doc) = self.docs.get(&key) else {
            return;
        };
        let s = &doc.sprite;
        let ck = (doc.layer, doc.frame);
        let m = Mask::marquee(s.width(), s.height(), a, b);
        let Some(p) = PatternBrush::from_selection(s, ck, &m) else {
            return;
        };
        let recusa = corta.then(|| crate::camadas::recusa_pintar(doc)).flatten();
        let op = (corta && recusa.is_none())
            .then(|| crate::edicao::limpa(s, ck, &m, self.fundo(s)))
            .flatten();
        self.pega_pincel(p);
        if let Some(motivo) = recusa {
            self.recusa(motivo);
        }
        self.passo(key, op);
    }
}

/// Premir com o recorte pendente: começa o rectângulo (o direito corta).
pub(crate) fn press(app: &mut PixelApp, key: u32, at: Cell, corta: bool) {
    app.laco = Some(Laco {
        key,
        tool: Tool::RectMarquee,
        mode: SelectMode::Replace,
        pts: vec![at, at],
        moved: false,
        held: true,
        pincel: Some(corta),
    });
}
