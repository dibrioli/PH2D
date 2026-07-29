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
    n_segs: u32,
    blend: u32,
    noise_scale: f32,
    octaves: u32,
    seed: u32,
    mode: u32,
    org: vec2<f32>,
    grow_px: f32,
    _pad: f32,
};
@group(0) @binding(0) var t0: texture_2d<f32>;
@group(0) @binding(1) var t1: texture_2d<f32>;
@group(0) @binding(2) var<uniform> g: Globals;
// A SILHUETA em segmentos (`x0,y0,x1,y1`), no espaço de texel do scratch. Vazia = `n_segs == 0`.
@group(0) @binding(4) var<storage, read> segs: array<vec4<f32>>;

// ── A TRANSFERÊNCIA sRGB ──────────────────────────────────────────────────────────────────────
//
// **O espaço de trabalho da pilha é LINEAR, premultiplicado; sRGB só nas fronteiras.** É a
// convenção de toda composição séria (o default `linearRGB` do `color-interpolation-filters` do
// SVG · o *Blend Colors Using 1.0 Gamma* do AE · Nuke/Fusion/Flame · OpenEXR/ACES) e é a que o
// próprio Vello já usa a montante: ele compõe em luz linear e codifica em sRGB só para caber em 8
// bits, então `stored = encode(a · linear(cor))`.
//
// ⚠️ **É por isso que `rgb/a` sobre os bytes NÃO era a des-premultiplicação.** Medido em âmbar
// (235,175,60) a meia cobertura: o byte guardado é (173,128,41) e a divisão ingênua devolvia
// (255,255,82) — branco lavado no fio da borda, que é onde estes efeitos inteiros vivem.
//
// O alfa **nunca** é transferido: ele já é linear por definição.
fn srgb_to_linear(c: f32) -> f32 {
    if (c <= 0.04045) { return c / 12.92; }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn linear_to_srgb(c: f32) -> f32 {
    if (c <= 0.0031308) { return c * 12.92; }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}

fn srgb_to_linear3(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(srgb_to_linear(c.r), srgb_to_linear(c.g), srgb_to_linear(c.b));
}

fn linear_to_srgb3(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(linear_to_srgb(c.r), linear_to_srgb(c.g), linear_to_srgb(c.b));
}

// A cor do degrau, em LINEAR. O `tint` chega do painel em sRGB (é o que a swatch mostra e o que o
// picker escreve), e o miolo da pilha só fala linear — a conversão mora aqui, na fronteira, e não
// em cinco sítios de uso.
fn tint_lin() -> vec3<f32> { return srgb_to_linear3(g.tint.rgb); }

fn is_inner() -> bool { return g.kind == KIND_INNER_SHADOW || g.kind == KIND_INNER_GLOW; }

// **Quem precisa do campo FORA da forma?** O feather (a rampa é centrada na fronteira) e o contorno
// (ele mora todo lá fora). Esses semeiam a CASCA — a primeira fileira de dentro —, que dá os dois
// lados de uma vez. Quem só olha para dentro semeia os texels de FORA, que é a medida exata do que
// eles perguntam: *a que distância estou de deixar de existir*. ⚠️ A diferença aparece na quina
// CÔNCAVA — a casca de um lado só a estima ~0,6 px pior, e é justamente ali que o modo Contour
// existe para acertar.
// ⚠️ **A pergunta é feita, não ENUMERADA — e a lista de nomes já apodreceu uma vez.** Ela dizia
// `FEATHER || OUTLINE || BEVEL || GLOW`, uma enumeração dos leitores conhecidos, e a morfologia
// (que lê o campo dos DOIS lados: crescer olha para fora, encolher olha para dentro) nasceu a cair
// no `else` — ou seja com o campo definido de um lado só, e nada a dizer. O que distingue os dois
// grupos é *para onde este degrau olha*, e isso já tem nome uma linha acima: **os de DENTRO** são a
// exceção, todo o resto quer os dois lados. Escrita assim, o próximo tipo do campo nasce certo.
//
// ⚠️ Sobre o conjunto que de facto chega aqui (só os degraus de campo com semente de raster) as
// duas formas eram EQUIVALENTES no dia desta troca — os gates de contorno, feather, bevel, glow e
// dos de dentro são a prova de que nada se moveu.
fn seeds_shell() -> bool {
    return !is_inner();
}

// **O PÉ EXATO na silhueta**, do centro do texel `p` — o ponto mais próximo sobre os segmentos.
//
// ⚠️ É o único lugar onde a geometria entra, e é o que separa esta wave de todas as tentativas de
// raster que a precederam. A rampa de AA ocupa 1,0–1,41 texel e o estêncil da diferença central
// ocupa 2, então a estimativa pela COBERTURA sempre inclui amostras saturadas e o recorte é função
// da FASE do texel na escada de rasterização: medido, até 0,68 px de erro de profundidade e — numa
// aresta a 4,6°, onde a escada tem passo de 12,4 texels e um 3×3 lê a aresta como horizontal — erro
// de direção igual ao ÂNGULO INTEIRO. Com o pé exato o ripple medido cai de 42,4 para 1,0 nível.
//
// ⚠️ **A regra de QUAIS texels semeiam continua sendo a do raster** (a casca), e isso é deliberado:
// ela identifica a fronteira da SILHUETA, que numa forma com traço não é o path de preenchimento.
// Uma aresta de fill coberta por um traço fica a meia-largura dali, então nunca é a mais próxima de
// um texel de casca — a geometria só responde *onde exatamente*, nunca *se*.
fn exact_foot(pi: vec2<f32>) -> vec2<f32> {
    // ⚠️ **O CENTRO do texel, não a quina.** A cobertura que o rasterizador escreve em `(x,y)` é a
    // do quadrado `[x,x+1]×[y,y+1]`, cujo centro é `(x+0,5, y+0,5)`; medir da quina metia 0,707
    // texel de erro SISTEMÁTICO entre o campo e o raster que decide o sinal. O `off` devolvido é
    // relativo ao centro, e é por isso que `round(src + off)` continua indexando o texel certo:
    // `floor((src + 0,5) + off)` é exatamente isso.
    let p = pi + vec2<f32>(0.5, 0.5);
    var best = vec2<f32>(0.0);
    var bd = 1.0e30;
    for (var i = 0u; i < g.n_segs; i = i + 1u) {
        let s = segs[i];
        let a = s.xy;
        let b = s.zw;
        let ab = b - a;
        let l2 = dot(ab, ab);
        var t = 0.0;
        if (l2 > 1.0e-12) { t = clamp(dot(p - a, ab) / l2, 0.0, 1.0); }
        let q = a + ab * t;
        let d = dot(p - q, p - q);
        if (d < bd) { bd = d; best = q; }
    }
    return best - p;
}

// **A LEI DE MISTURA de um degrau, aplicada à COR dele.**
//
// `B(Cb, Cs)` vem do `blend_modes.wgsl` — o MESMO arquivo que o compositor de camadas compila, e
// que está pinado bit a bit contra o Rust. Aqui só se resolve o que ele exige: ele fala RETO e
// LINEAR, e o miolo desta pilha é linear PREMULTIPLICADO.
//
// ⚠️ **O peso é o alfa do FUNDO, e é a fórmula do W3C, não uma escolha de gosto:**
// `Cs' = (1−ab)·Cs + ab·B(Cb,Cs)`. Onde não há nada por baixo (`ab = 0`) não há com que misturar e
// a lei devolve a própria cor — é isso que faz a rampa de anti-aliasing desvanecer para Normal em
// vez de ganhar uma orla de cor inventada.
//
// ⚠️ **O early-out em Normal é LOAD-BEARING, não higiene.** `mix(x, x, a)` é `x·(1−a) + x·a`, que
// em ponto flutuante **não é exactamente `x`** — sem este `return` a pilha inteira deixaria de ser
// byte-idêntica ao mundo pré-blend no caso default, e a wave passaria a mudar a aparência de toda
// arte já autorada.
fn fx_blend(backdrop: vec4<f32>, colour: vec3<f32>) -> vec3<f32> {
    if (g.blend == 0u) { return colour; }
    let ab = clamp(backdrop.a, 0.0, 1.0);
    if (ab <= 0.0) { return colour; }
    let cb = clamp(backdrop.rgb / ab, vec3<f32>(0.0), vec3<f32>(1.0));
    var b: vec3<f32>;
    if (is_hsl(g.blend)) {
        b = blend_hsl(g.blend, cb, colour);
    } else {
        b = vec3<f32>(
            blend_sep(g.blend, cb.r, colour.r),
            blend_sep(g.blend, cb.g, colour.g),
            blend_sep(g.blend, cb.b, colour.b),
        );
    }
    return mix(colour, clamp(b, vec3<f32>(0.0), vec3<f32>(1.0)), ab);
}

// O halo de dentro, aplicado com a lei que NÃO move a cobertura: um efeito de dentro tinge o que
// já está lá, ele não é uma camada nova. Porta única de TODOS os que moram dentro.
//
// ⚠️ **É por ser porta única que o blend chegou aos três de dentro numa linha:** Inner Shadow,
// Inner Glow e Bevel passam todos por aqui, e o Bevel de graça (a cor dele já é escolhida antes —
// branco na face iluminada, o tint na oposta — e a lei se aplica à cor que sair).
fn inner_tint(over: vec4<f32>, colour: vec3<f32>, strength: f32) -> vec4<f32> {
    let tinted = vec4<f32>(fx_blend(over, colour) * over.a, over.a);
    return mix(over, tinted, clamp(strength, 0.0, 1.0));
}

// A imagem é premultiplicada sobre um campo TRANSPARENTE: fora da textura não há nada. Isto
// substitui o `clamp` de coordenada da W2 — que ESTICAVA o texel da borda para dentro do kernel.
// Nos ops de fora as duas respostas coincidem (a margem garante borda transparente); nos de
// DENTRO só esta é correta, e é ela que lhes dá margem ZERO.
fn tap_img_at(p: vec2<f32>) -> vec4<f32> {
    return tap_img(t0, i32(p.x), i32(p.y));
}

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

// **A PORTA DE ENTRADA**: a fonte do Vello (sRGB, alfa **RETO**) -> o espaço de trabalho da pilha
// (LINEAR, **premultiplicado**). Duas conversões, uma porta.
//
// ⚠️ **A FONTE NÃO É PREMULTIPLICADA, e a pilha inteira afirmava que era.** Medido no rasterizador
// REAL, com uma estrela: dos **1696 texels de cobertura parcial, 1696 trazem a cor CHEIA**
// `(235,175,60)` com o alfa baixo ao lado — zero premultiplicados. O `render_to_intermediate`
// entrega alfa reto, e o doc deste módulo dizia o contrário.
//
// ⚠️ **Por que isso atravessou dezenas de gates verdes:** num texel OPACO e num texel VAZIO as duas
// convenções dão exatamente os mesmos bytes. Toda fixture com cobertura parcial deste módulo foi
// escrita pela mesma mão que escreveu a premissa, então nenhuma podia contradizê-la; e a única
// coisa capaz de arbitrar — comparar com o que o Vello de facto escreve — só passou a existir
// quando a sonda ganhou o modo `PH2D_FX_VELLO=1`. O sintoma renderizado era o **contorno
// tracejado** do feather: fora da silhueta a cor reta saía até 40 níveis clara.
//
// A álgebra a jusante EXIGE alfa associado — Porter-Duff, o borrão que soma cor com peso, o
// `inner_tint`, o halo por baixo. Premultiplicar aqui é o que torna tudo isso válido, e fazê-lo
// DEPOIS de linearizar é o que o torna correto (multiplicar cobertura por INTENSIDADE).
//
// ⚠️ **Roda SEMPRE, inclusive para uma pilha vazia.** Um flag lido dentro do `tap_img` custaria um
// ramo no laço mais interno do borrão E deixaria o `resolve` com duas convenções para escolher.
// Uma porta, uma resposta: **do `cs_ingest` em diante tudo é linear e premultiplicado**.
//
// O `rgba16float` dos intermediários não é luxo aqui: linear em 8 bits BANDEARIA nos tons escuros
// (a transferência comprime a faixa baixa por um fator ~12,9), e é para isso que meia-precisão em
// ponto flutuante existe num pipeline de composição.
@compute @workgroup_size(8, 8, 1)
fn cs_ingest(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let s = tap_img(t0, i32(id.x), i32(id.y));
    textureStore(
        dst,
        vec2<i32>(i32(id.x), i32(id.y)),
        vec4<f32>(srgb_to_linear3(s.rgb) * s.a, s.a),
    );
}

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
        outc = inner_tint(over, tint_lin(), b.a * g.tint.a * g.opacity);
    } else {
        let a = b.a * g.tint.a * g.opacity;
        let halo = vec4<f32>(tint_lin() * a, a);
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
    //
    // ⚠️ É aqui que o Color Overlay ganha as vinte leis, e é o que o torna o RECOLORIDOR do módulo:
    // em `Color` ele troca a matiz preservando a luminosidade (o *tint/duotone* que a fila listava
    // como item à parte), em `Multiply` tinge sem apagar o sombreado.
    let rgb = mix(src.rgb, fx_blend(src, tint_lin()) * src.a, k);
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), vec4<f32>(rgb, src.a));
}

