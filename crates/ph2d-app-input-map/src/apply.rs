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
        other => tracing::warn!(?other, "input map: edit not wired yet"),
    }
}
