//! ⭐⭐⭐ **O WGSL DO TRAÇADO** — o texto dos shaders, separado de quem os despacha.
//!
//! # ⚠️ Porque ele é DOIS pedaços e não um
//!
//! O passe que **pinta** ([`crate::paint`]) não marcha nada: ele lê o que a marcha escreveu. Mas
//! precisa das MESMAS três coisas — a `struct Setup`, os bindings do grupo `0` e a reconstrução do
//! raio de cada pixel. ⇒ o [`COMUM`] é o que os dois leem e o [`MARCHA`] é o que só a marcha tem.
//!
//! ⛔⛔ **Uma segunda cópia do `ray_at_plane` seria a segunda resposta à mesma pergunta**, e a nota
//! do topo do [`crate::trace`] já avisa contra ela por escrito: *«duas respostas para «que raio sai
//! daqui?»»*. A imagem que o pintor entrega é feita do ponto que a marcha achou — se os dois
//! reconstruírem o raio de maneiras diferentes, o material é lido num sítio e a luz noutro.

/// A parte que o traçado e o pintor partilham: o uniforme, os bindings do grupo `0` e o raio.
pub(crate) const COMUM: &str = r"
struct Setup {
    w: u32, h: u32, budget: u32, ao_rays: u32,
    // ⭐ `chao`: `1` quando o quadro tem CHÃO (`docs/Render3d/07`), e a altura dele é o `chao_y`.
    // ⭐⭐⭐ `mole`: `1` quando este quadro tem o canal da BORDA MOLE (`docs/Render3d/10` §12) — o
    // raio dele, por canal e em PÍXEIS, vive no `mole_raio`. `0` é o passo de sempre, ao bit.
    // ⭐⭐⭐ `longe`: o índice do cabeçalho da GRADE DE LONGE no `k`, MAIS UM — `0` é a marcha de
    // sempre, ao bit (`crate::longe`).
    n_lamps: u32, chao: u32, mole: u32, longe: u32,
    half_extent: f32, half_px: f32, ortho_start: f32, eye_distance: f32,
    hit_eps: f32, normal_eps: f32, step: f32, t_max: f32,
    ball_radius: f32, ao_reach: f32, edge_cos: f32, chao_y: f32,
    alvo: vec3<f32>, right: vec3<f32>, up: vec3<f32>, fwd: vec3<f32>,
    ball_center: vec3<f32>,
    // ⭐⭐⭐ **O RAIO DA BORDA MOLE, por canal e em PÍXEIS** — o `sss_shadow::raio_em_pixeis` da CPU.
    // ⚠️ Ele é por CANAL porque a distância de espalhamento é por canal, e é isso que faz a borda
    // ficar avermelhada num jade: o vermelho viaja mais e entra mais fundo na sombra.
    mole_raio: vec3<f32>,
    // ⭐⭐⭐ **AS LÂMPADAS, e não uma** — `xyz` é a posição no MUNDO. Ver `MAX_LAMPS`.
    lamps: array<vec4<f32>, {MAX_LAMPS}>,
};
@group(0) @binding(0) var<uniform> s: Setup;
@group(0) @binding(1) var<storage, read> k: array<f32>;
@group(0) @binding(2) var<storage, read_write> centro: array<vec4<f32>>;
// ⭐⭐⭐ **A LUZ POR PIXEL, com passo `1 + n_lamps`**: o slot `0` é o CÉU e os seguintes são a
// visibilidade de cada lâmpada.
//
// ⛔⛔ **Era um `vec2` — o céu e UMA sombra — e isso era o tecto de uma lâmpada.** Com duas, a
// segunda ficava sem sombra **em silêncio**, e por isso o chamador caía na CPU inteira em vez de a
// ignorar. *Um formato que não tem onde pôr a segunda resposta é um tecto escrito em bytes.*
@group(0) @binding(3) var<storage, read_write> luz: array<f32>;
@group(0) @binding(4) var<storage, read_write> conta: atomic<u32>;
@group(0) @binding(5) var<storage, read_write> borda: array<vec4<f32>>;
// ⭐⭐⭐ **AS GRADES DAS ESCULTURAS, concatenadas** — o cabeçalho de cada uma vive no `k` e diz onde
// ela começa aqui. ⚠️ Ela sobe UMA VEZ e fica: `128³` são `8 MB`, e reenviá-la por quadro custaria
// mais barramento do que a imagem inteira que este passe veio poupar.
@group(0) @binding(6) var<storage, read> grades: array<f32>;

