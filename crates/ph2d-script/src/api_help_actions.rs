//! **As páginas de ajuda das acções do Input Map** (`ph2d.action_*`, plano InputMap §5) — irmão do
//! [`crate::api_help`] pelo tecto de LOC; o texto diz o que `action_api.rs` faz (portões em
//! `scene_action_tests.rs`).

use crate::api_help::ApiHelp;

pub(crate) const ACTION_HELP: &[ApiHelp] = &[
    ApiHelp {
        key: "ph2d.action_down",
        params: &[(
            "action",
            r#"The name of an action in the Input Map, such as "jump" or "move_left"."#,
        )],
        returns: "`true` while any key or button of the action is held.",
        more: "An action is a name the game reads, with the keys and gamepad buttons that make it. Read the action, not the key: when you or a player change the key of \"jump\" in the Input Map, the script keeps working.\n\nA name the Input Map does not have is an error that stops the object and lists the actions it has, so a typo cannot read as an action nobody presses.",
        example: r#"ph2d.property("speed", 4)

function update(self, dt)
  local x = ph2d.get(self.id, "x") or 0
  if ph2d.action_down("move_left") then x = x - self.speed * dt end
  if ph2d.action_down("move_right") then x = x + self.speed * dt end
  ph2d.set(self.id, "x", x)
end"#,
    },
    ApiHelp {
        key: "ph2d.action_pressed",
        params: &[("action", "An action name, as in [ph2d.action_down].")],
        returns: "`true` on the first tick after the action went down, `false` on every other tick.",
        more: "Use it for things that should happen once per press, like a jump or a shot. When a frame runs two ticks, only the first one sees it.",
        example: r#"function update(self, dt)
  if ph2d.action_pressed("jump") then
    local y = ph2d.get(self.id, "y") or 0
    ph2d.set(self.id, "y", y + 2)
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.action_released",
        params: &[("action", "An action name, as in [ph2d.action_down].")],
        returns: "`true` on the first tick after the action went up, `false` on every other tick.",
        more: "It pairs with [ph2d.action_pressed], for example to charge a shot while the action is held.",
        example: r#"function update(self, dt)
  if ph2d.action_down("jump") then self.charge = (self.charge or 0) + dt end
  if ph2d.action_released("jump") then
    print("charged", self.charge)
    self.charge = 0
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.action_strength",
        params: &[("action", "An action name, as in [ph2d.action_down].")],
        returns: "A number from `0` to `1`: `0` when nothing pushes the action, `1` when it is pushed all the way.",
        more: "A key gives `0` or `1`. A stick or a trigger gives the values between, after the action's dead zone in the Input Map: a small wobble of the stick reads `0`.",
        example: r#"ph2d.property("speed", 4)

function update(self, dt)
  local push = ph2d.action_strength("move_right") - ph2d.action_strength("move_left")
  local x = ph2d.get(self.id, "x") or 0
  ph2d.set(self.id, "x", x + push * self.speed * dt)
end"#,
    },
];
