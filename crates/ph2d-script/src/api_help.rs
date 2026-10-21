//! **O texto da página de ajuda da API** (CodeEditor Onda 2, item 5; `docs/CodeEditor/04_pagina_de_ajuda.md`):
//! cada argumento, o que devolve, um parágrafo a mais e um exemplo, para cada linha da [`crate::api_table`].
//!
//! Chaveado pelo NOME (`ph2d.set`, `motion.fire`, `hook.update`, `type.PoseField`) e não por um campo novo
//! em `ApiEntry`: uma linha da tabela acrescentada por outra linha de trabalho compila na mesma e reprova o
//! gate `tests/it/a_api_e_a_tabela.rs::toda_entrada_da_api_tem_ajuda`. O [`crate::api_table::luau_docs`]
//! lê daqui o texto de cada argumento ⇒ o balão do `luau-lsp` e a página dizem o mesmo.
//!
//! ⚠️ O texto diz o que o código FAZ (lido de `host.rs`, `scene.rs`, `lane.rs`, `module.rs`,
//! `lateral.rs`, `motion_api.rs`, e quem chama o `ScriptHost` na app), medido pelos portões de
//! `scene_api_tests.rs` (CodeEditor item 6, `docs/CodeEditor/05_api_que_funciona.md`).

/// O texto de uma entrada.
pub struct ApiHelp {
    /// `<tabela>.<função>` · `hook.<gancho>` · `type.<Tipo>`.
    pub key: &'static str,
    /// `(nome, texto)` de cada argumento, pela ordem da assinatura.
    pub params: &'static [(&'static str, &'static str)],
    /// O que devolve; vazio = nada.
    pub returns: &'static str,
    /// Parágrafos depois da linha `doc` da tabela (linha em branco entre eles); vazio = nenhum.
    pub more: &'static str,
    /// Código Luau que o editor aceita sem aviso (gate `oraculo_lsp` do `ph2d-app-code`).
    pub example: &'static str,
}