/// O passo de [`luz`] — o céu, uma visibilidade por lâmpada, e os TRÊS do ricochete.
///
/// ⭐⭐⭐ **O ricochete mora AQUI e não num canal ao lado** (`docs/Render3d/08`), e a razão é a que o
/// `ph2d_field_render::Shadows` já escreve: *«é a MESMA pergunta que as lâmpadas respondem —
/// quanto desta fonte chega a este pixel? Um segundo canal ao lado faria o pintor perguntar duas
/// vezes a mesma coisa, e é assim que dois canais divergem.»*
///
/// ⭐⭐ **E o ricochete ocupa SEIS e não três** — o CRU e o de uma passagem de borrão. Ver
/// `ph2d_field_render::BOUNCE_BLUR_PASSES`: a lei mede **duas** passagens de `3×3`, e o pintor só
/// consegue fazer uma delas ao ler (a outra tem de ser um despacho, com destino próprio — escrever
/// no mesmo sítio de onde os vizinhos estão a ler é uma corrida).
/// ⭐⭐⭐ **E a BORDA MOLE ocupa SEIS por lâmpada quando existe** (`docs/Render3d/10` §12): três do
/// intermediário da passagem HORIZONTAL e três do resultado, que é o que o pintor lê.
///
/// ⛔ Os dois não podem ser o mesmo sítio: a segunda passagem lê os vizinhos do que a primeira
/// escreveu, e escrever onde eles estão a ler é uma corrida — a mesma razão que os slots do
/// ricochete liso já pagam, um bloco acima.
///
/// ⚠️ **Com `mole = 0` o passo é o de sempre, ao bit**, e nenhuma cena de hoje paga um byte.
fn passo_da_luz() -> u32 { return 1u + s.n_lamps + 6u + s.mole * 6u * s.n_lamps; }

/// Onde começam os TRÊS do intermediário da lâmpada `l` — o que a `borra_mole_h` escreve.
fn base_do_mole_tmp(i: u32, l: u32) -> u32 { return i * passo_da_luz() + 7u + s.n_lamps + l * 3u; }

/// Onde começam os TRÊS da borda mole da lâmpada `l` — o que a `borra_mole_v` escreve e o pintor lê.
fn base_do_mole(i: u32, l: u32) -> u32 { return base_do_mole_tmp(i, s.n_lamps) + l * 3u; }

/// Onde começam os três `f32` do ricochete CRU deste pixel — o que a `pinta_ricochete` escreve.
fn base_do_ricochete(i: u32) -> u32 { return i * passo_da_luz() + 1u + s.n_lamps; }

/// Onde começam os três do ricochete já com UMA passagem de borrão — o que a `borra_ricochete`
/// escreve e a `ricochete_no_pixel` lê (e volta a borrar, o que dá as duas da lei).
fn base_do_ricochete_liso(i: u32) -> u32 { return base_do_ricochete(i) + 3u; }

// ⚠️ **A MESMA lei de marcha da CPU**, e ela é uma função porque as quatro amostras do
// anti-serrilhado a repetem: uma segunda cópia seria a segunda resposta à mesma pergunta.
fn raio(px: f32, py: f32) -> vec2<f32> {
    let u = (px - f32(s.w) * 0.5) / s.half_px * s.half_extent;
    let v = -(py - f32(s.h) * 0.5) / s.half_px * s.half_extent;
    return vec2<f32>(u, v);
}
struct Raio { o: vec3<f32>, d: vec3<f32> };
// ⭐⭐ **Onde o raio toca o CHÃO** (`xyz`), e `w = 1` quando toca — a `Ground::hit` da CPU, linha a
// linha: só de CIMA, e o `y` é a ALTURA escrita, nunca `o.y + d.y·t`.
fn chao_em(r: Raio) -> vec4<f32> {
    if (s.chao == 0u || !(r.d.y < 0.0)) { return vec4<f32>(0.0); }
    let t = (s.chao_y - r.o.y) / r.d.y;
    if (t <= 0.0) { return vec4<f32>(0.0); }
    return vec4<f32>(r.o.x + r.d.x * t, s.chao_y, r.o.z + r.d.z * t, 1.0);
}
fn ray_at_plane(uv: vec2<f32>) -> Raio {
    let on_plane = s.alvo + s.right * uv.x + s.up * uv.y;
    var r: Raio;
    if (s.eye_distance == 0.0) {
        r.o = on_plane + s.fwd * s.ortho_start;
        r.d = -s.fwd;
    } else {
        let eye = s.alvo + s.fwd * s.eye_distance;
        let d = on_plane - eye;
        let len = length(d);
        r.o = eye;
        if (len <= 0.0) { r.d = -s.fwd; } else { r.d = d / len; }
    }
    return r;
}";

