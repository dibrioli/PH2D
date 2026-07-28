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
// hardness é a do PAINTER — e é a do **TRAÇO** dele, não a de um **DAB** (o Painter
// carimba uma fileira de dabs e os compõe por `over`; ver `hardness_mask`). **NÃO** é a
// do GP. Com AA de ~1px por `fwidth` (o GP conta com MSAA; aqui não há).
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
//    de material, não o default. Aqui ele é o bit `FLAG_SELF_OVERLAP` por-traço
//    (`FlipStroke::self_overlap`, 03 §8): a depth vira por-SEGMENTO e as faces
//    sobrepostas passam a BLENDAR — a tinta escurece no cruzamento (opt-in).
//
//    ⚠️ **DEFEITO ABERTO E MEDIDO neste modo — `self_overlap` CONTA DUAS VEZES.**
//    Cada face que passa o depth computa a **UNIÃO GLOBAL** (o `min` sobre TODAS as
//    cápsulas alcançáveis), então as `N` faces sobrepostas compõem `1−(1−u)^N` com o
//    MESMO `u` — e `u` é a cobertura da passagem mais PRÓXIMA, creditada `N` vezes.
//    `N` é o número de QUADS sobrepostos, não o de PASSAGENS. A lei certa é a de
//    TINTA: `α = 1 − Π_passagem (1 − mask(min das cápsulas DAQUELA passagem))`.
//    Medido (`measure_the_self_overlap_double_count`, X macio, opacidade 0,5): erro de
//    até **43/255 (17%)**, e a assinatura é exata — todo pixel sai em `1−(1−OFF)²`,
//    inclusive onde a 2ª passagem está a `dn = 0,82` e deveria contribuir ~nada.
//    ⚠️ O default é `self_overlap: false`, então isto **não** é o artefato de
//    cruzamento que o pincel comum mostra (esse era a lista de vizinhos, já curada).
//    A cura pede a lista de extras PARTICIONADA por passagem (a fita local já é
//    separada desde o orçamento próprio) + depth de volta a por-TRAÇO — o que mata
//    junto a colisão de `f32` do degrau por-segmento em `sid` alto. Wave própria.
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
// BEVEL/MITER): a cobertura do fragment é a **UNIÃO** das cápsulas alcançáveis
// (`min` das distâncias normalizadas — o perfil é monótono decrescente, logo
// min-distância ⇔ max-cobertura). Na sobreposição de uma quina, a janela de A
// (`{prev,A,B}`) e a de B (`{A,B,next}`) contêm ambas `{A,B}` → computam o MESMO
// mínimo → o first-wins volta a ser invisível.
//
// ⚠️ **A união é POR PASSAGEM, e entre passagens a lei é COMPOSIÇÃO** (2026-07-28, 2ª foto do
// Enio): `min` de duas funções lisas tem VINCO na bissetriz do cruzamento — invisível em
// hardness 1 (máscara binária) e uma costura com pincel macio. Dois traços distintos nunca
// tiveram o problema porque o depth deles difere e o mais novo pinta POR CIMA, ou seja **já
// compõe**; um traço cruzando a si mesmo tem o MESMO depth e caía na união. Hoje a lista de
// vizinhos vem particionada (`neighbors::SegExtras`) e o fragment faz
// `1 − (1−mask_própria)(1−mask_estranha)` — a hipótese de cobertura independente, exatamente o
// que o `over` de dois traços produz. Medido: o desvio entre as duas rotas cai de **48/255**
// (hardness 0,4) e **35/255** (0,7) para **1/255**. Compõe-se a COBERTURA, nunca o ALFA: a
// opacidade multiplica depois, então um traço a opacity 0,5 segue sem escurecer sobre si mesmo.
//
// ⚠️ **A união é GLOBAL, não local — este parágrafo já mentiu e a correção importa.**
// Ele terminava afirmando *"teto conhecido: sobreposição com vizinhos i±2 e
// auto-cruzamento NÃO-adjacente seguem first-wins (semântica do GP, pinada em
// teste)"*. As três partes estavam erradas: a união alcança os vizinhos i±k E o
// auto-cruzamento não-adjacente (é para isso que existe `seg_extras`, o 4º binding
// deste shader, cheio por `neighbors.rs`), e **o teste que ela citava nunca foi
// escrito**. Um comentário que contradiz o código shipado é pior que comentário
// nenhum: ele mandou a investigação do artefato de cruzamento para a hipótese
// errada antes de a medição a derrubar.
//
// **O teto REAL, medido:** a lista de vizinhos é capeada (`MAX_RIBBON_EXTRAS` +
// o orçamento da grade em `neighbors.rs`), e a fita local tem orçamento SEPARADO
// justamente porque ela saturava a lista antes de qualquer cruzamento entrar —
// razão alcance/passo constante, ~6 vizinhos da própria passagem sempre presentes.
// Acima do teto, aqueles pixels voltam ao first-wins do GP.
// Spec + análise: `docs/Flip/03_traco_rasterizacao.md`.

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
    // A PONTA ao longo do traço (o *tip* pontilhado — pack.rs `tip_code`): 0=Continuous,
    // 1=Dots, 2=Squares. `dot_spacing` = o espaçamento centro-a-centro das contas como
    // MÚLTIPLO do diâmetro do traço (relativo à espessura, NÃO mundo absoluto — senão um
    // traço grosso funde as contas num borrão). `ref_width` = a espessura de referência do
    // traço (mundo, a largura MÁXIMA — pack.rs), o que dá a pitch escala com a grossura.
    tip: u32,
    dot_spacing: f32,
    ref_width: f32,
}

