//! **O rato, o teclado e a queda** sobre a aba. O premido é nativo (o Down captura, o Up DENTRO dispara);
//! uma alça e um número começam no Down; uma queda diz o que vai acontecer ANTES de soltar.

use ph2d_editor_core::doc_surface::Key;

use crate::doc::{Drag, Dropping, Edit, Field, Hit, JointEditorDoc};
use crate::stage::role_at;
use crate::widgets::format;

impl JointEditorDoc {
    /// O Down. `true` = era da aba.
    pub fn press(&mut self, x: f32, y: f32) -> bool {
        if !self.area.contains(x, y) {
            self.finish_typing();
            return false;
        }
        let hit = self.hit_at(x, y).cloned();
        if !matches!(hit, Some(Hit::Field(f)) if self.typing.as_ref().is_some_and(|(t, _)| *t == f)) {
            self.finish_typing();
        }
        match hit {
            Some(Hit::Handle(h)) if self.menu.is_none() => {
                let value = self.value(h.field()).unwrap_or(0.0);
                self.drag = Some(Drag { handle: h, value });
            }
            Some(Hit::Field(f)) if self.menu.is_none() => {
                if self.typing.is_none() {
                    self.typing = Some((f, String::new()));
                }
            }
            other => self.pressed = other,
        }
        true
    }

    /// O movimento. `true` = a pintura mudou.
    pub fn motion(&mut self, x: f32, y: f32) -> bool {
        let before = (self.hover.clone(), self.drag);
        self.hover = self.hit_at(x, y).cloned();
        if let Some(d) = self.drag.as_mut()
            && let Some((_, g)) = self.geo.iter().find(|(h, _)| *h == d.handle)
        {
            d.value = g.value_at(x, y);
        }
        before != (self.hover.clone(), self.drag)
    }

    /// O Up: a alça presa escreve; o controlo premido dispara só se o Up for DENTRO dele.
    pub fn release(&mut self, x: f32, y: f32) -> bool {
        if let Some(d) = self.drag.take() {
            self.commit(d.handle.field(), d.value);
            return true;
        }
        let Some(p) = self.pressed.take() else {
            return false;
        };
        if self.hit_at(x, y) == Some(&p) {
            self.activate(p);
        }
        true
    }

    /// Um clique num alvo.
    pub fn activate(&mut self, hit: Hit) {
        if let Some(open) = self.menu {
            self.menu = None;
            match hit {
                Hit::Kind(k) => self.choose_kind(k),
                Hit::Template(id) => self.choose_template(id),
                Hit::OpenMenu(m) if m != open => self.menu = Some(m),
                _ => {}
            }
            return;
        }
        match hit {
            Hit::Row(sid) => self.open_on(sid),
            Hit::New => self.new_draft(),
            Hit::Remove => {
                if let Some(a) = self.assembly() {
                    let sid = a.sid;
                    self.pending.push(Edit::Remove { assembly: sid });
                    self.note = None;
                }
            }
            Hit::OpenMenu(m) => self.menu = Some(m),
            Hit::Kind(k) => self.choose_kind(k),
            Hit::Template(id) => self.choose_template(id),
            Hit::Shape(role, n) => {
                let bits = self
                    .assembly()
                    .and_then(|a| a.parts_of(role).nth(usize::from(n)).map(|p| p.bits))
                    .filter(|&b| b != 0);
                if let Some(bits) = bits {
                    self.pending.push(Edit::Select { bits });
                }
            }
            Hit::World(role) => {
                let (assembly, template) = (self.assembly().map_or(0, |a| a.sid), self.template().id.0);
                self.pending.push(Edit::PlaceWorld {
                    assembly,
                    template,
                    role,
                });
            }
            Hit::Scope(i) => self.set_scope(i),
            Hit::Flag(f) => self.toggle(f),
            Hit::Drive(d) => self.set_drive(d),
            Hit::Inspect => {
                if let Some(bits) = self.targets().first().map(|j| j.bits) {
                    self.pending.push(Edit::Select { bits });
                }
            }
            Hit::Field(_) | Hit::Handle(_) => {}
        }
    }

    /// Escreve o número que se estava a escrever (vazio = nada muda).
    pub fn finish_typing(&mut self) {
        if let Some((f, s)) = self.typing.take()
            && let Ok(v) = s.trim().replace(',', ".").parse::<f32>()
        {
            self.commit(f, v);
        }
    }

    /// Uma tecla. `true` = tomada.
    pub fn key(&mut self, key: Key, pressed: bool, text: Option<&str>) -> bool {
        if !pressed {
            return self.typing.is_some();
        }
        if let Some((_, buf)) = self.typing.as_mut() {
            if matches!(key, Key::Enter | Key::Tab) {
                self.finish_typing();
            } else if key == Key::Escape {
                self.typing = None;
            } else if key == Key::Backspace {
                buf.pop();
            } else if let Some(t) = text {
                buf.extend(t.chars().filter(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '-')));
            }
            return true;
        }
        if key == Key::Escape && (self.menu.is_some() || self.drag.is_some()) {
            self.menu = None;
            self.drag = None;
            return true;
        }
        false
    }

    /// O papel debaixo de `(x, y)` no desenho.
    #[must_use]
    pub fn role_under(&self, x: f32, y: f32) -> Option<u16> {
        let map = self.map?;
        role_at(&map, self.template(), &self.counts, x, y)
    }

    /// Um arrasto de objetos passa por cima (`Some`) ou saiu (`None`). `true` = largar ali faz uma junta.
    pub fn drop_hover(&mut self, at: Option<(f32, f32, &[String])>) -> bool {
        let Some((x, y, names)) = at else {
            self.dropping = None;
            return false;
        };
        let role = self.role_under(x, y);
        let (ok, line) = match role {
            Some(r) => self.verdict(r, names),
            None => (false, ph2d_i18n::tr("joint_editor.drop.over_a_shape").to_owned()),
        };
        self.dropping = Some(Dropping {
            role,
            names: names.to_vec(),
            ok,
            line,
        });
        ok
    }

    /// Largou-se. `true` = a aba recebeu (o pedido sai no quadro).
    pub fn drop_at(&mut self, x: f32, y: f32, names: &[String]) -> bool {
        self.dropping = None;
        self.role_under(x, y).is_some_and(|r| self.drop_into(r, names))
    }

    /// O que o campo mostra enquanto se escreve nele: o valor de antes, até à primeira tecla.
    #[must_use]
    pub fn typing_shown(&self, f: Field) -> Option<String> {
        let (t, s) = self.typing.as_ref()?;
        (*t == f).then(|| {
            if s.is_empty() {
                self.value(f).map_or_else(String::new, format)
            } else {
                s.clone()
            }
        })
    }
}
