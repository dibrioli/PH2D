//! **A seção FRAME** do painel — irmã de [`super::paint_symmetry`] pelo teto de 600 LOC, e o corte
//! é o mesmo: aqui mora a seção de UM assunto.
//!
//! # O que ela NÃO tem, e é a decisão inteira
//!
//! ⚠️ **Não há W/H aqui.** A seção **Transform** já os mostra, para qualquer seleção, e uma
//! moldura é uma forma como as outras. Um segundo par de campos seria a falha de duas-portas que a
//! própria moldura evita no modelo (o componente não guarda tamanho: o tamanho é o da forma) — e
//! na tela ela é pior, porque os dois pares estariam visíveis ao mesmo tempo.
//!
//! Os **presets de dispositivo** escrevem naqueles campos: um preset é uma segunda forma de PEDIR
//! a edição de W/H, e a shell os aplica pela mesmíssima porta (`apply_vec_transform`). É a lição
//! da W1 — *o arrasto é uma 2ª forma de pedir, não um 2º caminho de fazer*.
//!
//! # A seção só existe com uma moldura selecionada
//!
//! `state::frame_clip()` é `None` quando a seleção não é moldura, e então nada é pintado. Uma
//! seção *Frame* sempre visível seria dois controles mortos em toda seleção que não é moldura.

use ph2d_i18n::tr;
use ph2d_tool_vector::frames::DEVICE_PRESETS;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state;

impl BodyCtx<'_> {
    /// **A seção FRAME** — o contêiner: recorta, e os atalhos de tamanho.
    pub(crate) fn frame_section(&mut self, y: f32) -> f32 {
        let Some(clip) = state::frame_clip() else {
            return y;
        };
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_FRAME,
            tr("panel.vector.section.frame"),
            y,
        );
        if collapsed {
            return y;
        }
        y = self.segmented(
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
        );
        // **O painel AUTORADO** (plano UI/UX W8b.2) — *que painel esta moldura descreve?*
        //
        // ⚠️ Oferecido para TODA moldura, inclusive a que ainda não tem um filho vestido: uma
        // moldura vazia descreve um painel vazio, e o gerador o emite (a lei da *face vazia* que a
        // seção de física e a de estados já seguem). Recusar aqui faria a feature ser descoberta
        // só por quem já a conhece.
        let panel_open = state::frame_panel_open();
        y = self.segmented(
            tr("panel.vector.frame.panel"),
            &[
                (
                    ids::VECTOR_FRAME_PANEL_OFF,
                    tr("panel.vector.frame.panel.off"),
                    !panel_open,
                ),
                (
                    ids::VECTOR_FRAME_PANEL_ON,
                    tr("panel.vector.frame.panel.on"),
                    panel_open,
                ),
            ],
            y,
        );
        // ⚠️ A fileira é construída a partir da TABELA (`DEVICE_PRESETS`), não de uma lista
        // paralela: um aparelho novo entra lá e ganha o botão de graça, e o clique já resolve
        // pela mesma tabela. Duas listas divergiriam num botão que pinta e não faz nada.
        //
        // Nenhum deles fica "aceso": um preset não é um MODO, é um gesto — ele escreve dois
        // números e acaba. Acender o que casa com o tamanho atual afirmaria que a moldura *é* um
        // telefone, quando o artista pode ter mudado um pixel depois.
        let cols = 2usize;
        self.button_grid(y, cols, DEVICE_PRESETS.len(), |i| {
            let p = DEVICE_PRESETS[i];
            (p.id, p.label, false)
        })
    }
}