/// O que só a marcha tem: a fita da peça, a marcha, a visibilidade e as duas passagens.
pub(crate) const LEIS: &str = r"
{TRILINEAR}
{LONGE}
{ESCULTURAS}
{FIELD}

// Devolve `vec4(t, normal em VISTA)`, com `t < 0` quando não acerta.
//
// ⚠️ **O alcance é ARGUMENTO desde o ricochete** (`docs/Render3d/08`): um raio de câmera anda até
// `s.t_max` e um raio de hemisfério anda até sair da bola que contém a peça. *Uma segunda marcha
// para a segunda pergunta seria a segunda resposta a «onde este raio para?», que é precisamente o
// que a nota do topo do `trace` proíbe.*
fn marcha(r: Raio) -> vec4<f32> {
    return marcha_ate(r, s.t_max);
}

fn marcha_ate(r: Raio, t_max: f32) -> vec4<f32> {
    var t = 0.0;
    var fim = t_max;
    var acertou = false;
    // ⭐⭐⭐ **A GRADE DE LONGE** (`crate::longe`) — com `s.longe = 0` nada disto corre e a marcha
    // é a de sempre, ao bit.
    //
    // ⚠️ **O recorte primeiro:** um raio que não toca a caixa da peça não tem superfície a achar.
    if (s.longe != 0u) {
        let c = longe_caixa(r);
        if (c.x > c.y || c.y <= 0.0) { return vec4<f32>(-1.0, 0.0, 0.0, 0.0); }
        t = max(c.x, 0.0);
        fim = min(t_max, c.y);
    }
    // ⚠️ **O orçamento conta só as avaliações da ÁRVORE** — é ela que o `budget` foi medido a
    // pagar. Os saltos têm tecto próprio, e cada um anda pelo menos `longe_perto()`.
    var n: u32 = 0u;
    var saltos: u32 = 0u;
    // ⚠️ **Decidido UMA vez por raio, fora do laço:** com o recorte sem grade (`res = 0`) a
    // pergunta à grade devolve sempre `0`, e fazê-la em todo passo custava uma leitura do `k` e um
    // ramo por passo — medido na cena `=1` (campo barato), `9,47 → 10,96 ms` só por isso.
    let com_grade = s.longe != 0u && longe_tem_grade() && !longe_so_ceu();
    loop {
        if (n >= s.budget || saltos >= {SALTOS_MAX}u || t >= fim) { break; }
        let p = r.o + r.d * t;
        if (com_grade) {
            let lb = longe_limite(p);
            if (lb > longe_perto()) {
                t = t + lb;
                saltos = saltos + 1u;
                continue;
            }
        }
        let d = field(p);
        n = n + 1u;
        if (d < s.hit_eps) { acertou = true; break; }
        t = t + d * s.step;
    }
    if (!acertou) { return vec4<f32>(-1.0, 0.0, 0.0, 0.0); }
    let p = r.o + r.d * t;
    let e = s.normal_eps;
    let o0 = vec3<f32>( 1.0, -1.0, -1.0);
    let o1 = vec3<f32>(-1.0, -1.0,  1.0);
    let o2 = vec3<f32>(-1.0,  1.0, -1.0);
    let o3 = vec3<f32>( 1.0,  1.0,  1.0);
    let world = o0 * field(p + o0 * e) + o1 * field(p + o1 * e)
              + o2 * field(p + o2 * e) + o3 * field(p + o3 * e);
    let len = length(world);
    if (len <= 0.0) { return vec4<f32>(-1.0, 0.0, 0.0, 0.0); }
    let nrm = world / len;
    return vec4<f32>(t, dot(nrm, s.right), dot(nrm, s.up), dot(nrm, s.fwd));
}

