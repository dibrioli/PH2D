//! **A QUANTIZAÇÃO** — quantos quads cabem em cada lado de cada patch.
//!
//! Esta é a fase **F4** do plano ([`docs/3D/quad-remesh/PLAN.md`]), e é a que o
//! resto do pipeline inteiro depende: é aqui que *"100 % quads"* deixa de ser
//! uma esperança e vira uma **restrição satisfeita**.
//!
//! Clean-room a partir de Heistermann, Warnett e Bommes, *"Min-Deviation-Flow in
//! Bi-directed Graphs for T-Mesh Quantization"* (SIGGRAPH 2023), §3 e §4.4, e de
//! Pietroni et al., *QuadWild* (SIGGRAPH 2021), §7. ⚠️ Nenhuma linha traduzida
//! de fonte GPL — ver **ADR-0161**, Trilha A.
//!
//! # O problema, em uma frase
//!
//! Depois do traçado (F3) a superfície é uma colcha de **patches** de 3 a 6
//! lados. Cada lado é uma cadeia de **arcos**, e cada arco é partilhado por
//! exatamente dois patches. Escolher quantas arestas de quad cada arco recebe é
//! escolher um **inteiro por arco** — e a escolha só é válida se **todo patch
//! puder ser ladrilhado só com quadriláteros**.
//!
//! # ⭐ A condição, e por que ela é UMA só para todas as valências
//!
//! Um patch de valência `n` que se ladrilha com **um** vértice irregular no
//! centro é, necessariamente, a subdivisão regular do leque que liga um ponto de
//! cada lado a esse centro (Bi-MDF §4.4.1). Chamando `e_j` o comprimento da
//! aresta interior que vai do centro ao ponto do lado `j`, cada quad do leque
//! força `e` de um lado a casar com meio lado do outro, e sobra **uma** lei:
//!
//! ```text
//!     L_i = e_{i-1} + e_{i+1}          (índices módulo n,  e_j >= 1)
//! ```
//!
//! ⚠️ **Ela contém os casos que a literatura costuma escrever à parte:**
//!
//! | valência | o que a lei vira | o nome usual |
//! |---|---|---|
//! | 3 | `L_0 = e_1+e_2`, `L_1 = e_2+e_0`, `L_2 = e_0+e_1` | a desigualdade triangular + paridade |
//! | 4 | `L_0 = L_2` e `L_1 = L_3` | *"lados opostos iguais"* |
//! | 5, 6 | um sistema cíclico invertível | os padrões de Takayama |
//!
//! E ela dá de graça a condição global de Takayama/Tarini: `Σ L_i = 2·Σ e_j` é
//! **par**, sempre. *Não se testa a paridade — ela não pode ser violada.*
//!
//! # Por que isto é um FLUXO, e por que ele é BI-dirigido
//!
//! Ponha um nó por **lado** de patch. A lei acima é a conservação nesse nó:
//! o que entra pelos arcos do lado sai pelas duas arestas interiores vizinhas.
//! Então cada variável toca **exatamente dois** nós, com coeficiente `±1` — que
//! é a matriz de incidência de um grafo. ⚠️ Mas os dois sinais são **iguais**
//! (um arco entra `+1` nos dois patches; uma aresta interior sai `−1` dos dois
//! lados), e uma aresta cujas duas pontas apontam para o mesmo lado é o que se
//! chama **bi-dirigida**. É daí que vem toda a dificuldade — e é por isso que
//! isto não é um min-cost-flow de manual.
//!
//! # O que esta crate entrega, e o que ela ainda não sabe
//!
//! - [`quantize`] devolve uma quantização **válida por construção** ([`verify`])
//!   e um [`Report`] com o **limite inferior certificado** do ótimo.
//! - ⭐ **O `gap` do relatório é uma prova, não uma opinião.** O limite vem da
//!   relaxação por *dupla cobertura* (§3.6 do paper), que é um relaxamento
//!   legítimo do problema inteiro: `gap = 0` significa **ótimo demonstrado**.
//! - ⛔ O solver **exato por matching** (§3.7, Blossom) **não** está aqui. Ele
//!   só se justifica se o `gap` medido no corpus for maior que zero — e essa
//!   medição está no PLAN. *Medir antes de limitar* (CLAUDE.md §0.0).

#![forbid(unsafe_code)]

