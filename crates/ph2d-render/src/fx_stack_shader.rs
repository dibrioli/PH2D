//! **O SHADER da pilha de FX raster** — o que o DEVICE executa, separado de como o host o
//! alimenta (`fx_stack.rs`: pipelines, globals, bind groups, dispatch).
//!
//! O corte é por responsabilidade, não por tamanho: aqui moram os três módulos WGSL, os dois
//! números que definem o corte do Outline (que são semântica do shader, não do host) e o gerador
//! dos códigos de tipo.

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
    jump: i32,
    band: f32,
};
@group(0) @binding(0) var t0: texture_2d<f32>;
@group(0) @binding(1) var t1: texture_2d<f32>;
@group(0) @binding(2) var<uniform> g: Globals;

fn is_inner() -> bool { return g.kind == KIND_INNER_SHADOW || g.kind == KIND_INNER_GLOW; }

// O halo de dentro, aplicado com a lei que NÃO move a cobertura: um efeito de dentro tinge o que
// já está lá, ele não é uma camada nova. Porta única dos DOIS modos.
fn inner_tint(over: vec4<f32>, strength: f32) -> vec4<f32> {
    let tinted = vec4<f32>(g.tint.rgb * over.a, over.a);
    return mix(over, tinted, clamp(strength, 0.0, 1.0));
}

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
//   Inner Shadow / Glow : o alfa invertido borrado, mascarado pela forma, POR CIMA da entrada.
@compute @workgroup_size(8, 8, 1)
fn cs_op_v(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let b = blur_v_at(id);
    // A entrada, intacta — é ela que faz o op ser imagem -> imagem.
    let over = tap_img(t0, i32(id.x), i32(id.y));
    var outc: vec4<f32>;
    if (g.kind == KIND_BLUR) {
        // ⚠️ MISTURA com a entrada, não `b * opacity`. A opacidade de um degrau é *quanto DESTE
        // efeito*, então em 0 ele tem de ser no-op — e `b * 0` apagava a forma inteira.
        outc = mix(over, b, g.opacity);
    } else if (is_inner()) {
        // `b.a` é o alfa INVERTIDO borrado: alto perto da borda, ~0 no miolo.
        //
        // ⚠️ **A COBERTURA NÃO SE MOVE.** Compor o halo como uma CAMADA por cima
        // (`halo + over*(1-halo.a)`) SOMA alfa: na borda anti-aliased `over.a = 0,5` com
        // `halo.a = 0,25` dava 0,625, e como o `resolve` des-premultiplica, dividir por um alfa
        // maior CLAREIA — era o rim claro de 1 px em volta da forma. Um efeito de DENTRO tinge o
        // que já está lá; ele não é uma camada nova.
        outc = inner_tint(over, b.a * g.tint.a * g.opacity);
    } else {
        let a = b.a * g.tint.a * g.opacity;
        let halo = vec4<f32>(g.tint.rgb * a, a);
        outc = over + halo * (1.0 - over.a);
    }
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), outc);
}

// ── O CAMPO DE DISTÂNCIA (JFA limitado) ───────────────────────────────────────────────────────
//
// Por que ele existe: o modo PROXIMITY mede *quanto de fora há por perto* (o alfa invertido
// borrado). Numa reentrância o "fora" subtende um ângulo pequeno, então a sombra quase não nasce
// lá — foi o que o smoke reportou: a estrela ficava com sombra só nas pontas. A DISTÂNCIA à borda
// não tem essa dependência de ângulo: ela é 0 em TODO ponto do contorno, reentrâncias incluídas.
//
// `t1` guarda por texel o OFFSET inteiro até o texel de FORA mais próximo (`.xy`), com `.z = 1`
// quando já há semente. ⚠️ Os offsets são limitados pela banda, e f16 representa inteiros até 2048
// EXATAMENTE — o campo é exato na faixa que nos interessa, não "aproximado porque é f16".

@compute @workgroup_size(8, 8, 1)
fn cs_sdf_seed(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let a = tap_img(t0, i32(id.x), i32(id.y)).a;
    var v = vec4<f32>(0.0);
    // Quem semeia depende da pergunta: um degrau de DENTRO mede a distância ao FORA (semente = os
    // texels de fora), o CONTORNO mede a distância à FORMA (semente = os texels dela). Fora da
    // TEXTURA é fora da forma nos dois casos — o `tap_img` devolve transparente.
    let seed = select(a > 0.5, a <= 0.5, is_inner());
    if (seed) { v = vec4<f32>(0.0, 0.0, 1.0, 0.0); }
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), v);
}

