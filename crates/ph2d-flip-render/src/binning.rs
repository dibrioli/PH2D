//! **O BINNING POR TILE** — o esqueleto do motor de traço novo
//! ([doc 12](../../../docs/Flip/12_novo_motor_pesquisa.md) §6.3, passo 2).
//!
//! O motor de HOJE responde *"que caminho está perto deste fragmento?"* com um **canal lateral
//! de tamanho FIXO** (`neighbors.rs`: `MAX_EXTRAS_PER_SEGMENT = 16`) e elege UM fragmento por
//! pixel com um **depth test**. As duas coisas são as propriedades (B) e (C) do §3 do handoff, e
//! são elas que prendem o traço na lei atual.
//!
//! Aqui a pergunta muda de dono: a **tela** é dividida em ladrilhos, e cada ladrilho carrega a
//! **lista dos segmentos que o alcançam** — limitada por MEMÓRIA, nunca por uma constante. O
//! consumidor percorre essa lista **em ordem de z**, e é isso que apaga (B) e (C) de uma vez:
//!
//! - não há teto: a lista é um `Vec` concatenado com `[offset, count]` por ladrilho;
//! - não há eleição: dentro de um traço a lei de acúmulo é comutativa (passo 3), e ENTRE traços
//!   a ordem é o `sid`, que é explícito no percurso em vez de codificado numa profundidade.
//!
//! ⚠️ **A granularidade é MEDIDA, não escolhida** (doc 12 §6): a bbox de um traço diagonal é a
//! tela inteira (**76,9× a fita**, 67 telas cheias por frame com 200 gestos) e um ladrilho de 64
//! desperdiça em traço fino (16,5 telas); o ladrilho de **16** custa ~3× a fita nas duas cenas.
//!
//! ⚠️ **O que este módulo NÃO faz, de propósito:** não tem lei de tinta. O `walk_pixel` daqui
//! resolve a **união dura** (`dist <= r`), que é a semântica de `hardness = 1` sem anti-aliasing
//! — o CONTROLE de todos os smokes (§8 do handoff). A integral `τ` entra no passo 3, **dentro
//! deste mesmo percurso**; o esqueleto existe para que a lei nova nasça já sem teto e sem depth.
//!
//! Puro (sem wgpu) — testável headless, como o `pack.rs`.

use crate::pack::{FLAG_CLOSED, FlipGpuData};
use crate::pipeline::CameraRaw;

/// O lado do ladrilho em px. **16** é o número medido no doc 12 §6.2 (~3× a fita, contra 8,7-13×
/// do ladrilho de 64); também é exatamente um workgroup de 16×16 quando isto virar compute.
pub const DEFAULT_TILE: u32 = 16;

/// A largura mínima rasterizável, em px. ⚠️ **Tem de bater com o `MIN_WIDTH_PX` do `flip.wgsl`**
/// — um binner que use um raio menor que o do consumidor DROPA segmentos que ele quer, e a tinta
/// some sem erro nenhum. O gate `the_min_width_matches_the_shader` lê o WGSL e compara.
pub const MIN_WIDTH_PX: f32 = 1.3;

/// Uma entrada da lista de um ladrilho: qual traço, e o par de pontos do segmento.
///
/// O `stroke` viaja **inline** em vez de sair de `point_stroke[a]`: o percurso é por-PIXEL, então
/// uma indireção aqui é relida uma vez por pixel do ladrilho. 12 bytes, sem padding.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BinSeg {
    /// Índice do traço em `strokes` — o **z** (maior fica por cima) e a fonte de `hardness`/flags.
    pub stroke: u32,
    /// Índice GLOBAL do primeiro ponto em `points`.
    pub a: u32,
    /// Índice GLOBAL do segundo ponto. Num traço fechado a costura é `(último, primeiro)`, então
    /// `b` **não** é sempre `a + 1` — por isso os dois viajam explícitos.
    pub b: u32,
}

/// A grade + as listas concatenadas.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TileBins {
    /// Lado do ladrilho em px.
    pub tile: u32,
    /// Colunas e linhas da grade (cobrindo o viewport inteiro).
    pub cols: u32,
    pub rows: u32,
    /// `cols * rows` entradas `[offset, count]` em [`Self::segs`], em ordem row-major.
    pub ranges: Vec<[u32; 2]>,
    /// A lista concatenada. Dentro de um ladrilho a ordem é **(traço crescente, segmento
    /// crescente)** — o percurso depende disso para agrupar por traço sem ordenar nada.
    pub segs: Vec<BinSeg>,
}

