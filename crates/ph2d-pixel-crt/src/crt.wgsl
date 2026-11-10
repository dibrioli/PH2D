// A CRT do Pixel Art — a MESMA lei de `crt_cpu` (doc do módulo em `lib.rs`).
// Passe de ecrã cheio: um triângulo, a origem lida por `textureLoad` (nearest exacto), o alvo do
// tamanho da saída (`w·escala × h·escala`).

struct Params {
    w_src: u32,
    h_src: u32,
    escala: u32,
    k: i32,
    scanline: f32,
    mascara: f32,
    brilho: f32,
    bloom: f32,
};

@group(0) @binding(0) var origem: texture_2d<f32>;
@group(0) @binding(1) var<uniform> p: Params;

const Q: u32 = 14u;
const UM: i32 = 16384;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
}

fn cx_q(x: u32, lado: u32) -> i32 {
    return ((2 * i32(x) + 1 - i32(lado)) << Q) / i32(lado);
}

// -1 = fora (preto).
fn curva_eixo(x: u32, lado: u32, r2: i32) -> i32 {
    let u = cx_q(x, lado);
    let f = UM + ((p.k * r2) >> 12u);
    let du = (u * f) >> Q;
    if du < -UM || du >= UM {
        return -1;
    }
    return ((du + UM) * i32(lado)) >> (Q + 1u);
}

fn nivel(t: f32) -> f32 {
    return floor(t * 255.0 + 0.5);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let x = u32(pos.x);
    let y = u32(pos.y);
    let ow = p.w_src * p.escala;
    let oh = p.h_src * p.escala;
    let u = cx_q(x, ow);
    let v = cx_q(y, oh);
    let r2 = i32((u32(u * u) + u32(v * v)) >> Q);
    let sx = curva_eixo(x, ow, r2);
    let sy = curva_eixo(y, oh, r2);
    if sx < 0 || sy < 0 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let tx = u32(sx) / p.escala;
    let ty = u32(sy) / p.escala;
    let e = select(tx - 1u, 0u, tx == 0u);
    let d = min(tx + 1u, p.w_src - 1u);
    let s4 = textureLoad(origem, vec2<u32>(tx, ty), 0);
    let e4 = textureLoad(origem, vec2<u32>(e, ty), 0);
    let d4 = textureLoad(origem, vec2<u32>(d, ty), 0);
    var linha = 1.0;
    if (y & 1u) == 1u {
        linha = 1.0 - p.scanline;
    }
    let col = x % 3u;
    var out = vec3<f32>(0.0);
    for (var c = 0u; c < 3u; c = c + 1u) {
        var m = 1.0 - p.mascara;
        if c == col {
            m = 1.0;
        }
        let g = p.brilho * linha * m;
        let s = nivel(s4[c]);
        let val = s * g + p.bloom * (nivel(e4[c]) + s + nivel(d4[c])) / 3.0;
        out[c] = clamp(floor(val + 0.5), 0.0, 255.0) / 255.0;
    }
    return vec4<f32>(out, s4.a);
}