// ⭐ **A MARCHA DE VISIBILIDADE** — a da sombra e a da oclusão são a mesma, e diferem só na cerca
// e na dureza. `INFINITY` para a oclusão (a pergunta é binária); `8` para a sombra (penumbra).
fn visivel(origem: vec3<f32>, dir: vec3<f32>, t_max: f32, dureza: f32) -> f32 {
    var vis = 1.0;
    var t = s.hit_eps * 4.0;
    for (var n: u32 = 0u; n < s.budget; n = n + 1u) {
        let d = field(origem + dir * t);
        if (d < s.hit_eps) { return 0.0; }
        vis = min(vis, dureza * d / t);
        t = t + d * s.step;
        if (t >= t_max) { break; }
    }
    return vis;
}

// ⭐⭐⭐⭐ **A MESMA visibilidade, sobre o campo dos CONES** — ver `campo_do_ceu` na lei da grade de
// longe. ⚠️ Uma função irmã e não um argumento: o WGSL não passa funções, e um `if` dentro do laço
// da `visivel` pagaria a pergunta em todo passo de toda sombra.
fn visivel_ceu(origem: vec3<f32>, dir: vec3<f32>, t_max: f32, dureza: f32) -> f32 {
    var vis = 1.0;
    var t = s.hit_eps * 4.0;
    for (var n: u32 = 0u; n < s.budget; n = n + 1u) {
        let d = campo_do_ceu(origem + dir * t);
        if (d < s.hit_eps) { return 0.0; }
        vis = min(vis, dureza * d / t);
        t = t + d * s.step;
        if (t >= t_max) { break; }
    }
    return vis;
}

// ⭐⭐⭐ A direcção `k` do conjunto de cones — o `ph2d_field_render::cone_dir`, linha a linha.
// Reticulado de Fibonacci esférico em coordenadas de MUNDO: nem o pixel nem a câmera entram.
fn direccao_do_cone(k: u32, total: u32) -> vec3<f32> {
    let n = f32(max(total, 1u));
    let ki = f32(k);
    let z = 1.0 - (2.0 * ki + 1.0) / n;
    let r = sqrt(max(1.0 - z * z, 0.0));
    let phi = 6.283185307 * fract(ki * 0.618034);
    return vec3<f32>(r * cos(phi), r * sin(phi), z);
}

// A saída da bola, que é o DOMÍNIO da pergunta — ver `ph2d_field_render::shadow`.
fn cerca_da_bola(p: vec3<f32>, dir: vec3<f32>, ate: f32) -> f32 {
    return cerca_com(p, dir, ate, 0.0);
}

// A mesma, com a bola ALARGADA por `folga` — a cerca de um raio que parte do CHÃO usa
// `folga = distância à luz / dureza`. Ver `ph2d_field_render::shadow::cerca_com`: sem ela a penumbra
// era cortada numa elipse dura à volta da sombra.
fn cerca_com(p: vec3<f32>, dir: vec3<f32>, ate: f32, folga: f32) -> f32 {
    let raio = s.ball_radius + folga;
    let oc = p - s.ball_center;
    let b = dot(oc, dir);
    let c = dot(oc, oc) - raio * raio;
    let disc = b * b - c;
    if (disc <= 0.0) { return 0.0; }
    return clamp(-b + sqrt(disc), 0.0, ate);
}

// ⭐⭐⭐ **QUANTO DO CÉU CHEGA A UM PONTO DO CHÃO** — a `ph2d_field_render::ground::ground_sky`, com as
// mesmas constantes (elas são LIDAS do ficheiro que as declara) e a mesma ordem de soma.
fn ceu_do_chao(q: vec3<f32>) -> f32 {
    var soma = 0.0;
    var soma_w = 0.0;
    var w = 1.0;
    for (var k: u32 = 1u; k <= {CHAO_N}u; k = k + 1u) {
        let h = s.ao_reach * f32(k) / {CHAO_N}.0;
        let p = vec3<f32>(q.x, q.y + h, q.z);
        soma_w = soma_w + w;
        // A cerca: fora dela o termo é zero num campo de distância exacto.
        if (length(p - s.ball_center) - s.ball_radius < {CHAO_ESPALHA} * h) {
            let t = clamp(1.0 - field(p) / ({CHAO_ESPALHA} * h), 0.0, 1.0);
            soma = soma + w * t;
        }
        w = w * {CHAO_QUEDA};
    }
    return clamp(1.0 - {CHAO_FORCA} * (soma / soma_w), 0.0, 1.0);
}
";