impl TileBins {
    /// O índice do ladrilho que contém o pixel, ou `None` se ele cai fora da grade.
    #[must_use]
    pub fn tile_of_pixel(&self, px: f32, py: f32) -> Option<usize> {
        if px < 0.0 || py < 0.0 || self.tile == 0 {
            return None;
        }
        let tx = (px as u32) / self.tile;
        let ty = (py as u32) / self.tile;
        (tx < self.cols && ty < self.rows).then(|| (ty * self.cols + tx) as usize)
    }

    /// A lista daquele ladrilho.
    #[must_use]
    pub fn segs_of(&self, tile_index: usize) -> &[BinSeg] {
        let Some(&[off, count]) = self.ranges.get(tile_index) else {
            return &[];
        };
        &self.segs[off as usize..(off + count) as usize]
    }
}

/// O mapa mundo → pixels de tela. Espelha os três números da [`CameraRaw`]; existe para que o
/// binner (CPU) e o consumidor respondam *"onde este ponto cai?"* pela MESMA aritmética.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ScreenSpace {
    /// Mundo → clip (o mesmo afim do uniform).
    pub world_to_clip: [[f32; 4]; 4],
    /// Alvo em px.
    pub viewport: [f32; 2],
    /// Px por unidade de mundo (a escala de espessura).
    pub px_per_world: f32,
}

impl ScreenSpace {
    /// A partir da câmera do passe — a porta única, para os dois nunca discordarem.
    #[must_use]
    pub fn from_camera(cam: &CameraRaw) -> Self {
        Self {
            world_to_clip: cam.world_to_clip,
            viewport: cam.viewport,
            px_per_world: cam.px_per_world,
        }
    }

    /// Um ponto de mundo em px de tela. Espelha o `to_screen` do `flip.wgsl`.
    #[must_use]
    pub fn point_px(&self, w: [f32; 2]) -> [f32; 2] {
        let m = &self.world_to_clip;
        // Coluna-major, como o WGSL: `clip = M * (x, y, 0, 1)`.
        let cx = m[0][0] * w[0] + m[1][0] * w[1] + m[3][0];
        let cy = m[0][1] * w[0] + m[1][1] * w[1] + m[3][1];
        let cw = m[0][3] * w[0] + m[1][3] * w[1] + m[3][3];
        let inv = if cw == 0.0 { 1.0 } else { 1.0 / cw };
        [
            (cx * inv * 0.5 + 0.5) * self.viewport[0],
            (cy * inv * 0.5 + 0.5) * self.viewport[1],
        ]
    }

    /// O raio em px de um ponto de largura `width` (mundo), **com o mesmo piso do shader**.
    #[must_use]
    pub fn radius_px(&self, width: f32) -> f32 {
        (width * 0.5 * self.px_per_world).max(MIN_WIDTH_PX * 0.5)
    }
}

// ————————————————————————————————— geometria —————————————————————————————————

/// Distância exata de um ponto a um segmento.
fn point_seg_distance(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (t, cx, cy) = closest_on_seg(p, a, b);
    let _ = t;
    ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt()
}