/// **A LEI POR PATCH** — `L_i = e_{i-1} + e_{i+1}` — ver [`corners`].
pub mod corners;
/// **O FLUXO DE CUSTO MÍNIMO** sobre a dupla cobertura — ver [`mcf`].
pub mod mcf;
/// **O LAYOUT vira rede bi-dirigida** — ver [`network`].
pub mod network;
/// **A ESCADA DE CUSTO e o REFINAMENTO** — o porte do libSatsuma — ver [`refine`].
pub mod refine;
/// **O QUE A FASE DIZ** — o relatório — ver [`report`].
pub mod report;
/// **O SOLVER** — dupla cobertura, simetrização, reparo — ver [`solve`].
pub mod solve;

pub use corners::{CornerError, solve_corners};
pub use network::BiNetwork;
pub use refine::quantize_within;
pub use report::Report;
pub use solve::{
    Budget, MAX_AUGMENTATIONS, MAX_EXPANSIONS, MAX_SOLVES, SolveError, branch, quantize,
};

/// **UM ARCO** do layout — o pedaço de fronteira entre dois cantos consecutivos.
///
/// ⚠️ **O arco é a unidade FINA, o lado é a grossa.** Um lado de um patch pode
/// conter vários arcos quando o patch vizinho tem um canto no meio dele — a
/// junção em T. É por isso que a variável é do arco e a lei é do lado.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcSpec {
    /// Quantas arestas de quad este arco **gostaria** de ter: o comprimento
    /// geométrico dividido pelo tamanho de aresta alvo. Real de propósito.
    pub target: f64,
    /// O peso da isometria deste arco no custo. `1.0` é o normal.
    pub weight: f64,
    /// O mínimo admissível. ⚠️ **`1` no pipeline do QuadWild**: um arco em zero
    /// colapsaria o canto, e o estágio a jusante não sabe lidar com isso.
    pub min: u32,
    /// ⭐⭐ **A FORMA do custo** — e é ela que decide entre esmagar e espalhar.
    pub kind: Deviation,
}

/// **A FORMA DO CUSTO DE UM ARCO** — porte do `CostFunction` do libSatsuma (MIT).
///
/// ⭐⭐ **A escolha entre estas duas não é de afinação: ela troca o SINAL da
/// preferência do ótimo.** O solver minimiza a soma; o que decide se ele esmaga um
/// arco longo ou espalha o erro por vários é se a **marginal** cresce.
///
/// | escolher | `w·|x−t|`, `w = 1/t` | `w·(x−t)²`, `w` uniforme |
/// |---|---|---|
/// | esmagar o arco `t=4,1` até `1` | `3,1/4,1 =` **0,76** | `(1−4,1)² =` **9,6** |
/// | espalhar por 3 arcos `t=1` | `3 × 1,0 =` **3,0** | `3 × 1 =` **3,0** |
/// | ⇒ o ótimo escolhe | ⛔ **esmagar** | ✅ **espalhar** |
///
/// ⚠️ **O `Abs` fica**, e não por compatibilidade: o oráculo de força bruta dos
/// gates precisa do custo mais simples que existe para as duas respostas serem
/// comparáveis termo a termo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Deviation {
    /// `w·|x − t|` — o `AbsDeviation` da referência. ⚠️ A marginal é **constante**
    /// de cada lado do alvo, então mover a primeira unidade custa o mesmo que mover
    /// a décima: *esmagar e espalhar custam exactamente o mesmo.*
    Abs,
    /// `w·(x − t)²` — o `QuadDeviation` da referência, e ⭐ **o default de toda
    /// aresta de sub-lado no `qr_flow.cpp` do QuadWild** (`add_subside_edge`, com
    /// `ObjectiveKind obj = ObjectiveKind::QuadraticDeviation`).
    #[default]
    Quad,
    /// `w·max((x+ε)/(t+ε), (t+ε)/(x+ε))` — o `ScaleFactor` da referência.
    ///
    /// ⭐ **É a única das três que é ASSIMÉTRICA, e a assimetria é o ponto:** para
    /// `x > t` ela cresce **linear** (declive `w/(t+ε)`); para `x < t` ela é uma
    /// **hipérbole**, que explode quando o arco se aproxima de zero. *Encolher um
    /// arco à metade e dobrá-lo não são o mesmo estrago, e só esta lei sabe disso.*
    ///
    /// ⚠️ O `ε` existe para o ramo `t/x` não divergir em `x = 0`; o valor `0,1` é o
    /// da referência (`CostFunction.hh`), não uma escolha nossa.
    Scale,
}

/// O `eps` do [`Deviation::Scale`] — **o número da referência**, não nosso.
const SCALE_EPS: f64 = 0.1;

