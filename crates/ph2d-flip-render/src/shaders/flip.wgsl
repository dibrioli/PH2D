// Flip — rasterização de traço (ADR-0113, W1 T1.2/T1.3), clean-room do Grease
// Pencil 5.2 (draw_grease_pencil_lib.glsl), adaptado ao 2D-ortográfico do PH2D:
// a matemática 3D do GP COLAPSA — sem perspectiva, thickness_px = raio·zoom, e o
// plano do traço É o plano da tela.
//
// Um draw não-instanciado de `total_points * 6` vértices: cada ponto vira um quad
// (2 triângulos) para o segmento ponto→próximo. O vertex shader expande o segmento
// numa fita em SCREEN-SPACE com junção por miter (clampado). O fragment aplica a
// máscara de hardness (pow/smoothstep) + AA por fwidth.

struct Camera {
    world_to_clip: mat4x4<f32>,
    viewport: vec2<f32>,   // px
    px_per_world: f32,     // pixels por unidade de mundo (zoom da Camera2d)
    _pad: f32,
}

struct GpuPoint {
    pos: vec2<f32>,
    width: f32,
    opacity: f32,
    color: vec4<f32>,
}

struct GpuStroke {
    first_point: u32,
    point_count: u32,
    flags: u32,
    hardness: f32,
    material: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0) var<uniform> cam: Camera;
@group(0) @binding(1) var<storage, read> points: array<GpuPoint>;
@group(0) @binding(2) var<storage, read> strokes: array<GpuStroke>;
@group(0) @binding(3) var<storage, read> point_stroke: array<u32>;

const FLAG_CLOSED: u32 = 1u;
// Miter clampado (evita spikes em quinas afiadas). O GP quebra pra bevel; aqui o
// clamp do comprimento é a aproximação de v1.
const MITER_LIMIT: f32 = 4.0;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) v_perp: f32,       // -1..+1 através da fita (0 = eixo)
    @location(1) color: vec4<f32>,  // cor por-ponto interpolada
    @location(2) opacity: f32,
    @location(3) hardness: f32,
}

fn to_screen(world: vec2<f32>) -> vec2<f32> {
    let clip = cam.world_to_clip * vec4<f32>(world, 0.0, 1.0);
    let ndc = clip.xy / clip.w;
    return (ndc * 0.5 + vec2<f32>(0.5)) * cam.viewport;
}

fn to_clip(screen: vec2<f32>) -> vec4<f32> {
    let ndc = screen / cam.viewport * 2.0 - vec2<f32>(1.0);
    return vec4<f32>(ndc, 0.0, 1.0);
}

