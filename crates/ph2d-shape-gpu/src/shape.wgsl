// O PASSE DE FORMAS INSTANCIADO (doc 121 do Motion) — N cópias de geometrias vectoriais numa
// chamada, com a área de cada pixel calculada EXACTAMENTE como o rasterizador fino do Vello.
//
// A conta da área (`contribuicao`) é portada de `vello_shaders/shader/fine.wgsl` (`fill_path`,
// o modo `AaConfig::Area` que esta casa usa):
//   Copyright 2022 the Vello Authors — SPDX-License-Identifier: Apache-2.0 OR MIT OR Unlicense
// A única mudança é a de CONTEXTO: o Vello corre por LADRILHO, com um `backdrop` e um `y_edge` que
// carregam a contribuição dos segmentos cortados à esquerda do ladrilho; aqui cada pixel soma TODOS
// os segmentos da geometria, e um segmento que acaba à esquerda do pixel dá `a = 1` pela mesma
// fórmula (`xmax < 0`), logo os dois termos de ladrilho desaparecem.

struct View {
    // pixel = lin · mundo + t, com lin = [[a, c], [b, d]] guardado como (a, b, c, d).
    lin: vec4<f32>,
    t: vec2<f32>,
    alvo: vec2<f32>,
}

struct Instance {
    pos: vec2<f32>,
    size: vec2<f32>,
    basis: vec4<f32>,
    anchor: vec2<f32>,
    geometry: u32,
    _pad: u32,
    tint: vec4<f32>,
}

struct Record {
    bbox: vec4<f32>,
    stroke_color: vec4<f32>,
    tol: array<vec4<f32>, 2>,
    ranges: array<vec4<u32>, 8>,
    flags: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<storage, read> instances: array<Instance>;
@group(0) @binding(2) var<storage, read> records: array<Record>;
@group(0) @binding(3) var<storage, read> segs: array<vec4<f32>>;
// Os handles de geometria, ORDENADOS — a posição de um handle é o índice do registo dele.
@group(0) @binding(4) var<storage, read> handles: array<u32>;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    // O afim local → pixel da cópia: (a, b, c, d) e a translação.
    @location(0) @interpolate(flat) lin: vec4<f32>,
    @location(1) @interpolate(flat) t: vec2<f32>,
    @location(2) @interpolate(flat) fill: vec2<u32>,
    @location(3) @interpolate(flat) stroke: vec2<u32>,
    @location(4) @interpolate(flat) tint: vec4<f32>,
    @location(5) @interpolate(flat) stroke_color: vec4<f32>,
    @location(6) @interpolate(flat) even_odd: u32,
}

// O índice do registo de um handle, ou `0xffffffff` se a geometria não existe.
fn registo_de(handle: u32) -> u32 {
    var lo = 0u;
    var hi = arrayLength(&handles);
    while lo < hi {
        let meio = (lo + hi) / 2u;
        let h = handles[meio];
        if h == handle {
            return meio;
        }
        if h < handle {
            lo = meio + 1u;
        } else {
            hi = meio;
        }
    }
    return 0xffffffffu;
}

