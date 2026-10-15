//! ⭐⭐ **A API Luau dos scripts — a FONTE ÚNICA** (CodeEditor Onda 0, T2; ADR-0188 §3).
//!
//! Cada global e cada função que a fiação do [`crate::ScriptHost`] instala tem aqui UMA linha, com o
//! tipo Luau lido da conversão de argumentos de quem a instala (`host.rs`, `motion_api.rs`,
//! `module.rs`, `valores.rs`, o `ph2d.wait` do `ScriptRuntime`). Dela sai o `runtime/luau/ph2d.d.luau`
//! ([`luau_definitions`], escrito pelo `tools/ph2d-bindgen`).
//!
//! Gate nos DOIS sentidos: `tests/it/a_api_e_a_tabela.rs` monta a VM pela mesma fiação e compara as
//! chaves — função sem linha aqui ⇒ vermelho; linha aqui sem função ⇒ vermelho.

/// Um global que a fiação instala (uma tabela de funções).
pub struct ApiGlobal {
    pub name: &'static str,
    pub doc: &'static str,
}

/// Uma função de um global.
pub struct ApiEntry {
    /// O global a que pertence (`"ph2d"` · `"motion"`).
    pub table: &'static str,
    pub name: &'static str,
    /// O tipo Luau da função, `(args) -> ret`.
    pub luau_type: &'static str,
    /// Uma linha.
    pub doc: &'static str,
}

/// Um tipo exportado que as assinaturas usam.
pub struct ApiType {
    pub name: &'static str,
    pub definition: &'static str,
    pub doc: &'static str,
}

/// Um gancho: função GLOBAL do script que o MOTOR chama (`ScriptModule::hook`).
pub struct ApiHook {
    pub name: &'static str,
    /// A assinatura com que a cena o chama.
    pub luau_type: &'static str,
    pub doc: &'static str,
}

/// Os ganchos, lidos de `scene.rs` (`tick` / `hear`); o teste da tabela compara-os com os
/// `.hook("…")` do fonte. Saem no `ph2d.d.luau` como linhas `-- @hook <nome>: <tipo>` — comentário,
/// porque DECLARÁ-los como globais faz o `luau-lsp` acusar `BuiltinGlobalWrite` em cada script
/// (medido 15/10); o `ph2d-code-lsp` lê essas linhas para não acusar `FunctionUnused` neles.
pub const HOOKS: &[ApiHook] = &[
    ApiHook {
        name: "init",
        luau_type: "(self: ScriptSelf) -> ()",
        doc: "Once, before the first update, with the properties already in `self`.",
    },
    ApiHook {
        name: "update",
        luau_type: "(self: ScriptSelf, dt: number) -> ()",
        doc: "Every fixed tick; `dt` in seconds.",
    },
    ApiHook {
        name: "on_signal",
        luau_type: "(self: ScriptSelf, signal: string) -> ()",
        doc: "For each signal heard this tick, by name.",
    },
];

pub const API_GLOBALS: &[ApiGlobal] = &[
    ApiGlobal {
        name: "ph2d",
        doc: "The PH2D scene API: move objects, read input, signals, properties, values.",
    },
    ApiGlobal {
        name: "motion",
        doc: "Writes the inputs that the Motion graph reads (`value.input`).",
    },
];

pub const API_TYPES: &[ApiType] = &[
    ApiType {
        name: "Entity",
        definition: "number",
        doc: "A scene object, as `self.id` and `ph2d.find_by_name` give it.",
    },
    ApiType {
        name: "PoseField",
        definition: "\"x\" | \"y\" | \"rotation\" | \"scale_x\" | \"scale_y\"",
        doc: "The pose fields a scene script reads and writes.",
    },
    ApiType {
        name: "Vec2",
        definition: "{ __kind: \"vec2\", x: number, y: number }",
        doc: "A position, as `ph2d.vec2` builds it and `self` reads it.",
    },
    ApiType {
        name: "Color",
        definition: "{ __kind: \"color\", r: number, g: number, b: number, a: number }",
        doc: "A linear RGBA colour, as `ph2d.color` builds it and `self` reads it.",
    },
    ApiType {
        name: "PropertyDefault",
        definition: "number | boolean | string | Vec2 | Color",
        doc: "What a property may default to; the default decides its kind.",
    },
    ApiType {
        name: "PropertyOptions",
        definition: "{ min: number?, max: number?, step: number?, options: { string }? }",
        doc: "Editing hints for a property; `options` makes a text property a list.",
    },
    ApiType {
        name: "StateValue",
        definition: "boolean | number | string | { [string]: StateValue }",
        doc: "Plain data that survives a reload in the per-object state table.",
    },
    // `born` cria o `self` só com o `id`; os números entram pelos nomes das propriedades e o script
    // guarda o que quiser ⇒ o indexador `any` é a verdade, não preguiça. O `luau-lsp` 1.70.1 expõe um
    // `export type` do ficheiro de definições como tipo global dos scripts (medido 15/10).
    ApiType {
        name: "ScriptSelf",
        definition: "{ id: Entity, [string]: any }",
        doc: "The `self` a hook receives: its `id`, the declared properties and whatever the script keeps.",
    },
];

