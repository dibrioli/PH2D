//! The GPU **lowering** pass — the compute twin of
//! `ph2d_eval_motion::lower_to_instances_onto`: gathers the stream convention
//! columns (`P` / `size` / `rot` / `tint` / `uv_rect` / `texture_id`, absent →
//! the same defaults the CPU applies) straight into a buffer laid out as
//! [`ph2d_render::RenderInstance`], which the sprite renderer binds as its
//! instance vertex buffer. This is what makes the path readback-free: the
//! cook's last write IS the renderer's input.
//!
//! `texture_id` (word 41) is the newest reader, and the reason the GPU can now
//! draw a `source.object` graph: a duplicated sprite carries the object's baked
//! tile / individual-texture handle in this column, the deformer suffix copies
//! it position-for-position, and this pass writes it into the instance so the
//! device buffer is a FAITHFUL `RenderInstance` array (the renderer binds the
//! matching texture per run — see `renderer_draw`). Absent ⇒ `0` = the shared
//! atlas, byte-identical to every non-object graph that shipped before it.
//!
//! The instance is written word-by-word into an `array<u32>` (with
//! `bitcast` for the float fields) because the WGSL alignment rules cannot
//! mirror the `#[repr(C)]` struct — `anchor: vec2<f32>` sits at byte offset
//! 68, and WGSL requires vec2 alignment 8. The word layout below is pinned
//! against `size_of::<RenderInstance>()` by a unit test AND by the render
//! crate's own `render_instance_pod_size_v4` gate.

use crate::codegen::WORKGROUP_SIZE;

/// `RenderInstance` is 184 bytes = 46 32-bit words.
pub const INSTANCE_WORDS: u32 = 46;

/// The six stream columns the lowering reads, in binding order. Presence of
/// each (bit `i` of the pipeline-cache signature) selects between a storage
/// binding and the default. `texture_id` is a `Scalar` column like `rot` — it
/// is read as `f32` and truncated to `u32`, mirroring the CPU lowering's
/// `scalar_at(tex, i, 0.0) as u32`.
pub const LOWER_COLUMNS: [&str; 7] = [
    "P",
    "size",
    "rot",
    "tint",
    "uv_rect",
    "texture_id",
    // doc 89, folha 07 — o OPERADOR POR-LINHA (o *Echo Operator* do AE). Escalar como o
    // `rot`; ausente ⇒ `0`, que quer dizer *o modo do sink* e é o que esta geradora
    // escrevia como constante antes da coluna existir ⇒ byte-idêntico.
    "blend",
];

