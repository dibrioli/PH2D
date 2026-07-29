//! **O RUÍDO PROCEDURAL da pilha de FX** — o campo que a turbulência lê, e o passe que deforma a
//! imagem com ele (plano 24 W6b).
//!
//! Irmão de [`super::fx_stack_shader`] pelo teto de LOC, e o corte é por RESPONSABILIDADE: aquele
//! arquivo é *o FOLD* (ingest → borrão/campo de distância → resolve), este é *o CAMPO* — uma
//! função de posição que não sabe o que é um degrau, mais a única porta que a consome.
//!
//! Os dois blocos são **prefixados** aos módulos WGSL pelo `module_sources` do
//! [`super::fx_stack`], junto com o das leis de mistura: um `include` de WGSL não existe, e uma
//! cópia por módulo divergiria — que é exatamente o que a extração das leis de mistura curou.
//!
//! ⚠️ Nenhum dos dois PARSEIA sozinho (o `NOISE_WGSL` chama `MODE_CREASED`, gerado do
//! `ph2d_ecs::FxOp`; o `WARP_WGSL` chama `tap_img`, do fold). São prefixos de módulo, não módulos —
//! o gate que os valida é o `the_fx_stack_modules_parse_and_validate_via_naga`, sobre a fonte
//! MONTADA.

/// O campo: hash → gradiente → Perlin → a soma de oitavas.
pub(crate) const NOISE_WGSL: &str = r#"
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
"#;

/// O passe que DEFORMA a imagem com o campo (o entry point `cs_op_warp` + o sampler dele).
pub(crate) const WARP_WGSL: &str = r#"
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
