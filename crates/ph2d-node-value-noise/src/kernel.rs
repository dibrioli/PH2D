//! **O que o DISPOSITIVO roda** — o kernel WGSL de `value.noise` e a lei de
//! contagem que o sequenciador consulta.
//!
//! Saiu do `lib.rs` no teto de LOC, por RESPONSABILIDADE e seguindo o precedente
//! do irmão `motion.noise` (que tem o `kernel.rs` dele pelo mesmo motivo): o pai
//! fica com **o que o nó É** (o manifesto, a amostra, o `eval`, a superfície de
//! painel) e este irmão com **o gémeo em WGSL** — que é o que cresce quando o
//! catálogo de kernels cresce.

use crate::VALUE_COL;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::port::Dim;

/// GPU compute kernel (ADR-0126) — the WGSL port of [`Sample::at`] + [`noise`],
/// **fully device-resident**. Reads `params.playhead` (the magic uniform every
/// kernel gets, like `value.lfo`). No `applicable` gate — the sequencer never
/// falls back to the CPU for this node (the "maximize GPU" north). The lattice
/// hash + fade are byte-mirrors of `motion.wiggle`'s WGSL (`vn_` ↔ `wg_`), so the
/// two nodes sample the same field; the fBm loop is bounded by a hard `8`
/// (== `noise::MAX_OCTAVES`).
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vn_oct = clamp(i32(vn_round(params.octaves)), 1, 8);\n\
        // O eixo: a fila (o indice) ou o ESPACO (a posicao). ⚠️ `HAS_P` e o que\n\
        // impede a coluna AUSENTE de ler a identidade zero e colapsar o campo\n\
        // num valor so -- sem posicao nao ha espaco a amostrar, e a queda e o\n\
        // indice, como na CPU.\n\
        let vn_world = i32(vn_round(params.space)) == 1 && HAS_P;\n\
        // A COSTURA DO LACO -- a transcricao de `ph2d_fbm::loop_times`. O tempo\n\
        // WRAPA antes de entrar no campo (misturar campo(t) com campo(t-L) nao\n\
        // fecha: sao campos diferentes nas duas pontas) e o peso e smoothstep\n\
        // (com peso linear o valor fecha e a DERIVADA salta na costura).\n\
        var vn_ta = params.playhead;\n\
        var vn_tb = vn_ta;\n\
        var vn_w = 0.0;\n\
        if (params.loop_period > 0.0) {\n\
        \x20   let vn_u0 = params.playhead / params.loop_period;\n\
        \x20   let vn_u = vn_u0 - floor(vn_u0);\n\
        \x20   vn_ta = vn_u * params.loop_period;\n\
        \x20   vn_tb = vn_ta - params.loop_period;\n\
        \x20   vn_w = vn_u * vn_u * (3.0 - 2.0 * vn_u);\n\
        }\n\
        // O eixo do reticulado: a fila ou o espaco, mais o PAN (o `seed` feito\n\
        // continuo -- a mesma regua, que e' por isso que ele soma ao lado dele).\n\
        var vn_base_x = params.pan_x;\n\
        var vn_y = f32(i) * params.frequency + params.seed + params.pan_y;\n\
        if (vn_world) {\n\
        \x20   let vn_p = read_P(i);\n\
        \x20   vn_base_x = vn_p.x * params.frequency + params.pan_x;\n\
        \x20   vn_y = vn_p.y * params.frequency + params.seed + params.pan_y;\n\
        }\n\
        // O CATALOGO de kernels (doc 89 folha 15): 0 Value (o que sempre\n\
        // shipou) - 1 Perlin - 2 Cellular. Um indice fora da lista cai no Value,\n\
        // como na CPU.\n\
        let vn_kern = i32(vn_round(params.kernel));\n\
        let vn_feat = i32(vn_round(params.feature));\n\
        let vn_a = vn_fbm(vn_base_x + vn_ta * params.speed, vn_y, vn_oct,\n\
        \x20   params.roughness, vn_kern, vn_feat, params.jitter, params.lacunarity);\n\
        var vn_n = vn_a;\n\
        // `w == 0` e o caminho de sempre: a segunda amostra nem e' avaliada.\n\
        if (vn_w != 0.0) {\n\
        \x20   let vn_b = vn_fbm(vn_base_x + vn_tb * params.speed, vn_y, vn_oct,\n\
        \x20       params.roughness, vn_kern, vn_feat, params.jitter, params.lacunarity);\n\
        \x20   vn_n = vn_a + (vn_b - vn_a) * vn_w;\n\
        }\n\
        write_v(i, vn_n * params.amplitude + params.offset);\n",
    wgsl_lib: "\
        fn vn_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn vn_hash_bits(ix: i32, iy: i32) -> u32 {\n\
            // Same mix as noise::hash_bits — u32 wraps mod 2^32 (== Rust wrapping_*),\n\
            // bitcast<u32> == Rust `as u32` (bit reinterpretation, not a value cast).\n\
            var h: u32 = bitcast<u32>(ix) * 0x27d4eb2du + bitcast<u32>(iy) * 0x165667b1u;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x2c1b3c6du;\n\
            h = h ^ (h >> 12u);\n\
            h = h * 0x297175f9u;\n\
            h = h ^ (h >> 15u);\n\
            return h;\n\
        }\n\
        fn vn_hash2(ix: i32, iy: i32) -> f32 {\n\
            return (f32(vn_hash_bits(ix, iy)) / f32(0xffffffffu)) * 2.0 - 1.0;\n\
        }\n\
        fn vn_fade(t: f32) -> f32 {\n\
            return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);\n\
        }\n\
        fn vn_value_noise(x: f32, y: f32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let u = vn_fade(x - x0);\n\
            let v = vn_fade(y - y0);\n\
            let n00 = vn_hash2(ix, iy);\n\
            let n10 = vn_hash2(ix + 1, iy);\n\
            let n01 = vn_hash2(ix, iy + 1);\n\
            let n11 = vn_hash2(ix + 1, iy + 1);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return nx0 + v * (nx1 - nx0);\n\
        }\n\
        fn vn_dot_grad(h: u32, dx: f32, dy: f32) -> f32 {\n\
            // Os oito gradientes de Perlin 2002, escritos como +-u +- 2v.\n\
            let g = h & 7u;\n\
            var u = dx;\n\
            var v = dy;\n\
            if (g >= 4u) { u = dy; v = dx; }\n\
            var a = u;\n\
            if ((g & 1u) != 0u) { a = -u; }\n\
            var b = 2.0 * v;\n\
            if ((g & 2u) != 0u) { b = -2.0 * v; }\n\
            return a + b;\n\
        }\n\
        fn vn_perlin(x: f32, y: f32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let fx = x - x0;\n\
            let fy = y - y0;\n\
            let u = vn_fade(fx);\n\
            let v = vn_fade(fy);\n\
            let n00 = vn_dot_grad(vn_hash_bits(ix, iy), fx, fy);\n\
            let n10 = vn_dot_grad(vn_hash_bits(ix + 1, iy), fx - 1.0, fy);\n\
            let n01 = vn_dot_grad(vn_hash_bits(ix, iy + 1), fx, fy - 1.0);\n\
            let n11 = vn_dot_grad(vn_hash_bits(ix + 1, iy + 1), fx - 1.0, fy - 1.0);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return (nx0 + v * (nx1 - nx0)) * (1.0 / 1.5);\n\
        }\n\
        fn vn_cell(x: f32, y: f32, feature: i32, jitter: f32) -> f32 {\n\
            // Worley 3x3: com jitter <= 1 a semente nunca sai da propria celula,\n\
            // entao o vizinho mais proximo esta sempre nesta janela (F1 exacto).\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let fx = x - x0;\n\
            let fy = y - y0;\n\
            let j = clamp(jitter, 0.0, 1.0);\n\
            var d1 = 1e30;\n\
            var d2 = 1e30;\n\
            for (var oy = -1; oy <= 1; oy = oy + 1) {\n\
            \x20   for (var ox = -1; ox <= 1; ox = ox + 1) {\n\
            \x20       let bits = vn_hash_bits(ix + ox, iy + oy);\n\
            \x20       let rx = f32(bits & 0xffffu) * (1.0 / 65535.0);\n\
            \x20       let ry = f32(bits >> 16u) * (1.0 / 65535.0);\n\
            \x20       let px = f32(ox) + 0.5 + (rx - 0.5) * j;\n\
            \x20       let py = f32(oy) + 0.5 + (ry - 0.5) * j;\n\
            \x20       let dx = px - fx;\n\
            \x20       let dy = py - fy;\n\
            \x20       let d = dx * dx + dy * dy;\n\
            \x20       if (d < d1) { d2 = d1; d1 = d; } else if (d < d2) { d2 = d; }\n\
            \x20   }\n\
            }\n\
            let f1 = sqrt(d1);\n\
            var raw = f1;\n\
            if (feature == 1) { raw = sqrt(d2) - f1; }\n\
            return clamp(raw, 0.0, 1.0) * 2.0 - 1.0;\n\
        }\n\
        fn vn_base(kern: i32, feature: i32, jitter: f32, x: f32, y: f32) -> f32 {\n\
            if (kern == 1) { return vn_perlin(x, y); }\n\
            if (kern == 2) { return vn_cell(x, y, feature, jitter); }\n\
            return vn_value_noise(x, y);\n\
        }\n\
        fn vn_fbm(x: f32, y: f32, oct: i32, rough: f32, kern: i32, feature: i32,\n\
        \x20   jitter: f32, lac: f32) -> f32 {\n\
            // O gemeo do `ph2d_fbm::eval`: ganho clampado, a COORDENADA escalada\n\
            // por multiplicacao repetida (com lacunarity 2 nao arredonda), e a\n\
            // normalizacao pela soma dos pesos.\n\
            let gain = clamp(rough, 0.0, 1.0);\n\
            var px = x;\n\
            var py = y;\n\
            var sum = 0.0;\n\
            var amp = 1.0;\n\
            var total = 0.0;\n\
            for (var o = 0; o < oct; o = o + 1) {\n\
            \x20   sum = sum + amp * vn_base(kern, feature, jitter, px, py);\n\
            \x20   total = total + amp;\n\
            \x20   amp = amp * gain;\n\
            \x20   px = px * lac;\n\
            \x20   py = py * lac;\n\
            }\n\
            return sum / total;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        // ⚠️ A `identity` de um `P` AUSENTE é zero, e zero não é uma posição — o
        // campo colapsaria num valor só. Por isso o corpo ramifica no `HAS_P`.
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[
        "space",
        "frequency",
        "speed",
        "octaves",
        "roughness",
        "amplitude",
        "offset",
        "seed",
        "kernel",
        "feature",
        "jitter",
        // ⚠️ Esta lista NÃO é derivada do manifesto: um param novo compila, coza na
        // CPU, e o device **recusa o shader** (`invalid field accessor`). É falha
        // alta e não silêncio, mas o sítio esquece-se — a lição do `motion.boids`.
        "lacunarity",
        "loop_period",
        "pan_x",
        "pan_y",
    ],
    count_law: Some(noise_count),
    variant_by_param: None,
    applicable: None,
};

/// **How wide is the field?** — the same law `value.lfo`/`value.instance_field`
/// use: connected, one value per instance; **unconnected, ONE global value** (held
/// across every instance by `motion.drive`'s broadcast). The engine's default —
/// "as wide as port 0" — sizes an unconnected one at 0 and SKIPS the stage.
fn noise_count(c: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(c.inputs.first().copied().unwrap_or(0).max(1) as usize)
}
