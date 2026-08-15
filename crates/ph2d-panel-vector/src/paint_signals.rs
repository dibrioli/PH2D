//! ⭐ **A TABELA SINAL → PAPEL** (item 4 do estudo dos contêineres) — irmã de
//! [`super::paint_states`] pelo mesmo corte de assunto: ali mora *que poses esta forma tem*, aqui
//! *o que a faz mudar de pose*.
//!
//! # Por que ela é autorada do lado do HOSPEDEIRO
//!
//! A tabela é por NOME (o contrato do ADR-0143: quem escuta casa numa string e nunca pergunta a
//! origem), mas a **autoria** é do lado de quem RESPONDE: o artista seleciona o menu e diz a que
//! nome ele abre. A alternativa — uma tabela global `nome → alvos` — obrigá-lo-ia a procurar um
//! objeto numa lista em vez de o ter selecionado, e teria de ganhar a própria varredura de
//! hospedeiros mortos. Ligada ao hospedeiro, uma forma apagada leva as ligações dela pelo
//! `retain_hosts` que já corre por frame.
//!
//! # Duas linhas por ligação, e a segunda é a que evita a mentira
//!
//! O **nome** ocupa uma linha inteira porque é texto livre e um campo estreito esconderia o que
//! está escrito; os **quatro papéis** ocupam a seguinte porque são um catálogo fixo e uma
//! segmentada diz de relance qual está aceso. Espremer os dois numa linha só daria um campo de
//! texto de meia dúzia de caracteres — que é como o artista aprende a não digitar nomes longos.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{
    Button, ButtonKind, TextInput, TextInputState, paint_button, paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;

use crate::ids;
use crate::paint_sections::BodyCtx;

/// A largura do botão de apagar, ao fim da linha do nome.
const REMOVE_W: f32 = 28.0; // LITERAL-PX-OK: um botão de ícone quadrado na altura da row

