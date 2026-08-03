// O carimbo de pigmento no device — a metade estreita e QUENTE do `stamp_dab_inner`.
//
// ⚠️ **Este shader não conhece a lei do falloff, e é assim de propósito.** Ele AMOSTRA a tabela que a
// CPU encheu com o `falloff_weight` que já shipa (a mesma cura do LUT especular do `ImpastoLightPass`):
// um kernel que re-derivasse o perfil seria uma segunda resposta a *"que forma tem este dab?"*,
// divergindo só numa screenshot. Ver `docs/Painter/33_plano_gpu_do_carimbo.md` §2.
//
// A aritmética abaixo é a de `dab/bands.rs::stamp_band` + `blend::blend_over` + `encode`, transcrita na
// MESMA ordem de operações — a ordem é o que decide a paridade, porque a única liberdade que resta
// ao driver é contrair `a*b + c` num FMA.

struct Dab {
    // Centro em pixels de canvas.
    center: vec2<f32>,
    radius: f32,
    coverage: f32,
    color: vec3<f32>,
    // A LINHA 0 e a LINHA 1 do mapa linear do footprint (flatten & rotate), avaliadas na CPU nos
    // vetores da base. Um deform de dab É linear, e há gate provando isso contra o `apply` real —
    // a premissa virou teste em vez de comentário.
    m0: vec2<f32>,
    m1: vec2<f32>,
    _pad: vec2<f32>,
};

struct Params {
    // A janela que este despacho escreve, em pixels de canvas.
    rx: u32,
    ry: u32,
    rw: u32,
    rh: u32,
    // Largura do canvas (o `stride` do base/out é a largura da REGIÃO, não do canvas).
    dab_count: u32,
    lut_len: u32,
    // 1 = alpha-lock: a cobertura é portada pelo alpha que já está no pixel.
    preserve_alpha: u32,
    _pad: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> dabs: array<Dab>;
@group(0) @binding(2) var<storage, read> lut: array<f32>;
@group(0) @binding(3) var<storage, read> base: array<u32>;
@group(0) @binding(4) var<storage, read_write> out: array<u32>;

// A tabela do perfil, amostrada em `t ∈ [0,1]`. `t >= 1` é fora do dab — zero, sem consultar.
//
// ⚠️ **Nearest, e a RESOLUÇÃO é o parâmetro de paridade** — quem constrói a tabela a define. A CPU
// avalia `falloff_weight(t)` EXATO por texel, então o degrau entre nós é a ÚNICA fonte de
// divergência que sobra depois de todo o resto ser transcrito (medido: é ela, e não o blend). O
// gate varre a escada e o produto usa **65 536 nós = 256 KB**, o joelho — e a tabela é função só de
// `hardness`/`falloff`, que não mudam dentro de um traço, então o preenchimento é UMA vez por
// traço, nunca por lote.
fn profile(t: f32) -> f32 {
    if (t >= 1.0) {
        return 0.0;
    }
    let n = f32(params.lut_len);
    let idx = u32(clamp(t, 0.0, 1.0) * (n - 1.0) + 0.5);
    return lut[min(idx, params.lut_len - 1u)];
}

fn unpack(px: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(px & 0xffu) / 255.0,
        f32((px >> 8u) & 0xffu) / 255.0,
        f32((px >> 16u) & 0xffu) / 255.0,
        f32((px >> 24u) & 0xffu) / 255.0,
    );
}

// `encode` do `bands.rs`: round-to-nearest, clampado. O `+ 0.5` é load-bearing — tirá-lo move um
// nível inteiro em milhares de bytes, e um gate que só olhasse MAGNITUDE não veria.
fn pack(c: vec4<f32>) -> u32 {
    let q = clamp(c, vec4<f32>(0.0), vec4<f32>(1.0)) * 255.0 + vec4<f32>(0.5);
    return u32(q.x) | (u32(q.y) << 8u) | (u32(q.z) << 16u) | (u32(q.w) << 24u);
}

