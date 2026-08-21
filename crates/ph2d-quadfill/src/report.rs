//! **O QUE A MONTAGEM DIZ** — a recusa, o relatório e a proveniência de cada ponto.
//!
//! Irmão da [`crate::stitch`], e o corte foi **forçado pelo teto de LOC** (o pai
//! chegou a 714 contra 700). ⭐ **Mas ele é de ASSUNTO e não de conveniência:** lá
//! mora *como* a malha se monta; aqui, *o que se pode dizer sobre ela* — e é este
//! ficheiro que a auditoria de 2026-08-21 mostrou ser o mais importante dos dois.
//!
//! ⚠️ **A lição que mora aqui:** todo campo do [`FillReport`] menos os dois de
//! aresta é **função pura dos ÍNDICES**. Uma malha com as posições embaralhadas
//! devolve o relatório **byte-idêntico** — foi assim que 10.515 gates ficaram
//! verdes sobre um produto destruído.

use ph2d_mesh::Mesh;

/// Por que a malha não pôde ser montada.
// ⚠️ Deixou de ser `Eq` quando a `ArcNotOfThisMesh` passou a carregar os dois
// comprimentos: sem eles, a recusa diria *que* não bate e não **quanto**, e a
// diferença entre `1,0001×` e `5,40×` é a diferença entre ruído e catástrofe.
#[derive(Debug, Clone, PartialEq)]
pub enum FillError {
    /// ⚠️ **A lei do patch não bate com a quantização.** `L_i` tinha de ser
    /// `e_{i-1} + e_{i+1}`, e não é. Isto é **bug a montante**, não uma
    /// propriedade da malha: o F4 devolve `e` que satisfazem a lei por
    /// construção, logo ou os lados vieram fora de ordem ou o `e` é de outro
    /// patch. Recusar em vez de remendar é o que impede uma malha torcida.
    Mismatch {
        /// Qual patch.
        patch: usize,
        /// Qual lado.
        side: usize,
        /// O que a lei exigia.
        expected: u32,
        /// O que os arcos somaram.
        got: u32,
    },
    /// Um lado não emenda no seguinte — a fronteira do patch não fecha.
    ///
    /// ⚠️ **Ela carrega os VÉRTICES desde 2026-08-21**, e não só os índices: com
    /// `patch: 3, side: 0` e nada mais, a recusa diz *que* não fecha e não **onde**
    /// — e a diferença entre "o lado acaba num vértice que o seguinte não começa" e
    /// "um dos dois está vazio" manda procurar em sítios diferentes.
    Broken {
        /// Qual patch.
        patch: usize,
        /// Qual lado.
        side: usize,
        /// Onde este lado acaba.
        ends_at: Option<u32>,
        /// Onde o seguinte começa.
        next_starts_at: Option<u32>,
        /// Quantos lados o patch tem.
        sides: usize,
    },
    /// A malha resultante não monta.
    Mesh(String),
    /// ⭐⭐ **O LAYOUT NÃO É DESTA MALHA** — o defeito que destruiu o produto em
    /// 2026-08-21, e que nenhum dos 10.515 gates conseguia ver.
    ///
    /// ⚠️ **É a pré-condição mais barata que existe**, e ela existe porque o
    /// sintoma é invisível a jusante: um `arc_chain` de outra malha produz uma
    /// saída com **topologia perfeita** — 100 % quads, característica de Euler
    /// exacta, zero arestas de bordo, contagem de irregulares idêntica — e
    /// geometria destruída. *Nenhum número do [`FillReport`] muda.*
    ///
    /// A régua: o comprimento da polilinha de cada arco, medido **na malha que se
    /// vai amostrar**, tem de bater com o `arc_length` que o F3 declarou e que o
    /// F4 já usou para decidir quantos segmentos aquele arco leva. Medido: no
    /// caminho coerente a razão é **1,000 exacto** (é a mesma soma dos mesmos
    /// `f32`); no caminho destruído foi **5,40×**, com o pior arco a **9,04×**.
    /// *Três ordens de grandeza de margem — não há flake possível.*
    ArcNotOfThisMesh {
        /// Qual arco.
        arc: usize,
        /// O comprimento que o F3 declarou.
        declared: f32,
        /// O que a malha recebida de facto mede — ou `None` se um índice do arco
        /// nem sequer existe nela.
        measured: Option<f32>,
    },
}

