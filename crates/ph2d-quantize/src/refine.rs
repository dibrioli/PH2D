//! **A ESCADA DE CUSTO E O REFINAMENTO** — o porte da lei do libSatsuma (MIT).
//!
//! Irmão do [`super::solve`], e o corte foi forçado pelo teto de LOC (855 contra
//! 700). ⭐ **Mas ele é de ASSUNTO, e é o assunto da peça que este ficheiro
//! portou:** lá mora *como o problema inteiro se resolve* — a dupla cobertura, o
//! mergulho, o ramifica-e-limita; aqui *como um custo convexo cabe numa rede de
//! fluxo sem a fazer explodir*, e como a aproximação que isso custa se corrige
//! sozinha.
//!
//! # A lei, em duas frases
//!
//! Um custo estritamente convexo não tem dois passos com a mesma marginal, então a
//! escada que o representa tem **um arco por unidade** — e a rede passa de 778
//! arestas a **82 832 arcos de fluxo**. A referência corta isso emitindo a marginal
//! verdadeira só numa **janela** à volta de um palpite e linearizando o resto, e
//! depois **re-centra a janela na resposta** e repete até ela parar de se mexer.
//!
//! ⚠️ **A aproximação é auto-declarada.** [`super::Report::outside_window`] conta
//! quantas arestas acabaram fora da janela; com ele em zero a linearização nunca
//! mordeu e a prova do `gap` vale.

use crate::network::BiEdge;
use crate::report::Report;
use crate::solve::{Budget, SolveError, attempt};
use crate::{Layout, Quantization};

/// ⭐⭐ **A META-JANELA da escada de custo** — quantas unidades de cada lado do
/// palpite ganham um arco próprio, com a marginal verdadeira.
///
/// ⚠️ **É um teto de TAMANHO DE REDE, e o recurso é o relógio.** Fora da janela o
/// custo é linearizado em dois blocos; dentro dela ele é exacto. O
/// [`Report::outside_window`] conta quantos arcos acabaram **fora** da janela —
/// `0` quer dizer que a linearização nunca mordeu e a resposta é a exacta.
///
/// A referência usa `max_deviation = 2` e **itera** (`Highlevel.cc:69`, o
/// refinamento por matching). Enquanto o refinamento não existir aqui, este número
/// é maior de propósito: uma janela larga aproxima menos e ainda assim corta a
/// rede em uma ordem de grandeza.
pub(crate) const MAX_EXACT_DEVIATION: i64 = 8;

/// **O CUSTO CONVEXO, fatiado numa JANELA EXACTA e dois blocos linearizados.**
///
/// ⭐⭐ **Porte da lei do `BiMDF_to_BiMCF` do libSatsuma (MIT)**: a partir de um
/// **palpite** — o inteiro mais barato —, emite um arco por unidade dentro de
/// `± max_deviation` com a marginal **verdadeira**, e **um** arco de cada lado para
/// o resto, com a marginal **média** do bloco.
///
/// ⛔ **A versão anterior emitia a escada INTEIRA, e isso é a parede do produto.**
/// Com `w·|x − t|` as marginais são constantes de cada lado e a escada colapsava
/// em três degraus; com o custo **quadrático** (que entrou em 21/08 e é o que
/// impede o solver de esmagar um arco longo) **nenhum degrau se funde** — cada
/// unidade vira um arco. Medido em 2026-08-21:
///
/// | patches | arcos do layout | **arcos da REDE** | relógio |
/// |---|---|---|---|
/// | 28 | 67 | 2 938 (44×) | 1 ms |
/// | 72 | 189 | 6 274 (33×) | 52 ms |
/// | ⛔ **271** | **778** | ⛔ **82 832 (106×)** | ⛔ **176 s** |
///
/// *Uma rede de 778 arestas era uma rede de fluxo de oitenta mil arcos, e é a
/// segunda que o caminho-mais-curto percorre.*
///
/// ⭐ **A convexidade sobrevive à linearização:** a média das marginais de um bloco
/// fica entre a primeira e a última, então a sequência de custos continua
/// **não-decrescente** — que é a única coisa que o fluxo precisa para as preencher
/// na ordem certa.
///
/// ⚠️ **E para o custo ABSOLUTO ela é EXACTA e não uma aproximação:** as marginais
/// são constantes de cada lado, logo a média de um bloco **é** o valor de cada
/// degrau dele. O oráculo de força bruta dos gates usa esse custo, e por isso
/// continua a comparar duas respostas do mesmo problema.
pub(crate) fn segments(edge: &BiEdge, lo: i64, hi: i64) -> Vec<(i64, f64)> {
    let mut out = Vec::new();
    if hi <= lo {
        return out;
    }
    if edge.weight == 0.0 {
        out.push((hi - lo, 0.0));
        return out;
    }
    // ⭐ O PALPITE vive na ARESTA — ver [`BiEdge::guess`]. Ele nasce no inteiro
    // mais barato e o refinamento re-centra-o na solução da ronda anterior.
    let g = edge.guess.clamp(lo, hi);
    let a = (g - MAX_EXACT_DEVIATION).max(lo);
    let b = (g + MAX_EXACT_DEVIATION).min(hi);
    #[allow(clippy::cast_precision_loss)]
    let block = |from: i64, to: i64| (edge.cost(to) - edge.cost(from)) / (to - from) as f64;
    // O bloco de BAIXO, linearizado.
    if a > lo {
        out.push((a - lo, block(lo, a)));
    }
    // ⭐ A JANELA EXACTA: um degrau por unidade, com a marginal verdadeira.
    for k in (a + 1)..=b {
        out.push((1, edge.step_cost(k)));
    }
    // O bloco de CIMA, linearizado.
    if hi > b {
        out.push((hi - b, block(b, hi)));
    }
    out
}

