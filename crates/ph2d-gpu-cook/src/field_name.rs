//! **Naming a kernel param's uniform field** — the one place a param name is
//! translated before it becomes WGSL.
//!
//! A `ParamSpec.name` is authored for the ARTIST and lives in saved documents, so
//! it can never be renamed to suit the shader. But two kinds of name cannot be
//! emitted verbatim into the generated uniform struct:
//!
//! - a **WGSL reserved word** — `motion.noise` calls its fractal-sum selector
//!   `type`, and naga rejects `type: f32` outright;
//! - a field the **engine already owns** — `motion.pin_constraint` has a param
//!   called `count`, and the engine writes `count` into every uniform block, so
//!   the struct would declare it twice.
//!
//! Both took the node out of GPU reach entirely (the second was an explicit
//! refusal in `plan::eligible`). Both are answered the same way: the field takes a
//! trailing underscore and the kernel body says `params.type_` / `params.count_`.
//! No shipped kernel's params collide, so this is a no-op for all of them.

/// Field names the generated module **already owns** — the engine writes these
/// into every uniform block (`count`/`playhead` always, the rest conditionally).
/// A param of the same name would declare the field twice, so it takes the same
/// underscore as a reserved word. `motion.pin_constraint` really does have a param
/// called `count`.
const ENGINE_UNIFORMS: &[&str] = &[
    "count",
    "playhead",
    "gather_prev_n",
    "window_first",
    "window_age",
];

/// WGSL **reserved words** that a node's param name could plausibly collide with.
///
/// A `ParamSpec.name` is authored for the ARTIST (`motion.noise` calls its
/// fractal-sum selector `type`), and the generated uniform struct uses that name
/// verbatim — so `type: f32` is emitted, naga rejects it (*"name `type` is a
/// reserved keyword"*), and the node simply cannot have a kernel. The param may
/// not be renamed: it is the artist's vocabulary and it is in saved documents.
///
/// So the FIELD gets a trailing underscore and the kernel body says `params.type_`.
/// None of the shipped kernels' params collide, so this is a no-op for every one
/// of them (there is a gate).
///
/// The list is the plausible subset, not all ~150 reserved words, and that is safe
/// **because of where the backstop is**: `generated_wgsl_validates` parses every
/// registered kernel's module under naga on every CI lane, so a word this list
/// misses fails there — loudly, at `cargo test`, not at first dispatch on somebody
/// else's GPU.
const WGSL_RESERVED: &[&str] = &[
    "type",
    "var",
    "let",
    "const",
    "fn",
    "struct",
    "if",
    "else",
    "loop",
    "for",
    "while",
    "break",
    "continue",
    "return",
    "switch",
    "case",
    "default",
    "discard",
    "true",
    "false",
    "bool",
    "i32",
    "u32",
    "f32",
    "f16",
    "vec2",
    "vec3",
    "vec4",
    "mat2x2",
    "mat3x3",
    "mat4x4",
    "array",
    "atomic",
    "ptr",
    "sampler",
    "texture",
    "override",
    "alias",
    "enable",
    "requires",
    "diagnostic",
];

/// The WGSL struct-field name for a kernel param — the param's own name, unless
/// that is reserved, in which case it takes a trailing underscore.
pub fn wgsl_field(param: &str) -> String {
    if WGSL_RESERVED.contains(&param) || ENGINE_UNIFORMS.contains(&param) {
        format!("{param}_")
    } else {
        param.to_string()
    }
}
