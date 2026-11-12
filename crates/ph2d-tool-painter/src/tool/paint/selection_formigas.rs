//! B20.11: o overlay da Seleção GUARDADO entre quadros. Refaz-se só quando muda o PASSO das formigas
//! (relógio de parede, [`PASSO_DAS_FORMIGAS_MS`]) ou aquilo de que o desenho depende (a máscara, os
//! contornos do Offset, a opacidade); nos outros quadros a cena leva o mesmo [`StableImage`], cujo id
//! o Vello já tem (zero subida). Análise: `docs/Painter/missao_4k/BACKLOG.md`.

use super::PainterTool;
use ph2d_vector::StableImage;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Weak};

/// O passo das formigas, em ms de relógio de parede: a velocidade não depende do monitor.
pub const PASSO_DAS_FORMIGAS_MS: u64 = 100;

/// De que o overlay depende. As máscaras entram por IDENTIDADE: um `Arc<Vec<u8>>` só muda de conteúdo
/// trocando de alocação, e o `Weak` segura o endereço (não os bytes) contra o reuso.
struct Chave {
    passo: u32,
    tamanho: (u32, u32),
    opacidade: u32,
    mascara: Weak<Vec<u8>>,
    niveis: Vec<Weak<Vec<u8>>>,
}

impl PartialEq for Chave {
    fn eq(&self, o: &Self) -> bool {
        self.passo == o.passo
            && self.tamanho == o.tamanho
            && self.opacidade == o.opacidade
            && self.mascara.ptr_eq(&o.mascara)
            && self.niveis.len() == o.niveis.len()
            && self.niveis.iter().zip(&o.niveis).all(|(a, b)| a.ptr_eq(b))
    }
}

thread_local! {
    /// Derivado e só do fio que desenha; a chave distingue telas e tools.
    static GUARDADO: RefCell<Option<(Chave, StableImage)>> = const { RefCell::new(None) };
    static CONSTRUCOES: Cell<u64> = const { Cell::new(0) };
}

/// Quantas vezes o overlay foi construído neste fio (gate do quadro parado).
#[doc(hidden)]
#[must_use]
pub fn construcoes_do_overlay_da_selecao() -> u64 {
    CONSTRUCOES.with(Cell::get)
}

impl PainterTool {
    fn chave_do_overlay(&self, passo: u32) -> Chave {
        let niveis = self
            .selection_offset_contour_levels()
            .iter()
            .filter_map(|&l| self.selection_offset_level_mask(l).map(Arc::downgrade))
            .collect();
        Chave {
            passo,
            tamanho: self.source_size,
            opacidade: self.paint.selection_overlay_opacity.to_bits(),
            mascara: Arc::downgrade(&self.paint.selection_mask),
            niveis,
        }
    }

    /// O overlay da Seleção no `passo` das formigas, guardado: o mesmo passo sobre a mesma seleção
    /// devolve a MESMA imagem. `None` sem seleção viva.
    #[must_use]
    pub fn selection_overlay_imagem(&self, passo: u32) -> Option<StableImage> {
        if !self.paint.selection_active {
            return None;
        }
        let chave = self.chave_do_overlay(passo);
        GUARDADO.with_borrow_mut(|g| {
            if g.as_ref().is_none_or(|(c, _)| *c != chave) {
                // Larga a imagem velha antes de alocar a nova (uma tela 4K são 33 MB).
                *g = None;
                let (rgba, w, h) = self.selection_overlay_rgba(passo)?;
                CONSTRUCOES.set(CONSTRUCOES.get() + 1);
                *g = Some((chave, StableImage::from_rgba(Arc::new(rgba), w, h)?));
            }
            g.as_ref().map(|(_, img)| img.clone())
        })
    }
}

#[cfg(test)]
#[path = "selection_formigas_tests.rs"]
mod tests;
