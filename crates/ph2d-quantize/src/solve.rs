//! **O SOLVER** — dupla cobertura, simetrização, e o limite que CERTIFICA.
//!
//! # A dupla cobertura, em três frases
//!
//! Duplique cada nó em `v⁺` e `v⁻`. Cada aresta bi-dirigida vira **duas** arestas
//! dirigidas comuns que cruzam entre as duas camadas, escolhidas de modo que a
//! conservação em `v⁺` reproduza exatamente a conservação bi-dirigida em `v`, e a
//! de `v⁻` a reproduza negada. O resultado é uma rede de fluxo **normal**, que
//! Dijkstra resolve exatamente ([`crate::mcf`]).
//!
//! ⭐ **O que se ganha com isso é uma PROVA.** Todo fluxo bi-dirigido viável vira
//! um fluxo simétrico na dupla cobertura, de custo exatamente o dobro. Logo o
//! ótimo da dupla cobertura, dividido por dois, é um **limite inferior** do ótimo
//! inteiro que se procura — e ele vale mesmo quando não conseguimos atingi-lo.
//! *Um número de qualidade sem limite ao lado é uma opinião.*
//!
//! # E o que se perde
//!
//! Um ótimo da dupla cobertura pode ser **assimétrico**. Espelhar as duas
//! camadas dá outro ótimo, e a média dos dois é simétrica — mas **meio-inteira**.
//! Onde ela cai em inteiro, o resultado é ótimo demonstrado. Onde não cai, um
//! **mergulho** prende as arestas rebeldes num inteiro (sempre terminando, sempre
//! válido) e um **ramifica-e-limita** melhora a partir daí, usando a mesma dupla
//! cobertura como limite de cada ramo. Se a fila esgotar dentro do orçamento, o
//! [`Report::proved`] fica `true` e o custo é o ótimo inteiro; se não, o `gap`
//! diz exatamente quão longe do certificado a resposta ficou.
//!
//! ⛔ **O solver EXATO do §3.7 (matching / Blossom) não está aqui.** Ele é a cura
//! da meia-integralidade, e o seu preço é uma implementação de emparelhamento
//! geral com custos. Construí-lo antes de medir quantas arestas de facto saem
//! meio-inteiras seria escrever um limite sem a tabela ao lado.

use std::collections::BTreeMap;

use crate::mcf::{Mcf, McfError};
use crate::network::{self, BiNetwork};
use crate::report::Report;
use crate::{CornerError, Layout, Quantization, verify};

/// O que impede uma quantização de existir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolveError {
    /// ⛔ **Nem a relaxação FRACIONÁRIA fecha.** Não existe quantização regular
    /// para este layout, nem sequer com comprimentos reais — algum patch
    /// precisaria de mais de um vértice irregular. É a porta do *emergency node*
    /// do §4.4.1 — ver o doc de [`crate::network`].
    Infeasible,
    /// ⚠️ **A relaxação fecha, mas nenhum INTEIRO foi encontrado.** É mais fraco
    /// que [`SolveError::Infeasible`]: uma solução inteira pode existir e o
    /// mergulho não a ter alcançado. Distingui-las importa — uma diz *"muda o
    /// layout"*, a outra diz *"melhora a busca"*.
    NoIntegral,
    /// O fluxo fechou mas um patch não fecha. ⚠️ Isto é um **bug do solver**, não
    /// uma propriedade do layout: a conservação da rede é a lei do patch.
    Inconsistent(CornerError),
    /// ⚠️ **O orçamento de resoluções acabou antes de haver resposta.** É
    /// diferente de [`SolveError::Infeasible`] de propósito: aquilo é uma
    /// afirmação sobre o LAYOUT, isto é uma sobre o SOLVER. Confundi-los faria
    /// um layout perfeitamente quantizável parecer impossível.
    Exhausted {
        /// Quantas resoluções de fluxo foram gastas.
        solves: usize,
    },
}

