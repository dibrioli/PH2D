// ⭐⭐ P4 (doc 132 §4 do Motion) — o pipeline das IMAGENS SOZINHAS (as corridas de Soma, Tela e
// Multiplica, e as páginas das células): as entradas sobre o meio (`motion_shape_placa_imagens_meio.wgsl`),
// que é o mesmo texto do desenho ORDENADO das corridas `Normal` com as duas médias.

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    // O afim local → pixel da linha.
    @location(0) @interpolate(flat) lin: vec4<f32>,
    @location(1) @interpolate(flat) t: vec2<f32>,
    @location(2) @interpolate(flat) tint: vec4<f32>,
    // O recorte em texels: canto (xy) e extensão (zw).
    @location(3) @interpolate(flat) recorte: vec4<f32>,
    // (a textura já é pré-multiplicada, a amostra é por ponto)
    @location(4) @interpolate(flat) modo: vec2<u32>,
}

fn aplica(lin: vec4<f32>, t: vec2<f32>, p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(lin.x * p.x + lin.z * p.y, lin.y * p.x + lin.w * p.y) + t;
}

@vertex
fn vs_imagem(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    let s = segundo_vs(vi, ii);
    var out: VsOut;
    out.pos = s.pos;
    out.lin = s.lin;
    out.t = s.t;
    out.tint = s.tint;
    out.recorte = s.extra_f;
    out.modo = s.extra_u.xy;
    return out;
}

fn cor_da_imagem(in: VsOut) -> vec4<f32> {
    return segundo_cor(SegundoVs(in.pos, in.lin, in.t, in.tint, in.recorte, vec4<u32>(in.modo, 0u, 0u)));
}

@fragment
fn fs_imagem(in: VsOut) -> @location(0) vec4<f32> {
    return cor_da_imagem(in);
}

// A cópia de uma corrida com MISTURA: presa a `[0, 1]` como o Vello a grava (o `cor_de_mistura` das
// formas).
fn cor_de_mistura(in: VsOut) -> vec4<f32> {
    let c = cor_da_imagem(in);
    let a = min(c.a, 1.0);
    return vec4<f32>(min(c.rgb, vec3<f32>(a)), a);
}

@fragment
fn fs_imagem_mistura(in: VsOut) -> @location(0) vec4<f32> {
    return cor_de_mistura(in);
}
