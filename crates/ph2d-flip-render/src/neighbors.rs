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
pub(crate) fn extras_for_stroke(segs: &[Seg]) -> Vec<SegExtras> {
    let n = segs.len();
    let mut out = vec![SegExtras::default(); n];
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
    // *A caminhada chegou aqui* — separado do `stamp` (*já está na lista*). Ver `push_ribbon_local`.
    let mut walked = vec![0u32; n];
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
        push_ribbon_local(
            segs,
            i,
            closed,
            max_radius,
            &mut stamp,
            &mut walked,
            visit,
            &mut out[i].list,
        );
        // ⚠️ TUDO que entrou até aqui é a PRÓPRIA passagem — e é essa fronteira que o fragment
        // precisa para COMPOR em vez de unir. Ela é gratuita: o `push_ribbon_local` já a define
        // por comprimento de ARCO, então basta CONTAR o que ele deixou.
        out[i].ribbon = out[i].list.len() as u32;
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
        // ⚠️ O que o grid trouxe se PARTICIONA: o que a caminhada tinha alcançado (o transbordo
        // do teto da fita) é a MESMA passagem e vai para a FRENTE, contando no `ribbon`; o resto
        // é cruzamento de verdade. Sem isto o transbordo viraria passagem estranha e o fragment
        // comporia tinta da fita consigo mesma.
        let (same, foreign): (Vec<u32>, Vec<u32>) = top
            .iter()
            .map(|&(_, j)| j)
            .partition(|&j| walked[j as usize] == visit);
        out[i].ribbon += same.len() as u32;
        out[i].list.extend(same);
        out[i].list.extend(foreign);
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
    /// Os índices dos vizinhos: **PRIMEIRO os da própria passagem** (a fita local, por arco),
    /// depois os de OUTRAS passagens (os cruzamentos, pelo grid). A ordem é o contrato.
    pub(crate) list: Vec<u32>,
    /// Quantos dos primeiros pertencem à PRÓPRIA passagem.
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
// ⚠️ Oito argumentos: os dois carimbos (`stamp` = *já está na lista* · `walked` = *a caminhada
// chegou aqui*) são estado do CHAMADOR reusado entre iterações — juntá-los num struct só
// para agradar o lint esconderia justamente a distinção que esta wave existe para fazer.
#[allow(clippy::too_many_arguments)]
fn push_ribbon_local(
    segs: &[Seg],
    i: usize,
    closed: bool,
    max_radius: f32,
    stamp: &mut [u32],
    walked: &mut [u32],
    visit: u32,
    out: &mut Vec<u32>,
) {
    let n = segs.len();
    // Cota conservadora do alcance: o raio do dono entra DOBRADO (o teto do esticão do
    // miter) e o do vizinho é cotado pelo MAIOR do traço — o teste por-par abaixo é o
    // exato, este só decide quando PARAR de andar.
    let _ = max_radius;
    stamp[i] = visit;
    walked[i] = visit;
    for dir in [1i64, -1i64] {
        let mut j = i as i64;
        for _ in 0..n {
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
                walked[ju] = visit;
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
            // ⚠️ **DOIS carimbos, e a distinção é o que impede as duas falhas opostas.**
            // `walked` diz *a caminhada CHEGOU aqui* (⇒ é a MESMA passagem, sempre); `stamp` diz
            // *já está na lista* (⇒ o grid não duplica). Acima do teto o segmento fica sem
            // `stamp` — então o grid PODE recolhê-lo — mas com `reach`, então ele entra como
            // PRÓPRIA passagem e não como estranha.
            //
            // As duas versões anteriores erraram para lados opostos, as duas medidas na sonda
            // `measure_a_sharp_corner_that_also_crosses_itself`: carimbar tudo tirava o
            // transbordo das DUAS listas e abria **BURACO** (pintou 0 onde a união pede 252 — a
            // cunha escura da 3ª foto do Enio); não carimbar nada o fazia virar passagem
            // estranha e o fragment **ADICIONAVA** tinta (+63).
            walked[ju] = visit;
            if out.len() < MAX_RIBBON_EXTRAS {
                stamp[ju] = visit;
                out.push(ju as u32);
            }
        }
    }
}

/// Os testes moram no irmão `neighbors_tests.rs` — o `neighbors.rs` cruzou o teto de 700 LOC
/// quando a partição por passagem entrou. Por `#[path]` eles seguem sendo módulos FILHOS, então
/// `use super::*` continua alcançando os privados e nenhuma visibilidade muda.
#[cfg(test)]
#[path = "neighbors_tests.rs"]
mod tests;
