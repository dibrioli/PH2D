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

/// Teto de CÁPSULAS de cruzamento por segmento (outras passagens). Um traço patológico
/// (rabisco denso rabiscado por cima de si mesmo dezenas de vezes) não pode fazer o
/// fragment iterar sem fim; as cápsulas são ordenadas por proximidade, então o corte
/// descarta as que menos contribuem. Além do teto o traço volta a ter o first-wins do GP
/// naqueles pixels (o artefato histórico), nunca algo pior.
///
/// ⚠️ **O teto é de CÁPSULAS, não de segmentos, e é essa a diferença que importa.** Uma
/// cápsula cobre um PEDAÇO DE CAMINHO de comprimento arbitrário (ver [`MERGE_SAGITTA`]),
/// então o número necessário é função da CURVATURA e do alcance — nunca de quão fino o
/// motor amostrou o caminho.
pub(crate) const MAX_EXTRAS_PER_SEGMENT: usize = 16;

/// **A tolerância de FUSÃO de cápsulas, em fração do raio.**
///
/// Segmentos consecutivos quase-colineares descrevem o mesmo pedaço de caminho; uma ÚNICA
/// cápsula ligando as pontas cobre a mesma tinta, com erro igual à FLECHA da corda. Fundir
/// enquanto a flecha ficar abaixo de `MERGE_SAGITTA × raio` torna o número de cápsulas
/// função da GEOMETRIA, não da amostragem.
///
/// Numa curva de raio de curvatura `R`, a corda que atinge a tolerância mede
/// `L = √(8·R·tol·r)`; o alcance a cobrir é `≈ 3·r`, então o número de cápsulas por lado é
/// `3r/L`. Com `tol = 1/32`:
///
/// | curvatura | corda fundida | cápsulas por lado |
/// |---|---|---|
/// | RETA (`R = ∞`) | o alcance inteiro | **1** |
/// | `R = 4·r` (curva larga) | `1,00·r` | 3 |
/// | `R = r` (curva do tamanho do pincel) | `0,50·r` | 6 |
/// | `R = r/4` (grampo) | `0,25·r` | 12 |
///
/// ⚠️ **E numa RETA a fusão é EXATA** (flecha zero), que é o caso da esmagadora maioria do
/// arco de qualquer traço: o alcance inteiro vira uma cápsula só.
const MERGE_SAGITTA: f32 = 0.031_25;

/// Teto de PASSOS da caminhada da fita, por direção. Não é um teto de qualidade (as cápsulas
/// fundidas cobrem o alcance com um punhado): é o guarda contra o rabisco patológico, onde a
/// caminhada dentro do alcance poderia ter milhares de segmentos. A degradação é a mesma do
/// [`PAIR_BUDGET`] — e no meio de um borrão sólido de tinta ela é invisível.
const MAX_RIBBON_WALK: usize = 512;

