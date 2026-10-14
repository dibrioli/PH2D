// ⭐ doc 132 §4.1 (3) do Motion — AS DUAS MÉDIAS NA ORDEM DAS LINHAS: um desenho percorre a faixa
// de um sink nos dois buffers (`Corrida::ordenada`); a instância `ii` é a linha nas FORMAS, e quando
// ela não tem geometria a mesma linha é a do SEGUNDO MEIO (o meio que o chamador junta a este módulo,
// `ShapePass::com_segundo_meio`). A mistura de hardware compõe as primitivas pela ordem de submissão,
// logo o `Normal` sai na ordem das linhas sem um desenho por linha.
//
// O CONTRATO do meio (WGSL dele, com os bindings no grupo 2):
//   struct SegundoVs { pos: vec4<f32>, lin: vec4<f32>, t: vec2<f32>, tint: vec4<f32>,
//                      extra_f: vec4<f32>, extra_u: vec4<u32> }
//   fn segundo_vs(vi: u32, ii: u32) -> SegundoVs      // `pos.x = 2` = nada
//   fn segundo_cor(s: SegundoVs) -> vec4<f32>         // pré-multiplicada; descarta o que não cobre
//
// A linha do meio é `ii + (vi / 6) − VIES_DO_SEGUNDO`: o deslocamento entre as duas faixas viaja no
// `first_vertex` do desenho (`6 · (deslocamento + VIES)`), sem uniforme por desenho.

const VIES_DO_SEGUNDO: i32 = 268435456;

struct VsOrdenado {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) lin: vec4<f32>,
    @location(1) @interpolate(flat) t: vec2<f32>,
    @location(2) @interpolate(flat) fill: vec2<u32>,
    @location(3) @interpolate(flat) stroke: vec2<u32>,
    @location(4) @interpolate(flat) tint: vec4<f32>,
    @location(5) @interpolate(flat) stroke_color: vec4<f32>,
    @location(6) @interpolate(flat) even_odd: u32,
    @location(7) @interpolate(flat) eixo_rg: vec4<u32>,
    @location(8) @interpolate(flat) tela: u32,
    @location(9) @interpolate(flat) extra_f: vec4<f32>,
    @location(10) @interpolate(flat) extra_u: vec4<u32>,
    // 0 = a forma, 1 = o segundo meio.
    @location(11) @interpolate(flat) meio: u32,
}

@vertex
fn vs_ordenado(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOrdenado {
    var o: VsOrdenado;
    let canto = vi % 6u;
    if registo_de(instances[ii].geometry) != 0xffffffffu {
        let f = vs_da_copia(canto, ii);
        o.pos = f.pos;
        o.lin = f.lin;
        o.t = f.t;
        o.fill = f.fill;
        o.stroke = f.stroke;
        o.tint = f.tint;
        o.stroke_color = f.stroke_color;
        o.even_odd = f.even_odd;
        o.eixo_rg = f.eixo_rg;
        o.tela = f.tela;
        o.meio = 0u;
        return o;
    }
    let s = segundo_vs(canto, u32(i32(ii) + i32(vi / 6u) - VIES_DO_SEGUNDO));
    o.pos = s.pos;
    o.lin = s.lin;
    o.t = s.t;
    o.tint = s.tint;
    o.extra_f = s.extra_f;
    o.extra_u = s.extra_u;
    o.meio = 1u;
    return o;
}

@fragment
fn fs_ordenado(in: VsOrdenado) -> @location(0) vec4<f32> {
    if in.meio != 0u {
        return segundo_cor(SegundoVs(in.pos, in.lin, in.t, in.tint, in.extra_f, in.extra_u));
    }
    var f: VsOut;
    f.pos = in.pos;
    f.lin = in.lin;
    f.t = in.t;
    f.fill = in.fill;
    f.stroke = in.stroke;
    f.tint = in.tint;
    f.stroke_color = in.stroke_color;
    f.even_odd = in.even_odd;
    f.eixo_rg = in.eixo_rg;
    f.tela = in.tela;
    return cor_da_copia(f);
}
