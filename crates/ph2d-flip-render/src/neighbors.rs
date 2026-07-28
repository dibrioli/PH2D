//! Broadphase dos **vizinhos geométricos** de cada segmento — a peça que torna a
//! cobertura do traço a UNIÃO GLOBAL da polilinha num único passe.
//!
//! **O problema.** O fragment mede a cobertura como a distância à polilinha local.
//! Os vizinhos de sequência (`p0`/`p3`) chegam de graça (o vertex já os busca para
//! o miter) e fecham a classe "quina quebrada". Mas um traço pode voltar sobre si
//! mesmo — um zigzag apertado, um laço, uma letra — e aí o quad do segmento `i`
//! cobre pixels que pertencem ao NÚCLEO do segmento `j` com `|i-j| >= 2`. Como o
//! depth é first-wins, o segmento de índice MENOR vence e pinta a sua queda macia
//! por cima do núcleo do outro: a "mordida". A janela de sequência não vê `j`.
//!
//! **A solução (um passe, custo O(1) por fragmento).** Aqui, na CPU — dentro do
//! `pack`, que é **cacheado por desenho** (`TessCache` no shell) — descobrimos,
//! para cada segmento, quais segmentos NÃO-adjacentes podem influenciar os pixels
//! do seu quad, e emitimos essa lista curta para o shader. O fragment soma essas
//! cápsulas ao `min`. Na esmagadora maioria dos traços (linhas, arcos, curvas sem
//! retorno) a lista é VAZIA e o custo é zero.
//!
//! **O critério (conservador, sem falso-negativo).** Um pixel do quad de `i` está,
//! no máximo, a `2·r_i` do eixo de `i` (o esticão do miter é limitado a 2× pelo
//! `MITER_BREAK_COS`; a extensão de ponta idem). Para o segmento `j` influenciar
//! esse pixel, ele precisa alcançá-lo: `dist(pixel, j) < r_j`. Pela desigualdade
//! triangular, basta testar `dist(seg_i, seg_j) < 2·r_i + r_j`. O teste é
//! ASSIMÉTRICO (o raio do "dono do quad" entra dobrado), então cada direção é
//! avaliada por si.

use ph2d_core::Vec2;

/// Teto de vizinhos por segmento. Um traço patológico (rabisco denso rabiscado por
/// cima de si mesmo dezenas de vezes) não pode fazer o fragment iterar sem fim; os
/// candidatos são ordenados por proximidade, então o corte descarta os que menos
/// contribuem. Além do teto o traço volta a ter o first-wins do GP naqueles pixels
/// (o artefato histórico), nunca algo pior.
pub(crate) const MAX_EXTRAS_PER_SEGMENT: usize = 16;

/// Teto de TRABALHO do broadphase (pares candidatos examinados), por traço.
///
/// O `pack` do traço EM CURSO roda a cada frame (o preview ao vivo), então o custo
/// precisa de um teto duro. Um traço normal — por mais longo que seja — nem chega
/// perto disto (uma onda de 4000 pontos custa ~1.7 ms). Quem estoura é o caso
/// PATOLÓGICO: milhares de pontos rabiscados por cima de si mesmos num palmo de
/// tela, onde cada segmento tem centenas de vizinhos reais. Ali o teto entra e os
/// segmentos restantes ficam sem lista de extras — voltam ao first-wins do GP.
///
/// **A degradação é onde ela não importa:** esse traço é um borrão sólido de tinta
/// sobreposta; a mordida (uma borda macia sobre um núcleo) é invisível no meio dele.
/// O comportamento continua determinístico (mesmo desenho ⇒ mesmo buffer).
const PAIR_BUDGET: usize = 700_000;

/// Um segmento do traço, já resolvido em índices GLOBAIS de ponto.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Seg {
    /// Índice global do ponto inicial (é também a identidade do segmento no shader).
    pub a: u32,
    /// Índice global do ponto final (o wrap do traço fechado já resolvido).
    pub b: u32,
    pub pa: Vec2,
    pub pb: Vec2,
    /// O MAIOR raio dos dois pontos (px de mundo) — o lado conservador do teste.
    pub radius: f32,
}

