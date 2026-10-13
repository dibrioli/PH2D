// ⭐⭐ P4 (doc 132 §4 do Motion) — AS IMAGENS DE UM GRUPO NA PLACA DE FORMAS: a linha de imagem de
// uma saída que mistura em grupo, desenhada na MESMA camada e na MESMA corrida que as formas, com a
// conta que a cena vectorial da CPU faz (`motion_shape_mistura::draw_quad`):
// - a COBERTURA é a do caminho de quatro rectas (o quad `[−½, ½]²` pela pose), pela `contribuicao`
//   do Vello (`area_do_vello.wgsl`, prefixada pela placa);
// - a AMOSTRA é a do `fine.wgsl` do Vello 0.10 (`CMD_IMAGE`: `Low` = ponto, `Medium` = bilinear dos
//   quatro texels pré-multiplicados, `extend` `Pad`, presa ao RECORTE) — Copyright 2022 the Vello
//   Authors, Apache-2.0 OR MIT OR Unlicense — com os bytes CODIFICADOS (o tom das formas): o átlas é
//   `Rgba8UnormSrgb`, a leitura descodifica, e a conta volta a codificar;
// - a TINTA é a pilha de camadas da W6 (doc 118 §7) à letra: a cor por baixo guardada em 8 bits, a
//   imagem por cima em `Multiply`+`SrcIn`, a alfa da tinta na camada de fora — e cada camada
//   multiplica pela máscara do quad, como o `END_CLIP` do Vello.
//
// O prefixo (`motion_shape_placa_imagens.rs`) traz `contribuicao` e as palavras do `RenderInstance`.

struct Vista {
    lin: vec4<f32>,
    t: vec2<f32>,
    alvo: vec2<f32>,
    // O bit `tag` diz que a tag de filtro amplia por PONTO (`Low`); o bit 0 é o projecto.
    ponto: array<vec4<u32>, 2>,
}

@group(0) @binding(0) var<uniform> vista: Vista;
@group(0) @binding(1) var<storage, read> linhas: array<u32>;
@group(0) @binding(2) var arte: texture_2d<f32>;

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

fn palavra(ii: u32, k: u32) -> f32 {
    return bitcast<f32>(linhas[ii * PALAVRAS + k]);
}
fn par(ii: u32, k: u32) -> vec2<f32> {
    return vec2<f32>(palavra(ii, k), palavra(ii, k + 1u));
}
fn quatro(ii: u32, k: u32) -> vec4<f32> {
    return vec4<f32>(palavra(ii, k), palavra(ii, k + 1u), palavra(ii, k + 2u), palavra(ii, k + 3u));
}

fn aplica(lin: vec4<f32>, t: vec2<f32>, p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(lin.x * p.x + lin.z * p.y, lin.y * p.x + lin.w * p.y) + t;
}

// O pedaço de um `uv_cell` dentro do rectângulo (`ph2d_eval_motion::uv_do_pedaco`): a identidade
// devolve o rectângulo AO BIT.
fn pedaco(uv: vec4<f32>, cel: vec4<f32>) -> vec4<f32> {
    if all(cel == vec4<f32>(1.0, 1.0, 0.0, 0.0)) {
        return uv;
    }
    let w = uv.z - uv.x;
    let h = uv.w - uv.y;
    return vec4<f32>(uv.x + w * cel.z, uv.y + h * cel.w, uv.x + w * (cel.z + cel.x), uv.y + h * (cel.w + cel.y));
}

@vertex
fn vs_imagem(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(2.0, 2.0, 0.0, 1.0);
    let size = par(ii, W_SIZE);
    let tint = quatro(ii, W_TINT);
    // A linha de FORMA da faixa chega muda (tamanho 0) e a tinta apagada não pinta (`Tinta::Apagada`).
    if tint.a <= 0.0 || (size.x == 0.0 && size.y == 0.0) {
        return out;
    }
    // O recorte — a conta do `motion_leaf_images::crop`, com as dimensões do nível 0.
    let uv = pedaco(quatro(ii, W_UV), quatro(ii, W_UV_XFORM));
    let dim = vec2<f32>(textureDimensions(arte, 0));
    let lo = floor(clamp(min(uv.xy, uv.zw), vec2<f32>(0.0), vec2<f32>(1.0)) * dim);
    let hi = ceil(clamp(max(uv.xy, uv.zw), vec2<f32>(0.0), vec2<f32>(1.0)) * dim);
    let ext = min(hi - lo, dim);
    if ext.x <= 0.0 || ext.y <= 0.0 {
        return out;
    }
    // A pose: mundo = P + basis · (anchor + q · size) — a do `instance_pose` e a do `copia_de`.
    let b = quatro(ii, W_BASIS);
    let p = par(ii, W_P);
    let a = par(ii, W_ANCHOR);
    let w_lin = vec4<f32>(b.x * size.x, b.y * size.x, b.z * size.y, b.w * size.y);
    let w_t = p + vec2<f32>(b.x * a.x + b.z * a.y, b.y * a.x + b.w * a.y);
    let v = vista.lin;
    let lin = vec4<f32>(
        v.x * w_lin.x + v.z * w_lin.y,
        v.y * w_lin.x + v.w * w_lin.y,
        v.x * w_lin.z + v.z * w_lin.w,
        v.y * w_lin.z + v.w * w_lin.w,
    );
    let t = aplica(vista.lin, vista.t, w_t);
    let c0 = aplica(lin, t, vec2<f32>(-0.5, -0.5));
    let c1 = aplica(lin, t, vec2<f32>(0.5, -0.5));
    let c2 = aplica(lin, t, vec2<f32>(0.5, 0.5));
    let c3 = aplica(lin, t, vec2<f32>(-0.5, 0.5));
    let caixa_lo = floor(min(min(c0, c1), min(c2, c3)));
    let caixa_hi = ceil(max(max(c0, c1), max(c2, c3)));
    var canto = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    let px = mix(caixa_lo, caixa_hi, canto[vi]);
    out.pos = vec4<f32>(px.x / vista.alvo.x * 2.0 - 1.0, 1.0 - px.y / vista.alvo.y * 2.0, 0.0, 1.0);
    out.lin = lin;
    out.t = t;
    out.tint = tint;
    out.recorte = vec4<f32>(lo, ext);
    let tag = linhas[ii * PALAVRAS + W_SAMPLING] & 0xffu;
    let ponto = (vista.ponto[tag / 128u][(tag / 32u) % 4u] >> (tag % 32u)) & 1u;
    out.modo = vec2<u32>(select(0u, 1u, palavra(ii, W_PREMUL) > 0.5), ponto);
    return out;
}

