//! **Normal map sheet** (P14d): a `_n.png` tem o MESMO lugar de cada peça que a folha de cor; cada
//! peça corta-se de [`compose_normal`] do seu quadro pelo mesmo canto e tamanho. O lugar é o da cor
//! (o recorte e o merge leem a cor), para que um importador use um só JSON para as duas.

use std::collections::BTreeMap;

use ph2d_pixel_core::{FrameId, Sprite, compose_normal};

use super::{OpcoesFolha, Peca, carimba, celula_da_grelha};

/// O sprite tem alguma camada Normal Map? (a caixa Normal map sheet só se habilita assim).
#[must_use]
pub fn tem_normal(s: &Sprite) -> bool {
    s.layers().iter().any(|l| l.normal)
}

pub(super) struct Normais<'a> {
    s: &'a Sprite,
    rgba: Option<Vec<u8>>,
    por_quadro: BTreeMap<FrameId, Vec<u8>>,
}

impl<'a> Normais<'a> {
    pub(super) fn nova(s: &'a Sprite, o: &OpcoesFolha, len: usize) -> Self {
        Self {
            s,
            rgba: (o.normal && tem_normal(s)).then(|| vec![0; len]),
            por_quadro: BTreeMap::new(),
        }
    }

    pub(super) fn carimba(&mut self, fw: u32, l: [u32; 2], p: &Peca, o: &OpcoesFolha) {
        let Some(rgba) = self.rgba.as_mut() else {
            return;
        };
        let s = self.s;
        let n = self
            .por_quadro
            .entry(p.quadro)
            .or_insert_with(|| compose_normal(s, p.quadro));
        let px = celula_da_grelha(
            n,
            s.width(),
            s.height(),
            (p.de[0], p.de[1]),
            p.rect[2],
            p.rect[3],
        );
        let peca = Peca {
            nome: String::new(),
            grupo: p.grupo,
            rect: p.rect,
            px,
            duration_ms: p.duration_ms,
            dono: None,
            quadro: p.quadro,
            de: p.de,
            fatia: p.fatia,
        };
        carimba(rgba, fw, l, &peca, o);
    }

    pub(super) fn folha(self) -> Option<Vec<u8>> {
        self.rgba
    }
}