/// O ponto mais próximo do segmento, e o parâmetro `t ∈ [0,1]` onde ele cai.
fn closest_on_seg(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> (f32, f32, f32) {
    let (vx, vy) = (b[0] - a[0], b[1] - a[1]);
    let len2 = vx * vx + vy * vy;
    let t = if len2 <= 1e-12 {
        0.0
    } else {
        (((p[0] - a[0]) * vx + (p[1] - a[1]) * vy) / len2).clamp(0.0, 1.0)
    };
    (t, a[0] + vx * t, a[1] + vy * t)
}

/// Distância de um ponto a uma caixa alinhada (0 se dentro).
fn point_box_distance(p: [f32; 2], lo: [f32; 2], hi: [f32; 2]) -> f32 {
    let dx = (lo[0] - p[0]).max(p[0] - hi[0]).max(0.0);
    let dy = (lo[1] - p[1]).max(p[1] - hi[1]).max(0.0);
    (dx * dx + dy * dy).sqrt()
}

/// O segmento cruza a caixa? (clip Liang-Barsky — se sobra qualquer pedaço, cruza.)
fn seg_hits_box(a: [f32; 2], b: [f32; 2], lo: [f32; 2], hi: [f32; 2]) -> bool {
    let d = [b[0] - a[0], b[1] - a[1]];
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for i in 0..2 {
        if d[i].abs() < 1e-12 {
            if a[i] < lo[i] || a[i] > hi[i] {
                return false;
            }
        } else {
            let inv = 1.0 / d[i];
            let (mut ta, mut tb) = ((lo[i] - a[i]) * inv, (hi[i] - a[i]) * inv);
            if ta > tb {
                core::mem::swap(&mut ta, &mut tb);
            }
            t0 = t0.max(ta);
            t1 = t1.min(tb);
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

/// Distância EXATA de um segmento a uma caixa alinhada.
///
/// Sem interseção, o par mais próximo entre dois convexos inclui um vértice de um deles ⇒ o mínimo
/// está entre {4 quinas → segmento} e {2 pontas → caixa}. ⚠️ **O teste de interseção é
/// obrigatório**: um segmento que atravessa a caixa de lado a lado não tem ponta dentro nem quina
/// perto, e sem ele a distância sairia positiva sobre uma sobreposição.
fn seg_box_distance(a: [f32; 2], b: [f32; 2], lo: [f32; 2], hi: [f32; 2]) -> f32 {
    if seg_hits_box(a, b, lo, hi) {
        return 0.0;
    }
    let corners = [
        [lo[0], lo[1]],
        [hi[0], lo[1]],
        [lo[0], hi[1]],
        [hi[0], hi[1]],
    ];
    let mut best = f32::MAX;
    for c in corners {
        best = best.min(point_seg_distance(c, a, b));
    }
    best.min(point_box_distance(a, lo, hi))
        .min(point_box_distance(b, lo, hi))
}

// ————————————————————————————————— o binning —————————————————————————————————

/// Enumera os segmentos de um traço em ordem de caminho (a costura por último, se fechado).
fn stroke_segs(data: &FlipGpuData, sid: u32) -> impl Iterator<Item = (u32, u32)> + '_ {
    let s = data.strokes[sid as usize];
    let (first, count) = (s.first_point, s.point_count);
    let closed = s.flags & FLAG_CLOSED != 0;
    let open = (0..count.saturating_sub(1)).map(move |i| (first + i, first + i + 1));
    let seam = (closed && count >= 3).then(|| (first + count - 1, first));
    open.chain(seam)
}

/// **A PORTA.** Percorre os traços em ordem de `sid` e os segmentos em ordem de caminho,
/// depositando cada um nos ladrilhos que ele alcança.
///
/// ⚠️ **O alcance é medido da CAIXA do ladrilho, não do centro** — a caixa contém todo pixel do
/// ladrilho, então `dist(seg, caixa) <= r` já inclui todo segmento capaz de influenciar qualquer
/// pixel dali. É por isso que o percurso **não precisa de halo**: nenhuma lista de vizinho, nenhum
/// caso de borda.
///
/// A ordem de saída é (traço, segmento) **estável** por ladrilho: o `sid` cresce com o z e o
/// depósito é um counting-sort, que preserva a ordem de chegada dentro de cada balde.
#[must_use]
pub fn bin_segments(data: &FlipGpuData, screen: &ScreenSpace, tile: u32) -> TileBins {
    let tile = tile.max(1);
    let cols = (screen.viewport[0].ceil().max(0.0) as u32).div_ceil(tile);
    let rows = (screen.viewport[1].ceil().max(0.0) as u32).div_ceil(tile);
    let n_tiles = (cols as usize) * (rows as usize);
    let mut bins = TileBins {
        tile,
        cols,
        rows,
        ranges: vec![[0, 0]; n_tiles],
        segs: Vec::new(),
    };
    if n_tiles == 0 || data.strokes.is_empty() {
        return bins;
    }

    let mut pairs: Vec<(u32, BinSeg)> = Vec::new();
    for sid in 0..data.strokes.len() as u32 {
        for (a, b) in stroke_segs(data, sid) {
            let (pa, pb) = (data.points[a as usize], data.points[b as usize]);
            let sa = screen.point_px(pa.pos);
            let sb = screen.point_px(pb.pos);
            let r = screen.radius_px(pa.width).max(screen.radius_px(pb.width));
            // Rejeição barata: a faixa de ladrilhos da bbox da cápsula.
            let lo_x = ((sa[0].min(sb[0]) - r) / tile as f32).floor();
            let hi_x = ((sa[0].max(sb[0]) + r) / tile as f32).floor();
            let lo_y = ((sa[1].min(sb[1]) - r) / tile as f32).floor();
            let hi_y = ((sa[1].max(sb[1]) + r) / tile as f32).floor();
            let tx0 = lo_x.max(0.0) as u32;
            let ty0 = lo_y.max(0.0) as u32;
            let tx1 = (hi_x.max(0.0) as u32).min(cols.saturating_sub(1));
            let ty1 = (hi_y.max(0.0) as u32).min(rows.saturating_sub(1));
            if hi_x < 0.0 || hi_y < 0.0 || lo_x >= cols as f32 || lo_y >= rows as f32 {
                continue;
            }
            for ty in ty0..=ty1 {
                for tx in tx0..=tx1 {
                    let lo = [(tx * tile) as f32, (ty * tile) as f32];
                    let hi = [lo[0] + tile as f32, lo[1] + tile as f32];
                    if seg_box_distance(sa, sb, lo, hi) <= r {
                        pairs.push((ty * cols + tx, BinSeg { stroke: sid, a, b }));
                    }
                }
            }
        }
    }

    // Counting-sort por ladrilho: estável, O(n), e uma alocação só (o `Vec<Vec<_>>` ingênuo faria
    // uma por ladrilho — 8100 alocações a 1080p).
    let mut counts = vec![0u32; n_tiles];
    for (t, _) in &pairs {
        counts[*t as usize] += 1;
    }
    let mut off = 0u32;
    for (i, c) in counts.iter().enumerate() {
        bins.ranges[i] = [off, *c];
        off += c;
    }
    let mut cursor: Vec<u32> = bins.ranges.iter().map(|r| r[0]).collect();
    bins.segs = vec![BinSeg::default(); pairs.len()];
    for (t, s) in pairs {
        let i = &mut cursor[t as usize];
        bins.segs[*i as usize] = s;
        *i += 1;
    }
    bins
}

// ————————————————————————————————— o percurso —————————————————————————————————

/// O que um traço deposita num pixel, antes de compor: cobertura + cor linear straight-alpha.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Deposit {
    cover: f32,
    rgba: [f32; 4],
}

/// A contribuição de UM traço num pixel, dada a fatia da lista que pertence a ele.
///
/// ⚠️ **É AQUI que a lei mora, e ela é a integral de arco** ([`crate::tau`]): a MESMA lei que o
/// `flip.wgsl` já usa (`α = 1 − exp(−τ)`), com `τ` somado sobre o **caminho que existe** em vez de
/// sobre uma reta fictícia. Toda a estrutura à volta — binning, agrupamento, composição — foi
/// desenhada no passo 2 para não precisar mudar quando esta função mudasse.
fn stroke_deposit(
    run: &[BinSeg],
    data: &FlipGpuData,
    screen: &ScreenSpace,
    p: [f32; 2],
) -> Option<Deposit> {
    let hardness = data.strokes[run.first()?.stroke as usize].hardness;
    let (sd, near, dist) = stroke_silhouette(run, data, screen, p)?;
    // **O ANTI-ALIASING** — a fração do pixel coberta pela silhueta, por filtro-caixa.
    //
    // ⚠️ Espelha o `edge` do `flip.wgsl` TERMO A TERMO: lá é `clamp(0.5 + (1 − dn)/aa, 0, 1)` com
    // `aa = fwidth(dn)`; como `dn = d/r` e um pixel vale `1/r` em `dn`, o termo `(1 − dn)/aa` **é**
    // `r − d`, a distância com sinal em PIXELS. A mesma expressão aqui é `0.5 − sd`, e sem derivada
    // de tela nenhuma — o percurso tem os segmentos na mão, então o `min` sobre as passagens é
    // EXATO. (O shader precisa do `fwidth` de um `min`, que salta na costura, e por isso o AA de
    // lá é por-PASSAGEM; o próprio comentário dele registra o preço.)
    //
    // Ele fecha em `sd = 0,5`: além disso o pixel não é tocado, e sair aqui poupa a integral
    // inteira num pixel que ia devolver zero.
    let edge = 0.5 - sd;
    if edge <= 0.0 {
        return None;
    }
    // ⚠️ **O PERFIL É AMOSTRADO DENTRO DA SILHUETA, e é isto que apaga o caso especial do
    // shader.** Com o centro do pixel FORA (`sd > 0`) a integral não tem amostra nenhuma — a
    // janela de cada segmento é vazia — e `τ = 0`; a meia-borda de fora ficava em ZERO, e a
    // medição contra a área disse **−127/255**. O `flip.wgsl` escapa disso com um ramo
    // (`profile = 1.0` incondicional quando a borda é dura) e paga o preço de o perfil e o AA
    // serem duas leis. Empurrar o ponto para logo dentro da silhueta reproduz os DOIS regimes
    // com um mecanismo só: em dureza 1 o perfil ali é 1 (⇒ a máscara vira o `edge`, como no
    // shader) e num pincel macio ele já é ~0 (⇒ a máscara continua ~0, como no shader).
    // ⚠️ **Meio pixel, e o número TEM significado:** é a meia-largura do próprio filtro-caixa —
    // a média que ele quer é a do perfil na metade INTERNA do pixel. Varrido, `0,25 · 0,5 · 0,75 ·
    // 1,0` dão o mesmo (−9/255 contra a área) porque em dureza 1 o perfil é chapado; o que NÃO
    // funciona é empurrar de leve: com `1e-3` a corda amostrada fica quase tangente (meio
    // comprimento `√(2rε)` ≈ 0,12 px contra um passo de quadratura de 0,35) e a integral **não
    // pega amostra nenhuma** — medido, −98/255. O valor não foi afinado até passar; ele é o que
    // a geometria do filtro pede, e a varredura só confirmou que a vizinhança concorda.
    let p_eval = if sd > 0.0 && dist > 1e-6 {
        let f = (sd + 0.5) / dist;
        [p[0] + (near[0] - p[0]) * f, p[1] + (near[1] - p[1]) * f]
    } else {
        p
    };
    let (tau, rgba) = crate::tau::stroke_tau(run, data, screen, hardness, p_eval)?;
    let cover = (1.0 - (-tau).exp()) * edge.min(1.0);
    (cover > 0.0).then_some(Deposit { cover, rgba })
}

/// A silhueta do traço vista deste pixel: `(distância COM SINAL, ponto mais próximo, distância)`
/// — a com sinal é **negativa dentro**.
///
/// É o `min` sobre as passagens que o `flip.wgsl` também faz — só que aqui **EXATO**, porque o
/// percurso tem os segmentos na mão. O shader precisa estimar o tamanho de um pixel em unidades de
/// `dn` (`aa = fwidth(dn)`) e o próprio comentário dele registra o preço: sobre a UNIÃO o `fwidth`
/// mede o gradiente de um `min`, que **salta na costura**, e por isso o AA de lá é por-PASSAGEM.
/// Aqui não há derivada de tela envolvida, e por isso não há costura para saltar.
fn stroke_silhouette(
    run: &[BinSeg],
    data: &FlipGpuData,
    screen: &ScreenSpace,
    p: [f32; 2],
) -> Option<(f32, [f32; 2], f32)> {
    let mut best: Option<(f32, [f32; 2], f32)> = None;
    for seg in run {
        let (pa, pb) = (data.points[seg.a as usize], data.points[seg.b as usize]);
        let sa = screen.point_px(pa.pos);
        let sb = screen.point_px(pb.pos);
        let (t, cx, cy) = closest_on_seg(p, sa, sb);
        let dist = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt();
        let r = screen.radius_px(pa.width) * (1.0 - t) + screen.radius_px(pb.width) * t;
        let sd = dist - r;
        if best.is_none_or(|(prev, _, _)| sd < prev) {
            best = Some((sd, [cx, cy], dist));
        }
    }
    best
}

/// ⚠️ **A LEI DO PASSO 2, CONGELADA COMO ORÁCULO** — a união dura (`dist ≤ r`), que é a semântica
/// de `hardness = 1` sem anti-aliasing. Ela é o CONTROLE do §8 do handoff: em dureza 1 a integral
/// tem de reproduzi-la, e o gate mede em quantos pixels e por quanto elas discordam.
///
/// Sob `cfg(test)` porque não tem chamador de produção — um `pub` seria uma segunda resposta
/// esperando alguém chamá-la (a lição do `warp_axis`).
#[cfg(test)]
fn hard_union_deposit(
    run: &[BinSeg],
    data: &FlipGpuData,
    screen: &ScreenSpace,
    p: [f32; 2],
) -> Option<Deposit> {
    let mut best: Option<(f32, [f32; 4])> = None;
    for seg in run {
        let (pa, pb) = (data.points[seg.a as usize], data.points[seg.b as usize]);
        let sa = screen.point_px(pa.pos);
        let sb = screen.point_px(pb.pos);
        let (t, cx, cy) = closest_on_seg(p, sa, sb);
        let dist = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt();
        let r = screen.radius_px(pa.width) * (1.0 - t) + screen.radius_px(pb.width) * t;
        let signed = dist - r;
        if best.is_none_or(|(prev, _)| signed < prev) {
            let mut rgba = [0.0f32; 4];
            for (out, (ca, cb)) in rgba.iter_mut().zip(pa.color.iter().zip(&pb.color)) {
                *out = ca * (1.0 - t) + cb * t;
            }
            rgba[3] *= pa.opacity * (1.0 - t) + pb.opacity * t;
            best = Some((signed, rgba));
        }
    }
    let (signed, rgba) = best?;
    (signed <= 0.0).then_some(Deposit { cover: 1.0, rgba })
}

/// **O PERCURSO POR PIXEL.** Lê a lista do ladrilho, agrupa por traço (a lista já vem ordenada por
/// `sid`), resolve cada traço e compõe `over` em ordem de z. Devolve RGBA **premultiplicado**, a
/// convenção que o passe já usa.
///
/// ⚠️ **O agrupamento é um scan de RUN, não um mapa** — é a ordem estável do binning que o torna
/// correto, e é por isso que ela é parte do contrato do [`TileBins`].
#[must_use]
pub fn walk_pixel(
    bins: &TileBins,
    data: &FlipGpuData,
    screen: &ScreenSpace,
    p: [f32; 2],
) -> [f32; 4] {
    let Some(ti) = bins.tile_of_pixel(p[0], p[1]) else {
        return [0.0; 4];
    };
    walk_list(bins.segs_of(ti), data, screen, p)
}

/// O núcleo do percurso, sobre uma lista já escolhida. Compartilhado com o oráculo de força bruta
/// dos gates — as duas rotas TÊM de dar a mesma resposta, então só a LISTA pode diferir.
fn walk_list(list: &[BinSeg], data: &FlipGpuData, screen: &ScreenSpace, p: [f32; 2]) -> [f32; 4] {
    let mut acc = [0.0f32; 4];
    let mut i = 0;
    while i < list.len() {
        let sid = list[i].stroke;
        let mut j = i;
        while j < list.len() && list[j].stroke == sid {
            j += 1;
        }
        if let Some(d) = stroke_deposit(&list[i..j], data, screen, p) {
            let a = (d.rgba[3] * d.cover).clamp(0.0, 1.0);
            for (dst, src) in acc.iter_mut().zip(&d.rgba).take(3) {
                *dst = src * a + *dst * (1.0 - a);
            }
            acc[3] = a + acc[3] * (1.0 - a);
        }
        i = j;
    }
    acc
}

#[cfg(test)]
impl TileBins {
    /// **O ORÁCULO.** A MESMA resposta pela lista COMPLETA — o binning é estrutura de aceleração,
    /// então a única coisa que ele pode fazer de errado é mudar o resultado.
    ///
    /// Mora sob `cfg(test)` de propósito: sem chamador de produção, um `pub` seria uma segunda
    /// resposta esperando alguém chamá-la.
    fn walk_pixel_brute(data: &FlipGpuData, screen: &ScreenSpace, p: [f32; 2]) -> [f32; 4] {
        let mut all = Vec::new();
        for sid in 0..data.strokes.len() as u32 {
            for (a, b) in stroke_segs(data, sid) {
                all.push(BinSeg { stroke: sid, a, b });
            }
        }
        walk_list(&all, data, screen, p)
    }
}

#[cfg(test)]
#[path = "binning_tests.rs"]
mod tests;