// Os códigos do *tip* — TÊM de bater com `pack.rs::tip_code`.
const TIP_DOTS: u32 = 1u;
const TIP_SQUARES: u32 = 2u;

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
// Comprimento de arco cumulativo (MUNDO) por-ponto: o *tip* pontilhado espaça as contas por
// arco. O vertex lê `arc_len[gp]` (início do segmento) e soma `|b−a|` p/ o fim.
@group(0) @binding(6) var<storage, read> arc_len: array<f32>;

const FLAG_CLOSED: u32 = 1u;
const FLAG_START_FLAT: u32 = 2u;
const FLAG_END_FLAT: u32 = 4u;
// Auto-sobreposição com ACÚMULO (`FlipStroke::self_overlap`, 03 §8) — TÊM de bater com
// `pack.rs::FLAG_SELF_OVERLAP`. Com o bit, a profundidade vira por-SEGMENTO (abaixo, no
// vertex) e as faces sobrepostas do mesmo traço BLENDam em vez de serem descartadas.
const FLAG_SELF_OVERLAP: u32 = 8u;
// Pincel AIRBRUSH analítico (`FlipStroke::airbrush`, 03 §8) — TÊM de bater com
// `pack.rs::FLAG_AIRBRUSH`. Com o bit, o `hardness_mask` troca o falloff do Painter (platô+`Smooth`)
// pela transmitância física de um dab esférico (Beer-Lambert). Lido no FRAGMENT (via o varying `flags`).
const FLAG_AIRBRUSH: u32 = 16u;
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
    // O *tip* pontilhado: `arc4` = o arco MUNDO nos 4 pontos da janela (p0,p1,p2,p3), medido
    // por offsets LOCAIS a partir de `arc_len[p1]` (contínuo através da costura de um anel).
    // O fragment lê o arco do PONTO MAIS PRÓXIMO da polilinha (união), não a projeção no
    // segmento próprio — senão a conta ganha um degrau na junção de um traço grosso. `tip` =
    // 0/1/2; `dot_spacing` = a pitch como MÚLTIPLO do diâmetro; `ref_width` = a espessura de
    // referência do traço (mundo). `Continuous` (tip 0) ignora tudo isto.
    @location(10) @interpolate(flat) arc4: vec4<f32>,
    @location(11) @interpolate(flat) tip: u32,
    @location(12) @interpolate(flat) dot_spacing: f32,
    @location(13) @interpolate(flat) ref_width: f32,
    // Os `FLAG_*` do traço, para o fragment testar (hoje só `FLAG_AIRBRUSH` — a máscara é
    // função da flag). FLAT: os bits chegam exatos, sem interpolação.
    @location(14) @interpolate(flat) flags: u32,
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
    //
    // **Self Overlap** (FLAG_SELF_OVERLAP, 03 §8): a profundidade vira por-SEGMENTO —
    // cada segmento ganha um degrau próprio DENTRO do slot do traço `[(2sid+2), (2sid+3))`
    // (acima do próprio fill em `2sid+1`, abaixo do próximo fill em `2sid+3`). As faces
    // sobrepostas de partes DIFERENTES do mesmo traço passam então o GREATER estrito e
    // BLENDam (premult over) em vez de serem descartadas → a tinta ESCURECE no cruzamento
    // (o `GP_STROKE_OVERLAP`, opção de material do GP). O degrau `li/count·1.9e-7 < 2e-7`
    // fica no slot; `Depth32Float` resolve o passo. Sem a flag: byte-idêntico (o ramo nem roda).
    out.clip = to_clip(corner);
    var z = f32(2u * sid + 2u) * 2e-7;
    if ((st.flags & FLAG_SELF_OVERLAP) != 0u) {
        z = z + f32(li) / f32(count) * 1.9e-7;
    }
    out.clip.z = z;
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
    // O *tip* pontilhado: o arco MUNDO nos 4 pontos da janela. `arc_p1` vem da CPU
    // (cumulativo); p0/p2/p3 são offsets LOCAIS a partir dele (as mesmas |Δ| do quad), então
    // o segmento de FECHO de um anel fecha certo em vez de saltar para zero, e um vizinho
    // ausente (sentinela p==p1) tem offset 0 e cápsula degenerada — o fragment o ignora.
    let arc_p1 = arc_len[gp];
    let arc_p0 = arc_p1 - length(a.pos - p_prev.pos);
    let arc_p2 = arc_p1 + length(b.pos - a.pos);
    let arc_p3 = arc_p2 + length(p_next.pos - b.pos);
    out.arc4 = vec4<f32>(arc_p0, arc_p1, arc_p2, arc_p3);
    out.tip = st.tip;
    out.dot_spacing = st.dot_spacing;
    out.ref_width = st.ref_width;
    out.flags = st.flags;
    return out;
}

