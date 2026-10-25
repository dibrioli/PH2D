//! **As páginas de ajuda do ciclo de vida** (GameEngineFuncional N0) — o irmão do [`crate::api_help`]
//! pelo tecto de LOC; lidas pela mesma [`crate::api_help::all_help`]. O texto diz o que
//! `scene.rs`/`scene_life.rs` fazem (portões em `scene_life_tests.rs`).

use crate::api_help::ApiHelp;

pub(crate) const LIFE_HELP: &[ApiHelp] = &[
    ApiHelp {
        key: "hook.ready",
        params: &[("self", "The object's own table, as [hooks.init] left it.")],
        returns: "",
        more: "Every object that starts in a tick runs its [hooks.init] first; then each of them runs `ready`. So in `ready` the other objects that started with it have already set themselves up, and [ph2d.find_by_name] finds them ready to use.\n\nIt runs once per start: again after a rewind or when the object gets another script, never when you save a new version of the script.",
        example: r#"function ready(self)
  self.door = ph2d.find_by_name("Door")
end"#,
    },
    ApiHelp {
        key: "hook.physics",
        params: &[
            ("self", "The object's own table."),
            ("dt", "The length of one tick in seconds (1/60 by default)."),
        ],
        returns: "",
        more: "It runs on every fixed tick while the scene plays, with the same `dt` as [hooks.update] and right before it. Put movement and collisions here: the result does not depend on the frame rate.",
        example: r#"ph2d.property("speed", 2)

function physics(self, dt)
  local x = ph2d.get(self.id, "x") or 0
  ph2d.set(self.id, "x", x + self.speed * dt)
end"#,
    },
    ApiHelp {
        key: "hook.process",
        params: &[
            ("self", "The object's own table."),
            (
                "dt",
                "The time since the last drawn frame, in seconds; it changes from frame to frame.",
            ),
        ],
        returns: "",
        more: "It runs once per drawn frame while the game runs, after that frame's ticks and signals, never while it is paused. A fast screen calls it more often than [hooks.physics], a slow one less. Use it for what only has to look smooth, such as a blink or a wobble; game rules go in [hooks.physics].",
        example: r#"function process(self, dt)
  self.time = (self.time or 0) + dt
  ph2d.set(self.id, "scale_x", 1 + math.sin(self.time * 8) * 0.05)
end"#,
    },
    ApiHelp {
        key: "hook.exit",
        params: &[("self", "The object's own table, for the last time.")],
        returns: "",
        more: "It runs once when the object leaves: [ph2d.despawn] (at the end of that frame, while the object still exists), when it dies or loses its script (on the next tick, when it may already be gone and [ph2d.get] on it gives `nil`), when [ph2d.attach_script] gives it another script, and when the game stops. An object stopped by an error does not run it.\n\nWhen the game stops, whatever `exit` changes is thrown away, because the scene goes back to how you made it; there it cannot create, remove or give scripts to objects.",
        example: r#"function exit(self)
  print("bye from", self.id)
end"#,
    },
    ApiHelp {
        key: "ph2d.connect",
        params: &[
            (
                "entity",
                "The object to listen to, such as one from [ph2d.find_by_name].",
            ),
            (
                "signal",
                "The name of the signal, as that object sends it. An empty name is an error.",
            ),
            (
                "callback",
                "The function to call, with this object's `self` and the object that sent the signal.",
            ),
        ],
        returns: "",
        more: "Unlike [hooks.on_signal], which hears every signal of the scene, the callback runs only when THAT object sends THAT signal — by [ph2d.emit] in its script, or from the scene (a sensor, a timer). It runs in the same place as [hooks.on_signal], right after it.\n\nThe link ends when either object leaves (see [hooks.exit]). It works only while the scene runs, inside a hook; [hooks.ready] is the usual place.",
        example: r#"function ready(self)
  local button = ph2d.object("Button")
  ph2d.connect(button, "pressed", function(me, from)
    ph2d.set(me.id, "y", 2)
  end)
end"#,
    },
];
