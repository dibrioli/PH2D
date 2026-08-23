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
use ph2d_trace::PatchLayout;

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
    /// ⭐⭐ **QUANTOS PATCHES ACHATARAM** — ver [`crate::param`].
    ///
    /// ⚠️ **É a régua que diz se a cura CHEGOU ao patch que dói.** Um patch que não
    /// achata volta à construção antiga — a que interpola em `ℝ³` e agarra à face
    /// mais próxima —, e ela dobra. Sem esta contagem, *"as dobras não caíram"* não
    /// distingue *"a cura não funciona"* de *"a cura não correu"*.
    pub flattened: usize,
    /// Quantos patches o layout tinha — o denominador do [`Self::flattened`].
    pub patches: usize,
    /// ⭐⭐ **QUANTOS PONTOS DE INTERIOR CAÍRAM FORA do achatamento** e tiveram de
    /// usar o caminho antigo.
    ///
    /// ⚠️ **Sem ele, `flattened: 19/19` mente por omissão.** Um patch pode achatar e
    /// mesmo assim não colocar ponto nenhum pelo domínio — basta o `uv` de cada
    /// ponto cair fora de todo triângulo. *Uma contagem de FASES não substitui uma
    /// contagem de PONTOS* ([[feedback_a_defect_count_without_provenance_names_the_wrong_phase]]).
    pub sample_misses: usize,
    /// Quantos pontos de interior o domínio de facto colocou.
    pub sampled: usize,
    /// ⚠️ **O pior resíduo com que um achatamento parou.** O teorema de Tutte fala
    /// da solução **convergida**; uma iteração parada a meio continua dentro do
    /// polígono mas pode ter triângulos virados. *Se este número não for pequeno, a
    /// garantia não se aplica.*
    pub flatten_residual: f32,
    /// Quantas rondas o achatamento mais caro gastou.
    pub flatten_rounds: usize,
    /// ⭐⭐ **QUANTAS FACES DOBRARAM** — ver [`folded_against`], que é quem a mede.
    ///
    /// ⚠️ **É o defeito que o artista fotografa**, e é geométrico: as fendas
    /// escuras de 2026-08-21 são faces cuja normal aponta para o lado oposto ao da
    /// superfície por baixo delas. Nenhum outro campo deste relatório a vê — uma
    /// malha com 100 % de quads, casca fechada e a contagem certa de irregulares
    /// pode estar cheia delas.
    pub folded: usize,
    /// ⭐⭐ **A SEGUNDA régua** — ver [`folded_by_neighbours`]. Ela não consulta a
    /// referência, então o pescoço fino não a confunde. ⚠️ **As duas juntas é que
    /// são a régua**: esta é cega a uma malha inteiramente ao contrário, aquela tem
    /// piso de ruído onde a superfície se dobra sobre si mesma.
    pub folded_local: usize,
    /// ⭐⭐ **DE QUE FASE são os vértices das faces DOBRADAS** — ver [`Provenance`].
    ///
    /// ⚠️ **É a régua que impede arranjar a fase errada.** `folded: 18` diz que há
    /// trabalho e **não diz onde**: uma dobra entre pontos de `Grid` é da construção
    /// do interior; uma entre pontos de `Arc` é do TRAÇADO, e nenhuma mudança na
    /// construção lhe toca. *Uma contagem de defeitos sem proveniência nomeia a fase
    /// errada.*
    pub folded_prov: [usize; Provenance::COUNT],
    /// ⭐⭐ **DE QUE FASE são as pontas das arestas LONGAS** (acima de `3×` a
    /// mediana) — ver [`Provenance`].
    ///
    /// ⚠️ **É a régua que diz até onde o slider pode ir.** Medido em 2026-08-21: ao
    /// pedir `15×` mais quads a mediana fica em `1,03×` o alvo — a densidade está
    /// certa — e a **máxima** vai de `2,71×` a `8,14×`. *Uma grandeza que não
    /// acompanha o alvo tem um dono, e sem esta decomposição procura-se por
    /// eliminação.*
    pub edge_long_prov: [usize; Provenance::COUNT],
    /// ⭐⭐⭐ **A FORMA DE CADA QUAD** — ver [`QuadShape`] e [`quad_shape`].
    ///
    /// ⛔ **Ela entrou em 2026-08-22, depois de a quarta foto do artista vir com a
    /// palavra «péssimo» sobre uma malha que passava em TODAS as réguas deste
    /// struct** — incluindo [`Self::edge_max`], que nesse mesmo dia tinha caído de
    /// `57 %` da peça para `5,5 %`.
    ///
    /// ⚠️ **Todas as outras grandezas geométricas daqui são GLOBAIS**: a aresta mais
    /// longa da malha, a mediana de todas as arestas. *Um quad de `0,02 × 0,30` não
    /// move nenhuma das duas* — a longa dele está muito abaixo da máxima e a curta
    /// afunda-se na mediana de dezenas de milhares. E o defeito da foto é exactamente
    /// esse: faces esmagadas em faixas, numa malha cujos extremos estão bem.
    pub shape: crate::shape::QuadShape,
    /// ⚠️ **O pior desacordo do campo PENTEADO dentro de um patch**, em graus — ver
    /// [`crate::aligned`]. Perto de zero = os patches são combáveis; grande = há
    /// singularidade **dentro** de um deles, e a dívida é do **traçado**.
    ///
    /// ⭐ *É a régua que impede culpar esta fase por um patch que o F3 entregou mal.*
    pub holonomy: f32,
    /// ⭐ **Quantos patches RECUARAM para o achatamento harmónico** porque o
    /// alinhado virou triângulos no domínio — ver [`crate::aligned::flipped`].
    ///
    /// ⛔ **Sem esta contagem, «o alinhamento não mudou nada» não distingue *a cura
    /// não funciona* de *a cura não correu*.** É a mesma lição do
    /// [`Self::flattened`], um nível abaixo.
    pub fell_back: usize,
    /// ⭐⭐⭐ **Quantos patches acharam o mapa do RECTÂNGULO** — ver
    /// [`crate::rectangle`]. O denominador é o número de patches de **quatro lados**,
    /// que é o único `n` que passa por lá.
    ///
    /// ⛔ **Ela existe pela mesma razão que o [`Self::fell_back`], e a lição custou
    /// uma medição inteira:** a primeira corrida com o mapa novo devolveu números
    /// **idênticos** aos de sempre, e *«idêntico» lê exactamente igual quando a cura
    /// não funciona e quando a cura nunca correu*. As três redes do
    /// [`crate::rectangle`] recusam em silêncio de propósito — é esta contagem que
    /// torna o silêncio legível.
    pub slid: usize,
    /// **O DENOMINADOR do [`Self::slid`]** — quantos patches de **quatro lados** o
    /// layout tinha. ⚠️ Sem ele, `slid: 0` não distingue *nenhum deslizou* de *não
    /// havia nenhum*, que é a mesma omissão que o [`Self::patches`] cura um nível acima.
    pub quad_patches: usize,
    /// ⭐⭐⭐ **QUANTAS CÉLULAS DE DOMÍNIO cada coluna do [`Self::domain_skew`] mediu**
    /// — `(rectângulo, leque)`.
    ///
    /// ⛔⛔ **Sem isto, `domain_skew.0 = 0,0°` lê-se «a grade do rectângulo nasce
    /// PERFEITA» quando o que aconteceu foi que ninguém a mediu** — e foi exactamente
    /// essa leitura que partiu a investigação de 2026-08-23 em duas metades, uma delas
    /// inexistente (o balde do rectângulo estava vazio; a mediana de um vector vazio é
    /// zero). ⚠️ *Um zero de «não medido» e um zero de «perfeito» são o mesmo byte.*
    pub domain_cells: (usize, usize),
    /// ⭐⭐⭐ **O ENVIESAMENTO MEDIANO por FASE de origem** — ver [`Provenance`].
    ///
    /// ⛔ **Ela existe porque duas curas teoricamente correctas não moveram o
    /// número** (2026-08-22): pôr o campo no interior do achatamento, e pôr os lados
    /// do domínio na proporção dos segmentos. As duas mudaram o enviesamento mediano
    /// da orelha de `27°` para `27°`.
    ///
    /// ⚠️ **Quando duas hipóteses boas falham, o defeito não está onde se pensa** —
    /// e a resposta honesta é parar de supor e perguntar à malha *onde* ele mora. A
    /// face é classificada pela proveniência **dominante** dos quatro cantos dela.
    pub skew_prov: [f32; Provenance::COUNT],
    /// ⭐⭐⭐ **O enviesamento mediano de um RECTÂNGULO contra o de um LEQUE** — ver
    /// [`crate::shape::skew_by_fan`]. É o número que decide se o F3 tem de passar a
    /// entregar só patches de quatro lados.
    pub skew_by_fan: (f32, f32),
    /// ⭐⭐⭐ **O enviesamento mediano da grade NO DOMÍNIO** — antes de ela tocar na
    /// superfície. Ver [`crate::patch::Domain::dom_skew`].
    ///
    /// ⭐⭐⭐ **A leitura é a COMPARAÇÃO com a superfície, e ela mede a
    /// CONFORMALIDADE do mapa:** um mapa que preserva ângulos entrega na superfície o
    /// ângulo que o domínio encomendou. Esfera lisa, `d = 0,55`, rectângulos:
    ///
    /// | | domínio | superfície | folga |
    /// |---|---|---|---|
    /// | fronteira **presa** (o que shipa) | `1,0°` | `16°` | ⛔ **`15°` sem nome** |
    /// | fronteira a **deslizar** ([`crate::rectangle`]) | `12,4°` | `14°` | ⭐ `1,6°` |
    ///
    /// ⇒ a conformalidade **muda o enviesamento de sítio**, não o reduz.
    ///
    /// ⛔⛔ **A primeira coluna mediu NADA durante um dia** — ver
    /// [`Self::domain_cells`], que é a contagem sem a qual `0,0°` se lê como
    /// «perfeito».
    pub domain_skew: (f32, f32),
}

