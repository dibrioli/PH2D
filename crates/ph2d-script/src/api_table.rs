//! ⭐⭐ **A API Luau dos scripts — a FONTE ÚNICA** (CodeEditor Onda 0, T2; ADR-0188 §3).
//!
//! Cada global e cada função que a fiação do [`crate::ScriptHost`] instala tem aqui UMA linha, com o
//! tipo Luau lido da conversão de argumentos de quem a instala (`host.rs`, `motion_api.rs`,
//! `module.rs`, `valores.rs`, o `ph2d.wait` do `ScriptRuntime`). Dela sai o `runtime/luau/ph2d.d.luau`
//! ([`luau_definitions`], escrito pelo `tools/ph2d-bindgen`).
//!
//! Gate nos DOIS sentidos: `tests/it/a_api_e_a_tabela.rs` monta a VM pela mesma fiação e compara as
//! chaves — função sem linha aqui ⇒ vermelho; linha aqui sem função ⇒ vermelho.
//!
//! ⚠️ Cada `ident` é um IDENTIFICADOR Luau — código que o `.d.luau` declara e o script escreve, nunca
//! língua do ecrã; por isso não se chama `name`, o campo que o censo da fronteira lê como rótulo.

/// Um global que a fiação instala (uma tabela de funções).
pub struct ApiGlobal {
    pub ident: &'static str,
    pub doc: &'static str,
}

/// Uma função de um global.
pub struct ApiEntry {
    /// O global a que pertence (`"ph2d"` · `"motion"`).
    pub table: &'static str,
    pub ident: &'static str,
    /// O tipo Luau da função, `(args) -> ret`.
    pub luau_type: &'static str,
    /// Uma linha.
    pub doc: &'static str,
}

/// Um tipo exportado que as assinaturas usam.
pub struct ApiType {
    pub ident: &'static str,
    pub definition: &'static str,
    pub doc: &'static str,
}

/// Um gancho: função GLOBAL do script que o MOTOR chama (`ScriptModule::hook`).
pub struct ApiHook {
    pub ident: &'static str,
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
        ident: "init",
        luau_type: "(self: ScriptSelf) -> ()",
        doc: "Once, before the first update, with the properties already in `self`.",
    },
    ApiHook {
        ident: "update",
        luau_type: "(self: ScriptSelf, dt: number) -> ()",
        doc: "Every fixed tick; `dt` in seconds.",
    },
    ApiHook {
        ident: "on_signal",
        luau_type: "(self: ScriptSelf, signal: string) -> ()",
        doc: "For each signal heard this tick, by name.",
    },
];

pub const API_GLOBALS: &[ApiGlobal] = &[
    ApiGlobal {
        ident: "ph2d",
        doc: "The PH2D scene API: move objects, read input, signals, properties, values.",
    },
    ApiGlobal {
        ident: "motion",
        doc: "Writes the inputs that the Motion graph reads (`value.input`).",
    },
];

pub const API_TYPES: &[ApiType] = &[
    ApiType {
        ident: "Entity",
        definition: "number",
        doc: "A scene object, as `self.id` and `ph2d.find_by_name` give it.",
    },
    ApiType {
        ident: "PoseField",
        definition: "\"x\" | \"y\" | \"rotation\" | \"scale_x\" | \"scale_y\"",
        doc: "The pose fields a scene script reads and writes.",
    },
    ApiType {
        ident: "Vec2",
        definition: "{ __kind: \"vec2\", x: number, y: number }",
        doc: "A position, as `ph2d.vec2` builds it and `self` reads it.",
    },
    ApiType {
        ident: "Color",
        definition: "{ __kind: \"color\", r: number, g: number, b: number, a: number }",
        doc: "A linear RGBA colour, as `ph2d.color` builds it and `self` reads it.",
    },
    ApiType {
        ident: "PropertyDefault",
        definition: "number | boolean | string | Vec2 | Color",
        doc: "What a property may default to; the default decides its kind.",
    },
    ApiType {
        ident: "PropertyOptions",
        definition: "{ min: number?, max: number?, step: number?, options: { string }? }",
        doc: "Editing hints for a property; `options` makes a text property a list.",
    },
    ApiType {
        ident: "StateValue",
        definition: "boolean | number | string | { [string]: StateValue }",
        doc: "Plain data that survives a reload in the per-object state table.",
    },
    // `born` cria o `self` só com o `id`; os números entram pelos nomes das propriedades e o script
    // guarda o que quiser ⇒ o indexador `any` é a verdade, não preguiça. O `luau-lsp` 1.70.1 expõe um
    // `export type` do ficheiro de definições como tipo global dos scripts (medido 15/10).
    ApiType {
        ident: "ScriptSelf",
        definition: "{ id: Entity, [string]: any }",
        doc: "The `self` a hook receives: its `id`, the declared properties and whatever the script keeps.",
    },
];

