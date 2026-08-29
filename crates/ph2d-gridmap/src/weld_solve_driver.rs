//! ⭐⭐⭐ **O CONDUTOR DO G3 SOLDADO** — o relatório, as constantes e as duas portas
//! públicas ([`solve_welded`] / [`solve_welded_with`]).
//!
//! ⚠️ **Separado do [`crate::weld_solve`] por RESPONSABILIDADE, não por tamanho:** lá vive
//! o **relaxador** (uma variável de cada vez, e as leis de quem a escreve); aqui vive quem
//! o **conduz** — quantas rondas, quantas passagens de endurecimento, o que se mede — e a
//! álgebra `2×2` que ele usa. *O corte foi forçado pelo tecto de LOC; a linha do corte foi
//! escolhida pela pergunta «quem lê isto, e porquê».*

use ph2d_mesh::Mesh;

use crate::comb::Combed;
use crate::cut::CutMesh;
use crate::solve::{GridMap, SolveReport, Step, assemble, measure};
use crate::weld::{WeldReport, weld};
use crate::weld_flat::{ClosureSystem, FlatReport};
use crate::weld_solve::WeldRelaxer;

/// Quantas rondas de Gauss–Seidel o sistema soldado gasta.
///
/// ⭐⭐ **É `20×` menos que o penalizado, e o número sai de medição** — o sistema
/// penalizado é mal condicionado **por causa do peso**; sem ele é a Poisson pura.
pub const ROUNDS: usize = 8_000;

/// O que o solver soldado mediu de si próprio, além das réguas do mapa.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WeldSolveReport {
    /// As réguas do mapa — as mesmas do [`crate::solve`].
    pub solve: SolveReport,
    /// A estrutura da soldadura.
    pub weld: WeldReport,
    /// O sistema dos fechos.
    pub flat: FlatReport,
    /// Rondas gastas.
    pub rounds: usize,
    /// ⭐ O maior movimento da última ronda — a régua da convergência.
    pub last_move: f32,
    /// ⭐⭐ **Triângulos VIRADOS depois da 1.ª resolução** — antes de endurecer nada.
    pub folded_before: usize,
    /// ⭐⭐⭐ **Triângulos VIRADOS no fim** — a régua do endurecimento local.
    pub folded_after: usize,
    /// Quantas passagens de endurecimento correram de facto.
    pub stiffen_passes: usize,
    /// ⭐⭐⭐ Grupos de escalares **amarrados** pelos arcos que entraram de facto.
    pub tie_groups: usize,
    /// ⛔⛔ Grupos **RECUSADOS** — algum membro já tem quem o escreva.
    ///
    /// ⚠️ *Recusa-se o grupo INTEIRO, e conta-se.* Aceitar metade seria pôr duas leis
    /// sobre a mesma variável — o defeito que a obra A mediu nos dois subsistemas.
    pub tie_refused: usize,
    /// ⭐ A razão da recusa: `[dependente, livre do sistema, pregada]`.
    pub tie_refused_why: [usize; 3],
    /// ⭐ `H / H_fingida` das amarras — o **ganho** que a 1.ª redacção aplicava ao passo
    /// (ver [`WeldRelaxer::tie_gain`]). `1,0` = o denominador escalar estava certo.
    pub tie_gain_p50: f32,
    /// O pior ganho de todos os grupos.
    pub tie_gain_max: f32,
    /// ⭐⭐ Grupos cuja RAIZ é classe simples — ver [`WeldRelaxer::plain_roots`].
    pub tie_plain_roots: usize,
    /// ⭐ A 1.ª ronda em que o mapa contínuo deixa de ser finito (`0` = nunca).
    pub nonfinite_round: usize,
    /// O maior movimento da varredura NESSA ronda.
    pub nonfinite_move: f32,
    /// ⭐⭐ Qual escritor estourou: `0` classe · `1` amarra · `2` livre · `3` ciclo de
    /// arco · `4` nenhum deles (o mapa já vinha torto).
    pub nonfinite_who: usize,
    /// ⛔⛔⛔ **Coordenadas NÃO-FINITAS no mapa, no fim do contínuo.**
    ///
    /// ⚠️ *Sem esta coluna, «o solver divergiu» e «o solver produziu um mapa são que
    /// alguém estragou depois» leem o mesmo `NaN` no fim da cadeia* — e as duas pedem
    /// curas em fases diferentes.
    pub nonfinite: usize,
    /// A mesma contagem **logo após a 1.ª ronda** — separa «nasceu torto» de «degradou».
    pub nonfinite_first: usize,
    /// ⭐⭐⭐ **Equações de CICLO de arco que entraram** — o A3, a condição sobre as
    /// translações.
    pub arc_cycles: usize,
}

