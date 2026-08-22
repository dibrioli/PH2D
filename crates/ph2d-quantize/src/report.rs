//! **O QUE A QUANTIZAÇÃO DIZ** — o relatório da fase.
//!
//! Irmão do [`super::solve`], e o corte foi forçado pelo teto de LOC. ⭐ **Ele é
//! de assunto:** lá mora *como o problema se resolve*; aqui *o que se pode afirmar
//! sobre a resposta* — e três destes campos são a diferença entre uma prova e uma
//! opinião.
//!
//! ⚠️ **A lei que mora aqui:** o [`Report::gap`] só é uma prova quando o
//! [`Report::outside_window`] é zero. Fora da janela da escada o custo da rede é
//! uma linearização, então o limite inferior é de **outro problema** — e o `gap`
//! entre os dois já saiu **negativo** numa versão desta fase. *Um número que não
//! pode ser negativo e sai negativo é a prova a dizer que não é prova.*

/// O que o solver mediu ao resolver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Report {
    /// O custo de isometria da resposta.
    pub cost: f64,
    /// ⭐ O **limite inferior certificado** do ótimo inteiro.
    pub lower_bound: f64,
    /// `cost − lower_bound`. **Zero = ótimo demonstrado.**
    pub gap: f64,
    /// Quantas arestas saíram meio-inteiras na primeira resolução — a medida de
    /// **quanto** o problema é bi-dirigido de facto.
    pub half_integral: usize,
    /// Quantos nós a busca expandiu.
    pub expansions: usize,
    /// Quantas resoluções de fluxo custou, mergulho incluído.
    pub solves: usize,
    /// ⭐ Quantos **aumentos** de fluxo ao todo — a unidade de esforço que de
    /// facto move o relógio.
    pub augmentations: usize,
    /// ⭐ **A busca esgotou-se**, logo `cost` é o ótimo inteiro, demonstrado.
    /// `false` quer dizer que o teto de expansões mordeu e a resposta é válida
    /// mas apenas **boa** — e o `gap` diz quão boa.
    pub proved: bool,
    /// ⚠️ Quantos arcos ficaram **encostados no teto** da rede. Tem de ser `0`:
    /// um arco no teto quer dizer que o teto — e não a medição — escolheu o
    /// resultado. Ver [`crate::network`].
    pub cap_binding: usize,
    /// Quantos nós de lado. Serve para dimensionar o problema no relatório.
    pub nodes: usize,
    /// Quantas arestas bi-dirigidas.
    pub edges: usize,
    /// ⚠️ **Em que degrau de teto a rede coube** ([`crate::network::CAP_STEPS`]).
    /// Acima de `1` quer dizer que o teto apertado teria dito *"inviável"* sobre
    /// um layout que não é.
    pub cap_step: i64,
    /// ⭐⭐ **QUANTOS ARCOS a rede de fluxo de facto teve** — a escada de custo
    /// desdobrada, nas duas folhas da dupla cobertura.
    ///
    /// ⚠️ **Ele NÃO é `edges`, e a diferença é o relógio inteiro.** Cada aresta
    /// bi-dirigida vira uma **escada** de arcos paralelos, um degrau por faixa de
    /// custo marginal constante ([`segments`]) — e com um custo estritamente
    /// convexo *não há faixas*: cada unidade é o seu próprio degrau. *Uma rede de
    /// 778 arestas pode ser uma rede de fluxo de cem mil arcos, e é a segunda que
    /// o Dijkstra percorre.*
    pub mcf_arcs: usize,
    /// ⭐⭐ **Quantas arestas acabaram FORA da janela exacta da escada de custo** —
    /// ver [`MAX_EXACT_DEVIATION`]. `0` quer dizer que a linearização nunca mordeu.
    ///
    /// ⚠️ **É a condição da PROVA.** Fora da janela o custo da rede é uma
    /// linearização, então o `lower_bound` é de outro problema — e o `gap` entre os
    /// dois pode sair **negativo**, que foi o que a primeira versão desta escada
    /// imprimiu (`−86,97`). *Um número que não pode ser negativo e sai negativo é a
    /// prova a dizer que não é prova.*
    pub outside_window: usize,
    /// Quantas rondas de **re-centragem** correram — ver [`quantize_within`].
    pub refinements: usize,
}