/// Generate the lowering module for a concrete column set. Binding 0 = the
/// uniforms, binding 1 = the instance output; then one `read` binding per
/// present column, in [`LOWER_COLUMNS`] order.
///
/// `blend` is the sink's `BlendMode` tag (doc 89, folha 17), packed into the
/// instance's `flip_uv` bits 5-7 by the SAME function the CPU lowering uses. It
/// is a **codegen constant**, not a uniform: it is one number for the whole sink,
/// it cannot change within a dispatch, and it joins [`lower_signature`] so the
/// pipeline cache keys on it — a uniform would have cost a binding and a write
/// per frame to say something the source can simply spell. Tag 0 emits the
/// literal `0u` this generator wrote before the param existed ⇒ byte-identical.
pub fn lower_module(present: [bool; 7], blend: u8) -> String {
    let blend_bits = ph2d_render::RenderInstance::pack_blend_bits(blend);
    // O teto e o deslocamento vêm do RENDERER, nunca de literais: um `6`/`5` cravados aqui
    // continuariam a compilar no dia em que um sétimo modo nascesse.
    let top = ph2d_render::pipeline::BLEND_PIPELINE_COUNT;
    let shift = ph2d_render::RenderInstance::BLEND_SHIFT;
    let mut src = String::with_capacity(2048);
    src.push_str(
        "struct LowerParams {\n\
         \x20   count: u32,\n\
         \x20   _pad0: u32,\n\
         \x20   default_size: vec2<f32>,\n\
         \x20   default_uv: vec4<f32>,\n\
         }\n\
         @group(0) @binding(0) var<uniform> params: LowerParams;\n\
         @group(0) @binding(1) var<storage, read_write> instances: array<u32>;\n",
    );
    let tys = [
        "vec2<f32>",
        "vec2<f32>",
        "f32",
        "vec4<f32>",
        "vec4<f32>",
        "f32",
        "f32",
    ];
    let mut slot = 2u32;
    for (i, col) in LOWER_COLUMNS.iter().enumerate() {
        if present[i] {
            src.push_str(&format!(
                "@group(0) @binding({slot}) var<storage, read> in_{col}: array<{}>;\n",
                tys[i]
            ));
            slot += 1;
        }
    }
    src.push('\n');
    // Reader per column: buffer when present, else the SAME default the CPU
    // lowering applies (`scalar_at`/`vec2_at`/`vec4_at` fallbacks).
    let defaults = [
        "vec2<f32>(0.0, 0.0)",           // P
        "params.default_size",           // size (caller-supplied, like the CPU)
        "0.0",                           // rot
        "vec4<f32>(1.0, 1.0, 1.0, 1.0)", // tint
        "params.default_uv",             // uv_rect (caller-supplied)
        "0.0",                           // texture_id (absent → atlas 0)
        "0.0",                           // blend (absent → 0 = o modo do SINK)
    ];
    for (i, col) in LOWER_COLUMNS.iter().enumerate() {
        if present[i] {
            src.push_str(&format!(
                "fn read_{col}(i: u32) -> {ty} {{ return in_{col}[i]; }}\n",
                ty = tys[i]
            ));
        } else {
            src.push_str(&format!(
                "fn read_{col}(i: u32) -> {ty} {{ _ = i; return {d}; }}\n",
                ty = tys[i],
                d = defaults[i]
            ));
        }
    }
    src.push_str(&format!(
        "\n\
        fn wf(w: u32, v: f32) {{ instances[w] = bitcast<u32>(v); }}\n\
        \n\
        @compute @workgroup_size({WORKGROUP_SIZE})\n\
        fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
        \x20   let i = gid.x;\n\
        \x20   if (i >= params.count) {{ return; }}\n\
        \x20   let base = i * {INSTANCE_WORDS}u;\n\
        \x20   // world_pos (words 0-1) · size (2-3) · atlas_uv (4-7) · tint (8-11)\n\
        \x20   let p = read_P(i);\n\
        \x20   wf(base + 0u, p.x);\n\
        \x20   wf(base + 1u, p.y);\n\
        \x20   let s = read_size(i);\n\
        \x20   wf(base + 2u, s.x);\n\
        \x20   wf(base + 3u, s.y);\n\
        \x20   let uv = read_uv_rect(i);\n\
        \x20   wf(base + 4u, uv.x);\n\
        \x20   wf(base + 5u, uv.y);\n\
        \x20   wf(base + 6u, uv.z);\n\
        \x20   wf(base + 7u, uv.w);\n\
        \x20   let t = read_tint(i);\n\
        \x20   wf(base + 8u, t.x);\n\
        \x20   wf(base + 9u, t.y);\n\
        \x20   wf(base + 10u, t.z);\n\
        \x20   wf(base + 11u, t.w);\n\
        \x20   // basis (12-15): `rot` is authored in DEGREES (the app's angle\n\
        \x20   // unit); the conversion lives only here, exactly like the CPU\n\
        \x20   // lowering. RenderInstance is PresentWorld-only (HR-5 exempt),\n\
        \x20   // so hardware sin/cos is fine — parity vs the CPU's sin_cos is\n\
        \x20   // held by the ε gate, not bit-for-bit.\n\
        \x20   let rad = read_rot(i) * 0.017453292519943295;\n\
        \x20   let sn = sin(rad);\n\
        \x20   let cs = cos(rad);\n\
        \x20   wf(base + 12u, cs);\n\
        \x20   wf(base + 13u, sn);\n\
        \x20   wf(base + 14u, -sn);\n\
        \x20   wf(base + 15u, cs);\n\
        \x20   // premultiplied (16) + anchor (17-18): zero.\n\
        \x20   instances[base + 16u] = 0u;\n\
        \x20   instances[base + 17u] = 0u;\n\
        \x20   instances[base + 18u] = 0u;\n\
        \x20   // per_corner_tint (19-34): identity white.\n\
        \x20   for (var k = base + 19u; k < base + 35u; k = k + 1u) {{\n\
        \x20       wf(k, 1.0);\n\
        \x20   }}\n\
        \x20   // opacity (35) = 1 · flip_uv (36) = the sink's blend bits · uv_xform (37-40) = identity.\n\
        \x20   wf(base + 35u, 1.0);\n\
        \x20   // ⚠️ **flip_uv (36): a coluna `blend` decide por LINHA, e `0` quer dizer\n\
        \x20   // *o modo do sink*** (doc 89 folha 07) — a MESMA escada do `blend_at` da\n\
        \x20   // rota da CPU, porque as duas têm de compor igual. Sem a coluna o\n\
        \x20   // `read_blend` é a constante `0.0` e isto dobra na constante de sempre.\n\
        \x20   let bt = read_blend(i);\n\
        \x20   var bb = {blend_bits}u;\n\
        \x20   if (bt >= 0.5) {{\n\
        \x20       bb = ((min(u32(round(bt)), {top}u) - 1u) & 7u) << {shift}u;\n\
        \x20   }}\n\
        \x20   instances[base + 36u] = bb;\n\
        \x20   wf(base + 37u, 1.0);\n\
        \x20   wf(base + 38u, 1.0);\n\
        \x20   instances[base + 39u] = 0u;\n\
        \x20   instances[base + 40u] = 0u;\n\
        \x20   // texture_id (41): the object's tile/individual handle, from the\n\
        \x20   // stream column (absent → 0 = atlas). `u32(f32)` truncates toward\n\
        \x20   // zero, exactly like the CPU lowering's `scalar_at(..) as u32`.\n\
        \x20   // z_order · sampling · clip_group · clip_meta (42-45): the CPU's.\n\
        \x20   instances[base + 41u] = u32(read_texture_id(i));\n\
        \x20   instances[base + 42u] = 0u;\n\
        \x20   instances[base + 43u] = 0u;\n\
        \x20   instances[base + 44u] = 0u;\n\
        \x20   instances[base + 45u] = 0u;\n\
        }}\n"
    ));
    src
}

