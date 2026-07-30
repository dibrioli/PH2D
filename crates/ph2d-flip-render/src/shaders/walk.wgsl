// **O PERCURSO POR LADRILHO, NO DEVICE** — o port do `binning.rs::walk_pixel` + `tau.rs`
// (doc 12 §14). Um workgroup de 16×16 É um ladrilho: a lista que ele lê é a mesma para as 256
// threads, que é a razão inteira de o binning existir.
//
// ⚠️ **Isto é um ESPELHO, não uma segunda lei.** Cada função abaixo tem o irmão em Rust nomeado
// no comentário, e o gate `the_device_walk_is_the_cpu_walk` compara os dois no MESMO desenho. As
// operações estão na MESMA ORDEM de propósito — em `f32` a ordem é o valor (a lição que o
// `dab_weight` do passo 3 pagou nascendo vermelho por `p*p*(3−2p)` contra `3p²−2p³`).

struct Pt {
    pos: vec2<f32>,
    width: f32,
    opacity: f32,
    color: vec4<f32>,
}

struct Stroke {
    first_point: u32,
    point_count: u32,
    flags: u32,
    hardness: f32,
    material: u32,
    tip: u32,
    dot_spacing: f32,
    ref_width: f32,
}

struct Seg {
    stroke: u32,
    a: u32,
    b: u32,
}

struct Screen {
    world_to_clip: mat4x4<f32>,
    // xy = viewport, z = px_per_world, w = pad
    view: vec4<f32>,
    // x = tile, y = cols, z = rows, w = pad
    grid: vec4<u32>,
}

@group(0) @binding(0) var<uniform> sc: Screen;
@group(0) @binding(1) var<storage, read> points: array<Pt>;
@group(0) @binding(2) var<storage, read> strokes: array<Stroke>;
@group(0) @binding(3) var<storage, read> ranges: array<vec2<u32>>;
@group(0) @binding(4) var<storage, read> segs: array<Seg>;
// ⚠️ **UMA saída, e é uma TEXTURA** — a que o produto de fato usa (o `hdr` premult 16F do
// `FlipCompose`). Uma 2ª saída em buffer daria dois caminhos para o mesmo pixel, e a paridade
// passaria a medir um deles enquanto o outro shipa.
@group(0) @binding(5) var out_tex: texture_storage_2d<rgba16float, write>;
// O PISO: os fills, já rasterizados pelo pipeline que sempre os desenhou (premult, linear). O
// percurso compõe os traços POR CIMA — fill abaixo, traço em cima, a ordem do `draw`.
@group(0) @binding(6) var fills_tex: texture_2d<f32>;
// Comprimento de arco cumulativo (MUNDO) por-ponto — `FlipGpuData::arc_len`, o MESMO buffer que o
// vertex do rasterizador lê. É o que espaça as contas do *tip* por ARCO em vez de por densidade de
// input, e é cumulativo ⇒ não dá para derivar de um segmento solto.
@group(0) @binding(7) var<storage, read> arc_len: array<f32>;

// `pack.rs`
const FLAG_CLOSED: u32 = 1u;
const FLAG_START_FLAT: u32 = 2u;
const FLAG_END_FLAT: u32 = 4u;
// `pack.rs::FLAG_SELF_OVERLAP` — com o bit as PASSAGENS do traço compõem em vez de saturarem.
const FLAG_SELF_OVERLAP: u32 = 8u;
const FLAG_AIRBRUSH: u32 = 16u;
// `pack.rs::tip_code` (a porta única CPU→GPU) — TÊM de bater com o `flip.wgsl` e o `tau.rs`.
const TIP_DOTS: u32 = 1u;
const TIP_SQUARES: u32 = 2u;
// `binning.rs::MIN_WIDTH_PX`
const MIN_WIDTH_PX: f32 = 1.3;
// `tau.rs`
const PAINTER_SPACING: f32 = 0.10;
const MIN_PITCH_PX: f32 = 0.25;
const SUB: f32 = 4.0;
const F_MAX: f32 = 16.0;
// `flip.wgsl::AIRBRUSH_K_MIN/MAX`
const AIRBRUSH_K_MIN: f32 = 1.0;
const AIRBRUSH_K_MAX: f32 = 8.0;