// Perfil redondo: **a LEI DO PAINTER** (`BrushSpec::falloff_weight` + `Falloff::Smooth`) —
// núcleo CHEIO até `hardness`, e a curva `3p²−2p³` corre na faixa `[hardness, 1]`.
// `dn` = distância normalizada à linha-de-centro (0 centro, 1 borda). `aa` (~1px) fecha
// a borda com AA — o GP usa MSAA; aqui a cobertura é analítica, então o AA sai do `fwidth`.
//
// ⚠️ Isto NÃO é o Grease Pencil, e a divergência é DELIBERADA (Enio, 2026-07-28, com foto
// lado a lado: *"o correto é o aspecto do cruzamento de baixo [o Painter] e o flip deveria
// ser idêntico"*). O que morava aqui era o `gpencil_stroke_round_cap_mask` ao pé da letra —
// `smoothstep(0,1, pow(1−dn, mix(0,10, 1−hardness)))` — fiel ao Blender e **incompatível com
// o resto do app**: sem platô, o traço ENCOLHE ao amaciar (medido, o `dn` onde a tinta cruza
// meia-tinta: hardness 0,9 → 0,500 contra 0,951 do Painter · 0,7 → 0,207 contra 0,850 ·
// 0,5 → 0,130 contra 0,751). Em hardness 0,5 a largura VISÍVEL era 13% da pedida e o resto
// era névoa; a mesma palavra "Hardness" governava duas leis em dois módulos do mesmo app.
//
// ⚠️ `hardness ≥ 1` é BYTE-IDÊNTICO nas duas leis (disco duro), e `DEFAULT_HARDNESS = 1.0`
// ⇒ o traço padrão do Flip não se move. A paridade termo-a-termo com a função Rust REAL do
// Painter é gateada em `tests/hardness_law.rs` — a lei vive em dois idiomas e duas escritas
// de uma lei só divergem, então o oráculo é `ph2d_painter_brush`, nunca uma cópia local.
// Densidade óptica do airbrush no CENTRO (`k = 2·μ·R`): o slider Hardness a controla. Faixa
// ESTÉTICA (quão densa a névoa mais forte), não limite de recurso — a borda do airbrush é
// SEMPRE macia (mesmo em `k = K_MAX`, os últimos %% rolam suave a zero), que é a razão de ele
// existir ao lado do `pow`. `K_MIN` (hardness 0) = névoa tênue (centro `1−e^{−1}=0.63`);
// `K_MAX` (hardness 1) = domo largo quase sólido de borda macia.
const AIRBRUSH_K_MIN: f32 = 1.0;
const AIRBRUSH_K_MAX: f32 = 8.0;