impl ArcSpec {
    /// Um arco de peso unitário que não pode colapsar.
    ///
    /// ⚠️ **Peso `1` é deviação ABSOLUTA**, e ele existe para o oráculo de força
    /// bruta dos gates: ali o custo tem de ser o mais simples possível para as duas
    /// respostas serem comparáveis. ⛔ **Não é o que a malha quer** — ver
    /// [`Self::relative`].
    #[must_use]
    pub fn new(target: f64) -> Self {
        Self {
            target,
            weight: 1.0,
            min: 1,
            kind: Deviation::Abs,
        }
    }

    /// **O ARCO ISOMÉTRICO** — `((x − alvo) / alvo)²`, o **quadrado do erro
    /// RELATIVO**.
    ///
    /// ⭐⭐ **A forma é a `QuadDeviation` da referência; o peso `1/alvo²` é o que
    /// torna o custo uma RAZÃO.** Juntos eles são exactamente o princípio que esta
    /// crate já afirmava e não implementava: *a qualidade de uma grade é uma razão,
    /// nunca uma contagem* — um arco que pedia 24 e recebeu 20 está a 17 % do alvo;
    /// um que pedia 2 e recebeu 6 está a 200 %.
    ///
    /// ⛔ **A versão anterior — LINEAR com peso `1/alvo` — tinha o sinal invertido
    /// no regime que importa.** Sobre um custo linear a marginal é constante, então
    /// esmagar um arco e espalhar o erro custam o mesmo — e com `1/alvo` esmagar um
    /// arco LONGO passa a ser **4× mais barato**. Medido na `hooked_sphere`
    /// (2026-08-21): o arco `#21` pedia **4,1** segmentos e recebeu **1** — uma
    /// corda recta de comprimento `1,105` numa peça de raio `1,0` —, e **6 de 50**
    /// arcos pediam mais do dobro do que receberam. A grade de Coons construída
    /// sobre uma corda dessas nasce do lado errado da forma, e ⚠️ **o alisamento
    /// não a desfaz**: ele move o interior do patch, e a corda **é** a fronteira.
    /// *Uma resposta provadamente óptima para o objectivo errado.*
    ///
    /// ⭐ **A varredura que a escolheu** (`which_arc_weight_law_protects_the_grid`,
    /// orçamento de busca grande para separar *inviável* de *acabou o tempo*):
    ///
    /// | lei | dobras 48×72 | 96×144 | esculpida | **pior arco** | pior relógio |
    /// |---|---|---|---|---|---|
    /// | `abs · 1/t` (a anterior) | 30,4 % | 1,9 % | 0,2 % | 3,2 / **8,1** / 2,6 | 25 ms |
    /// | `abs · 1` | 33,3 % | 3,4 % | 0,1 % | 1,7 / 6,1 / 1,3 | 156 ms |
    /// | `quad · 1` (o default da referência) | 37,5 % | 2,8 % | 0,1 % | 2,8 / 2,4 / 1,7 | ⛔ **2 744 ms** |
    /// | ⭐ **`quad · 1/t²` (esta)** | **26,2 %** | **1,8 %** | 0,1 % | 2,9 / 6,1 / 2,6 | 76 ms |
    /// | ⏸️ `scale` (o `ScaleFactor` da referência) | ⛔ 36,3 % | 2,2 % | **0,0 %** | ⭐ **1,8 / 3,0 / 1,7** | 20 ms |
    ///
    /// ⏸️ **A [`Deviation::Scale`] ganha na grandeza que nomeia o defeito** — o
    /// pior arco, onde ela é a melhor das cinco nas três malhas — e dá **zero**
    /// dobras na esculpida, que é a fixtura que se parece com o trabalho do
    /// artista. Ela **não foi escolhida** porque reprova o gate da 48×72 (36,3 %
    /// contra a barra de 33 %), e ⛔ **a barra não se afrouxa**. ⚠️ Mas essa
    /// fixtura está num regime em que a aresta máxima é **33 a 40× o alvo em TODAS
    /// as leis** — o artefacto de grão que a sonda
    /// `how_fine_may_the_quad_be_against_the_reference_facet` mede —, então ela não
    /// está a arbitrar entre custos. *Reabrir esta escolha depois de o grão estar
    /// curado é trabalho pendente, não uma recusa.*
    ///
    /// ⚠️ **O piso de `1` no divisor** evita que um arco de alvo minúsculo ganhe
    /// peso arbitrariamente grande e passe a mandar no layout inteiro.
    #[must_use]
    pub fn isometric(target: f64) -> Self {
        Self {
            target,
            weight: 1.0 / (target.max(1.0) * target.max(1.0)),
            min: 1,
            kind: Deviation::Quad,
        }
    }

