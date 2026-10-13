//! Os três passes pequenos das células, em WGSL.

use ph2d_render::RenderInstance;

/// A palavra (`u32`) onde um campo da [`RenderInstance`] começa — lida do `repr(C)`, nunca escrita
/// à mão (o stride já mudou de 144 para 188 B sem o comentário acompanhar).
const fn palavra(bytes: usize) -> usize {
    bytes / 4
}

/// **As caixas das SPRITES no ecrã** — a pose do `sprite.wgsl` (`mundo = pos + basis·(anchor +
/// q·size)`, `q ∈ [−½, ½]²`) pelo afim mundo → pixel da camada. Dois `vec4` por instância, como o
/// `cs_caixas` das formas. Gate de paridade: `the_sprite_box_holds_every_pixel_it_draws`. ⭐ O quad de
/// tamanho ZERO (a linha calada) não cobre pixel nenhum: caixa vazia (`lo > hi`), fora do plano —
/// gate `uma_linha_calada_nao_e_celula`.
pub(crate) fn limites_wgsl() -> String {
    format!(
        r"
const STRIDE: u32 = {stride}u;
const WP: u32 = {wp}u;
const SZ: u32 = {sz}u;
const BA: u32 = {ba}u;
const AN: u32 = {an}u;
struct P {{ lin: vec4<f32>, t: vec2<f32>, n: u32, calada_vazia: u32 }};
@group(0) @binding(0) var<uniform> p: P;
@group(0) @binding(1) var<storage, read> inst: array<u32>;
@group(0) @binding(2) var<storage, read_write> caixas: array<vec4<f32>>;

fn f(i: u32) -> f32 {{ return bitcast<f32>(inst[i]); }}
fn px(w: vec2<f32>) -> vec2<f32> {{
    return vec2<f32>(p.lin.x * w.x + p.lin.z * w.y, p.lin.y * w.x + p.lin.w * w.y) + p.t;
}}

@compute @workgroup_size(64)
fn cs(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if i >= p.n {{ return; }}
    let b = i * STRIDE;
    let pos = vec2<f32>(f(b + WP), f(b + WP + 1u));
    let size = vec2<f32>(f(b + SZ), f(b + SZ + 1u));
    let ba = vec4<f32>(f(b + BA), f(b + BA + 1u), f(b + BA + 2u), f(b + BA + 3u));
    let an = vec2<f32>(f(b + AN), f(b + AN + 1u));
    if p.calada_vazia != 0u && size.x == 0.0 && size.y == 0.0 {{
        caixas[2u * i] = vec4<f32>(1.0e30, 1.0e30, -1.0e30, -1.0e30);
        caixas[2u * i + 1u] = vec4<f32>(px(pos), 0.0, 0.0);
        return;
    }}
    var lo = vec2<f32>(1.0e30);
    var hi = vec2<f32>(-1.0e30);
    for (var k = 0u; k < 4u; k = k + 1u) {{
        let q = vec2<f32>(f32(k & 1u) - 0.5, f32(k >> 1u) - 0.5);
        let l = an + q * size;
        let w = pos + vec2<f32>(ba.x * l.x + ba.z * l.y, ba.y * l.x + ba.w * l.y);
        let s = px(w);
        lo = min(lo, s);
        hi = max(hi, s);
    }}
    caixas[2u * i] = vec4<f32>(lo, hi);
    caixas[2u * i + 1u] = vec4<f32>(px(pos), 0.0, 0.0);
}}
",
        stride = palavra(std::mem::size_of::<RenderInstance>()),
        wp = palavra(std::mem::offset_of!(RenderInstance, world_pos)),
        sz = palavra(std::mem::offset_of!(RenderInstance, size)),
        ba = palavra(std::mem::offset_of!(RenderInstance, basis)),
        an = palavra(std::mem::offset_of!(RenderInstance, anchor)),
    )
}