fn aplica(lin: vec4<f32>, t: vec2<f32>, p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(lin.x * p.x + lin.z * p.y, lin.y * p.x + lin.w * p.y) + t;
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    var out: VsOut;
    let inst = instances[ii];
    let r = registo_de(inst.geometry);
    if r == 0xffffffffu {
        // Sem geometria: um triângulo degenerado, fora de tudo.
        out.pos = vec4<f32>(2.0, 2.0, 0.0, 1.0);
        return out;
    }
    let rec = records[r];
    // mundo = pos + basis · (anchor + q · size) — a MESMA pose da sprite e do `instance_pose`.
    let b = inst.basis;
    let w_lin = vec4<f32>(b.x * inst.size.x, b.y * inst.size.x, b.z * inst.size.y, b.w * inst.size.y);
    let w_t = inst.pos + vec2<f32>(b.x * inst.anchor.x + b.z * inst.anchor.y, b.y * inst.anchor.x + b.w * inst.anchor.y);
    let v = view.lin;
    let lin = vec4<f32>(
        v.x * w_lin.x + v.z * w_lin.y,
        v.y * w_lin.x + v.w * w_lin.y,
        v.x * w_lin.z + v.z * w_lin.w,
        v.y * w_lin.z + v.w * w_lin.w,
    );
    let t = aplica(view.lin, view.t, w_t);
    // A maior escala do afim (o maior valor singular): quantos pixels vale uma unidade local.
    let e = 0.5 * (lin.x + lin.w);
    let f = 0.5 * (lin.x - lin.w);
    let g = 0.5 * (lin.y + lin.z);
    let h = 0.5 * (lin.y - lin.z);
    let escala = sqrt(e * e + h * h) + sqrt(f * f + g * g);
    // O nível mais GROSSO cujo erro no ecrã cabe na tolerância do Vello (0,25 px).
    var nivel = 7u;
    for (var k = 0u; k < 8u; k += 1u) {
        if rec.tol[k / 4u][k % 4u] * escala <= 0.25 {
            nivel = k;
            break;
        }
    }
    let rg = rec.ranges[nivel];
    // O quad: a caixa da forma no ECRÃ, arredondada PARA FORA ao pixel inteiro.
    // ⚠️ **Sem margem, e é medido:** a caixa é a dos SEGMENTOS aplanados, e um pixel só tem
    // cobertura se um segmento (ou o interior entre eles) lhe toca — logo o `floor`/`ceil` já
    // inclui todo pixel de borda. A 1.ª redacção alargava um pixel de cada lado, e a mutação que o
    // apagava SOBREVIVEU à paridade de pixel: não era lei, era trabalho a mais.
    let c0 = aplica(lin, t, rec.bbox.xy);
    let c1 = aplica(lin, t, rec.bbox.zy);
    let c2 = aplica(lin, t, rec.bbox.xw);
    let c3 = aplica(lin, t, rec.bbox.zw);
    let lo = floor(min(min(c0, c1), min(c2, c3)));
    let hi = ceil(max(max(c0, c1), max(c2, c3)));
    var canto = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    let px = mix(lo, hi, canto[vi]);
    out.pos = vec4<f32>(px.x / view.alvo.x * 2.0 - 1.0, 1.0 - px.y / view.alvo.y * 2.0, 0.0, 1.0);
    out.lin = lin;
    out.t = t;
    out.fill = rg.xy;
    out.stroke = rg.zw;
    out.tint = inst.tint;
    out.stroke_color = rec.stroke_color;
    out.even_odd = rec.flags & 1u;
    return out;
}

// A contribuição de UM segmento (em pixels) para a área do pixel cujo canto é `xy` — portada do
// `fill_path` do Vello (ver o cabeçalho).
fn contribuicao(p0: vec2<f32>, p1: vec2<f32>, xy: vec2<f32>) -> f32 {
    let y = p0.y - xy.y;
    let delta = p1 - p0;
    let y0 = clamp(y, 0.0, 1.0);
    let y1 = clamp(y + delta.y, 0.0, 1.0);
    let dy = y0 - y1;
    if dy == 0.0 {
        return 0.0;
    }
    let vec_y_recip = 1.0 / delta.y;
    let t0 = (y0 - y) * vec_y_recip;
    let t1 = (y1 - y) * vec_y_recip;
    let startx = p0.x - xy.x;
    let x0 = startx + t0 * delta.x;
    let x1 = startx + t1 * delta.x;
    let xmin0 = min(x0, x1);
    let xmax0 = max(x0, x1);
    let xmin = min(xmin0, 1.0) - 1.0e-6;
    let xmax = xmax0;
    let b = min(xmax, 1.0);
    let c = max(b, 0.0);
    let d = max(xmin, 0.0);
    let a = (b + 0.5 * (d * d - c * c) - xmin) / (xmax - xmin);
    return a * dy;
}

fn area(inicio: u32, n: u32, lin: vec4<f32>, t: vec2<f32>, xy: vec2<f32>) -> f32 {
    var s = 0.0;
    for (var i = 0u; i < n; i += 1u) {
        let seg = segs[inicio + i];
        s += contribuicao(aplica(lin, t, seg.xy), aplica(lin, t, seg.zw), xy);
    }
    return s;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let xy = floor(in.pos.xy);
    var af = area(in.fill.x, in.fill.y, in.lin, in.t, xy);
    // As duas regras, à letra do Vello.
    if in.even_odd != 0u {
        af = abs(af - 2.0 * round(0.5 * af));
    } else {
        af = min(abs(af), 1.0);
    }
    // O traço é sempre não-nulo: o contorno expandido é um preenchimento.
    let as_ = min(abs(area(in.stroke.x, in.stroke.y, in.lin, in.t, xy)), 1.0);
    let f = vec4<f32>(in.tint.rgb * in.tint.a, in.tint.a) * af;
    let s = vec4<f32>(in.stroke_color.rgb * in.stroke_color.a, in.stroke_color.a) * as_;
    // O traço POR CIMA do preenchimento — a ordem dos dois `fill` do Vello, composta aqui.
    let c = s + f * (1.0 - s.a);
    if c.a <= 0.0 {
        discard;
    }
    return c;
}
