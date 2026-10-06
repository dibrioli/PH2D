//! **O PAPEL do documento** (pedido do dono, 2026-10-05: *«A cor do papel na watercolor precisa ser
//! revisto pois funciona mal. Deveria funcionar para todos os modos e deveria ter um botão para
//! aplicar no papel como um todo.»*; escolhas: a cor substitui o branco do papel, nos quatro meios,
//! ao vivo, invisível no painel de camadas).
//!
//! Medido antes (`cor_do_papel_tests::diag_a_cor_do_papel_em_cada_meio`): a cor só existia como o
//! chão óptico da aquarela — no Digital, no Impasto e no Wet Paint mudava `0` texels; na aquarela, `0`
//! sobre a tela branca opaca, e numa camada transparente tingia o PRÓPRIO TRAÇO (`1 211` texels) com
//! o papel em volta sem cor.
//!
//! O papel é uma propriedade do DOCUMENTO (`PainterTool::papel`): o composite de todas as portas é
//! *camadas SOBRE o papel*, antes da luz do relevo (o dente ilumina o papel), e o chão da aquarela é
//! ele. O botão **Aplicar no papel** separa o papel da tinta numa camada de fundo branca opaca: o
//! branco PURO da camada de baixo vira transparente (a tinta, opaca ou não, fica), e o papel passa a
//! ser a cor escolhida — um passo de desfazer. Daí em diante mexer na cor repinta o papel ao vivo
//! (uma rajada de mudanças = um passo de desfazer).
//!
//! ⚠️ **A orla anti-aliased de um traço pintado ANTES de aplicar** guarda o branco com que ela se
//! misturou (só o branco puro sai): sobre um papel de cor, um fio claro de 1 px em volta desses traços.
//! Os traços pintados depois caem sobre o papel transparente e não têm orla.

use super::PainterTool;
use crate::compositor::Region;
use std::sync::Arc;

/// **A lei única: um píxel RGBA8 (alfa direito) SOBRE o papel `p`** — opaco no fim. Inteira, com
/// arredondamento ao mais próximo, para ser a mesma em toda porta.
#[inline]
pub(crate) fn sobre_o_papel(px: &mut [u8; 4], p: [u8; 3]) {
    let a = u32::from(px[3]);
    if a == 255 {
        return;
    }
    for c in 0..3 {
        px[c] = ((u32::from(px[c]) * a + u32::from(p[c]) * (255 - a) + 127) / 255) as u8;
    }
    px[3] = 255;
}

impl PainterTool {
    /// O papel do documento ligado (`None` = sem papel: o composite é o de sempre, ao byte).
    #[must_use]
    pub fn papel(&self) -> Option<[u8; 3]> {
        self.papel
    }

    /// **Compõe a região `r` do composite `rgba` (do tamanho da região) sobre o papel** — a porta
    /// que toda saída do documento chama ENTRE o composite e a luz do relevo.
    pub(crate) fn compoe_sobre_o_papel(&self, rgba: &mut [u8], r: Region) {
        let Some(p) = self.papel else {
            return;
        };
        debug_assert_eq!(rgba.len(), (r.w as usize) * (r.h as usize) * 4);
        for px in rgba.as_chunks_mut::<4>().0 {
            sobre_o_papel(px, p);
        }
    }

    /// A cor do chão que a óptica da aquarela vê onde nada está pintado por baixo: o papel, ou o
    /// branco de sempre quando o documento não tem papel.
    #[must_use]
    pub(crate) fn cor_do_chao(&self) -> [u8; 3] {
        self.papel.unwrap_or([255, 255, 255])
    }

    /// **O botão Aplicar no papel** — o papel passa a ser a cor do seletor, e o branco PURO da camada
    /// raster de baixo vira transparente (onde o papel aparece). Um passo de desfazer. Com o papel já
    /// aplicado, só a cor muda.
    pub fn aplica_o_papel(&mut self) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let before = self.snapshot_model();
        self.papel = Some(self.paper_color_rgb8());
        let fundo = self.layers.z_order_bottom_up().into_iter().find(|&id| {
            matches!(
                self.layers.get(id).map(|l| &l.kind),
                Some(crate::layers::LayerKind::Raster(_))
            )
        });
        if let Some(fundo) = fundo {
            let tira = |px: &mut [u8; 4]| {
                if *px == [255, 255, 255, 255] {
                    *px = [0, 0, 0, 0];
                }
            };
            if self.layers.active() == Some(fundo) {
                let mut c = self.canvas_rgba.as_ref().clone();
                c.as_chunks_mut::<4>().0.iter_mut().for_each(tira);
                self.replace_canvas(Arc::new(c));
            } else if let Some(img) = self.images.get(&fundo) {
                let mut img = img.as_ref().clone();
                img.rgba8.as_chunks_mut::<4>().0.iter_mut().for_each(tira);
                self.images.insert(fundo, Arc::new(img));
            }
            self.bump_layer_pixels(Some(fundo));
        }
        self.edited_since_bind = true;
        self.commit_structural_edit(before);
        self.invalidate_composite();
    }

    /// A cor do papel mudou no seletor: com o papel aplicado ele muda AO VIVO — uma rajada de
    /// mudanças é um passo de desfazer ([`crate::undo::CoalesceKind::CorDoPapel`]).
    pub(crate) fn papel_segue_a_cor(&mut self) {
        let Some(antes) = self.papel else {
            return;
        };
        let nova = self.paper_color_rgb8();
        if nova == antes {
            return;
        }
        let before = self.snapshot_model();
        self.papel = Some(nova);
        self.edited_since_bind = true;
        self.commit_structural_edit_coalesced(crate::undo::CoalesceKind::CorDoPapel, before);
        self.invalidate_composite();
    }
}