// `ScreenSpace::point_px`
fn point_px(w: vec2<f32>) -> vec2<f32> {
    let m = sc.world_to_clip;
    let cx = m[0][0] * w.x + m[1][0] * w.y + m[3][0];
    let cy = m[0][1] * w.x + m[1][1] * w.y + m[3][1];
    let cw = m[0][3] * w.x + m[1][3] * w.y + m[3][3];
    var inv = 1.0;
    if (cw != 0.0) {
        inv = 1.0 / cw;
    }
    // O Y inverte: clip +1 é o TOPO, e a linha 0 da textura é o topo (ver o irmão em Rust).
    return vec2<f32>(
        (cx * inv * 0.5 + 0.5) * sc.view.x,
        (0.5 - cy * inv * 0.5) * sc.view.y,
    );
}

// `ScreenSpace::radius_px`
fn radius_px(width: f32) -> f32 {
    return max(width * 0.5 * sc.view.z, MIN_WIDTH_PX * 0.5);
}

// `ScreenSpace::thickness_px` — a espessura CRUA, sem o piso. Irmã da de cima, nunca substituta: a
// forma usa o raio com piso (senão a linha fina pisca), a cobertura usa esta (senão ela sai grossa).
fn thickness_px(width: f32) -> f32 {
    return width * sc.view.z;
}

// `tau.rs::sub_pixel_fade` — o `gpencil_frag.glsl:534` que o `flip.wgsl` multiplica na máscara.
// ⚠️ Aqui é o **builtin**, o MESMO que o rasterizador chama, para o device e ele serem a mesma
// aritmética; o Rust carrega a expansão termo a termo, com gate irmão.
fn sub_pixel_fade(w_px: f32) -> f32 {
    return smoothstep(0.0, 1.0, w_px);
}