/// Teto de candidatos de CRUZAMENTO colhidos do grid antes de fundir. Os candidatos crus são
/// segmentos (muitos, quando o traço é denso); as cápsulas que sobram são poucas. Este teto
/// existe para o buffer não crescer sem fim num rabisco patológico.
const MAX_CROSS_CANDIDATES: usize = 1024;

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
pub(crate) fn extras_for_stroke(segs: &[Seg]) -> Vec<SegExtras> {
    let n = segs.len();
    let mut out = vec![SegExtras::default(); n];
    if n < 3 {
        return out; // nada não-adjacente existe
    }
    let closed = segs[n - 1].b == segs[0].a;
    let grid = Grid::build(segs);
    // Dedup por GERAÇÃO (o segmento aparece em várias células da consulta): o carimbo
    // é o `i+1` da iteração — nada a limpar entre iterações, e a classe de bug
    // "limpeza incompleta do visitado" deixa de existir.
    let mut stamp = vec![0u32; n];
    // Buffers reusados entre iterações (zero alocação por segmento).
    let mut walk: Vec<usize> = Vec::with_capacity(MAX_RIBBON_WALK);
    let mut cand: Vec<(f32, u32)> = Vec::with_capacity(MAX_CROSS_CANDIDATES);
    let mut caps: Vec<(f32, Capsule)> = Vec::with_capacity(MAX_EXTRAS_PER_SEGMENT + 1);
    // Buffer da fusão dos cruzamentos — reusado, senão cada segmento aloca por run (medido:
    // o rabisco patológico de 4000 pontos custou +38 % com a alocação).
    let mut run: Vec<usize> = Vec::with_capacity(MAX_CROSS_CANDIDATES);
    let mut budget = PAIR_BUDGET;
    for i in 0..n {
        if budget == 0 {
            break; // teto de trabalho — ver PAIR_BUDGET
        }
        let si = segs[i];
        let visit = i as u32 + 1;
        // A FITA LOCAL entra PRIMEIRO e é carimbada por INTEIRO — a consulta do grid abaixo a
        // pula, então os slots de cruzamento ficam inteiros. Ver `push_ribbon_local`.
        push_ribbon_local(
            segs,
            i,
            closed,
            &mut stamp,
            visit,
            &mut walk,
            &mut out[i].list,
        );
        // ⚠️ TUDO que entrou até aqui é a PRÓPRIA passagem — e é essa fronteira que o fragment
        // precisa para COMPOR em vez de unir.
        out[i].ribbon = out[i].list.len() as u32;
        // CONSULTA com pad `2·r_i` — o alcance dos pixels do quad deste segmento.
        cand.clear();
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
                    if d2 < reach * reach && cand.len() < MAX_CROSS_CANDIDATES {
                        cand.push((d2, j as u32));
                    }
                }
            }
        }
        // ⚠️ **Os cruzamentos também FUNDEM, e pela mesma razão que a fita.** Um cruzamento raso
        // sobre polilinha densa traz DEZENAS de segmentos da outra passagem; capear isso por
        // CONTAGEM era a mesma falha, no outro lado. Agrupa-se por índice CONSECUTIVO (é um
        // pedaço contíguo do caminho) e cada grupo vira as cápsulas que a geometria pedir.
        // ⚠️ **Tudo que o grid traz agora é OUTRA passagem**, por construção: a caminhada já
        // carimbou a própria. É isso que mantém a fronteira `ribbon` — a que o fragment usa
        // para COMPOR em vez de unir — exata sem nenhum segundo carimbo.
        caps.clear();
        cand.sort_unstable_by_key(|&(_, j)| j);
        let mut k = 0usize;
        while k < cand.len() {
            run.clear();
            run.push(cand[k].1 as usize);
            let mut d2 = cand[k].0;
            let mut e = k + 1;
            while e < cand.len() && cand[e].1 == cand[e - 1].1 + 1 {
                run.push(cand[e].1 as usize);
                d2 = d2.min(cand[e].0);
                e += 1;
            }
            merge_run(segs, &run, &mut |c| caps.push((d2, c)));
            k = e;
        }
        // Ordena por proximidade e corta — o corte descarta CÁPSULAS (pedaços de caminho
        // inteiros), não segmentos avulsos de um pedaço que outra fatia já cobre.
        caps.sort_by(|a, b| a.0.total_cmp(&b.0));
        caps.truncate(MAX_EXTRAS_PER_SEGMENT);
        out[i].list.extend(caps.iter().map(|&(_, c)| c));
    }
    out
}

/// Uma cápsula da lista de vizinhos: o par de índices GLOBAIS de ponto que o fragment usa para
/// montar a cápsula (`points[a]` → `points[b]`). Pode abranger VÁRIOS segmentos consecutivos —
/// é isso que torna o teto uma propriedade da geometria em vez da amostragem.
pub(crate) type Capsule = (u32, u32);

/// **A FUSÃO.** Recebe segmentos em ordem de ARCO (consecutivos) e emite as cápsulas que os
/// cobrem, cortando onde a corda deixa de descrever o caminho.
///
/// O corte usa a **flecha estimada** `L·θ/8` (a sagitta de um arco de comprimento `L` que virou
/// `θ`), com `θ` acumulado sem trigonometria (`|cross|/(|d₁||d₂|) ≈ |sin Δθ|`, HR-5) — e uma
/// virada acima de 90° (`dot < 0`) corta sempre, porque ali o seno volta a cair e a estimativa
/// deixaria de ser conservadora. A variação de RAIO também corta: a cápsula interpola o raio
/// linearmente entre as pontas, então uma barriga de pressão no meio ficaria descoberta.
fn merge_run(segs: &[Seg], run: &[usize], emit: &mut dyn FnMut(Capsule)) {
    merge_run_capped(segs, run, usize::MAX, emit);
}

