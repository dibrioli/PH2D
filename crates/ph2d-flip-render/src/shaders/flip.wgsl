// Flip — rasterização de traço (ADR-0114, W1), clean-room do Grease Pencil 5.2
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
//
// **O INVARIANTE que sustenta o tripé (e a 4ª peça, que o GP NÃO tem):**
// como o depth first-wins elege um vencedor ARBITRÁRIO entre os quads que cobrem
// o mesmo pixel, o sistema só é correto se *os quads sobrepostos computarem a
// MESMA máscara*. O GP quebra esse invariante no canto ROUND quebrado com
// hardness < 1 — o quad anterior vence e pinta a sua queda RADIAL por cima do
// NÚCLEO do seguinte (a "mordida"; artefato ABERTO no Blender, issue #140075: o
// default hardness=1.0 + SMAA o escondem lá, mas aqui o pincel macio é o caso
// comum). Portanto **divergimos do GP de propósito**, na direção que o próprio
// shader dele aponta (ele já passa p0/p3 ao fragment, e os usa nas cunhas
// BEVEL/MITER): a cobertura do fragment é a **UNIÃO LOCAL das 3 cápsulas**
// (`min(dn_prev, dn_own, dn_next)` — o perfil é monótono decrescente, logo
// min-distância ⇔ max-cobertura). Na sobreposição de uma quina, a janela de A
// (`{prev,A,B}`) e a de B (`{A,B,next}`) contêm ambas `{A,B}` → computam o MESMO
// mínimo → o first-wins volta a ser invisível. Teto conhecido: sobreposição com
// vizinhos i±2 e auto-cruzamento NÃO-adjacente seguem first-wins (semântica do
// GP, pinada em teste). Spec + análise: `docs/Flip/03_traco_rasterizacao.md`.

