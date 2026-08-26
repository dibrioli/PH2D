//! **O KERNEL DE DEVICE do `sim.collide`** — o WGSL, a biblioteca dele e os bindings.
//!
//! Irmão do [`super`] pelo teto de LOC (HR-18, 700 nas crates), no corte que a pergunta
//! desenha: lá mora *qual é a lei da colisão*, aqui *como ela corre no dispositivo*.
//!
//! ⚠️ **As duas metades têm de mover-se juntas.** Todo termo daqui é o gémeo de uma função do
//! módulo pai (`contact` · `respond` · `particle_radius` · `element_restitution` · `plane_normal`),
//! e é o gate de paridade CPU/GPU que prova que continuam a ser. Um literal mudado só de um
//! lado é o modo de falha desta família — e é por isso que os comentários que dizem *"termo a
//! termo"* ficam onde estão, do lado que é fácil de esquecer.

use super::*;

/// The WGSL port of [`contact`] + [`respond`], element for element (ADR-0135 —
/// the sim-zone family on the GPU). Single-port, no clock, no grid: it reads `P`
/// and `vel` off the state and writes them back. The shape is a **uniform branch**
/// (every element shares the `shape` param), so it is coherent on the device.
///
/// The param clamps are the kernel's own (`radius.max(0)`, restitution/friction
/// clamped to `[0,1]`) — a clamp that lives only on the CPU is a divergence waiting
/// for a slider at its edge. Position is pushed out **unconditionally**; only the
/// velocity reflection is gated on `vn < 0` (already leaving ⇒ do not re-reflect,
/// the classic collider buzz), and the whole response is dropped when non-finite.
pub(super) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let sc_shape = i32(round(params.shape));\n\
        let sc_c = vec2<f32>(params.center_x, params.center_y);\n\
        let sc_radius = max(params.radius, 0.0);\n\
        let sc_rest_authored = clamp(params.restitution, 0.0, 1.0);\n\
        // A aleatoriedade por elemento — `element_restitution` termo a termo. A chave e' o\n\
        // `id` quando ele existe; `HAS_id` e o que impede a coluna AUSENTE de ler zero para\n\
        // todo elemento (a identidade do binding), o que daria a TODOS a mesma sorte.\n\
        let sc_rnd = clamp(params.restitution_randomness, 0.0, 1.0);\n\
        var sc_key = i;\n\
        if (HAS_id) { sc_key = u32(max(read_id(i), 0.0)); }\n\
        let sc_seed = u32(max(params.seed, 0.0));\n\
        let sc_rest = sc_rest_authored * (1.0 - sc_rnd * sc_rand01(sc_seed, sc_key));\n\
        let sc_fric = clamp(params.friction, 0.0, 1.0);\n\
        let sc_p = read_P(i);\n\
        let sc_v = read_vel(i);\n\
        // The particle's own radius — `particle_radius`, term for term and in the\n\
        // same multiply order, so the two paths cannot answer this differently.\n\
        let sc_size = read_size(i);\n\
        // The plane's normal — the SAME polynomial as `trig.rs`, the same `sqrt`\n\
        // normalisation, and `0.0 - s` so `angle = 0` lands on the literal (0, 1).\n\
        let sc_cs = sc_cos_sin(params.angle / 360.0);\n\
        let sc_inv = 1.0 / sqrt(sc_cs.x * sc_cs.x + sc_cs.y * sc_cs.y);\n\
        let sc_pn = vec2<f32>((0.0 - sc_cs.y) * sc_inv, sc_cs.x * sc_inv);\n\
        var sc_r = 0.0;\n\
        if (i32(round(params.radius_from)) == SC_R_FIXED) {\n\
        \x20   sc_r = max(params.particle_radius, 0.0);\n\
        } else if (i32(round(params.radius_from)) == SC_R_SIZE) {\n\
        \x20   sc_r = max(min(abs(sc_size.x), abs(sc_size.y)) * 0.5 * params.size_scale, 0.0);\n\
        }\n\
        var sc_hit = false;\n\
        var sc_n = vec2<f32>(0.0, 1.0);\n\
        var sc_depth = 0.0;\n\
        if (sc_shape == SC_DISC || sc_shape == SC_BOWL) {\n\
        \x20   let sc_d = sc_p - sc_c;\n\
        \x20   let sc_dist = sqrt(sc_d.x * sc_d.x + sc_d.y * sc_d.y);\n\
        \x20   // Dead centre has no way out: pick up rather than divide by zero.\n\
        \x20   var sc_dir = vec2<f32>(0.0, 1.0);\n\
        \x20   if (sc_dist > SC_EPS) { sc_dir = sc_d / sc_dist; }\n\
        \x20   if (sc_shape == SC_DISC) {\n\
        \x20       // The obstacle GROWS by the radius; the container SHRINKS by it.\n\
        \x20       let sc_grown = sc_radius + sc_r;\n\
        \x20       if (sc_dist < sc_grown) { sc_hit = true; sc_n = sc_dir; sc_depth = sc_grown - sc_dist; }\n\
        \x20   } else {\n\
        \x20       let sc_inner = max(sc_radius - sc_r, 0.0);\n\
        \x20       if (sc_dist > sc_inner) { sc_hit = true; sc_n = -sc_dir; sc_depth = sc_dist - sc_inner; }\n\
        \x20   }\n\
        } else {\n\
        \x20   // The plane: the world is the side the normal points to, so out IS the\n\
        \x20   // normal — and what touches it is the element's near face, `sd - r`.\n\
        \x20   let sc_sd = dot(sc_p, sc_pn);\n\
        \x20   if (sc_sd - sc_r < params.height) { sc_hit = true; sc_n = sc_pn; sc_depth = params.height - (sc_sd - sc_r); }\n\
        }\n\
        var sc_out_p = sc_p;\n\
        var sc_out_v = sc_v;\n\
        // Accumulates over the tick: a CHAIN of colliders reports the deepest contact,\n\
        // never just the last one's. The zone strips it, so it never crosses a tick.\n\
        var sc_out_hit = read_hit(i);\n\
        if (sc_hit) {\n\
        \x20   let sc_rp = sc_p + sc_n * sc_depth;\n\
        \x20   var sc_rv = sc_v;\n\
        \x20   let sc_vn = dot(sc_rv, sc_n);\n\
        \x20   // Already leaving (or sliding): touching must not change it.\n\
        \x20   if (sc_vn < 0.0) {\n\
        \x20       let sc_bounce = (1.0 + sc_rest) * sc_vn;\n\
        \x20       let sc_reflected = sc_rv - sc_bounce * sc_n;\n\
        \x20       let sc_vn_out = dot(sc_reflected, sc_n);\n\
        \x20       let sc_tangent = sc_reflected - sc_vn_out * sc_n;\n\
        \x20       sc_rv = sc_vn_out * sc_n + sc_tangent * (1.0 - sc_fric);\n\
        \x20   }\n\
        \x20   if (collide_finite(sc_rp) && collide_finite(sc_rv)) {\n\
        \x20       sc_out_p = sc_rp;\n\
        \x20       sc_out_v = sc_rv;\n\
        \x20       sc_out_hit = max(sc_out_hit, sc_depth);\n\
        \x20   }\n\
        }\n\
        write_P(i, sc_out_p);\n\
        write_vel(i, sc_out_v);\n\
        write_hit(i, sc_out_hit);\n",
    wgsl_lib: "\
        // `hash.rs` transliterado. WGSL `u32` faz wrap e `>>` e' logico, entao a\n\
        // avalanche e' a mesma; o golden do modulo Rust prende os dois lados.\n\
        fn sc_rand01(seed: u32, index: u32) -> f32 {\n\
        \x20   var h = seed * 0x9e3779b9u + index * 0x85ebca6bu;\n\
        \x20   h = h ^ (h >> 16u);\n\
        \x20   h = h * 0x7feb352du;\n\
        \x20   h = h ^ (h >> 15u);\n\
        \x20   h = h * 0x846ca68bu;\n\
        \x20   h = h ^ (h >> 16u);\n\
        \x20   return f32(h >> 8u) / 16777216.0;\n\
        }\n\
        const SC_DISC: i32 = 1;\n\
        const SC_BOWL: i32 = 2;\n\
        const SC_R_FIXED: i32 = 1;\n\
        const SC_R_SIZE: i32 = 2;\n\
        const SC_EPS: f32 = 1.1920929e-7;\n\
        const SC_F32_MAX: f32 = 3.4028235e38;\n\
        fn collide_finite(v: vec2<f32>) -> bool {\n\
        \x20   return abs(v.x) <= SC_F32_MAX && abs(v.y) <= SC_F32_MAX;\n\
        }\n\
        fn sc_sin_cycles(phase: f32) -> f32 {\n\
        \x20   // The corrected parabolic sine (see trig.rs) — the SAME polynomial as\n\
        \x20   // the CPU, so parity holds; phase is in cycles (deg / 360).\n\
        \x20   let ff = phase - floor(phase);\n\
        \x20   var p: f32;\n\
        \x20   if (ff < 0.5) { let u = ff * 2.0; p = 4.0 * u * (1.0 - u); }\n\
        \x20   else { let u = (ff - 0.5) * 2.0; p = -4.0 * u * (1.0 - u); }\n\
        \x20   return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn sc_cos_sin(phase: f32) -> vec2<f32> {\n\
        \x20   return vec2<f32>(sc_sin_cycles(phase + 0.25), sc_sin_cycles(phase));\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // READ-ONLY: the collider never resizes anything, it only asks how big the
        // thing it is catching is. `SIZE_IDENTITY = [1, 1]` — the same absence the
        // CPU's `sizes()` and the lowering itself fall back to.
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        // The contact channel. `ReadWrite` with identity 0, never `Write`: see [`HIT_COL`] —
        // a chain of colliders has to report the DEEPEST contact of the tick, not the last
        // one's, and that is an accumulation, so the kernel must be able to read what the
        // collider before it left.
        ColumnBinding {
            column: HIT_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        // ⚠️ A `identity` de uma coluna `id` AUSENTE e zero, e zero nao e o indice — e por
        // isso o corpo ramifica no `HAS_id` em vez de confiar nela. Sem essa ramificacao um
        // conjunto sem `id` daria a todo elemento a MESMA sorte, e a aleatoriedade seria uma
        // multiplicacao constante com nome de acaso.
        ColumnBinding {
            column: "id",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[
        "shape",
        "height",
        "center_x",
        "center_y",
        "radius",
        "restitution",
        "friction",
        "radius_from",
        "particle_radius",
        "size_scale",
        "angle",
        RANDOMNESS,
        SEED,
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};
