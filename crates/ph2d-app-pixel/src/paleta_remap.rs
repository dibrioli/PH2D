//! **Reordenar sem remapear** (o Trial, `docs/PixelArt/ferramentas/oraculo_paleta.md`): em Indexed
//! arrastar, ordenar, mover e inverter mudam só a paleta — os pixels guardam o índice — e o
//! *Remap Palette* pendente faz os pixels seguirem as cores num passo seu. Em RGBA só a paleta muda.

use ph2d_pixel_core::palette;
use ph2d_pixel_core::{ColorMode, Op};

use crate::PixelApp;

impl PixelApp {
    /// A selecção (2+ amostras: da primeira à última) ou a paleta inteira.
    pub(super) fn trecho(&self, key: u32) -> std::ops::Range<usize> {
        match (self.sel.first(), self.sel.last()) {
            (Some(&a), Some(&b)) if self.sel.len() >= 2 => a..b + 1,
            _ => 0..self.docs[&key].sprite.palette().len(),
        }
    }

    /// Reordena a paleta por `perm` (`perm[novo] = antigo`) como UM passo.
    pub(super) fn reordena(&mut self, key: u32, perm: Vec<usize>, nome: &'static str) -> bool {
        let s = &self.docs[&key].sprite;
        if s.mode() != ColorMode::Indexed {
            return self.passo_nomeado(key, Some(Op::Remap { perm }), nome);
        }
        let colors = perm.iter().map(|&o| s.palette()[o]).collect();
        let transparent = s.transparent_index();
        let onde = palette::invert_permutation(&perm);
        let op = Op::Palette {
            colors,
            transparent,
        };
        let ok = self.passo_nomeado(key, Some(op), nome);
        if ok {
            self.pendente(key, &onde);
        }
        ok
    }

    /// Junta `onde` (antigo → novo) ao remapeamento pendente de `key`.
    fn pendente(&mut self, key: u32, onde: &[usize]) {
        let antes = match self.remap.take() {
            Some((k, o)) if k == key => o,
            _ => (0..self.docs[&key].sprite.palette().len().min(onde.len())).collect(),
        };
        let total: Option<Vec<usize>> = antes.iter().map(|&j| onde.get(j).copied()).collect();
        self.remap = total.map(|t| (key, t));
    }

    /// *Ctrl+arrastar*: cópias da selecção (ou só de `from`) inseridas na célula `to`.
    pub(super) fn copia(&mut self, key: u32, from: usize, to: usize) {
        let bloco: Vec<usize> = if self.sel.contains(&from) {
            self.sel.iter().copied().collect()
        } else {
            vec![from]
        };
        let s = &self.docs[&key].sprite;
        let Some((colors, onde)) = palette::copy_insert(s.palette(), &bloco, to) else {
            return;
        };
        let indexed = s.mode() == ColorMode::Indexed;
        let transparent = s.transparent_index();
        let op = Op::Palette {
            colors,
            transparent,
        };
        if !self.passo_nomeado(key, Some(op), "pixel.history.copy_colors") {
            return;
        }
        if indexed {
            self.pendente(key, &onde);
        }
        self.sel = (to..to + bloco.len()).collect();
        self.ancora = Some(to);
        self.ultima = Some(to);
    }

    /// *Gradient* / *Gradient by Hue*: da primeira à última amostra da selecção (2+).
    pub(super) fn gradiente(&mut self, key: u32, hue: bool) {
        let r = self.trecho(key);
        let s = &self.docs[&key].sprite;
        if self.sel.len() < 2 || r.len() < 3 {
            return;
        }
        let (a, b) = (s.palette()[r.start], s.palette()[r.end - 1]);
        let rampa = if hue {
            palette::gradient_hue(a, b, r.len())
        } else {
            palette::gradient(a, b, r.len())
        };
        let mut colors = s.palette().to_vec();
        colors.splice(r, rampa);
        let transparent = s.transparent_index();
        let op = Op::Palette {
            colors,
            transparent,
        };
        self.passo_nomeado(key, Some(op), "pixel.history.palette_gradient");
    }

    /// *Remap Palette*: os pixels seguem as cores. Monta a paleta de antes (`P'`, do tamanho da de
    /// agora) e a permutação que a leva à de agora: `Palette{P'}` + `Remap{perm}` num passo, cada
    /// um com inverso exacto. O transparente fica no índice que tem.
    pub(super) fn remapeia(&mut self, key: u32) {
        let Some((_, onde)) = self.remap.take().filter(|r| r.0 == key) else {
            return;
        };
        let s = &self.docs[&key].sprite;
        let q = s.palette();
        let m = q.len();
        let mut perm = vec![usize::MAX; m];
        for (i, &j) in onde.iter().enumerate() {
            match perm.get_mut(j) {
                Some(p) if *p == usize::MAX => *p = i,
                _ => return,
            }
        }
        let mut extra = onde.len();
        let mut antes = vec![[0; 4]; m];
        for (novo, p) in perm.iter_mut().enumerate() {
            if *p == usize::MAX {
                *p = extra;
                extra += 1;
            }
            antes[*p] = q[novo];
        }
        let Ok(transparent) = u8::try_from(perm[usize::from(s.transparent_index())]) else {
            return;
        };
        let op = Op::Batch(vec![
            Op::Palette {
                colors: antes,
                transparent,
            },
            Op::Remap { perm },
        ]);
        self.passo_nomeado(key, Some(op), "pixel.history.remap_palette");
    }
}
