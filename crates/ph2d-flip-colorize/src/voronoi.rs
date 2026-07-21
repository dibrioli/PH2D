//! **O Voronoi geodésico POR PIXEL de um componente contestado** (`09 §3`; 4º smoke,
//! 2026-07-20) — quem divide um componente que ≥2 cores reivindicam.
//!
//! # As três leis da métrica
//!
//! 1. **A tinta é INTRANSPONÍVEL.** A frente de uma cor só anda por PAPEL — quem quer o
//!    outro lado de uma linha paga a volta pelo vão. É daqui que vem o casamento com a
//!    linha num componente contestado: cada lado da linha pertence a quem está nele, e a
//!    fronteira visível é a própria tinta. (A versão anterior pesava `V_pq`, o custo de
//!    CORTE do LazyBrush — tinta barata —, que numa métrica de DISTÂNCIA é a direção
//!    errada: a linha ficava invisível para a frente e a fronteira caía no meio dos
//!    rabiscos. O gate `a_contested_boundary_lands_on_the_ink_not_between_the_scribbles`
//!    nasceu vermelho sobre isso.)
//! 2. **Chanfro 5/7** (reto/diagonal, 7/5 = 1,4 ≈ √2, erro ≤ 2%): as faixas numa área
//!    aberta saem retas, não os losangos da métrica de Manhattan. ⚠️ O passo diagonal com
//!    tinta nos DOIS cantos é recusado — senão a frente escorrega POR DENTRO de uma linha
//!    diagonal que o flood de componentes (4-conexo) trata como parede, e as duas leis
//!    divergem (`a_diagonal_line_is_a_wall_not_a_sieve`).
//! 3. **Pedágio de APERTO**: entrar num pixel a distância `d` da tinta custa
//!    `SQUEEZE/(1+d²)` além do passo. É a intuição da trapped-ball em forma contínua — um
//!    vão estreito é um gargalo caro (quase selado), um vão largo é passagem honesta — e é
//!    o que mata a *película*: sem ele, uma cor cuja semente está perto de um vão ganha a
//!    corrida geodésica e pinta um filme rente à face ALHEIA da linha (medido no fixture
//!    do gate de fronteira: com `SQUEEZE=0` a cor da direita forra a face esquerda do
//!    divisor). Longe da tinta o pedágio é zero e a corrida é limpa (faixas parelhas).
//!
//! O empate resolve pela ordem determinística de processamento (rótulos em ordem estável,
//! sementes em ordem de índice, baldes FIFO por distância — HR-5, só inteiros e `Vec`).
//!
//! # Custo
//!
//! Dial (fila de baldes): O(pixels·8 + dist_máx). Sem heap, sem comparação — o mesmo
//! O(N) da dilatação da partição, e só roda nos componentes DISPUTADOS.

use crate::segment::{NO_REGION, Segmentation};
use ph2d_flip_fill::{BOUNDARY, Grid};

/// Passo reto do chanfro (por pixel de papel).
const STEP: u32 = 5;
/// Passo diagonal (7/5 = 1,4 ≈ √2 — erro ≤ 2% contra a métrica euclidiana).
const STEP_DIAG: u32 = 7;