/// **QUANTIZA UM LAYOUT.**
///
/// # Errors
/// Ver [`quantize_within`], que é onde este atalho vai dar.
pub fn quantize(layout: &Layout) -> Result<(Quantization, Report), SolveError> {
    crate::refine::quantize_within(layout, Budget::default())
}

/// **O ORÇAMENTO da fase**, nas duas unidades que ela de facto gasta.
///
/// ⚠️ **São dois porque falham por motivos diferentes.** As *expansões* limitam a
/// BUSCA, que é opcional — gastá-las devolve uma resposta válida sem prova. As
/// *resoluções de fluxo* limitam também o MERGULHO, que é obrigatório — gastá-las
/// devolve [`SolveError::Exhausted`]. Um orçamento só não distingue *"boa sem
/// prova"* de *"não deu"*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    /// Quantos nós o ramifica-e-limita pode expandir.
    pub expansions: usize,
    /// Quantas resoluções de fluxo a fase inteira pode gastar.
    pub solves: usize,
    /// ⭐ Quantos **aumentos** cada resolução de fluxo pode gastar. É a unidade
    /// de esforço REAL: o relógio de uma resolução não é função do tamanho do
    /// grafo, é de quanto desequilíbrio há para rotear (medido: uma grelha
    /// uniforme de 8 192 arcos custa 5 ms; a mesma com alvos dispersos, 1,6 s).
    pub augmentations: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            expansions: MAX_EXPANSIONS,
            solves: MAX_SOLVES,
            augmentations: MAX_AUGMENTATIONS,
        }
    }
}

impl Budget {
    /// Um orçamento explícito, com o teto de aumentos no default medido.
    #[must_use]
    pub fn new(expansions: usize, solves: usize) -> Self {
        Self {
            expansions,
            solves,
            ..Self::default()
        }
    }
}

/// **QUANTIZA com um ORÇAMENTO de busca.**
///
/// ⚠️ **O orçamento nunca troca uma resposta por nenhuma.** Antes de a busca
/// começar, um **mergulho** produz uma quantização válida; a busca só serve para
/// a melhorar. Gastar o orçamento devolve essa resposta com [`Report::proved`] em
/// `false` — nunca um erro.
///
/// # Errors
/// [`SolveError::Infeasible`] se nenhuma quantização regular existe;
/// [`SolveError::Exhausted`] se o mergulho gastou [`MAX_SOLVES`] resoluções de
/// fluxo antes de chegar a uma resposta — ⚠️ afirmação sobre o **solver**, não
/// sobre o layout; [`SolveError::Inconsistent`] se o fluxo fechou mas um patch
/// não, que é um bug.
impl From<CornerError> for SolveError {
    fn from(e: CornerError) -> Self {
        Self::Inconsistent(e)
    }
}

