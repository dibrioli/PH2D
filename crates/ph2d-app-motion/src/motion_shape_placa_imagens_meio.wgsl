// ⭐⭐ P4 (doc 132 §4 do Motion) — AS IMAGENS DE UM GRUPO NA PLACA DE FORMAS, como o SEGUNDO MEIO
// do passe de formas (o contrato está em `ph2d-shape-gpu/src/ordenado.wgsl`): o mesmo texto serve o
// pipeline das imagens sozinhas (`motion_shape_placa_imagens.wgsl`) e o desenho ORDENADO das corridas
// `Normal` com as duas médias (doc 132 §4.1 (3)). A conta é a da cena vectorial da CPU
// (`motion_shape_mistura::draw_quad`):
// - a COBERTURA é a do caminho de quatro rectas (o quad `[−½, ½]²` pela pose), pela `contribuicao`
//   do Vello (`area_do_vello.wgsl`);
// - a AMOSTRA é a do `fine.wgsl` do Vello 0.10 (`CMD_IMAGE`: `Low` = ponto, `Medium` = bilinear dos
//   quatro texels pré-multiplicados, `extend` `Pad`, presa ao RECORTE) — Copyright 2022 the Vello
//   Authors, Apache-2.0 OR MIT OR Unlicense — com os bytes CODIFICADOS (o tom das formas): o átlas é
//   `Rgba8UnormSrgb`, a leitura descodifica, e a conta volta a codificar;
// - a TINTA é a pilha de camadas da W6 (doc 118 §7) à letra: a cor por baixo guardada em 8 bits, a
//   imagem por cima em `Multiply`+`SrcIn`, a alfa da tinta na camada de fora — e cada camada
//   multiplica pela máscara do quad, como o `END_CLIP` do Vello.
//
// Quem junta este texto traz `contribuicao`, `aplica`, `codifica`, as palavras do `RenderInstance` e
// os bindings `vista`, `linhas_img` e `arte` (`motion_shape_placa_imagens.rs`).

struct Vista {
    lin: vec4<f32>,
    t: vec2<f32>,
    alvo: vec2<f32>,
    // O bit `tag` diz que a tag de filtro amplia por PONTO (`Low`); o bit 0 é o projecto.
    ponto: array<vec4<u32>, 2>,
    // `1` ⇒ as linhas deste troço não têm textura que alguma loja conheça: a imagem não desenha.
    mudo: u32,
}

// O vértice de uma linha: `extra_f` = o recorte em texels (canto, extensão); `extra_u.xy` = (a textura
// é pré-multiplicada, a amostra é por ponto).
struct SegundoVs {
    pos: vec4<f32>,
    lin: vec4<f32>,
    t: vec2<f32>,
    tint: vec4<f32>,
    extra_f: vec4<f32>,
    extra_u: vec4<u32>,
}

fn palavra(ii: u32, k: u32) -> f32 {
    return bitcast<f32>(linhas_img[ii * PALAVRAS + k]);
}
fn par(ii: u32, k: u32) -> vec2<f32> {
    return vec2<f32>(palavra(ii, k), palavra(ii, k + 1u));
}
fn quatro(ii: u32, k: u32) -> vec4<f32> {
    return vec4<f32>(palavra(ii, k), palavra(ii, k + 1u), palavra(ii, k + 2u), palavra(ii, k + 3u));
}

// O pedaço de um `uv_cell` dentro do rectângulo (`ph2d_eval_motion::uv_do_pedaco`): a identidade
// devolve o rectângulo AO BIT.
fn pedaco_do_uv(uv: vec4<f32>, cel: vec4<f32>) -> vec4<f32> {
    if all(cel == vec4<f32>(1.0, 1.0, 0.0, 0.0)) {
        return uv;
    }
    let w = uv.z - uv.x;
    let h = uv.w - uv.y;
    return vec4<f32>(uv.x + w * cel.z, uv.y + h * cel.w, uv.x + w * (cel.z + cel.x), uv.y + h * (cel.w + cel.y));
}

fn segundo_vs(vi: u32, ii: u32) -> SegundoVs {
    var out: SegundoVs;
    out.pos = vec4<f32>(2.0, 2.0, 0.0, 1.0);
    if vista.mudo != 0u {
        return out;
    }
    let size = par(ii, W_SIZE);
    let tint = quatro(ii, W_TINT);
    // A linha de FORMA da faixa chega muda (tamanho 0) e a tinta apagada não pinta (`Tinta::Apagada`).
    if tint.a <= 0.0 || (size.x == 0.0 && size.y == 0.0) {
        return out;
    }
    // O recorte — a conta do `motion_leaf_images::crop`, com as dimensões do nível 0.
    let uv = pedaco_do_uv(quatro(ii, W_UV), quatro(ii, W_UV_XFORM));
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
    out.extra_f = vec4<f32>(lo, ext);
    let tag = linhas_img[ii * PALAVRAS + W_SAMPLING] & 0xffu;
    let ponto = (vista.ponto[tag / 128u][(tag / 32u) % 4u] >> (tag % 32u)) & 1u;
    out.extra_u = vec4<u32>(select(0u, 1u, palavra(ii, W_PREMUL) > 0.5), ponto, 0u, 0u);
    return out;
}

// O byte CODIFICADO que a textura guarda: a leitura de um `…Srgb` descodifica, e o Vello mistura
// os bytes crus — `codifica` é a do `ph2d_render::CODIFICA_WGSL` (prefixada), a MESMA da leitura que
// dá a arte à rota da CPU.

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
fn segundo_cor(s: SegundoVs) -> vec4<f32> {
    let xy = floor(s.pos.xy);
    let c0 = aplica(s.lin, s.t, vec2<f32>(-0.5, -0.5));
    let c1 = aplica(s.lin, s.t, vec2<f32>(0.5, -0.5));
    let c2 = aplica(s.lin, s.t, vec2<f32>(0.5, 0.5));
    let c3 = aplica(s.lin, s.t, vec2<f32>(-0.5, 0.5));
    let soma = contribuicao(c0, c1, xy) + contribuicao(c1, c2, xy) + contribuicao(c2, c3, xy) + contribuicao(c3, c0, xy);
    let area = min(abs(soma), 1.0);
    if area <= 0.0 {
        discard;
    }
    // O centro do pixel pelo afim inverso, em texels do recorte: a imagem tem `ext` texels em `[−½, ½]`.
    let d = xy + vec2<f32>(0.5) - s.t;
    let det = s.lin.x * s.lin.w - s.lin.z * s.lin.y;
    let q = vec2<f32>(s.lin.w * d.x - s.lin.z * d.y, s.lin.x * d.y - s.lin.y * d.x) / det;
    let off = s.extra_f.xy;
    let ext = s.extra_f.zw;
    let u = clamp((q + vec2<f32>(0.5)) * ext, vec2<f32>(0.0), ext);
    let maximo = off + ext - vec2<f32>(1.0);
    var img: vec4<f32>;
    if s.extra_u.y != 0u {
        img = texel(clamp(u + off, off, maximo), s.extra_u.x);
    } else {
        let a_uv = u + off - vec2<f32>(0.5);
        let preso = clamp(a_uv, off, maximo);
        let quad = vec4<f32>(floor(preso), ceil(preso));
        let fr = fract(a_uv);
        let ta = texel(quad.xy, s.extra_u.x);
        let tb = texel(quad.xw, s.extra_u.x);
        let tc = texel(quad.zy, s.extra_u.x);
        let td = texel(quad.zw, s.extra_u.x);
        img = mix(mix(ta, tb, fr.y), mix(tc, td, fr.y), fr.x);
    }
    let t = s.tint;
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