/// Distância mínima entre dois segmentos 2D, **ao QUADRADO**. O `sqrt` era o custo
/// dominante — este laço roda milhões de vezes num rabisco denso — e a ordem de
/// comparação (e portanto o ranking dos vizinhos) é idêntica sem ele.
fn seg_seg_distance_sq(a1: Vec2, b1: Vec2, a2: Vec2, b2: Vec2) -> f32 {
    if segments_cross(a1, b1, a2, b2) {
        return 0.0;
    }
    // Disjuntos: o mínimo está sempre num dos 4 pares ponto-segmento.
    point_seg_distance_sq(a1, a2, b2)
        .min(point_seg_distance_sq(b1, a2, b2))
        .min(point_seg_distance_sq(a2, a1, b1))
        .min(point_seg_distance_sq(b2, a1, b1))
}

fn point_seg_distance_sq(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.x * ab.x + ab.y * ab.y;
    let d = if len_sq < 1e-9 {
        p - a
    } else {
        let t = (((p - a).x * ab.x + (p - a).y * ab.y) / len_sq).clamp(0.0, 1.0);
        p - (a + ab * t)
    };
    d.x * d.x + d.y * d.y
}

/// Rejeição barata (só comparações) antes da distância exata: as bboxes, expandidas
/// pelo alcance, nem se tocam.
fn bbox_far(si: &Seg, sj: &Seg, reach: f32) -> bool {
    si.pa.x.min(si.pb.x) - reach > sj.pa.x.max(sj.pb.x)
        || sj.pa.x.min(sj.pb.x) - reach > si.pa.x.max(si.pb.x)
        || si.pa.y.min(si.pb.y) - reach > sj.pa.y.max(sj.pb.y)
        || sj.pa.y.min(sj.pb.y) - reach > si.pa.y.max(si.pb.y)
}

/// Insere `(d2, j)` na lista dos `MAX_EXTRAS_PER_SEGMENT` mais próximos, mantida
/// ORDENADA por `(distância², índice)`. O desempate por índice é obrigatório: num
/// rabisco denso dezenas de segmentos cruzam o mesmo (distância 0), e sem ele o
/// corte dependeria da ordem de descoberta — o mesmo desenho geraria buffers
/// diferentes (determinismo é contrato do projeto: replay-hash).
///
/// Rejeita em O(1) quem é pior que o último — é o que evita ordenar centenas de
/// candidatos por segmento.
fn push_top(top: &mut Vec<(f32, u32)>, d2: f32, j: u32) {
    let worse_than = |a: (f32, u32), b: (f32, u32)| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).is_ge();
    if top.len() == MAX_EXTRAS_PER_SEGMENT && worse_than((d2, j), top[top.len() - 1]) {
        return;
    }
    let at = top.partition_point(|&e| !worse_than(e, (d2, j)));
    top.insert(at, (d2, j));
    top.truncate(MAX_EXTRAS_PER_SEGMENT);
}

fn cross2(o: Vec2, a: Vec2, b: Vec2) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

fn segments_cross(a1: Vec2, b1: Vec2, a2: Vec2, b2: Vec2) -> bool {
    let d1 = cross2(a1, b1, a2);
    let d2 = cross2(a1, b1, b2);
    let d3 = cross2(a2, b2, a1);
    let d4 = cross2(a2, b2, b1);
    (d1 * d2 < 0.0) && (d3 * d4 < 0.0)
}

/// Dois segmentos são ADJACENTES quando compartilham um ponto (inclusive pelo wrap
/// de um traço fechado). Estes já chegam ao fragment pela janela de sequência
/// (`p0`/`p3`) e não entram na lista de extras.
fn is_adjacent(si: &Seg, sj: &Seg) -> bool {
    si.a == sj.a || si.a == sj.b || si.b == sj.a || si.b == sj.b
}

/// Grid uniforme de células, para não pagar `O(n²)`: o traço EM CURSO é re-packado
/// a cada frame (o preview ao vivo), e um rabisco de milhares de pontos derrubaria
/// o frame-rate com o par-a-par ingênuo.
struct Grid {
    cell: f32,
    min: Vec2,
    cols: i32,
    rows: i32,
    /// Índices de segmento por célula (linear `row * cols + col`).
    buckets: Vec<Vec<u32>>,
}

