// B5 — os passes de ECRÃ INTEIRO das corridas com mistura (`mistura.rs`): o CORTE a `1` (mistura
// `Min` com uma fonte branca) e a COLAGEM de um grupo isolado por cima do alvo.

@vertex
fn vs_tela(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_um() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}

@group(0) @binding(0) var grupo: texture_2d<f32>;

@fragment
fn fs_grupo(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    return textureLoad(grupo, vec2<i32>(p.xy), 0);
}
