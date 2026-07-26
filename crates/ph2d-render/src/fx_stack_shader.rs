//! **O SHADER da pilha de FX raster** — o que o DEVICE executa, separado de como o host o
//! alimenta (`fx_stack.rs`: pipelines, globals, bind groups, dispatch).
//!
//! O corte é por responsabilidade, não por tamanho: aqui moram os três módulos WGSL, os dois
//! números que definem o corte do Outline (que são semântica do shader, não do host) e o gerador
//! dos códigos de tipo.

/// **O nível de corte do Outline.** Para uma aresta reta, a silhueta borrada vale `Φ(−d/σ)` a uma
/// distância `d` para fora; cortar em `Φ(−1) = 0,15866` põe a borda do contorno **exatamente a
/// `σ` px** — é isso que faz o slider "Width" prometer o que entrega (há gate que MEDE).
pub(crate) const OUTLINE_LEVEL: f32 = 0.158_655_25;

/// A meia-banda de anti-aliasing do corte, em unidades de VALOR, para uma banda de ~1 px em
/// DISTÂNCIA: o gradiente do perfil no nível de corte é `φ(1)/σ = 0,242/σ`, logo meia banda de
/// meio pixel é `0,121/σ`. Derivado, não escolhido.
pub(crate) const OUTLINE_AA_NUM: f32 = 0.120_985;

pub(crate) const FX_STACK_WGSL: &str = r#"
struct Globals {
    dims: vec2<u32>,
    half: u32,
    kind: u32,
    tint: vec4<f32>,
    inv_two_sigma2: f32,
    opacity: f32,
    off_x: i32,
    off_y: i32,
    edge: f32,
    aa: f32,
    _pad: vec2<f32>,
};
@group(0) @binding(0) var t0: texture_2d<f32>;
@group(0) @binding(1) var t1: texture_2d<f32>;
@group(0) @binding(2) var<uniform> g: Globals;

fn is_inner() -> bool { return g.kind == KIND_INNER_SHADOW || g.kind == KIND_INNER_GLOW; }

// A imagem é premultiplicada sobre um campo TRANSPARENTE: fora da textura não há nada. Isto
// substitui o `clamp` de coordenada da W2 — que ESTICAVA o texel da borda para dentro do kernel.
// Nos ops de fora as duas respostas coincidem (a margem garante borda transparente); nos de
// DENTRO só esta é correta, e é ela que lhes dá margem ZERO.
fn tap_img(t: texture_2d<f32>, x: i32, y: i32) -> vec4<f32> {
    if (x < 0 || y < 0 || x >= i32(g.dims.x) || y >= i32(g.dims.y)) { return vec4<f32>(0.0); }
    return textureLoad(t, vec2<i32>(x, y), 0);
}

// **A FONTE do borrão.** Os ops de fora borram a imagem; os de DENTRO borram o alfa INVERTIDO —
// alto onde não há forma —, e é isso que faz a sombra nascer na borda e morrer para o miolo.
fn source_of(s: vec4<f32>) -> vec4<f32> {
    if (is_inner()) { return vec4<f32>(0.0, 0.0, 0.0, 1.0 - s.a); }
    return s;
}

// O intermediário JÁ é a fonte borrada, então fora da textura ele vale `source_of(transparente)`:
// 1 para os de dentro (lá fora é tudo "fora da forma"), 0 para os de fora.
fn tap_mid(t: texture_2d<f32>, x: i32, y: i32) -> vec4<f32> {
    if (x < 0 || y < 0 || x >= i32(g.dims.x) || y >= i32(g.dims.y)) {
        return source_of(vec4<f32>(0.0));
    }
    return textureLoad(t, vec2<i32>(x, y), 0);
}

fn gauss(i: f32) -> f32 { return exp(-i * i * g.inv_two_sigma2); }

fn blur_h_at(id: vec3<u32>) -> vec4<f32> {
    var acc = vec4<f32>(0.0);
    var wsum = 0.0;
    let h = i32(g.half);
    for (var k = -h; k <= h; k = k + 1) {
        let wt = gauss(f32(k));
        acc = acc + source_of(tap_img(t0, i32(id.x) - g.off_x + k, i32(id.y))) * wt;
        wsum = wsum + wt;
    }
    return acc / wsum;
}

fn blur_v_at(id: vec3<u32>) -> vec4<f32> {
    var acc = vec4<f32>(0.0);
    var wsum = 0.0;
    let h = i32(g.half);
    for (var k = -h; k <= h; k = k + 1) {
        let wt = gauss(f32(k));
        acc = acc + tap_mid(t1, i32(id.x), i32(id.y) - g.off_y + k) * wt;
        wsum = wsum + wt;
    }
    return acc / wsum;
}
"#;