/// O que a montagem mediu.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FillReport {
    /// Quantos quads.
    pub quads: usize,
    /// Quantas faces que **não** são quads. ⭐ Tem de ser **zero** — é a promessa
    /// inteira desta família de algoritmos.
    pub non_quads: usize,
    /// Quantos vértices.
    pub verts: usize,
    /// ⭐ Quantos vértices **irregulares** (valência ≠ 4). É a grandeza que o
    /// artista vê, e a que o pivô existiu para derrubar: a família local entregava
    /// 21 a 49 %, o oráculo entrega 0,2 %.
    pub irregular: usize,
    /// ⚠️ Quantas arestas ficaram com **uma** face só. Tem de ser zero numa
    /// superfície fechada: é o instrumento que denuncia a malha rasgada.
    pub boundary_edges: usize,
    /// Quantas rondas de alisamento correram.
    pub smoothing: usize,
    /// ⚠️ Quantas faces tiveram de ser **invertidas** para o volume ficar
    /// positivo. É `0` ou `todas` — qualquer outro número seria orientação
    /// inconsistente, que é outro defeito.
    pub flipped: usize,
    /// ⭐ **DE ONDE vêm os irregulares** — ver [`Provenance`]. Sem esta
    /// decomposição, `irregular: 47` diz que há trabalho e não diz **em que
    /// fase**: um irregular num canto do layout é dívida do F3, um no centro de
    /// um patch é da valência que o F3 entregou, e um no interior de uma grade
    /// seria um bug desta crate.
    pub by_provenance: [usize; Provenance::COUNT],
    /// ⭐⭐ **A ARESTA MAIS LONGA da saída** — e ela é a primeira grandeza
    /// GEOMÉTRICA que este relatório alguma vez teve.
    ///
    /// ⛔ **Todo o resto deste struct é função pura dos ÍNDICES.** `quads`,
    /// `non_quads`, `boundary_edges` e `irregular` saem da combinatória das faces;
    /// uma malha com as posições embaralhadas dá exactamente os mesmos números.
    /// Foi assim que 10.515 gates ficaram verdes sobre um produto destruído
    /// (auditoria de 2026-08-21): *não existia uma única asserção que olhasse uma
    /// coordenada.*
    ///
    /// A régua do chamador é a razão para o alvo dele. Medido: caminho correcto
    /// **≤ 4× o alvo**; caminho destruído **18×** — que era o **diâmetro da peça**,
    /// uma aresta a atravessar a esfera de lado a lado.
    pub edge_max: f32,
    /// A aresta mediana. ⭐ É a que diz se a DENSIDADE saiu no alvo — a máxima diz
    /// se alguma coisa se partiu, esta diz se a grade tem o passo pedido.
    pub edge_median: f32,
}

/// **De onde um vértice da saída veio** — a chave para saber de quem é a dívida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Um **canto do layout**: onde três ou mais arcos se encontram. A valência
    /// dele é o número de arcos, e ⭐ **é o F3 que a decide** — cada junção em T
    /// que o traçado cria é um canto a mais.
    Corner,
    /// O interior de um **arco** partilhado. Deviam ser todos regulares.
    Arc,
    /// O **centro** de um patch. A valência é a do patch, logo um patch de 3 ou 5
    /// lados produz aqui um irregular **por construção** — é o preço do leque.
    Center,
    /// O interior de um **raio** do leque, do centro ao corte de um lado.
    Spoke,
    /// O interior de uma **grade** de Coons. ⛔ Um irregular aqui seria bug desta
    /// crate: uma grade regular não tem nenhum.
    Grid,
}

impl Provenance {
    /// Quantas classes existem.
    pub const COUNT: usize = 5;
    /// Os nomes, na ordem do array de [`FillReport::by_provenance`].
    pub const NAMES: [&'static str; Self::COUNT] =
        ["canto (F3)", "arco", "centro (F3)", "raio", "grade"];
}

/// **OS PONTOS DA SAÍDA, com a origem de cada um.**
///
/// ⭐ **Existe para que a posição e a proveniência não possam divergir.** A
/// primeira versão eram dois `Vec` a crescer lado a lado com um comentário a pedir
/// que assim continuassem — e um `push` esquecido num dos cinco sítios daria uma
/// decomposição deslocada, que **soma certo** e atribui a dívida à fase errada.
/// Aqui há um único `push`, e ele exige as duas coisas.
pub(crate) struct Points {
    pub(crate) pos: Vec<[f32; 3]>,
    pub(crate) prov: Vec<Provenance>,
}

impl Points {
    pub(crate) fn new() -> Self {
        Self {
            pos: Vec::new(),
            prov: Vec::new(),
        }
    }

    /// Acrescenta um ponto e devolve o índice dele.
    pub(crate) fn push(&mut self, p: [f32; 3], from: Provenance) -> u32 {
        self.pos.push(p);
        self.prov.push(from);
        u32::try_from(self.pos.len() - 1).unwrap_or(u32::MAX)
    }

    /// **Acrescenta um ponto POUSADO na superfície.**
    ///
    /// ⭐⭐ **É a diferença entre construir a grade NO ESPAÇO e construí-la SOBRE a
    /// forma**, e ela vale mais do que qualquer alisamento posterior.
    ///
    /// ⛔ **A primeira versão interpolava tudo em linha reta e deixava a
    /// reprojecção para o alisamento no fim.** Numa esfera de raio 1,0 a corda de
    /// um raio de leque mergulha para dentro, o Coons construído sobre cordas
    /// mergulhadas fica pior ainda, e as faces **dobram sobre si mesmas** —
    /// exactamente as fendas escuras que o Enio fotografou em 2026-08-21.
    ///
    /// Medido nessa esfera (4 922 quads), faces dobradas contra rondas de
    /// alisamento: `0 → 405 · 1 → 403 · 3 → 289 · 6 → 205 · 12 → 135`. ⭐ **O
    /// alisamento REPARA e não CAUSA** — ele nunca chega a zero porque o estrago
    /// já veio pronto da construção. *Um remédio que melhora monotonicamente e não
    /// cura está a tratar o sintoma.*
    pub(crate) fn push_on(&mut self, mesh: &Mesh, p: [f32; 3], seed: f32, from: Provenance) -> u32 {
        self.push(ph2d_remesh_iso::project_onto(mesh, p, seed), from)
    }
}