pub(crate) fn attempt(
    layout: &Layout,
    budget: Budget,
    guess: Option<&[i64]>,
) -> Result<(Quantization, Report, Vec<i64>), SolveError> {
    // ⚠️ **Uma resolução de fluxo é a unidade de custo desta fase**, e o mergulho
    // pode precisar de muitas. Contá-las é o que impede o solver de correr sem
    // fim num layout patológico — ver [`MAX_SOLVES`].
    let mut work = Work::default();
    // ⭐ **O teto ESCALA em vez de mentir.** Um teto apertado é rápido, e quando
    // ele não chega o solver diria *"não existe quantização"* — uma afirmação
    // sobre o layout — quando o que não cabia era o teto. Medido em 2026-08-20 na
    // `sphere_noisy`. Cada degrau só corre se o anterior tiver recusado.
    let mut net = BiNetwork::build_centred(layout, network::CAP_STEPS[0], guess);
    let mut root = run(&net, &BTreeMap::new(), &mut work, budget.augmentations);
    let mut cap_step = network::CAP_STEPS[0];
    for &step in &network::CAP_STEPS[1..] {
        if root.is_ok() {
            break;
        }
        cap_step = step;
        net = BiNetwork::build_centred(layout, step, guess);
        work = Work::default();
        root = run(&net, &BTreeMap::new(), &mut work, budget.augmentations);
    }
    let root = root.map_err(|e| match e {
        McfError::Exhausted { .. } => SolveError::Exhausted { solves: 1 },
        McfError::Infeasible { .. } => SolveError::Infeasible,
    })?;
    let base: f64 = net.edges().iter().map(|e| e.cost(e.lo)).sum();
    let lower_bound = base + root.dc_cost / 2.0;
    let half_integral = root.num.iter().filter(|n| *n % 2 != 0).count();

    // ⭐ **Ramifica-e-limita sobre as arestas meio-inteiras.** Cada nó da busca
    // aperta a FAIXA de algumas arestas; o seu `bound` é o ótimo da dupla
    // cobertura **daquele ramo**, que é um limite inferior legítimo de qualquer
    // inteiro que ainda caiba nele. Expandir por ordem de `bound` e parar quando
    // o melhor limite da fila já não bate o incumbente é a busca **exata**.
    //
    // ⚠️ **Os dois filhos são `x <= piso` e `x >= piso+1`, nunca `x = piso` e
    // `x = piso+1`.** Ramificar em PONTOS parece equivalente e não é: ele
    // descarta, sem aviso, todo inteiro que não seja um dos dois — e a busca
    // continua a esgotar-se e a declarar-se provada. Medido em 2026-08-20: a
    // `sculpt_hooked` devolvia **29,86** numa configuração e **29,92** noutra,
    // as duas com `prova = sim`. *Duas provas do mesmo ótimo não podem discordar;
    // uma delas não era prova.* Meias-retas particionam, pontos não.
    //
    // ⭐ **O incumbente vem ANTES da busca**, pelo mergulho — é o que garante que
    // gastar o orçamento devolve uma resposta válida em vez de um erro, e dá à
    // busca um teto contra o qual podar desde o primeiro nó.
    let mut best = dive(&net, layout, &root, &mut work, budget)?;
    let mut heap: Vec<Node> = vec![Node {
        bound: lower_bound,
        range: BTreeMap::new(),
        round: root,
    }];
    let mut expansions = 0usize;
    let mut proved = true;
    while !heap.is_empty() {
        // Menor limite primeiro; empate pelo nó mais fundo (mais decidido), e
        // depois pela ordem de inserção — determinístico dos dois lados.
        let pick = (0..heap.len())
            .min_by(|&i, &j| {
                heap[i]
                    .bound
                    .total_cmp(&heap[j].bound)
                    .then_with(|| heap[j].range.len().cmp(&heap[i].range.len()))
                    .then_with(|| i.cmp(&j))
            })
            .expect("a fila nao esta' vazia");
        let node = heap.swap_remove(pick);
        if node.bound >= best.0 - 1e-9 {
            break;
        }
        // ⚠️ O teto é conferido ANTES de contar: `expansions` diz quantos nós
        // foram de facto expandidos, e um orçamento de zero tem de dar zero.
        if expansions >= budget.expansions || work.solves >= budget.solves {
            proved = false;
            break;
        }
        expansions += 1;
        let Some(bad) = node.round.num.iter().position(|n| n % 2 != 0) else {
            let flow = flow_of(&node.round);
            let cost = cost_of(&net, layout, &flow);
            if cost < best.0 {
                best = (cost, flow);
            }
            continue;
        };
        let edge = &net.edges()[bad];
        let (lo, hi) = node.range.get(&bad).copied().unwrap_or((edge.lo, edge.hi));
        // O piso do meio-inteiro: é aí que a faixa se parte.
        let cut = node.round.lo[bad] + node.round.num[bad] / 2;
        for (a, b) in branch(lo, hi, cut) {
            if a > b {
                continue;
            }
            let mut range = node.range.clone();
            range.insert(bad, (a, b));
            if let Ok(round) = run(&net, &range, &mut work, budget.augmentations) {
                heap.push(Node {
                    bound: base + round.dc_cost / 2.0,
                    range,
                    round,
                });
            }
        }
    }
    let (cost, flow) = best;
    // ⭐ **O tamanho REAL da rede de fluxo** — ver [`Report::mcf_arcs`]. Medido na
    // raiz (sem os cortes do ramifica-e-limita), que é a que todo nó da busca
    // volta a percorrer.
    let mcf_arcs: usize = net
        .edges()
        .iter()
        .map(|e| crate::refine::segments(e, e.lo, e.hi).len() * 2)
        .sum();
    debug_assert!(
        net.residual(&flow).iter().all(|r| *r == 0),
        "a conservacao da rede bi-dirigida E' a lei do patch: violá-la aqui e' bug do solver"
    );

    let mut x = vec![0u32; layout.arcs().len()];
    let mut cap_binding = 0usize;
    for (e, edge) in net.edges().iter().enumerate() {
        if flow[e] >= edge.hi {
            cap_binding += 1;
        }
        if let Some(a) = edge.arc {
            x[a as usize] = u32::try_from(flow[e]).unwrap_or(u32::MAX);
        }
    }
    let corners = verify(layout, &x)?;
    // ⭐⭐ **Quantas arestas acabaram FORA da janela exacta** — ver
    // [`MAX_EXACT_DEVIATION`]. Zero quer dizer que a linearização nunca mordeu, e
    // aí o `lower_bound` é do problema verdadeiro e a prova vale.
    let outside_window = net
        .edges()
        .iter()
        .enumerate()
        // ⚠️ **Só as arestas com CUSTO têm escada.** Uma aresta de leque tem peso
        // zero e vira **um** arco livre em [`segments`] — não há janela para ela
        // morder. Contá-la aqui dava `fora-da-janela = 4` num layout cuja resposta
        // tinha `gap = 0,000`, ou seja: a prova era negada por arestas que não
        // participam da aproximação. *Um falso negativo numa prova custa tanto
        // quanto um falso positivo — ele manda refinar o que já estava certo.*
        .filter(|(e, edge)| {
            edge.weight != 0.0 && (flow[*e] - edge.guess).abs() > crate::refine::MAX_EXACT_DEVIATION
        })
        .count();
    Ok((
        Quantization { arc: x, corners },
        Report {
            cost,
            lower_bound,
            gap: cost - lower_bound,
            half_integral,
            expansions,
            solves: work.solves,
            augmentations: work.augmentations,
            // ⚠️ **A prova exige a janela INTACTA.** Fora dela o custo da rede é
            // uma linearização, então o limite inferior é de outro problema — e um
            // `gap` calculado entre os dois pode até sair **negativo**, que foi o
            // que a primeira versão desta escada imprimiu. *Um número que não pode
            // ser negativo e sai negativo é a prova a dizer que não é prova.*
            proved: proved && outside_window == 0,
            cap_binding,
            nodes: net.nodes(),
            edges: net.edges().len(),
            mcf_arcs,
            cap_step,
            outside_window,
            refinements: 0,
        },
        flow,
    ))
}