pub const API: &[ApiEntry] = &[
    // ── ph2d: o objecto e a cena ──
    ApiEntry {
        table: "ph2d",
        name: "set",
        luau_type: "(entity: Entity, field: PoseField, value: number) -> ()",
        doc: "Writes a pose field; applied at the end of the tick (everyone reads the same photo).",
    },
    ApiEntry {
        table: "ph2d",
        name: "get",
        luau_type: "(entity: Entity, field: PoseField) -> number?",
        doc: "Reads a pose field as it was at the start of the tick; nil if unknown.",
    },
    ApiEntry {
        table: "ph2d",
        name: "input",
        luau_type: "(key: string) -> number?",
        doc: "The value of an input key or axis this frame; nil when not held.",
    },
    ApiEntry {
        table: "ph2d",
        name: "emit",
        luau_type: "(signal: string) -> ()",
        doc: "Emits a signal from this object; an empty name is an error.",
    },
    ApiEntry {
        table: "ph2d",
        name: "find_by_name",
        luau_type: "(name: string) -> Entity?",
        doc: "The object with this name, if there is one.",
    },
    ApiEntry {
        table: "ph2d",
        name: "spawn",
        luau_type: "() -> ()",
        doc: "Queues an empty object, created between ticks.",
    },
    ApiEntry {
        table: "ph2d",
        name: "spawn_named",
        luau_type: "(name: string) -> ()",
        doc: "Queues a named object; `find_by_name` finds it from the next tick.",
    },
    ApiEntry {
        table: "ph2d",
        name: "despawn",
        luau_type: "(entity: Entity) -> ()",
        doc: "Queues the removal of an object.",
    },
    ApiEntry {
        table: "ph2d",
        name: "attach_script",
        luau_type: "(entity: Entity, bytecode_id: string) -> ()",
        doc: "Queues attaching a compiled script, named by its 64-character hex id.",
    },
    // ── ph2d: as declarações e os valores ──
    ApiEntry {
        table: "ph2d",
        name: "property",
        luau_type: "(name: string, default: PropertyDefault, options: PropertyOptions?) -> ()",
        doc: "Declares an Inspector property; only at the top of the script.",
    },
    ApiEntry {
        table: "ph2d",
        name: "vec2",
        luau_type: "(x: number, y: number) -> Vec2",
        doc: "A position value.",
    },
    ApiEntry {
        table: "ph2d",
        name: "color",
        luau_type: "(r: number, g: number, b: number, a: number?) -> Color",
        doc: "A linear RGBA colour value; opaque when `a` is missing.",
    },
    // ── ph2d: o estado lateral (sobrevive à recarga) ──
    ApiEntry {
        table: "ph2d",
        name: "state_get",
        luau_type: "(entity: Entity, field: string) -> StateValue?",
        doc: "Reads a field of the object's lateral state.",
    },
    ApiEntry {
        table: "ph2d",
        name: "state_set",
        luau_type: "(entity: Entity, field: string, value: StateValue?) -> ()",
        doc: "Writes a field of the object's lateral state; only plain data.",
    },
    ApiEntry {
        table: "ph2d",
        name: "state_keys",
        luau_type: "(entity: Entity) -> { string }",
        doc: "The fields of the object's lateral state, sorted.",
    },
    ApiEntry {
        table: "ph2d",
        name: "state_clear",
        luau_type: "(entity: Entity) -> ()",
        doc: "Drops every field of the object's lateral state.",
    },
    ApiEntry {
        table: "ph2d",
        name: "wait",
        luau_type: "(seconds: number) -> ()",
        doc: "Yields the running coroutine for `seconds`; scene hooks are not coroutines.",
    },
    // ── motion ──
    ApiEntry {
        table: "motion",
        name: "set",
        luau_type: "(input: string, value: number | boolean) -> ()",
        doc: "Sets a number input of the Motion graph (true/false read as 1/0).",
    },
    ApiEntry {
        table: "motion",
        name: "color",
        luau_type: "(input: string, r: number, g: number, b: number, a: number?) -> ()",
        doc: "Sets a colour input of the Motion graph, linear RGBA.",
    },
    ApiEntry {
        table: "motion",
        name: "fire",
        luau_type: "(trigger: string) -> ()",
        doc: "Fires a trigger input of the Motion graph.",
    },
];

/// ⭐ **O `ph2d.d.luau`** — as definições que o `luau-lsp` carrega, geradas da tabela.
#[must_use]
pub fn luau_definitions() -> String {
    let mut out = String::from(
        "-- ============================================================\n\
         -- AUTO-GENERATED by tools/ph2d-bindgen — DO NOT EDIT BY HAND.\n\
         --\n\
         -- Source of truth: crates/ph2d-script/src/api_table.rs\n\
         -- Regenerate:      cargo run -p ph2d-bindgen -- --write\n\
         -- ============================================================\n",
    );
    for t in API_TYPES {
        out.push_str(&format!(
            "\n--- {}\nexport type {} = {}\n",
            t.doc, t.name, t.definition
        ));
    }
    for g in API_GLOBALS {
        out.push_str(&format!("\n--- {}\ndeclare {}: {{\n", g.doc, g.name));
        for e in API.iter().filter(|e| e.table == g.name) {
            out.push_str(&format!(
                "    --- {}\n    {}: {},\n",
                e.doc, e.name, e.luau_type
            ));
        }
        out.push_str("}\n");
    }
    out.push_str("\n-- Engine hooks: global functions of a script that the engine calls.\n");
    for h in HOOKS {
        out.push_str(&format!("--- {}\n-- @hook {}: {}\n", h.doc, h.name, h.luau_type));
    }
    out
}
