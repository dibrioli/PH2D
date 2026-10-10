// O PASSE DA LENTE sobre o acumulador do mundo (doc 129 §3.2): aberração → vinheta → grade →
// grão, ordem FIXA, a mesma conta de `ph2d_lens::aplica_cpu`.
//
// A fonte é uma CÓPIA do sub-rect da cena (não se lê e escreve a mesma textura), lida por
// `textureLoad` em bytes CRUS: o acumulador guarda sRGB PRÉ-MULTIPLICADO (o `over` no espaço do
// desenhista, `world_rt.rs`), e a des-pré-multiplicação tem de vir ANTES da descodificação — a
// vista sRGB do hardware descodificaria o valor ainda multiplicado. A escrita substitui o texel
// (sem mistura) com o mesmo alfa, de novo pré-multiplicado em sRGB.

struct LensU {
    // (x, y, w, h) do sub-rect da cena, em pixels do alvo.
    rect: vec4<f32>,
    // Aberração · vinheta · grade · grão: 1 = corre.
    liga: vec4<u32>,
    // (aberração linear, LUT, grão colorido, semente do grão).
    modo: vec4<u32>,
    // (centro.x, centro.y, escala do vermelho, escala do azul).
    ab: vec4<f32>,
    // (deslocamento linear x, y, —, —).
    ab_d: vec4<f32>,
    // (centro.x, centro.y, semi.x, semi.y) da vinheta.
    vin_geo: vec4<f32>,
    // (expoente, σ, |amount| ≤ 1, —).
    vin_par: vec4<f32>,
    // A cor para onde a vinheta puxa (RGB linear).
    vin_alvo: vec4<f32>,
    // (2^exposure, 2^contraste, 1 + saturação, lut_mix ≤ 1).
    grade: vec4<f32>,
    // As três linhas da matriz do balanço de branco.
    bal0: vec4<f32>,
    bal1: vec4<f32>,
    bal2: vec4<f32>,
    // (DOMAIN_MIN, n) e (DOMAIN_MAX, —) da LUT.
    lut_min: vec4<f32>,
    lut_max: vec4<f32>,
    // (amount × σ por amount, size, response, —).
    grao: vec4<f32>,
};

@group(0) @binding(0) var fonte: texture_2d<f32>;
@group(0) @binding(1) var lut: texture_3d<f32>;
@group(0) @binding(2) var<uniform> u: LensU;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    // Triângulo de tela cheia — o viewport é o sub-rect.
    var out: VsOut;
    let x = f32((vi << 1u) & 2u);
    let y = f32(vi & 2u);
    out.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

// Um texel da cópia, borda REPLICADA: RGB linear de alfa reto, e o alfa.
fn lz_texel(i: vec2<i32>) -> vec4<f32> {
    let wh = vec2<i32>(u.rect.zw);
    let c = textureLoad(fonte, clamp(i, vec2<i32>(0), wh - vec2<i32>(1)), 0);
    var cod = vec3<f32>(0.0);
    if (c.a > 0.0) {
        cod = min(c.rgb / c.a, vec3<f32>(1.0));
    }
    return vec4<f32>(lz_s2l3(cod), c.a);
}

// `ph2d_lens::aberracao::bilinear` sobre os centros dos pixels.
fn lz_bilinear(pos: vec2<f32>, canal: u32) -> f32 {
    let x = pos.x - 0.5;
    let y = pos.y - 0.5;
    let x0 = floor(x);
    let y0 = floor(y);
    let fx = x - x0;
    let fy = y - y0;
    let i = vec2<i32>(i32(x0), i32(y0));
    let a = lz_texel(i)[canal];
    let b = lz_texel(i + vec2<i32>(1, 0))[canal];
    let c = lz_texel(i + vec2<i32>(0, 1))[canal];
    let d = lz_texel(i + vec2<i32>(1, 1))[canal];
    let topo = a + (b - a) * fx;
    let base = c + (d - c) * fx;
    return topo + (base - topo) * fy;
}

fn lz_lut_em(r: u32, g: u32, b: u32) -> vec3<f32> {
    return textureLoad(lut, vec3<i32>(i32(r), i32(g), i32(b)), 0).rgb;
}

