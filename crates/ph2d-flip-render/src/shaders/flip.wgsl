// Flip — rasterização de traço (ADR-0113, W1), clean-room do Grease Pencil 5.2
// (`draw_grease_pencil_lib.glsl`), adaptado ao 2D-ortográfico do PH2D: a matemática
// 3D do GP COLAPSA — sem perspectiva, `thickness_px = raio·zoom`, e o plano do
// traço É o plano da tela.
//
// **Cobertura ANALÍTICA (como o GP).** Cada segmento vira um quad que só precisa
// COBRIR a fita (retângulo + tampas); a forma EXATA sai no fragment, da **distância
// do pixel à linha-de-centro** (`gpencil_stroke_segment_mask`). Clampar o parâmetro
// ao segmento dá **junções e tampas REDONDAS de graça** — sem miter, sem spikes, e
// (crucial) sem double-blend nas quinas: dois quads adjacentes calculam a MESMA
// distância à mesma reta, então a cobertura é consistente e o depth só escolhe um.
// Era o bug das quinas/curvas com hardness baixo (a coordenada `v_perp` por-quad
// distorcia nas junções). O perfil de hardness é o do GP (`pow` + smoothstep), com
// AA de ~1px por `fwidth` (o GP conta com MSAA; aqui não há).

struct Camera {
    world_to_clip: mat4x4<f32>,
    viewport: vec2<f32>,   // px
    px_per_world: f32,     // escala de espessura (1 = tela absoluta; object scale quando escalado)
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
const FLAG_START_FLAT: u32 = 2u;
const FLAG_END_FLAT: u32 = 4u;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    // Extremos do segmento em SCREEN-SPACE (bottom-up, como `to_screen`), CONSTANTES
    // no quad → o fragment mede a distância do pixel a esta reta.
    @location(0) @interpolate(flat) ss_p1: vec2<f32>,
    @location(1) @interpolate(flat) ss_p2: vec2<f32>,
    @location(2) color: vec4<f32>,   // cor por-ponto interpolada
    @location(3) opacity: f32,
    @location(4) thickness: f32,      // espessura (px de tela) interpolada a→b
    @location(5) @interpolate(flat) hardness: f32,
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

    // Segmento a=gp → b=próximo. A cauda de um traço ABERTO degenera (clipada).
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

    let a = points[gp];
    let b = points[next_gp];
    let sa = to_screen(a.pos);
    let sb = to_screen(b.pos);
    let r_a = max(a.width * 0.5 * cam.px_per_world, 0.0);
    let r_b = max(b.width * 0.5 * cam.px_per_world, 0.0);

    let seg = sb - sa;
    let seg_len = max(length(seg), 1e-6);
    let dir = seg / seg_len;
    let n_seg = perp(dir);

    // Geometria = quad-STADIUM por segmento (retângulo perpendicular estendido `r` ao
    // longo da reta em cada ponta, exceto tampa Flat). Convexo — NUNCA dobra sobre si
    // (o "spike/estrela" nas quinas era o miter DOBRANDO a fita = triângulo invertido,
    // a suspeita do Enio sobre "normais da face"). Segmentos adjacentes se sobrepõem,
    // mas o depth GREATER estrito + write-depth descarta a 2ª face no mesmo pixel (não
    // acumula). O fragment (distância à reta) carrega a junção/tampa REDONDA.
    let is_start = (li == 0u) && !closed;
    let is_end = (next_gp == last) && !closed;
    let flat_start = is_start && ((st.flags & FLAG_START_FLAT) != 0u);
    let flat_end = is_end && ((st.flags & FLAG_END_FLAT) != 0u);
    let ext_a = select(r_a, 0.0, flat_start);
    let ext_b = select(r_b, 0.0, flat_end);

    // Quad = 2 triângulos [0,1,2, 2,1,3]; idx 0=(a,esq) 1=(a,dir) 2=(b,esq) 3=(b,dir).
    var idx = ci;
    if (ci == 3u) { idx = 2u; }
    else if (ci == 4u) { idx = 1u; }
    else if (ci == 5u) { idx = 3u; }
    let at_b = idx >= 2u;
    let right = (idx == 1u) || (idx == 3u);
    let side = select(1.0, -1.0, right);

    var corner: vec2<f32>;
    if (at_b) {
        corner = sb + dir * ext_b + n_seg * (side * r_b);
        out.color = b.color;
        out.opacity = b.opacity;
        out.thickness = 2.0 * r_b;
    } else {
        corner = sa - dir * ext_a + n_seg * (side * r_a);
        out.color = a.color;
        out.opacity = a.opacity;
        out.thickness = 2.0 * r_a;
    }

    // Ordem 2D (GP 2D): profundidade por-traço = (2·sid+2)·2e-7, teste GREATER estrito
    // + write-depth. O sid maior compõe por cima; no MESMO depth (uma face do próprio
    // traço sobre outra — quina/cruzamento) o 2º fragmento é DESCARTADO, não misturado
    // → sem acúmulo de cor. Fill do traço em (2·sid+1), abaixo.
    out.clip = to_clip(corner);
    out.clip.z = f32(2u * sid + 2u) * 2e-7;
    out.ss_p1 = sa;
    out.ss_p2 = sb;
    out.hardness = st.hardness;
    return out;
}

// Perfil redondo do Grease Pencil (`gpencil_stroke_hardess_mask`): núcleo cheio até
// `hardness`, queda `pow`+smoothstep até a borda. `dn` = distância normalizada à
// linha-de-centro (0 centro, 1 borda). `aa` (~1px) fecha a borda com AA — o GP usa
// MSAA; aqui a cobertura é analítica, então o AA sai do `fwidth`.
fn hardness_mask(dn: f32, hardness: f32, aa: f32) -> f32 {
    let inv = clamp(1.0 - dn, 0.0, 1.0); // 1 no centro, 0 na borda
    var profile: f32;
    if (hardness > 0.999) {
        profile = 1.0; // borda dura: o núcleo é chapado; a borda é o AA abaixo
    } else {
        let soft = 1.0 - hardness;
        profile = smoothstep(0.0, 1.0, pow(inv, mix(0.0, 10.0, soft)));
    }
    let edge = 1.0 - smoothstep(1.0 - aa, 1.0, dn); // AA na borda do raio (dn=1)
    return profile * edge;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Pixel em SCREEN-SPACE bottom-up (o `@builtin(position)` é top-down → flip Y
    // p/ casar `to_screen`).
    let frag = vec2<f32>(in.clip.x, cam.viewport.y - in.clip.y);
    let pos1 = frag - in.ss_p1;
    let line1 = in.ss_p2 - in.ss_p1;
    let len_sq = max(dot(line1, line1), 1e-6);
    // Distância à linha-de-centro CLAMPADA ao segmento → junções/tampas redondas.
    let t1 = clamp(dot(pos1, line1) / len_sq, 0.0, 1.0);
    let dist = length(pos1 - t1 * line1); // px de tela
    let radius = max(in.thickness * 0.5, 1e-4);
    let dn = dist / radius; // 0 centro, 1 borda
    let aa = max(fwidth(dn), 1e-4);
    let mask = hardness_mask(dn, in.hardness, aa);
    let alpha = in.color.a * in.opacity * mask;
    // Saída PREMULTIPLICADA (blend = One, OneMinusSrcAlpha).
    return vec4<f32>(in.color.rgb * alpha, alpha);
}