/// **De onde um vértice da saída veio** — a chave para saber de quem é a dívida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
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
    /// ⭐⭐ **E ele leva a DIREÇÃO de que lado o ponto veio.** Ver
    /// [`ph2d_remesh_iso::project_facing`]: dentro de um vinco côncavo o ponto mais
    /// próximo pode estar do **outro lado** da dobra — o eixo medial encosta na
    /// superfície —, e a face entre dois vizinhos assim aterrados vira uma lasca.
    /// `None` é o caminho antigo, e continua a ser o certo onde a direção seria uma
    /// **estimativa** em vez de um facto (ver o alisamento em [`crate::stitch`]).
    pub(crate) fn push_facing(
        &mut self,
        mesh: &Mesh,
        p: [f32; 3],
        seed: f32,
        facing: Option<[f32; 3]>,
        from: Provenance,
    ) -> u32 {
        self.push(ph2d_remesh_iso::project_facing(mesh, p, seed, facing), from)
    }
}

/// **A TOLERÂNCIA da pré-condição**, em fração do comprimento declarado.
///
/// ⚠️ **Ela é folga de ARREDONDAMENTO e nada mais.** No caminho correto os dois
/// números são a **mesma soma dos mesmos `f32`**, então a razão é `1,000` exacto;
/// `1e-3` cobre uma reordenação de soma e ainda deixa **três ordens de grandeza**
/// até o `5,40×` que o defeito produziu. ⛔ Alargá-la não compra robustez nenhuma:
/// compra o direito de voltar a montar uma malha sobre índices de outra.
pub(crate) const ARC_LENGTH_TOLERANCE: f32 = 1.0e-3;