/// ⭐⭐⭐ **OS DOIS KERNELS DA MARCHA** — o que só o traçado despacha.
///
/// ⚠️⚠️ **Eles saíram das [`LEIS`] quando o pintor passou a precisar das leis** (`docs/Render3d/08`
/// §12): o ricochete marcha a partir da superfície, logo o passe que PINTA precisa do campo, da
/// marcha e da visibilidade — e **não** precisa de declarar outra vez as duas entradas que escrevem
/// no `centro`, na `luz` e na lista de bordas. *Um segundo ponto de entrada num módulo que ninguém
/// despacha é código que não se apaga porque compila.*
pub(crate) const KERNELS: &str = r"
// ⭐⭐⭐⭐ **SÓ O CENTRO** — a marcha e a normal, e nada mais. É a entrada do MATCAP, que não lê a
// luz: ver a nota do `marcha_com`. ⚠️ Ela escreve o `centro` pela MESMA `marcha` que o
// `centro_e_luz`, logo os dois dão o mesmo G-buffer ao bit — só a luz fica por escrever.
@compute @workgroup_size(8, 8, 1)
fn centro_so(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    centro[i] = marcha(ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5)));
}

@compute @workgroup_size(8, 8, 1)
fn centro_e_luz(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let r = ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5));
    let c = marcha(r);
    centro[i] = c;
    escreve_a_luz(i, r, c);
}

// ⭐⭐⭐⭐ **SÓ A LUZ** — a metade do `centro_e_luz` que vem DEPOIS da marcha, sobre o `centro` que o
// `centro_so` já escreveu (`docs/Render3d/03` §W9). ⚠️ O raio é recalculado pela MESMA aritmética,
// logo o ponto e a normal que a luz lê são os mesmos ao bit.
@compute @workgroup_size(8, 8, 1)
fn luz_so(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    escreve_a_luz(i, ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5)), centro[i]);
}

fn escreve_a_luz(i: u32, r: Raio, c: vec4<f32>) {
    let base = i * passo_da_luz();
    if (c.x < 0.0) {
        // ⚠️ Um pixel que não acerta recebe **luz inteira** nos canais de SOMBRA — é o que a CPU
        // devolve (`vis` nasce a `1.0` e o laço salta quem não acerta).
        //
        // ⛔⛔ **Mas NÃO no ricochete, e a lei é a OPOSTA:** uma sombra que não foi calculada é
        // *ausência de sombra* (`1`); uma luz que não foi calculada é **ausência de luz** (`0`).
        // *Inventar luz é a única das duas que acende o que devia estar escuro.*
        for (var l: u32 = 0u; l <= s.n_lamps; l = l + 1u) { luz[base + l] = 1.0; }
        for (var c: u32 = 0u; c < 6u; c = c + 1u) { luz[base_do_ricochete(i) + c] = 0.0; }
        // ⭐⭐⭐ **O CHÃO QUE SÓ RECEBE**: o que este pixel mostra é o chão, e os canais passam a dizer
        // quanto de cada fonte chega A ELE. Ver `docs/Render3d/07`.
        let q = chao_em(r);
        if (q.w == 0.0) { return; }
        let up = vec3<f32>(0.0, 1.0, 0.0);
        let erguido = q.xyz + up * (s.hit_eps * 4.0);
        for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
            let d = s.lamps[l].xyz - q.xyz;
            let dist = length(d);
            // A normal do chão é `+y`: `N·L > 0` é a lâmpada estar ACIMA dele.
            if (dist <= 1e-6 || d.y <= 0.0) { continue; }
            let dir = d / dist;
            let ate = cerca_com(q.xyz, dir, dist, dist / 8.0);
            // ⚠️ **Um raio que nem toca a bola alargada não marcha** — a CPU salta-o também, e um
            // raio marchado com cerca `0` ainda avaliaria o campo uma vez.
            if (ate <= 0.0) { continue; }
            luz[base + 1u + l] = visivel(erguido, dir, ate, 8.0);
        }
        luz[base] = ceu_do_chao(q.xyz);
        return;
    }

    let p = r.o + r.d * c.x;
    // A normal volta ao MUNDO — a base é ortonormal, logo a transposta é a inversa.
    let n = s.right * c.y + s.up * c.z + s.fwd * c.w;
    let erguido = p + n * (s.hit_eps * 4.0);

    // ⭐⭐⭐ **A SOMBRA, UMA POR LÂMPADA** — só quem VÊ a luz recebe raio.
    //
    // ⚠️ **O custo é LINEAR nas lâmpadas e é a parte cara do passe**: um raio de sombra custa
    // `29,3` amostras contra `8,7` de um raio de câmera. O tecto de `MAX_LAMPS` sai daí.
    for (var l: u32 = 0u; l < s.n_lamps; l = l + 1u) {
        var sombra = 1.0;
        let d = s.lamps[l].xyz - p;
        let dist = length(d);
        if (dist > 1e-6) {
            let dir = d / dist;
            if (dot(n, dir) > 0.0) {
                sombra = visivel(erguido, dir, cerca_da_bola(erguido, dir, dist), 8.0);
            }
        }
        luz[base + 1u + l] = sombra;
    }

    // ⚠️ **O ricochete nasce a ZERO e é o passe do PINTOR que o enche** — ele precisa dos
    // materiais, que vivem no grupo `1` daquele passe. Sem esse passe o canal fica vazio, e um
    // canal vazio é o quadro de sempre **ao bit**.
    //
    // ⛔⛔ **SEIS e não três, e a diferença é o quadro de MOVIMENTO:** o pintor lê sempre os slots
    // LISOS, e quem os escreve é a `borra_ricochete`, que só é despachada com `ao_rays > 0`. Com
    // três, um quadro de movimento lia slots **nunca escritos** e ficava a depender de o buffer
    // nascer a zero — *uma propriedade do driver a segurar uma lei do produto*.
    for (var c: u32 = 0u; c < 6u; c = c + 1u) { luz[base_do_ricochete(i) + c] = 0.0; }

    // ⭐⭐⭐ **A OCLUSÃO POR CONES** — `ao_rays` direcções FIXAS de mundo, pesadas pelo cosseno.
    //
    // A dureza de cada cone é `1/(n·d)`: é o cone que ROÇA o plano tangente, e é ele que faz um
    // corpo CONVEXO ler exactamente `1,0`. Ver `ph2d_field_render::cone_dir` para o porquê de o
    // conjunto ser de MUNDO e não de um referencial tangente.
    var ceu = 1.0;
    if (s.ao_rays > 0u) {
        var soma = 0.0;
        var peso = 0.0;
        for (var j: u32 = 0u; j < s.ao_rays; j = j + 1u) {
            let dd = direccao_do_cone(j, s.ao_rays);
            let c = dot(n, dd);
            if (c <= 0.0) { continue; }
            peso = peso + c;
            let ate = min(s.ao_reach, cerca_da_bola(erguido, dd, s.ao_reach));
            soma = soma + c * visivel_ceu(erguido, dd, ate, 1.0 / c);
        }
        if (peso > 0.0) { ceu = soma / peso; }
    }
    luz[base] = ceu;
}

