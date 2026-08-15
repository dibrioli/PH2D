//! As oito agregações do `value.reduce` e as duas portas OPCIONAIS que as
//! escopam — a metade PURA do nó, longe da fiação do registry.
//!
//! O que mora aqui é a LEI: *dado um conjunto de números, que número o
//! representa?* — e ela é a mesma nos dois caminhos (a CPU corre-a como está
//! escrita; o WGSL do `lib.rs` transcreve-a op a op sobre as reduções que o
//! sequenciador já dobrou). Uma segunda cópia dessa pergunta é como a face do
//! artista e o device passam a discordar sobre o que "desvio padrão" quer dizer.

use ph2d_nodegraph::gpu::ReduceOp;
use std::collections::BTreeMap;

/// Qual agregação de conjunto o nó difunde.
///
/// ⚠️ **Os índices 0..3 são os que já shipavam e NÃO se movem** — o `mode` é um
/// param que o GRAFO GUARDA, então renumerar re-aponta em silêncio todo
/// documento salvo. Os quatro novos são APENDADOS.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Mode {
    /// `Σ vᵢ` — o total (ε: a adição de floats não é associativa).
    Sum,
    /// `Σ vᵢ / N` — a média (a contagem é exata; a soma é ε).
    Mean,
    /// O menor elemento (bit-exato em qualquer ordem).
    Min,
    /// O maior elemento (bit-exato em qualquer ordem).
    Max,
    /// `max − min` — a EXTENSÃO do conjunto.
    ///
    /// ⚠️ Esta linha era **P2** na folha 15 (*"exprimível a 3 nós: `reduce(Max) →
    /// math(Subtract) ← reduce(Min)`"*) e entra porque as duas reduções que ela
    /// pede **já estão dobradas** — é um braço de `switch`, não uma capacidade
    /// nova, e deixá-la de fora do menu em que as outras sete moram seria
    /// arbitrário (o Attribute Statistic do Blender publica as oito juntas).
    Range,
    /// `Σ(v − μ)² / N` — a variância POPULACIONAL (divide por `N`, não `N−1`).
    ///
    /// ⚠️ **DOIS passos, e a medição é o motivo** (ver [`variance`]). O device
    /// **recusa** este modo pela mesma razão que recusa a mediana: o passo dois
    /// precisa da média, que só existe depois do passo um, e as reduções correm
    /// todas ANTES do kernel sem uma poder ler a outra.
    Variance,
    /// `√Variance` — o desvio padrão, na unidade do próprio campo.
    StdDev,
    /// O elemento do MEIO (média dos dois centrais em contagem par).
    ///
    /// ⚠️ **Não é um monóide** — não há combinador associativo que o dobre —, e é
    /// a razão pela qual ele **RECUSA o device** (`applicable`) ao lado de
    /// `Variance`/`StdDev`: um rank global pede ordenação, que é outro maquinário
    /// e não um braço de `switch`. A recusa é nomeada; o censo diz `[no-kernel]`.
    Median,
}

impl Mode {
    pub(crate) fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Mode::Mean,
            2 => Mode::Min,
            3 => Mode::Max,
            4 => Mode::Range,
            5 => Mode::Variance,
            6 => Mode::StdDev,
            7 => Mode::Median,
            _ => Mode::Sum,
        }
    }
}

/// **A variância em DOIS passos** — a média primeiro, depois `Σ(v − μ)² / N`.
///
/// ⚠️ **A fórmula de um passo (`E[v²] − E[v]²`) foi CONSTRUÍDA, MEDIDA e
/// DESCARTADA**, e o número é o argumento inteiro. Ela subtrai dois valores
/// quase iguais, então o que sobra é a diferença entre dois arredondamentos —
/// medido sobre um campo de desvio **1** deslocado para a média `μ`:
///
/// | média  | desvio REAL | um passo  | dois passos |
/// |--------|-------------|-----------|-------------|
/// | 0..100 | 1           | 1,000000  | 1,000000    |
/// | 1e3    | 1           | **0,50**  | 1,000000    |
/// | 1e4    | 1           | **0,00**  | 1,000000    |
/// | 1e5    | 1           | **55,4**  | 1,000000    |
/// | 1e5    | **0**       | **71,6**  | 0,000000    |
///
/// A última linha é a que fecha a discussão: um campo **CONSTANTE** — o caso em
/// que a resposta certa é a mais óbvia que existe — reporta desvio **71**. Um
/// número errado apresentado como certo é pior que um caminho mais lento, e é
/// por isso que este modo (e o `StdDev`) **recusam o device** em vez de aceitar
/// o limite: as reduções correm todas antes do kernel e nenhuma lê o resultado
/// da outra, logo `Σ(v − μ)²` não é exprimível ali.
///
/// ⚠️ **E o caminho de device EXISTE, por composição** — é o que a folha 15 já
/// media em cinco nós: `reduce(Mean) → math(Sub) ← in → unary(Square) →
/// reduce(Mean) → unary(Sqrt)`. Quem precisa das duas coisas ao mesmo tempo tem
/// a cadeia; quem quer um nó tem este.
///
/// Uma soma de quadrados não pode ser negativa, então não há clamp a fazer: o
/// `sqrt` do `StdDev` nunca vê um argumento que o faça devolver NaN.
pub(crate) fn variance(field: &[f32]) -> f32 {
    let n = field.len();
    if n == 0 {
        return 0.0;
    }
    let n = n as f32;
    let mean = ReduceOp::Sum.cpu(field) / n;
    field.iter().fold(0.0f32, |a, v| {
        let d = v - mean;
        a + d * d
    }) / n
}