/// ⚠️ **O teto de expansões, e o que ele é.** Não é uma opinião sobre qualidade:
/// é o ponto a partir do qual a busca deixa de ser barata. Quem o atinge sai com
/// [`Report::proved`] em `false` e uma resposta **válida**, nunca uma inventada.
/// Medido sobre o corpus do oráculo em 2026-08-20: o pior layout **fechado**
/// (`sculpt_hooked`, 15 patches) gastou **222** — a tabela está no `PLAN.md`
/// §4-quater.
pub const MAX_EXPANSIONS: usize = 4096;

/// ⚠️ **O teto de RESOLUÇÕES DE FLUXO, e por que ele existe além do de expansões.**
/// O mergulho corre **antes** da busca e não é opcional: sem teto próprio, um
/// layout patológico pode ficar a resolver fluxo indefinidamente e nunca chegar à
/// busca, onde o orçamento moraria.
///
/// ⚠️ **Ele conta resoluções, não segundos.** Uma resolução não custa o mesmo em
/// todo layout — [`tests/scaling.rs`] mede o crescimento — então este número é
/// um limite de **trabalho**, e o relógio que ele compra depende do tamanho.
/// Medido 2026-08-20 sobre o corpus do oráculo: o pior layout **fechado**
/// (`sculpt_hooked`, 15 patches) gastou algumas centenas; o teto está uma ordem
/// de grandeza acima disso, de propósito.
pub const MAX_SOLVES: usize = 4096;