struct Camera {
    world_to_clip: mat4x4<f32>,
    viewport: vec2<f32>,   // px
    px_per_world: f32,     // escala de espessura (1 = tela absoluta; object scale quando escalado)
    _pad: f32,
    // Ghost Frames (W3): `a > 0` = este passe é um FANTASMA — a arte vira SILHUETA
    // 100% recolorida em `rgb` (não é um blend da cor original: é o look clássico do
    // onion) e o alpha inteiro é multiplicado por `a` (o fade 1/|Δ|). `a == 0` = passe
    // normal, e o `if` nem toca a cor.
    ghost_tint: vec4<f32>,
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
// `points` é visível ao VERTEX (expansão do quad) e ao FRAGMENT (as cápsulas dos
// vizinhos GEOMÉTRICOS, abaixo).
@group(0) @binding(1) var<storage, read> points: array<GpuPoint>;
@group(0) @binding(2) var<storage, read> strokes: array<GpuStroke>;
@group(0) @binding(3) var<storage, read> point_stroke: array<u32>;
// A janela de vizinhos GEOMÉTRICOS (`neighbors.rs`): por segmento (indexado pelo
// seu ponto inicial), `(offset, count)` na lista `seg_extras` de pares (a, b) de
// pontos. São os segmentos NÃO-adjacentes cujas cápsulas podem alcançar os pixels
// deste quad — sem eles, um traço que volta sobre si mesmo tem a "mordida" de
// longo alcance (o quad de índice menor vence o núcleo do outro por depth). Vazio
// na esmagadora maioria dos traços: `count == 0` ⇒ custo ZERO no fragment.
@group(0) @binding(4) var<storage, read> seg_extra_range: array<vec2<u32>>;
@group(0) @binding(5) var<storage, read> seg_extras: array<vec2<u32>>;

const FLAG_CLOSED: u32 = 1u;
const FLAG_START_FLAT: u32 = 2u;
const FLAG_END_FLAT: u32 = 4u;
// `miter_break` do GP (`gpencil_vertex` ~l.696): a quina deixa de mitrar quando
// `cos_angle > 0.5` (virada > 120°). cos_angle = -dot(dir_in, dir_out): -1 numa
// reta, +1 num hairpin. Abaixo do limite o esticão do miter é ≤ 1/cos(60°) = 2 —
// bounded por construção (o clamp MITER_LIMIT antigo saiu junto com a dobra).
const MITER_BREAK_COS: f32 = 0.5;
// Largura MÍNIMA rasterizada, em px de tela. Abaixo disto o traço não afina mais —
// ele desbota (o `mask *= smoothstep(0,1, thickness)` do fragment, com a espessura
// NÃO-clampada). É o par do Grease Pencil (`gpencil_frag.glsl:534` + o clamp da
// espessura): sem o clamp, a fita fina não cobre o centro de nenhum pixel e a linha
// PISCA (o rasterizador acerta ou erra); sem o fade, a linha fina fica grossa demais.
// Juntos, a energia é preservada e a linha desaparece suavemente ao dar zoom out.
const MIN_WIDTH_PX: f32 = 1.3;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    // A JANELA de 4 pontos em SCREEN-SPACE (bottom-up, como `to_screen`), FLAT no
    // quad: o segmento (p1→p2) e os dois VIZINHOS (p0, p3). O fragment mede a
    // distância à POLILINHA local (mín. das 3 cápsulas) — ver o cabeçalho.
    // Sentinela de borda: `ss_p0 == ss_p1` (sem prev) / `ss_p3 == ss_p2` (sem next);
    // como os varyings são FLAT (sem interpolação), a igualdade é EXATA e o
    // `capsule_dn` reconhece a cápsula degenerada. Um port que interpole estes
    // varyings quebra a sentinela.
    @location(0) @interpolate(flat) ss_p1: vec2<f32>,
    @location(1) @interpolate(flat) ss_p2: vec2<f32>,
    @location(2) color: vec4<f32>,   // cor por-ponto interpolada
    @location(3) opacity: f32,
    @location(4) thickness: f32,      // espessura REAL em px (SEM o clamp mínimo), só para o fade sub-pixel
    @location(5) @interpolate(flat) hardness: f32,
    @location(6) @interpolate(flat) ss_p0: vec2<f32>,
    @location(7) @interpolate(flat) ss_p3: vec2<f32>,
    // Raios em PX dos 4 pontos da janela: (r_p0, r_p1, r_p2, r_p3).
    @location(8) @interpolate(flat) radii: vec4<f32>,
    // (offset, count) dos vizinhos GEOMÉTRICOS deste segmento em `seg_extras`.
    @location(9) @interpolate(flat) extras: vec2<u32>,
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

// Normalização que NUNCA devolve NaN: um ponto DUPLICADO no traço (o tablet repete
// uma amostra; o smooth/simplify funde dois) faz `normalize(0)` explodir e o quad
// inteiro vira NaN — um buraco no traço. Devolve `false` quando o vetor é nulo,
// e o chamador trata como "sem vizinho" (perpendicular reta, sem miter).
fn safe_dir(v: vec2<f32>, out_dir: ptr<function, vec2<f32>>) -> bool {
    let len_sq = dot(v, v);
    if (len_sq < 1e-12) {
        return false;
    }
    *out_dir = v * inverseSqrt(len_sq);
    return true;
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
    let p_prev = points[prev_gp];
    let p_next = points[nn_gp];
    let sa = to_screen(a.pos);
    let sb = to_screen(b.pos);
    let sp = to_screen(p_prev.pos);
    let sn = to_screen(p_next.pos);
    // Raios CLAMPADOS ao mínimo rasterizável (geometria + máscara). A espessura
    // real (não-clampada) vai separada no varying `thickness`, para o fade
    // sub-pixel do fragment — ver MIN_WIDTH_PX.
    let min_r = MIN_WIDTH_PX * 0.5;
    let r_a = max(a.width * 0.5 * cam.px_per_world, min_r);
    let r_b = max(b.width * 0.5 * cam.px_per_world, min_r);
    let r_prev = max(p_prev.width * 0.5 * cam.px_per_world, min_r);
    let r_next = max(p_next.width * 0.5 * cam.px_per_world, min_r);
    // Espessura REAL em px (sem clamp), interpolada a→b: quanto o traço "deveria"
    // ter de largura. O fragment a usa só para o fade.
    let raw_w_a = a.width * cam.px_per_world;
    let raw_w_b = b.width * cam.px_per_world;

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
    var d_prev = vec2<f32>(0.0, 0.0);
    if (prev_gp != gp && safe_dir(sa - sp, &d_prev)) {
        if (-dot(dir, d_prev) > MITER_BREAK_COS) {
            ext_a = r_a;
        } else {
            var m_tan = vec2<f32>(0.0, 0.0);
            if (safe_dir(d_prev + dir, &m_tan)) {
                miter_a = perp(m_tan);
                scale_a = 1.0 / max(dot(m_tan, d_prev), MITER_BREAK_COS);
            } else {
                ext_a = r_a; // 180° exato: a bissetriz não existe — trata como quebra
            }
        }
    }
    var miter_b = n_seg;
    var scale_b = 1.0;
    var d_next = vec2<f32>(0.0, 0.0);
    if (nn_gp != next_gp && safe_dir(sn - sb, &d_next)) {
        if (-dot(dir, d_next) > MITER_BREAK_COS) {
            ext_b = r_b;
        } else {
            var m_tan = vec2<f32>(0.0, 0.0);
            if (safe_dir(dir + d_next, &m_tan)) {
                miter_b = perp(m_tan);
                scale_b = 1.0 / max(dot(m_tan, d_next), MITER_BREAK_COS);
            } else {
                ext_b = r_b;
            }
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
        out.thickness = raw_w_b;
    } else {
        corner = sa - dir * ext_a + miter_a * (side * r_a * scale_a);
        out.color = a.color;
        out.opacity = a.opacity;
        out.thickness = raw_w_a;
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
    // A janela de vizinhos para a união local do fragment. SENTINELA de borda: sem
    // prev/next (extremo de traço aberto), o vizinho COINCIDE com o próprio extremo
    // → a cápsula degenera e o `capsule_dn` a ignora. Traço FECHADO já tem
    // `prev_gp`/`nn_gp` com wrap, então a costura ganha a janela de graça.
    out.ss_p0 = select(sa, sp, prev_gp != gp);
    out.ss_p3 = select(sb, sn, nn_gp != next_gp);
    out.radii = vec4<f32>(r_prev, r_a, r_b, r_next);
    // O segmento é identificado pelo seu ponto INICIAL (`gp`) — o mesmo índice que
    // a CPU usou ao preencher `seg_extra_range` (pack.rs).
    out.extras = seg_extra_range[gp];
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
    // AA da borda = a FRAÇÃO DO PIXEL coberta pelo disco: a distância (com sinal) do
    // pixel à borda, medida em pixels (`aa = fwidth(dn)` é o tamanho de 1 px em
    // unidades de `dn`), somada a meio pixel. Em `dn = 1` dá 0.5 — meio pixel
    // coberto, que é a resposta certa. A forma antiga (`1 - smoothstep(1-aa, 1, dn)`)
    // SUBESTIMAVA a cobertura quando o traço é fino (`aa > 1`): a linha de 1 px saía
    // 10× mais fraca do que devia, e o par clamp+fade não conseguia salvá-la.
    let edge = clamp(0.5 + (1.0 - dn) / aa, 0.0, 1.0);
    return profile * edge;
}

// Distância NORMALIZADA (0 = centro, 1 = borda) do pixel à cápsula `a`→`b` de raios
// `ra`/`rb` — o raio efetivo é o interpolado pelo `t` CLAMPADO. **Uma função, três
// chamadas** (própria + os 2 vizinhos): se o segmento próprio usasse o `thickness`
// interpolado no QUAD (que inclui as extensões de `miter_break`/cap), com largura
// por-ponto (pressão!) os dois quads que cobrem um mesmo pixel normalizariam por
// raios DIFERENTES — o invariante quebraria de novo e a mordida sobreviveria em 2ª
// ordem. Cápsula degenerada (sentinela de borda: p0==p1) devolve "infinito" → o
// `min` a ignora.
fn capsule_dn(frag: vec2<f32>, a: vec2<f32>, b: vec2<f32>, ra: f32, rb: f32) -> f32 {
    let ab = b - a;
    let len_sq = dot(ab, ab);
    if (len_sq < 1e-6) {
        return 1e9;
    }
    let t = clamp(dot(frag - a, ab) / len_sq, 0.0, 1.0);
    let d = length(frag - a - t * ab);
    return d / max(mix(ra, rb, t), 1e-4);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Pixel em SCREEN-SPACE bottom-up (o `@builtin(position)` é top-down → flip Y
    // p/ casar `to_screen`).
    let frag = vec2<f32>(in.clip.x, cam.viewport.y - in.clip.y);
    // A UNIÃO LOCAL da polilinha: a MENOR distância normalizada entre a cápsula do
    // próprio segmento e as dos dois vizinhos. O perfil de hardness é monótono
    // decrescente em `dn`, então min-distância ⇔ max-cobertura — os dois quads que
    // se sobrepõem numa quina computam o MESMO valor e o depth first-wins volta a
    // ser invisível. (Sem isto, o quad anterior pintava a sua queda RADIAL sobre o
    // NÚCLEO do seguinte: a "mordida". Ver o cabeçalho + docs/Flip/03.)
    // NOTA DE FRONTEIRA: tudo chega por varying FLAT — os storage buffers têm
    // `visibility: VERTEX` (pipeline.rs); buscar `points[]` aqui exigiria mudar a BGL.
    let dn_own = capsule_dn(frag, in.ss_p1, in.ss_p2, in.radii.y, in.radii.z);
    let dn_prev = capsule_dn(frag, in.ss_p0, in.ss_p1, in.radii.x, in.radii.y);
    let dn_next = capsule_dn(frag, in.ss_p2, in.ss_p3, in.radii.z, in.radii.w);
    var dn = min(dn_own, min(dn_prev, dn_next));
    // Os vizinhos GEOMÉTRICOS (não-adjacentes) que a CPU descobriu para ESTE
    // segmento — o que estende a união local à UNIÃO GLOBAL da polilinha e mata a
    // mordida de longo alcance (traço que volta sobre si mesmo). `count` é 0 num
    // traço que não se auto-aproxima: o laço nem executa.
    for (var k = 0u; k < in.extras.y; k = k + 1u) {
        let e = seg_extras[in.extras.x + k];
        let ea = points[e.x];
        let eb = points[e.y];
        dn = min(dn, capsule_dn(
            frag,
            to_screen(ea.pos),
            to_screen(eb.pos),
            max(ea.width * 0.5 * cam.px_per_world, 0.0),
            max(eb.width * 0.5 * cam.px_per_world, 0.0),
        ));
    }
    // `dn` é contínuo (os campos coincidem onde trocam de dono), então o `fwidth`
    // do AA só vê um salto de DERIVADA na costura do `min` — nunca um degrau.
    let aa = max(fwidth(dn), 1e-4);
    var mask = hardness_mask(dn, in.hardness, aa);
    // Fade SUB-PIXEL (`gpencil_frag.glsl:534`): um traço mais fino que um pixel não
    // "afina" — ele perde OPACIDADE. Sem isto, a linha fina pisca e serrilha ao
    // mover/zoomar (o rasterizador acerta ou erra o centro do pixel); com isto, a
    // energia é preservada e a linha desbota suavemente. `thickness` é a espessura
    // em px de tela, sem clamp.
    mask *= smoothstep(0.0, 1.0, in.thickness);
    // Ghost Frames: a SILHUETA da arte, chapada na cor do lado (verde = passado,
    // azul = futuro) e esmaecida pelo fade. A cobertura (`mask`) é a mesma — só a
    // cor e o alpha mudam, então o fantasma tem exatamente a forma do desenho.
    var rgb = in.color.rgb;
    var a = in.color.a * in.opacity;
    if (cam.ghost_tint.a > 0.0) {
        rgb = cam.ghost_tint.rgb;
        a = a * cam.ghost_tint.a;
    }
    let alpha = a * mask;
    // Fragmento ~transparente é DESCARTADO (GP `gpencil_frag.glsl`: a < 0.001) —
    // senão ele ESCREVE depth e fura a geometria sobreposta que chega depois (o
    // canto vazio do quad viraria um buraco no traço vizinho).
    if (alpha < 0.001) {
        discard;
    }
    // Saída PREMULTIPLICADA (blend = One, OneMinusSrcAlpha).
    return vec4<f32>(rgb * alpha, alpha);
}