// `ph2d_lens::Cubo::tetraedrica` — `textureLoad`, sem a filtragem de hardware (pesos de 8 bits).
fn lz_tetra(c: vec3<f32>) -> vec3<f32> {
    let n = u32(u.lut_min.w);
    let top = f32(n - 1u);
    let t = clamp((c - u.lut_min.xyz) / (u.lut_max.xyz - u.lut_min.xyz), vec3<f32>(0.0), vec3<f32>(1.0)) * top;
    let i = min(vec3<u32>(floor(t)), vec3<u32>(n - 2u));
    let f = t - vec3<f32>(i);
    let r = f.x;
    let g = f.y;
    let b = f.z;
    let c000 = lz_lut_em(i.x, i.y, i.z);
    let c111 = lz_lut_em(i.x + 1u, i.y + 1u, i.z + 1u);
    var a: vec3<f32>;
    var bv: vec3<f32>;
    var w: vec3<f32>;
    if (r > g) {
        if (g > b) {
            a = lz_lut_em(i.x + 1u, i.y, i.z);
            bv = lz_lut_em(i.x + 1u, i.y + 1u, i.z);
            w = vec3<f32>(r, g, b);
        } else if (r > b) {
            a = lz_lut_em(i.x + 1u, i.y, i.z);
            bv = lz_lut_em(i.x + 1u, i.y, i.z + 1u);
            w = vec3<f32>(r, b, g);
        } else {
            a = lz_lut_em(i.x, i.y, i.z + 1u);
            bv = lz_lut_em(i.x + 1u, i.y, i.z + 1u);
            w = vec3<f32>(b, r, g);
        }
    } else if (b > g) {
        a = lz_lut_em(i.x, i.y, i.z + 1u);
        bv = lz_lut_em(i.x, i.y + 1u, i.z + 1u);
        w = vec3<f32>(b, g, r);
    } else if (b > r) {
        a = lz_lut_em(i.x, i.y + 1u, i.z);
        bv = lz_lut_em(i.x, i.y + 1u, i.z + 1u);
        w = vec3<f32>(g, b, r);
    } else {
        a = lz_lut_em(i.x, i.y + 1u, i.z);
        bv = lz_lut_em(i.x + 1u, i.y + 1u, i.z);
        w = vec3<f32>(g, r, b);
    }
    return (1.0 - w.x) * c000 + (w.x - w.y) * a + (w.y - w.z) * bv + w.z * c111;
}

fn lz_contraste(v: f32, k: f32) -> f32 {
    if (v > 0.0) {
        return 0.18 * pow(v / 0.18, k);
    }
    return 0.0;
}

// `ph2d_lens::grade::aplica`.
fn lz_grade(c0: vec3<f32>) -> vec3<f32> {
    let c1 = c0 * u.grade.x;
    let c2 = vec3<f32>(
        u.bal0.x * c1.x + u.bal0.y * c1.y + u.bal0.z * c1.z,
        u.bal1.x * c1.x + u.bal1.y * c1.y + u.bal1.z * c1.z,
        u.bal2.x * c1.x + u.bal2.y * c1.y + u.bal2.z * c1.z,
    );
    let k = u.grade.y;
    let c3 = vec3<f32>(lz_contraste(c2.x, k), lz_contraste(c2.y, k), lz_contraste(c2.z, k));
    var ok = pc_to_oklab(c3);
    ok.y = ok.y * u.grade.z;
    ok.z = ok.z * u.grade.z;
    let c4 = max(pc_from_oklab(ok), vec3<f32>(0.0));
    if (u.modo.y != 0u) {
        let cod = lz_l2s3(min(c4, vec3<f32>(1.0)));
        let olhado = lz_s2l3(lz_tetra(cod));
        return c4 + (olhado - c4) * u.grade.w;
    }
    return c4;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // O centro do pixel `i` do sub-rect é `i + 0,5` — a convenção da `ph2d-lens`.
    let p = in.pos.xy - u.rect.xy;
    let base = lz_texel(vec2<i32>(floor(p)));
    var c = base.rgb;
    if (u.liga.x != 0u) {
        var pr: vec2<f32>;
        var pb: vec2<f32>;
        if (u.modo.x != 0u) {
            pr = p - u.ab_d.xy;
            pb = p + u.ab_d.xy;
        } else {
            pr = u.ab.xy + (p - u.ab.xy) / u.ab.z;
            pb = u.ab.xy + (p - u.ab.xy) / u.ab.w;
        }
        c.x = lz_bilinear(pr, 0u);
        c.z = lz_bilinear(pb, 2u);
    }
    if (u.liga.y != 0u) {
        let w = lz_mascara(p - u.vin_geo.xy, u.vin_geo.zw, u.vin_par.x, u.vin_par.y);
        let t = u.vin_par.z * (1.0 - w);
        c = c + (u.vin_alvo.xyz - c) * t;
    }
    if (u.liga.z != 0u) {
        c = lz_grade(c);
    }
    if (u.liga.w != 0u) {
        let cod = lz_l2s3(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)));
        let g = lz_grao(cod, p, u.grao.x, u.grao.y, u.grao.z, u.modo.z != 0u, u.modo.w);
        c = lz_s2l3(clamp(g, vec3<f32>(0.0), vec3<f32>(1.0)));
    }
    return vec4<f32>(lz_l2s3(c) * base.a, base.a);
}