// ⭐⭐⭐ **A SEGUNDA PASSAGEM: a borda re-amostrada.** Ela precisa dos VIZINHOS, logo não pode
// viver na primeira — e é por isso que são dois despachos e não um.
@compute @workgroup_size(8, 8, 1)
fn bordas(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let c = centro[i];
    // A MESMA regra da CPU: direita e baixo, e os DOIS pixels ficam marcados.
    var e = false;
    if (g.x + 1u < s.w) { e = e || difere(i, i + 1u); }
    if (g.y + 1u < s.h) { e = e || difere(i, i + s.w); }
    if (g.x > 0u) { e = e || difere(i - 1u, i); }
    if (g.y > 0u) { e = e || difere(i - s.w, i); }
    if (!e) { return; }

    let slot = atomicAdd(&conta, 1u);
    // ⚠️ Um lote cheio **descarta** em vez de escrever fora — a borda perde-se, o quadro não.
    if (slot * 5u + 4u >= arrayLength(&borda)) { return; }
    borda[slot * 5u] = vec4<f32>(bitcast<f32>(i), 0.0, 0.0, 0.0);
}

// ⭐⭐⭐⭐ **A RE-AMOSTRAGEM COMPACTA: uma thread por (borda, sub-amostra)** (`docs/Render3d/03`
// §W9, «a borda que esperava pelas vizinhas»). ⚠️ Marchar as quatro sub-amostras DENTRO do
// `bordas` punha um punhado de pixels de borda a marchar quatro vezes em série enquanto o resto
// do warp esperava — medido no nó a `1920×1080`, a passagem custava `7,38` dos `11,86 ms` do
// quadro. Aqui a lista já está escrita, e threads vizinhas marcham a MESMA borda.
//
// ⚠️ O despacho é a IMAGEM inteira e o laço anda em passos do tamanho dela: a contagem mora no
// dispositivo, e uma lista maior do que `w × h / 4` (imagens minúsculas) continua coberta.
@compute @workgroup_size(8, 8, 1)
fn bordas_marcha(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let cabem = arrayLength(&borda) / 5u;
    let total = min(atomicLoad(&conta), cabem) * 4u;
    let passo = s.w * s.h;
    // O padrão 4-rook (RGSS), o mesmo da CPU.
    let rook = array<vec2<f32>, 4>(
        vec2<f32>(0.125, 0.625), vec2<f32>(0.375, 0.125),
        vec2<f32>(0.625, 0.875), vec2<f32>(0.875, 0.375));
    for (var k = g.y * s.w + g.x; k < total; k = k + passo) {
        let slot = k / 4u;
        let j = k % 4u;
        let i = bitcast<u32>(borda[slot * 5u].x);
        let px = f32(i % s.w);
        let py = f32(i / s.w);
        let o = rook[j];
        borda[slot * 5u + 1u + j] = marcha(ray_at_plane(raio(px + o.x, py + o.y)));
    }
}

