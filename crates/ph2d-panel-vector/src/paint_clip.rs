//! **A seção CLIP** do painel — o recorte, e nada mais.
//!
//! Irmã de [`super::paint_frame`], e a separação é o pedido do Enio de 2026-08-21 (*"coloque a
//! feature Clip Content para qualquer forma vetorial fechada"*) encontrando o que a seção Frame
//! carrega: além do recorte, ela traz o *Show as Panel* e os quatro presets de dispositivo, que
//! são perguntas sobre uma MOLDURA. Pintar o chip lá dentro teria dado a escolha entre oferecer
//! três controles mortos sobre uma estrela, ou não oferecer o recorte a ela.
//!
//! # A seção só existe onde o recorte pode ser expresso
//!
//! `state::frame_clip()` é `None` quando a seleção não oferece o controlo — sem forma fechada que
//! contenha o que está selecionado. A regra inteira (por que FECHADA, e por que *"a que contém a
//! seleção"*) mora do lado da shell, em `vec_clip_edit`, que é quem alcança o mundo.
//!
//! ⚠️ **Uma moldura recebe as DUAS seções**, e é o que se quer: ela é uma forma fechada como as
//! outras, e o que a distingue continua junto na seção Frame.

use ph2d_i18n::tr;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state;

impl BodyCtx<'_> {
    /// **A seção CLIP** — *esta forma esconde o que sai dela?*
    pub(crate) fn clip_section(&mut self, y: f32) -> f32 {
        let Some(clip) = state::frame_clip() else {
            return y;
        };
        let (y, collapsed) =
            self.section_header(ids::VECTOR_SECTION_CLIP, tr("panel.vector.section.clip"), y);
        if collapsed {
            return y;
        }
        self.segmented(
            tr("panel.vector.frame.clip"),
            &[
                (
                    ids::VECTOR_FRAME_CLIP_OFF,
                    tr("panel.vector.frame.clip.off"),
                    !clip,
                ),
                (
                    ids::VECTOR_FRAME_CLIP_ON,
                    tr("panel.vector.frame.clip.on"),
                    clip,
                ),
            ],
            y,
        )
    }
}