/// ⭐ **A folga do posto de [`solve2`], RELATIVA à escala da própria matriz.**
///
/// ⛔⛔⛔ **Medido (2026-08-27): o limiar ABSOLUTO era o `inf` da `sphere_uv`.** O guarda
/// era `> 1,0e-12` sobre um sistema normal cuja escala é `Σ den·|J|²` — um número que não
/// tem unidade fixa, e que muda com o tamanho da peça e com `h`. ⇒ uma coluna
/// **numericamente nula** passava o guarda e devolvia um passo enorme.
///
/// ⚠️ **Só aparece com as amarras porque congelar um eixo é o que escolhe o ramo `1-D`**,
/// em que o divisor deixa de ser o determinante e passa a ser `h[k][k]` sozinho: a matriz
/// `2×2` inteira pode estar bem condicionada e aquela coluna não. *A esfera derivava até
/// `inf` na ronda `6134` — devagar, que é a assinatura de um raio espectral pouco acima
/// de 1, e não de um passo isolado a estourar.*
const RANK_REL: f32 = 1.0e-6;

/// ⭐ **De quantas em quantas rondas a rede de não-finitos varre o mapa** — ver o uso.
///
/// ⚠️ **É um número de DIAGNÓSTICO, não de produto:** ele decide a precisão com que a
/// «ronda em que estourou» é conhecida, e nada mais. A `1` (o que estava) a varredura
/// custava `O(V)` em toda ronda de toda peça sã.
const NONFINITE_PROBE: usize = 64;

/// ⭐⭐⭐ **QUANDO O CONTÍNUO DESISTE** — o maior movimento de uma ronda, em **células da
/// grade**.
///
/// ⛔⛔⛔ **O laço não tinha saída nenhuma: ele gastava [`ROUNDS`] sempre.** Medido em
/// 2026-08-28 na `sculpt_wrinkled` (densidade do botão), variando só o tecto de rondas:
///
/// | rondas | aspecto p50 / p99 | enviesamento p50 / p99 |
/// |---|---|---|
/// | **8 000** (o que shipava) | `1,10` / `1,49` | `4,5°` / `32,0°` |
/// | 4 000 | `1,10` / `1,49` | `4,5°` / `32,0°` |
/// | 2 000 | `1,10` / `1,50` | `4,6°` / `32,1°` |
/// | 1 000 | `1,09` / `1,42` | `4,4°` / `31,3°` |
/// | ⭐ 500 | `1,09` / `1,45` | **`4,2°`** / `31,4°` |
///
/// ⇒ **as `8 000` não compram nada** — a saída com `500` é igual ou ligeiramente melhor, e a
/// diferença está dentro da banda de caos do guloso. ⚠️ *Mas um tecto de rondas é uma cerca
/// cujo tamanho muda com a peça* (a taxa de convergência de Gauss–Seidel escala com `N`), e
/// foi essa a lição que o acabamento já pagou: **a lei é o assentamento**, o tecto é a rede.
///
/// ⭐ O número é em células porque é isso que o mapa mede, e o passo seguinte **arredonda
/// para inteiros**: um movimento abaixo disto não pode mudar nenhuma decisão a jusante.
pub const WELD_SETTLE: f32 = 1.0e-6;