// `binning.rs::closest_on_seg` — devolve (t, cx, cy).
fn closest_on_seg(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> vec3<f32> {
    let v = b - a;
    let len2 = dot(v, v);
    var t = 0.0;
    if (len2 > 1e-12) {
        t = clamp(dot(p - a, v) / len2, 0.0, 1.0);
    }
    let c = a + v * t;
    return vec3<f32>(t, c.x, c.y);
}

// `tau.rs::dab_weight`
fn dab_weight(dn: f32, hardness: f32) -> f32 {
    let h = clamp(hardness, 0.0, 1.0);
    if (h >= 1.0) {
        return select(0.0, 1.0, dn < 1.0);
    }
    let remapped = clamp((dn - h) / (1.0 - h), 0.0, 1.0);
    if (remapped >= 1.0) {
        return 0.0;
    }
    let q = 1.0 - remapped;
    return 3.0 * q * q - 2.0 * q * q * q;
}

// `tau.rs::f_of` — só o perfil PADRÃO (o airbrush tem outra medida; ver `d_tau_of`).
fn f_of(dn: f32, hardness: f32) -> f32 {
    let w = dab_weight(dn, hardness);
    if (w >= 1.0) {
        return F_MAX;
    }
    return min(-log(1.0 - w), F_MAX);
}

// `tau.rs::d_tau_of` — a porta única, e ela decide a MEDIDA. Padrão: por DAB (`step/pitch`).
// Airbrush: densidade UNIFORME no disco, por unidade de CAMINHO (`step/2r`) — o `pitch` cancela.
// A uniformidade é a inversão de Abel da corda `√(1−dn²)` que o `flip.wgsl` escreve (ver o irmão
// em Rust: `f = k·√(1−dn²)` por dab foi medido e REPROVADO, super-entinta).
fn d_tau_of(dn: f32, hardness: f32, airbrush: bool, step: f32, r: f32, pitch: f32) -> f32 {
    if (airbrush) {
        if (dn >= 1.0) {
            return 0.0;
        }
        let k = mix(AIRBRUSH_K_MIN, AIRBRUSH_K_MAX, clamp(hardness, 0.0, 1.0));
        return k * step / (2.0 * r);
    }
    let fv = f_of(dn, hardness);
    if (fv <= 0.0) {
        return 0.0;
    }
    return fv * step / pitch;
}

// `tau.rs::f_bead_of` — a densidade de UM CARIMBO. No airbrush é a CORDA (a projeção de Abel de um
// dab), que é a resposta certa para um carimbo e a errada para um caminho; ver o irmão em Rust.
fn f_bead_of(dn: f32, hardness: f32, airbrush: bool) -> f32 {
    if (dn >= 1.0) {
        return 0.0;
    }
    if (airbrush) {
        let k = mix(AIRBRUSH_K_MIN, AIRBRUSH_K_MAX, clamp(hardness, 0.0, 1.0));
        return k * sqrt(max(1.0 - dn * dn, 0.0));
    }
    return f_of(dn, hardness);
}

// `tau.rs::TipShape::of` — a pitch das contas em MUNDO, ou `0` para a linha cheia. Conta mais junta
// que o dab do próprio pincel É a linha cheia (a lei, não um cap), e é isso que limita o laço.
fn bead_pitch_of(st: Stroke) -> f32 {
    if (st.tip != TIP_DOTS && st.tip != TIP_SQUARES) {
        return 0.0;
    }
    let pitch = st.dot_spacing * st.ref_width;
    if (!(st.dot_spacing > PAINTER_SPACING) || !(pitch > 0.0)) {
        return 0.0;
    }
    return pitch;
}

// `tau.rs::dab_reach` — o alcance de um dab a partir da linha-de-centro. Um carimbo QUADRADO chega
// a `r√2` na diagonal, e o binner (CPU) pergunta à MESMA porta.
fn dab_reach(pitch: f32, square: bool, rmax: f32) -> f32 {
    if (pitch > 0.0 && square) {
        return rmax * 1.4142135623730951;
    }
    return rmax;
}

// `tau.rs::Bead`
struct BeadG {
    c: vec2<f32>,
    r: f32,
    dir: vec2<f32>,
}

// `tau.rs::bead_at` — a fração de arco É a fração de tela (câmera uniforme).
fn bead_at(sa: vec2<f32>, sb: vec2<f32>, ra: f32, rb: f32, arc_a: f32, wlen: f32, k: i32, pitch: f32) -> BeadG {
    var f = 0.0;
    if (wlen > 1e-12) {
        f = clamp((f32(k) * pitch - arc_a) / wlen, 0.0, 1.0);
    }
    let v = sb - sa;
    let len = length(v);
    var dir = vec2<f32>(1.0, 0.0);
    if (len > 1e-6) {
        dir = v / len;
    }
    var b: BeadG;
    b.c = sa + v * f;
    b.r = max(ra * (1.0 - f) + rb * f, 1e-4);
    b.dir = dir;
    return b;
}

// `tau.rs::bead_dn` — distância ao PONTO, nunca `√(dn² + arco²)` (o arco curva e a conta viraria
// banana). O quadrado é a Chebyshev no frame da tangente.
fn bead_dn(p: vec2<f32>, b: BeadG, square: bool) -> f32 {
    let d = p - b.c;
    if (square) {
        let along = abs(dot(d, b.dir));
        let across = abs(dot(d, vec2<f32>(-b.dir.y, b.dir.x)));
        return max(along, across) / b.r;
    }
    return length(d) / b.r;
}

// `tau.rs::bead_range` — as contas que este segmento POSSUI (meio-aberta no fim: a conta de uma
// junção tem UM dono). `arc_b = arc_a + |b−a|`, nunca `arc_len[b]` (o fecho do anel). O
// `end_inclusive` é a conta da PONTA, que não tem segmento seguinte para adotá-la.
fn bead_range(arc_a: f32, arc_b: f32, pitch: f32, end_inclusive: bool) -> vec2<i32> {
    let k0 = i32(ceil(arc_a / pitch));
    if (end_inclusive) {
        return vec2<i32>(k0, i32(floor(arc_b / pitch)));
    }
    return vec2<i32>(k0, i32(ceil(arc_b / pitch)) - 1);
}

// `tau.rs::tail_point` — o último ponto de um traço ABERTO, ou `0xffffffff` se não há.
fn tail_point(sid: u32) -> u32 {
    let st = strokes[sid];
    if ((st.flags & FLAG_CLOSED) != 0u || st.point_count == 0u) {
        return 0xffffffffu;
    }
    return st.first_point + st.point_count - 1u;
}

// `tau.rs::bead_window` — a janela do pixel, alargada de uma conta em cada lado (só pode sobrar).
fn bead_window(lo: f32, hi: f32, pitch: f32) -> vec2<i32> {
    return vec2<i32>(i32(floor(lo / pitch)), i32(ceil(hi / pitch)));
}

// ————————————— `pixel_area.rs` — a AREA do pixel sob a uniao dos semi-planos —————————————
//
// ⚠️ **Isto e' o port do `pixel_area.rs`, e o `walk_gpu_parity` e' quem prova que os dois
// concordam — medido, o pior |delta| e' 4,883e-4, que E' o quantum da meia precisao do alvo.**
// A lei antiga era `edge = 0.5 - sd`, o filtro-caixa 1-D ao longo da normal — exato SO' quando a
// borda e' paralela a um eixo do pixel. Medido no produto: 0,00/255 a 0 e 90 graus, mas 9,72 a 45; e
// numa quina, onde duas bordas cruzam o mesmo pixel, 63,75 (a uniao cobre 3/4 e o `min` dos SDFs
// dizia meio). Recortar o quadrado pelos semi-planos de FORA responde as duas com um mecanismo so'.
// = `core::f32::consts::FRAC_1_SQRT_2` do lado da CPU; os digitos batem porque isto e' um
// LIMIAR, e um limiar que difere entre os dois lados decide diferente na janela da diferenca.
const PLANE_REACH: f32 = 0.70710678;
// Uma vaga vazia carrega `sd = 1e30`, que ja' cai no mesmo `>= PLANE_REACH` do descartavel — uma
// pergunta, dois significados, sem contador separado para desandar.
const PLANE_EMPTY: vec3<f32> = vec3<f32>(1.0, 0.0, 1e30);

// `PlaneSet::offer` — entra ordenado por `sd` crescente e empurra o maior para fora.
fn plane_offer(planes: ptr<function, array<vec3<f32>, 3>>, n: vec2<f32>, sd: f32) {
    if (sd >= PLANE_REACH) {
        return;
    }
    var entrando = vec3<f32>(n.x, n.y, sd);
    for (var i = 0; i < 3; i = i + 1) {
        let atual = (*planes)[i];
        if (entrando.z < atual.z) {
            (*planes)[i] = entrando;
            entrando = atual;
        }
    }
}

// `PlaneSet::coverage` — Sutherland-Hodgman + sapateiro sobre o quadrado unitario.
fn plane_coverage(planes: array<vec3<f32>, 3>) -> f32 {
    // ⚠️ Os dois atalhos saem da mesma aritmetica do `PLANE_REACH` (`|n.u| <= raiz(2)/2` sobre o
    // quadrado): nenhum plano em alcance => nada coberto; o mais recortante ja' engole o quadrado
    // => tudo coberto. Sao exatos, e MEDIDOS valem 4% aqui (4,91 -> 4,71 ms) — pixel de fronteira e'
    // espalhado, entao quase todo warp toma o caminho longo de qualquer jeito.
    if (planes[0].z >= PLANE_REACH) {
        return 0.0;
    }
    if (planes[0].z <= -PLANE_REACH) {
        return 1.0;
    }
    var poly: array<vec2<f32>, 7>;
    poly[0] = vec2<f32>(-0.5, -0.5);
    poly[1] = vec2<f32>(0.5, -0.5);
    poly[2] = vec2<f32>(0.5, 0.5);
    poly[3] = vec2<f32>(-0.5, 0.5);
    var n = 4;
    for (var k = 0; k < 3; k = k + 1) {
        let pl = planes[k];
        if (pl.z >= PLANE_REACH) {
            continue;
        }
        var src: array<vec2<f32>, 7> = poly;
        var out = 0;
        for (var j = 0; j < n; j = j + 1) {
            let a = src[j];
            let b = src[select(j + 1, 0, j + 1 == n)];
            let da = pl.x * a.x + pl.y * a.y + pl.z;
            let db = pl.x * b.x + pl.y * b.y + pl.z;
            if (da >= 0.0) {
                poly[out] = a;
                out = out + 1;
            }
            if ((da >= 0.0) != (db >= 0.0)) {
                let t = da / (da - db);
                poly[out] = a + (b - a) * t;
                out = out + 1;
            }
        }
        n = out;
        if (n == 0) {
            // O plano engoliu o quadrado: nada descoberto, cobertura cheia.
            return 1.0;
        }
    }
    var duas = 0.0;
    for (var i = 0; i < n; i = i + 1) {
        let a = poly[i];
        let b = poly[select(i + 1, 0, i + 1 == n)];
        duas = duas + a.x * b.y - b.x * a.y;
    }
    return clamp(1.0 - abs(duas * 0.5), 0.0, 1.0);
}

struct Sil {
    sd: f32,
    near: vec2<f32>,
    dist: f32,
    planes: array<vec3<f32>, 3>,
}

// `binning.rs::stroke_silhouette` — `sd = 1e30` se vazio.
// Com contas a silhueta é a UNIÃO DELAS (um disco é uma cápsula degenerada): sem isto o `edge`
// mediria a borda da FITA e as contas sairiam sem anti-aliasing.
fn stroke_silhouette(lo: u32, hi: u32, p: vec2<f32>, pitch: f32, square: bool) -> Sil {
    var best = vec4<f32>(1e30, 0.0, 0.0, 0.0);
    var planes = array<vec3<f32>, 3>(PLANE_EMPTY, PLANE_EMPTY, PLANE_EMPTY);
    var tail = 0xffffffffu;
    // `tau.rs::flat_caps` — os pontos onde a fita é CORTADA em vez de arredondada. `0xffffffff` = sem
    // tampa. Num traço FECHADO não há ponta (o `flip.wgsl` gateia em `!closed` pelo mesmo motivo).
    var cap_head = 0xffffffffu;
    var cap_tail = 0xffffffffu;
    if (hi > lo) {
        let sid = segs[lo].stroke;
        tail = tail_point(sid);
        let st = strokes[sid];
        if ((st.flags & FLAG_CLOSED) == 0u && st.point_count > 0u) {
            if ((st.flags & FLAG_START_FLAT) != 0u) {
                cap_head = st.first_point;
            }
            if ((st.flags & FLAG_END_FLAT) != 0u) {
                cap_tail = st.first_point + st.point_count - 1u;
            }
        }
    }
    for (var k = lo; k < hi; k = k + 1u) {
        let s = segs[k];
        let pa = points[s.a];
        let pb = points[s.b];
        let sa = point_px(pa.pos);
        let sb = point_px(pb.pos);
        let tc = closest_on_seg(p, sa, sb);
        let ra = radius_px(pa.width);
        let rb = radius_px(pb.width);
        // O CORTE deste segmento (`-1e30` = sem tampa; `max` com ele é a identidade exata, então
        // todo traço de tampa redonda é byte-intocado). A normal aponta para FORA.
        var cut = -1e30;
        if (s.a == cap_head || s.b == cap_tail) {
            let v = sb - sa;
            let vlen = length(v);
            if (vlen > 1e-6) {
                let dir = v / vlen;
                if (s.a == cap_head) {
                    cut = max(cut, cap_sd(p, sa, -dir));
                }
                if (s.b == cap_tail) {
                    cut = max(cut, cap_sd(p, sb, dir));
                }
            }
        }
        if (pitch > 0.0) {
            let arc_a = arc_len[s.a];
            let wlen = length(pb.pos - pa.pos);
            let own = bead_range(arc_a, arc_a + wlen, pitch, s.b == tail);
            if (own.x > own.y) {
                continue;
            }
            let base = i32(floor((arc_a + tc.x * wlen) / pitch));
            for (var q = 0; q < 2; q = q + 1) {
                let kb = clamp(base + q, own.x, own.y);
                let b = bead_at(sa, sb, ra, rb, arc_a, wlen, kb, pitch);
                // A "distância" é `dn·r`: no disco isso É `|p − c|`, e no quadrado é a Chebyshev.
                let dist = bead_dn(p, b, square) * b.r;
                let sd = max(dist - b.r, cut);
                plane_offer(&planes, plane_normal(p, b.c, dist), sd);
                if (sd < best.x) {
                    best = vec4<f32>(sd, b.c.x, b.c.y, dist);
                }
            }
            continue;
        }
        let dist = length(p - vec2<f32>(tc.y, tc.z));
        let r = ra * (1.0 - tc.x) + rb * tc.x;
        let sd = max(dist - r, cut);
        plane_offer(&planes, plane_normal(p, vec2<f32>(tc.y, tc.z), dist), sd);
        if (sd < best.x) {
            best = vec4<f32>(sd, tc.y, tc.z, dist);
        }
    }
    return Sil(best.x, vec2<f32>(best.y, best.z), best.w, planes);
}

// A normal de FORA: do ponto mais proximo em direcao ao pixel. Com o pixel EM CIMA do eixo
// (`dist ~ 0`) ela e' indefinida e nao importa — ali a silhueta e' um disco centrado no pixel,
// equidistante em toda direcao, entao qualquer normal unitaria da' a mesma area.
fn plane_normal(p: vec2<f32>, near: vec2<f32>, dist: f32) -> vec2<f32> {
    if (dist > 1e-6) {
        return (p - near) / dist;
    }
    return vec2<f32>(1.0, 0.0);
}

// `tau.rs::seg_window` — os `t` onde uma amostra cai dentro do alcance de um dab em torno de `p`.
// `z < 0` = o segmento nao alcança este pixel. ⚠️ Porta ÚNICA: a quadratura usa para saber ONDE
// integrar e o `pass_end` para saber SE o segmento pertence a passagem.
fn seg_window(p: vec2<f32>, sa: vec2<f32>, sb: vec2<f32>, reach: f32) -> vec3<f32> {
    let v = sb - sa;
    let len2 = dot(v, v);
    if (len2 <= 1e-12) {
        return vec3<f32>(0.0, 0.0, -1.0);
    }
    let w = sa - p;
    let wv = dot(w, v);
    let ww = dot(w, w);
    let disc = wv * wv - len2 * (ww - reach * reach);
    if (disc <= 0.0) {
        return vec3<f32>(0.0, 0.0, -1.0);
    }
    let sq = sqrt(disc);
    let t0 = clamp((-wv - sq) / len2, 0.0, 1.0);
    let t1 = clamp((-wv + sq) / len2, 0.0, 1.0);
    if (t1 <= t0) {
        return vec3<f32>(0.0, 0.0, -1.0);
    }
    return vec3<f32>(t0, t1, 1.0);
}

// `tau.rs::pass_end` — o fim (exclusivo) da cadeia CONTÍGUA que começa em `start`. Sem predicado de
// alcance e sem épsilon: o binner lista todo segmento a `r` do LADRILHO, então um buraco na cadeia
// significa que os do meio nao alcançam nem o ladrilho — o traço foi embora e voltou. Ver o irmão em
// Rust para a versao por ALCANCE, construida, MEDIDA e reprovada (passagens fantasma).
fn pass_end(lo: u32, hi: u32, start: u32) -> u32 {
    var k = start;
    loop {
        if (k + 1u >= hi) { break; }
        if (segs[k + 1u].a != segs[k].b) { break; }
        k = k + 1u;
    }
    return k + 1u;
}

// `tau.rs::cap_sd` — o SDF (px, positivo = FORA) do corte de uma tampa chata. Interseção com a
// cápsula = `max`, e é isso que dá anti-aliasing à borda reta de graça.
fn cap_sd(p: vec2<f32>, q: vec2<f32>, n: vec2<f32>) -> f32 {
    return dot(p - q, n);
}

// `tau.rs::end_dab` — o termo de fronteira, SÓ no começo (a assimetria da referência).
fn end_dab(sid: u32, hardness: f32, airbrush: bool, pitch: f32, p: vec2<f32>, tau: ptr<function, f32>, acc: ptr<function, vec4<f32>>, fade_acc: ptr<function, f32>) {
    let st = strokes[sid];
    if ((st.flags & FLAG_CLOSED) != 0u || st.point_count == 0u) {
        return;
    }
    if ((st.flags & FLAG_START_FLAT) != 0u) {
        return;
    }
    // Uma fileira de CONTAS não tem termo de fronteira — ali a soma já é discreta, e a conta `k = 0`
    // cai no arco 0, ou seja no primeiro ponto.
    if (pitch > 0.0) {
        return;
    }
    let pt = points[st.first_point];
    let s = point_px(pt.pos);
    let r = max(radius_px(pt.width), 1e-4);
    let dn = length(p - s) / r;
    // O airbrush não tem termo de fronteira (a medida dele é caminho, não contagem de dabs).
    if (airbrush) {
        return;
    }
    let d_tau = 0.5 * f_of(dn, hardness);
    if (d_tau <= 0.0) {
        return;
    }
    *tau = *tau + d_tau;
    // Meio dab é um dab: ele carrega o fade da própria espessura, como qualquer outro.
    *fade_acc = *fade_acc + sub_pixel_fade(thickness_px(pt.width)) * d_tau;
    *acc = *acc + vec4<f32>(
        pt.color.x * d_tau,
        pt.color.y * d_tau,
        pt.color.z * d_tau,
        pt.color.w * pt.opacity * d_tau,
    );
}

// `tau.rs::stroke_tau` — devolve (tau, acc, fade) com a cor e o fade já divididos por τ (o `Ink`);
// tau <= 0 ⇒ sem depósito.
fn stroke_tau(lo: u32, hi: u32, sid: u32, hardness: f32, airbrush: bool, pitch: f32, square: bool, p: vec2<f32>, out_rgb: ptr<function, vec4<f32>>, out_fade: ptr<function, f32>) -> f32 {
    var tau = 0.0;
    var acc = vec4<f32>(0.0);
    var fade_acc = 0.0;
    let tail = tail_point(sid);
    for (var k = lo; k < hi; k = k + 1u) {
        let s = segs[k];
        let pa = points[s.a];
        let pb = points[s.b];
        let sa = point_px(pa.pos);
        let sb = point_px(pb.pos);
        let ra = radius_px(pa.width);
        let rb = radius_px(pb.width);
        // As espessuras CRUAS, e o atalho do caso comum: com as duas pontas ≥ 1 px toda amostra
        // entre elas mede ≥ 1 px (combinação convexa) e o fade é `1.0` exato.
        let wa = thickness_px(pa.width);
        let wb = thickness_px(pb.width);
        let full = wa >= 1.0 && wb >= 1.0;
        let v = sb - sa;
        let len2 = dot(v, v);
        if (len2 <= 1e-12) {
            continue;
        }
        let len = sqrt(len2);

        let rmax = max(ra, rb);
        let reach = dab_reach(pitch, square, rmax);
        // A janela vem da porta ÚNICA, a MESMA que o `pass_end` pergunta.
        let win = seg_window(p, sa, sb, reach);
        if (win.z < 0.0) {
            continue;
        }
        let t0 = win.x;
        let t1 = win.y;

        // **AS CONTAS** — a soma sobre os carimbos que esta janela alcança, SEM peso de arco: uma
        // conta é UM dab, não um tubo varrido.
        if (pitch > 0.0) {
            let arc_a = arc_len[s.a];
            let wlen = length(pb.pos - pa.pos);
            let own = bead_range(arc_a, arc_a + wlen, pitch, s.b == tail);
            let win = bead_window(arc_a + t0 * wlen, arc_a + t1 * wlen, pitch);
            let kb0 = max(own.x, win.x);
            let kb1 = min(own.y, win.y);
            for (var kb = kb0; kb <= kb1; kb = kb + 1) {
                let b = bead_at(sa, sb, ra, rb, arc_a, wlen, kb, pitch);
                let d_tau = f_bead_of(bead_dn(p, b, square), hardness, airbrush);
                if (d_tau <= 0.0) {
                    continue;
                }
                tau = tau + d_tau;
                // A conta carrega UMA cor — a de onde ela está. É o que um carimbo é.
                var f = 0.0;
                if (wlen > 1e-12) {
                    f = clamp((f32(kb) * pitch - arc_a) / wlen, 0.0, 1.0);
                }
                var fd = 1.0;
                if (!full) { fd = sub_pixel_fade(wa * (1.0 - f) + wb * f); }
                fade_acc = fade_acc + fd * d_tau;
                let op = pa.opacity * (1.0 - f) + pb.opacity * f;
                let col = pa.color * (1.0 - f) + pb.color * f;
                acc = acc + vec4<f32>(col.x * d_tau, col.y * d_tau, col.z * d_tau, col.w * op * d_tau);
            }
            continue;
        }

        let pitch_min = max(PAINTER_SPACING * 2.0 * min(ra, rb), MIN_PITCH_PX);
        let ds = pitch_min / SUB;
        // ⚠️ A grade resolve a JANELA, nao o SEGMENTO (irmao do `tau.rs`; o doc 12 §22.8/§22.10
        // traz o porque e o preco medido). Ancorada no segmento, um pixel cuja cobertura so' vem de
        // perto de um EXTREMO nao pegava amostra nenhuma e o percurso DERRUBAVA a tinta.
        let win_len = (t1 - t0) * len;
        let n = max(ceil(win_len / ds), 1.0);
        let step = win_len / n;
        let i1 = u32(n) - 1u;

        for (var i = 0u; i <= i1; i = i + 1u) {
            let t = clamp(t0 + (f32(i) + 0.5) * step / len, 0.0, 1.0);
            let sp = sa + v * t;
            let r = max(ra * (1.0 - t) + rb * t, 1e-4);
            let dn = length(p - sp) / r;
            let pitch = max(PAINTER_SPACING * 2.0 * r, MIN_PITCH_PX);
            let d_tau = d_tau_of(dn, hardness, airbrush, step, r, pitch);
            if (d_tau <= 0.0) {
                continue;
            }
            tau = tau + d_tau;
            var fd = 1.0;
            if (!full) { fd = sub_pixel_fade(wa * (1.0 - t) + wb * t); }
            fade_acc = fade_acc + fd * d_tau;
            let op = pa.opacity * (1.0 - t) + pb.opacity * t;
            let col = pa.color * (1.0 - t) + pb.color * t;
            acc = acc + vec4<f32>(col.x * d_tau, col.y * d_tau, col.z * d_tau, col.w * op * d_tau);
        }
    }
    end_dab(sid, hardness, airbrush, pitch, p, &tau, &acc, &fade_acc);
    if (tau <= 0.0) {
        return 0.0;
    }
    *out_rgb = acc / tau;
    *out_fade = fade_acc / tau;
    return tau;
}

@compute @workgroup_size(16, 16, 1)
fn walk(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = u32(sc.view.x);
    let h = u32(sc.view.y);
    if (gid.x >= w || gid.y >= h) {
        return;
    }
    let p = vec2<f32>(f32(gid.x) + 0.5, f32(gid.y) + 0.5);
    var acc = textureLoad(fills_tex, vec2<i32>(i32(gid.x), i32(gid.y)), 0);

    // `TileBins::tile_of_pixel`
    let tile = sc.grid.x;
    let tx = gid.x / tile;
    let ty = gid.y / tile;
    if (tx < sc.grid.y && ty < sc.grid.z) {
        let rng = ranges[ty * sc.grid.y + tx];
        let lo = rng.x;
        let hi = rng.x + rng.y;
        // `binning.rs::walk_list` — o scan de RUN. A lista já vem ordenada por (traço, segmento),
        // e é essa ordem que torna o agrupamento correto sem ordenar nada aqui.
        var i = lo;
        loop {
            if (i >= hi) { break; }
            let sid = segs[i].stroke;
            var j = i;
            loop {
                if (j >= hi || segs[j].stroke != sid) { break; }
                j = j + 1u;
            }
            // `binning.rs::stroke_deposit` — o ESTILO do traço é lido UMA vez (o `StrokeStyle::of`).
            let st = strokes[sid];
            let pitch = bead_pitch_of(st);
            let square = st.tip == TIP_SQUARES;
            // **SELF OVERLAP** — sem o bit o traço é UMA passagem (a sub-lista é a lista toda, e o
            // laço roda uma vez ⇒ byte-idêntico); com o bit cada PASSAGEM é tratada como um traço,
            // pela MESMA composição `over` de baixo. Ver o irmão em `binning.rs::walk_list`.
            let overlap = (st.flags & FLAG_SELF_OVERLAP) != 0u;
            var ps = i;
            loop {
            if (ps >= j) { break; }
            var pe = j;
            if (overlap) {
                pe = pass_end(i, j, ps);
            }
            let sil = stroke_silhouette(ps, pe, p, pitch, square);
            let edge = plane_coverage(sil.planes);
            if (sil.sd < 1e29 && edge > 0.0) {
                var p_eval = p;
                // ⚠️ O empurrao segue lendo `sd`, NAO o `edge` — sao perguntas diferentes (onde ao
                // longo da NORMAL amostrar o perfil, contra quanto do PIXEL a silhueta cobre), e o
                // irmao em `binning.rs` carrega a mesma nota.
                if (sil.sd > -0.5 && sil.dist > 1e-6) {
                    let f = (sil.sd + 0.5) * 0.5 / sil.dist;
                    p_eval = p + (sil.near - p) * f;
                }
                var rgba = vec4<f32>(0.0);
                var fade = 0.0;
                let airbrush = (st.flags & FLAG_AIRBRUSH) != 0u;
                let tau = stroke_tau(ps, pe, sid, st.hardness, airbrush, pitch, square, p_eval, &rgba, &fade);
                if (tau > 0.0) {
                    // O fade multiplica a COBERTURA, ao lado do `edge` — nunca o `τ` (ver o irmão em
                    // Rust: escalar o `τ` satura junto com ele e a linha fina sairia opaca).
                    let cover = (1.0 - exp(-tau)) * min(edge, 1.0) * fade;
                    if (cover > 0.0) {
                        let a = clamp(rgba.w * cover, 0.0, 1.0);
                        acc = vec4<f32>(
                            rgba.x * a + acc.x * (1.0 - a),
                            rgba.y * a + acc.y * (1.0 - a),
                            rgba.z * a + acc.z * (1.0 - a),
                            a + acc.w * (1.0 - a),
                        );
                    }
                }
            }
            ps = pe;
            }
            i = j;
        }
    }
    textureStore(out_tex, vec2<i32>(i32(gid.x), i32(gid.y)), acc);
}
