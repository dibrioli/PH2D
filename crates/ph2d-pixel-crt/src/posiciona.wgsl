// Cópia nearest de `origem` (src) para o rectângulo `dst` em `off` do alvo; fora dele, preto.
// `decodifica = 1`: a origem traz níveis crus e o alvo é *Srgb — desfaz-se a curva para que a
// codificação da placa devolva o MESMO byte.

struct P {
    src: vec2<u32>,
    dst: vec2<u32>,
    off: vec2<u32>,
    decodifica: u32,
    _pad: u32,
};

@group(0) @binding(0) var origem: texture_2d<f32>;
@group(0) @binding(1) var<uniform> p: P;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
}

fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let q = vec2<u32>(pos.xy);
    if any(q < p.off) || any(q >= p.off + p.dst) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let r = q - p.off;
    let s = ((2u * r + 1u) * p.src) / (2u * p.dst);
    var c = textureLoad(origem, s, 0);
    if p.decodifica == 1u {
        c = vec4<f32>(linear(c.r), linear(c.g), linear(c.b), c.a);
    }
    return c;
}
