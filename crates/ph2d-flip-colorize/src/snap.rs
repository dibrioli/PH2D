//! **O snap da borda ao EIXO** (5º smoke, 2026-07-20 — o serrilhado do handoff §3.1).
//!
//! # O defeito que este módulo cura
//!
//! A borda de uma região colorida que corre AO LONGO de uma linha é vetorizada por marching
//! squares + RDP (`simplify_ring`), e numa linha de mão a amplitude do tremor senta
//! exatamente no ε do RDP (1,25 px). Nesse regime a simplificação cai, por sorte de anel, num
//! de dois resultados — uma **corda reta que ignora a onda** ou um **zigue-zague com vértices
//! só nos extremos** — e o zigue-zague fica visível sob a saia translúcida de um pincel
//! macio: os "dentes de serra regulares" do 5º smoke, num lado só, com o outro liso
//! (medido na sonda `probe_the_sawtooth_boundary_along_the_divider`: o plano `assign` e o
//! plano de pixels pós-`expand_under_ink` são lisos e SIMÉTRICOS — rugosidade 0,2 px,
//! idêntica nos dois lados; a assimetria nasce inteira na vetorização).
//!
//! # A lei
//!
//! **Um trecho de borda que está sobre a tinta pertence ao EIXO da linha** — a mesma âncora
//! zoom-proof do balde (BUGS #14). Depois do `simplify_ring`, cada segmento do anel é
//! amostrado a ~1 px; toda amostra a ≤ [`SNAP_PX`] do eixo de uma linha VISÍVEL é cravada no
//! ponto exato do eixo (`nearest_on_axis_indexed` — a MESMA busca que a dilatação usa; uma
//! segunda resposta a "onde está o eixo?" divergiria). O que está longe da tinta — a
//! fronteira de Voronoi num vão aberto, a lente — fica onde está: a corda é a forma honesta
//! ali.
//!
//! ⚠️ **Cravar não basta: é preciso CAMINHAR o eixo.** A projeção euclidiana pula o fundo de
//! um V — do lado côncavo de uma dobra, o ponto mais próximo de toda amostra salta por cima
//! do fundo (medido: amostras em y=2,158/2,146 cravavam em q's a y=2,175/2,136, e a dobra em
//! y≈2,16 ficava sem ponto nenhum — 0,98 px de resíduo, SÓ do lado côncavo; o convexo segue a
//! dobra de graça, e é por isso que a doença sobrevivia de um lado só). Quando duas cravadas
//! consecutivas caem no MESMO traço em segmentos não-adjacentes, os vértices do próprio eixo
//! entre elas são inseridos — a borda passa a SER a polilinha da linha.
//!
//! # A borda NÃO fica no eixo: ela atravessa até a face OPOSTA (6º smoke)
//!
//! A v1 deste módulo cravava as duas bordas EXATAMENTE no eixo — a mesma curva a 0,03 px,
//! medido — e o 6º smoke mostrou um ZÍPER fino na costura: dois polígonos ADJACENTES
//! rasterizam a aresta compartilhada cada um por si, e as diferenças de quantização de
//! 1 px alternam (dentes das duas cores + riscas da linha por baixo). ⚠️ O produto SHIPADO
//! nunca teve isso porque o crave (`expand_under_ink`) levava cada fill ATRAVÉS da tinta
//! até a face oposta — **os fills se sobrepunham sob a linha**, e sobreposição não tem
//! costura. A v1 destruiu essa sobreposição ao "corrigir" as bordas para o eixo.
//!
//! A lei inteira: **a borda cravada segue a FORMA do eixo, deslocada [`PUSH_PX`] além dele
//! pela normal externa** ([`ph2d_flip_fill::outward_normals`] — a MESMA convenção da
//! dilatação). Cada lado invade ~2 px o lado alheio, sob a linha; a costura é coberta duas
//! vezes e nenhuma quantização a abre. É a topologia do crave, com a forma do eixo — sem
//! os dentes do RDP (5º smoke) e sem a costura aberta (6º).
//!
//! # Por que DEPOIS do RDP, e não antes
//!
//! Cravar o anel denso e simplificar depois devolveria o problema: o RDP a ε = 1,25 px
//! re-polygonalizaria a onda cravada em extremos de novo. A ordem é simplificar (barato,
//! anel leve), cravar + caminhar (a fidelidade volta só onde há eixo) e **decimar com ε
//! pequeno** ([`DECIMATE_EPS_PX`]) — que só colapsa as corridas colineares de volta aos
//! vértices do próprio eixo.
//!
//! O pré-filtro é a EDT que a partição já computou (`Segmentation::ink_dist2`) — recomputar
//! distância à tinta aqui seria uma 2ª porta para a mesma pergunta (`segment.rs`).

