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

// **Quem precisa do campo FORA da forma?** O feather (a rampa é centrada na fronteira) e o contorno
// (ele mora todo lá fora). Esses semeiam a CASCA — a primeira fileira de dentro —, que dá os dois
// lados de uma vez. Quem só olha para dentro semeia os texels de FORA, que é a medida exata do que
// eles perguntam: *a que distância estou de deixar de existir*. ⚠️ A diferença aparece na quina
// CÔNCAVA — a casca de um lado só a estima ~0,6 px pior, e é justamente ali que o modo Contour
// existe para acertar.
fn seeds_shell() -> bool { return g.kind == KIND_FEATHER || g.kind == KIND_OUTLINE; }

// O halo de dentro, aplicado com a lei que NÃO move a cobertura: um efeito de dentro tinge o que
// já está lá, ele não é uma camada nova. Porta única de TODOS os que moram dentro.
fn inner_tint(over: vec4<f32>, colour: vec3<f32>, strength: f32) -> vec4<f32> {
    let tinted = vec4<f32>(colour * over.a, over.a);
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
        outc = inner_tint(over, g.tint.rgb, b.a * g.tint.a * g.opacity);
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

// **Onde a fronteira REALMENTE está dentro deste texel.** O alfa anti-aliased é, perto da borda,
// uma rampa de ~1 px ao longo da normal, então a fronteira (onde ele cruza 0,5) fica a `a - 0,5` do
// centro, na direção em que o alfa DECRESCE.
//
// ⚠️ **É isto que mata o PENTE.** Semear no centro do texel faz a distância saltar em degraus
// inteiros ao andar paralelo a uma aresta obliqua — medido, 33 níveis de oscilação numa aresta a
// 21,8° (a 45° o artefato some por simetria, e foi assim que ele passou pelo primeiro gate). Com a
// semente sub-texel a distância é contínua, e a correção de meio texel que existia à mão
// DESAPARECE: ela era o caso particular disto para uma borda dura.
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
    if (sx >= 0 && sy >= 0 && sx < i32(g.dims.x) && sy < i32(g.dims.y)) {
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
        // A borda vira uma RAMPA CENTRADA na fronteira — sem borrar o miolo, que é o que separa
        // isto de um Blur. Fora da forma o pixel não tem cor própria: ele herda a do texel de
        // borda mais próximo, que é justamente para onde o campo aponta.
        // A cor da metade de FORA vem do texel de borda mais próximo. ⚠️ Duas cautelas, e as duas
        // custaram artefato: **arredondar** (truncar um passo de −0,5 devolvia o próprio texel,
        // transparente, apagando a metade de fora) e **cair DENTRO** — a amostra oblíqua às vezes
        // pousava num texel ainda transparente, e o buraco resultante é um dente de pente. Se o
        // primeiro passo falha, anda mais um texel na mesma direção.
        let here = vec2<f32>(f32(id.x), f32(id.y));
        let dir = normalize(off + vec2<f32>(1.0e-6, 0.0));
        var edge = tap_img_at(round(here + off + dir * 0.5));
        if (edge.a < 0.5) { edge = tap_img_at(round(here + off + dir * 1.5)); }
        let base = select(edge, over, over.a > 0.5);
        let f = smoothstep(-w * 0.5, w * 0.5, sdist);
        outc = mix(over, base * f, g.opacity);
    } else if (g.kind == KIND_BEVEL) {
        // O relevo da borda: a face virada para a LUZ clareia, a oposta escurece, e o efeito morre
        // para o miolo. `off` aponta para a borda mais próxima, então ele É a normal 2D do rebordo.
        var shade = 0.0;
        if (!far && inside) {
            let n = field_normal(sx, sy, normalize(off + vec2<f32>(1.0e-6, 0.0)));
            let lit = vec2<f32>(f32(g.off_x), f32(g.off_y));
            let l = select(vec2<f32>(0.0, -1.0), normalize(lit), dot(lit, lit) > 0.0);
            shade = dot(n, l) * (1.0 - smoothstep(0.0, w, dist)) * g.opacity;
        }
        let colour = select(g.tint.rgb, vec3<f32>(1.0), shade > 0.0);
        outc = inner_tint(over, colour, abs(shade) * g.tint.a);
    } else if (is_inner()) {
        outc = inner_tint(over, g.tint.rgb, (1.0 - smoothstep(0.0, w, dist)) * g.tint.a * g.opacity);
    } else {
        // CONTORNO: a borda cai exatamente em `w`, com ~1 px de anti-aliasing. Isto é uma DILATAÇÃO
        // de verdade (`d <= w`), ao contrário do corte num campo borrado, que ENCOLHE na quina
        // convexa — medido, uma ponta de 36° não recebia contorno NENHUM.
        let outward = max(-sdist, 0.0);
        let cov = 1.0 - smoothstep(w - 0.5, w + 0.5, outward);
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
         const KIND_FEATHER: u32 = {}u;\n\
         const KIND_BEVEL: u32 = {}u;\n\
         const MODE_CONTOUR: u32 = {}u;\n",
        FxOp::BLUR,
        FxOp::INNER_SHADOW,
        FxOp::INNER_GLOW,
        FxOp::OUTLINE,
        FxOp::FEATHER,
        FxOp::BEVEL,
        FxOp::MODE_CONTOUR,
    )
}
