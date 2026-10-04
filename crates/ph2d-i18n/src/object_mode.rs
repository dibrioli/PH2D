//! **O MODO DE EDIÇÃO DO OBJECTO** — o seletor *Mode* da fila, o `Tab` e o cadeado
//! (`docs/UI_New_and_Simple/spec/06_tipos_e_modos_de_objeto.md`, F2). Os nomes seguem o Blender.

/// A tradução de uma chave do modo, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "object_mode.menu" => "Mode",
        "object_mode.object" => "Object Mode",
        "object_mode.paint" => "Paint Mode",
        "object_mode.sculpt" => "Sculpt Mode",
        "object_mode.draw" => "Draw Mode",
        "object_mode.edit" => "Edit Mode",
        "object_mode.entered" => "{mode} \u{2014} Tab returns to Object Mode",
        "object_mode.left" => "Object Mode",
        "object_mode.only_object" => "{name} has only Object Mode",
        "object_mode.nothing_selected" => "Select an object to change its mode",
        "object_mode.locked" => "Leave {mode} (Tab) to select another object",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
