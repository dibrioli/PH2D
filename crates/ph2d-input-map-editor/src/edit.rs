//! Os pedidos da aba ao mapa (plano §3): a família aplica-os em `hero.input_map` no `frame`.

use ph2d_input::recipes::Recipe;
use ph2d_input::{ActionId, Binding, InputMap, Key, PlayerKeys};

#[derive(Clone, Debug, PartialEq)]
pub enum InputMapEdit {
    /// Uma acção nova com este nome.
    Add(String),
    Remove(ActionId),
    AddBinding(ActionId, Binding),
    /// Tira a ligação `n` da acção.
    RemoveBinding(ActionId, usize),
    /// `(dead_zone, press_point)`.
    SetZone(ActionId, f32, f32),
    StartFrom(Recipe),
    SetPlayersCanChange(bool),
    SetShown(ActionId, bool),
    SetOpenKey(Key),
    ForgetPlayerKeys,
}

/// **Aplica** um pedido no mapa do projecto (a família chama-a no `frame`; o undo por diff faz de cada uma UM passo).
/// `player_file` = o ficheiro do jogador deste jogo neste computador (o `ForgetPlayerKeys` apaga-o).
pub fn apply(map: &mut InputMap, edit: InputMapEdit, player_file: Option<&std::path::Path>) {
    match edit {
        InputMapEdit::Add(name) => {
            map.create(name);
        }
        InputMapEdit::Remove(id) => {
            map.remove(id);
        }
        InputMapEdit::AddBinding(_, Binding::Key(k)) if k.system_owned().is_some() => {
            // ⛔ Uma tecla do SISTEMA (Windows, Fn, Print Screen) nunca chega ao jogo (plano §13.1).
            tracing::warn!(key = %k.label(), "Input Map: a system key never binds; the request was refused");
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
        InputMapEdit::StartFrom(r) => map.start_from(r),
        InputMapEdit::SetPlayersCanChange(on) => map.controls_mut().players_can_change_keys = on,
        InputMapEdit::SetShown(id, on) => map.controls_mut().set_shown_in_game(id, on),
        InputMapEdit::SetOpenKey(k) => map.controls_mut().open_key = k,
        InputMapEdit::ForgetPlayerKeys => {
            // Gravar «nada trocado» apaga o ficheiro (`file::save`).
            if let Some(p) = player_file {
                let _ = ph2d_player_controls::file::save(p, &PlayerKeys::new());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔ Ligar uma tecla do sistema é recusado (o mapa fica igual); uma tecla do jogo liga.
    #[test]
    fn a_system_key_never_binds_and_a_game_key_does() {
        let mut map = InputMap::with_player_defaults();
        let id = map.id("jump").expect("fábrica");
        let before = map.clone();
        for k in Key::system_keys() {
            apply(
                &mut map,
                InputMapEdit::AddBinding(id, Binding::Key(k)),
                None,
            );
        }
        assert_eq!(map, before, "uma tecla do sistema entrou no mapa");
        let kp7 = Key::named_in_script("kp7").expect("o numérico")[0];
        apply(
            &mut map,
            InputMapEdit::AddBinding(id, Binding::Key(kp7)),
            None,
        );
        let a = map.get(id).expect("existe");
        assert!(a.bindings.contains(&Binding::Key(kp7)));
    }
}
