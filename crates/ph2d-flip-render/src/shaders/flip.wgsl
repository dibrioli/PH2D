// Flip — rasterização de traço (ADR-0113, W1), clean-room do Grease Pencil 5.2
// (`draw_grease_pencil_lib.glsl`), adaptado ao 2D-ortográfico do PH2D: a matemática
// 3D do GP COLAPSA — sem perspectiva, `thickness_px = raio·zoom`, e o plano do
// traço É o plano da tela.
//
// **Cobertura ANALÍTICA (como o GP).** Cada segmento vira um quad que só precisa
// COBRIR a fita (retângulo + tampas); a forma EXATA sai no fragment, da **distância
// do pixel à linha-de-centro** (`gpencil_stroke_segment_mask`, corner type ROUND —
// o default do GP, `GP_CORNER_TYPE_ROUND_BITS = 0`). Clampar o parâmetro ao
// segmento dá **junções e tampas REDONDAS de graça** — sem spikes, e (crucial) com
// cobertura CONSISTENTE onde dois quads cobrem o mesmo pixel (ambos medem a
// distância ao MESMO ponto de junção), então o depth só escolhe um. O perfil de
// hardness é o do GP (`pow` + smoothstep), com AA de ~1px por `fwidth` (o GP conta
// com MSAA; aqui não há).
//
// **O tripé anti-artefato (estado EXATO do GP 2D — cada perna é obrigatória):**
// 1. **Fita CONECTADA por miter + `miter_break`** (`gpencil_vertex`, ~l.705):
//    segmentos adjacentes compartilham o vértice de junção (abutam, não sobrepõem
//    → sem bead/escama); numa quina AFIADA (virada > 120°) a fita NÃO mitra — o
//    offset fica na perpendicular do próprio segmento e o quad ESTENDE `r` ao
//    longo da linha (cobre o disco da junção sem nunca dobrar sobre si → fim do
//    bowtie/spike que o miter puro cuspia na bissetriz).
// 2. **Depth GREATER estrito + write-depth** (`gpencil_cache_utils.cc:449`),
//    depth por-STROKE: a 2ª face no MESMO pixel (sobreposição na quina quebrada,
//    auto-cruzamento) é DESCARTADA, não misturada → zero acúmulo. É o default do
//    GP ("the stroke cannot overlap itself", `gpencil_vert.glsl`); o modo
//    per-ponto (`GP_STROKE_OVERLAP`) que deixa o traço acumular sobre si é opção
//    de material, não o default.
// 3. **Discard de fragmento ~transparente** (`gpencil_frag.glsl`: `a < 0.001`):
//    sem ele, o canto transparente de um quad escreve depth e FURA a geometria
//    que chega depois (era o "escamado"/corrente-de-ovais do stadium+GREATER).

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
// `miter_break` do GP (`gpencil_vertex` ~l.696): a quina deixa de mitrar quando
// `cos_angle > 0.5` (virada > 120°). cos_angle = -dot(dir_in, dir_out): -1 numa
// reta, +1 num hairpin. Abaixo do limite o esticão do miter é ≤ 1/cos(60°) = 2 —
// bounded por construção (o clamp MITER_LIMIT antigo saiu junto com a dobra).
const MITER_BREAK_COS: f32 = 0.5;

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

    // Vizinhos para o miter da FITA CONECTADA: predecessor de a e sucessor de b, com
    // clamp (aberto) / wrap (fechado). Segmentos adjacentes computam o MESMO vértice
    // de junção (compartilham prev/nn) → a fita não se sobrepõe nas junções, logo NÃO
    // há double-blend (o "mastigado" com hardness baixo era o over-blend de quads
    // sobrepostos). Onde quads SE sobrepõem (quina quebrada, cruzamento real), o
    // depth GREATER estrito descarta a 2ª face — pinta uma vez, nunca acumula.
    var prev_gp = gp;
    if (li > 0u) { prev_gp = gp - 1u; }
    else if (closed) { prev_gp = last; }
    let b_li = next_gp - first;
    var nn_gp = next_gp;
    if (b_li + 1u < count) { nn_gp = next_gp + 1u; }
    else if (closed) { nn_gp = first; }

    let a = points[gp];
    let b = points[next_gp];
    let sa = to_screen(a.pos);
    let sb = to_screen(b.pos);
    let sp = to_screen(points[prev_gp].pos);
    let sn = to_screen(points[nn_gp].pos);
    let r_a = max(a.width * 0.5 * cam.px_per_world, 0.0);
    let r_b = max(b.width * 0.5 * cam.px_per_world, 0.0);

    let seg = sb - sa;
    let seg_len = max(length(seg), 1e-6);
    let dir = seg / seg_len;
    let n_seg = perp(dir);

    // Tampas: um EXTREMO de traço aberto marcado `Flat` corta reto (sem estender além
    // do ponto). Tampa Round estende `r` ao longo da reta → a distância clampada
    // desenha a meia-lua. Junções internas mitradas NÃO estendem (o miter conecta a
    // fita); junções QUEBRADAS (miter_break) estendem como uma tampa redonda.
    let is_start = (li == 0u) && !closed;
    let is_end = (next_gp == last) && !closed;
    let round_start = is_start && ((st.flags & FLAG_START_FLAT) == 0u);
    let round_end = is_end && ((st.flags & FLAG_END_FLAT) == 0u);
    var ext_a = select(0.0, r_a, round_start);
    var ext_b = select(0.0, r_b, round_end);

    // Junção em a (compartilhada com o segmento anterior; sem prev = perpendicular
    // reta). Miter na bisetriz de prev→a→b, esticado 1/cos(θ/2) pra alcançar o
    // vértice — MAS numa quina afiada (miter_break, GP ~l.705) NÃO mitra: o offset
    // fica em n_seg e o quad estende `r` ao longo da linha (`screen_ofs += line·x`
    // do GP) — cobre o disco da junção sem a fita jamais dobrar sobre si (bowtie).
    var miter_a = n_seg;
    var scale_a = 1.0;
    if (prev_gp != gp) {
        let d_prev = normalize(sa - sp);
        if (-dot(dir, d_prev) > MITER_BREAK_COS) {
            ext_a = r_a;
        } else {
            let m_tan = normalize(d_prev + dir);
            miter_a = perp(m_tan);
            scale_a = 1.0 / max(dot(m_tan, d_prev), MITER_BREAK_COS);
        }
    }
    var miter_b = n_seg;
    var scale_b = 1.0;
    if (nn_gp != next_gp) {
        let d_next = normalize(sn - sb);
        if (-dot(dir, d_next) > MITER_BREAK_COS) {
            ext_b = r_b;
        } else {
            let m_tan = normalize(dir + d_next);
            miter_b = perp(m_tan);
            scale_b = 1.0 / max(dot(m_tan, d_next), MITER_BREAK_COS);
        }
    }

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
        corner = sb + dir * ext_b + miter_b * (side * r_b * scale_b);
        out.color = b.color;
        out.opacity = b.opacity;
        out.thickness = 2.0 * r_b;
    } else {
        corner = sa - dir * ext_a + miter_a * (side * r_a * scale_a);
        out.color = a.color;
        out.opacity = a.opacity;
        out.thickness = 2.0 * r_a;
    }

    // Ordem 2D (GP §2): profundidade por-traço = (2·sid+2)·2e-7, teste GREATER
    // estrito. O sid maior tem depth estritamente maior e ganha; no MESMO depth
    // (sobreposição do próprio traço) a 2ª face é descartada — o traço pinta uma
    // vez, nunca acumula ("the stroke cannot overlap itself", gpencil_vert.glsl).
    // Fill do traço em (2·sid+1), abaixo.
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
    // Fragmento ~transparente é DESCARTADO (GP `gpencil_frag.glsl`: a < 0.001) —
    // senão ele ESCREVE depth e fura a geometria sobreposta que chega depois (o
    // canto vazio do quad viraria um buraco no traço vizinho).
    if (alpha < 0.001) {
        discard;
    }
    // Saída PREMULTIPLICADA (blend = One, OneMinusSrcAlpha).
    return vec4<f32>(in.color.rgb * alpha, alpha);
}