// O passo da fileira de dabs do Painter, **em raios**: o `spacing` default dele é 0,10 do
// DIÂMETRO (`ph2d-painter-brush/src/spec_default.rs:29`) ⇒ 0,20 do raio.
const DEPOSIT_STEP: f32 = 0.2;
// Meia-largura do laço. Um dab a `|k|·STEP ≥ 1` está fora do disco para qualquer `dn ≥ 0`
// (`d = √(dn² + along²) ≥ along`), então `k = 5` já não contribui: 4 basta, e o `if` interno
// cobre o resto sem depender deste número estar apertado.
const DEPOSIT_HALF: i32 = 4;

fn hardness_mask(dn: f32, hardness: f32, aa: f32, airbrush: bool) -> f32 {
    var profile: f32;
    if (airbrush) {
        // Airbrush analítico (Ciallo, 03 §8): a transmitância de Beer-Lambert por um dab
        // esférico. A corda pela esfera no deslocamento `dn` é `2R·√(1−dn²)`, a tinta absorve
        // exponencialmente ao longo dela ⇒ cobertura `A = 1 − exp(−k·√(1−dn²))`. Domo largo de
        // núcleo chato e borda sempre macia (em `dn=1` a corda some ⇒ `A=0`, sem degrau). O
        // `max(…,0)` guarda o `√` de `dn` fora do disco (fragmento descartado de todo jeito).
        let k = mix(AIRBRUSH_K_MIN, AIRBRUSH_K_MAX, clamp(hardness, 0.0, 1.0));
        profile = 1.0 - exp(-k * sqrt(max(1.0 - dn * dn, 0.0)));
    } else if (hardness > 0.999) {
        profile = 1.0; // borda dura: o núcleo é chapado; a borda é o AA abaixo
    } else {
        // **O PERFIL É O DO TRAÇO DO PAINTER, NÃO O DE UM DAB DELE.**
        //
        // O Painter não pinta um dab: ele carimba uma FILEIRA deles a cada `spacing × diâmetro`
        // de arco e os compõe por `over`. O que o artista vê na tela é o PRODUTO, e ele é muito
        // mais cheio que a queda de um dab sozinho — medido em hardness 0,4: em `dn = 0,70` um
        // dab pesa **0,500** e o traço pesa **0,916**. Era essa a distância entre as duas fotos
        // do Enio (2026-07-28): com a queda do dab, a cunha escura da quina media **−138 de 255**
        // contra o depósito real; com esta, **−43**.
        //
        // ⚠️ **Isto NÃO reintroduz dependência de amostragem** (a doença que esta linha curou
        // quatro vezes): `DEPOSIT_STEP` é uma propriedade do PINCEL DO PAINTER (o `spacing`
        // default dele, `spec_default.rs:29`), não de quão fino o motor amostrou o caminho. A
        // máscara continua sendo função PURA da distância ao caminho.
        //
        // ⚠️ **A fase da grade de dabs é IRRELEVANTE e isso foi medido**, não suposto: deslocar
        // a fileira de meio passo move o perfil em **0,003** (sonda `deposit_profile`). Por isso
        // a fase é 0 e não há ondulação a modelar.
        //
        // ⚠️ **Em `hardness ≥ 1` os dois modelos são a MESMA função** (todo dab é disco duro ⇒
        // o produto satura em `dn < 1` e some em `dn ≥ 1`), e `DEFAULT_HARDNESS` é 1 ⇒ o traço
        // padrão do Flip não se move um bit. O ramo acima já o resolve sem entrar no laço.
        let h = clamp(hardness, 0.0, 1.0);
        var keep = 1.0;
        for (var k = -DEPOSIT_HALF; k <= DEPOSIT_HALF; k = k + 1) {
            let along = f32(k) * DEPOSIT_STEP;
            let d = sqrt(dn * dn + along * along);
            if (d < 1.0) {
                // A queda de UM dab: `BrushSpec::falloff_weight` + o preset `Falloff::Smooth`,
                // com as MESMAS operações na MESMA ordem que o Rust — "termo a termo".
                let remapped = clamp((d - h) / (1.0 - h), 0.0, 1.0);
                let p = 1.0 - remapped;
                keep = keep * (1.0 - (3.0 * p * p - 2.0 * p * p * p));
            }
        }
        profile = 1.0 - keep;
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

// Como `capsule_dn`, mas devolve `vec2(dn, arco)` — o arco MUNDO no ponto MAIS PRÓXIMO da
// cápsula (o `t` clampado interpola entre `arc_a` e `arc_b`). O *tip* pontilhado escolhe o
// arco da cápsula de menor `dn` (a que de fato cobre o pixel), e não a projeção no segmento
// próprio: dois quads adjacentes que se sobrepõem numa junção contêm as MESMAS cápsulas na
// janela ⇒ concordam no vencedor ⇒ o arco fica CONTÍNUO na costura, e a conta não ganha
// degrau num traço grosso (o report "pontos deformados em linhas grossas"). Cápsula
// degenerada (sentinela de borda) devolve `1e9` e nunca vence.
fn capsule_dn_arc(
    frag: vec2<f32>,
    a: vec2<f32>,
    b: vec2<f32>,
    ra: f32,
    rb: f32,
    arc_a: f32,
    arc_b: f32,
) -> vec2<f32> {
    let ab = b - a;
    let len_sq = dot(ab, ab);
    if (len_sq < 1e-6) {
        return vec2<f32>(1e9, arc_a);
    }
    let t = clamp(dot(frag - a, ab) / len_sq, 0.0, 1.0);
    let d = length(frag - a - t * ab);
    return vec2<f32>(d / max(mix(ra, rb, t), 1e-4), mix(arc_a, arc_b, t));
}

// O PONTO (tela) do centro de conta no arco `sc`, interpolado no segmento `a`→`b` cujos
// extremos estão nos arcos `arc_a`/`arc_b`. Como a câmera é uniforme, a fração de arco é a
// fração de tela. Clampado — se `sc` cai fora do segmento, devolve o extremo (o chamador só
// usa o segmento que de fato contém `sc`; o clamp é sanidade). Devolve `xy` = ponto, `zw` =
// direção UNITÁRIA do segmento (para orientar o quadrado; reta degenerada → +x).
fn bead_point(a: vec2<f32>, b: vec2<f32>, arc_a: f32, arc_b: f32, sc: f32) -> vec4<f32> {
    let f = clamp((sc - arc_a) / max(arc_b - arc_a, 1e-6), 0.0, 1.0);
    let p = a + (b - a) * f;
    var dir = vec2<f32>(1.0, 0.0);
    _ = safe_dir(b - a, &dir);
    return vec4<f32>(p, dir);
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
    //
    // ⚠️ O piso `min_r` é o MESMO das cápsulas própria/anterior/seguinte (`min_r` no
    // vertex): "qual é o raio desta cápsula?" é UMA pergunta, e este laço já foi a
    // segunda porta que esquecia a regra (media com piso `0.0`). O regime em que isso
    // morde é o SUB-PIXEL — que não é exótico, é todo traço depois de um zoom out: a
    // cápsula do CRUZAMENTO ficava mais fina que a da própria fita, então a união
    // media MENOS cobertura exatamente onde as duas passagens se encontram.
    //
    // ⚠️ **E a lista vem PARTICIONADA POR PASSAGEM** (`neighbors::SegExtras`): os primeiros
    // `n_ribbon` são a PRÓPRIA fita, o resto são OUTRAS passagens (o traço que voltou). A 2ª
    // palavra do range carrega os dois números (count nos 16 baixos, ribbon nos altos).
    // Ver o §"UMA PASSAGEM, UMA COBERTURA" abaixo para o porquê de eles não se misturarem.
    let min_r = MIN_WIDTH_PX * 0.5;
    let n_all = in.extras.y & 0xffffu;
    let n_ribbon = min(in.extras.y >> 16u, n_all);
    var dn_ribbon = dn;
    var dn_cross = 1e9;
    for (var k = 0u; k < n_all; k = k + 1u) {
        let e = seg_extras[in.extras.x + k];
        let ea = points[e.x];
        let eb = points[e.y];
        let d = capsule_dn(
            frag,
            to_screen(ea.pos),
            to_screen(eb.pos),
            max(ea.width * 0.5 * cam.px_per_world, min_r),
            max(eb.width * 0.5 * cam.px_per_world, min_r),
        );
        if (k < n_ribbon) {
            dn_ribbon = min(dn_ribbon, d);
        } else {
            dn_cross = min(dn_cross, d);
        }
    }
    dn = min(dn_ribbon, dn_cross);
    // **O *tip* pontilhado** (Dots/Squares, 03 §8): a linha vira CONTAS — cada uma um DISCO
    // (ou QUADRADO) EUCLIDIANO de raio = a meia-espessura, centrado num ponto da linha-de-centro
    // a cada `pitch = dot_spacing × ref_width` de ARCO. A pitch é RELATIVA À ESPESSURA (múltiplo
    // do diâmetro), não uma distância de mundo fixa — senão um traço grosso funde as contas num
    // borrão (1º report do Enio, 2026-07-25). ⚠️ A conta é a distância EUCLIDIANA `|frag − C|` ao
    // PONTO-centro `C`, **não** `√(dn² + da_arco²)`: o arco curva, então a métrica mista esticava
    // a conta numa banana ao longo da curva (2º report do Enio: *"pontos deformados em linhas
    // grossas"* — medido: o L a 90° e a senoide saíam idênticos com/sem o arco de junção, então
    // não era a costura, era o LENS de arco). Com o centro como ponto 2D e distância reta, a
    // conta fica redonda em qualquer curva. `Continuous` (tip 0) NÃO toca em `dn` ⇒ byte-idêntico
    // à linha cheia. A depth é por-TRAÇO, então contas que se sobrepõem numa quina são first-wins.
    if (in.tip != 0u && in.dot_spacing > 0.0) {
        // O arco `s` do PONTO MAIS PRÓXIMO da polilinha (união prev/own/next), não a projeção
        // no segmento próprio — o vencedor de menor `dn` é o mesmo nos quads que se sobrepõem
        // numa junção, então `s` (e o centro de conta escolhido) é CONTÍNUO na costura.
        var best = capsule_dn_arc(
            frag, in.ss_p1, in.ss_p2, in.radii.y, in.radii.z, in.arc4.y, in.arc4.z);
        let cp = capsule_dn_arc(
            frag, in.ss_p0, in.ss_p1, in.radii.x, in.radii.y, in.arc4.x, in.arc4.y);
        if (cp.x < best.x) { best = cp; }
        let cn = capsule_dn_arc(
            frag, in.ss_p2, in.ss_p3, in.radii.z, in.radii.w, in.arc4.z, in.arc4.w);
        if (cn.x < best.x) { best = cn; }
        let s = best.y;
        // A pitch. `max(…,1e-6)` cobre `ref_width == 0` (largura zero, que nem renderiza).
        let pitch = max(in.dot_spacing * in.ref_width, 1e-6);
        // O arco do centro de conta mais próximo, e o PONTO 2D (tela) ali — no segmento da
        // janela que o contém (own / prev / next).
        let sc = round(s / pitch) * pitch;
        var c = bead_point(in.ss_p1, in.ss_p2, in.arc4.y, in.arc4.z, sc);
        if (sc < in.arc4.y) {
            c = bead_point(in.ss_p0, in.ss_p1, in.arc4.x, in.arc4.y, sc);
        } else if (sc > in.arc4.z) {
            c = bead_point(in.ss_p2, in.ss_p3, in.arc4.z, in.arc4.w, sc);
        }
        // Distância EUCLIDIANA (tela) à conta, normalizada pela meia-espessura LOCAL — a mesma
        // unidade que `dn`. O tamanho segue a espessura local; só a pitch é per-traço.
        let r = max(in.thickness * 0.5, 1e-4);
        let d = frag - c.xy;
        var dn_dot: f32;
        if (in.tip == TIP_SQUARES) {
            // Quadrado orientado à TANGENTE local (gira com o traço, como o pincel manda).
            let nrm = vec2<f32>(-c.w, c.z);
            dn_dot = max(abs(dot(d, c.zw)), abs(dot(d, nrm))) / r;
        } else {
            dn_dot = length(d) / r;
        }
        // A conta é o traço ∩ o disco: `max` = a MENOR cobertura (perfil decrescente em `dn`).
        // ⚠️ A interseção vale para CADA passagem, não só para a união — senão a passagem
        // que compõe abaixo entregaria cobertura fora da conta.
        dn_ribbon = max(dn_ribbon, dn_dot);
        dn_cross = max(dn_cross, dn_dot);
        dn = max(dn, dn_dot);
    }
    // `dn` é contínuo (os campos coincidem onde trocam de dono), então o `fwidth`
    // do AA só vê um salto de DERIVADA na costura do `min` — nunca um degrau.
    let aa = max(fwidth(dn), 1e-4);
    // O AA é POR PASSAGEM: `fwidth(dn)` sobre a UNIÃO mede o gradiente do `min`, que salta no
    // vinco da bissetriz, e cada cobertura deve fechar a própria borda com o próprio gradiente
    // — que é liso. Calculados FORA do `if`: `fwidth` exige fluxo uniforme.
    //
    // ⚠️ **Honestidade sobre este trecho: ele foi MEDIDO INERTE nas fixtures desta wave** (a
    // sonda `measure_one_stroke_crossing_itself_against_two_strokes` não move um nível com ou
    // sem ele). Eu o escrevi atribuindo a ele um resíduo de 11 níveis em hardness 1.0 e a
    // medição REFUTOU a atribuição. Fica por princípio — não porque um número o comprove.
    let aa_ribbon = max(fwidth(dn_ribbon), 1e-4);
    let aa_cross = max(fwidth(dn_cross), 1e-4);
    // **UMA PASSAGEM, UMA COBERTURA — e elas COMPÕEM.**
    //
    // Tomar `hardness_mask(min(...))` sobre TODAS as passagens é a UNIÃO, e `min` de duas funções
    // lisas tem **VINCO** (gradiente descontínuo) na bissetriz do cruzamento. Com hardness 1 a
    // máscara é binária e o vinco não existe; com pincel macio ele é uma costura visível — o 2º
    // report do Enio (2026-07-28), e a razão de dois traços cruzados parecerem CERTOS e um traço
    // cruzando a si mesmo parecer ERRADO: com traços distintos o depth difere e o mais novo pinta
    // por cima, ou seja **já compõe**.
    //
    // Compor as coberturas — `1 − (1−a)(1−b)`, a hipótese de cobertura independente, exatamente o
    // que o `over` de dois traços produz — é liso e faz as duas rotas desenharem a mesma coisa.
    //
    // ⚠️ **Compõe-se a COBERTURA, nunca o ALFA**: a opacidade multiplica DEPOIS. No centro do
    // cruzamento as duas coberturas são 1, o composto é 1, e o alfa continua sendo `opacity` —
    // então um traço a opacity 0.5 **não escurece sobre si mesmo** (a regra do GP, *"the stroke
    // cannot overlap itself"*, que segue gateada). O que muda é só o OMBRO, onde a cobertura
    // é parcial e a união tinha o vinco.
    //
    // ⚠️ **Sem cruzamento é BYTE-IDÊNTICO por construção:** `n_ribbon == n_all` ⇒ o ramo nem roda,
    // e `dn_ribbon == dn`. Todo traço que não volta sobre si mesmo pinta o que sempre pintou.
    var mask = hardness_mask(dn_ribbon, in.hardness, aa_ribbon, (in.flags & FLAG_AIRBRUSH) != 0u);
    if (n_ribbon < n_all) {
        let m_cross =
            hardness_mask(dn_cross, in.hardness, aa_cross, (in.flags & FLAG_AIRBRUSH) != 0u);
        mask = 1.0 - (1.0 - mask) * (1.0 - m_cross);
    }
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
