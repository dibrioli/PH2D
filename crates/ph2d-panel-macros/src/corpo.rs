//! **O corpo do painel**: Gravar ▸ Parar, o resumo e o erro da macro do objecto escolhido, a lista
//! de etapas e as propriedades da escolhida.

use crate::ids;
use crate::nomes;
use crate::state::{Escolhida, MacrosPanelState};
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::macro_edits::{
    EtapaView, FamiliaDaMacro, MacroObjetoView, MacroPanelInfo, PropsDaEtapa,
};
use ph2d_editor_core::paint::{paint_text, paint_text_block, resolve};
use ph2d_editor_core::panel::rows::{RowCtx, value_col};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ColorSwatch, RowHighlight, TextInput, TextInputState, paint_button,
    paint_color_swatch, paint_row_highlight, paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

pub(crate) struct Corpo<'a> {
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
    pub store: &'a WidgetStore,
    pub hit_index: &'a mut HitIndex,
    pub theme: Theme,
    pub x: f32,
    pub w: f32,
}

impl Corpo<'_> {
    pub(crate) fn tudo(&mut self, info: &MacroPanelInfo, st: &MacrosPanelState, y: f32) -> f32 {
        let mut y = self.gravar(info, y);
        let Some(o) = &info.objeto else {
            return self.bloco(tr("panel.macros.empty"), ColorToken::Text3, y);
        };
        y = self.linha(
            &tr_with(
                "panel.macros.summary",
                &[
                    ("steps", &o.etapas.len()),
                    ("seconds", &nomes::segundos(o.duracao_s)),
                ],
            ),
            ColorToken::Text2,
            y,
        );
        if let Some(n) = o.editada_desde {
            y = self.linha(
                &tr_with("panel.macros.edited_since", &[("n", &(n + 1))]),
                ColorToken::Text3,
                y,
            );
        }
        if let Some(e) = &o.erro {
            let frase = match &e.detalhe {
                Some(d) => tr_with(e.chave, &[("detail", d)]),
                None => tr(e.chave).to_owned(),
            };
            y = self.bloco(&frase, ColorToken::Danger, y);
        }
        for (i, e) in o.etapas.iter().enumerate() {
            let esta = st.escolhida.is_some_and(|s| {
                s.entity_bits == o.entity_bits && s.trilha == e.trilha && s.k == e.k
            });
            y = self.etapa(i, e, esta, esta && st.renomeando, y);
        }
        if let Some(e) = st.escolhida.and_then(|s| escolhida(o, s)) {
            y = self.propriedades(e, y + Spacing::Sm.px());
        }
        y
    }

    /// Gravar ▸ Parar. O botão Gravar só se pinta quando há quem grave; senão a frase diz porquê.
    fn gravar(&mut self, info: &MacroPanelInfo, y: f32) -> f32 {
        let (id, rotulo, kind) = match (info.gravando, info.pode_gravar) {
            (Some(_), _) => (
                ids::MACROS_STOP,
                tr("panel.macros.stop"),
                ButtonKind::Danger,
            ),
            (None, Some(_)) => (
                ids::MACROS_RECORD,
                tr("panel.macros.record"),
                ButtonKind::Accent,
            ),
            (None, None) => {
                let k = info
                    .recusa
                    .unwrap_or("panel.macros.refusal.nothing_to_record");
                return self.bloco(tr(k), ColorToken::Text3, y);
            }
        };
        let bw = (self.text_system.prefix_width(rotulo, TypeToken::Sm.px())
            + Spacing::Lg.px() * 2.0)
            .min(self.w);
        let rect = Rect::new(self.x, y, bw, ROW_H_PX);
        let b = Button::new(id, rotulo)
            .kind(kind)
            .visual(self.store.button_visual(id));
        paint_button(&b, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        let y = y + ROW_H_PX + ph2d_tokens::control_gap_px();
        let frase = match (info.gravando, info.pode_gravar) {
            (Some(f), _) => tr_with("panel.macros.recording", &[("family", &nomes::familia(f))]),
            (None, Some(f)) => tr_with("panel.macros.ready", &[("family", &nomes::familia(f))]),
            (None, None) => String::new(),
        };
        let y = self.bloco(&frase, ColorToken::Text3, y);
        // A recusa da última tentativa (o Parar que não coube na animação) fica à vista.
        match info
            .recusa
            .filter(|_| info.gravando.is_some() || info.pode_gravar.is_some())
        {
            Some(k) => self.bloco(tr(k), ColorToken::Danger, y),
            None => y,
        }
    }

    fn linha(&mut self, texto: &str, cor: ColorToken, y: f32) -> f32 {
        paint_text(
            self.text_system,
            self.scene,
            texto,
            self.x + Spacing::Sm.px(),
            y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            (self.w - Spacing::Sm.px() * 2.0).max(1.0),
            resolve(cor, self.theme),
        );
        y + ROW_H_PX
    }

    /// Uma frase que QUEBRA (nunca cortada): a linha cresce pelo que ela transborda.
    fn bloco(&mut self, texto: &str, cor: ColorToken, y: f32) -> f32 {
        let topo = y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5;
        let alta = paint_text_block(
            self.text_system,
            self.scene,
            texto,
            self.x + Spacing::Sm.px(),
            topo,
            TypeToken::Sm.px(),
            (self.w - Spacing::Sm.px() * 2.0).max(1.0),
            resolve(cor, self.theme),
        );
        y + ROW_H_PX + (topo + alta - (y + ROW_H_PX)).max(0.0)
    }

    /// Uma linha da lista: `n. nome` à esquerda, a amostra da cor (Painter) e o fim em segundos à
    /// direita.
    fn etapa(&mut self, i: usize, e: &EtapaView, esta: bool, renomeando: bool, y: f32) -> f32 {
        let id = ids::row_id(e.trilha, e.k);
        let rect = Rect::new(self.x, y, self.w, ROW_H_PX);
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
        let texto_y = y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5;
        let tempo = nomes::segundos(e.t1_s);
        let tempo_w = self.text_system.prefix_width("0000.00", TypeToken::Sm.px());
        let mut direita = self.x + self.w - Spacing::Sm.px() - tempo_w;
        paint_text(
            self.text_system,
            self.scene,
            &tempo,
            direita,
            texto_y,
            TypeToken::Sm.px(),
            tempo_w,
            resolve(ColorToken::Text3, self.theme),
        );
        if let PropsDaEtapa::Pincelada { cor, .. } = e.props {
            let lado = TypeToken::Sm.px();
            direita -= lado + Spacing::Sm.px();
            let r = Rect::new(direita, y + (ROW_H_PX - lado) * 0.5, lado, lado);
            let cs = ColorSwatch::new(id, "", [cor[0], cor[1], cor[2], 255]);
            paint_color_swatch(&cs, r, self.scene, self.theme);
        }
        let nome_x = self.x + Spacing::Sm.px();
        let nome_w = (direita - Spacing::Sm.px() - nome_x).max(1.0);
        if renomeando {
            let r = Rect::new(nome_x, y, nome_w, ROW_H_PX);
            self.hit_index.register(ids::MACROS_RENAME_INPUT, r);
            let (st, text, caret, anchor) = match self.store.get(ids::MACROS_RENAME_INPUT) {
                Some(InteractiveState::TextInput {
                    state,
                    text,
                    caret,
                    selection_anchor,
                }) => (*state, text.clone(), *caret, *selection_anchor),
                _ => (TextInputState::Focused, String::new(), 0, None),
            };
            let input = TextInput::new(ids::MACROS_RENAME_INPUT, "")
                .visual((st, self.store.hover_live(ids::MACROS_RENAME_INPUT)));
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
        } else {
            let texto = format!("{}. {}", i + 1, nomes::nome(e));
            paint_text(
                self.text_system,
                self.scene,
                &texto,
                nome_x,
                texto_y,
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
        y + ROW_H_PX
    }

    /// As propriedades da escolhida: renomear, a espera e as da família.
    fn propriedades(&mut self, e: &EtapaView, y: f32) -> f32 {
        let gap = ph2d_tokens::control_gap_px();
        let mut r = RowCtx {
            scene: &mut *self.scene,
            text_system: &mut *self.text_system,
            store: self.store,
            hit_index: &mut *self.hit_index,
            theme: self.theme,
            inner_x: self.x,
            inner_w: self.w,
            row_h: ROW_H_PX,
            row_gap: gap,
            font: TypeToken::Base.px(),
            open_fold: None,
        };
        let mut y = r.action_button(ids::MACROS_RENAME, tr("panel.macros.rename"), y);
        y = r.labeled_number_field(tr("panel.macros.field.wait"), ids::MACROS_WAIT, 0.1, y);
        match e.props {
            PropsDaEtapa::Pincelada { cor, .. } => {
                y = cor_row(&mut r, cor, y);
                y = r.labeled_number_field(tr("panel.macros.field.size"), ids::MACROS_SIZE, 1.0, y);
            }
            PropsDaEtapa::Param { .. } if e.familia == FamiliaDaMacro::Motion => {
                y = r.labeled_number_field(
                    tr("panel.macros.field.value"),
                    ids::MACROS_VALUE,
                    0.1,
                    y,
                );
            }
            _ => {}
        }
        y
    }
}

/// A linha da cor: o rótulo e a amostra que abre o selector partilhado.
fn cor_row(r: &mut RowCtx<'_>, cor: [u8; 3], y: f32) -> f32 {
    paint_text(
        r.text_system,
        r.scene,
        tr("panel.macros.field.color"),
        r.inner_x,
        y + (r.row_h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        ph2d_editor_core::panel::rows::label_col_w(r.inner_x, r.inner_w),
        resolve(ColorToken::Text2, r.theme),
    );
    let rect = value_col(r.inner_x, r.inner_w, y, r.row_h);
    let rgba = r
        .store
        .widget_color(ids::MACROS_COLOR)
        .unwrap_or([cor[0], cor[1], cor[2], 255]);
    paint_color_swatch(
        &ColorSwatch::new(ids::MACROS_COLOR, "", rgba),
        rect,
        r.scene,
        r.theme,
    );
    r.hit_index.register(ids::MACROS_COLOR, rect);
    y + r.row_h + r.row_gap
}

fn escolhida(o: &MacroObjetoView, s: Escolhida) -> Option<&EtapaView> {
    (o.entity_bits == s.entity_bits)
        .then(|| o.etapas.iter().find(|e| e.trilha == s.trilha && e.k == s.k))
        .flatten()
}