/// **QUANTIZA com um ORÇAMENTO de busca.**
///
/// ⭐⭐ **Ela é o REFINAMENTO ITERADO, e é o porte da lei do `Highlevel.cc` do
/// libSatsuma (MIT):** aproximar, re-centrar na resposta, repetir até ela parar de
/// se mexer.
///
/// A rede emite a marginal verdadeira só dentro de uma janela à volta de um
/// **palpite** ([`MAX_EXACT_DEVIATION`]) e lineariza o resto — sem isso ela tem
/// **106 arcos de fluxo por aresta do layout** e a fase leva minutos (a tabela está
/// no doc do [`segments`]). ⚠️ **Mas uma janela fixa aproxima**, e a aproximação
/// aparece como `outside_window > 0` e como uma prova que não vale.
///
/// ⭐ **Re-centrar cura as duas coisas de uma vez:** a ronda seguinte põe a janela
/// onde a resposta de facto está, então a linearização deixa de morder. *Quando a
/// solução para de se mexer, `outside_window` é zero e a resposta é a exacta —
/// obtida com uma rede uma ordem de grandeza menor.*
///
/// # Errors
/// [`SolveError::Infeasible`] se nenhuma quantização regular existe;
/// [`SolveError::Exhausted`] se o mergulho gastou o orçamento de resoluções de
/// fluxo antes de chegar a uma resposta — ⚠️ afirmação sobre o **solver**, não
/// sobre o layout; [`SolveError::Inconsistent`] se o fluxo fechou mas um patch não,
/// que é um bug.
pub fn quantize_within(
    layout: &Layout,
    budget: Budget,
) -> Result<(Quantization, Report), SolveError> {
    let (mut best_q, mut best_r, mut flow) = attempt(layout, budget, None)?;
    for round in 1..=MAX_REFINEMENTS {
        // ⚠️ **A janela intacta é a condição de paragem certa**, e não *"o custo
        // não desceu"*: com ela zero, a resposta é a exacta do problema verdadeiro
        // e mais uma ronda não pode melhorar nada.
        if best_r.outside_window == 0 {
            break;
        }
        let (q, mut r, next) = attempt(layout, budget, Some(&flow))?;
        r.refinements = round;
        // ⚠️ **O custo tem de descer ESTRITAMENTE**, senão duas rondas podem
        // trocar de resposta para sempre — é a mesma cerca que o
        // `res.cost_change > -1e-20` da referência põe.
        if r.cost >= best_r.cost - 1.0e-9 {
            best_r.refinements = round;
            break;
        }
        best_r = r;
        best_q = q;
        flow = next;
    }
    Ok((best_q, best_r))
}

/// ⭐ **Quantas rondas de re-centragem no máximo.**
///
/// ⚠️ **Teto de ESPERA e não de qualidade:** quem o atinge sai com uma resposta
/// **válida** e com [`Report::outside_window`] a dizer que a janela ainda mordia.
/// A referência itera até o custo parar de descer, sem teto; aqui ele existe pelo
/// mesmo motivo que o das expansões — um layout patológico não pode prender o
/// gesto do artista.
const MAX_REFINEMENTS: usize = 8;