/// Pipeline-cache signature for a lowering column set (bit per column) **and the
/// blend tag** — the two things [`lower_module`] bakes into its source. The tag
/// rides above the column bits, so a document that only changes its sink's blend
/// gets its own cached pipeline instead of silently reusing the previous mode's.
pub fn lower_signature(present: [bool; 7], blend: u8) -> u64 {
    let cols = present
        .iter()
        .enumerate()
        .fold(0u64, |sig, (i, &p)| sig | ((p as u64) << i));
    cols | (u64::from(blend) << LOWER_COLUMNS.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_render::pipeline::BLEND_PIPELINE_COUNT;

    #[test]
    fn instance_words_matches_render_instance_size() {
        // 46 words × 4 = size_of::<RenderInstance>() — if the render ABI grows,
        // this trips before the GPU writes garbage into the tail.
        assert_eq!(
            (INSTANCE_WORDS as usize) * 4,
            std::mem::size_of::<ph2d_render::RenderInstance>()
        );
    }

    #[test]
    fn absent_columns_read_the_cpu_defaults() {
        let src = lower_module([false; 7], 0);
        assert!(src.contains("return vec2<f32>(0.0, 0.0);")); // P
        assert!(src.contains("return params.default_size;"));
        assert!(src.contains("return params.default_uv;"));
        assert!(src.contains("return vec4<f32>(1.0, 1.0, 1.0, 1.0);")); // tint
        assert!(!src.contains("var<storage, read> in_"));
    }

    /// Word 41 (`texture_id`) is written FROM the column, not hardcoded to the
    /// atlas. Red-first: this asserts the exact line the atlas-hardcode bug
    /// (`instances[base + 41u] = 0u;`) never produced — a `source.object`'s
    /// tile handle only reaches the device if the lowering reads the column.
    #[test]
    fn the_lowering_carries_texture_id() {
        let mut present = [false; 7];
        present[5] = true; // texture_id present
        let src = lower_module(present, 0);
        // The column is bound and read as f32 (like `rot`), truncated to u32.
        assert!(src.contains("var<storage, read> in_texture_id: array<f32>;"));
        assert!(src.contains("fn read_texture_id(i: u32) -> f32 { return in_texture_id[i]; }"));
        assert!(src.contains("instances[base + 41u] = u32(read_texture_id(i));"));
        // And the old atlas hardcode is gone.
        assert!(!src.contains("instances[base + 41u] = 0u;"));
    }

    /// Absent `texture_id` still writes word 41 = 0 (atlas), so every non-object
    /// graph is byte-identical — the reader falls back to `0.0`, truncating to 0.
    #[test]
    fn absent_texture_id_is_the_atlas() {
        let src = lower_module([false; 7], 0);
        assert!(src.contains("fn read_texture_id(i: u32) -> f32 { _ = i; return 0.0; }"));
        assert!(src.contains("instances[base + 41u] = u32(read_texture_id(i));"));
    }

    /// **The neutral tag emits the literal this generator always wrote** (doc 89,
    /// folha 17): word 36 (`flip_uv`) is `0u` for `Mix`, so every document that
    /// never touched the param produces byte-identical source — and therefore a
    /// byte-identical instance.
    #[test]
    fn the_neutral_blend_emits_the_zero_word_it_always_did() {
        let src = lower_module([false; 7], 0);
        // ⚠️ A palavra deixou de ser um literal e passou a ser um `if` sobre a coluna
        // `blend` (doc 89 folha 07). O que continua a valer é a CONSTANTE de que ele parte:
        // sem coluna, o `read_blend` é `0.0`, o ramo nunca corre, e o que sai é este `0u`.
        assert!(
            src.contains("var bb = 0u;"),
            "o default parou de escrever a constante que esta geradora sempre escreveu"
        );
        assert!(src.contains("fn read_blend(i: u32) -> f32 { _ = i; return 0.0; }"));
    }

    /// **A chosen tag reaches word 36, in the RENDERER's packing.** The oracle is
    /// `pack_blend_bits` — the renderer's own packer, the inverse of the
    /// `unpack_blend` that `compute_runs` keys draw runs on — not a
    /// re-implementation of the shift here.
    ///
    /// ⚠️ This is the half **no device-free gate could otherwise see**: a
    /// generator that ignored `blend` still emits valid WGSL, so the naga sweep
    /// stays green and only an artist on a machine with a GPU would find it.
    #[test]
    fn an_authored_blend_is_baked_into_the_generated_source() {
        for blend in 1..BLEND_PIPELINE_COUNT as u8 {
            let bits = ph2d_render::RenderInstance::pack_blend_bits(blend);
            let src = lower_module([false; 7], blend);
            assert!(
                src.contains(&format!("var bb = {bits}u;")),
                "blend {blend} did not reach word 36 of the generated source"
            );
        }
    }

    /// **A COLUNA `blend` CHEGA À PALAVRA 36 NA ROTA DA GPU** (doc 89, folha 07).
    ///
    /// ⚠️ **É a metade que nenhuma varredura de naga vê:** uma geradora que ligasse a coluna
    /// e não a LESSE emite WGSL perfeitamente válido, e o `Echo Operator` funcionaria na CPU
    /// e não no device — o defeito que o cabeçalho do arch-gate do sink chama de *"o artista
    /// vê a feature funcionar e depois parar sem mexer em nada"*.
    #[test]
    fn the_blend_column_reaches_word_36_on_the_device_route() {
        let mut present = [false; 7];
        present[6] = true; // a coluna `blend`
        let src = lower_module(present, 0);
        assert!(src.contains("var<storage, read> in_blend: array<f32>;"));
        assert!(src.contains("fn read_blend(i: u32) -> f32 { return in_blend[i]; }"));
        assert!(
            src.contains("let bt = read_blend(i);"),
            "a palavra tem de LER a coluna"
        );
        assert!(src.contains("instances[base + 36u] = bb;"));
        // E o deslocamento é o do RENDERER, nunca um literal desta geradora.
        let shift = ph2d_render::RenderInstance::BLEND_SHIFT;
        assert!(src.contains(&format!("<< {shift}u;")));
    }

    /// **The pipeline cache can TELL two blends apart.** The tag is a codegen
    /// constant, so two sources that differ only in it must not collide on one
    /// cache key — a collision would hand the second mode the first mode's
    /// compiled pipeline and draw it in the wrong blend, on the device, silently.
    ///
    /// Asserts the PROPERTY (all signatures distinct) rather than the bit layout,
    /// so moving the tag's shift does not falsify a correct generator.
    #[test]
    fn two_blends_never_share_one_pipeline_cache_key() {
        for mask in 0u8..64 {
            let present = std::array::from_fn(|i| mask & (1 << i) != 0);
            let sigs: Vec<u64> = (0..BLEND_PIPELINE_COUNT as u8)
                .map(|b| lower_signature(present, b))
                .collect();
            let mut sorted = sigs.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(
                sorted.len(),
                sigs.len(),
                "mask {mask:06b}: two blends collide on one cache key"
            );
        }
        // And the column bits still separate column sets at a FIXED blend — the
        // tag must not have eaten the bits it rides above.
        let a = lower_signature([false; 7], 0);
        let b = lower_signature([true; 7], 0);
        assert_ne!(a, b, "the column bits stopped separating column sets");
    }
}
