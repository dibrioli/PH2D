//! **A cena `PH2D_MACRO_SMOKE=8`** (W19): o que cada passo É, no ecrã — o player pintado, o collider e
//! «Simular», gravados e aplicados num projecto limpo.

/// A tradução de uma chave `app.macros.smoke8.*`.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "app.macros.smoke8.floor" => "Floor",
        "app.macros.smoke8.step.1" => {
            "1 of 11 \u{2014} The dark bar is the Floor: a Static body with a Collider, outside any macro. Add \u{25b8} Macro: an empty Macro object appears (the ring is its handle)."
        }
        "app.macros.smoke8.step.2" => {
            "2 of 11 \u{2014} Record: the Macro object records what is made inside it \u{2014} paint, components, simulation."
        }
        "app.macros.smoke8.step.3" => {
            "3 of 11 \u{2014} A new white image (128\u{d7}128) is born inside the Macro object."
        }
        "app.macros.smoke8.step.4" => {
            "4 of 11 \u{2014} Paint a player: the head in red, the body and legs in blue, the arms in green."
        }
        "app.macros.smoke8.step.5" => {
            "5 of 11 \u{2014} Inspector \u{25b8} Add Physics Body: the player gets a body and a Collider the size of the image (the Physics card)."
        }
        "app.macros.smoke8.step.6" => {
            "6 of 11 \u{2014} Physics on (transport bar) and Play: the player falls and lands on the Floor."
        }
        "app.macros.smoke8.step.7" => {
            "7 of 11 \u{2014} Pause, then Stop recording: the steps are listed in the Macros panel. The Macro object now replays its recording at the playhead (at 2.5 s the painting has barely started)."
        }
        "app.macros.smoke8.step.8" => {
            "8 of 11 \u{2014} Save to library: the macro joins your list."
        }
        "app.macros.smoke8.step.9" => {
            "9 of 11 \u{2014} The Macro object and its player are deleted."
        }
        "app.macros.smoke8.step.10" => {
            "10 of 11 \u{2014} A clean project: the old Floor goes, and a new Floor is made lower down."
        }
        "app.macros.smoke8.step.11" => {
            "11 of 11 \u{2014} Apply \u{25b8} Create new, then Play: the image is born, gets painted, gets its collider and falls \u{2014} onto THIS Floor."
        }
        _ => return None,
    })
}