fn difere(a: u32, b: u32) -> bool {
    let ca = centro[a];
    let cb = centro[b];
    let ha = ca.x >= 0.0;
    let hb = cb.x >= 0.0;
    if (ha != hb) { return true; }
    if (!ha) { return false; }
    return dot(ca.yzw, cb.yzw) < s.edge_cos;
}";

/// ⭐ **O molde do traçado, inteiro** — [`COMUM`] mais as [`leis`] mais os [`KERNELS`].
///
/// ⚠️ Ele é uma função e não uma constante porque o `concat!` só junta LITERAIS. O custo é uma
/// alocação por quadro, ao lado do `replace` do `{FIELD}` que o cache de pipelines já faz.
pub(crate) fn molde() -> String {
    format!("{}{}{KERNELS}", comum(), leis())
}

/// ⭐⭐⭐ **AS LEIS DA MARCHA, sem os kernels** — o campo, a marcha, a visibilidade, o conjunto de
/// cones e as cercas.
///
/// ⚠️ **Ela é `pub(crate)` porque o passe que PINTA passou a precisar delas** (`docs/Render3d/08`
/// §12): o ricochete marcha a partir da superfície, e marchar é isto. *O pintor recebe as leis e
/// não os kernels — declarar outra vez as duas entradas que escrevem no `centro` e na `luz` daria
/// um módulo com pontos de entrada que ninguém despacha.*
pub(crate) fn leis() -> String {
    // ⚠️ **As constantes do chão são LIDAS do ficheiro que as declara** — transcritas aqui, elas
    // divergiriam no dia em que a varredura que as ajustou fosse refeita.
    LEIS.replace("{TRILINEAR}", crate::sculpt::TRILINEAR)
        .replace("{LONGE}", crate::longe::LEI)
        .replace("{SALTOS_MAX}", &crate::longe::SALTOS_MAX.to_string())
        .replace(
            "{CHAO_N}",
            &ph2d_field_render::GROUND_SKY_SAMPLES.to_string(),
        )
        .replace(
            "{CHAO_ESPALHA}",
            &numero(ph2d_field_render::GROUND_SKY_SPREAD),
        )
        .replace(
            "{CHAO_QUEDA}",
            &numero(ph2d_field_render::GROUND_SKY_FALLOFF),
        )
        .replace(
            "{CHAO_FORCA}",
            &numero(ph2d_field_render::GROUND_SKY_STRENGTH),
        )
}

/// Um `f32` que o WGSL leia como `f32` — o irmão do `paint::formata`, e pela mesma razão.
fn numero(v: f32) -> String {
    let s = format!("{v:?}");
    if s.contains('.') || s.contains('e') {
        s
    } else {
        format!("{s}.0")
    }
}

/// ⭐ **O [`COMUM`] com o tecto de lâmpadas preenchido.**
///
/// ⚠️ **O `8` do `array<vec4, N>` é o [`crate::trace::MAX_LAMPS`]**, e não um literal ao lado dele:
/// escrito duas vezes, um dos dois envelhece na wave que mexer no outro — e o sintoma seria o
/// uniforme a ler lixo a partir da lâmpada `N+1`, sem erro nenhum.
pub(crate) fn comum() -> String {
    COMUM.replace("{MAX_LAMPS}", &crate::trace::MAX_LAMPS.to_string())
}