/// **O LAYOUT É DESTA MALHA?** — a pré-condição do [`fill`].
///
/// ⭐ **Ela responde à única pergunta que a montagem não pode responder sozinha**,
/// e responde-a com aritmética que já está paga: o F3 mediu o comprimento de cada
/// arco quando o traçou, e o F4 usou esse número para decidir a quantização. Se a
/// malha que chega aqui medir outra coisa, o `arc_chain` **não é dela**.
///
/// ⚠️ **E ela absorve de graça o segundo defeito da mesma família:** quando o F1
/// REFINA em vez de grosseirar (toda entrada mais grossa que ~2.500 vértices), o
/// índice sai do alcance e o `src[v]` **panica** — a janela morre com a peça por
/// gravar. Aqui o mesmo `get` devolve uma recusa nomeada. *Reproduzido: o SEGUNDO
/// clique do botão era panic certo.*
pub(crate) fn check_arcs_belong_to(mesh: &Mesh, layout: &PatchLayout) -> Result<(), FillError> {
    let pos = mesh.positions();
    for (a, chain) in layout.arc_chain.iter().enumerate() {
        let declared = layout.arc_length.get(a).copied().unwrap_or(0.0);
        let mut measured = 0.0f32;
        for w in chain.windows(2) {
            // ⚠️ `get` e não `[]`: um índice fora do alcance é a MESMA doença, e
            // um panic no meio de um gesto do artista é a pior forma de a dizer.
            let (Some(a0), Some(a1)) = (pos.get(w[0] as usize), pos.get(w[1] as usize)) else {
                return Err(FillError::ArcNotOfThisMesh {
                    arc: a,
                    declared,
                    measured: None,
                });
            };
            let d = [a1[0] - a0[0], a1[1] - a0[1], a1[2] - a0[2]];
            measured += d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
        }
        if (measured - declared).abs() > ARC_LENGTH_TOLERANCE * declared.max(1.0e-6) {
            return Err(FillError::ArcNotOfThisMesh {
                arc: a,
                declared,
                measured: Some(measured),
            });
        }
    }
    Ok(())
}