impl Grid {
    /// Constrói um grid cuja célula acomoda o alcance de influência típico. O nº de
    /// células é limitado (a célula CRESCE se o traço for grande) — a memória fica
    /// bounded e o custo por consulta continua baixo.
    fn build(segs: &[Seg]) -> Self {
        const MAX_CELLS: i32 = 4096;
        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);
        let mut reach: f32 = 1.0;
        for s in segs {
            for p in [s.pa, s.pb] {
                min = Vec2::new(min.x.min(p.x), min.y.min(p.y));
                max = Vec2::new(max.x.max(p.x), max.y.max(p.y));
            }
            reach = reach.max(3.0 * s.radius); // `2·r_i + r_j` com raios comparáveis
        }
        let extent = Vec2::new((max.x - min.x).max(1.0), (max.y - min.y).max(1.0));
        let mut cell = reach.max(1.0);
        // Cresce a célula até caber no teto de memória.
        while ((extent.x / cell).ceil() + 1.0) * ((extent.y / cell).ceil() + 1.0)
            > f64::from(MAX_CELLS) as f32
        {
            cell *= 2.0;
        }
        let cols = (extent.x / cell).ceil() as i32 + 1;
        let rows = (extent.y / cell).ceil() as i32 + 1;
        let mut grid = Self {
            cell,
            min,
            cols,
            rows,
            buckets: vec![Vec::new(); (cols * rows) as usize],
        };
        // INSERÇÃO com pad `r_j` = o alcance da cápsula do próprio segmento.
        for (i, s) in segs.iter().enumerate() {
            let (c0, c1) = grid.cell_range(s, s.radius);
            for row in c0.1..=c1.1 {
                for col in c0.0..=c1.0 {
                    grid.buckets[(row * grid.cols + col) as usize].push(i as u32);
                }
            }
        }
        grid
    }

    /// As células que a bbox do segmento, expandida por `pad`, toca.
    ///
    /// **A assimetria é load-bearing** (um teste a guarda): o critério de vizinhança
    /// é `dist < 2·r_i + r_j` — o dono do quad conta o raio DOBRADO (o teto do
    /// esticão do miter) e o vizinho conta uma vez (o alcance da cápsula dele). Logo
    /// a INSERÇÃO usa pad `r_j` e a CONSULTA usa pad `2·r_i`: duas regiões que se
    /// tocam compartilham ao menos uma célula, então nenhum par escapa. Usar o mesmo
    /// pad nos dois lados PERDE vizinhos mais grossos que o dono (a mordida volta,
    /// silenciosamente, só naqueles pixels).
    fn cell_range(&self, s: &Seg, pad: f32) -> ((i32, i32), (i32, i32)) {
        let lo = Vec2::new(
            s.pa.x.min(s.pb.x) - pad - self.min.x,
            s.pa.y.min(s.pb.y) - pad - self.min.y,
        );
        let hi = Vec2::new(
            s.pa.x.max(s.pb.x) + pad - self.min.x,
            s.pa.y.max(s.pb.y) + pad - self.min.y,
        );
        let clamp = |v: f32, n: i32| ((v / self.cell).floor() as i32).clamp(0, n - 1);
        (
            (clamp(lo.x, self.cols), clamp(lo.y, self.rows)),
            (clamp(hi.x, self.cols), clamp(hi.y, self.rows)),
        )
    }
}