pub const API: &[ApiEntry] = &[
    // ── ph2d: o objecto e a cena ──
    ApiEntry {
        table: "ph2d",
        ident: "set",
        luau_type: "(entity: Entity, field: PoseField, value: number) -> ()",
        doc: "Writes a pose field; applied at the end of the tick (everyone reads the same photo).",
    },
    ApiEntry {
        table: "ph2d",
        ident: "get",
        luau_type: "(entity: Entity, field: PoseField) -> number?",
        doc: "Reads a pose field as it was at the start of the tick; nil if unknown.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "input",
        luau_type: "(key: string) -> number?",
        doc: "The value of an input key or axis this frame; nil when not held.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "emit",
        luau_type: "(signal: string) -> ()",
        doc: "Emits a signal from this object; an empty name is an error.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "find_by_name",
        luau_type: "(name: string) -> Entity?",
        doc: "The object with this name, if there is one.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "spawn",
        luau_type: "() -> ()",
        doc: "Queues an empty object, created between ticks.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "spawn_named",
        luau_type: "(name: string) -> ()",
        doc: "Queues a named object; `find_by_name` finds it from the next tick.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "despawn",
        luau_type: "(entity: Entity) -> ()",
        doc: "Queues the removal of an object.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "attach_script",
        luau_type: "(entity: Entity, script: string) -> ()",
        doc: "Queues attaching a project script, named by its name in the project (e.g. \"bob.luau\").",
    },
    // ── ph2d: as declarações e os valores ──
    ApiEntry {
        table: "ph2d",
        ident: "property",
        luau_type: "(name: string, default: PropertyDefault, options: PropertyOptions?) -> ()",
        doc: "Declares an Inspector property; only at the top of the script.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "vec2",
        luau_type: "(x: number, y: number) -> Vec2",
        doc: "A position value.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "color",
        luau_type: "(r: number, g: number, b: number, a: number?) -> Color",
        doc: "A linear RGBA colour value; opaque when `a` is missing.",
    },
    // ── ph2d: o estado lateral (sobrevive à recarga) ──
    ApiEntry {
        table: "ph2d",
        ident: "state_get",
        luau_type: "(entity: Entity, field: string) -> StateValue?",
        doc: "Reads a field of the object's lateral state.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "state_set",
        luau_type: "(entity: Entity, field: string, value: StateValue?) -> ()",
        doc: "Writes a field of the object's lateral state; only plain data.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "state_keys",
        luau_type: "(entity: Entity) -> { string }",
        doc: "The fields of the object's lateral state, sorted.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "state_clear",
        luau_type: "(entity: Entity) -> ()",
        doc: "Drops every field of the object's lateral state.",
    },
    ApiEntry {
        table: "ph2d",
        ident: "wait",
        luau_type: "(seconds: number) -> ()",
        doc: "Yields the running coroutine for `seconds`; scene hooks are not coroutines.",
    },
    // ── motion ──
    ApiEntry {
        table: "motion",
        ident: "set",
        luau_type: "(input: string, value: number | boolean) -> ()",
        doc: "Sets a number input of the Motion graph (true/false read as 1/0).",
    },
    ApiEntry {
        table: "motion",
        ident: "color",
        luau_type: "(input: string, r: number, g: number, b: number, a: number?) -> ()",
        doc: "Sets a colour input of the Motion graph, linear RGBA.",
    },
    ApiEntry {
        table: "motion",
        ident: "fire",
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
            t.doc, t.ident, t.definition
        ));
    }
    for g in API_GLOBALS {
        out.push_str(&format!("\n--- {}\ndeclare {}: {{\n", g.doc, g.ident));
        for e in API.iter().filter(|e| e.table == g.ident) {
            out.push_str(&format!(
                "    --- {}\n    {}: {},\n",
                e.doc, e.ident, e.luau_type
            ));
        }
        out.push_str("}\n");
    }
    out.push_str("\n-- Engine hooks: global functions of a script that the engine calls.\n");
    for h in HOOKS {
        out.push_str(&format!(
            "--- {}\n-- @hook {}: {}\n",
            h.doc, h.ident, h.luau_type
        ));
    }
    out
}