"#;

/// O passe final, que escreve em `rgba8unorm` com alfa RETO (o que o Vello amostra).
pub(crate) const FX_STACK_OUT_WGSL: &str = r#"
@group(0) @binding(3) var dst: texture_storage_2d<rgba8unorm, write>;

// **A PORTA DE SAÍDA**: linear premultiplicado -> sRGB reto, que é o que o `register_texture` do
// Vello amostra.
//
// ⚠️ **A divisão pelo alfa acontece em LINEAR, e é a metade que faltava.** Premultiplicar é
// multiplicar a INTENSIDADE por uma cobertura, então desfazê-lo só é a inversa no espaço em que a
// multiplicação ocorreu. Feito sobre os bytes sRGB, `rgb/a` sobre-corrige: a cor de um texel de
// meia cobertura saía ~1,45× clara, e num texel de um quarto saía BRANCA — a lavagem que o smoke
// leu como dentes na borda do feather.
@compute @workgroup_size(8, 8, 1)
fn cs_resolve(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let premul = textureLoad(t0, vec2<i32>(i32(id.x), i32(id.y)), 0);
    var rgb = vec3<f32>(0.0);
    if (premul.a > 0.0001) { rgb = linear_to_srgb3(premul.rgb / premul.a); }
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
         const KIND_GLOW: u32 = {}u;\n\
         const KIND_INNER_SHADOW: u32 = {}u;\n\
         const KIND_INNER_GLOW: u32 = {}u;\n\
         const KIND_OUTLINE: u32 = {}u;\n\
         const KIND_FEATHER: u32 = {}u;\n\
         const KIND_BEVEL: u32 = {}u;\n\
         const KIND_MORPHOLOGY: u32 = {}u;\n\
         const MODE_CONTOUR: u32 = {}u;\n\
         const MODE_CREASED: u32 = {}u;\n",
        FxOp::BLUR,
        FxOp::GLOW,
        FxOp::INNER_SHADOW,
        FxOp::INNER_GLOW,
        FxOp::OUTLINE,
        FxOp::FEATHER,
        FxOp::BEVEL,
        FxOp::MORPHOLOGY,
        FxOp::MODE_CONTOUR,
        FxOp::MODE_CREASED,
    )
}
