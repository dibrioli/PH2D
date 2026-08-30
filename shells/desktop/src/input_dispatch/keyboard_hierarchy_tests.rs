//! A lei pura da [`super::verb_for`] — as cinco condições e os dois verbos.

use super::{HierKeyVerb, KeyFacts, verb_for};
use winit::keyboard::KeyCode;

/// Um estado que devia recusar, com o nome dele. ⚠️ `fn(&mut KeyFacts)` e não uma closure nua:
/// duas closures idênticas têm tipos DIFERENTES, e um array delas não compila.
type Caso = (&'static str, fn(&mut KeyFacts));

/// O caso vivo: tecla premida, ponteiro sobre o painel, sem texto focado.
fn vivo(key: KeyCode, cmd: bool) -> KeyFacts {
    KeyFacts {
        pressed: true,
        repeat: false,
        over_panel: true,
        text_focused: false,
        cmd,
        key: Some(key),
    }
}

#[test]
fn delete_asks_to_delete_and_cmd_d_asks_to_duplicate() {
    assert_eq!(
        verb_for(vivo(KeyCode::Delete, false)),
        Some(HierKeyVerb::Delete)
    );
    // ⚠️ O `Backspace` conta: num teclado de portátil ele **é** a tecla de apagar.
    assert_eq!(
        verb_for(vivo(KeyCode::Backspace, false)),
        Some(HierKeyVerb::Delete)
    );
    assert_eq!(
        verb_for(vivo(KeyCode::KeyD, true)),
        Some(HierKeyVerb::Duplicate)
    );
}

/// ⛔ **As CINCO recusas** — e cada uma existe porque a tecla tem outro dono naquele estado.
#[test]
fn every_guard_refuses_and_each_one_alone_is_enough() {
    // O `D` nu é a booleana de subtração do modo vetorial; o `Delete` com modificador é «apagar a
    // palavra» em todo campo de texto do mundo.
    assert_eq!(
        verb_for(vivo(KeyCode::KeyD, false)),
        None,
        "o `D` nu não é nosso"
    );
    assert_eq!(
        verb_for(vivo(KeyCode::Delete, true)),
        None,
        "`Ctrl+Delete` não é nosso"
    );
    let casos: [Caso; 5] = [
        ("solta", |f: &mut KeyFacts| f.pressed = false),
        ("repetida", |f: &mut KeyFacts| f.repeat = true),
        ("fora do painel", |f: &mut KeyFacts| f.over_panel = false),
        ("texto focado", |f: &mut KeyFacts| f.text_focused = true),
        ("tecla desconhecida", |f: &mut KeyFacts| f.key = None),
    ];
    for (nome, mudanca) in casos {
        let mut f = vivo(KeyCode::Delete, false);
        mudanca(&mut f);
        assert_eq!(
            verb_for(f),
            None,
            "«{nome}» devia recusar sozinha — as guardas não se cobrem umas às outras"
        );
        // E o mesmo estado tem de recusar o outro verbo, senão a guarda protege só metade da lei.
        let mut f = vivo(KeyCode::KeyD, true);
        mudanca(&mut f);
        assert_eq!(verb_for(f), None, "«{nome}» deixou o Duplicate passar");
    }
}