/// O elemento do MEIO. Contagem ímpar devolve o central; par devolve a **média
/// dos dois centrais** (a definição estatística padrão — a alternativa, o
/// central-de-baixo, faz a mediana SALTAR quando um elemento entra no conjunto).
fn median(field: &[f32]) -> f32 {
    let n = field.len();
    if n == 0 {
        return 0.0;
    }
    let mut s = field.to_vec();
    // `partial_cmp` basta: NaN está fora de contrato nesta família (ver
    // `ReduceOp`), e tratá-lo como igual mantém a ordenação total.
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    if n % 2 == 1 {
        s[n / 2]
    } else {
        (s[n / 2 - 1] + s[n / 2]) * 0.5
    }
}

/// O número único que representa `field` sob `mode`.
///
/// ⚠️ **Um conjunto VAZIO devolve `0.0` em TODOS os modos.** Antes das portas
/// opcionais isto era inobservável (um campo vazio emite um stream vazio, e o
/// valor difundido zero vezes não chega a ninguém); com uma máscara que não
/// seleciona nada o conjunto pode ser vazio **num campo de N elementos**, e aí a
/// identidade do `Min` (`+∞`) seria difundida para os N — um infinito que
/// atravessa toda a cadeia a jusante. Zero é o que o `Mean` já respondia.
pub(crate) fn aggregate(field: &[f32], mode: Mode) -> f32 {
    if field.is_empty() {
        return 0.0;
    }
    match mode {
        Mode::Sum => ReduceOp::Sum.cpu(field),
        Mode::Mean => ReduceOp::Sum.cpu(field) / field.len() as f32,
        Mode::Min => ReduceOp::Min.cpu(field),
        Mode::Max => ReduceOp::Max.cpu(field),
        Mode::Range => ReduceOp::Max.cpu(field) - ReduceOp::Min.cpu(field),
        Mode::Variance => variance(field),
        Mode::StdDev => variance(field).sqrt(),
        Mode::Median => median(field),
    }
}

/// **O elemento `i` é CONTADO?** — a porta `mask`, ausente ⇒ todos.
///
/// Comprimento 1 **difunde** (uma chave liga/desliga o conjunto inteiro — a
/// convenção `ReadBroadcast` da engine); fora do alcance conta como zero, ou
/// seja excluído, porque uma máscara mais curta que o campo não tem opinião
/// sobre a cauda e o silêncio dela não pode virar inclusão.
pub(crate) fn selected(mask: &[f32], i: usize) -> bool {
    match mask.len() {
        0 => true,
        1 => mask[0] != 0.0,
        _ => mask.get(i).copied().unwrap_or(0.0) != 0.0,
    }
}

/// **A que grupo o elemento `i` pertence?** — a porta `group`, ausente ⇒ um
/// grupo só (que é exactamente o mundo de antes).
pub(crate) fn group_of(group: &[f32], i: usize) -> i64 {
    let g = match group.len() {
        0 => return 0,
        1 => group[0],
        _ => group.get(i).copied().unwrap_or(0.0),
    };
    if g.is_finite() { g.round() as i64 } else { 0 }
}

/// A saída inteira do nó: o agregado DIFUNDIDO de volta ao comprimento do campo.
///
/// ⚠️ **A máscara diz quem é CONTADO, nunca quem é RESPONDIDO.** A razão de
/// existir do nó é tornar um valor RELATIVO ao conjunto (`reduce(Mean) →
/// math(Subtract)` centra o campo), e nessa cadeia todo elemento precisa do
/// número — inclusive os que ficaram de fora da estatística. É também a leitura
/// do Attribute Statistic do Blender, cuja `Selection` filtra a amostra e não a
/// saída.
///
/// Com a porta `group` ligada há **um agregado por grupo**, e cada elemento
/// recebe o do SEU grupo; um grupo sem nenhum membro selecionado recebe `0.0`
/// pela mesma lei do conjunto vazio.
pub(crate) fn reduce_field(field: &[f32], mode: Mode, mask: &[f32], group: &[f32]) -> Vec<f32> {
    let n = field.len();
    if mask.is_empty() && group.is_empty() {
        // O caminho que já shipava, literalmente: um agregado, difundido.
        return vec![aggregate(field, mode); n];
    }
    // `BTreeMap` e não `HashMap`: a ordem de iteração alimenta a agregação de
    // cada balde, e uma ordem de hash faria a soma de um grupo variar entre
    // execuções — o mesmo argumento que a ponte de física escreve para o `c9`.
    let mut buckets: BTreeMap<i64, Vec<f32>> = BTreeMap::new();
    for (i, v) in field.iter().enumerate() {
        if selected(mask, i) {
            buckets.entry(group_of(group, i)).or_default().push(*v);
        }
    }
    let aggs: BTreeMap<i64, f32> = buckets
        .into_iter()
        .map(|(k, vs)| (k, aggregate(&vs, mode)))
        .collect();
    (0..n)
        .map(|i| aggs.get(&group_of(group, i)).copied().unwrap_or(0.0))
        .collect()
}
