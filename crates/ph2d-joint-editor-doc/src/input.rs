//! **O rato, o teclado e a queda** sobre a aba. O premido é nativo (o Down captura, o Up DENTRO dispara);
//! uma alça e um número começam no Down; uma queda diz o que vai acontecer ANTES de soltar.

use ph2d_editor_core::doc_surface::{Key, KeyInput};
use ph2d_host::PointerKind;
use ph2d_text::TextSystem;

use ph2d_editor_core::interaction::field_scrub::FieldScrub;

use crate::doc::{Drag, DragBy, Dropping, Edit, Hit, JointEditorDoc};
use crate::stage::role_at;

impl JointEditorDoc {
    /// O Down. `true` = era da aba. `ts` mede o clique no texto da caixa (o caret).
    pub fn press(&mut self, x: f32, y: f32, ts: Option<&mut TextSystem>) -> bool {
        if !self.area.contains(x, y) {
            self.finish_typing();
            return false;
        }
        let hit = self.hit_at(x, y).cloned();
        if self.menu.is_none() && self.grab_start(hit.as_ref(), x, y) {
            self.finish_typing();
            return true;
        }
        if !matches!(hit, Some(Hit::Field(f)) if self.typing == Some(f)) {
            self.finish_typing();
        }
        match hit {
            Some(Hit::Handle(h)) if self.menu.is_none() => {
                let (field, value) = (h.field(), self.value(h.field()).unwrap_or(0.0));
                let by = DragBy::Handle(h);
                self.drag = Some(Drag { field, value, by });
            }
            Some(Hit::Scrub(f)) if self.menu.is_none() && self.scrub_law(f).is_some() => {
                let (v, step, bounds) = self.scrub_law(f).unwrap_or_default();
                let by = DragBy::Scrub(FieldScrub::new(x, f64::from(v), step, bounds));
                self.drag = Some(Drag {
                    field: f,
                    value: v,
                    by,
                });
            }
            Some(Hit::Field(f)) if self.menu.is_none() => self.press_box(f, x, y, ts),
            Some(Hit::Scrub(f)) if self.menu.is_none() => self.start_typing(f),
            other => self.pressed = other,
        }
        true
    }

    /// O movimento. `true` = a pintura mudou.
    pub fn motion(&mut self, x: f32, y: f32, ts: Option<&mut TextSystem>) -> bool {
        if self.pointer_box(PointerKind::Move, x, y, ts) || self.grab_move(x, y) {
            return true;
        }
        let before = (self.hover.clone(), self.drag);
        self.hover = self.hit_at(x, y).cloned();
        let grain = self.drag.map(|d| self.step_of(d.field).0 as f32);
        if let Some(d) = self.drag.as_mut() {
            match &mut d.by {
                // A alça anda no grão do campo (1 °/s, 1 cm), como o número que o Inspector mostra.
                DragBy::Handle(h) => {
                    if let Some((_, g)) = self.geo.iter().find(|(k, _)| k == h) {
                        let q = grain.unwrap_or(1.0);
                        d.value = (g.value_near(x, y, d.value) / q).round() * q;
                    }
                }
                #[allow(clippy::cast_possible_truncation)] // o número do cartão é f32
                DragBy::Scrub(s) => {
                    if let Some(v) = s.motion(x, self.mods) {
                        d.value = v as f32;
                    }
                }
            }
        }
        before != (self.hover.clone(), self.drag)
    }

    /// O Up: a alça presa escreve; o controlo premido dispara só se o Up for DENTRO dele.
    pub fn release(&mut self, x: f32, y: f32, ts: Option<&mut TextSystem>) -> bool {
        if self.pointer_box(PointerKind::Up, x, y, ts) || self.grab.take().is_some() {
            return true;
        }
        if let Some(d) = self.drag.take() {
            match d.by {
                // Abaixo do limiar a letra foi um clique: abre a escrita, como no Inspector.
                DragBy::Scrub(s) if !s.moved() => self.start_typing(d.field),
                _ => self.commit(d.field, d.value),
            }
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
                let (assembly, template) =
                    (self.assembly().map_or(0, |a| a.sid), self.template().id.0);
                self.pending.push(Edit::PlaceWorld {
                    assembly,
                    template,
                    role,
                });
            }
            Hit::Scope(i) => self.set_scope(i),
            Hit::Flag(f) => self.toggle(f),
            Hit::Drive(d) => self.set_drive(d),
            Hit::Make => {
                let (template, counts, at) =
                    (self.template().id.0, self.draft_counts(), self.view_centre);
                let edits = self.draft_edits.clone();
                self.pending.push(Edit::Make {
                    template,
                    counts,
                    at,
                    edits,
                });
            }
            Hit::Inspect => {
                if let Some(bits) = self.targets().first().map(|j| j.bits) {
                    self.pending.push(Edit::Select { bits });
                }
            }
            Hit::Fold(k) => self.toggle_fold(k),
            Hit::Field(_) | Hit::Scrub(_) | Hit::Handle(_) | Hit::Split | Hit::Bar => {}
        }
    }

    /// Uma tecla. `true` = tomada.
    pub fn key(&mut self, input: &KeyInput<'_>) -> bool {
        if self.key_box(input) {
            return true;
        }
        if input.pressed && input.key == Key::Escape && (self.menu.is_some() || self.drag.is_some())
        {
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
            None => (
                false,
                ph2d_i18n::tr("joint_editor.drop.over_a_shape").to_owned(),
            ),
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
        self.role_under(x, y)
            .is_some_and(|r| self.drop_into(r, names))
    }
}