@compute @workgroup_size(8, 8, 1)
fn cs_sdf_jump(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let me = vec2<i32>(i32(id.x), i32(id.y));
    var best = textureLoad(t1, me, 0);
    var bd = 1.0e30;
    if (best.z > 0.5) { bd = dot(best.xy, best.xy); }
    for (var j = -1; j <= 1; j = j + 1) {
        for (var i = -1; i <= 1; i = i + 1) {
            let step = vec2<i32>(i * g.jump, j * g.jump);
            let s = me + step;
            let delta = vec2<f32>(f32(step.x), f32(step.y));
            var off = vec2<f32>(0.0);
            var has = false;
            if (s.x < 0 || s.y < 0 || s.x >= i32(g.dims.x) || s.y >= i32(g.dims.y)) {
                // ⚠️ Fora da textura é FORA DA FORMA — semente para quem mede a distância ao fora
                // (os degraus de dentro), e NÃO para quem mede a distância à forma (o contorno).
                // Semear os dois igual fazia o contorno crescer a partir da borda da textura para
                // dentro da cena: medido, 63 px de halo numa largura de 4.
                off = delta;
                has = is_inner();
            } else {
                let n = textureLoad(t1, s, 0);
                if (n.z > 0.5) { off = n.xy + delta; has = true; }
            }
            if (has) {
                let dd = dot(off, off);
                if (dd < bd) { bd = dd; best = vec4<f32>(off, 1.0, 0.0); }
            }
        }
    }
    textureStore(dst, me, best);
}

// **O finalize sobre o CAMPO DE DISTÂNCIA** — serve os degraus de dentro em modo Contour E o
// contorno. Sem borrão nenhum: a largura É a distância, então ela é a MESMA em toda a volta.
@compute @workgroup_size(8, 8, 1)
fn cs_op_field(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let over = tap_img(t0, i32(id.x), i32(id.y));
    // A LUZ entra amostrando o campo DESLOCADO — a banda engorda do lado de onde a luz vem, e
    // encolhe do outro, exatamente como no modo de proximidade.
    let sx = i32(id.x) - g.off_x;
    let sy = i32(id.y) - g.off_y;
    var d = 1.0e6;
    if (sx < 0 || sy < 0 || sx >= i32(g.dims.x) || sy >= i32(g.dims.y)) {
        // Mesma assimetria do salto: lá fora não há forma. Para quem mede o FORA a distância é 0
        // (sombra cheia na borda); para o contorno é infinita (não há o que contornar).
        if (is_inner()) { d = 0.0; }
    } else {
        let f = textureLoad(t1, vec2<i32>(sx, sy), 0);
        if (f.z > 0.5) { d = length(f.xy); }
    }
    // ⚠️ MEIO TEXEL, e ele é derivado: o JFA mede até o CENTRO do texel semente, e a fronteira
    // geométrica está a 0,5 px dele. Sem isto o contorno sai 1 px mais fino do que a largura que o
    // slider promete — medido, 2,5 px numa largura de 4.
    let dist = max(d - 0.5, 0.0);
    let w = max(g.band, 1.0e-4);
    var outc: vec4<f32>;
    if (is_inner()) {
        outc = inner_tint(over, (1.0 - smoothstep(0.0, w, dist)) * g.tint.a * g.opacity);
    } else {
        // CONTORNO: a borda cai exatamente em `w`, com ~1 px de anti-aliasing. Isto é uma DILATAÇÃO
        // de verdade (`d <= w`), ao contrário do corte num campo borrado, que ENCOLHE na quina
        // convexa — medido, uma ponta de 36° não recebia contorno NENHUM.
        let cov = 1.0 - smoothstep(w - 0.5, w + 0.5, dist);
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
         const KIND_OUTLINE: u32 = {}u;\n\
         const MODE_CONTOUR: u32 = {}u;\n",
        FxOp::BLUR,
        FxOp::INNER_SHADOW,
        FxOp::INNER_GLOW,
        FxOp::OUTLINE,
        FxOp::MODE_CONTOUR,
    )
}