/// Como [`merge_run`], mas emitindo no máximo `room` cápsulas — e devolvendo **quantos
/// segmentos do `run` de fato ficaram COBERTOS**.
///
/// ⚠️ O retorno é o que impede a falha silenciosa: quem estoura o teto precisa saber onde a
/// cobertura parou, para não carimbar como *já resolvido* um pedaço de caminho que ninguém
/// carrega. É a lição que o par de carimbos desta função pagou duas vezes (carimbar tudo abria
/// BURACO; não carimbar nada ADICIONAVA tinta).
fn merge_run_capped(
    segs: &[Seg],
    run: &[usize],
    room: usize,
    emit: &mut dyn FnMut(Capsule),
) -> usize {
    if run.is_empty() || room == 0 {
        return 0;
    }
    let mut start = 0usize;
    let mut emitted = 0usize;
    let mut len = 0.0f32;
    let mut turn = 0.0f32;
    let (mut r_lo, mut r_hi) = (segs[run[0]].radius, segs[run[0]].radius);
    let mut prev_dir: Option<Vec2> = None;
    for (k, &j) in run.iter().enumerate() {
        let s = segs[j];
        let d = s.pb - s.pa;
        let l = (d.x * d.x + d.y * d.y).sqrt();
        // ⚠️ **CONTIGUIDADE é conferida, nunca assumida.** Índice consecutivo não é o mesmo que
        // caminho contíguo: fundir dois segmentos que não compartilham ponto desenharia uma
        // cápsula ao longo de uma reta que o traço nunca percorreu. A polilinha do produto é
        // contígua por construção, e é exatamente por isso que a premissa passaria despercebida.
        let mut cut = k > start && segs[run[k - 1]].b != s.a;
        if let Some(pd) = prev_dir {
            let denom = (pd.x * pd.x + pd.y * pd.y).sqrt() * l;
            if denom > 1e-9 {
                let sin = ((pd.x * d.y - pd.y * d.x) / denom).abs();
                let straight = pd.x * d.x + pd.y * d.y >= 0.0;
                turn += sin;
                // Uma quina (>90°) NUNCA funde: o seno volta a cair e a flecha estimada
                // mentiria — e uma quina é justamente onde a corda mais se afasta do caminho.
                cut = cut || !straight;
            }
        }
        let nr_lo = r_lo.min(s.radius);
        let nr_hi = r_hi.max(s.radius);
        let tol = MERGE_SAGITTA * segs[run[0]].radius.max(1e-4);
        // Flecha estimada da corda que iria de `start` até AQUI, e a barriga de raio.
        cut = cut || (len + l) * turn * 0.125 > tol || (nr_hi - nr_lo) > tol;
        if cut && k > start {
            if emitted == room {
                return start; // o teto: a cobertura parou ONDE a última cápsula terminou
            }
            emit((segs[run[start]].a, segs[run[k - 1]].b));
            emitted += 1;
            start = k;
            len = 0.0;
            turn = 0.0;
            r_lo = s.radius;
            r_hi = s.radius;
        } else {
            r_lo = nr_lo;
            r_hi = nr_hi;
        }
        len += l;
        prev_dir = Some(d);
    }
    if emitted == room {
        return start;
    }
    emit((segs[run[start]].a, segs[run[run.len() - 1]].b));
    run.len()
}