/// ⭐⭐⭐ **QUANTAS RONDAS SEM DESCER ANTES DE DESISTIR** — e é esta que manda, não a
/// [`WELD_SETTLE`].
///
/// ⛔⛔⛔ **Medido 2026-08-28, `sculpt_wrinkled`, a curva do maior movimento:**
///
/// ```text
///   ronda    0 : 5,33e-1        ronda 2000 : 1,25e-5
///   ronda  500 : 1,09e-2        ronda 2500 : 3,25e-6
///   ronda 1000 : 1,09e-3        ronda 2750 : 3,27e-6   ← o PATAMAR
///   ronda 1500 : 1,02e-4        ronda 3000..8000 : 3,271e-6, constante
/// ```
///
/// ⇒ **a partir da ronda ~2 750 o movimento é ruído de `f32` e não desce mais.** As últimas
/// `5 250` rondas (`66 %`) reciclam o mesmo número. ⚠️ *Um limiar absoluto não apanha isto —
/// o patamar fica ACIMA de qualquer limiar honesto e depende da peça.* A lei que apanha é a
/// mesma que o acabamento já pagou: **desistir quando deixa de melhorar.**
pub const WELD_PATIENCE: usize = 256;

/// Quanto é preciso descer para contar como descida — abaixo disto é o mesmo número.
const DESCENT: f32 = 0.999;

/// A porta de teste da [`solve2`] — ela é livre e privada, e o gate da folga de posto
/// precisa de a chamar directamente.
#[cfg(test)]
pub(crate) fn solve2_pub(h: [[f32; 2]; 2], g: [f32; 2], frozen: [bool; 2]) -> Option<[f32; 2]> {
    solve2(h, g, frozen)
}

/// Resolve `H·x = g` em `2×2`, honrando as componentes pregadas.
///
/// ⚠️ **A escala é o TRAÇO**, e ele é sempre `≥ 0`: as diagonais são somas de `den·|col|²`.
pub(crate) fn solve2(h: [[f32; 2]; 2], g: [f32; 2], frozen: [bool; 2]) -> Option<[f32; 2]> {
    let scale = h[0][0] + h[1][1];
    match frozen {
        [true, true] => None,
        [true, false] => (h[1][1] > RANK_REL * scale).then(|| [0.0, g[1] / h[1][1]]),
        [false, true] => (h[0][0] > RANK_REL * scale).then(|| [g[0] / h[0][0], 0.0]),
        [false, false] => {
            let d = h[0][0].mul_add(h[1][1], -(h[0][1] * h[1][0]));
            (d.abs() > RANK_REL * scale * scale).then(|| {
                [
                    g[0].mul_add(h[1][1], -(h[0][1] * g[1])) / d,
                    h[0][0].mul_add(g[1], -(g[0] * h[1][0])) / d,
                ]
            })
        }
    }
}