fn perp(v: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(-v.y, v.x);
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let gp = vi / 6u;
    let ci = vi % 6u;
    let sid = point_stroke[gp];
    let st = strokes[sid];
    let first = st.first_point;
    let count = st.point_count;
    let last = first + count - 1u;
    let closed = (st.flags & FLAG_CLOSED) != 0u;
    let li = gp - first;

    // Segmento a=gp → b=próximo. A cauda de um traço ABERTO não tem segmento
    // adiante → degenera (fora do NDC, clipado).
    var next_gp = gp;
    var degenerate = false;
    if (li + 1u < count) {
        next_gp = gp + 1u;
    } else if (closed) {
        next_gp = first;
    } else {
        degenerate = true;
    }
    if (degenerate) {
        out.clip = vec4<f32>(2.0, 2.0, 2.0, 1.0);
        return out;
    }

    // Predecessor de a (miter em a) e sucessor de b (miter em b), com clamp
    // aberto / wrap fechado — nunca cruza a fronteira do traço.
    var prev_gp = gp;
    if (li > 0u) {
        prev_gp = gp - 1u;
    } else if (closed) {
        prev_gp = last;
    }
    let b_li = next_gp - first;
    var nn_gp = next_gp;
    if (b_li + 1u < count) {
        nn_gp = next_gp + 1u;
    } else if (closed) {
        nn_gp = first;
    }

    let a = points[gp];
    let b = points[next_gp];
    let sa = to_screen(a.pos);
    let sb = to_screen(b.pos);
    let sp = to_screen(points[prev_gp].pos);
    let sn = to_screen(points[nn_gp].pos);

    let seg = sb - sa;
    let seg_len = max(length(seg), 1e-6);
    let d_seg = seg / seg_len;
    let n_seg = perp(d_seg);

    // Miter em a.
    var miter_a = n_seg;
    var scale_a = 1.0;
    if (prev_gp != gp) {
        let d_prev = normalize(sa - sp);
        let tang = normalize(d_prev + d_seg);
        let m = perp(tang);
        miter_a = m;
        scale_a = 1.0 / max(dot(m, n_seg), 1.0 / MITER_LIMIT);
    }
    // Miter em b.
    var miter_b = n_seg;
    var scale_b = 1.0;
    if (nn_gp != next_gp) {
        let d_next = normalize(sn - sb);
        let tang = normalize(d_seg + d_next);
        let m = perp(tang);
        miter_b = m;
        scale_b = 1.0 / max(dot(m, n_seg), 1.0 / MITER_LIMIT);
    }

    let half_a = a.width * 0.5 * cam.px_per_world;
    let half_b = b.width * 0.5 * cam.px_per_world;

    // Mapeamento do canto: quad = 2 triângulos [0,1,2, 2,1,3].
    // idx 0=(a,esq) 1=(a,dir) 2=(b,esq) 3=(b,dir).
    var idx = ci;
    if (ci == 3u) { idx = 2u; }
    else if (ci == 4u) { idx = 1u; }
    else if (ci == 5u) { idx = 3u; }
    let at_b = idx >= 2u;
    let right = (idx == 1u) || (idx == 3u);
    let side = select(1.0, -1.0, right); // esq=+1, dir=-1

    var center = sa;
    var miter = miter_a;
    var scale = scale_a;
    var half = half_a;
    out.color = a.color;
    out.opacity = a.opacity;
    if (at_b) {
        center = sb;
        miter = miter_b;
        scale = scale_b;
        half = half_b;
        out.color = b.color;
        out.opacity = b.opacity;
    }

    let screen_pos = center + side * miter * (half * scale);
    // Ordem 2D (GP §2): profundidade por traço = (2·sid+2)·2e-7, teste GREATER —
    // o traço mais novo (sid maior) ganha, e o auto-overlap de UM traço
    // (junções/miter) não recompõe (mesma profundidade → o 2º fragmento falha o
    // GREATER, sem double-blend). O fill do mesmo traço fica em (2·sid+1), logo
    // abaixo → o traço ganha sobre o próprio fill.
    let c = to_clip(screen_pos);
    let depth = f32(2u * sid + 2u) * 2e-7;
    out.clip = vec4<f32>(c.xy, depth, 1.0);
    out.v_perp = side;
    out.hardness = st.hardness;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dn = abs(in.v_perp);                 // 0 no eixo, 1 na borda
    // Falloff de hardness (GP): d = clamp(1-dn,0,1); pow(d, mix(0,10, 1-hard)).
    let d = clamp(1.0 - dn, 0.0, 1.0);
    let softness = 1.0 - in.hardness;
    let shaped = smoothstep(0.0, 1.0, pow(d, mix(0.0, 10.0, softness)));
    // AA de borda (~meio pixel em unidades normalizadas), robusto p/ borda dura.
    let aa = max(fwidth(dn), 1e-4);
    let cov = 1.0 - smoothstep(1.0 - aa, 1.0, dn);
    let alpha = in.color.a * in.opacity * shaped * cov;
    // Saída PREMULTIPLICADA (blend = One, OneMinusSrcAlpha).
    return vec4<f32>(in.color.rgb * alpha, alpha);
}
