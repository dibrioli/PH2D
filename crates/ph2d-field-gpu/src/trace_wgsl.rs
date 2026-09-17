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
    n_lamps: u32, chao: u32, _p1: u32, _p2: u32,
    half_extent: f32, half_px: f32, ortho_start: f32, eye_distance: f32,
    hit_eps: f32, normal_eps: f32, step: f32, t_max: f32,
    ball_radius: f32, ao_reach: f32, edge_cos: f32, chao_y: f32,
    alvo: vec3<f32>, right: vec3<f32>, up: vec3<f32>, fwd: vec3<f32>,
    ball_center: vec3<f32>,
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

/// O passo de [`luz`] — o céu mais uma visibilidade por lâmpada.
fn passo_da_luz() -> u32 { return 1u + s.n_lamps; }

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
pub(crate) const MARCHA: &str = r"
{ESCULTURAS}
{FIELD}

// Devolve `vec4(t, normal em VISTA)`, com `t < 0` quando não acerta.
fn marcha(r: Raio) -> vec4<f32> {
    var t = 0.0;
    var acertou = false;
    for (var n: u32 = 0u; n < s.budget; n = n + 1u) {
        let d = field(r.o + r.d * t);
        if (d < s.hit_eps) { acertou = true; break; }
        t = t + d * s.step;
        if (t >= s.t_max) { break; }
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

@compute @workgroup_size(8, 8, 1)
fn centro_e_luz(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    let r = ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5));
    let c = marcha(r);
    centro[i] = c;
    let base = i * passo_da_luz();
    if (c.x < 0.0) {
        // ⚠️ Um pixel que não acerta recebe **luz inteira** em todos os canais — é o que a CPU
        // devolve (`vis` nasce a `1.0` e o laço salta quem não acerta).
        for (var l: u32 = 0u; l < passo_da_luz(); l = l + 1u) { luz[base + l] = 1.0; }
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
            soma = soma + c * visivel(erguido, dd, ate, 1.0 / c);
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
    // O padrão 4-rook (RGSS), o mesmo da CPU.
    let rook = array<vec2<f32>, 4>(
        vec2<f32>(0.125, 0.625), vec2<f32>(0.375, 0.125),
        vec2<f32>(0.625, 0.875), vec2<f32>(0.875, 0.375));
    for (var j = 0u; j < 4u; j = j + 1u) {
        let o = rook[j];
        borda[slot * 5u + 1u + j] = marcha(ray_at_plane(raio(f32(g.x) + o.x, f32(g.y) + o.y)));
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

/// ⭐ **O molde do traçado, inteiro** — [`COMUM`] mais [`MARCHA`], na ordem em que o WGSL os lê.
///
/// ⚠️ Ele é uma função e não uma constante porque o `concat!` só junta LITERAIS. O custo é uma
/// alocação por quadro, ao lado do `replace` do `{FIELD}` que o cache de pipelines já faz.
pub(crate) fn molde() -> String {
    // ⚠️ **As constantes do chão são LIDAS do ficheiro que as declara** — transcritas aqui, elas
    // divergiriam no dia em que a varredura que as ajustou fosse refeita.
    let marcha = MARCHA
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
        );
    format!("{}{marcha}", comum())
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
