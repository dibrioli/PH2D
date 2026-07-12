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
        out[i] = top.iter().map(|&(_, j)| j).collect();
    }
    out
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
    fn brute_force(segs: &[Seg]) -> Vec<Vec<u32>> {
        let n = segs.len();
        let mut out = vec![Vec::new(); n];
        for i in 0..n {
            let mut cand: Vec<(f32, u32)> = Vec::new();
            for j in 0..n {
                let (si, sj) = (segs[i], segs[j]);
                if i == j || is_adjacent(&si, &sj) {
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
            out[i] = cand.iter().map(|&(_, j)| j).collect();
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