/// O numerador do pedágio de aperto: entrar num pixel a distância `d` px da tinta custa
/// `SQUEEZE/(1+d²)` além do passo.
///
/// **MEDIDO** (2026-07-20) no fixture da película — divisor em x=0,700 com vão de 40 px,
/// sementes assimétricas 0,25/0,85 a precisão 400 (a da direita 3× mais perto da linha) — e
/// no blob aberto de 4 cores. `max_x` = até onde a cor da ESQUERDA chega longe do vão (a
/// película alheia a empurra para antes da linha); `lente` = quão fundo a cor da direita
/// entra pelo vão (min_x, total / longe da banda do vão); `blob` = razão máx/mín das faixas:
///
/// | SQUEEZE | max_x longe do vão | lente (total/longe) | blob máx/mín |
/// |--------:|-------------------:|--------------------:|-------------:|
/// |       0 |  0,679 (película!) |       0,553 / 0,576 |         1,08 |
/// |     256 |    0,697 (a linha) |       0,553 / 0,627 |         1,09 |
/// |    1024 |              0,697 |       0,572 / 0,653 |         1,12 |
/// |  *4096* |              0,697 |   0,648 / **0,702** |         1,20 |
/// |   16384 |              0,697 |       0,685 / 0,702 |         1,29 |
///
/// A película morre já em 256; `4096` é onde o vazamento **fora da banda do vão** zera
/// (0,702 = a própria linha) com as faixas abertas ainda parelhas (1,20 ≪ o gate de 1,8).
/// 16384 compra ~6 px de lente por +0,09 de desvio no blob — passou do joelho. O pedágio
/// decai com d² ⇒ no miolo de um blob é zero, e no meio de um vão LARGO (48 px, o smoke da
/// caixa) vale `4096/577 ≈ 7` ≈ um passo: passagem honesta.
/// O pedágio de aperto DEFAULT (`Bleed` no painel do Colorize) — o comportamento aprovado
/// no 5º smoke. O painel expõe uma faixa em torno dele (ver [`crate::DEFAULT_SQUEEZE`]); a
/// tabela medida na doc de [`crate::colorize`] mostra a lente por valor.
pub const DEFAULT_SQUEEZE: u32 = 4096;

/// Rascunho reutilizado entre componentes contestados (os planos são O(grade)).
pub(crate) struct Scratch {
    dist: Vec<u32>,
    lab: Vec<u32>,
    /// O pedágio por pixel, pré-computado da EDT da partição (u16: o produto satura em ~2¹⁷).
    toll: Vec<u16>,
    /// A fila de baldes de Dial: `dist % ring` indexa; `ring > STEP_DIAG + squeeze`.
    buckets: Vec<Vec<u32>>,
}

impl Scratch {
    pub(crate) fn new(ink_dist2: &[u32], squeeze: u32) -> Self {
        let n = ink_dist2.len();
        // Divisão INTEIRA de propósito (HR-5: nenhum float na métrica). O `toll` é u16, mas o
        // `squeeze` pode passar de 65535 (o painel vai até ~131072); o clamp mantém a
        // representação sã sem estreitar a faixa útil (no meio de um vão de 48 px, `d² ≈ 576`,
        // então `toll = squeeze/577` cabe em u16 muito além do teto do slider).
        // ⚠️ `saturating_add`: a EDT devolve `u32::MAX` quando o conjunto de TINTA é vazio (o
        // contrato está documentado em `edt.rs`), e `1 + u32::MAX` wrapa para `0` ⇒ divisão por
        // zero. Não é alcançável pelo `boundaries()` do shell (que filtra traços com < 2
        // pontos), mas `colorize_with` é `pub` e a guarda pertence a quem divide.
        let toll = ink_dist2
            .iter()
            .map(|&sq| (squeeze / sq.saturating_add(1).max(1)).min(u32::from(u16::MAX)) as u16)
            .collect();
        // O ring do Dial só precisa ser maior que o maior INCREMENTO único (`base + toll`).
        // O `toll` de um pixel de PAPEL é `squeeze/(1+d²)` com `d² ≥ 1` (papel colado na
        // linha tem `d = 1`), logo o teto é `squeeze/2` (clampado ao u16). Dimensionar por
        // aqui, e não pelo `squeeze` bruto, mantém o ring justo (default 4096 ⇒ 2056 baldes,
        // MENOR que os 4104 de antes; slider no máximo ⇒ ~16 k).
        let max_toll = (squeeze / 2).min(u32::from(u16::MAX));
        Scratch {
            dist: vec![u32::MAX; n],
            lab: vec![u32::MAX; n],
            toll,
            buckets: vec![Vec::new(); (max_toll + STEP_DIAG) as usize + 1],
        }
    }
}

