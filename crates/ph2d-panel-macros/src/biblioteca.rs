//! **A secção Biblioteca do painel MACROS** (W18, `docs/Macros/16_W18_plano.md` §2.6, §4): guardar
//! a macro do objecto mostrado (na biblioteca ou como asset), importar, a lista das macros guardadas
//! e os verbos da escolhida (criar nova, aplicar por cima, exportar, apagar).
//!
//! Cada botão pinta-se activo só quando [`pedido_da_biblioteca`] tem pedido para ele — a mesma
//! tabela que o despacho lê; apagado, não regista hit-rect (sem rect não há clique).

use crate::corpo::Corpo;
use crate::ids;
use crate::nomes;
use crate::seam::pedido_da_biblioteca;
use crate::state::MacrosPanelState;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::macro_edits::{
    ErroDaMacroView, FamiliaDaMacro, MacroDaBibliotecaView, MacroPanelInfo,
};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::property_row::{abaixo_do_botao, caixa_do_botao};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, RowHighlight, TextInput, TextInputState, paint_button,
    paint_row_highlight, paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

/// **A convenção da cor do aviso:** uma chave com o segmento `.error.` ou `.refusal.` é um erro
/// (cor Danger); qualquer outra é uma confirmação (guardou, exportou para…).
#[must_use]
pub(crate) fn aviso_e_erro(chave: &str) -> bool {
    chave.contains(".error.") || chave.contains(".refusal.")
}

/// O detalhe curto de uma macro: as famílias, a duração e o tamanho da tela que ela cria.
#[must_use]
pub(crate) fn detalhe(m: &MacroDaBibliotecaView) -> String {
    let mut fams: Vec<&str> = Vec::new();
    for f in &m.familias {
        let n = match f {
            FamiliaDaMacro::Painter => tr("panel.macros.lib.family.painter"),
            FamiliaDaMacro::Motion => tr("panel.macros.lib.family.motion"),
            FamiliaDaMacro::Mundo => tr("panel.macros.lib.family.world"),
        };
        if !fams.contains(&n) {
            fams.push(n);
        }
    }
    let families = fams.join(", ");
    let seconds = nomes::segundos(m.duracao_s);
    match m.tela {
        Some((w, h)) => tr_with(
            "panel.macros.lib.detail_canvas",
            &[
                ("families", &families),
                ("seconds", &seconds),
                ("w", &w),
                ("h", &h),
            ],
        ),
        None => tr_with(
            "panel.macros.lib.detail",
            &[("families", &families), ("seconds", &seconds)],
        ),
    }
}