// `blend_over(BrushBlend::Mix, …)` — o source-over de alfa STRAIGHT que esta rota de fato usa.
//
// ⚠️ **É esta a função do produto**, e a primeira versão deste shader transcreveu a ERRADA: o
// `stamp_rgba` (o lerp premultiplicado), que só é o caminho do produto no modo
// `RampAlphaMode::TextureAlpha` — precisamente o que este predicado exclui. Com `sa = 1` os dois são
// *algebricamente* o mesmo, e eu previ que a troca fecharia a divergência de 14-300 bytes, porque
// `(b·ab)·(1−a)` e `b·(ab·(1−a))` não são o mesmo número em `f32` (a multiplicação IEEE-754 é
// comutativa e **não é associativa**).
//
// ⚠️ **A medição REFUTOU essa previsão: os seis números saíram idênticos, byte a byte.** A
// divergência inteira era a resolução da TABELA (varrida: 1 024 reprova · 16 384 → 71 · 65 536 → 18
// · 262 144 → 8 bytes). A transcrição certa fica pelo motivo que sempre valeu — *é a função que
// shipa*, e um doc dizendo o contrário seria um doc que mente —, não por um ganho que ela não deu.
//
// O `pigment_mix` do `blend_over_pigment` é excluído pelo predicado de estreitamento (o crossfade
// RYB é outra lei); com ele em zero aquela função devolve `plain` — este valor — sem tocá-lo.
fn blend_over_mix(dst: vec4<f32>, color: vec3<f32>, a_in: f32) -> vec4<f32> {
    let a = clamp(a_in, 0.0, 1.0);
    let ab = dst.w;
    let ao = a + ab * (1.0 - a);
    if (ao <= 0.0) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(
        (color.x * a + dst.x * ab * (1.0 - a)) / ao,
        (color.y * a + dst.y * ab * (1.0 - a)) / ao,
        (color.z * a + dst.z * ab * (1.0 - a)) / ao,
        ao,
    );
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.rw || gid.y >= params.rh) {
        return;
    }
    let i = gid.y * params.rw + gid.x;
    // Coordenada de CANVAS, e o `+ 0.5` é o centro do pixel — a convenção do `stamp_band`.
    let px = f32(params.rx + gid.x) + 0.5;
    let py = f32(params.ry + gid.y) + 0.5;

    var acc = unpack(base[i]);
    // ⚠️ **Os dabs são percorridos na ORDEM da lista, em série dentro do texel.** Um dab compõe
    // sobre o resultado do anterior — a lei é um PRODUTO sobre a lista, não uma soma —, então
    // paralelizar sobre dabs mudaria o desenho. O paralelismo aqui é por TEXEL, que é disjunto.
    for (var d = 0u; d < params.dab_count; d = d + 1u) {
        let dab = dabs[d];
        let inv_r = 1.0 / dab.radius;
        let dx = px - dab.center.x;
        let dy = py - dab.center.y;
        let u = vec2<f32>(dx * inv_r, dy * inv_r);
        // O footprint como mapa linear, avaliado na CPU. Identidade num pincel redondo.
        let wv = vec2<f32>(dot(dab.m0, u), dot(dab.m1, u));
        let t = sqrt(wv.x * wv.x + wv.y * wv.y);
        let w = profile(t);
        if (w <= 0.0) {
            continue;
        }
        var a = w * dab.coverage;
        if (params.preserve_alpha == 1u) {
            a = a * acc.w;
        }
        if (a <= 0.0) {
            continue;
        }
        // ⚠️ **A ida e volta por `u8` entre dabs É A LEI, não uma perda a evitar.** A CPU escreve
        // `dst[i] = encode(out)` e o dab seguinte relê `f32::from(dst[i]) / 255.0`, então o perfil
        // acumulado passa por 8 bits a cada passo. Guardar `acc` em `f32` ao longo do laço é
        // ESTRITAMENTE MAIS PRECISO e por isso **diverge** — e com a sobreposição de ~10× que uma
        // figura tem por quadro (doc 33 §1) a diferença não é sutil. Precisão a mais é uma segunda
        // resposta como qualquer outra.
        acc = unpack(pack(blend_over_mix(acc, dab.color, a)));
    }
    out[i] = pack(acc);
}
