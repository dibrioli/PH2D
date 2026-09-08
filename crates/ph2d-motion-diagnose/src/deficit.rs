//! ⭐ **O VOCABULÁRIO DO DIAGNÓSTICO** — o que pode estar errado, e com que confiança o editor
//! pode agir.
//!
//! Módulo irmão por TETO DE LOC (HR-18, 700 para `crates/`), e o corte é por RESPONSABILIDADE:
//! aqui mora *o que se pode dizer sobre um nó*, e no `lib.rs` *como se descobre*. É o corte que a
//! própria [`Deficit::ALL`] já pedia — ela existe para que um censo leia a LISTA sem ler o
//! percurso.

use ph2d_nodegraph::graph::NodeId;

/// One diagnosed defect: a node whose placement makes its output inert.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// The offending producer.
    pub node: NodeId,
    /// What is wrong.
    pub deficit: Deficit,
    /// How to fix it (and how aggressively the editor may act — see [`Fix`]).
    pub fix: Fix,
}

/// The kind of defect found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deficit {
    /// This node writes the named transient column, and no node reachable
    /// downstream (via forward, non-`delayed` edges) consumes it — so it does
    /// nothing.
    InertProducer(&'static str),
    /// This node READS the named [required-upstream](REQUIRED_UPSTREAM) column (a
    /// deformer/force needs `P` to work on) but has NOTHING wired into it — so it has no
    /// stream to act on and is silently a no-op. Always a [`Fix::Offer`]: WHICH source
    /// (grid / emitter / object) is a creative choice.
    MissingSource(&'static str),
    /// This node declares the named input port REQUIRED (`NodeRegistry::required_inputs`,
    /// e.g. `motion.duplicator`'s `shape`/`points`) but that port has no edge — with
    /// nothing to copy, or nowhere to put it, the node is a silent no-op. Always a
    /// [`Fix::Offer`]: WHAT to wire into it is the artist's choice. Carries the PORT NAME.
    MissingInput(&'static str),
    /// **Outro nó do MESMO tipo já ocupa o único lugar que ele tem** — este é inerte.
    ///
    /// Os outros três déficits são sobre a POSIÇÃO de um nó no grafo; este é sobre a
    /// EXISTÊNCIA de um irmão. Ele existe para os nós que não são passos de um fluxo mas
    /// **configuram um passe de tela inteira**, lido uma vez a partir do grafo — para esses
    /// o segundo nó não compõe, ele é ignorado. Carrega o NOME DO TIPO.
    ///
    /// ⚠️ **Sempre [`Fix::Offer`]**: apagar qual dos dois é decisão do artista, e um
    /// auto-heal que apagasse um nó seria a única cura desta casa que destrói trabalho.
    Shadowed(&'static str),
    /// ⭐⭐ **O SUJEITO DO NÓ É ESCOLHIDO POR NOME, e ninguém o escolheu** — carrega o param de
    /// TEXTO em falta.
    ///
    /// Irmão exacto do [`Self::MissingInput`], um canal ao lado: aquele pergunta por uma PORTA
    /// sem aresta, este por um param de **texto** vazio. ⚠️ **A assimetria não é de estilo:** um
    /// `ParamSpec` é `f32`, e *«qual forma?»* não cabe num número — a resposta vive no canal de
    /// texto, ao lado do manifesto congelado. É a mesma razão por que o `ParamGateText` existe ao
    /// lado do `ParamGate`, e nenhuma pergunta sobre arestas vê este caso.
    ///
    /// Medido no `motion.spline_wrap` (Enio, 2026-09-08): sem forma escolhida ele não tem curva
    /// sobre que embrulhar, devolve o layout **intacto**, e nada no ecrã dizia porquê — o artista
    /// via um nó ligado, uma cadeia completa, e nenhuma mudança.
    ///
    /// ⚠️ **Sempre [`Fix::Offer`]**: QUAL caminho é escolha do artista, e um auto-heal que
    /// escolhesse um por ele inventaria conteúdo.
    MissingChoice(&'static str),
    /// **Um BURACO no meio das portas de um roteador** — `in1` vazia com `in2` ligada.
    ///
    /// Um `value.switch` escolhe por índice: `clamp(round(select), 0, N−1)`. Uma porta
    /// vazia é um índice que existe e **lê `0.0`** — e `0` é um valor legítimo, então o
    /// artista não distingue *"esta ramificação está vazia"* de *"esta ramificação vale
    /// zero"*. Medido: com só `in0`/`in1` ligadas, `select = 2` e `select = 3` devolvem
    /// `0.000` sem sinal nenhum.
    ///
    /// ⚠️ **Só o buraco do MEIO é diagnosticado, e a distinção é o que impede o ruído:**
    /// deixar `in2`/`in3` vazias é como se escreve um mux de duas vias — legítimo, comum,
    /// e o `select` nem lá chega (o clamp para em `N−1`... e é justamente por o clamp
    /// parar em `N−1` e não no ÚLTIMO LIGADO que a cauda vazia também lê zero; isso é
    /// comportamento **documentado e deliberado** no doc-comment do kernel daquele nó, e
    /// mudá-lo seria outra função). Uma porta vazia **antes** de uma ligada não tem
    /// leitura inocente: o índice dela está no meio do alcance que o artista está a
    /// varrer.
    ///
    /// ⚠️ **Sempre [`Fix::Offer`]**: o que ligar ali é escolha do artista, e ligar por
    /// palpite é a única cura desta casa que INVENTA conteúdo. Carrega o NOME DA PORTA.
    DeadBranch(&'static str),
}

impl Deficit {
    /// **Um exemplar de CADA variante — a fonte de qualquer censo sobre esta lista.**
    ///
    /// ⚠️ **Ela existe porque um `enum` não se itera, e o consumidor que mais importa termina
    /// num `_ =>`:** o `explain` do shell escolhe a frase que o artista lê, e um variante novo
    /// cai no catch-all *em silêncio* — compila, corre, e diz ao artista uma coisa que não é o
    /// defeito dele. Uma lista escrita à mão do lado do gate teria o mesmo buraco um nível
    /// acima (é preciso lembrar de a estender); aqui ela mora **ao lado da definição**, onde
    /// quem acrescenta o variante já está.
    ///
    /// Os payloads são exemplos reais do repo, não placeholders: uma frase pode depender do
    /// conteúdo (`InertProducer("accel")` tem braço próprio), então um censo sobre nomes
    /// inventados provaria menos do que parece.
    pub const ALL: &'static [Deficit] = &[
        Deficit::InertProducer("accel"),
        Deficit::InertProducer("inv_mass"),
        Deficit::InertProducer("falloff"),
        Deficit::MissingSource("P"),
        Deficit::MissingInput("shape"),
        Deficit::MissingChoice("path"),
        Deficit::Shadowed("fx.glow"),
        Deficit::DeadBranch("in1"),
    ];
}

/// The suggested cure, carrying how confidently the editor may apply it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    /// Insert this canonical consumer node type to make the producer live
    /// (`accel` → `motion.integrate`, or `sim.step` in a particle chain). The
    /// **AUTO-HEAL** candidate — unambiguous plumbing the artist forgot.
    Insert(&'static str),
    /// A consumer of this column exists in the graph, but not on the producer's
    /// forward path — the cure is to REORDER (put the producer upstream of it),
    /// never to insert a second one (one integrator applies). An **OFFER**.
    Reorder,
    /// The missing consumer is a creative choice with no canonical inserter
    /// (`inv_mass` needs *a* solver; `falloff` needs *a* force/deformer) — surface
    /// it, never guess. An **OFFER/AVISO**.
    Offer,
}