/// O corpo dos dois passes que escrevem em `rgba16float` (entre ops).
pub(crate) const FX_STACK_MID_WGSL: &str = r#"
@group(0) @binding(3) var dst: texture_storage_2d<rgba16float, write>;

// Horizontal: entrada premultiplicada -> temp premultiplicada (com o deslocamento em X).
@compute @workgroup_size(8, 8, 1)
fn cs_blur_h(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), blur_h_at(id));
}

// Vertical + finalize + composite: temp -> saída do OP, premultiplicada.
//   Blur                : o borrado, com a opacidade do degrau.
//   Glow / Drop Shadow  : a cor do efeito com o alfa da silhueta borrada, POR BAIXO da entrada.
//   Outline             : o mesmo halo com CORTE DURO — largura, não desvanecimento.
//   Inner Shadow / Glow : o alfa invertido borrado, mascarado pela forma, POR CIMA da entrada.
@compute @workgroup_size(8, 8, 1)
fn cs_op_v(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let b = blur_v_at(id);
    // A entrada, intacta — é ela que faz o op ser imagem -> imagem.
    let over = tap_img(t0, i32(id.x), i32(id.y));
    var outc: vec4<f32>;
    if (g.kind == KIND_BLUR) {
        outc = b * g.opacity;
    } else if (is_inner()) {
        // `b.a` é o alfa INVERTIDO borrado: alto perto da borda, ~0 no miolo. Multiplicar pela
        // cobertura da forma prende o halo DENTRO dela — fora, `over.a` é 0 e não sobra nada.
        let a = b.a * g.tint.a * g.opacity * over.a;
        let halo = vec4<f32>(g.tint.rgb * a, a);
        outc = halo + over * (1.0 - halo.a);
    } else {
        var cov = b.a;
        if (g.kind == KIND_OUTLINE) {
            // Corte DURO no nível que põe a borda a `sigma` px da silhueta; a banda de AA vem do
            // gradiente do perfil ali, então a espessura da transição é ~1 px em qualquer sigma.
            cov = smoothstep(g.edge - g.aa, g.edge + g.aa, b.a);
        }
        let a = cov * g.tint.a * g.opacity;
        let halo = vec4<f32>(g.tint.rgb * a, a);
        outc = over + halo * (1.0 - over.a);
    }
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), outc);
}

// **O op PONTUAL** (Color Overlay): um dispatch, sem vizinho nenhum. Repinta o RGB e **não move um
// texel de cobertura** — o alfa sai byte-idêntico ao que entrou, e é isso que o separa de um halo.
@compute @workgroup_size(8, 8, 1)
fn cs_op_point(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let src = tap_img(t0, i32(id.x), i32(id.y));
    let k = clamp(g.tint.a * g.opacity, 0.0, 1.0);
    // Premultiplicado: a cor cheia neste texel é `tint.rgb * src.a`.
    let rgb = mix(src.rgb, g.tint.rgb * src.a, k);
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), vec4<f32>(rgb, src.a));
}
"#;

/// O passe final, que escreve em `rgba8unorm` com alfa RETO (o que o Vello amostra).
pub(crate) const FX_STACK_OUT_WGSL: &str = r#"
@group(0) @binding(3) var dst: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8, 8, 1)
fn cs_resolve(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let premul = textureLoad(t0, vec2<i32>(i32(id.x), i32(id.y)), 0);
    var rgb = vec3<f32>(0.0);
    if (premul.a > 0.0001) { rgb = premul.rgb / premul.a; }
    let outc = vec4<f32>(rgb, premul.a);
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), clamp(outc, vec4<f32>(0.0), vec4<f32>(1.0)));
}
"#;

/// ⚠️ **64 bytes de propósito.** O `min_binding_size` do layout é `size_of::<Globals>()`, e o
/// WGSL arredonda o tamanho de um struct de uniform ao alinhamento dele (16, por causa do `vec4`).
/// Sem o padding explícito o Rust diria 56 e o wgpu recusaria o bind group.
use ph2d_ecs::FxOp;

/// Os códigos de tipo, **gerados a partir do `ph2d_ecs::FxOp`** e prefixados ao shader.
///
/// ⚠️ É a única forma de não ter uma segunda tabela do outro lado da fronteira de linguagem: um
/// `if (g.kind == 3u)` escrito à mão no WGSL é exatamente o tipo de número que sobrevive a uma
/// renumeração e passa a desenhar o efeito errado, com todos os gates de unidade verdes.
pub(crate) fn kind_consts_wgsl() -> String {
    format!(
        "const KIND_BLUR: u32 = {}u;\n\
         const KIND_INNER_SHADOW: u32 = {}u;\n\
         const KIND_INNER_GLOW: u32 = {}u;\n\
         const KIND_OUTLINE: u32 = {}u;\n",
        FxOp::BLUR,
        FxOp::INNER_SHADOW,
        FxOp::INNER_GLOW,
        FxOp::OUTLINE,
    )
}