impl Corpo<'_> {
    pub(crate) fn biblioteca(
        &mut self,
        info: &MacroPanelInfo,
        st: &MacrosPanelState,
        y: f32,
    ) -> f32 {
        let escolhida = st.lib_escolhida.as_deref();
        let mut y = self.linha(tr("panel.macros.lib.title"), ColorToken::Text1, y);
        for (id, k) in [
            (ids::MACROS_LIB_SAVE, "panel.macros.lib.save"),
            (ids::MACROS_LIB_SAVE_ASSET, "panel.macros.lib.save_asset"),
            (ids::MACROS_LIB_IMPORT, "panel.macros.lib.import"),
        ] {
            y = self.verbo(info, escolhida, id, tr(k), ButtonKind::Default, y);
        }
        if let Some(a) = &info.aviso {
            let cor = if aviso_e_erro(a.chave) {
                ColorToken::Danger
            } else {
                ColorToken::Text2
            };
            y = self.bloco(&frase(a), cor, y);
        }
        if info.biblioteca_ilegiveis > 0 {
            y = self.bloco(
                &tr_with(
                    "panel.macros.lib.unreadable",
                    &[("n", &info.biblioteca_ilegiveis)],
                ),
                ColorToken::Danger,
                y,
            );
        }
        if info.biblioteca.is_empty() {
            return self.bloco(tr("panel.macros.lib.empty"), ColorToken::Text3, y);
        }
        // Os itens de uma coisa só: o vão de LISTA entre eles; os verbos são controlos, a seguir.
        for (i, m) in info.biblioteca.iter().enumerate() {
            if i > 0 {
                y += ph2d_tokens::list_row_gap_px();
            }
            let esta = escolhida == Some(m.nome.as_str());
            y = self.entrada(m, esta, esta && st.lib_renomeando, y);
        }
        if escolhida.is_none() {
            return y;
        }
        y += ph2d_tokens::control_gap_px();
        y = self.verbo(
            info,
            escolhida,
            ids::MACROS_LIB_APPLY_NEW,
            tr("panel.macros.lib.apply_new"),
            ButtonKind::Accent,
            y,
        );
        match (&info.por_cima_de, info.por_cima_alvo) {
            (Some(n), Some(_)) => {
                let rotulo = tr_with("panel.macros.lib.apply_over", &[("name", n)]);
                y = self.verbo(
                    info,
                    escolhida,
                    ids::MACROS_LIB_APPLY_OVER,
                    &rotulo,
                    ButtonKind::Default,
                    y,
                );
            }
            _ => {
                let k = info
                    .por_cima_recusa
                    .unwrap_or("panel.macros.lib.apply_over_needs_target");
                y = self.bloco(tr(k), ColorToken::Text3, y);
            }
        }
        y = self.verbo(
            info,
            escolhida,
            ids::MACROS_LIB_EXPORT,
            tr("panel.macros.lib.export"),
            ButtonKind::Default,
            y,
        );
        self.verbo(
            info,
            escolhida,
            ids::MACROS_LIB_DELETE,
            tr("panel.macros.lib.delete"),
            ButtonKind::Danger,
            y,
        )
    }

    /// Um verbo da tabela: activo (com hit-rect) só se a tabela tem pedido para ele.
    fn verbo(
        &mut self,
        info: &MacroPanelInfo,
        escolhida: Option<&str>,
        id: ph2d_a11y::NodeId,
        rotulo: &str,
        kind: ButtonKind,
        y: f32,
    ) -> f32 {
        let activo = pedido_da_biblioteca(id, info, escolhida).is_some();
        let rect = if kind == ButtonKind::Accent {
            Rect::new(self.x, y, self.w, ROW_H_PX)
        } else {
            caixa_do_botao(self.text_system, self.x, self.w, y, rotulo)
        };
        let visual = if activo {
            self.store.button_visual(id)
        } else {
            (ButtonState::Disabled, 0.0)
        };
        let b = Button::new(id, rotulo).kind(kind).visual(visual);
        paint_button(&b, rect, self.scene, self.text_system, self.theme);
        if activo {
            self.hit_index.register(id, rect);
        }
        abaixo_do_botao(rect)
    }

    /// Uma macro da lista: o nome (ou o campo de renomear) e, por baixo, o detalhe.
    fn entrada(&mut self, m: &MacroDaBibliotecaView, esta: bool, renomeando: bool, y: f32) -> f32 {
        let id = ids::lib_row_id(&m.nome);
        let rect = Rect::new(self.x, y, self.w, ROW_H_PX * 2.0);
        paint_row_highlight(
            self.scene,
            rect,
            self.theme,
            if esta {
                RowHighlight::Selected
            } else if self.store.hover_live(id) > 0.0 {
                RowHighlight::Hovered
            } else {
                RowHighlight::None
            },
            0.0,
        );
        self.hit_index.register(id, rect);
        let nome_x = self.x + Spacing::Sm.px();
        let nome_w = (self.w - Spacing::Sm.px() * 2.0).max(1.0);
        if renomeando {
            self.campo_de_texto(
                ids::MACROS_LIB_RENAME_INPUT,
                Rect::new(nome_x, y, nome_w, ROW_H_PX),
            );
        } else {
            paint_text(
                self.text_system,
                self.scene,
                &m.nome,
                nome_x,
                y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
                TypeToken::Sm.px(),
                nome_w,
                resolve(
                    if esta {
                        ColorToken::Text1
                    } else {
                        ColorToken::Text2
                    },
                    self.theme,
                ),
            );
        }
        paint_text(
            self.text_system,
            self.scene,
            &detalhe(m),
            nome_x,
            y + ROW_H_PX + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            nome_w,
            resolve(ColorToken::Text3, self.theme),
        );
        y + ROW_H_PX * 2.0
    }

    /// Um campo de texto aberto (renomear uma etapa ou uma macro), com o hit-rect dele.
    pub(crate) fn campo_de_texto(&mut self, id: ph2d_a11y::NodeId, r: Rect) {
        self.hit_index.register(id, r);
        let (st, text, caret, anchor) = match self.store.get(id) {
            Some(InteractiveState::TextInput {
                state,
                text,
                caret,
                selection_anchor,
            }) => (*state, text.clone(), *caret, *selection_anchor),
            _ => (TextInputState::Focused, String::new(), 0, None),
        };
        let input = TextInput::new(id, "").visual((st, self.store.hover_live(id)));
        paint_text_input_with_buffer(
            &input,
            Some(text.as_str()),
            Some(caret),
            anchor,
            r,
            self.scene,
            self.text_system,
            self.theme,
        );
    }
}

/// A frase de um aviso: a chave traduzida, com o detalhe que ela cita.
fn frase(a: &ErroDaMacroView) -> String {
    match &a.detalhe {
        Some(d) => tr_with(a.chave, &[("detail", d)]),
        None => tr(a.chave).to_owned(),
    }
}
