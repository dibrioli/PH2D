//! **O borrão por GRUPO** (doc 132 §1, o P1 do Motion): os dois passes do borrão gaussiano
//! separável — `cs_blur_h` e `cs_op_v` (o V com o fecho, `compose_blur`) — com a fila de taps e os
//! pesos na memória do GRUPO. Um grupo de 64 pixels de uma linha (ou coluna) carrega UMA vez os
//! `64 + 2h` taps e os `2h + 1` pesos, e cada pixel soma da memória do grupo.
//!
//! O passe por PIXEL que estava aqui fazia `2h + 1` leituras de textura E `2h + 1` exponenciais por
//! pixel e passe (33 + 33 na Glow de σ 5,3). Os mesmos taps, os mesmos pesos, a mesma ordem de soma:
//! o MESMO byte em 8 pilhas (gate `o_borrao_por_grupo_da_o_mesmo_byte`, commit `2cfacfe11f`), e a
//! pilha de «cada forma» a 4 096 discos `3,57 → 2,96 ms` (Outer Glow), `4,92 → 3,93` (Drop Shadow),
//! `2,18 → 1,83` (Inner Shadow); a camada inteira de 1600×972 `0,37 → 0,28 ms` (doc 132 §7).
//!
//! Concatenado DEPOIS do `FX_STACK_MID_WGSL` (consome o `dst`, o `tap_fonte`, o `tap_mid`, a `gauss`,
//! o `shift_split` e o `compose_blur`).

use crate::fx_stack::MAX_HALF;

/// Os pixels de uma linha (ou coluna) por grupo.
pub(crate) const GRUPO: u32 = 64;

/// O WGSL dos dois passes, com os tamanhos da memória do grupo derivados de [`MAX_HALF`] (o
/// `g.half` nunca o passa: [`crate::kernel_half`] corta lá).
pub(crate) fn borrao_grupo_wgsl() -> String {
    let taps = GRUPO + 2 * MAX_HALF + 1;
    let pesos = 2 * MAX_HALF + 2;
    format!(
        r"
const BG_N: u32 = {GRUPO}u;
var<workgroup> bg_tap: array<vec4<f32>, {taps}>;
var<workgroup> bg_peso: array<f32, {pesos}>;

// Os pesos do degrau, um por tap (o `k0` e o `n` do laço de sempre).
fn bg_pesos(lid: u32, k0: i32, n: u32, fr: f32) {{
    for (var j = lid; j < n; j = j + BG_N) {{ bg_peso[j] = gauss(f32(k0 + i32(j)) + fr); }}
}}

@compute @workgroup_size({GRUPO}, 1, 1)
fn cs_blur_h(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) lid: u32,
) {{
    let h = i32(g.half);
    let s = shift_split(g.off_x);
    let k0 = select(-h, -h - 1, s.y > 0.0);
    let n = u32(h - k0 + 1);
    let y = i32(wg.y);
    let x = i32(wg.x * BG_N + lid);
    // O tap `k` do pixel `x` é `x − ⌊off⌋ + k`: o grupo lê de `x0 − ⌊off⌋ + k0` em diante.
    let ini = i32(wg.x * BG_N) - i32(s.x) + k0;
    for (var j = lid; j < BG_N + n - 1u; j = j + BG_N) {{ bg_tap[j] = tap_fonte(t0, ini + i32(j), y); }}
    bg_pesos(lid, k0, n, s.y);
    workgroupBarrier();
    if (u32(x) >= g.dims.x || u32(y) >= g.dims.y) {{ return; }}
    var acc = vec4<f32>(0.0);
    var wsum = 0.0;
    for (var j = 0u; j < n; j = j + 1u) {{
        let wt = bg_peso[j];
        acc = acc + bg_tap[lid + j] * wt;
        wsum = wsum + wt;
    }}
    var b = acc / wsum;
    if (wsum < 1.0e-6) {{
        let base = x - i32(s.x);
        b = mix(tap_fonte(t0, base, y), tap_fonte(t0, base - 1, y), s.y);
    }}
    textureStore(dst, vec2<i32>(x, y), b);
}}

@compute @workgroup_size(1, {GRUPO}, 1)
fn cs_op_v(
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) lid: u32,
) {{
    let h = i32(g.half);
    let s = shift_split(g.off_y);
    let k0 = select(-h, -h - 1, s.y > 0.0);
    let n = u32(h - k0 + 1);
    let x = i32(wg.x);
    let y = i32(wg.y * BG_N + lid);
    let ini = i32(wg.y * BG_N) - i32(s.x) + k0;
    for (var j = lid; j < BG_N + n - 1u; j = j + BG_N) {{ bg_tap[j] = tap_mid(t1, x, ini + i32(j)); }}
    bg_pesos(lid, k0, n, s.y);
    workgroupBarrier();
    if (u32(x) >= g.dims.x || u32(y) >= g.dims.y) {{ return; }}
    var acc = vec4<f32>(0.0);
    var wsum = 0.0;
    for (var j = 0u; j < n; j = j + 1u) {{
        let wt = bg_peso[j];
        acc = acc + bg_tap[lid + j] * wt;
        wsum = wsum + wt;
    }}
    var b = acc / wsum;
    if (wsum < 1.0e-6) {{
        let base = y - i32(s.x);
        b = mix(tap_mid(t1, x, base), tap_mid(t1, x, base - 1), s.y);
    }}
    let id = vec3<u32>(u32(x), u32(y), 0u);
    textureStore(dst, vec2<i32>(x, y), compose_blur(id, b));
}}
"
    )
}
