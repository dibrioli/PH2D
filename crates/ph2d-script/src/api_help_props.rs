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
    // O raio (GEF 07, RS-7/RS-8).
    ApiHelp {
        key: "ph2d.ray",
        params: &[(
            "entity",
            "A Ray Sensor object, such as `ph2d.object(\"Eye Ray\")`.",
        )],
        returns: "A RayView: `seeing`, the list `hits` (nearest first) and the nearest hit's fields; nil when it is not a Ray Sensor.",
        more: "It reads the ray as it looked on this frame, with every option of its card (Filter, Thickness, Cone, Aim at). With See all hits or a Cone, `hits` has one entry per object it sees. `tags` are the tags of the collision shape it hit, so a shape tagged \"Ice\" tells the ray what it stands on. An Enabled box that is off reads `seeing = false`.",
        example: r#"function update(self, dt)
  local eye = ph2d.ray(ph2d.object("Eye Ray"))
  if eye and eye.seeing then
    print("I see", eye.object, "at", eye.distance)
  end
end"#,
    },
    ApiHelp {
        key: "ph2d.raycast",
        params: &[
            ("from", "Where the ray starts, such as `ph2d.vec2(0, 1)`."),
            (
                "to",
                "Where it ends: its reach is the distance between the two.",
            ),
            (
                "filter",
                "The Ray Sensor's options, by the names its card has: `{ only_see = \"only_tag\", tag = \"Enemy\" }`. Leave it out to see everything but yourself.",
            ),
        ],
        returns: "The hits, nearest first; the first one is `[1]`, and an empty list means nothing was hit.",
        more: "It is the same ray as the Ray Sensor object, answered on the spot: an instant shot (hitscan). It ignores your own body unless `ignore_my_body = false`. Without `see_all_hits` or a `cone` it gives at most one hit. Angles (`cone`) are in radians, like every angle in scripts.",
        example: r#"function on_signal(self, signal)
  if signal == "fire" then
    local from = ph2d.vec2(ph2d.get(self.id, "x"), ph2d.get(self.id, "y"))
    local hit = ph2d.raycast(from, ph2d.vec2(from.x + 10, from.y), { only_see = "only_tag", tag = "Enemy" })[1]
    if hit then
      ph2d.damage(hit.object, 10)
    end
  end
end"#,
    },
    ApiHelp {
        key: "type.RayHit",
        params: &[],
        returns: "",
        more: "`object` is the one with the body; `shape` is the collision shape that was hit, and `tags` are that shape's tags (its «material»). `friction` and `bounce` are the shape's.",
        example: r#"function update(self, dt)
  local eye = ph2d.ray(self.id)
  if eye and eye.seeing and table.find(eye.tags or {}, "Ice") then
    print("slippery!")
  end
end"#,
    },
    ApiHelp {
        key: "type.RayView",
        params: &[],
        returns: "",
        more: "`seeing` is true while the ray sees something; the nearest hit's fields sit at the top so `ph2d.ray(id).object` reads directly.",
        example: r#"function update(self, dt)
  local eye = ph2d.ray(self.id)
  if eye then
    print(eye.seeing, #eye.hits)
  end
end"#,
    },
    ApiHelp {
        key: "type.RayFilter",
        params: &[],
        returns: "",
        more: "Every name is optional and means what the Ray Sensor card's field of the same name means; names of objects and tags go by name.",
        example: r#"function on_signal(self, signal)
  local hits = ph2d.raycast(ph2d.vec2(0, 0), ph2d.vec2(0, -5), { see_all_hits = true, also_ignore = { "Crate" } })
  print(#hits)
end"#,
    },
];