/// ⚠️ **O teto de AUMENTOS da fase inteira.** É o único dos três que limita o
/// relógio de uma resolução **isolada** — sem ele, uma única chamada ao fluxo
/// pode gastar minutos, e nem o teto de resoluções nem o de expansões a alcançam.
///
/// ⚠️ **Ele conta aumentos, não segundos**, e um aumento não custa o mesmo em
/// todo layout: ele carrega uma travessia do grafo. Medido em 2026-08-20
/// ([`tests/scaling.rs`], grelha toroidal com alvos dispersos):
///
/// | arcos | aumentos | ms | µs por aumento |
/// |---|---|---|---|
/// | 512 | 5 107 | 129 | 25 |
/// | 1 152 | 11 986 | 988 | 82 |
/// | 2 048 | 10 692 | 1 710 | 160 |
///
/// ⭐ E com alvos **uniformes** o mesmo layout de 8 192 arcos gasta **zero**
/// aumentos: o desequilíbrio inicial já é nulo. *O esforço é da heterogeneidade,
/// não do tamanho.*
///
/// ⚠️ **O teto está 8× acima do pior caso LEGÍTIMO medido** — a `sculpt_hooked`,
/// que resolve e prova, gastou **30 219**. Um teto que só o dobrasse reprovaria
/// trabalho bom no dia em que um layout ligeiramente maior aparecesse.
///
/// ⛔ **E ele não limita o RELÓGIO**, porque um aumento não custa o mesmo em todo
/// layout. Quem quiser um limite de tempo passa um [`Budget`] menor — é por isso
/// que ele é parâmetro e não só constante.
pub const MAX_AUGMENTATIONS: usize = 250_000;