/// Para cada segmento de UM traço, a lista de segmentos não-adjacentes que podem
/// influenciar os pixels do seu quad (ordenada por proximidade, cortada em
/// [`MAX_EXTRAS_PER_SEGMENT`]).
///
/// Custo ~linear no nº de segmentos (grid uniforme). Roda no `pack`, que é
/// cacheado por desenho (`TessCache` no shell) — e a cada frame para o traço EM
/// CURSO, por isso o grid importa: um rabisco de milhares de pontos com o par-a-par
/// ingênuo custaria dezenas de ms.
pub(crate) fn extras_for_stroke(segs: &[Seg]) -> Vec<Vec<u32>> {
    let n = segs.len();
    let mut out = vec![Vec::new(); n];
    if n < 3 {
        return out; // nada não-adjacente existe
    }
    let closed = segs[n - 1].b == segs[0].a;
    let max_radius = segs.iter().fold(0.0f32, |m, s| m.max(s.radius));
    let grid = Grid::build(segs);
    let mut top: Vec<(f32, u32)> = Vec::with_capacity(MAX_EXTRAS_PER_SEGMENT + 1);
    // Dedup por GERAÇÃO (o segmento aparece em várias células da consulta): o carimbo
    // é o `i+1` da iteração — nada a limpar entre iterações, e a classe de bug
    // "limpeza incompleta do visitado" deixa de existir.
    let mut stamp = vec![0u32; n];
    let mut budget = PAIR_BUDGET;
    for i in 0..n {
        if budget == 0 {
            break; // teto de trabalho — ver PAIR_BUDGET
        }
        top.clear();
        let si = segs[i];
        let visit = i as u32 + 1;
        // A FITA LOCAL entra PRIMEIRO, com orçamento PRÓPRIO, e é carimbada — a
        // consulta do grid abaixo a pula, então os `MAX_EXTRAS_PER_SEGMENT` slots
        // ficam INTEIROS para os cruzamentos. Ver `push_ribbon_local`.
        push_ribbon_local(segs, i, closed, max_radius, &mut stamp, visit, &mut out[i]);
        // CONSULTA com pad `2·r_i` — o alcance dos pixels do quad deste segmento.
        let (c0, c1) = grid.cell_range(&si, 2.0 * si.radius);
        for row in c0.1..=c1.1 {
            for col in c0.0..=c1.0 {
                for &j in &grid.buckets[(row * grid.cols + col) as usize] {
                    let j = j as usize;
                    if j == i || stamp[j] == visit {
                        continue;
                    }
                    stamp[j] = visit;
                    budget = budget.saturating_sub(1);
                    let sj = segs[j];
                    if is_adjacent(&si, &sj) {
                        continue;
                    }
                    // Assimétrico: o raio do DONO do quad entra dobrado (o teto do
                    // esticão do miter), o do vizinho conta uma vez (o alcance da
                    // cápsula dele).
                    let reach = 2.0 * si.radius + sj.radius;
                    if bbox_far(&si, &sj, reach) {
                        continue;
                    }
                    let d2 = seg_seg_distance_sq(si.pa, si.pb, sj.pa, sj.pb);
                    if d2 < reach * reach {
                        push_top(&mut top, d2, j as u32);
                    }
                }
            }
        }
        out[i].extend(top.iter().map(|&(_, j)| j));
    }
    out
}

/// Teto de vizinhos da FITA LOCAL por segmento — um orçamento **separado** do
/// [`MAX_EXTRAS_PER_SEGMENT`] dos cruzamentos, e essa separação é a wave inteira.
///
/// **O que quebrou.** O alcance do broadphase é `2·r_i + r_j ≈ 3·r` e o passo da
/// reamostragem suave é `0.4 × largura = 0.8·r` (`flip_draw::resample_step`). As duas
/// grandezas são proporcionais ao raio, então a razão `alcance / passo = 3,75` é
/// **constante**: os vizinhos `i±1 … i±4` da PRÓPRIA fita caem dentro do alcance em
/// QUALQUER espessura de pincel, antes de existir cruzamento nenhum. Com um orçamento
/// único e ordenado por distância, eles — que estão a distância ~0 — ganhavam sempre, e
/// o segmento da passagem que de fato cruza era **cortado**. Aquele pixel voltava ao
/// first-wins e a GPU pintava a cauda macia de uma passagem sobre o NÚCLEO de outra
/// (medido: 520 px divergentes, pior desvio −127/255, numa hachura densificada).
///
/// **O valor é MEDIDO, não escolhido** (`measure_ribbon_budget`, rode com
/// `-- --nocapture`), na densidade que o `resample_smooth` de fato produz:
///
/// | cenário (raio 5, passo `0.8·r`) | máx | média |
/// |---|---|---|
/// | reta | 4 | 3,8 |
/// | arco raio 10·r | **12** | 9,8 |
/// | arco raio 2·r (curvatura alta) | 11 | 7,6 |
/// | arco raio 1·r (o limite do pincel) | 11 | 7,6 |
/// | hachura gap 0,5·r | 6 | 4,2 |
///
/// Pior caso do produto **12** ⇒ teto **16**, 33% de folga. ⚠️ **A degradação, nomeada:**
/// entrada 4× mais densa que o passo (mão LENTA numa curva, onde o RDP preserva pontos —
/// a reamostragem só ACRESCENTA, nunca remove) **satura**, e ali a lista guarda os 16 mais
/// próximos POR ARCO (8 de cada lado), que são os que mais contribuem. Isso nunca é pior
/// que o mundo pré-esta-wave: o orçamento dos CRUZAMENTOS
/// ([`MAX_EXTRAS_PER_SEGMENT`]) fica intacto, que é a razão de existir desta separação.
pub(crate) const MAX_RIBBON_EXTRAS: usize = 16;