use ph2d_core::Vec2;
use ph2d_flip_fill::{Grid, nearest_on_axis_indexed, simplify_ring};

/// Raio de captura do snap, em px da grade. Cobre o pior desvio de vetorização de um trecho
/// colado na linha: o resíduo do crave (`expand_under_ink`, ±1,5 px medido na sonda) + o ε
/// do RDP (1,25) + o RUÍDO do traço de mão (±1,3 px na taxa do ponteiro — o 6º smoke) ⇒
/// ~4,1 px no pior caso; um vértice cru que escapa da captura vira um ESPINHO solitário na
/// costura. Um ponto além de 5 px não está "na linha" — está numa fronteira de papel
/// aberto, e cravá-lo mentiria a forma (o pedágio de aperto já mantém as fronteiras
/// contestadas fora dessa banda).
const SNAP_PX: f32 = 5.0;

/// Passo da amostragem dos segmentos do anel, em px da grade. Menor que [`SNAP_PX`] para a
/// corda não pular por cima de uma dobra do eixo. 1 px, e não 2: com 2 px um segmento de
/// 3,9 px ficava SEM amostra interna (truncamento) e a corda cortava a dobra do eixo em
/// ~1 px — medido no gate do serrilhado (1,06 px de resíduo).
const STEP_PX: f32 = 1.0;

/// ε da decimação final, em px da grade — colapsa SÓ o exatamente colinear (as amostras
/// cravadas sobre um segmento do eixo), e **nunca corta uma quina do eixo** — nem a de
/// RUÍDO. ⚠️ 0,3 foi o zíper do 6º smoke: numa linha de MÃO o eixo tem quinas de ruído de
/// ~1,3 px, e um ε que as poda decide DIFERENTE em cada anel (o RDP é recursivo, as
/// âncoras diferem) ⇒ os dois lados da costura divergem em até 2ε e a diferença alterna —
/// os dentes finos e regulares da foto, com riscas brancas onde nenhum fill cobre. As duas
/// bordas só são a MESMA curva se a decimação preservar TODA quina da polilinha do eixo.
const DECIMATE_EPS_PX: f32 = 0.05;

/// O EMPURRÃO além do eixo, em px da grade — a sobreposição sob a linha (ver o topo).
/// 2 px engole qualquer diferença de quantização de rasterização; o cap por meia-espessura
/// (`0.6 · w/2`) mantém o empurrão DEBAIXO da linha visível numa linha fina.
const PUSH_PX: f32 = 2.0;
const PUSH_MAX_HALF_FRACTION: f32 = 0.6;

/// Teto (px da grade) da distância entre duas cravadas consecutivas para o caminhar-do-eixo
/// ligar uma à outra. O pulo-do-V medido é ~3 px; um par mais distante que isto não é uma
/// dobra pulada — é a borda saindo da linha e voltando (a boca do vão, a lente), e ali
/// ligar pelo eixo REESCREVERIA uma fronteira legítima de papel aberto.
const WALK_MAX_PX: f32 = 8.0;