/// **O MERGULHO** — desce fixando arestas meio-inteiras até chegar a inteiro.
/// Devolve uma quantização válida, ou [`SolveError::Infeasible`] se não existe
/// nenhuma, ou [`SolveError::Exhausted`] se o orçamento acabou antes.
///
/// ⚠️ **Ele prende o LOTE INTEIRO de uma vez, e isso não é conforto.** Uma aresta
/// por resolução é `O(arestas)` resoluções completas de fluxo, e uma resolução
/// não é barata num layout grande. O paper faz o mesmo movimento por outro nome —
/// o *evening* do §3.6 decide a **classe de paridade de todas as arestas ao mesmo
/// tempo**, não uma a uma.
///
/// Se o lote não fecha, ele é **partido ao meio**, e no fim tenta-se **cada**
/// aresta em separado (piso e depois teto). Se nem isso, **RECUA-SE um lote** —
/// porque o beco pode ter sido cavado numa ronda anterior, e nenhuma fixação
/// adicional o desfaz (medido 2026-08-20: o octaedro reprovava exatamente assim).
///
/// ⚠️ **Recuar, não recomeçar.** A primeira cura foi correr o mergulho todo outra
/// vez em modo de uma-aresta-de-cada-vez; ela é correta e **inutilizável** num
/// layout grande, porque paga do zero as `O(arestas)` resoluções que o lote
/// existia para evitar.
///
/// ⚠️ **Descer primeiro é uma escolha, e ela tem razão.** O piso de todo arco é
/// `1` e o custo é simétrico; descer aproxima do piso, que é onde a viabilidade
/// aperta.
fn dive(
    net: &BiNetwork,
    layout: &Layout,
    root: &Round,
    work: &mut Work,
    budget: Budget,
) -> Result<(f64, Vec<i64>), SolveError> {
    let mut fixed: Ranges = BTreeMap::new();
    let mut round = Round {
        dc_cost: root.dc_cost,
        num: root.num.clone(),
        lo: root.lo.clone(),
    };
    // Os estados ANTES de cada passo em lote — o único ponto a que vale recuar.
    let mut history: Vec<(Ranges, Round)> = Vec::new();
    // Depois de um recuo, o lote deixa de ser tentado: já se sabe que ele mente
    // nesta região.
    let mut single = false;
    loop {
        let odd: Vec<usize> = round
            .num
            .iter()
            .enumerate()
            .filter(|(_, n)| *n % 2 != 0)
            .map(|(e, _)| e)
            .collect();
        if odd.is_empty() {
            let flow = flow_of(&round);
            return Ok((cost_of(net, layout, &flow), flow));
        }
        let before = (fixed.clone(), round.clone());
        if work.solves >= budget.solves {
            return Err(SolveError::Exhausted {
                solves: work.solves,
            });
        }
        match dive_step(net, &fixed, &round, &odd, single, work, budget) {
            Some((next_fixed, next_round, batched)) => {
                if batched {
                    history.push(before);
                }
                fixed = next_fixed;
                round = next_round;
            }
            None => {
                // Beco sem saída: desfaz o último lote e segue passo a passo.
                // ⚠️ Sem lote nenhum para desfazer, o que falhou foi a BUSCA por
                // inteiros — não a existência deles.
                let (f, r) = history.pop().ok_or(SolveError::NoIntegral)?;
                fixed = f;
                round = r;
                single = true;
            }
        }
    }
}

/// UM passo do mergulho. Devolve o estado novo e se ele veio de um **lote**
/// (a única coisa que vale a pena poder desfazer).
fn dive_step(
    net: &BiNetwork,
    fixed: &Ranges,
    round: &Round,
    odd: &[usize],
    single: bool,
    work: &mut Work,
    budget: Budget,
) -> Option<(Ranges, Round, bool)> {
    // Prender = colapsar a faixa da aresta num ponto.
    let pin = |trial: &mut Ranges, e: usize, up: bool| {
        let edge = &net.edges()[e];
        let v = (round.lo[e] + round.num[e] / 2 + i64::from(up)).clamp(edge.lo, edge.hi);
        trial.insert(e, (v, v));
    };
    if !single {
        let mut take = odd.len();
        while take > 1 {
            let mut trial = fixed.clone();
            for &e in &odd[..take] {
                pin(&mut trial, e, false);
            }
            if let Ok(r) = run(net, &trial, work, budget.augmentations) {
                return Some((trial, r, true));
            }
            if work.solves >= budget.solves {
                return None;
            }
            take /= 2;
        }
    }
    // Uma aresta de cada vez: piso e depois teto, varrendo TODAS.
    for &e in odd {
        for up in [false, true] {
            let mut trial = fixed.clone();
            pin(&mut trial, e, up);
            if let Ok(r) = run(net, &trial, work, budget.augmentations) {
                return Some((trial, r, false));
            }
            if work.solves >= budget.solves {
                return None;
            }
        }
    }
    None
}