impl BodyCtx<'_> {
    /// **As ligações deste hospedeiro**, mais o botão que acrescenta uma.
    ///
    /// ⚠️ **Ela é pintada DEPOIS da autoria de poses e nunca durante a preview** — quem a chama
    /// já passou pelo `return` que fecha a autoria com o modo ligado, e pela mesma razão: uma
    /// ligação criada dentro da preview perderia o passo de undo, que ali está suprimido.
    pub(crate) fn signal_rows(
        &mut self,
        bindings: &[(String, usize)],
        role_labels: &[String; 4],
        y: f32,
    ) -> f32 {
        let mut y = self.label_line(tr("panel.vector.states.signals"), y);
        let shown = bindings.len().min(ids::MAX_SIGNAL_BINDINGS);
        for (i, (name, role)) in bindings.iter().enumerate().take(shown) {
            y = self.signal_name_row(i, name, y);
            let chips: Vec<(ph2d_a11y::NodeId, &str, bool)> = role_labels
                .iter()
                .enumerate()
                .map(|(r, lbl)| {
                    (
                        ids::vector_state_signal_role_id(i, r),
                        lbl.as_str(),
                        r == *role,
                    )
                })
                .collect();
            y = self.segmented("", &chips, y);
        }
        // ⚠️ **No teto o botão SOME, e não fica cinzento:** um botão que não faz nada é pior que
        // um botão que falta, e o teto já é visível — as seis linhas estão na tela.
        if shown < ids::MAX_SIGNAL_BINDINGS {
            let rect = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
            let st = self.store.button_visual(ids::VECTOR_STATE_SIGNAL_ADD);
            let btn = Button::new(
                ids::VECTOR_STATE_SIGNAL_ADD,
                tr("panel.vector.states.signals.add"),
            )
            .kind(ButtonKind::Default)
            .visual(st);
            paint_button(&btn, rect, self.scene, self.text_system, self.theme);
            self.hit_index.register(ids::VECTOR_STATE_SIGNAL_ADD, rect);
            y = y + self.row_h + Spacing::Xs.px();
        }
        y
    }

    /// A linha do NOME: o campo de texto e a lixeira.
    ///
    /// ⚠️ **O espelho do texto autorado é gateado no FOCO**, o mesmo que o campo de fórmula dos
    /// tokens: sem isso cada quadro reescreveria por cima do que está a ser digitado; com isso, um
    /// nome que a porta recusasse voltaria sozinho ao autorado quando o campo perde o foco — o
    /// *"não pegou"* fica visível em vez de silencioso.
    fn signal_name_row(&mut self, i: usize, authored: &str, y: f32) -> f32 {
        let id = ids::vector_state_signal_name_id(i);
        let field_w = (self.inner_w - REMOVE_W - Spacing::Xs.px()).max(0.0);
        let rect = Rect::new(self.inner_x, y, field_w, self.row_h);

        let (state, text, caret, anchor) = match self.store.get(id) {
            Some(InteractiveState::TextInput {
                state,
                text,
                caret,
                selection_anchor,
            }) => (*state, text.clone(), *caret, *selection_anchor),
            // ⚠️ O `authored` é o FALLBACK, e não a fonte: quem semeia e espelha o buffer é o
            // [`mirror`], num passe com a loja MUTÁVEL. Ler aqui o texto autorado por cima do
            // buffer faria o campo re-escrever-se por baixo do que está a ser digitado — que é
            // exatamente o que o espelho gateado no foco existe para impedir.
            _ => (
                TextInputState::Normal,
                authored.to_string(),
                authored.len(),
                None,
            ),
        };
        let input = TextInput::new(id, "")
            .placeholder(tr("panel.vector.states.signals.hint"))
            .state(state);
        paint_text_input_with_buffer(
            &input,
            Some(&text),
            Some(caret),
            anchor,
            rect,
            self.scene,
            self.text_system,
            self.theme,
        );
        self.hit_index.register(id, rect);

        let rid = ids::vector_state_signal_remove_id(i);
        let rrect = Rect::new(
            self.inner_x + field_w + Spacing::Xs.px(),
            y,
            REMOVE_W,
            self.row_h,
        );
        let rst = self.store.button_visual(rid);
        let rbtn = Button::new(rid, tr("panel.vector.states.signals.remove"))
            .kind(ButtonKind::Default)
            .visual(rst);
        paint_button(&rbtn, rrect, self.scene, self.text_system, self.theme);
        self.hit_index.register(rid, rrect);

        y + self.row_h + Spacing::Xs.px()
    }
}

/// **Semeia e ESPELHA os campos de nome** — o passe de loja MUTÁVEL, chamado pelo `paint` antes
/// do corpo.
///
/// ⚠️ **Ele mora fora do `BodyCtx` porque o corpo tem a loja em `&`**, de propósito: o passe de
/// pintura deste painel não escreve estado de widget. A alternativa seria tornar a loja mutável
/// no `BodyCtx` inteiro — trinta secções ganhariam permissão de escrita para que uma a usasse.
///
/// ⚠️ **O espelho é gateado no FOCO**, o mesmo do campo de fórmula dos tokens: sem isso cada
/// quadro reescreveria por cima do que está a ser digitado; com isso, um nome recusado pela porta
/// volta sozinho ao autorado quando o campo perde o foco — o *"não pegou"* fica visível.
pub(crate) fn mirror(store: &mut WidgetStore, bindings: &[(String, usize)]) {
    for (i, (name, _)) in bindings.iter().enumerate().take(ids::MAX_SIGNAL_BINDINGS) {
        let id = ids::vector_state_signal_name_id(i);
        let _ = store.register_if_absent(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: name.clone(),
                caret: name.len(),
                selection_anchor: None,
            },
        );
        if store.focus_id() == Some(id) {
            continue;
        }
        if let Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) = store.get_mut(id)
        {
            if text != name {
                text.clear();
                text.push_str(name);
            }
            *caret = (*caret).min(text.len());
            *selection_anchor = None;
        }
    }
}