/// O pacote com que o editor carrega o [`luau_definitions`] (`--definitions:@ph2d=…`): os símbolos da
/// base de documentação começam por ele (`@ph2d/global/ph2d.set`, medido em
/// `docs/CodeEditor/ferramentas/medicoes_onda2/I0b_luau_lsp_respostas.md` §3).
pub const DEFINITIONS_PACKAGE: &str = "@ph2d";

/// Os parâmetros `(nome, tipo)` de um tipo de função `(a: T, b: U?) -> R`, separados nas vírgulas
/// de topo (um `{ string }` ou um `(x) -> y` dentro de um tipo não parte).
#[must_use]
pub fn luau_params(luau_type: &str) -> Vec<(&str, &str)> {
    let Some(body) = luau_type.strip_prefix('(') else {
        return Vec::new();
    };
    let mut depth = 0usize;
    let mut end = body.len();
    let mut cuts = vec![0];
    for (i, c) in body.char_indices() {
        match c {
            '(' | '{' | '[' | '<' => depth += 1,
            ')' if depth == 0 => {
                end = i;
                break;
            }
            ')' | '}' | ']' | '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => cuts.push(i + 1),
            _ => {}
        }
    }
    cuts.push(end + 1);
    cuts.windows(2)
        .map(|w| body[w[0]..w[1] - 1].trim())
        .filter(|p| !p.is_empty())
        .map(|p| match p.split_once(':') {
            Some((n, t)) => (n.trim(), t.trim()),
            None => ("", p),
        })
        .collect()
}

/// ⭐ **A base de documentação do `luau-lsp`** (`--docs=<json>`), gerada da tabela: a documentação
/// de cada global e de cada função, e os parâmetros pelo NOME. A de um parâmetro só chega ao
/// servidor como SÍMBOLO de outra entrada (`…/param/<i>`; texto directo ali sai vazio — I0b §3), e
/// o texto é o do tipo exportado que ele usa (`entity: Entity` ⇒ a linha do `Entity`).
#[must_use]
pub fn luau_docs() -> String {
    use serde_json::{Map, Value, json};
    let type_doc = |ty: &str| {
        let ty = ty.trim_end_matches('?');
        API_TYPES.iter().find(|t| t.ident == ty).map(|t| t.doc)
    };
    let mut m = Map::new();
    for g in API_GLOBALS {
        m.insert(
            format!("{DEFINITIONS_PACKAGE}/global/{}", g.ident),
            json!({ "documentation": g.doc }),
        );
    }
    for e in API {
        let sym = format!("{DEFINITIONS_PACKAGE}/global/{}.{}", e.table, e.ident);
        let mut params = Vec::new();
        for (i, (name, ty)) in luau_params(e.luau_type).into_iter().enumerate() {
            let doc = match type_doc(ty) {
                Some(d) => {
                    let p = format!("{sym}/param/{i}");
                    m.insert(p.clone(), json!({ "documentation": d }));
                    p
                }
                None => String::new(),
            };
            params.push(json!({ "name": name, "documentation": doc }));
        }
        m.insert(sym, json!({ "documentation": e.doc, "params": params }));
    }
    serde_json::to_string_pretty(&Value::Object(m)).expect("JSON de valores simples")
}