/// ⭐⭐⭐ **RESOLVE O MAPA COM AS COSTURAS ELIMINADAS.**
#[must_use]
/// ⭐⭐⭐ **QUANTAS VEZES o endurecimento local corre** (MIQ 2009 §5.4).
///
/// A lei publicada: uma parametrização harmónica **não é injectiva** por construção — onde
/// a superfície comprime, triângulos viram do avesso. A cura é **pesar mais** os triângulos
/// virados e **resolver outra vez**, repetindo: cada passagem torna aquela região mais
/// rígida e empurra a distorção para onde há folga.
///
/// ⛔ **O número tem de ser MEDIDO** (`CLAUDE.md` §0.0) — ver a tabela em
/// [`STIFFEN_FACTOR`]. `0` devolve o caminho antigo bit a bit.
///
/// ⚠️ **O preço é linear**: cada passagem é uma resolução inteira do contínuo.
fn stiffen_passes(_rounds: usize) -> usize {
    std::env::var("PH2D_STIFFEN_PASSES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(STIFFEN_PASSES)
}

/// ⛔⛔⛔ **ZERO — o endurecimento local foi CONSTRUÍDO, MEDIDO e REJEITADO.**
///
/// A `0` esta função é bit a bit o caminho de sempre, e há gate a afirmá-lo
/// (`stiffening_at_zero_passes_is_the_old_path`).
///
/// # ⛔ A tabela da rejeição (2026-08-25, peça do artista, cadeia inteira)
///
/// | passagens | factor | triângulos VIRADOS | bordo | `χ` | enviesamento p50 | `>60°` |
/// |---|---|---|---|---|---|---|
/// | ⭐ **`0`** | — | **`14` ⇒ `14`** | **`10`** | **`1`** | `7,3°` | `5` |
/// | 2 | `2` | ⛔ `14` ⇒ **`15`** | `12` | `1` | `7,8°` | `2` |
/// | 5 | `2` | ⛔ `14` ⇒ **`18`** | `14` | `0` | `7,8°` | `7` |
/// | 2 | `4` | ⛔ `14` ⇒ **`19`** | `12` | `1` | `7,4°` | `5` |
/// | 5 | `4` | ⛔ `14` ⇒ **`23`** | `12` | `1` | `7,0°` | `3` |
/// | 2 | `10` | ⛔ `14` ⇒ **`19`** | `14` | `1` | `7,4°` | `5` |
/// | 2 | `0,5` | `14` ⇒ `14` | `10` | `1` | `6,7°` | `4` |
/// | 2 | `0,25` | `14` ⇒ `14` | ⛔ `18` | ⛔ `0` | `7,3°` | `4` |
/// | 2 | `0,1` | `14` ⇒ `14` | `12` | `1` | `7,5°` | `6` |
///
/// # ⭐⭐⭐ O MECANISMO, e é ele que fecha a família
///
/// ⛔ **Endurecer AUMENTA as dobras** — monotonamente no factor e nas passagens. Isso não
/// é afinação a faltar: é o sinal de que a cura publicada assume **outra energia**.
///
/// No *paper* a parametrização é **harmónica**, e pesar mais um triângulo virado força-o a
/// ser mais rígido, o que o desvira. ⭐⭐ **A nossa energia não é essa: ela é
/// `Σ area·|∇z − X/h|²`, ou seja «SEGUIR O CAMPO».** Pesar mais um triângulo virado manda-o
/// obedecer ao campo **com mais força** — exactamente no sítio onde obedecer ao campo é o
/// que o vira. *A cura empurra na direcção do defeito.*
///
/// ⚠️ **E o sinal contrário também não serve:** amolecer (`factor < 1`) deixa a contagem de
/// dobras **exactamente igual** (`14` ⇒ `14`) e move as outras colunas dentro da banda de
/// caos que o guloso já tem — medida à parte, `8k/16k/32k/64k` rondas dão `14/14/10/12`
/// arestas de bordo sobre o **mesmo** mapa. *Uma melhoria menor que a banda de caos do
/// sistema não é uma melhoria; é uma amostra.*
///
/// ⇒ **Duas hipóteses boas da mesma família falharam ⇒ a família está fechada.** A dobra
/// no domínio não se cura pesando triângulos: ela é uma propriedade do CAMPO que o mapa
/// está a seguir.
///
/// ⭐ **O que fica de valor é a RÉGUA:** [`WeldSolveReport::folded_before`] /
/// [`WeldSolveReport::folded_after`] medem as dobras no **contínuo**, antes do
/// arredondamento inteiro — e essa contagem não existia. A sonda independente do
/// `chain_info` mede-as **depois** (`20`), e as duas juntas dizem quanto o arredondamento
/// acrescenta.
pub const STIFFEN_PASSES: usize = 0;

/// ⭐ **POR QUANTO o peso de um triângulo virado é multiplicado, por passagem.**
///
/// ⛔ Inerte enquanto [`STIFFEN_PASSES`] for `0` — ver a tabela da rejeição lá.
pub const STIFFEN_FACTOR: f32 = 2.0;

fn stiffen_factor() -> f32 {
    std::env::var("PH2D_STIFFEN_FACTOR")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(STIFFEN_FACTOR)
}

pub fn solve_welded(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    step: Step<'_>,
    rounds: usize,
) -> (GridMap, WeldSolveReport) {
    solve_welded_with(mesh, cut, combed, step, rounds, None, None)
}

/// ⭐⭐⭐ **O MESMO, com as AMARRAS DOS ARCOS ligadas.**
///
/// ⚠️ **A escolha do eixo atravessado de cada arco sai do mapa que gerou as amarras**, que
/// na cadeia é a solução **livre**. *É por isso que isto é um segundo passe e não um
/// parâmetro:* a restrição precisa de saber para que lado cada arco corre, e quem o diz é
/// a solução sem ela.
///
/// ⛔ `ties = None` é **byte-idêntico** ao [`solve_welded`] — é o controlo.
#[must_use]
pub fn solve_welded_with(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    step: Step<'_>,
    rounds: usize,
    ties: Option<&crate::arcline::ScalarTies>,
    arcs: Option<(&[crate::arcline::ArcEquation], &[usize])>,
) -> (GridMap, WeldSolveReport) {
    solve_welded_settling(mesh, cut, combed, step, rounds, WELD_SETTLE, ties, arcs)
}

/// O mesmo, com o limiar de assentamento **à vista** — ver [`WELD_SETTLE`].
///
/// ⚠️ **Ele é um parâmetro e não uma variável de ambiente** pela lição que o acabamento
/// pagou em 2026-08-28: um limiar relativo só se calibra na composição em que corre, e uma
/// varredura tem de passar pela **porta** para não medir outro programa.
#[allow(clippy::too_many_arguments)]
pub fn solve_welded_settling(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    step: Step<'_>,
    rounds: usize,
    settle: f32,
    ties: Option<&crate::arcline::ScalarTies>,
    arcs: Option<(&[crate::arcline::ArcEquation], &[usize])>,
) -> (GridMap, WeldSolveReport) {
    let mut rep = WeldSolveReport::default();
    let (w, wrep) = weld(cut, combed);
    rep.weld = wrep;
    let mut a = assemble(mesh, cut, combed, step, &mut rep.solve);
    let jumped: Vec<bool> = (0..cut.seams.len())
        .map(|s| combed.jump.get(s).copied().flatten().is_some())
        .collect();
    rep.flat = ClosureSystem::build(&w, cut.seams.len(), &jumped).1;
    let mut map = GridMap {
        uv: cut
            .origin
            .iter()
            .map(|o| vec![[0.0f32; 2]; o.len()])
            .collect(),
        shift: vec![[0.0; 2]; cut.seams.len()],
    };
    // ⭐⭐⭐ **O ENDURECIMENTO LOCAL** — ver [`STIFFEN_PASSES`].
    //
    // ⚠️ **A 1.ª passagem é a de sempre, bit a bit**, e é isso que faz `passes = 0` ser o
    // caminho antigo: só depois de uma solução existir é que há dobras para endurecer.
    for pass in 0..=stiffen_passes(rounds) {
        let mut r = WeldRelaxer::new(&a, &w, cut, combed);
        if let Some(t) = ties {
            r.attach_ties(t);
            // ⭐⭐⭐ **O A3:** as equações que FECHAM CICLO passam a possuir um escalar de
            // translação. ⚠️ *Sem elas a condição não está no sistema* — e a §23.18 mediu
            // o preço: a esfera diverge.
            if let Some((eqs, cyc)) = arcs {
                r.attach_arc_cycles(eqs, cyc);
            }
            let (g, refused, why) = r.tie_counts();
            rep.tie_groups = g;
            rep.tie_refused = refused;
            rep.tie_refused_why = why;
            rep.arc_cycles = r.arc_cycle_count();
            rep.tie_plain_roots = r.plain_roots();
            // ⭐ A régua do defeito que o denominador fingido tinha — ver
            // [`WeldRelaxer::tie_gain`]. Lê-se sobre o mapa de PARTIDA, que é onde o
            // passo da 1.ª ronda de facto se calculava.
            let mut gains: Vec<f32> = (0..g).filter_map(|k| r.tie_gain(&map, k)).collect();
            gains.sort_by(f32::total_cmp);
            if !gains.is_empty() {
                rep.tie_gain_p50 = gains[gains.len() / 2];
                rep.tie_gain_max = gains[gains.len() - 1];
            }
        }
        let count_bad = |m: &GridMap| -> usize {
            m.uv.iter()
                .flatten()
                .filter(|z| !z[0].is_finite() || !z[1].is_finite())
                .count()
                + m.shift
                    .iter()
                    .filter(|t| !t[0].is_finite() || !t[1].is_finite())
                    .count()
        };
        // ⚠️ **A leitura da variável de ambiente sai do laço** — `env::var` aloca, e no
        // caminho quente ela custava uma alocação por ronda por peça.
        let trace = std::env::var("PH2D_G3_TRACE").as_deref() == Ok("1");
        // ⚠️ **A paciência é varrível pela porta** (`PH2D_G3_PATIENCE`), lida UMA vez: a
        // lição do `EXTRACT_SETTLE` é que um limiar só se calibra na composição em que corre,
        // e uma varredura que não passa pela porta mede outro programa. O default é a
        // constante, que é o que o produto usa.
        let patience: usize = std::env::var("PH2D_G3_PATIENCE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(WELD_PATIENCE);
        let (mut best_move, mut stuck) = (f32::INFINITY, 0usize);
        for round in 0..rounds {
            let parts = r.sweep_parts(&mut map);
            rep.last_move = parts.iter().copied().fold(0.0f32, f32::max);
            rep.rounds = round + 1;
            if round == 0 {
                rep.nonfinite_first = count_bad(&map);
            }
            // ⭐ A RONDA EM QUE O CONTÍNUO ESTOURA — *«3119 no fim» e «0 na 1.ª» não
            // distinguem uma deriva lenta de um rebentamento súbito*, e a cura de cada
            // uma é outra. ⭐⭐ E o ESCRITOR que a produziu, que é o que escolhe a cura.
            //
            // ⛔⛔ **A varredura NÃO pode correr em toda ronda, e corria** (medido
            // 2026-08-28): o `count_bad` percorre `uv` e `shift` inteiros, e a condição
            // avaliava-o **`8 000` vezes** em toda peça sã — uma sonda de diagnóstico no
            // caminho quente, exactamente o defeito que o acabamento tinha com o `rebuild`.
            //
            // ⭐ O sintoma BARATO é o movimento: um `NaN` num `uv` nasce de um passo `NaN`,
            // e esse aparece em `parts`. ⚠️ **Ele não é suficiente sozinho** — um `uv` pode
            // virar `inf` por *overflow* de `f32` com um passo finito —, e por isso a
            // varredura continua a correr, **periodicamente**. *A ronda fica conhecida a
            // menos de [`NONFINITE_PROBE`], que é folga de sobra para distinguir uma deriva
            // lenta de um rebentamento súbito — que é o que esta coluna existe para dizer.*
            // ⭐⭐ **Assentou: as rondas seguintes não podem mudar nada.** Ver
            // [`WELD_SETTLE`]. ⚠️ Só depois de a ronda ser contada e o `last_move` escrito —
            // o relatório tem de descrever a corrida que de facto houve.
            if trace && (round < 8 || round % 250 == 0) {
                eprintln!("      G3 ronda {round}: last_move {:.3e}", rep.last_move);
            }
            // ⭐⭐⭐ **Assentou, OU deixou de descer** — ver [`WELD_SETTLE`].
            if rep.last_move <= settle {
                break;
            }
            if rep.last_move < best_move * DESCENT {
                best_move = rep.last_move;
                stuck = 0;
            } else {
                stuck += 1;
                if stuck >= patience {
                    break;
                }
            }
            let bad_step = parts.iter().any(|v| !v.is_finite());
            if rep.nonfinite_round == 0
                && (bad_step || round % NONFINITE_PROBE == 0)
                && count_bad(&map) > 0
            {
                rep.nonfinite_round = round + 1;
                rep.nonfinite_move = rep.last_move;
                rep.nonfinite_who = parts.iter().position(|v| !v.is_finite()).unwrap_or(4);
            }
        }
        rep.nonfinite = count_bad(&map);
        let folded = a.folded(&map);
        rep.folded_after = folded.len();
        if pass == 0 {
            rep.folded_before = folded.len();
        }
        if folded.is_empty() || pass == stiffen_passes(rounds) {
            break;
        }
        rep.stiffen_passes = pass + 1;
        a.stiffen(&folded, stiffen_factor());
    }
    measure(&a, cut, combed, &map, step.h, &mut rep.solve);
    crate::weld::holonomy(&w, &map, &mut rep.weld);
    (map, rep)
}
