use super::*;
use crate::action::Binding;

fn every_binding() -> Vec<Binding> {
    Key::all_named()
        .chain([Key(0x1234)])
        .map(Binding::Key)
        .chain(GamepadButton::ALL.into_iter().map(Binding::PadButton))
        .chain(
            GamepadAxis::ALL
                .into_iter()
                .flat_map(|axis| [true, false].map(|positive| Binding::PadAxis { axis, positive })),
        )
        .collect()
}

#[test]
fn the_player_file_round_trips_every_binding_kind() {
    let all = every_binding();
    let mut k = PlayerKeys::new();
    k.set("jump", all.clone());
    k.set(
        "fire",
        vec![Binding::Key(Key(0x2C)), Binding::Key(Key(0x2E))],
    );
    k.set("none", Vec::new());
    let text = k.to_text();
    assert!(text.starts_with("# PH2D player keys\n"));
    assert_eq!(PlayerKeys::from_text(&text), k, "{text}");
}

#[test]
fn the_text_form_is_the_script_name_form() {
    assert_eq!(binding_text(Binding::Key(Key(0xF700))), "key:up");
    assert_eq!(binding_text(Binding::Key(Key(0x5A))), "key:z");
    assert_eq!(
        binding_text(Binding::PadButton(GamepadButton::South)),
        "pad:south"
    );
    assert_eq!(
        binding_text(Binding::PadAxis {
            axis: GamepadAxis::LeftStickX,
            positive: false
        }),
        "axis:left_stick_x-"
    );
}

#[test]
fn a_bad_line_is_skipped_and_the_rest_is_read() {
    let text = "# PH2D player keys\n\
                jump = key:up, key:z\n\
                broken = key:nonsense\n\
                also = pad:nope\n\
                = key:a\n\
                no equals here\n\
                axis = axis:left_stick_x*\n\
                dash = key:q pad:west\n\
                grab=key:r,pad:east\n";
    let k = PlayerKeys::from_text(text);
    assert_eq!(
        k.get("jump"),
        Some([Binding::Key(Key(0xF700)), Binding::Key(Key(0x5A))].as_slice())
    );
    assert_eq!(
        k.get("grab"),
        Some(
            [
                Binding::Key(Key(0x52)),
                Binding::PadButton(GamepadButton::East)
            ]
            .as_slice()
        )
    );
    for bad in ["broken", "also", "axis", "dash", ""] {
        assert_eq!(k.get(bad), None, "{bad}");
    }
}

#[test]
fn apply_over_swaps_only_actions_the_map_has() {
    let map = InputMap::with_player_defaults();
    let mut k = PlayerKeys::new();
    k.set("jump", vec![Binding::Key(Key(0x20))]);
    k.set("nobody_has_this", vec![Binding::Key(Key(0x41))]);
    let out = k.apply_over(&map);
    let jump = out.get(out.id("jump").expect("fábrica")).expect("existe");
    assert_eq!(jump.bindings, [Binding::Key(Key(0x20))]);
    assert_eq!(out.len(), map.len());
    assert!(out.id("nobody_has_this").is_none());
    for a in map.actions().iter().filter(|a| a.name != "jump") {
        assert_eq!(out.get(a.id), Some(a));
    }
}

#[test]
fn changed_counts_actions_whose_bindings_differ() {
    let map = InputMap::with_player_defaults();
    let mut k = PlayerKeys::new();
    assert_eq!(k.changed(&map), 0);
    let same = map.get(map.id("dash").expect("fábrica")).expect("existe");
    k.set("dash", same.bindings.clone());
    k.set("jump", vec![Binding::Key(Key(0x20))]);
    k.set("grab", Vec::new());
    k.set("ghost", vec![Binding::Key(Key(0x41))]);
    assert_eq!(k.changed(&map), 2, "dash é igual e ghost não existe");
    k.clear("jump");
    assert_eq!(k.changed(&map), 1);
    k.clear_all();
    assert!(k.is_empty());
}

/// ⭐ O numérico no ficheiro do jogador: `key:kp7` e os de sinal (`key:kp-`, `key:kp+`) voltam à MESMA
/// tecla, e Numpad 7 não vira 7 nem Home.
#[test]
fn numpad_keys_round_trip_through_the_player_file() {
    let kp7 = Key(0xF747);
    assert_eq!(binding_text(Binding::Key(kp7)), "key:kp7");
    let mut k = PlayerKeys::new();
    let signed = [0xF74A, 0xF74B, 0xF74C, 0xF74D, 0xF74E, 0xF74F].map(|c| Binding::Key(Key(c)));
    k.set("jump", vec![Binding::Key(kp7), Binding::Key(Key(0x37))]);
    k.set("signs", signed.to_vec());
    let text = k.to_text();
    assert!(text.contains("jump = key:kp7, key:7\n"), "{text}");
    let back = PlayerKeys::from_text(&text);
    assert_eq!(back, k, "{text}");
    assert_eq!(back.get("signs"), Some(signed.as_slice()));
}
