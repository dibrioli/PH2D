//! **As páginas de ajuda das propriedades dos componentes** (doc CodeEditor 08) — o irmão do
//! [`crate::api_help`] pelo tecto de LOC; lidas pela mesma [`crate::api_help::all_help`].

use crate::api_help::ApiHelp;

pub(crate) const PROPERTY_HELP: &[ApiHelp] = &[
    ApiHelp {
        key: "ph2d.properties",
        params: &[("entity", "Any object, such as `self.id`.")],
        returns: "The names, the pose first and then each property of each component the object has.",
        more: "Each name is what the Inspector shows, in snake_case, after the component's name: the label «Max» of the card «Health» is \"health.max\".",
        example: r#"function init(self)
  for _, name in ph2d.properties(self.id) do
    print(name, ph2d.get(self.id, name))
  end
end"#,
    },
    ApiHelp {
        key: "type.PropertyName",
        params: &[],
        returns: "",
        more: "The card's title and the field's label, in snake_case and joined by a dot: «Game Camera» › «Height» is \"game_camera.height\", «Health» › «Regen / s» is \"health.regen\" (the unit at the end stays out). The editor suggests them inside the quotes and underlines a name no component has. A field that points at something else takes its NAME: an object's name (\"camera_follow.target\"), a tag's path (\"Enemy/Flying\"; \"tags\" is a list of them), a sorting layer's name; nil = nobody, and a name nothing has is an error. A choice with rows under it (the agent's «Target», the factory's «Where») has one name per row: writing \"nav_agent.object\" also picks «Object», and a row of a choice not in use reads nil.",
        example: r#"function update(self, dt)
  local max = ph2d.get(self.id, "health.max")
  ph2d.set(self.id, "health.max", max + dt)
end"#,
    },
    ApiHelp {
        key: "ph2d.damage",
        params: &[
            (
                "entity",
                "An object with a Health component, such as `self.id`.",
            ),
            ("amount", "How many points to take, above 0."),
        ],
        returns: "",
        more: "It is a hit like any other: invincibility, dodge, armor and shield apply, the object blinks, the hit numbers show and at 0 it dies (its On Damage and On Death signals fire). It lands on the next tick of the run. To put the health at a number without any of that, use [ph2d.set] with \"health.now\".",
        example: r#"function on_signal(self, signal)
  if signal == "lava" then
    ph2d.damage(self.id, 10)
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.heal",
        params: &[
            (
                "entity",
                "An object with a Health component, such as `self.id`.",
            ),
            ("amount", "How many points to give back, above 0."),
        ],
        returns: "",
        more: "It stops at the Max unless Overheal is on, and a dead object is not healed. Its On Heal signal fires. It lands on the next tick of the run.",
        example: r#"function on_signal(self, signal)
  if signal == "potion" then
    ph2d.heal(self.id, 25)
  end
end"#,
    },
];