/// **A RAMIFICAÇÃO** — parte a faixa `[lo, hi]` em duas no corte `cut`.
///
/// ⭐ **Ela é uma função à parte para poder ser GATEADA sozinha**, e a
/// propriedade que a define cabe numa frase: *as duas metades têm de **PARTICIONAR**
/// a faixa* — disjuntas e cobrindo tudo. Uma ramificação que devolva os dois
/// **pontos** `{cut}` e `{cut+1}` parece a mesma coisa e descarta em silêncio
/// todo inteiro fora deles; a busca continua a esgotar-se e a declarar-se
/// **provada**, sobre um ótimo que não é o ótimo.
///
/// ⚠️ **Isto aconteceu, e nenhum layout sintético o apanhou.** Medido em
/// 2026-08-20 sobre a `sculpt_hooked` do oráculo (15 patches, 30 arcos): a mesma
/// busca dava **29,86**, **29,92** e — já com meias-retas — **29,81**, todas com
/// `prova = sim`. *Duas provas do mesmo ótimo não podem discordar.* O octaedro, o
/// prisma pequeno e o layout com junção em T sobrevivem à mutação; o que a mata é
/// esta invariante, verificada aqui e não no resultado.
#[must_use]
pub fn branch(lo: i64, hi: i64, cut: i64) -> [(i64, i64); 2] {
    [(lo, cut), (cut + 1, hi)]
}

/// As faixas apertadas por um ramo da busca: `aresta -> (piso, teto)`.
///
/// ⚠️ **Uma FAIXA, nunca um ponto.** Ver o comentário do ramifica-e-limita em
/// [`quantize_within`]: pontos não particionam o espaço, e a busca passa a
/// declarar-se provada sem o ser.
type Ranges = BTreeMap<usize, (i64, i64)>;

/// Um nó da busca: as faixas apertadas, a resolução e o limite dela.
struct Node {
    bound: f64,
    range: Ranges,
    round: Round,
}

/// O fluxo inteiro de uma resolução já integral.
fn flow_of(round: &Round) -> Vec<i64> {
    round
        .lo
        .iter()
        .zip(&round.num)
        .map(|(lo, num)| lo + num / 2)
        .collect()
}

/// O custo **verdadeiro** de um fluxo — recontado sobre os arcos do layout.
///
/// ⚠️ **Não é `base + dc/2`.** Aquele número é o da rede, e ele pode ficar
/// **acima** do custo real quando as duas cópias da dupla cobertura preenchem
/// degraus diferentes do mesmo arco. O que o utilizador vê é o custo do layout;
/// o da rede fica a ser só o limite.
fn cost_of(net: &BiNetwork, layout: &Layout, flow: &[i64]) -> f64 {
    let mut x = vec![0u32; layout.arcs().len()];
    for (e, edge) in net.edges().iter().enumerate() {
        if let Some(a) = edge.arc {
            x[a as usize] = u32::try_from(flow[e]).unwrap_or(u32::MAX);
        }
    }
    layout.cost(&x)
}

/// **O TRABALHO GASTO** — as duas unidades que a fase consome.
///
/// ⚠️ Elas andam juntas de propósito: `solves` conta chamadas ao fluxo e
/// `augmentations` conta o esforço DENTRO delas. Só a segunda explica o relógio.
#[derive(Debug, Clone, Copy, Default)]
struct Work {
    solves: usize,
    augmentations: usize,
}

/// O resultado de uma resolução da dupla cobertura.
#[derive(Clone)]
struct Round {
    dc_cost: f64,
    /// Por aresta bi-dirigida, **o dobro** do fluxo livre — para que meio-inteiro
    /// seja representável sem ponto flutuante.
    num: Vec<i64>,
    /// ⚠️ **O piso que ESTA resolução usou**, que não é o da rede quando a aresta
    /// foi fixada por uma ronda anterior. Ler o piso da rede aqui devolve o valor
    /// certo em toda ronda menos naquelas em que a fixação atuou — e o resultado
    /// passa a violar a conservação sem que a rede acuse nada.
    lo: Vec<i64>,
}