pub const API_HELP: &[ApiHelp] = &[
    // ── ph2d: o objecto e a cena ──
    ApiHelp {
        key: "ph2d.set",
        params: &[
            (
                "entity",
                "The object to change, usually `self.id`. An id of an object without a pose is ignored.",
            ),
            (
                "field",
                r#"Which part of the pose: "x" or "y" (position, relative to the parent), "rotation" (radians, counter-clockwise), "scale_x" or "scale_y" (1 = original size)."#,
            ),
            (
                "value",
                "The new value. It must be a real number: `nil`, infinity or NaN stop the object.",
            ),
        ],
        returns: "",
        more: "The write waits in a queue and lands when every script has run its hook for this tick, so [ph2d.get] on the same field still gives the old value until the next tick. When two writes hit the same field, the last one wins (objects run in a fixed order).\n\nA wrong field name or a value that is not a number is not refused on the spot: when the hook ends the object stops running its script and the console says why. Poses written while playing are a preview — rewinding puts each object back where you left it.",
        example: r#"ph2d.property("speed", 2)

function update(self, dt)
  local x = ph2d.get(self.id, "x") or 0
  ph2d.set(self.id, "x", x + self.speed * dt)
end"#,
    },
    ApiHelp {
        key: "ph2d.get",
        params: &[
            (
                "entity",
                "Any object with a pose — with or without a script — such as `self.id` or what [ph2d.find_by_name] gives.",
            ),
            (
                "field",
                r#"One of "x", "y", "rotation", "scale_x" or "scale_y"."#,
            ),
        ],
        returns: "The number as it was at the start of this tick, or `nil` when the id is not an object with a pose (it was removed, or never existed).",
        more: "Every script reads the scene as it was before the first hook of the tick, so the order in which objects run never changes what they see. A [ph2d.set] made earlier in the same hook is not visible yet; an object made by [ph2d.spawn] in this tick is, at the place it was made.\n\nA field that is not one of the five is an error that stops the object, so a typo cannot read as `nil`. It works only while the scene runs, inside a hook.",
        example: r#"ph2d.property("height", 0.5)

function init(self)
  self.base_y = ph2d.get(self.id, "y") or 0
  self.time = 0
end

function update(self, dt)
  self.time = self.time + dt
  ph2d.set(self.id, "y", self.base_y + math.sin(self.time * 3) * self.height)
end"#,
    },
    ApiHelp {
        key: "ph2d.input",
        params: &[(
            "key",
            r#"A gamepad button or axis, such as "gamepad.held.south" or "gamepad.axis.left_stick_x"."#,
        )],
        returns: "For a button, `1` while it is held and `nil` when it is not; for an axis, its position (a stick goes from -1 to 1), or `nil` until it has moved once; `nil` for any other name.",
        more: "This is the gamepad; for the keyboard use [ph2d.key_down] and for the mouse [ph2d.mouse_down] and [ph2d.mouse_position]. Buttons are `gamepad.held.` followed by `south`, `east`, `west`, `north`, `left_bumper`, `right_bumper`, `left_trigger`, `right_trigger`, `select`, `start`, `mode`, `left_stick`, `right_stick`, `dpad_up`, `dpad_down`, `dpad_left` or `dpad_right`. Axes are `gamepad.axis.` followed by `left_stick_x`, `left_stick_y`, `right_stick_x`, `right_stick_y`, `left_trigger` or `right_trigger`.\n\nA misspelt name is not an error: it reads `nil`, just like a button nobody holds. If nothing moves, check the spelling.",
        example: r#"ph2d.property("speed", 3)

function update(self, dt)
  local push = ph2d.input("gamepad.axis.left_stick_x") or 0
  local x = ph2d.get(self.id, "x") or 0
  ph2d.set(self.id, "x", x + push * self.speed * dt)
end"#,
    },
    ApiHelp {
        key: "ph2d.emit",
        params: &[(
            "signal",
            "The name of the signal. An empty name, or one made only of spaces, is an error.",
        )],
        returns: "",
        more: "The signal goes out when the tick ends and reaches the scene in the same frame. Scripts hear it in [hooks.on_signal] on the next frame — every object that has that hook, the sender included.\n\nAn empty name stops the object instead of being ignored: it is almost always a variable that was never filled in.",
        example: r#"ph2d.property("floor", 0)

function update(self, dt)
  local y = ph2d.get(self.id, "y") or 0
  if y < self.floor and not self.landed then
    self.landed = true
    ph2d.emit("landed")
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.find_by_name",
        params: &[(
            "name",
            "The object's name, exactly as written (capitals count).",
        )],
        returns: "The object's id, ready for [ph2d.get] and [ph2d.set], or `nil` when no object has that name.",
        more: "When several objects share a name, it gives the first one in the order of the object list (top to bottom, a parent before its children); copies made while the scene runs come after the objects you made. Masters are not objects of the scene and are never found here — [ph2d.spawn] uses them.\n\nAn object made by [ph2d.spawn] can be found in the same hook. One that [ph2d.despawn] removes can still be found until the frame ends. It works only while the scene runs, inside a hook.",
        example: r#"function update(self, dt)
  local door = ph2d.find_by_name("Door")
  if door then
    ph2d.set(door, "rotation", math.rad(90))
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.object",
        params: &[(
            "name",
            "The object's name, exactly as written (capitals count). Drag the object from the Hierarchy into the script and this is written for you.",
        )],
        returns: "The object's id, ready for [ph2d.get] and [ph2d.set].",
        more: "It is the way to point at one object you made: the editor checks the name while you type (a name no object has is marked in the list of problems), and renaming the object in the Hierarchy rewrites the name in every script of the project. When no object has that name it is an error that stops this object's script, with the name in the message — use [ph2d.find_by_name] when the object may not exist.\n\nWhen several objects share a name, it gives the first one in the order of the object list, like [ph2d.find_by_name]. It works only while the scene runs, inside a hook; to keep the object for later, keep it in `self` in `init`.",
        example: r#"function init(self)
  self.door = ph2d.object("Door")
end

function update(self, dt)
  ph2d.set(self.door, "rotation", math.rad(90))
end"#,
    },
    ApiHelp {
        key: "ph2d.spawn",
        params: &[
            (
                "master",
                "The name of a master in the scene, exactly as written. Left out, the new object is empty (a position and the name \"Object\").",
            ),
            (
                "x",
                "Where the copy lands, horizontally. Give `x` and `y` together, or neither (then the copy keeps the master's position).",
            ),
            ("y", "Where the copy lands, vertically."),
        ],
        returns: "The new object's id, usable at once with [ph2d.set], [ph2d.get] and [ph2d.attach_script].",
        more: "The copy exists right away — the next line of the hook can move it — and its own script, if the master has one, starts on the next tick. It belongs to this run only: it is not part of the scene you save, it is not in the object list, and rewinding removes it. Use [ph2d.despawn] to take it away sooner.\n\nA name with no master is an error that stops the object. It works only while the scene runs, inside a hook.",
        example: r#"function update(self, dt)
  if ph2d.key_pressed("space") then
    local x = ph2d.get(self.id, "x") or 0
    local y = ph2d.get(self.id, "y") or 0
    ph2d.spawn("Bullet", x, y + 1)
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.despawn",
        params: &[(
            "entity",
            "The object to remove — one created while the scene runs, such as `self.id` of a copy or an id from [ph2d.spawn].",
        )],
        returns: "`true` when the object will go; `false` when the id names no object, or names one you made in the scene.",
        more: "The object leaves when the frame ends, the same moment as everything else that dies in the scene; until then it can still be read and found, but its hooks no longer run and its waits are dropped.\n\nObjects you placed in the scene are never removed while playing — that would delete your work. The call is refused, the console says so in yellow, and the script carries on.",
        example: r#"function init(self)
  ph2d.wait(2)
  ph2d.despawn(self.id)
end"#,
    },
    ApiHelp {
        key: "ph2d.attach_script",
        params: &[
            (
                "entity",
                "An object created while the scene runs, such as an id from [ph2d.spawn].",
            ),
            (
                "script",
                r#"The script's name in the project, with its extension, such as "follow.luau"."#,
            ),
        ],
        returns: "`true` when the script was attached; `false` when the id names no object, or names one you made in the scene.",
        more: "The new script starts with [hooks.init] on the next tick — or on the next frame, when no other object runs that script yet and it has to be loaded first. A name that is not in the project is an error that stops the object.\n\nObjects you placed in the scene keep the scripts you gave them: attaching one while playing would change your scene, so it is refused, the console says so, and the script carries on.",
        example: r#"function update(self, dt)
  if ph2d.key_pressed("e") then
    local helper = ph2d.spawn()
    ph2d.attach_script(helper, "follow.luau")
  end
end"#,
    },
    // ── ph2d: o teclado e o rato ──
    ApiHelp {
        key: "ph2d.key_down",
        params: &[(
            "key",
            r#"A key name in lower case: "a" to "z", "0" to "9", "space", "enter", "tab", "escape", "backspace", "delete", "home", "up", "down", "left", "right", "f1" to "f12", "[", "]", ",", ".", and "shift", "ctrl" or "alt" (either side) or "left_shift", "right_ctrl" and so on."#,
        )],
        returns: "`true` while the key is held.",
        more: "A name that is not a key is an error that stops the object, so a typo cannot read as a key nobody presses. The editor lists the names as you type.\n\nA key pressed together with Ctrl, Alt or the system key is a shortcut of the editor, not a key of the game, and while a script tab has the keyboard nothing reaches the game.",
        example: r#"ph2d.property("speed", 4)

function update(self, dt)
  local x = ph2d.get(self.id, "x") or 0
  if ph2d.key_down("left") then x = x - self.speed * dt end
  if ph2d.key_down("right") then x = x + self.speed * dt end
  ph2d.set(self.id, "x", x)
end"#,
    },
    ApiHelp {
        key: "ph2d.key_pressed",
        params: &[("key", "A key name, as in [ph2d.key_down].")],
        returns: "`true` on the first tick after the key went down, `false` on every other tick.",
        more: "Use it for things that should happen once per press, like a jump or a shot. Even a very quick tap, pressed and let go between two ticks, counts once; when a frame runs two ticks, only the first one sees it.",
        example: r#"function update(self, dt)
  if ph2d.key_pressed("space") then
    local y = ph2d.get(self.id, "y") or 0
    ph2d.set(self.id, "y", y + 2)
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.key_released",
        params: &[("key", "A key name, as in [ph2d.key_down].")],
        returns: "`true` on the first tick after the key went up, `false` on every other tick.",
        more: "It pairs with [ph2d.key_pressed], for example to measure how long a key was held.",
        example: r#"function update(self, dt)
  if ph2d.key_pressed("space") then self.held = 0 end
  if ph2d.key_down("space") then self.held = (self.held or 0) + dt end
  if ph2d.key_released("space") then print("held for", self.held) end
end"#,
    },
    ApiHelp {
        key: "ph2d.mouse_down",
        params: &[("button", r#""left", "right" or "middle"."#)],
        returns: "`true` while the button is held after a click that started on the scene.",
        more: "A click on a panel or a button of the editor does not reach the game — only one that starts on the scene does. Letting go always counts, wherever the mouse is. Any other button name is an error.",
        example: r#"function update(self, dt)
  if ph2d.mouse_down("left") then
    local x, y = ph2d.mouse_position()
    if x and y then
      ph2d.set(self.id, "x", x)
      ph2d.set(self.id, "y", y)
    end
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.mouse_pressed",
        params: &[("button", r#""left", "right" or "middle"."#)],
        returns: "`true` on the first tick after a click that started on the scene, `false` otherwise.",
        more: "Like [ph2d.key_pressed], a quick click between two ticks counts once.",
        example: r#"function update(self, dt)
  if ph2d.mouse_pressed("left") then
    local x, y = ph2d.mouse_position()
    if x and y then
      ph2d.spawn("Star", x, y)
    end
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.mouse_released",
        params: &[("button", r#""left", "right" or "middle"."#)],
        returns: "`true` on the first tick after the button went up, `false` otherwise.",
        more: "Letting go is seen even when the mouse has left the scene, so a drag always ends.",
        example: r#"function update(self, dt)
  if ph2d.mouse_released("left") then
    print("dropped")
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.mouse_position",
        params: &[],
        returns: "Two numbers, the `x` and `y` of the scene point under the mouse — the same units [ph2d.set] uses for a top-level object — or two `nil`s when there is no scene view.",
        more: "It follows the view of the scene, so zooming or panning changes the numbers for the same spot on the screen but never the point of the scene under the mouse. It works over the whole window, not only on the scene.",
        example: r#"function update(self, dt)
  local mx, my = ph2d.mouse_position()
  if mx and my then
    local x = ph2d.get(self.id, "x") or 0
    local y = ph2d.get(self.id, "y") or 0
    ph2d.set(self.id, "rotation", math.atan2(my - y, mx - x))
  end
end"#,
    },
    // ── ph2d: as declarações e os valores ──
    ApiHelp {
        key: "ph2d.property",
        params: &[
            (
                "name",
                "The property's name: letters, digits and `_`, not starting with a digit. `id` belongs to the engine, and each name can be declared only once.",
            ),
            (
                "default",
                "The starting value. Its kind picks the Inspector control: a number, `true`/`false`, text, a position made by `ph2d.vec2`, or a colour made by `ph2d.color` (every channel from 0 to 1).",
            ),
            (
                "options",
                "Optional. `min`, `max` and `step` for a number or a position (`step` above 0, `min` not above `max`); `options = { ... }` turns text into a list to pick from, and the default must be in it. Any other key is an error.",
            ),
        ],
        returns: "",
        more: "Call it at the top of the script, outside any function: the editor runs the top once when the script loads to collect the properties, and a call inside a hook is an error. A script can declare up to 32 properties.\n\nThe value is in `self.<name>` before [hooks.init] runs. When you change it in the Inspector, the new value reaches `self` on the next tick, even while playing. If the script changed `self.<name>` itself, the Inspector only overwrites it when you change the value there again.",
        example: r#"ph2d.property("speed", 2, { min = 0, max = 10, step = 0.1 })
ph2d.property("mode", "walk", { options = { "walk", "run" } })

function update(self, dt)
  local step = self.speed * dt
  if self.mode == "run" then
    step = step * 2
  end
  local x = ph2d.get(self.id, "x") or 0
  ph2d.set(self.id, "x", x + step)
end"#,
    },
    ApiHelp {
        key: "ph2d.vec2",
        params: &[("x", "The horizontal part."), ("y", "The vertical part.")],
        returns: "A table with `x` and `y`, marked as a position ([Vec2]).",
        more: "Use it for a property default (the Inspector then shows two fields) or to keep a point. A plain table such as `{ x = 1, y = 2 }` is not a position for [ph2d.property]: only what `ph2d.vec2` builds carries the mark.\n\nThere is no maths on it — `a + b` between two positions is an error — so work on `.x` and `.y`.",
        example: r#"ph2d.property("target", ph2d.vec2(4, 2))

function update(self, dt)
  local x = ph2d.get(self.id, "x") or 0
  local y = ph2d.get(self.id, "y") or 0
  ph2d.set(self.id, "x", x + (self.target.x - x) * dt)
  ph2d.set(self.id, "y", y + (self.target.y - y) * dt)
end"#,
    },
    ApiHelp {
        key: "ph2d.color",
        params: &[
            ("r", "Red, from 0 to 1."),
            ("g", "Green, from 0 to 1."),
            ("b", "Blue, from 0 to 1."),
            (
                "a",
                "Opacity, from 0 (invisible) to 1 (opaque); 1 when left out.",
            ),
        ],
        returns: "A table with `r`, `g`, `b` and `a`, marked as a colour ([Color]).",
        more: "The channels are linear light, not the 0–255 of a colour picker: `0.5` looks lighter than the picker's 50% grey. The function itself takes any number, but a property default with a channel outside 0 to 1 is refused — for brighter-than-white values, declare plain numbers.\n\nTo send a colour to the Motion graph, pass its channels to [motion.color].",
        example: r#"ph2d.property("tint", ph2d.color(1, 0.4, 0.1))

function update(self, dt)
  motion.color("tint", self.tint.r, self.tint.g, self.tint.b, self.tint.a)
end"#,
    },
    // ── ph2d: o estado lateral (sobrevive à recarga) ──
    ApiHelp {
        key: "ph2d.state_get",
        params: &[
            ("entity", "Whose state, usually `self.id`."),
            ("field", "The name of the field."),
        ],
        returns: "What [ph2d.state_set] stored in that field, or `nil`. A table comes back with the same keys it had — a list is still a list.",
        more: "The state is kept outside the script, per object, for the length of a run. It survives saving a new version of the script (like `self`), and other objects can read it by id. Rewinding clears it, and so does removing the object with [ph2d.despawn].",
        example: r#"function on_signal(self, signal)
  if signal == "coin" then
    local coins = ph2d.state_get(self.id, "coins") or 0
    ph2d.state_set(self.id, "coins", coins + 1)
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.state_set",
        params: &[
            ("entity", "Whose state, usually `self.id`."),
            ("field", "The name of the field."),
            (
                "value",
                "Plain data: a number, `true`/`false`, text, or a table of those up to 16 levels deep, with text or whole-number keys. A function is an error. `nil` removes the field.",
            ),
        ],
        returns: "",
        more: "A table is copied when it is stored: changing it afterwards does not change the state until you store it again. Lists keep their order and their length (`#list` and `ipairs` work on what comes back).\n\nSetting a field to `nil` removes it from [ph2d.state_keys]; [ph2d.state_clear] removes them all.",
        example: r#"function init(self)
  ph2d.state_set(self.id, "path", { 1, 4, 9 })
end

function update(self, dt)
  local path = ph2d.state_get(self.id, "path") or {}
  print(#path)
end"#,
    },
    ApiHelp {
        key: "ph2d.state_keys",
        params: &[("entity", "Whose state, usually `self.id`.")],
        returns: "A list of the field names in alphabetical order; empty when the object has no state.",
        more: "A field set to `nil` is gone from the list.",
        example: r#"function init(self)
  for _, key in ipairs(ph2d.state_keys(self.id)) do
    print(key, ph2d.state_get(self.id, key))
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.state_clear",
        params: &[("entity", "Whose state, usually `self.id`.")],
        returns: "",
        more: "On an object without state it does nothing. Rewinding already clears every object's state, so this is for starting over in the middle of a run.",
        example: r#"function init(self)
  ph2d.state_clear(self.id)
  ph2d.state_set(self.id, "lives", 3)
end"#,
    },
    ApiHelp {
        key: "ph2d.wait",
        params: &[(
            "seconds",
            "How long to wait, in seconds of play time; 0 or left out waits until the end of this tick.",
        )],
        returns: "The seconds that actually passed — a whole number of ticks, at least the time you asked for.",
        more: "The hook stops at this line and carries on from it later, with every local variable as it was. The object keeps running meanwhile: [hooks.update] is called on every tick as usual, so a wait inside `update` starts a new wait on each tick. For a sequence (do this, wait, do that), wait in [hooks.init] or in [hooks.on_signal].\n\nThe time is play time: pausing the scene pauses the wait. A wait resumes at the end of the tick in which its time is reached, after every hook of that tick. Rewinding, removing the object, changing its script or an error drops the waits. Outside a hook — at the top of the script — it is an error; inside a coroutine you made yourself it pauses that coroutine instead, which then has to be resumed by your own code.",
        example: r#"function init(self)
  while true do
    ph2d.set(self.id, "scale_x", 1.2)
    ph2d.wait(0.5)
    ph2d.set(self.id, "scale_x", 1)
    ph2d.wait(1)
  end
end"#,
    },
    // ── motion ──
    ApiHelp {
        key: "motion.set",
        params: &[
            (
                "input",
                "The name of the input in the Motion graph. Spaces around it are dropped; an empty name is an error.",
            ),
            (
                "value",
                "The number to send; `true` is sent as 1 and `false` as 0. Text, `nil`, infinity or NaN are an error.",
            ),
        ],
        returns: "",
        more: "The value reaches the graph when the tick ends, before the Motion graph is drawn in the same frame, and it stays until a script sets it again. All scripts share one value per name, so the last write of the tick wins.\n\nA name that no input of the graph uses is not an error — nothing happens — so check the spelling against the graph.",
        example: r#"ph2d.property("max_health", 100)

function init(self)
  self.health = self.max_health
end

function on_signal(self, signal)
  if signal == "hit" then
    self.health = math.max(self.health - 10, 0)
    motion.set("health", self.health / self.max_health)
  end
end"#,
    },
    ApiHelp {
        key: "motion.color",
        params: &[
            (
                "input",
                "The name of the colour input in the Motion graph. An empty name is an error.",
            ),
            ("r", "Red, linear (1 is full)."),
            ("g", "Green, linear (1 is full)."),
            ("b", "Blue, linear (1 is full)."),
            ("a", "Opacity; 1 when left out."),
        ],
        returns: "",
        more: "It arrives like [motion.set]. It takes the channels as separate numbers, not a colour table: write `motion.color(\"tint\", c.r, c.g, c.b, c.a)` for a [ph2d.color] value. Here the channels are not limited to 0 to 1, but they must be real numbers.",
        example: r#"function update(self, dt)
  local x = ph2d.get(self.id, "x") or 0
  local warm = math.clamp(x / 10, 0, 1)
  motion.color("sky", warm, 0.3, 1 - warm)
end"#,
    },
    ApiHelp {
        key: "motion.fire",
        params: &[(
            "trigger",
            "The name of the trigger input in the Motion graph. An empty name is an error.",
        )],
        returns: "",
        more: "Every call counts as one shot, so firing twice in a tick fires twice. Unlike [motion.set], nothing stays on afterwards: the graph sees a single pulse.",
        example: r#"function on_signal(self, signal)
  if signal == "landed" then
    motion.fire("dust")
  end
end"#,
    },
    // ── os ganchos ──
    ApiHelp {
        key: "hook.init",
        params: &[(
            "self",
            "The object's own table: `self.id` and the declared properties are already in it.",
        )],
        returns: "",
        more: "It runs on the object's first tick of Play, right before its first [hooks.update], and again after every rewind. Saving a new version of the script does not run it again: the objects keep their `self` and carry on with the new code.\n\nIf `init` fails, the object stops and its `update` never runs; the console shows the error. Saving a fix makes it run again.",
        example: r#"ph2d.property("lives", 3)

function init(self)
  self.lives_left = self.lives
  self.start_x = ph2d.get(self.id, "x") or 0
end"#,
    },
    ApiHelp {
        key: "hook.update",
        params: &[
            ("self", "The object's own table, as [hooks.init] left it."),
            ("dt", "The length of one tick in seconds (1/60 by default)."),
        ],
        returns: "",
        more: "It runs once per tick while the scene plays, never while it is paused. When a frame is late, several ticks run in a row, each with the same `dt`, so the result does not depend on the frame rate: multiply speeds by `dt`.\n\nEach call has one frame (about 16.7 ms) to finish. A longer one, such as a loop that never ends, stops the object with an error instead of freezing the editor.",
        example: r#"ph2d.property("turns_per_second", 0.5)

function update(self, dt)
  local r = ph2d.get(self.id, "rotation") or 0
  ph2d.set(self.id, "rotation", r + self.turns_per_second * 2 * math.pi * dt)
end"#,
    },
    ApiHelp {
        key: "hook.on_signal",
        params: &[
            ("self", "The object's own table."),
            ("signal", "The name of the signal, as it was sent."),
        ],
        returns: "",
        more: "It runs once per signal and object, after the frame's ticks, only while the scene plays and only for objects that have already had their first tick. Signals come from [ph2d.emit] in any script (one frame later, the sender's own included) and from the rest of the scene.\n\nEvery object with this hook hears every signal, so check the name and ignore the rest.",
        example: r#"function on_signal(self, signal)
  if signal == "jump" then
    local y = ph2d.get(self.id, "y") or 0
    ph2d.set(self.id, "y", y + 1)
  end
end"#,
    },
    // ── os tipos ──
    ApiHelp {
        key: "type.Entity",
        params: &[],
        returns: "",
        more: "It is a plain number that is only valid while the editor is open: pass it to the functions that take an object, but don't do maths on it or rely on it after reopening the project.",
        example: r#"function init(self)
  print("my id is", self.id)
end"#,
    },
    ApiHelp {
        key: "type.PoseField",
        params: &[],
        returns: "",
        more: "`x` and `y` are the position relative to the parent, `rotation` is in radians counter-clockwise, and `scale_x` and `scale_y` multiply the size (1 is the original). Any other name stops the object, with [ph2d.get] and with [ph2d.set]; the editor underlines it as you type.",
        example: r#"function update(self, dt)
  local s = ph2d.get(self.id, "scale_x") or 1
  ph2d.set(self.id, "scale_x", s + dt)
end"#,
    },
    ApiHelp {
        key: "type.Vec2",
        params: &[],
        returns: "",
        more: "Build it with [ph2d.vec2]. A position property arrives in `self` in the same shape, so read `.x` and `.y`.",
        example: r#"ph2d.property("offset", ph2d.vec2(1, 0))

function init(self)
  print(self.offset.x, self.offset.y)
end"#,
    },
    ApiHelp {
        key: "type.Color",
        params: &[],
        returns: "",
        more: "Build it with [ph2d.color]. A colour property arrives in `self` in the same shape, so read `.r`, `.g`, `.b` and `.a`.",
        example: r#"ph2d.property("tint", ph2d.color(0.2, 0.6, 1))

function init(self)
  print(self.tint.r, self.tint.g, self.tint.b, self.tint.a)
end"#,
    },
    ApiHelp {
        key: "type.PropertyDefault",
        params: &[],
        returns: "",
        more: "A number gives a number field, `true`/`false` a checkbox, text a text field (or a list with `options`), [ph2d.vec2] two fields and [ph2d.color] a colour swatch. Any other value is refused.",
        example: r#"ph2d.property("speed", 2)
ph2d.property("loop", true)
ph2d.property("label", "hello")
ph2d.property("start", ph2d.vec2(0, 0))
ph2d.property("tint", ph2d.color(1, 1, 1))"#,
    },
    ApiHelp {
        key: "type.PropertyOptions",
        params: &[],
        returns: "",
        more: "`min`, `max` and `step` work on numbers and positions (both parts share them); `step` must be above 0 and `min` not above `max`. `options` is a list of different, non-empty texts for a text property, and its default must be one of them. Colours take none of these.",
        example: r#"ph2d.property("jump", 4, { min = 0, max = 10, step = 0.5 })
ph2d.property("side", "left", { options = { "left", "right" } })"#,
    },
    ApiHelp {
        key: "type.StateValue",
        params: &[],
        returns: "",
        more: "Tables keep their keys as they were — text or whole numbers — so a list comes back as a list. Functions are refused.",
        example: r#"function init(self)
  ph2d.state_set(self.id, "best", { score = 120, name = "Ana" })
end"#,
    },
    ApiHelp {
        key: "type.KeyName",
        params: &[],
        returns: "",
        more: "Letters and digits are their own name in lower case; the arrows are \"up\", \"down\", \"left\" and \"right\"; \"shift\", \"ctrl\" and \"alt\" mean either side, and \"left_shift\", \"right_shift\", \"left_ctrl\", \"right_ctrl\", \"left_alt\" and \"right_alt\" one side only.",
        example: r#"function update(self, dt)
  if ph2d.key_down("shift") and ph2d.key_pressed("r") then
    print("run")
  end
end"#,
    },
    ApiHelp {
        key: "type.MouseButtonName",
        params: &[],
        returns: "",
        more: "The three buttons of a mouse.",
        example: r#"function update(self, dt)
  if ph2d.mouse_pressed("right") then
    print("menu")
  end
end"#,
    },
    ApiHelp {
        key: "type.ScriptSelf",
        params: &[],
        returns: "",
        more: "Each object has its own `self`, made on its first tick and made again after a rewind. Whatever you store in it stays until then, and saving a new version of the script keeps it.",
        example: r#"function update(self, dt)
  self.time = (self.time or 0) + dt
end"#,
    },
];

/// O texto da entrada `key`.
#[must_use]
pub fn api_help(key: &str) -> Option<&'static ApiHelp> {
    API_HELP.iter().find(|h| h.key == key)
}
