//! ⭐ **A escrita num campo do painel é a da caixa do Inspector** (smoke do dono 16/11): uma `NumberInput`
//! numa `WidgetStore` própria, dirigida pelas portas públicas do despacho — o clique foca e seleciona tudo,
//! o 2.º clique põe o caret, o duplo clique seleciona tudo, Backspace/setas/Ctrl+A/C/X/V, Enter e Esc são
//! os do `dispatch_key`, e o número sai pelo parse do motor (`commit_number_buffer`, com contas).

use bumpalo::Bump;
use ph2d_editor_core::doc_surface::{Key, KeyInput};
use ph2d_editor_core::interaction::{
    HitIndex, InteractiveState, KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT, KEY_ARROW_UP,
    KEY_BACKSPACE, KEY_DELETE, KEY_ENTER, KEY_ESCAPE, KEY_KEY_A, KEY_KEY_C, KEY_KEY_V, KEY_KEY_X,
    WidgetEvent, WidgetStore, apply_clipboard_paste, blur_focus, dispatch_key,
    dispatch_pointer_with_text, dispatch_text_input,
};
use ph2d_editor_core::registry::hash_node_id;
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::widget::card_field::{CardFieldSpec, handle_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{KeyEvent, KeyKind, PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;

use crate::doc::{Field, Hit, JointEditorDoc};

/// O id da única caixa a ser escrita (o campo muda; a caixa é uma).
const BOX: ph2d_a11y::NodeId = hash_node_id("joint_editor.field_box");

/// A caixa do motor e onde ela está no ecrã.
#[derive(Default)]
pub struct FieldBox {
    store: WidgetStore,
    hits: HitIndex,
    /// Um Down dentro da caixa entregue ao motor (o Move e o Up seguem para lá).
    pressing: bool,
    /// O texto copiado por Ctrl+C/X, para a shell.
    copy: Option<String>,
    /// O número que o texto semeado diz: entrar e sair sem mudar nada não grava (o arredondado
    /// do texto, espalhado por todas as juntas do alcance).
    seeded: Option<f64>,
}

impl JointEditorDoc {
    /// O rect que o motor mede: a caixa com a célula da pega a acabar onde o valor começa (a pega da aba
    /// pode ser mais larga, «Min»), para o clique cair no byte pintado.
    fn box_rect(&self, f: Field) -> Option<Rect> {
        let field = self.hits_for(&Hit::Field(f))?;
        let value_x = self
            .hits_for(&Hit::Scrub(f))
            .map_or(field.x + handle_rect(field).w, |s| s.x + s.w);
        let x = value_x - handle_rect(field).w;
        Some(Rect::new(x, field.y, field.x + field.w - x, field.h))
    }

    /// Semeia a caixa com o valor de `f` e passa a escrevê-la (sem foco: quem foca é o gesto).
    fn seed_box(&mut self, f: Field) {
        let value = f64::from(self.value(f).unwrap_or(0.0));
        let (step, (lo, hi)) = self.step_of(f);
        let b = &mut self.field_box;
        b.seeded = ph2d_editor_core::interaction::util::format_number(value)
            .parse()
            .ok();
        b.store.set_focus(None);
        b.store.register(
            BOX,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: String::new(),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        // A seta ↑/↓ anda um passo do campo e pára na faixa dele (`card_field::arrow`).
        b.store.set_number_range(BOX, lo, hi, step);
        b.store.mark_card_field(
            BOX,
            CardFieldSpec {
                step,
                scrub: step,
                default: None,
            },
        );
        self.typing = Some(f);
    }

    /// Abre a escrita de `f` como o foco do Inspector: o texto todo selecionado.
    pub fn start_typing(&mut self, f: Field) {
        self.seed_box(f);
        ph2d_editor_core::interaction::dispatch::focus_number_field(&mut self.field_box.store, BOX);
    }

    /// Os eventos do motor: o número gravado vai para a cena; o `Blur` fecha a escrita.
    fn drain(&mut self, events: &[WidgetEvent]) {
        let Some(f) = self.typing else { return };
        for e in events {
            match e {
                WidgetEvent::ValueChanged(id) if *id == BOX => {
                    if let Some(v) = self.field_box.store.number_value(BOX)
                        && self.field_box.seeded != Some(v)
                    {
                        #[allow(clippy::cast_possible_truncation)] // o número do cartão é f32
                        self.commit(f, v as f32);
                        self.field_box.seeded = Some(v);
                    }
                }
                WidgetEvent::Blur(id) if *id == BOX => self.typing = None,
                _ => {}
            }
        }
        if self.field_box.store.focus_id() != Some(BOX) && !self.field_box.pressing {
            self.typing = None;
        }
    }

    /// Escreve o número que se estava a escrever (o parse do motor; inválido = nada muda).
    pub fn finish_typing(&mut self) {
        if self.typing.is_none() {
            return;
        }
        let arena = Bump::new();
        let ev = blur_focus(&mut self.field_box.store, &arena);
        self.drain(ev);
        self.field_box.pressing = false;
        self.typing = None;
    }

    /// Larga a escrita sem gravar.
    pub fn cancel_typing(&mut self) {
        self.field_box.store.set_focus(None);
        self.field_box.pressing = false;
        self.typing = None;
    }

    fn pointer_event(kind: PointerKind, x: f32, y: f32, now_ns: u128) -> PointerEvent {
        PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: now_ns,
        }
    }

    /// O Down no corpo do campo `f`: abre a escrita (se não estava) e entrega o gesto ao motor — foco com
    /// tudo selecionado, ou o caret no 2.º clique, ou tudo no duplo clique.
    pub(crate) fn press_box(&mut self, f: Field, x: f32, y: f32, ts: Option<&mut TextSystem>) {
        let Some(r) = self.box_rect(f) else { return };
        if self.typing != Some(f) {
            self.finish_typing();
            self.seed_box(f);
        }
        let now = self.now_ns;
        let b = &mut self.field_box;
        b.hits.clear_for_frame();
        b.hits.register(BOX, r);
        b.pressing = true;
        let arena = Bump::new();
        let ev = dispatch_pointer_with_text(
            &mut b.store,
            &b.hits,
            Self::pointer_event(PointerKind::Down, x, y, now),
            ts,
            &arena,
        );
        self.drain(ev);
    }

    /// O Move ou o Up de um Down entregue ao motor. `false` = não era dele.
    pub(crate) fn pointer_box(
        &mut self,
        kind: PointerKind,
        x: f32,
        y: f32,
        ts: Option<&mut TextSystem>,
    ) -> bool {
        if !self.field_box.pressing {
            return false;
        }
        if kind == PointerKind::Up {
            self.field_box.pressing = false;
        }
        let now = self.now_ns;
        let b = &mut self.field_box;
        let arena = Bump::new();
        let ev = dispatch_pointer_with_text(
            &mut b.store,
            &b.hits,
            Self::pointer_event(kind, x, y, now),
            ts,
            &arena,
        );
        // O arrasto no corpo grava só no Up (o motor emite a cada Move).
        if kind == PointerKind::Up {
            self.drain(ev);
        }
        true
    }

    /// Uma tecla para a caixa a ser escrita. `true` = tomada.
    pub(crate) fn key_box(&mut self, input: &KeyInput<'_>) -> bool {
        if self.typing.is_none() {
            return false;
        }
        if !input.pressed {
            return true;
        }
        let cmd = input.mods.ctrl || input.mods.meta;
        let code = match input.key {
            Key::Enter => Some(KEY_ENTER),
            Key::Escape => Some(KEY_ESCAPE),
            Key::Backspace => Some(KEY_BACKSPACE),
            Key::Delete => Some(KEY_DELETE),
            Key::Left => Some(KEY_ARROW_LEFT),
            Key::Right => Some(KEY_ARROW_RIGHT),
            Key::Up => Some(KEY_ARROW_UP),
            Key::Down => Some(KEY_ARROW_DOWN),
            Key::Char(c) if cmd => match c.to_ascii_lowercase() {
                'a' => Some(KEY_KEY_A),
                'c' => Some(KEY_KEY_C),
                'x' => Some(KEY_KEY_X),
                'v' => Some(KEY_KEY_V),
                _ => None,
            },
            _ => None,
        };
        if input.key == Key::Tab {
            self.finish_typing();
            return true;
        }
        let arena = Bump::new();
        if let Some(keycode) = code {
            let b = &mut self.field_box;
            let ev = dispatch_key(
                &mut b.store,
                KeyEvent {
                    keycode,
                    modifiers: input.mods,
                    kind: KeyKind::Down,
                    timestamp_ns: self.now_ns,
                },
                &arena,
            );
            if let Some(t) = b.store.take_clipboard_copy() {
                b.copy = Some(t);
            }
            if b.store.take_clipboard_paste_request().is_some()
                && let Some(p) = input.paste.as_deref()
            {
                apply_clipboard_paste(&mut b.store, BOX, p);
            }
            self.drain(ev);
        } else if !cmd && let Some(t) = input.text {
            t.chars().for_each(|c| _ = self.type_char(c));
        }
        true
    }

    /// Um carácter escrito, pela tecla ou pelo `Commit` do IME (a shell manda cada um por UM dos dois).
    /// `true` = havia um campo a ser escrito.
    pub fn type_char(&mut self, c: char) -> bool {
        if self.typing.is_none() {
            return false;
        }
        let arena = Bump::new();
        let ev = dispatch_text_input(&mut self.field_box.store, c, &arena);
        self.drain(ev);
        true
    }

    /// O que Ctrl+C/X copiou, para a área de transferência do sistema.
    pub fn take_copy(&mut self) -> Option<String> {
        self.field_box.copy.take()
    }

    /// A escrita de `f` como o motor a tem: `(texto, caret, âncora)` (`None` = `f` não está a ser escrito).
    #[must_use]
    pub fn editing(&self, f: Field) -> Option<(&str, usize, Option<usize>)> {
        if self.typing != Some(f) {
            return None;
        }
        self.field_box
            .store
            .number_input(BOX)
            .map(|(_, _, buf, caret, anchor)| (buf, caret, anchor))
    }
}
