//! ⭐⭐⭐ **O WGSL DO TRAÇADO** — o texto dos shaders, separado de quem os despacha.
//!
//! # ⚠️ Porque ele é DOIS pedaços e não um
//!
//! O passe que **pinta** o Matcap ([`crate::matcap`]) não marcha nada: ele lê o que a marcha
//! escreveu. Mas precisa das MESMAS três coisas — a `struct Setup`, os bindings do grupo `0` e a
//! reconstrução do raio de cada pixel. ⇒ o [`COMUM`] é o que os dois leem e as [`LEIS`] e os
//! [`KERNELS`] são o que só a marcha tem. ⛔ Uma segunda cópia do `ray_at_plane` seria a segunda
//! resposta a *«que raio sai daqui?»*.
//!
//! ⚠️ A luz, o céu, o chão e a borda mole do Render traçado viviam aqui e saíram em 03/10 com ele:
//! o modo Render desenha por malha (`ph2d-mesh-forward`).

/// A parte que a marcha e o pintor de Matcap partilham: o uniforme, os bindings do grupo `0` e o raio.
pub(crate) const COMUM: &str = r"
struct Setup {
    // ⭐⭐⭐ `longe`: o índice do cabeçalho da GRADE DE LONGE no `k`, MAIS UM — `0` é a marcha de
    // sempre, ao bit (`crate::longe`).
    w: u32, h: u32, budget: u32, longe: u32,
    half_extent: f32, half_px: f32, ortho_start: f32, eye_distance: f32,
    hit_eps: f32, normal_eps: f32, step: f32, t_max: f32,
    // ⚠️ O `edge_cos` mora no enchimento do `vec3` de cima: um `f32` a seguir a um `vec3` ocupa os
    // quatro bytes que sobram.
    alvo: vec3<f32>, edge_cos: f32,
    right: vec3<f32>, up: vec3<f32>, fwd: vec3<f32>,
};
@group(0) @binding(0) var<uniform> s: Setup;
@group(0) @binding(1) var<storage, read> k: array<f32>;
@group(0) @binding(2) var<storage, read_write> centro: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> conta: atomic<u32>;
@group(0) @binding(5) var<storage, read_write> borda: array<vec4<f32>>;
// ⭐⭐⭐ **AS GRADES DAS ESCULTURAS, concatenadas** — o cabeçalho de cada uma vive no `k` e diz onde
// ela começa aqui. ⚠️ Ela sobe UMA VEZ e fica: `128³` são `8 MB`, e reenviá-la por quadro custaria
// mais barramento do que a imagem inteira que este passe veio poupar.
@group(0) @binding(6) var<storage, read> grades: array<f32>;

// ⚠️ **A MESMA lei de marcha da CPU**, e ela é uma função porque as quatro amostras do
// anti-serrilhado a repetem: uma segunda cópia seria a segunda resposta à mesma pergunta.
fn raio(px: f32, py: f32) -> vec2<f32> {
    let u = (px - f32(s.w) * 0.5) / s.half_px * s.half_extent;
    let v = -(py - f32(s.h) * 0.5) / s.half_px * s.half_extent;
    return vec2<f32>(u, v);
}
struct Raio { o: vec3<f32>, d: vec3<f32> };
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

/// O que só a marcha tem: a fita da peça e a marcha.
pub(crate) const LEIS: &str = r"
{TRILINEAR}
{LONGE}
{ESCULTURAS}
{FIELD}

// Devolve `vec4(t, normal em VISTA)`, com `t < 0` quando não acerta.
fn marcha(r: Raio) -> vec4<f32> {
    var t = 0.0;
    var fim = s.t_max;
    var acertou = false;
    // ⭐⭐⭐ **A GRADE DE LONGE** (`crate::longe`) — com `s.longe = 0` nada disto corre e a marcha
    // é a de sempre, ao bit.
    //
    // ⚠️ **O recorte primeiro:** um raio que não toca a caixa da peça não tem superfície a achar.
    if (s.longe != 0u) {
        let c = longe_caixa(r);
        if (c.x > c.y || c.y <= 0.0) { return vec4<f32>(-1.0, 0.0, 0.0, 0.0); }
        t = max(c.x, 0.0);
        fim = min(s.t_max, c.y);
    }
    // ⚠️ **O orçamento conta só as avaliações da ÁRVORE** — é ela que o `budget` foi medido a
    // pagar. Os saltos têm tecto próprio, e cada um anda pelo menos `longe_perto()`.
    var n: u32 = 0u;
    var saltos: u32 = 0u;
    // ⚠️ **Decidido UMA vez por raio, fora do laço:** com o recorte sem grade (`res = 0`) a
    // pergunta à grade devolve sempre `0`, e fazê-la em todo passo custava uma leitura do `k` e um
    // ramo por passo — medido na cena `=1` (campo barato), `9,47 → 10,96 ms` só por isso.
    let com_grade = s.longe != 0u && longe_tem_grade();
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
";

/// ⭐⭐⭐ **OS KERNELS DA MARCHA** — o centro e as duas passagens da borda. Eles vivem à parte das
/// [`LEIS`] porque o pintor de Matcap só precisa do [`COMUM`], e um ponto de entrada num módulo que
/// ninguém despacha é código que não se apaga porque compila.
pub(crate) const KERNELS: &str = r"
// ⭐⭐⭐⭐ **O CENTRO** — a marcha e a normal de cada pixel; é tudo o que o MATCAP lê.
@compute @workgroup_size(8, 8, 1)
fn centro_so(@builtin(global_invocation_id) g: vec3<u32>) {
    if (g.x >= s.w || g.y >= s.h) { return; }
    let i = g.y * s.w + g.x;
    centro[i] = marcha(ray_at_plane(raio(f32(g.x) + 0.5, f32(g.y) + 0.5)));
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
    format!("{COMUM}{}{KERNELS}", leis())
}

/// ⭐⭐⭐ **AS LEIS DA MARCHA, sem os kernels** — o campo (com as esculturas e a grade de longe) e
/// a marcha.
pub(crate) fn leis() -> String {
    LEIS.replace("{TRILINEAR}", crate::sculpt::TRILINEAR)
        .replace("{LONGE}", crate::longe::LEI)
        .replace("{SALTOS_MAX}", &crate::longe::SALTOS_MAX.to_string())
}

/// O [`COMUM`] — uma função para os chamadores que o compõem com outros textos.
pub(crate) fn comum() -> String {
    COMUM.to_string()
}