/// Teto de CÁPSULAS da fita local por segmento — separado do [`MAX_EXTRAS_PER_SEGMENT`] dos
/// cruzamentos, porque as duas listas respondem a perguntas diferentes.
///
/// ⚠️ **A wave de 2026-07-28 trocou a UNIDADE deste teto, e é isso que ele significa hoje.**
/// Antes ele contava SEGMENTOS, e um teto em segmentos para cobrir um ALCANCE é um
/// multiplicador disfarçado de teto: a contagem necessária é `alcance / passo`, então o teto
/// era atravessado assim que a polilinha ficasse mais densa que `3·r/16 = 0,1875·r` — o que o
/// produto faz **quando a mão desenha devagar** (medido em
/// `flip_draw_tests::the_real_pipeline_step_in_radii`: passo mínimo 0,137·r num arco de 400
/// amostras e 0,108·r em 1200, com 125 de 251 segmentos abaixo da cerca). Passando a cerca, a
/// lista truncava, o pixel voltava ao first-wins do GP e a tinta SUMIA: −184 de 255 em 0,10·r
/// e −255 (tinta nenhuma) em 0,05·r, medidos contra o depósito real do Painter.
///
/// Hoje o teto conta **CÁPSULAS** (ver [`MERGE_SAGITTA`]), e o número necessário é função da
/// CURVATURA e do alcance — **em qualquer densidade de amostragem**. MEDIDO
/// (`measure_ribbon_budget`, rode com `-- --nocapture`), contra os números de antes:
///
/// | cenário (raio 5) | antes | agora |
/// |---|---|---|
/// | reta | 4 | **2** |
/// | arco raio 10·r | 12 | **4** |
/// | arco raio 2·r | 11 | **6** |
/// | arco raio 1·r (o limite do pincel) | 11 | **6** |
/// | hachura gap 0,5·r | 6 | **4** |
/// | **entrada 4× densa (mão LENTA)** | **16, SATURADO** | **8** |
///
/// Pior caso do produto **6** contra o teto **16** — e a linha que importa é a última: era ela
/// que estourava, e é ela que o artista produz desenhando devagar. O gate que pina a propriedade
/// é `sampling_invariance::the_ink_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled`
/// (desvio CONSTANTE de −3 de 255 em toda densidade de 0,80·r a 0,04·r).
pub(crate) const MAX_RIBBON_EXTRAS: usize = 16;

/// Os vizinhos geométricos de UM segmento, **particionados por PASSAGEM**.
///
/// ⚠️ A partição é a wave inteira (2026-07-28, 2ª foto do Enio: *"se o mesmo traço cruza a si
/// mesmo então temos o mesmo aspecto indesejado"*). Dois traços distintos têm depth diferente e o
/// mais novo pinta POR CIMA — **composição `over`**, lisa. Um traço que cruza a si mesmo tem o
/// MESMO depth, o `GREATER` estrito descarta o 2º quad, e sobra a **união** (`min` de distâncias)
/// — que tem **VINCO** na bissetriz, invisível em hardness 1 e uma costura com pincel macio
/// (medido: 48/255 em hardness 0,4 · 35/255 em 0,7).
///
/// Com a fronteira, o fragment compõe as duas coberturas em vez de tomar o `max` delas, e as duas
/// rotas passam a desenhar a mesma coisa.
#[derive(Clone, Default, Debug)]
pub(crate) struct SegExtras {
    /// As CÁPSULAS vizinhas, como pares de índices GLOBAIS de ponto: **PRIMEIRO as da própria
    /// passagem** (a fita local, por arco), depois as de OUTRAS passagens (os cruzamentos, pelo
    /// grid). A ordem é o contrato.
    ///
    /// ⚠️ **Um par NÃO é um segmento** — ele pode abranger vários segmentos consecutivos
    /// (ver [`MERGE_SAGITTA`]). O shader monta a cápsula de `points[a]` a `points[b]` e não
    /// precisa saber a diferença, então o formato do buffer e a BGL ficaram como estavam.
    pub(crate) list: Vec<Capsule>,
    /// Quantas das primeiras pertencem à PRÓPRIA passagem.
    pub(crate) ribbon: u32,
}

