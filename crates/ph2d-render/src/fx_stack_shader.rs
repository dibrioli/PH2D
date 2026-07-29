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
    _pad: vec2<f32>,
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
fn seeds_shell() -> bool {
    return g.kind == KIND_FEATHER || g.kind == KIND_OUTLINE || g.kind == KIND_BEVEL
        || g.kind == KIND_GLOW;
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

// ── O RUÍDO PROCEDURAL ────────────────────────────────────────────────────────────────────────
//
// **Gradiente (Perlin), não valor.** O ruído de VALOR tem extremos ancorados nos nós da grade,
// então a estrutura dele é visivelmente quadriculada — e um deslocamento amplifica isso, porque a
// silhueta passa a trair a grade. O de gradiente vale ZERO em todo nó (o extremo cai no meio da
// célula) e é isto que o `feTurbulence` do SVG especifica.
//
// ⚠️ **A grade é ANCORADA na forma, não na textura** — ver o `noise_p` do `cs_op_warp`.

fn hash_cell(c: vec2<i32>, seed: u32) -> u32 {
    // Um hash inteiro barato (xorshift-multiply). Determinístico e sem tabela: uma permutação
    // como a do SVG teria de viajar num buffer, e o padrão dela é o que o `seed` existe para
    // trocar.
    var h = u32(c.x) * 0x9e3779b9u ^ u32(c.y) * 0x85ebca6bu ^ seed * 0xc2b2ae35u;
    h = (h ^ (h >> 15u)) * 0x2545f491u;
    h = h ^ (h >> 13u);
    return h;
}

// O gradiente unitário de um nó da grade.
fn grad_at(c: vec2<i32>, seed: u32) -> vec2<f32> {
    let h = hash_cell(c, seed);
    let x = f32(h & 0xffffu) * (2.0 / 65535.0) - 1.0;
    let y = f32((h >> 16u) & 0xffffu) * (2.0 / 65535.0) - 1.0;
    // O `+eps` no x tira o caso degenerado de um par que casse exatamente em zero — `normalize`
    // de um vetor nulo é NaN, e um NaN aqui viaja para a posição de amostragem.
    return normalize(vec2<f32>(x + 1.0e-6, y));
}

// Perlin 2D em `[-1,1]` (o valor cru vive em ±√2/2; o fator o normaliza).
fn perlin(p: vec2<f32>, seed: u32) -> f32 {
    let cell = floor(p);
    let f = p - cell;
    let c = vec2<i32>(cell);
    // A quíntica de Perlin (2ª derivada nula nos nós) — a cúbica antiga deixa costura visível
    // exatamente onde o deslocamento é maior.
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let n00 = dot(grad_at(c, seed), f);
    let n10 = dot(grad_at(c + vec2<i32>(1, 0), seed), f - vec2<f32>(1.0, 0.0));
    let n01 = dot(grad_at(c + vec2<i32>(0, 1), seed), f - vec2<f32>(0.0, 1.0));
    let n11 = dot(grad_at(c + vec2<i32>(1, 1), seed), f - vec2<f32>(1.0, 1.0));
    let nx0 = mix(n00, n10, u.x);
    let nx1 = mix(n01, n11, u.x);
    return mix(nx0, nx1, u.y) * 1.4142136;
}

// **A soma de oitavas.** Cada uma tem metade do tamanho e metade da amplitude da anterior; a
// divisão pela soma dos pesos mantém a saída em `[-1,1]` seja qual for o `Detail` — sem isso, o
// slider de detalhe seria também um slider de intensidade, e o artista veria a forma saltar ao
// mexer nele.
fn fbm(p: vec2<f32>, seed: u32) -> f32 {
    var sum = 0.0;
    var norm = 0.0;
    var amp = 1.0;
    var q = p;
    let creased = g.mode == MODE_CREASED;
    for (var o = 0u; o < g.octaves; o = o + 1u) {
        // ⚠️ **Cada oitava tem SEMENTE PRÓPRIA, e isto é load-bearing — MEDIDO.** Com a mesma
        // semente, todas as oitavas leem a MESMA tabela de gradientes em células espacialmente
        // relacionadas (dobrar a frequência leva a célula `c` à célula `2c`), então elas ficam
        // correlacionadas e as quebras de inclinação de uma reforçam as da seguinte. O efeito não
        // é sutil: a rugosidade do modo **Smooth** sobe de 0,419 para 0,609 e **encosta na do
        // Creased** (0,602) — ou seja, sem esta linha o modo liso deixa de ser liso e os dois
        // modos desenham a mesma coisa. É o gate `the_creased_mode_breaks_the_slope…` que sangra.
        var n = perlin(q, seed + o * 0x9e37u);
        if (creased) { n = abs(n); }
        sum = sum + n * amp;
        norm = norm + amp;
        q = q * 2.0;
        amp = amp * 0.5;
    }
    var r = sum / max(norm, 1.0e-6);
    // O `turbulence` do SVG soma MÓDULOS ⇒ vive em `[0,1]`. Recentrar aqui (e não por oitava) é o
    // que preserva os VINCOS: recentrar antes de somar devolveria o campo com sinal.
    if (creased) { r = r * 2.0 - 1.0; }
    return r;
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

// **Onde a fronteira REALMENTE está dentro deste texel.** O alfa anti-aliased é, perto da borda,
// uma rampa de ~1 px ao longo da normal, então a fronteira (onde ele cruza 0,5) fica a `a - 0,5` do
// centro, na direção em que o alfa DECRESCE.
//
// ⚠️ **É isto que mata o PENTE.** Semear no centro do texel faz a distância saltar em degraus
// inteiros ao andar paralelo a uma aresta obliqua — medido, 33 níveis de oscilação numa aresta a
// 21,8° (a 45° o artefato some por simetria, e foi assim que ele passou pelo primeiro gate). Com a
// semente sub-texel a distância é contínua, e a correção de meio texel que existia à mão
// DESAPARECE: ela era o caso particular disto para uma borda dura.
// ⚠️ **Só o caminho SEM geometria chega aqui.** Havendo silhueta, o finalize computa o pé exato
// por texel e os passes de semente e salto nem são despachados — deixar um braço de geometria
// nesta função seria código morto que uma mutação não faz sangrar.
fn edge_offset(p: vec2<i32>, a: f32) -> vec2<f32> {
    let gx = tap_img(t0, p.x + 1, p.y).a - tap_img(t0, p.x - 1, p.y).a;
    let gy = tap_img(t0, p.x, p.y + 1).a - tap_img(t0, p.x, p.y - 1).a;
    let g = vec2<f32>(gx, gy);
    let m = length(g);
    if (m < 1.0e-5) { return vec2<f32>(0.0); }
    // ⚠️ A rampa de anti-aliasing NÃO tem 1 px de largura: numa aresta oblíqua ela é mais larga
    // (~|nx|+|ny|), então o alfa cai mais devagar e a fronteira está mais longe do que `a − 0,5`
    // sugere. A inclinação real é `|g|/2` (diferença central), logo a distância é `2(a−0,5)/|g|`.
    // Com a suposição de 1 px o campo errava ~0,09 px, e numa borda DURA isso lê como serrilha —
    // medido, 24 níveis de variação entre texels à mesma distância na borda do contorno.
    let t = clamp(2.0 * (a - 0.5) / m, -1.5, 1.5);
    return (-g / m) * t;
}

@compute @workgroup_size(8, 8, 1)
fn cs_sdf_seed(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let me = vec2<i32>(i32(id.x), i32(id.y));
    let a = tap_img(t0, me.x, me.y).a;
    var v = vec4<f32>(0.0);
    // **A CASCA da fronteira**: um texel de DENTRO com algum vizinho de fora. Uma regra só, e ela
    // dá o campo dos DOIS lados — o sinal vem do alfa de quem pergunta, não de outra semeadura.
    // (Fora da TEXTURA conta como fora da forma: o `tap_img` devolve transparente, então uma forma
    // encostada na borda tem casca ali, que é o que mantém o campo certo no limite do scratch.)
    if (seeds_shell()) {
        if (a > 0.5) {
            let l = tap_img(t0, me.x - 1, me.y).a;
            let r = tap_img(t0, me.x + 1, me.y).a;
            let u = tap_img(t0, me.x, me.y - 1).a;
            let d = tap_img(t0, me.x, me.y + 1).a;
            if (l <= 0.5 || r <= 0.5 || u <= 0.5 || d <= 0.5) {
                v = vec4<f32>(edge_offset(me, a), 1.0, 0.0);
            }
        }
    } else if (a <= 0.5) {
        v = vec4<f32>(edge_offset(me, a), 1.0, 0.0);
    }
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
                // Fora da textura não há CASCA (a casca é feita de texels da forma), então não há
                // semente — a assimetria que a semeadura por-lado exigia morreu com ela.
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

// A distância guardada no campo (ou "longe" onde o JFA não chegou).
fn field_dist(x: i32, y: i32) -> f32 {
    if (x < 0 || y < 0 || x >= i32(g.dims.x) || y >= i32(g.dims.y)) { return 1.0e6; }
    let f = textureLoad(t1, vec2<i32>(x, y), 0);
    if (f.z <= 0.5) { return 1.0e6; }
    return length(f.xy);
}

// **A normal do rebordo, pelo GRADIENTE do campo.**
//
// ⚠️ **Não use `normalize(off)`.** O vetor até a semente aponta para UMA semente, então ele salta na
// fronteira entre células de Voronoi — a distância continua exata (texels à mesma distância dão o
// mesmo número, há gate), mas a DIREÇÃO fica em degraus, e é ela que o bevel lê. Era esse o PENTE.
// O gradiente é uma diferença central de uma grandeza que já é suave, então não tem esse salto.
fn field_normal(x: i32, y: i32, fallback: vec2<f32>) -> vec2<f32> {
    let dx = field_dist(x + 1, y) - field_dist(x - 1, y);
    let dy = field_dist(x, y + 1) - field_dist(x, y - 1);
    let grad = vec2<f32>(dx, dy);
    // O gradiente cresce PARA DENTRO; a normal externa é o oposto.
    if (dot(grad, grad) < 1.0e-8 || abs(dx) > 1.0e5 || abs(dy) > 1.0e5) {
        return fallback;
    }
    return -normalize(grad);
}

// **O finalize sobre o CAMPO DE DISTÂNCIA** — serve QUATRO tipos: os degraus de dentro em modo
// Contour, o contorno, o feather e o bevel. Todos perguntam a mesma coisa (*a que distância da
// borda estou, e de que lado?*) e cada um responde com uma lei diferente.
@compute @workgroup_size(8, 8, 1)
fn cs_op_field(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let over = tap_img(t0, i32(id.x), i32(id.y));
    // ⚠️ O par de offset quer dizer coisas DIFERENTES conforme o tipo, e é por isso que a tabela o
    // ROTULA: numa sombra ele é um DESLOCAMENTO (amostra-se o campo mais adiante, e a banda anda
    // para o lado da luz); num bevel é uma DIREÇÃO (a luz), e deslocar por ela moveria o relevo
    // inteiro em vez de o iluminar.
    let disp = select(vec2<i32>(g.off_x, g.off_y), vec2<i32>(0), g.kind == KIND_BEVEL);
    let sx = i32(id.x) - disp.x;
    let sy = i32(id.y) - disp.y;
    let at = tap_img(t0, sx, sy);
    let inside = at.a > 0.5;
    var off = vec2<f32>(0.0);
    var far = true;
    if (g.n_segs > 0u) {
        // ⚠️ **Com geometria o JFA não responde nada — ele só PROPAGA.** Um texel que herda a
        // semente do vizinho recebe o vetor até o pé DAQUELE vizinho, não até o seu próprio: o
        // comprimento erra pouco (a envoltória de cones acerta a distância a menos de `s²/8d`),
        // mas a DIREÇÃO salta ao trocar de célula, e é dela que o bevel vive. Medido: com o campo
        // já exato pela semente, o feather caiu para 1,28 níveis e o bevel ficou em 117 — a prova
        // de que o que sobrava era a direção herdada, não o campo.
        //
        // O pé exato POR TEXEL custa o laço de segmentos onde a semente já o custava, e responde
        // as duas perguntas de uma vez.
        off = exact_foot(vec2<f32>(f32(sx), f32(sy)));
        far = false;
    } else if (sx >= 0 && sy >= 0 && sx < i32(g.dims.x) && sy < i32(g.dims.y)) {
        let f = textureLoad(t1, vec2<i32>(sx, sy), 0);
        if (f.z > 0.5) { off = f.xy; far = false; }
    }
    // ⚠️ MEIO TEXEL, e ele é DERIVADO: a casca é a primeira fileira DE DENTRO, cujo centro está a
    // 0,5 px da fronteira. Somando de dentro e subtraindo de fora, os dois lados começam em 0,5 —
    // o campo fica simétrico, que é o que um feather centrado na borda exige.
    // ⚠️ Sem correção nenhuma: a semente já aponta para a FRONTEIRA dentro do próprio texel
    // (`edge_offset`), então `|off|` É a distância. O meio texel que se somava à mão era o caso
    // particular disto para uma borda dura — e era ele que deixava o campo em degraus.
    var dist = 1.0e6;
    if (!far) {
        dist = length(off);
    }
    let sdist = select(-dist, dist, inside);
    let w = max(g.band, 1.0e-4);
    var outc: vec4<f32>;
    if (g.kind == KIND_FEATHER) {
        // A borda vira uma RAMPA CENTRADA na fronteira, sem borrar o miolo — é o que separa isto
        // de um Blur.
        //
        // ⚠️ **O ALFA é função da distância e a COR é RETA.** É a lei das três implementações
        // canônicas, e nenhuma delas reamostra cor: o feather do GIMP é um blur gaussiano da
        // MÁSCARA (σ = raio/3,5), o do Krita é uma gaussiana com `channelFlags(false, true)` — só
        // alfa —, e nos layer styles a cor entra DEPOIS, como fill.
        //
        // A lei anterior compunha `base * f` com `base` PREMULTIPLICADO, ou seja o alfa saía
        // `a_fonte · f` quando devia sair `f`: a cobertura era contada DUAS vezes, e só na fileira
        // do contorno (a única com `a_fonte` parcial). E a cor da metade de fora era buscada
        // andando `dir·0,5` a partir de `off`, com `dir` derivado de um `off` quase nulo — perto do
        // contorno ele desandava (medido: 50° fora) e o passo caía em texel transparente, com o
        // fallback disparando de forma intermitente. O resultado renderizado não era uma linha
        // escura: eram **459 texels de alfa ZERO** espalhados por 206 linhas, cercados por forma
        // dos dois lados. Um FURO, e a intermitência é o que o olho lê como tracejado.
        //
        // Agora não há direção a adivinhar: **onde a fonte existe, ela É a resposta** (o contorno
        // inteiro cai aqui, que é exatamente onde o furo nascia), e só onde não há nada é que se
        // busca a borda — para onde `off` já aponta, sem passo e sem fallback.
        let src = vec2<f32>(f32(sx), f32(sy));
        var straight = vec3<f32>(0.0);
        if (over.a > 0.1) {
            straight = over.rgb / over.a;
        } else {
            // ⚠️ A busca da cor NÃO pode ter modo de falha: devolver preto e ainda escrever alfa
            // pinta um DENTE escuro (medido — foi o que a primeira versão desta cura fez, trocando
            // o furo por um pente). Uma amostra única falha porque o `round` do ponto de fronteira
            // às vezes cai no vizinho ainda transparente.
            //
            // A extensão é a média das cores RETAS da vizinhança do ponto de fronteira, PESADA
            // pela cobertura — e ela é exatamente `Σ rgb_premultiplicado / Σ alfa`, porque cada
            // termo já vem multiplicado pelo próprio peso. Basta UM vizinho com tinta para a
            // resposta existir, e no ponto de fronteira isso é garantido por construção.
            //
            // ⚠️ Um peso ao QUADRADO foi construído aqui e REMOVIDO por medição, para ninguém o
            // reintroduzir: o argumento era que um vizinho de alfa 1/255 carrega uma cor reta
            // destruída pela quantização (a tinta premultiplicada arredonda para (1,1,0), cuja cor
            // reta é (255,255,0)). Verdade — e IRRELEVANTE: esse vizinho pesa 1/255 sobre um
            // `Σ alfa ≈ 4`, ou seja 0,1% de uma cor 4× errada = **1 nível**. A mutação que troca
            // o quadrado pelo linear NÃO sangra, e foi ela que expôs que o número que eu usara
            // para justificar o quadrado (7255 níveis) era um defeito do GATE, não do peso.
            let b = round(src + off);
            var acc = vec3<f32>(0.0);
            var wsum = 0.0;
            for (var dy = -1; dy <= 1; dy = dy + 1) {
                for (var dx = -1; dx <= 1; dx = dx + 1) {
                    let s = tap_img_at(b + vec2<f32>(f32(dx), f32(dy)));
                    acc = acc + s.rgb;
                    wsum = wsum + s.a;
                }
            }
            if (wsum > 1.0e-4) { straight = acc / wsum; }
        }
        let f = smoothstep(-w * 0.5, w * 0.5, sdist);
        outc = mix(over, vec4<f32>(straight * f, f), g.opacity);
    } else if (g.kind == KIND_BEVEL) {
        // O relevo da borda: a face virada para a LUZ clareia, a oposta escurece, e o efeito morre
        // para o miolo. `off` aponta para a borda mais próxima, então ele É a normal 2D do rebordo.
        var shade = 0.0;
        if (!far && inside) {
            // ⚠️ **Com o pé exato, a normal NÃO se estima: ela É `off`.** Por definição de ponto
            // mais próximo, o vetor do texel até o pé é perpendicular à silhueta — então derivar
            // um gradiente do campo aqui seria estimar por diferenças finitas o que já se tem
            // exato. O `field_normal` fica para o caminho sem geometria.
            var n = normalize(off + vec2<f32>(1.0e-6, 0.0));
            if (g.n_segs == 0u) { n = field_normal(sx, sy, n); }
            let lit = vec2<f32>(f32(g.off_x), f32(g.off_y));
            let l = select(vec2<f32>(0.0, -1.0), normalize(lit), dot(lit, lit) > 0.0);
            // ⚠️ **O relevo é a INCLINAÇÃO do rebordo, e ela é ZERO na silhueta.**
            //
            // O perfil antigo (`1 − smoothstep(0, w, dist)`) vale **1 em `dist = 0`**, ou seja
            // punha o valor EXTREMO do sombreado no texel mais externo da forma: o lado escuro
            // saía preto no fio da borda e o claro saía branco. Era isso que o smoke reportou como
            // "linhas pretas" — não um artefato numérico, mas o perfil errado.
            //
            // Um bevel é uma quina arredondada: a superfície começa PLANA na silhueta, sobe pela
            // banda e volta a ficar plana no miolo. Com a altura `h(t) = smoothstep(0,1,t)`, a
            // componente horizontal da normal é `h'(t) = 6t(1−t)` — que se anula nas DUAS pontas e
            // pica no meio da banda. Normalizada ao pico: `4t(1−t)`.
            //
            // É a mesma figura que o Bevel & Emboss do Photoshop desenha (a faixa de luz mora
            // DENTRO da banda, não no contorno), e mata a linha dura sem tocar no campo.
            let t = clamp(dist / w, 0.0, 1.0);
            shade = dot(n, l) * (4.0 * t * (1.0 - t)) * g.opacity;
        }
        let colour = select(tint_lin(), vec3<f32>(1.0), shade > 0.0);
        outc = inner_tint(over, colour, abs(shade) * g.tint.a);
    } else if (is_inner()) {
        // ⚠️ **`sdist`, com SINAL — e é aqui que o Inner Shadow deslocado se conserta.**
        //
        // Com a distância sem sinal, um texel cujo ponto amostrado cai FORA da forma tem `dist`
        // grande outra vez, então a sombra DESVANECE justamente do lado onde ela devia estar
        // saturada: a banda descola do contorno e deixa uma tira clara entre a borda e a sombra.
        // Medido numa aresta reta com deslocamento 8 (luminância por profundidade, tinta crua 180):
        // `110 96 81 64 45 24 3 9 31 52 …` — o ponto MAIS ESCURO ficava 7 texels dentro, e a borda
        // saía 3,6× mais clara que ele. Uma sombra interna é mais escura NA BORDA, sempre.
        //
        // Com sinal, o lado de fora satura (`smoothstep` de negativo é 0 ⇒ força 1) e o perfil
        // volta a ser monótono a partir da borda — que é o que a máscara-invertida-deslocada do
        // Photoshop desenha, e o que o modo Proximity (que borra uma REGIÃO, não uma distância)
        // sempre desenhou.
        //
        // ⚠️ Sem deslocamento é **byte-idêntico** ao anterior: para um texel de dentro
        // `sdist == +dist`, e um de fora é morto pelo `over.a` do `inner_tint`.
        outc = inner_tint(over, tint_lin(), (1.0 - smoothstep(0.0, w, sdist)) * g.tint.a * g.opacity);
    } else if (g.kind == KIND_GLOW) {
        // **GLOW em modo Contour**: uma banda de largura constante ao longo de TODO o contorno.
        //
        // O irmão Proximity (o borrão da silhueta) mede *quanta forma há por perto*, então o vão
        // entre duas pontas de uma estrela quase não brilha — o mesmo ângulo-subtendido que faz o
        // Inner Shadow não escurecer uma reentrância. A distância não tem essa dependência: ela é
        // zero em todo ponto do contorno.
        //
        // A queda vale exatamente 0 em `w` (por isso o `op_reach` deste caso é `w`, não `3σ`), e o
        // halo entra POR BAIXO da entrada — a mesma composição do irmão e do contorno, porque um
        // op tem de devolver UMA camada.
        let a = (1.0 - smoothstep(0.0, w, max(-sdist, 0.0))) * g.tint.a * g.opacity;
        let halo = vec4<f32>(tint_lin() * a, a);
        outc = over + halo * (1.0 - over.a);
    } else {
        // CONTORNO: a borda cai exatamente em `w`, com ~1 px de anti-aliasing. Isto é uma DILATAÇÃO
        // de verdade (`d <= w`), ao contrário do corte num campo borrado, que ENCOLHE na quina
        // convexa — medido, uma ponta de 36° não recebia contorno NENHUM.
        let outward = max(-sdist, 0.0);
        let cov = 1.0 - smoothstep(w - 0.5, w + 0.5, outward);
        let a = cov * g.tint.a * g.opacity;
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

// **A amostragem BILINEAR** — o warp lê entre texels por construção (o deslocamento é contínuo),
// e o vizinho mais próximo transformaria uma onda suave numa escada.
//
// ⚠️ Interpolar PREMULTIPLICADO é o correto e é o motivo de a pilha inteira trabalhar assim: a
// média de `(cor·α, α)` é a cor certa com a cobertura certa, enquanto a média de cor RETA pesa
// igual um texel transparente e um opaco, e a borda ganha um halo da cor do vazio.
fn sample_bilinear(p: vec2<f32>) -> vec4<f32> {
    let base = floor(p);
    let t = p - base;
    let x = i32(base.x);
    let y = i32(base.y);
    let c00 = tap_img(t0, x, y);
    let c10 = tap_img(t0, x + 1, y);
    let c01 = tap_img(t0, x, y + 1);
    let c11 = tap_img(t0, x + 1, y + 1);
    return mix(mix(c00, c10, t.x), mix(c01, c11, t.x), t.y);
}

// **A TURBULÊNCIA** — a imagem é DEFORMADA por um campo de ruído: `saída(p) = entrada(p + d(p))`.
//
// É o `feTurbulence` + `feDisplacementMap` do SVG **num degrau só**, que é como o AE (*Turbulent
// Displace*) e todo mundo depois dele o embrulharam. Numa pilha LINEAR como a nossa a fusão não é
// conveniência: um degrau que só GERASSE ruído teria de escrever a saída dele por cima da imagem
// que o degrau seguinte espera receber, e a pilha inteira é *imagem → imagem*.
//
// ⚠️ **A grade do ruído é ancorada na FORMA, não no scratch.** `g.org` é a margem que a pilha
// reservou (a mesma `stack_reach` que dimensionou a textura), então `(p − org)` é a posição
// relativa à caixa da forma NA TELA, e dividir pelo tamanho em pixels cancela o zoom:
// `(mundo − caixa_min)/tamanho_mundo`. Sem o `org`, mexer no raio de QUALQUER outro degrau muda a
// margem e o padrão inteiro **anda** — um efeito colateral entre degraus que ninguém consegue
// atribuir.
//
// ⚠️ **Dois campos independentes, não um girado**: `x` e `y` saem de sementes distantes. Um único
// campo usado nos dois eixos deslocaria tudo ao longo da diagonal.
@compute @workgroup_size(8, 8, 1)
fn cs_op_warp(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let p = vec2<f32>(f32(id.x), f32(id.y));
    let src = tap_img(t0, i32(id.x), i32(id.y));
    let np = (p - g.org) / max(g.noise_scale, 1.0e-3);
    let d = vec2<f32>(fbm(np, g.seed), fbm(np + vec2<f32>(137.13, 71.77), g.seed + 0x51ed2701u));
    let warped = sample_bilinear(p + d * g.band);
    // A opacidade é *quanto DESTE efeito*, como no Blur: em 0 o degrau é no-op byte-idêntico.
    textureStore(
        dst,
        vec2<i32>(i32(id.x), i32(id.y)),
        mix(src, warped, clamp(g.opacity, 0.0, 1.0)),
    );
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
         const MODE_CONTOUR: u32 = {}u;\n\
         const MODE_CREASED: u32 = {}u;\n",
        FxOp::BLUR,
        FxOp::GLOW,
        FxOp::INNER_SHADOW,
        FxOp::INNER_GLOW,
        FxOp::OUTLINE,
        FxOp::FEATHER,
        FxOp::BEVEL,
        FxOp::MODE_CONTOUR,
        FxOp::MODE_CREASED,
    )
}
