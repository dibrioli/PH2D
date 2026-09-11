//! **ARCH-GATE: Ctrl+E on the dope-sheet distributes the selected keys.**
//!
//! Distribute (crown-jewels §3) is parameterless, so it rides the timeline chord
//! block beside Copy/Cut/Paste/Duplicate/Reverse (C/X/V/D/R). That block lives in
//! `key_input`, and a key press there cannot be synthesised headlessly (the winit
//! `KeyEvent` has a private field — the same wall `the_hovered_area_owns_the_clipboard_chord`
//! documents), so this reads the SOURCE: the block must map `KeyE` (with a
//! selection) to `DistributeSelectedKeys`, and it must sit inside the timeline
//! chord block (after the Copy arm), not somewhere else.
//!
//! Mutation: drop the arm, or point it at a different intent -> the exact
//! substring vanishes -> RED. This is the same coverage the other five chord
//! verbs have (their route is smoke-verified behind the same barrier).
//!
//! ⚠️ **O bloco MUDOU DE ARQUIVO em 2026-07-27** (integração `line/anim` + `line/physics`):
//! o `keyboard.rs` cruzou o cap de 600 LOC na árvore combinada e as quatro travas da
//! timeline saíram para o irmão `keyboard_timeline.rs`. O ARM e a âncora `Copy` viajaram
//! JUNTOS, então a asserção de ordem intra-bloco continua valendo e continua podendo
//! falhar — o que este gate afirma é a rota, não o endereço.

use std::fs;

fn keyboard_src() -> String {
    fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/input_dispatch/keyboard_timeline.rs"
    ))
    .expect("keyboard_timeline.rs legível")
}

#[test]
fn ctrl_e_maps_to_distribute_selected_keys_in_the_timeline_chord_block() {
    let src = keyboard_src();
    let arm = "KeyCode::KeyE if has_selection => Some(I::DistributeSelectedKeys)";
    let at = src
        .find(arm)
        .expect("Ctrl+E must route to DistributeSelectedKeys in the chord block");
    // It belongs to the timeline chord block — after the Copy arm that opens it,
    // never a stray match elsewhere in the file.
    let copy = src
        .find("KeyCode::KeyC if has_selection => Some(I::CopySelection)")
        .expect("the timeline chord block (Copy arm)");
    assert!(
        copy < at,
        "the Distribute arm must sit inside the timeline chord block, after Copy"
    );
}