/// **A relocação**: a instância `idx[j]` copiada para o lugar `j`, com a posição (`pos` palavras a
/// partir do início) somada de `delta[j]` — a translação no MUNDO que leva a instância à célula.
pub(crate) const RELOCAR_WGSL: &str = r"
struct P { n: u32, stride: u32, pos: u32, _p: u32 };
@group(0) @binding(0) var<uniform> p: P;
@group(0) @binding(1) var<storage, read> origem: array<u32>;
@group(0) @binding(2) var<storage, read> idx: array<u32>;
@group(0) @binding(3) var<storage, read> delta: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read_write> destino: array<u32>;

@compute @workgroup_size(64)
fn cs(@builtin(global_invocation_id) id: vec3<u32>) {
    let j = id.x;
    if j >= p.n { return; }
    let s = idx[j] * p.stride;
    let d = j * p.stride;
    for (var k = 0u; k < p.stride; k = k + 1u) {
        destino[d + k] = origem[s + k];
    }
    let x = bitcast<f32>(origem[s + p.pos]) + delta[j].x;
    let y = bitcast<f32>(origem[s + p.pos + 1u]) + delta[j].y;
    destino[d + p.pos] = bitcast<u32>(x);
    destino[d + p.pos + 1u] = bitcast<u32>(y);
}
";

/// **A volta e o mapa da origem.** Um quad por célula, na ordem: o `fs_volta` lê o átlas filtrado
/// (sRGB RETO, a saída da pilha) no texel `p + t` e devolve-o PRÉ-multiplicado sobre a camada; o
/// `fs_mapa` pinta a célula, no ÁTLAS, com a âncora da instância (a origem do ruído por texel).
pub(crate) const COMPOE_WGSL: &str = r"
struct Cel { saida: vec4<i32>, t: vec2<i32>, org: vec2<f32> };
@group(0) @binding(0) var<storage, read> cels: array<Cel>;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var<uniform> alvo: vec4<f32>;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) t: vec2<i32>,
    @location(1) @interpolate(flat) org: vec2<f32>,
};

fn quad(vi: u32, r: vec4<f32>) -> vec4<f32> {
    let c = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    let q = mix(r.xy, r.zw, c[vi]);
    return vec4<f32>(q.x / alvo.x * 2.0 - 1.0, 1.0 - q.y / alvo.y * 2.0, 0.0, 1.0);
}

@vertex
fn vs_volta(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    let c = cels[ii];
    var o: VsOut;
    o.pos = quad(vi, vec4<f32>(c.saida));
    o.t = c.t;
    o.org = c.org;
    return o;
}

@vertex
fn vs_mapa(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    let c = cels[ii];
    let t = vec4<f32>(vec2<f32>(c.t), vec2<f32>(c.t));
    var o: VsOut;
    o.pos = quad(vi, vec4<f32>(c.saida) + t);
    o.t = c.t;
    o.org = c.org;
    return o;
}

fn volta_cor(i: VsOut) -> vec4<f32> {
    let p = vec2<i32>(floor(i.pos.xy)) + i.t;
    let s = textureLoad(atlas, p, 0);
    return vec4<f32>(s.rgb * s.a, s.a);
}

@fragment
fn fs_volta(i: VsOut) -> @location(0) vec4<f32> {
    return volta_cor(i);
}

@fragment
fn fs_mapa(i: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(i.org, 0.0, 1.0);
}
";

/// B5 — a volta com o MULTIPLY exacto (`ph2d_shape_gpu::Mistura`): a 2.ª saída é o factor do que está
/// por baixo. Só num módulo com `enable dual_source_blending;` à frente do [`COMPOE_WGSL`].
pub(crate) const VOLTA_MULTIPLICA_WGSL: &str = r"
struct DuasSaidas {
    @location(0) @blend_src(0) cor: vec4<f32>,
    @location(0) @blend_src(1) factor: vec4<f32>,
};

@fragment
fn fs_volta_multiplica(i: VsOut) -> DuasSaidas {
    let c = volta_cor(i);
    return DuasSaidas(c, vec4<f32>(vec3<f32>(1.0 - c.a) + c.rgb, 1.0));
}
";
