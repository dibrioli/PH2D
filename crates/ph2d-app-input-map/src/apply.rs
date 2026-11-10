//! Aplicar um [`InputMapEdit`] no mapa do projecto. As ondas R2–R4 ligam os braços que ainda avisam.

use ph2d_input::InputMap;
use ph2d_input_map_editor::InputMapEdit;

pub(crate) fn apply(map: &mut InputMap, edit: InputMapEdit) {
    match edit {
        InputMapEdit::Add(name) => {
            map.create(name);
        }
        InputMapEdit::Remove(id) => {
            map.remove(id);
        }
        InputMapEdit::AddBinding(id, b) => {
            if let Some(a) = map.get_mut(id)
                && !a.bindings.contains(&b)
            {
                a.bindings.push(b);
            }
        }
        InputMapEdit::RemoveBinding(id, n) => {
            if let Some(a) = map.get_mut(id)
                && n < a.bindings.len()
            {
                a.bindings.remove(n);
            }
        }
        InputMapEdit::SetZone(id, dead, press) => {
            if let Some(a) = map.get_mut(id) {
                a.set_zone(dead, press);
            }
        }
        other => tracing::warn!(?other, "input map: edit not wired yet"),
    }
}