    /// O custo de quantizar este arco em `x` — ver [`Deviation`].
    #[must_use]
    pub fn cost(&self, x: u32) -> f64 {
        deviation(self.kind, self.weight, self.target, f64::from(x))
    }
}

/// **O CUSTO, numa fórmula só** — porte do `CostFunction::cost` do libSatsuma.
///
/// ⭐ **Ela vive aqui e é usada pelos DOIS lados** ([`ArcSpec::cost`] e
/// [`crate::network::BiEdge::cost`]). Duas cópias divergiriam no dia em que uma
/// quarta forma entrasse, e a que decide o resultado é a do `BiEdge` — a outra só
/// aparece nos relatórios. *O sintoma seria um `gap` que não fecha sobre uma
/// resposta correcta.*
#[must_use]
pub fn deviation(kind: Deviation, weight: f64, target: f64, x: f64) -> f64 {
    let d = x - target;
    weight
        * match kind {
            Deviation::Abs => d.abs(),
            Deviation::Quad => d * d,
            Deviation::Scale => {
                let (a, b) = (x + SCALE_EPS, target + SCALE_EPS);
                (a / b).max(b / a)
            }
        }
}

/// **UM PATCH** — a lista ordenada dos seus lados, cada lado uma lista de arcos.
///
/// ⚠️ **A ordem é cíclica e importa**: a lei liga o lado `i` às arestas
/// interiores `i−1` e `i+1`, e uma lista fora de ordem devolve um sistema que
/// tem solução e descreve outro patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchSpec {
    /// Os lados, em volta. Cada um é a sequência de ids de arco que o compõe.
    pub sides: Vec<Vec<u32>>,
}

/// **O LAYOUT** — a entrada da fase, e nada além dela.
///
/// ⚠️ **Ele não conhece geometria.** Um `target` já é adimensional (comprimento
/// dividido pelo alvo de aresta), e é isso que permite testar esta crate contra
/// força bruta sem construir uma malha.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    arcs: Vec<ArcSpec>,
    patches: Vec<PatchSpec>,
}

/// O que pode estar errado num layout — todos são erros de **construção**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// Um lado referencia um arco que não existe.
    ArcOutOfRange {
        /// O patch onde está a referência.
        patch: usize,
        /// O id inexistente.
        arc: u32,
    },
    /// Um patch tem menos de 3 lados. ⚠️ Com 2 lados a lei vira `L_i = 2·e`, que
    /// é representável mas nunca aparece no traçado — recusar é mais honesto que
    /// aceitar em silêncio um layout que o F3 não deveria ter produzido.
    Valence {
        /// O patch culpado.
        patch: usize,
        /// Quantos lados ele tem.
        sides: usize,
    },
    /// Um arco não é usado por exatamente dois lados de patch.
    ///
    /// ⚠️ **Uma superfície FECHADA é a premissa desta fase.** Um arco de bordo
    /// (usado uma vez só) precisa da condição de contorno que o F3 traz; recusar
    /// aqui é a cerca que impede um resultado silenciosamente errado.
    ArcUse {
        /// O arco culpado.
        arc: u32,
        /// Quantos lados o usam.
        uses: usize,
    },
    /// Um alvo não é um número finito e não-negativo.
    Target {
        /// O arco culpado.
        arc: u32,
    },
    /// ⭐⭐ **O COMPLEXO DE PATCHES NÃO É A SUPERFÍCIE** — a decomposição perdeu
    /// (ou inventou) topologia.
    ///
    /// ⛔ **Esta variante nasceu de um remesh que fechava o buraco de um toro em
    /// silêncio** (2026-08-22). A malha saía com `χ = 2` onde a topologia exige
    /// `0`, e passava em **todas** as outras cercas: 100 % de quads, zero arestas
    /// de bordo, zero não-manifold, cada arco usado exactamente duas vezes.
    ///
    /// ⭐ **A régua é `V − E + F` sobre *cantos · arcos · patches*.** Uma
    /// decomposição honesta da superfície devolve o `χ` dela, seja qual for o
    /// número de patches — a estrutura CW mínima de um toro é *um* patch com uma
    /// aresta dupla, e dá `1 − 2 + 1 = 0`. Quando o número não bate, há um patch a
    /// ser contado como disco sem o ser: uma região com **género** entra na conta
    /// como `+1` quando vale `−1`, e a diferença é exactamente `2`.
    ///
    /// ⚠️ **É a única cerca desta fase que prevê o `χ` da MALHA final** — as outras
    /// falam de contagens, e um patch com uma asa lá dentro produz contagens
    /// perfeitamente válidas.
    ///
    /// ⚠️ **Recusar é o certo enquanto o F3 não souber CORTAR a asa.** Uma malha
    /// que perdeu o buraco é indistinguível de uma boa em toda coluna que o artista
    /// tem — e uma recusa com nome manda-o para o conserto.
    GenusLost {
        /// `V − E + F` sobre cantos, arcos e patches.
        complex: i64,
        /// A característica de Euler da superfície que entrou.
        surface: i64,
    },
}