/// Onde uma amostra cravou: o traço, o segmento da polilinha dele e a ESPESSURA da linha
/// ali (o cap do empurrão é fração da meia-espessura).
///
/// ⚠️ Um vértice inserido pelo caminhar carrega o SEU segmento (`seg = k`), nunca o do
/// par que disparou o caminho — a perp do segmento errado subcortava a mitra da quina
/// (medido: dev caía a 0,24 px onde a lei pede ~2). Com o endereço local, a perp do
/// próprio segmento basta (a bissetriz foi testada e não mudou o resultado).
#[derive(Clone, Copy, PartialEq)]
struct Address {
    stroke: usize,
    seg: usize,
    width: f32,
}

/// Crava no eixo os trechos do anel que correm sobre a tinta e CAMINHA o eixo entre
/// cravadas do mesmo traço (ver o topo do módulo).
///
/// `ink_dist2` é a EDT da partição (distância² de cada pixel à tinta); `strokes` é a MESMA
/// lista de fronteiras que o `colorize` recebeu — a busca só veste linha com espessura
/// (`w > 0`), então fechamentos invisíveis não atraem a borda.
pub(crate) fn snap_ring_to_axis(
    strokes: &[(Vec<Vec2>, Vec<f32>, bool)],
    ink_dist2: &[u32],
    grid: &Grid,
    ring: &[Vec2],
) -> Vec<Vec2> {
    if ring.len() < 3 {
        return ring.to_vec();
    }
    let snap_doc = SNAP_PX / grid.scale;
    let step_doc = STEP_PX / grid.scale;
    // O pré-filtro barato: a EDT já responde "há tinta por perto?" sem varrer as linhas.
    // A folga de +1,5 px cobre a parede (BOUNDARY tem +0,5 de AA) e o arredondamento de
    // pixel — o filtro só pode errar para o lado de PERGUNTAR (o exato decide).
    let pre = ((SNAP_PX + 1.5) * (SNAP_PX + 1.5)) as u32;
    let near_ink = |p: Vec2| -> bool {
        grid.pixel_of(p)
            .is_some_and(|(x, y)| ink_dist2[y * grid.w + x] <= pre)
    };
    let snap = |p: Vec2| -> Option<(Vec2, Address)> {
        if !near_ink(p) {
            return None;
        }
        let (width, q, stroke, seg) = nearest_on_axis_indexed(strokes, p)?;
        let (dx, dy) = (q.x - p.x, q.y - p.y);
        (dx * dx + dy * dy <= snap_doc * snap_doc).then_some((q, Address { stroke, seg, width }))
    };

    // Passo 1: amostra cada segmento do anel a ~1 px; amostra cravada entra COM o endereço
    // (uma corda em papel aberto continua uma corda — amostra crua não acrescenta forma).
    let n = ring.len();
    let mut out: Vec<(Vec2, Option<Address>)> = Vec::with_capacity(n * 2);
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + 1) % n]);
        match snap(a) {
            Some((q, ad)) => out.push((q, Some(ad))),
            None => out.push((a, None)),
        }
        let d = b - a;
        let len = (d.x * d.x + d.y * d.y).sqrt();
        // `ceil`, nunca truncar: truncando, um segmento de até 2·passo ficava sem amostra
        // interna e a corda saltava uma dobra inteira do eixo.
        let steps = (len / step_doc).ceil().max(1.0) as usize;
        for s in 1..steps {
            let t = s as f32 / steps as f32;
            if let Some((q, ad)) = snap(Vec2::new(a.x + d.x * t, a.y + d.y * t)) {
                out.push((q, Some(ad)));
            }
        }
    }

    // Passo 2: CAMINHA o eixo entre cravadas consecutivas do mesmo traço (o pulo-do-V).
    let walk_max = WALK_MAX_PX / grid.scale;
    let m = out.len();
    let mut walked: Vec<(Vec2, Option<Address>)> = Vec::with_capacity(m * 2);
    for i in 0..m {
        let (p, ad) = out[i];
        walked.push((p, ad));
        let (pn, ad_next) = out[(i + 1) % m];
        let (Some(a), Some(b)) = (ad, ad_next) else {
            continue;
        };
        if a.stroke != b.stroke || a.seg == b.seg {
            continue; // traços diferentes = junção; mesmo segmento = nada entre eles
        }
        let (dx, dy) = (pn.x - p.x, pn.y - p.y);
        if dx * dx + dy * dy > walk_max * walk_max {
            continue; // longe demais: é a borda saindo da linha, não uma dobra pulada
        }
        let pts = &strokes[a.stroke].0;
        // Os vértices do eixo entre os dois segmentos, na direção do par. (O teto de
        // distância acima limita o caminho a uma vizinhança — num traço fechado o lado
        // curto é o único que cabe nele.)
        let mut vert = |k: usize| {
            walked.push((
                pts[k % pts.len()],
                Some(Address {
                    stroke: a.stroke,
                    seg: k % pts.len(),
                    width: a.width,
                }),
            ));
        };
        if a.seg < b.seg {
            for k in (a.seg + 1)..=b.seg {
                vert(k);
            }
        } else {
            for k in ((b.seg + 1)..=a.seg).rev() {
                vert(k);
            }
        }
    }

    // Passo 3: o EMPURRÃO até a face oposta (6º smoke — ver o topo).
    //
    // ⚠️ A DIREÇÃO vem do EIXO, nunca da cadeia: a normal da cadeia (diferença central
    // sobre pontos cravados + vértices caminhados) é ruidosa, e um empurrão de 2 px ao
    // longo de uma normal ruidosa ESPALHA a cadeia em ±2 px — pior que o defeito
    // (medido: a cadeia empurrada retrocedia em y e o anel se autocruzava na ponta do
    // toco, abrindo um furo de winding). A perpendicular do SEGMENTO do eixo é limpa; a
    // normal externa do anel ([`ph2d_flip_fill::outward_normals`], a convenção da
    // dilatação) entra só como SINAL — de que lado da linha este anel está.
    let positions: Vec<Vec2> = walked.iter().map(|(p, _)| *p).collect();
    let normals = ph2d_flip_fill::outward_normals(&positions);
    let mut pushed: Vec<Vec2> = Vec::with_capacity(walked.len());
    for (i, (p, ad)) in walked.iter().enumerate() {
        match ad {
            Some(a) => {
                let push = (PUSH_PX / grid.scale)
                    .min(PUSH_MAX_HALF_FRACTION * a.width * 0.5)
                    .max(0.0);
                let pts = &strokes[a.stroke].0;
                let (sa, sb) = (pts[a.seg], pts[(a.seg + 1) % pts.len()]);
                let d = Vec2::new(sb.x - sa.x, sb.y - sa.y);
                let len = (d.x * d.x + d.y * d.y).sqrt();
                if len <= 1e-9 {
                    pushed.push(*p);
                    continue;
                }
                let perp = Vec2::new(-d.y / len, d.x / len);
                let side = normals[i].x * perp.x + normals[i].y * perp.y;
                if side.abs() < 1e-3 {
                    pushed.push(*p); // normal degenerada: fica no eixo
                    continue;
                }
                let dir = if side >= 0.0 { 1.0 } else { -1.0 };
                pushed.push(Vec2::new(
                    p.x + perp.x * push * dir,
                    p.y + perp.y * push * dir,
                ));
            }
            None => pushed.push(*p),
        }
    }

    // Amostras vizinhas cravadas no mesmo ponto do eixo colapsam; depois a decimação
    // devolve as corridas colineares aos vértices do próprio eixo. `smooth = 0`: alisar
    // aqui moveria os pontos PARA FORA da curva que se acabou de cravar.
    let min_gap = 0.05 / grid.scale;
    pushed.dedup_by(|a, b| {
        let d = Vec2::new(a.x - b.x, a.y - b.y);
        (d.x * d.x + d.y * d.y).sqrt() < min_gap
    });
    simplify_ring(&pushed, DECIMATE_EPS_PX / grid.scale, 0)
}