/// Resolve a dupla cobertura uma vez, com um conjunto de arestas fixadas.
fn run(
    net: &BiNetwork,
    over: &Ranges,
    work: &mut Work,
    max_augment: usize,
) -> Result<Round, McfError> {
    work.solves += 1;
    let mut mcf = Mcf::new(net.nodes() * 2);
    // As demandas vêm do piso obrigatório de cada aresta.
    let mut demand = vec![0i64; net.nodes()];
    let bounds: Vec<(i64, i64)> = net
        .edges()
        .iter()
        .enumerate()
        .map(|(e, edge)| over.get(&e).copied().unwrap_or((edge.lo, edge.hi)))
        .collect();
    for (e, edge) in net.edges().iter().enumerate() {
        let lo = bounds[e].0;
        demand[edge.a as usize] -= i64::from(edge.sa) * lo;
        demand[edge.b as usize] -= i64::from(edge.sb) * lo;
    }
    for (v, d) in demand.iter().enumerate() {
        mcf.demand(v * 2, *d);
        mcf.demand(v * 2 + 1, -*d);
    }

    let mut ids: Vec<(Vec<usize>, Vec<usize>)> = Vec::with_capacity(net.edges().len());
    for (e, edge) in net.edges().iter().enumerate() {
        let (lo, hi) = bounds[e];
        let [(t1, h1), (t2, h2)] = dc_pair(edge.a as usize, edge.sa, edge.b as usize, edge.sb);
        let mut one = Vec::new();
        let mut two = Vec::new();
        for (cap, cost) in crate::refine::segments(edge, lo, hi) {
            one.push(mcf.arc(t1, h1, cap, cost));
            two.push(mcf.arc(t2, h2, cap, cost));
        }
        // ⭐ **A partida a quente das arestas de LEQUE.** Elas têm custo zero,
        // logo nada as empurra: sem isto partem do piso `1` enquanto os arcos já
        // pré-saturaram perto do alvo, e o desequilíbrio de cada nó fica da ordem
        // do comprimento do lado — uma travessia de Dijkstra por unidade dele.
        // Em arco de custo zero a quantidade inicial é livre e o ótimo é o mesmo
        // ([`crate::mcf::Mcf::preload`]).
        if edge.weight == 0.0 {
            let want = edge.warm.clamp(lo, hi) - lo;
            for (&a, &b) in one.iter().zip(&two) {
                mcf.preload(a, want);
                mcf.preload(b, want);
            }
        }
        ids.push((one, two));
    }

    let dc_cost = mcf.solve(max_augment - work.augmentations.min(max_augment))?;
    work.augmentations += mcf.augmentations();
    let num = ids
        .iter()
        .map(|(one, two)| {
            let a: i64 = one.iter().map(|&id| mcf.flow(id)).sum();
            let b: i64 = two.iter().map(|&id| mcf.flow(id)).sum();
            a + b
        })
        .collect();
    Ok(Round {
        dc_cost,
        num,
        lo: bounds.iter().map(|(lo, _)| *lo).collect(),
    })
}

/// **AS DUAS ARESTAS DIRIGIDAS** que uma bi-aresta vira na dupla cobertura.
///
/// `v⁺` é `2v`, `v⁻` é `2v+1`. Cada caso está escrito por extenso de propósito:
/// a regra geral cabe numa linha e é exatamente onde um sinal trocado passa
/// despercebido — o fluxo continua a fechar, e o layout que sai é outro.
fn dc_pair(a: usize, sa: i8, b: usize, sb: i8) -> [(usize, usize); 2] {
    let (ap, am) = (a * 2, a * 2 + 1);
    let (bp, bm) = (b * 2, b * 2 + 1);
    match (sa, sb) {
        // head-head: as duas pontas somam. `b⁻ → a⁺` e `a⁻ → b⁺`.
        (1, 1) => [(bm, ap), (am, bp)],
        // tail-tail: as duas pontas consomem. `a⁺ → b⁻` e `b⁺ → a⁻`.
        (-1, -1) => [(ap, bm), (bp, am)],
        // dirigida `b → a`.
        (1, _) => [(bp, ap), (am, bm)],
        // dirigida `a → b`.
        _ => [(ap, bp), (bm, am)],
    }
}