/// Divide UM componente contestado entre as cores que o semeiam: Dijkstra multi-fonte sobre
/// os pixels de PAPEL do componente, na métrica do topo do módulo. Escreve `assign` para
/// todo pixel de papel do componente; a tinta fica sem cor (a linha desenha por cima).
pub(crate) fn claim(
    grid: &Grid,
    seg: &Segmentation,
    comp: u32,
    labels: &[(u16, Vec<usize>)],
    s: &mut Scratch,
    assign: &mut [Option<usize>],
) {
    let (w, h) = (grid.w, grid.h);
    let is_ink = |i: usize| grid.flags[i] & BOUNDARY != 0;
    let member = |i: usize| seg.component[i] == comp && !is_ink(i);

    s.dist.fill(u32::MAX);
    s.lab.fill(NO_REGION);
    for b in &mut s.buckets {
        b.clear();
    }

    // As sementes partem com distância 0. Rótulos em ordem estável; pixel disputado por dois
    // rabiscos fica com o primeiro (determinístico — `labels` preserva a ordem de chegada).
    let mut pending = 0usize;
    for (k, (_, pixels)) in labels.iter().enumerate() {
        for &p in pixels {
            if member(p) && s.lab[p] == NO_REGION {
                s.lab[p] = k as u32;
                s.dist[p] = 0;
                s.buckets[0].push(p as u32);
                pending += 1;
            }
        }
    }

    let ring = s.buckets.len();
    let mut d: u32 = 0;
    while pending > 0 {
        let idx = (d as usize) % ring;
        while let Some(pf) = s.buckets[idx].pop() {
            pending -= 1;
            let p = pf as usize;
            if s.dist[p] != d {
                continue; // entrada obsoleta: o pixel já foi alcançado por caminho menor
            }
            let k = s.lab[p];
            let (x, y) = (p % w, p / w);
            // Os 8 vizinhos; nos diagonais, os dois CANTOS ortogonais (a recusa da peneira).
            //
            // ⚠️ O array é construído **EAGER**: todo índice é avaliado ANTES de o `ok` dele ser
            // consultado. Então todo aritmético aqui tem de ser não-panicante mesmo na borda em
            // que o `ok` é falso — em `y == 0` o `p.wrapping_sub(w)` devolve um valor enorme, e
            // um `+ 1` cru sobre ele **panica em debug** (e o `ci-test` do ship roda em debug:
            // o produto panicava ao colorir num build de debug sempre que um componente
            // contestado alcançasse a linha 0 da grade — o caso NORMAL).
            type Arm = (bool, usize, u32, Option<(usize, usize)>);
            let arms: [Arm; 8] = [
                (x + 1 < w, p + 1, STEP, None),
                (x > 0, p.wrapping_sub(1), STEP, None),
                (y + 1 < h, p + w, STEP, None),
                (y > 0, p.wrapping_sub(w), STEP, None),
                (
                    x + 1 < w && y + 1 < h,
                    p + w + 1,
                    STEP_DIAG,
                    Some((p + 1, p + w)),
                ),
                (
                    x > 0 && y + 1 < h,
                    p + w - 1,
                    STEP_DIAG,
                    Some((p.wrapping_sub(1), p + w)),
                ),
                (
                    x + 1 < w && y > 0,
                    p.wrapping_sub(w).wrapping_add(1),
                    STEP_DIAG,
                    Some((p + 1, p.wrapping_sub(w))),
                ),
                (
                    x > 0 && y > 0,
                    p.wrapping_sub(w + 1),
                    STEP_DIAG,
                    Some((p.wrapping_sub(1), p.wrapping_sub(w))),
                ),
            ];
            for (ok, q, base, corners) in arms {
                if !ok || !member(q) {
                    continue;
                }
                if let Some((c1, c2)) = corners
                    && is_ink(c1)
                    && is_ink(c2)
                {
                    continue; // o passo diagonal POR DENTRO da linha (lei 2)
                }
                let nd = d + base + u32::from(s.toll[q]);
                if nd < s.dist[q] {
                    s.dist[q] = nd;
                    s.lab[q] = k;
                    s.buckets[(nd as usize) % ring].push(q as u32);
                    pending += 1;
                }
            }
        }
        d += 1;
    }

    for (i, a) in assign.iter_mut().enumerate() {
        if member(i) && s.lab[i] != NO_REGION {
            *a = Some(s.lab[i] as usize);
        }
    }
}