/// Os vizinhos da MESMA PASSAGEM: os segmentos alcançáveis **andando pela polilinha**
/// a partir de `i` dentro do alcance de influência do quad dele.
///
/// **A definição se escreve sozinha e não tem constante mágica.** Um segmento que está perto de
/// `i` ou é (a) a fita continuando — e então dá para CHEGAR nele andando sem nunca sair do
/// alcance — ou (b) o traço que foi embora e VOLTOU, isto é um cruzamento, e no meio do caminho
/// a caminhada SAIU. A fronteira é o primeiro segmento fora de alcance.
///
/// ⚠️ **A v1 desta wave cortava por ARCO e estava ERRADA — não re-derive.** Numa curva fechada a
/// mesma fita volta a ficar perto com arco grande, era classificada como passagem ESTRANHA, e o
/// fragment compunha tinta consigo mesma (medido, arco de raio 22 com pincel 14: pintou 196 onde
/// a união pede 184). O gate que pina isto é
/// `a_dense_soft_ribbon_that_never_crosses_itself_is_exactly_the_union`.
///
/// ⚠️ **E o teste ESPACIAL só é honesto sobre uma polilinha REAMOSTRADA:** a distância
/// segmento-a-segmento de dois segmentos ENORMES vê apenas as pontas, então uma ida-e-volta
/// desenhada com 2 segmentos nunca "sai" e o braço de volta viraria a mesma passagem. O produto
/// reamostra todo traço (`resample_smooth`, passo `0.4 × largura`) ⇒ o regime real é o de
/// segmentos curtos; a fixture do gate do cruzamento é DENSA por causa disto.
///
/// Os adjacentes (`i±1`) são pulados: chegam ao fragment pela janela de sequência
/// (`p0`/`p3`) e não gastam slot. Traço FECHADO dá a volta; aberto para nas pontas.
fn push_ribbon_local(
    segs: &[Seg],
    i: usize,
    closed: bool,
    stamp: &mut [u32],
    visit: u32,
    walk: &mut Vec<usize>,
    out: &mut Vec<Capsule>,
) {
    let n = segs.len();
    stamp[i] = visit;
    for dir in [1i64, -1i64] {
        walk.clear();
        let mut j = i as i64;
        for _ in 0..n.min(MAX_RIBBON_WALK) {
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
            // ⚠️ **A passagem acaba onde a caminhada SAI da vizinhança** — teste ESPACIAL, não
            // de arco. O arco era a v1 desta wave e ele QUEBRA em curva fechada: ali a mesma fita
            // volta a ficar perto com arco grande, era classificada como passagem ESTRANHA, e
            // compunha tinta consigo mesma (medido, arco de raio 22 com pincel 14: pintou 196
            // onde a união pede 184). É a definição que o doc desta função sempre afirmou —
            // *alcançável andando pela polilinha SEM SAIR do alcance* —, agora de fato aplicada.
            let reach = 2.0 * segs[i].radius + sj.radius;
            if seg_seg_distance_sq(segs[i].pa, segs[i].pb, sj.pa, sj.pb) > reach * reach {
                break;
            }
            // ⚠️ **UM carimbo, e o segundo MORREU junto com a causa dele.** Havia um par
            // (`walked` = *a caminhada chegou aqui* · `stamp` = *já está na lista*) porque a
            // fita TRUNCAVA: o segmento visitado que não coubesse ficava sem `stamp` para o
            // grid poder recolhê-lo, e precisava ser lembrado como *própria passagem* — senão
            // ele voltava como ESTRANHA e o fragment compunha tinta consigo mesma (+63, medido).
            //
            // Com as cápsulas fundidas a caminhada **absorve a passagem inteira** (1 cápsula
            // numa reta, ~6 numa curva do tamanho do pincel), então não há transbordo a
            // recolher — e as duas mutações que o par equilibrava passaram a NÃO sangrar em
            // fixture nenhuma: o mecanismo virou código morto, e código morto MENTE. O que
            // sobra é a lei simples: **o que a caminhada visitou é da própria passagem, e o
            // grid não o vê.** Na curvatura extrema em que o teto ainda morde, o arco mais
            // DISTANTE é descartado (first-wins ali, a degradação de sempre) — nunca composto.
            stamp[ju] = visit;
            walk.push(ju);
        }
        // A caminhada para trás visita em arco DECRESCENTE; a fusão precisa de ordem de arco.
        if dir < 0 {
            walk.reverse();
        }
        let room = MAX_RIBBON_EXTRAS.saturating_sub(out.len());
        merge_run_capped(segs, walk, room, &mut |c| out.push(c));
    }
}

/// Os testes moram no irmão `neighbors_tests.rs` — o `neighbors.rs` cruzou o teto de 700 LOC
/// quando a partição por passagem entrou. Por `#[path]` eles seguem sendo módulos FILHOS, então
/// `use super::*` continua alcançando os privados e nenhuma visibilidade muda.
#[cfg(test)]
#[path = "neighbors_tests.rs"]
mod tests;