/// Os vizinhos da MESMA PASSAGEM: os segmentos alcançáveis **andando pela polilinha**
/// a partir de `i` dentro do alcance de influência do quad dele.
///
/// **A definição se escreve sozinha e não tem constante mágica.** Um segmento que está
/// perto de `i` ou é (a) a fita continuando — e então o arco até ele é curto — ou (b)
/// o traço que foi embora e VOLTOU, isto é um cruzamento. O comprimento de arco separa
/// os dois exatamente, em qualquer densidade de amostragem e qualquer espessura: é a
/// mesma lei que esta linha já aplicou quatro vezes ao relevo — *a propriedade é do
/// CAMINHO, nunca de quão fino o motor amostrou o caminho*.
///
/// Os adjacentes (`i±1`) são pulados: chegam ao fragment pela janela de sequência
/// (`p0`/`p3`) e não gastam slot. Traço FECHADO dá a volta; aberto para nas pontas.
fn push_ribbon_local(
    segs: &[Seg],
    i: usize,
    closed: bool,
    max_radius: f32,
    stamp: &mut [u32],
    visit: u32,
    out: &mut Vec<u32>,
) {
    let n = segs.len();
    // Cota conservadora do alcance: o raio do dono entra DOBRADO (o teto do esticão do
    // miter) e o do vizinho é cotado pelo MAIOR do traço — o teste por-par abaixo é o
    // exato, este só decide quando PARAR de andar.
    let walk_reach = 2.0 * segs[i].radius + max_radius;
    stamp[i] = visit;
    for dir in [1i64, -1i64] {
        let mut arc = 0.0f32;
        let mut j = i as i64;
        for _ in 0..n {
            // O arco cresce pelo comprimento do segmento que acabamos de deixar para trás.
            let cur = &segs[j as usize];
            arc += (cur.pb - cur.pa).length();
            if arc > walk_reach {
                break;
            }
            j += dir;
            if j < 0 || j >= n as i64 {
                if !closed {
                    break;
                }
                j = j.rem_euclid(n as i64);
            }
            let ju = j as usize;
            if ju == i || stamp[ju] == visit {
                break; // deu a volta inteira num traço fechado curto
            }
            let sj = &segs[ju];
            if is_adjacent(&segs[i], sj) {
                stamp[ju] = visit; // adjacente: já vem pela janela `p0`/`p3`
                continue;
            }
            if arc > 2.0 * segs[i].radius + sj.radius {
                break; // fora do alcance REAL deste vizinho — e o arco só cresce
            }
            stamp[ju] = visit;
            out.push(ju as u32);
            if out.len() >= MAX_RIBBON_EXTRAS {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(a: u32, b: u32, pa: (f32, f32), pb: (f32, f32), r: f32) -> Seg {
        Seg {
            a,
            b,
            pa: Vec2::new(pa.0, pa.1),
            pb: Vec2::new(pb.0, pb.1),
            radius: r,
        }
    }

    #[test]
    fn a_straight_stroke_has_no_geometric_neighbors() {
        // Segmentos consecutivos SÃO adjacentes (a janela de sequência os cobre) e
        // os distantes não se aproximam: lista vazia = custo zero no fragment.
        let segs = [
            seg(0, 1, (0.0, 0.0), (20.0, 0.0), 2.0),
            seg(1, 2, (20.0, 0.0), (40.0, 0.0), 2.0),
            seg(2, 3, (40.0, 0.0), (60.0, 0.0), 2.0),
            seg(3, 4, (60.0, 0.0), (80.0, 0.0), 2.0),
        ];
        let extras = extras_for_stroke(&segs);
        assert!(
            extras.iter().all(Vec::is_empty),
            "reta não tem vizinho geométrico: {extras:?}"
        );
    }

    #[test]
    fn a_zigzag_that_folds_back_sees_the_far_segment() {
        // O 3º segmento passa RENTE ao 1º (2 px), com raio 5 — é a "mordida" de
        // longo alcance: os quads se sobrepõem mas os índices não são adjacentes.
        let segs = [
            seg(0, 1, (0.0, 0.0), (40.0, 0.0), 5.0),
            seg(1, 2, (40.0, 0.0), (10.0, 12.0), 5.0),
            seg(2, 3, (10.0, 12.0), (50.0, 3.0), 5.0),
        ];
        let extras = extras_for_stroke(&segs);
        assert_eq!(extras[0], vec![2], "o segmento 0 tem de VER o segmento 2");
        assert_eq!(
            extras[2],
            vec![0],
            "e vice-versa (o teste é simétrico aqui)"
        );
        assert!(extras[1].is_empty(), "o do meio é adjacente aos dois");
    }

    #[test]
    fn a_far_parallel_segment_is_not_a_neighbor() {
        // Mesmo traço voltando, mas LONGE (30 px com raio 2): fora do alcance de
        // qualquer pixel do quad → não entra na lista (o critério é conservador,
        // não paranoico).
        let segs = [
            seg(0, 1, (0.0, 0.0), (40.0, 0.0), 2.0),
            seg(1, 2, (40.0, 0.0), (40.0, 30.0), 2.0),
            seg(2, 3, (40.0, 30.0), (0.0, 30.0), 2.0),
        ];
        let extras = extras_for_stroke(&segs);
        assert!(extras[0].is_empty(), "30 px é longe demais: {extras:?}");
        assert!(extras[2].is_empty());
    }

    #[test]
    fn crossing_segments_are_neighbors() {
        // Um X: os segmentos 0 e 2 se CRUZAM (distância 0).
        let segs = [
            seg(0, 1, (0.0, 0.0), (40.0, 40.0), 4.0),
            seg(1, 2, (40.0, 40.0), (40.0, 0.0), 4.0),
            seg(2, 3, (40.0, 0.0), (0.0, 40.0), 4.0),
        ];
        let extras = extras_for_stroke(&segs);
        assert_eq!(extras[0], vec![2]);
        assert_eq!(extras[2], vec![0]);
    }

    /// A referência ingênua `O(n²)` — o oráculo do grid.
    /// O oráculo `O(n²)` do broadphase. ⚠️ **A fita local vem pela MESMA porta do
    /// produto** (`push_ribbon_local`) em vez de ser re-derivada aqui: o que este teste
    /// existe para provar é que o GRID não perde um CRUZAMENTO, e uma 2ª cópia da regra
    /// de fita transformaria uma divergência de partição num falso positivo (foi o que
    /// aconteceu quando a partição nasceu: o grid achava o vizinho `i±2` por arco e o
    /// oráculo o descartava por ranking de distância — o oráculo é que estava velho).
    fn brute_force(segs: &[Seg]) -> Vec<Vec<u32>> {
        let n = segs.len();
        let mut out = vec![Vec::new(); n];
        let closed = segs[n - 1].b == segs[0].a;
        let max_radius = segs.iter().fold(0.0f32, |m, s| m.max(s.radius));
        let mut stamp = vec![0u32; n];
        for i in 0..n {
            let visit = i as u32 + 1;
            push_ribbon_local(segs, i, closed, max_radius, &mut stamp, visit, &mut out[i]);
            let mut cand: Vec<(f32, u32)> = Vec::new();
            for j in 0..n {
                let (si, sj) = (segs[i], segs[j]);
                if i == j || is_adjacent(&si, &sj) || stamp[j] == visit {
                    continue;
                }
                let d2 = seg_seg_distance_sq(si.pa, si.pb, sj.pa, sj.pb);
                let reach = 2.0 * si.radius + sj.radius;
                if d2 < reach * reach {
                    cand.push((d2, j as u32));
                }
            }
            cand.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            cand.truncate(MAX_EXTRAS_PER_SEGMENT);
            out[i].extend(cand.iter().map(|&(_, j)| j));
        }
        out
    }

    #[test]
    fn the_grid_finds_exactly_what_the_pairwise_scan_finds() {
        // Um rabisco denso que se auto-cruza muito (LCG determinístico — HR-5: sem
        // transcendental, sem RNG do sistema). Se o grid perder um vizinho, um pixel
        // do traço volta a ter a mordida — e este teste é a única barreira.
        let mut state: u32 = 0x1234_5678;
        let mut rnd = || {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 8) as f32 / f32::from(u16::MAX) % 1.0
        };
        let mut segs = Vec::new();
        let mut prev = Vec2::new(32.0, 32.0);
        for k in 0..180u32 {
            let next = Vec2::new(rnd() * 60.0 + 2.0, rnd() * 60.0 + 2.0);
            let r = 2.0 + rnd() * 6.0;
            segs.push(seg(k, k + 1, (prev.x, prev.y), (next.x, next.y), r));
            prev = next;
        }
        let by_grid = extras_for_stroke(&segs);
        let by_pairs = brute_force(&segs);
        assert_eq!(
            by_grid.len(),
            by_pairs.len(),
            "mesma quantidade de segmentos"
        );
        for (i, (g, p)) in by_grid.iter().zip(&by_pairs).enumerate() {
            let (mut g, mut p) = (g.clone(), p.clone());
            g.sort_unstable();
            p.sort_unstable();
            assert_eq!(g, p, "segmento {i}: o grid divergiu do par-a-par");
        }
        assert!(
            by_pairs.iter().any(|v| !v.is_empty()),
            "o rabisco tem de produzir vizinhos (senão o teste não prova nada)"
        );
    }

    #[test]
    fn the_extras_list_is_capped_and_sorted_by_proximity() {
        // Um "pente" onde o segmento 0 é rente a MUITOS outros: o corte mantém os
        // mais próximos (o fragment nunca itera sem teto).
        let mut segs = vec![seg(0, 1, (0.0, 0.0), (100.0, 0.0), 6.0)];
        for k in 0..40u32 {
            let y = 1.0 + (k as f32) * 0.1; // todos rentes, cada vez mais longe
            let base = 2 + k * 2;
            segs.push(seg(
                base,
                base + 1,
                (0.0, y),
                (100.0, y),
                1.0, // raio pequeno: adjacência só pelo raio do dono (2·6 = 12)
            ));
        }
        let extras = extras_for_stroke(&segs);
        assert_eq!(extras[0].len(), MAX_EXTRAS_PER_SEGMENT, "a lista é capada");
        // Os escolhidos são os mais PRÓXIMOS (os primeiros do pente).
        assert!(
            extras[0].contains(&1) && extras[0].contains(&2),
            "os mais próximos entram: {:?}",
            extras[0]
        );
    }
}

#[cfg(test)]
mod ribbon_budget_measurement {
    use super::*;

    /// Constrói os segmentos de uma polilinha com raio uniforme (índices globais
    /// sequenciais, como o `pack` faz para um traço aberto).
    fn segs_of(pts: &[(f32, f32)], r: f32) -> Vec<Seg> {
        (0..pts.len() - 1)
            .map(|i| Seg {
                a: i as u32,
                b: i as u32 + 1,
                pa: Vec2::new(pts[i].0, pts[i].1),
                pb: Vec2::new(pts[i + 1].0, pts[i + 1].1),
                radius: r,
            })
            .collect()
    }

    /// Reamostra no passo do produto (`0.4 × largura = 0.8 · r`).
    fn densify(pts: &[(f32, f32)], step: f32) -> Vec<(f32, f32)> {
        let mut out = vec![pts[0]];
        for w in pts.windows(2) {
            let (a, b) = (w[0], w[1]);
            let d = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let n = (d / step).floor() as usize;
            for k in 1..=n {
                let t = k as f32 * step / d;
                if t < 1.0 {
                    out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
                }
            }
            out.push(b);
        }
        out
    }

    /// **A MEDIÇÃO que fixa `MAX_RIBBON_EXTRAS`.** Roda com
    /// `cargo test -p ph2d-flip-render ribbon_budget -- --nocapture`.
    ///
    /// A pergunta: quantos vizinhos da PRÓPRIA fita caem dentro do alcance de
    /// influência, na densidade que o produto de fato produz? O teto tem de ficar
    /// acima do pior caso REAL — e o pior caso não é uma reta, é a curvatura máxima
    /// que um pincel consegue desenhar (o raio de curvatura chega ao raio do pincel;
    /// abaixo disso a fita dobra sobre si mesma e vira cruzamento, não fita).
    #[test]
    fn measure_ribbon_budget() {
        const R: f32 = 5.0;
        let step = 0.8 * R;
        // ⚠️ Os arcos são gerados **no passo do produto** (comprimento de ARCO), não por
        // grau: uma varredura por grau num raio grande fica 4,6× mais densa que o que o
        // `resample_smooth` de fato produz, e mediria a fixture em vez do produto.
        let arc = |curv_r: f32, sweep_deg: f32| -> Vec<(f32, f32)> {
            let total = curv_r * sweep_deg.to_radians();
            let n = (total / step).max(2.0) as usize;
            (0..=n)
                .map(|k| {
                    let a = (k as f32 / n as f32) * sweep_deg.to_radians();
                    (curv_r * a.cos(), curv_r * a.sin())
                })
                .collect()
        };
        let cases: Vec<(&str, Vec<(f32, f32)>, bool)> = vec![
            ("reta", vec![(0.0, 0.0), (200.0, 0.0)], true),
            ("arco raio 10·r", arc(10.0 * R, 90.0), true),
            ("arco raio 2·r (curvatura alta)", arc(2.0 * R, 180.0), true),
            ("arco raio 1·r (o limite do pincel)", arc(R, 360.0), true),
            (
                "hachura gap 0.5·r",
                (0..5)
                    .flat_map(|i| {
                        let x = i as f32 * 0.5 * R;
                        if i % 2 == 0 {
                            [(x, 0.0), (x, 80.0)]
                        } else {
                            [(x, 80.0), (x, 0.0)]
                        }
                    })
                    .collect(),
                true,
            ),
            // A MÃO LENTA: o RDP preserva pontos numa curva apertada, então a entrada
            // pode chegar mais densa que o passo da reamostragem (que só ACRESCENTA).
            (
                "curva com entrada 4× densa (mão lenta)",
                (0..=240)
                    .map(|k| {
                        let a = (k as f32 / 240.0) * std::f32::consts::PI;
                        (4.0 * R * a.cos(), 4.0 * R * a.sin())
                    })
                    .collect(),
                false,
            ),
        ];

        println!("\n  cenário                                 | pts | máx | média | densidade");
        println!("  ----------------------------------------|-----|-----|-------|----------");
        let mut worst_product = 0usize;
        for (name, spine, product_density) in cases {
            let pts = densify(&spine, step);
            let segs = segs_of(&pts, R);
            let mut stamp = vec![0u32; segs.len()];
            let (mut max_n, mut sum) = (0usize, 0usize);
            for i in 0..segs.len() {
                let mut out = Vec::new();
                push_ribbon_local(&segs, i, false, R, &mut stamp, i as u32 + 1, &mut out);
                max_n = max_n.max(out.len());
                sum += out.len();
            }
            if product_density {
                worst_product = worst_product.max(max_n);
            }
            println!(
                "  {name:<39} | {:>3} | {max_n:>3} | {:>5.1} | {}",
                segs.len(),
                sum as f32 / segs.len() as f32,
                if product_density {
                    "produto"
                } else {
                    "4x densa"
                }
            );
        }
        println!(
            "\n  pior caso na densidade do PRODUTO: {worst_product}   \
             MAX_RIBBON_EXTRAS = {MAX_RIBBON_EXTRAS}\n"
        );
        assert!(
            MAX_RIBBON_EXTRAS >= worst_product,
            "o teto ({MAX_RIBBON_EXTRAS}) tem de cobrir o pior caso na densidade que o \
             produto de fato produz ({worst_product}) — abaixo dele a fita local volta a \
             perder vizinhos e a mordida ressurge no traço NORMAL"
        );
    }
}