impl Layout {
    /// **CONSTRÓI e VALIDA** o layout.
    ///
    /// # Errors
    /// Devolve [`LayoutError`] se algum patch tem valência < 3, se um id de arco
    /// não existe, se um arco não é usado por exatamente dois lados, ou se um
    /// alvo não é finito e não-negativo.
    pub fn new(arcs: Vec<ArcSpec>, patches: Vec<PatchSpec>) -> Result<Self, LayoutError> {
        for (a, spec) in arcs.iter().enumerate() {
            if !spec.target.is_finite() || spec.target < 0.0 || !spec.weight.is_finite() {
                return Err(LayoutError::Target {
                    arc: u32::try_from(a).unwrap_or(u32::MAX),
                });
            }
        }
        let mut uses = vec![0usize; arcs.len()];
        for (p, patch) in patches.iter().enumerate() {
            if patch.sides.len() < 3 {
                return Err(LayoutError::Valence {
                    patch: p,
                    sides: patch.sides.len(),
                });
            }
            for side in &patch.sides {
                for &a in side {
                    let Some(slot) = uses.get_mut(a as usize) else {
                        return Err(LayoutError::ArcOutOfRange { patch: p, arc: a });
                    };
                    *slot += 1;
                }
            }
        }
        for (a, n) in uses.iter().enumerate() {
            if *n != 2 {
                return Err(LayoutError::ArcUse {
                    arc: u32::try_from(a).unwrap_or(u32::MAX),
                    uses: *n,
                });
            }
        }
        Ok(Self { arcs, patches })
    }

    /// Os arcos.
    #[must_use]
    pub fn arcs(&self) -> &[ArcSpec] {
        &self.arcs
    }

    /// Os patches.
    #[must_use]
    pub fn patches(&self) -> &[PatchSpec] {
        &self.patches
    }

    /// O comprimento quantizado de um lado: a soma dos seus arcos.
    #[must_use]
    pub fn side_len(&self, patch: usize, side: usize, x: &[u32]) -> u32 {
        self.patches[patch].sides[side]
            .iter()
            .map(|&a| x[a as usize])
            .sum()
    }

    /// O custo de isometria total de uma quantização.
    #[must_use]
    pub fn cost(&self, x: &[u32]) -> f64 {
        self.arcs.iter().zip(x).map(|(spec, &v)| spec.cost(v)).sum()
    }
}

/// **A RESPOSTA** — o inteiro de cada arco, mais as arestas interiores que provam
/// que cada patch fecha.
///
/// ⚠️ **Os `corners` não são decoração: eles são o CERTIFICADO.** Guardar apenas
/// os arcos deixaria a validade por reconferir, e é exatamente o tipo de
/// afirmação que envelhece. Quem consome o resultado (o F5) precisa deles para
/// montar o leque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantization {
    /// O comprimento de cada arco, em arestas de quad.
    pub arc: Vec<u32>,
    /// Por patch, o comprimento `e_j` de cada aresta interior do leque.
    pub corners: Vec<Vec<u32>>,
}

/// **VERIFICA** uma quantização contra o layout — a régua, independente do solver.
///
/// ⚠️ **Ela reconstrói os `e_j` do zero** em vez de confiar nos que vieram na
/// resposta. Um verificador que lê o certificado que o solver escreveu prova que
/// o solver é consistente consigo mesmo, e mais nada.
///
/// # Errors
/// Devolve [`CornerError`] no primeiro patch que não fecha.
pub fn verify(layout: &Layout, x: &[u32]) -> Result<Vec<Vec<u32>>, CornerError> {
    let mut out = Vec::with_capacity(layout.patches.len());
    for (p, patch) in layout.patches.iter().enumerate() {
        let lens: Vec<u32> = (0..patch.sides.len())
            .map(|i| layout.side_len(p, i, x))
            .collect();
        out.push(solve_corners(&lens).map_err(|e| e.at_patch(p))?);
    }
    Ok(out)
}