// O byte CODIFICADO que a textura guarda: a leitura de um `…Srgb` descodifica, e o Vello mistura
// os bytes crus.
fn codifica(c: vec3<f32>) -> vec3<f32> {
    let baixo = c * 12.92;
    let alto = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(alto, baixo, c <= vec3<f32>(0.0031308));
}

fn texel(ij: vec2<f32>, premultiplicada: u32) -> vec4<f32> {
    let l = textureLoad(arte, vec2<i32>(ij), 0);
    let c = vec4<f32>(codifica(l.rgb), l.a);
    if premultiplicada != 0u {
        return c;
    }
    return vec4<f32>(c.rgb * c.a, c.a);
}

// O `pack4x8unorm` do Vello ao empilhar uma camada: o fundo guardado em 8 bits.
fn oito_bits(c: vec4<f32>) -> vec4<f32> {
    return floor(clamp(c, vec4<f32>(0.0), vec4<f32>(1.0)) * 255.0 + 0.5) / 255.0;
}

// `blend_mix_compose(fundo, fonte, Multiply | SrcIn)` do Vello (`blend.wgsl`).
fn multiplica_src_in(fundo: vec4<f32>, fonte: vec4<f32>) -> vec4<f32> {
    let cs = fonte.rgb / max(fonte.a, 1.0e-15);
    let cb = fundo.rgb / max(fundo.a, 1.0e-15);
    let misturada = mix(cs, cb * cs, fundo.a);
    let a = fonte.a * fundo.a;
    return vec4<f32>(a * misturada, min(a, 1.0));
}

// A cor pré-multiplicada da imagem neste pixel (descarta o que não cobre).
fn cor_da_imagem(in: VsOut) -> vec4<f32> {
    let xy = floor(in.pos.xy);
    let c0 = aplica(in.lin, in.t, vec2<f32>(-0.5, -0.5));
    let c1 = aplica(in.lin, in.t, vec2<f32>(0.5, -0.5));
    let c2 = aplica(in.lin, in.t, vec2<f32>(0.5, 0.5));
    let c3 = aplica(in.lin, in.t, vec2<f32>(-0.5, 0.5));
    let s = contribuicao(c0, c1, xy) + contribuicao(c1, c2, xy) + contribuicao(c2, c3, xy) + contribuicao(c3, c0, xy);
    let area = min(abs(s), 1.0);
    if area <= 0.0 {
        discard;
    }
    // O centro do pixel pelo afim inverso, em texels do recorte: a imagem tem `ext` texels em `[−½, ½]`.
    let d = xy + vec2<f32>(0.5) - in.t;
    let det = in.lin.x * in.lin.w - in.lin.z * in.lin.y;
    let q = vec2<f32>(in.lin.w * d.x - in.lin.z * d.y, in.lin.x * d.y - in.lin.y * d.x) / det;
    let off = in.recorte.xy;
    let ext = in.recorte.zw;
    let u = clamp((q + vec2<f32>(0.5)) * ext, vec2<f32>(0.0), ext);
    let maximo = off + ext - vec2<f32>(1.0);
    var img: vec4<f32>;
    if in.modo.y != 0u {
        img = texel(clamp(u + off, off, maximo), in.modo.x);
    } else {
        let a_uv = u + off - vec2<f32>(0.5);
        let preso = clamp(a_uv, off, maximo);
        let quad = vec4<f32>(floor(preso), ceil(preso));
        let fr = fract(a_uv);
        let ta = texel(quad.xy, in.modo.x);
        let tb = texel(quad.xw, in.modo.x);
        let tc = texel(quad.zy, in.modo.x);
        let td = texel(quad.zw, in.modo.x);
        img = mix(mix(ta, tb, fr.y), mix(tc, td, fr.y), fr.x);
    }
    let t = in.tint;
    var x: vec4<f32>;
    if all(t == vec4<f32>(1.0)) {
        x = img * area;
    } else if all(t.rgb == vec3<f32>(1.0)) {
        x = img * area * area * min(t.a, 1.0);
    } else {
        let cor = floor(clamp(t.rgb, vec3<f32>(0.0), vec3<f32>(1.0)) * 255.0 + 0.5) / 255.0;
        let fundo = oito_bits(vec4<f32>(cor, 1.0) * area);
        x = multiplica_src_in(fundo, img * area * area) * area * min(t.a, 1.0);
    }
    if x.a <= 0.0 {
        discard;
    }
    return x;
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
