//! `ph2d-field` — **o documento** do módulo de modelagem 3D ([ADR-0161]).
//!
//! O modelo não é uma malha nem um grid de voxels: é uma **árvore de expressão autorada**.
//! Primitivas, transformações e operações com raio. Perguntar *"esta forma existe no ponto p?"* é
//! avaliar `f(p)` — e é dessa escolha que decorrem, como consequência e não como promessa:
//!
//! - **booleana não pode falhar** (união é `min(a, b)`: não existe geometria degenerada para uma
//!   comparação de dois números);
//! - **o arredondamento não pode falhar**, e funciona onde três ou mais formas se encontram — o
//!   caso que quebra o `Bevel` do Blender e o rolling-ball do CAD;
//! - **o raio fica editável para sempre**, porque é parâmetro da operação e não geometria assada.
//!   ⭐ Nem o Blender nem o MoI dão isto.
//!
//! # ⚠️ Esta crate NÃO avalia
//!
//! Nenhuma linha aqui nomeia o motor de avaliação. Ele vive na `ph2d-field-eval`, e a fronteira é a
//! razão de existir desta crate: trocar de motor tem de ser trabalho de **uma** crate, e — o que
//! importa mais — **nenhum arquivo salvo pode quebrar** quando isso acontecer. O documento do
//! utilizador não pode ter a forma que um terceiro escolheu para a estrutura interna dele.
//!
//! # A arena é ORDENADA POR CONSTRUÇÃO
//!
//! Os nós vivem num `Vec` e referem-se por índice. A invariante é dura: **todo filho tem índice
//! estritamente menor que o do pai**. Isso não é estilo — é o que torna ciclo uma
//! **impossibilidade** em vez de um erro a detectar, e faz a avaliação ser uma passagem de baixo
//! para cima sem recursão nem pilha de visitados.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

pub mod blend;
pub mod dims;
pub mod dims_scale;
pub mod mods;
pub mod mods_dims;
/// ⭐ O que uma forma **é** — ver [`primitive`].
pub mod primitive;
pub mod profile;
pub mod radius;
pub mod xform;

pub use blend::{Blend, Character, Joint};
pub use dims::{Dim, Param, Span, clamp_round, dims, scale_primitive, set_dim};
pub use mods::{Unary, UnaryKind};
// ⚠️ **O `pub use` é o que mantém `ph2d_field::Primitive`** — cortar um arquivo não pode custar uma
// reescrita em cada sítio que o chamava.
pub use primitive::{
    MAX_GEAR_TEETH, MAX_PRISM_SIDES, MAX_STAR_POINTS, MIN_GEAR_TEETH, MIN_PRISM_SIDES,
    MIN_STAR_POINTS, Primitive, PrimitiveKind,
};
pub use profile::{
    DEFAULT_PROFILE_RESOLUTION, FillRule, MAX_PROFILE_RESOLUTION, Profile, ProfileError, coarsen,
    coarsen_to_normal_error,
};
pub use radius::{
    Bound, bounding_radius, chamfer_of, characteristic_size, edge_shrink, fillet_inflates,
    round_limit, round_of, set_shape_radius,
};
pub use xform::Xform;

use serde::{Deserialize, Serialize};

/// Versão do formato serializado (**HR-14**: save format é versionado e migrável).
///
/// ⚠️ **Este número SOMA entre linhas** — se duas o incrementarem em paralelo, o git funde os dois
/// lados sem saber que são o mesmo degrau. Ao mexer, **conte**, não escolha
/// ([`CLAUDE.md §5.0`]).
///
/// v2: as primitivas de **perfil** ([`Primitive::Extrude`] / [`Primitive::Revolve`]) — o desenho do
/// editor vetorial virando sólido.
///
/// v3: o [`Node`] ganhou a pilha de **modificadores** ([`mods::Unary`] — casca, afastamento,
/// espelho e matriz). É
/// campo novo numa struct, e postcard é **posicional**, então um documento v2 não desserializa aqui.
/// ⚠️ **A migração é vazia, e isso tem de estar escrito:** nada persiste um [`FieldDoc`] — ele é
/// **cozido** da cena a cada quadro, e o que o arquivo de projeto guarda são os *componentes* ECS.
/// O degrau sobe na mesma, porque a alternativa é o número deixar de querer dizer alguma coisa no
/// dia em que alguém o persistir.
///
/// v4: o [`NodeKind`] ganhou a **escultura** ([`NodeKind::Sampled`]) — a ponte da W5. É variante
/// nova num `enum`, e postcard escreve o discriminante por índice, então um documento v3 leria um nó
/// `Sampled` onde havia outra coisa. ⚠️ **A migração continua vazia pelo mesmo motivo de sempre**
/// (nada persiste um [`FieldDoc`]), e o degrau sobe pela mesma razão.
///
/// v5: o [`Node`] ganhou o **verbo** ([`Node::verb`]) — cada forma traz a operação com que dobra
/// sobre o resultado das anteriores, em vez de a herdar toda do pai. É campo novo numa struct, e
/// postcard é **posicional**; a migração continua vazia pelo motivo de sempre.
///
/// v6: o [`Blend`] ganhou o **chanfro** ([`Blend::Chamfer`]) e o campo do orgânico passou de `k`
/// (o alcance cru) para `radius` (o **entregue**, calibrado por [`Blend::ORGANIC_REACH`]). São
/// variante nova num `enum` **e** mudança de significado de um número: um documento v5 leria o
/// alcance de um orgânico como se fosse raio, e a peça mudaria de forma em silêncio.
///
/// v7: o [`Primitive`] ganhou **três formas** ([`Primitive::Cone`], [`Primitive::Capsule`],
/// [`Primitive::Prism`]). São variantes **acrescentadas no fim** do `enum`, então nenhum índice
/// existente se move e um documento v6 continua a ler-se certo — o degrau sobe na mesma, pela lei
/// do módulo: *um número que se lê errado em silêncio é pior do que um load que recusa em voz alta*.
///
/// v8: o [`Primitive::Prism`] passou a ter **duas pontas** (`bottom`/`top`, o que o torna também a
/// pirâmide e o tronco dela), e entraram a [`Primitive::Wedge`] e o [`Primitive::TorusArc`]. ⚠️ O
/// prisma **mudou de forma**, não só a lista cresceu: um documento v7 leria o `half_height` dele
/// como `top`, e a peça mudaria em silêncio.
///
/// v9: entraram a [`Primitive::Star`], o [`Primitive::BoxFrame`] e o [`Primitive::Ellipsoid`]. São
/// variantes **acrescentadas no fim** do `enum` — nenhum índice existente se move —, e o degrau sobe
/// pela lei do módulo, como o v7.
///
/// v10: o [`Primitive::TorusArc`] ganhou `round`. É campo novo numa variante, e postcard é
/// **posicional**: um documento v9 leria o ângulo dele como filete.
///
/// v13: entrou o [`Unary::Bend`] — o irmão da torção, com a mesma forma. Variante **acrescentada no
/// fim** do `enum Unary`; nenhum índice existente se move, e o degrau sobe pela lei do módulo.
///
/// v12: o [`Unary::Twist`] ganhou o `falloff` — o ombro da banda. É campo novo numa variante, e
/// postcard é **posicional**, então o degrau sobe mesmo tendo a variante nascido no degrau anterior.
///
/// v11: entrou o [`Unary::Twist`] — o primeiro modificador novo desde os dois espelhos. É variante
/// **acrescentada no fim** do `enum Unary`, nenhum índice existente se move, e o degrau sobe pela lei
/// do módulo, como o v9. ⚠️ **E o golden de forma NÃO o teria apanhado**: a fixtura dele tem
/// `mods: Vec::new()`, e um vetor vazio custa um byte independentemente de quantas variantes o `enum`
/// tem — *um golden que não instancia a coisa nova não a defende*.
///
/// v14: a [`Unary::Array`] e a [`Unary::Radial`] ganharam a **junta entre as cópias**
/// ([`Joint`], pedido do Enio em 2026-08-30). São campos novos em variantes existentes, e postcard
/// é **posicional** — um documento v13 leria a contagem seguinte como o chanfro.
///
/// ⭐⭐⭐ **E é o primeiro degrau desta escada que um golden DEFENDE.** Os três anteriores (v11, v12,
/// v13) passaram os dois goldens de forma a verde, porque as duas fixturas deles têm
/// `mods: Vec::new()` — a nota do v11 abaixo já o dizia, e ninguém tinha construído o instrumento.
/// Ele existe agora: `the_shape_of_a_saved_modifier_stack_is_pinned`, com a fixtura **derivada** de
/// [`UnaryKind::ALL`], para que um modificador novo entre nela sozinho.
///
/// v15: as **21 primitivas com aresta** ganharam o `chamfer` (Enio, 2026-08-30: *«em todas as peças
/// temos fillet para as bordas arredondadas mas não temos um slider para chamfer»*). É campo novo em
/// variantes existentes, e postcard é **posicional** — um documento v14 leria o campo seguinte de
/// cada forma como o chanfro dela.
///
/// ⚠️ **Este degrau OS DOIS goldens de forma apanham**, ao contrário dos v11–v13: as fixturas deles
/// instanciam primitivas (uma caixa e uma extrusão), e o que lhes faltava era instanciar
/// **modificadores** — que é o buraco que a v14 fechou.
///
/// [`CLAUDE.md §5.0`]: ../../../CLAUDE.md
pub const FIELD_DOC_VERSION: u32 = 15;

/// Índice de um nó na arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

/// essa derivação.
/// As três operações booleanas.
///
/// ⚠️ Só a **união** precisa de fórmula própria: intersecção e subtração saem por **De Morgan**
/// (`A ∩ B = ¬(¬A ∪ ¬B)`), sem fórmula nova. Duplicar a fórmula seria uma segunda resposta à mesma
/// pergunta, com uma chance a mais de divergir — e quem avalia (`ph2d-field-eval`) faz exatamente
/// essa derivação.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Op {
    Union(Blend),
    Intersection(Blend),
    /// `children[0]` menos todos os seguintes.
    Difference(Blend),
}

impl Op {
    #[must_use]
    pub fn blend(self) -> Blend {
        match self {
            Op::Union(b) | Op::Intersection(b) | Op::Difference(b) => b,
        }
    }
}

/// ⭐⭐⭐ **A RECEITA de uma combinação, numa frase:** as formas dobram na **ordem** em que estão, e
/// cada uma traz o **verbo** com que se junta ao resultado das anteriores.
///
/// `((c₀ ⊕₁ c₁) ⊕₂ c₂) …`, onde `⊕ᵢ` é o verbo de `cᵢ` — ou o **do pai**, quando `cᵢ` não trouxe
/// nenhum. É a mesma lei que o vetorial desta casa já paga desde 2026-08-22
/// (`docs/Vector Module/27_um_verbo_por_forma.md`), e ela vale aqui **pela mesma razão pela qual foi
/// barata lá**: os dois avaliadores já eram uma dobra à esquerda; o que estava fixo era só o verbo.
///
/// # ⚠️ Ausência é HERANÇA, não «sem verbo»
///
/// `None` não quer dizer *«esta forma não se combina»* — quer dizer *«use o do pai»*. As duas
/// consequências pesam para o mesmo lado:
///
/// - **todo documento anterior a esta versão avalia byte-idêntico**, porque nele ninguém se
///   pronunciou;
/// - **o seletor do pai não morre**: ele deixa de ser *a* operação e passa a ser o **padrão** de
///   quem não se pronunciou. Sem essa escolha ele ficaria inerte, que é o defeito *«parâmetro que
///   não muda nada»*.
///
/// # ⚠️ O verbo do PRIMEIRO filho nunca é perguntado
///
/// Ele **semeia** o acumulado — não há nada antes dele com que dobrar. Guardá-lo mesmo assim é
/// deliberado: *reordenar não pode destruir a escolha de quem passou pelo topo.* Arrastar o
/// terceiro filho para cima torna-o base sem nada a consertar, e arrastá-lo de volta devolve o
/// verbo que ele tinha.
///
/// ⛔ **E não é «começar do vazio»**, que seria a outra forma de o dizer: com o acumulado a nascer
/// vazio, uma subtração no topo apagaria a peça inteira (`∅ − a = ∅`) — uma reordenação que
/// destrói o modelo em silêncio.
#[must_use]
pub fn fold_verb(parent: Op, child: Option<Op>) -> Op {
    child.unwrap_or(parent)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    Leaf(Primitive),
    Combine {
        op: Op,
        children: Vec<NodeId>,
    },
    /// ⭐ **Uma ESCULTURA**, referida pelo nome — a ponte da W5.
    ///
    /// ⚠️ **É um `NodeKind` e não uma `Primitive`, e a diferença é a razão de ele existir.** Uma
    /// primitiva é uma forma com **números** (raio, meia-extensão, `round`), e o painel deriva as
    /// linhas dela. Uma escultura não tem números: ela é uma malha, e o que a define vive noutro
    /// módulo. Metê-la entre as primitivas obrigaria toda a tabela de dimensões a ter um caso que
    /// não devolve nada.
    ///
    /// ⚠️ **O documento guarda o NOME, nunca a grade.** Uma grade de 128³ pesa 12 MB; pô-la aqui
    /// faria cada `cook` — que corre por quadro — copiar isso, e faria um projeto guardado carregar
    /// a grade em vez de a **regenerar** da malha, que é a fonte. Quem resolve nome → campo é o
    /// registo do avaliador (`ph2d_field_eval::hybrid::Registry`).
    ///
    /// ⚠️ **Um nome desconhecido lê como espaço VAZIO**, e não como sólido: numa união some, numa
    /// subtração não corta. O oposto encheria a cena de um bloco que ninguém autorizou.
    Sampled {
        key: String,
    },
}

impl NodeKind {
    /// O que este nó **é**, sem os filhos. Ver [`NodeShape`].
    #[must_use]
    pub fn shape(&self) -> NodeShape {
        match self {
            NodeKind::Leaf(p) => NodeShape::Leaf(p.clone()),
            NodeKind::Combine { op, .. } => NodeShape::Combine(*op),
            NodeKind::Sampled { key } => NodeShape::Sampled { key: key.clone() },
        }
    }
}

/// **O que um nó é, SEM a lista de filhos.**
///
/// ⭐ Existe porque a mesma árvore vive em dois sítios, e só um deles pode ser dono dos filhos:
///
/// | Onde | Quem são os filhos |
/// |---|---|
/// | [`FieldDoc`] (o **cozido**, o que se avalia) | índices da arena, em `NodeKind::Combine` |
/// | a **cena** (a fonte, o que o artista vê e move) | a hierarquia ECS (`Children`) |
///
/// Guardar a lista nos dois seria a segunda verdade clássica, e o sintoma seria específico e feio:
/// uma peça cuja **forma discorda da Hierarquia** — arrastar um objeto para dentro de outro no
/// painel mudaria a árvore que o artista vê e não a que o traçador avalia.
///
/// *Uma árvore, um dono dos filhos.* É a mesma lei que o vetorial paga como **fonte ≠ cozido**
/// (ADR-0121/0132).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeShape {
    Leaf(Primitive),
    Combine(Op),
    /// A escultura, pelo nome. Ver [`NodeKind::Sampled`].
    Sampled {
        key: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub xform: Xform,
    pub kind: NodeKind,
    /// ⭐ **A pilha de modificadores**, aplicada ao campo deste nó **depois** do que ele é e
    /// **antes** da pose dele. Vazia na esmagadora maioria dos nós. Ver [`crate::mods`].
    #[serde(default)]
    pub mods: Vec<Unary>,
    /// ⭐⭐⭐ **O VERBO com que este nó dobra sobre o resultado dos irmãos anteriores** — `None`
    /// herda o do pai. A lei inteira, com o que cada metade compra, está em [`fold_verb`].
    ///
    /// ⚠️ **Aqui e não no pai**, e a diferença é estrutural: um verbo guardado no pai como lista
    /// paralela a `children` seria uma segunda resposta a *«quantos filhos há»*, e ela ficaria
    /// obsoleta em todo sítio que desloca índices — o [`FieldDoc::union_all`] é um deles. Preso ao
    /// nó, ele viaja com o nó de graça.
    #[serde(default)]
    pub verb: Option<Op>,
}

impl Node {
    /// Um nó sem modificadores e que **herda** o verbo do pai — a forma curta, que é o caso de
    /// quase todo nó.
    #[must_use]
    pub fn new(xform: Xform, kind: NodeKind) -> Self {
        Self {
            xform,
            kind,
            mods: Vec::new(),
            verb: None,
        }
    }

    /// ⭐ **O verbo com que este nó dobra**, dado o do pai. Ver [`fold_verb`] — a lei vive lá, e
    /// esta é a forma curta para quem tem o nó em mãos.
    #[must_use]
    pub fn fold_verb(&self, parent: Op) -> Op {
        fold_verb(parent, self.verb)
    }
}

/// O documento: a arena de nós e a raiz.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldDoc {
    pub version: u32,
    nodes: Vec<Node>,
    root: NodeId,
}

/// Por que um documento foi recusado.
///
/// ⚠️ Cada variante corresponde a um jeito de o campo **deixar de ser uma distância** ou de a
/// árvore deixar de ser uma árvore. Nenhuma é zelo: um documento inválido não produz um erro — ele
/// produz uma forma errada, em silêncio, três waves adiante.
// ⚠️ Sem `Eq`: `RoundTooLarge` carrega os `f32` que explicam a recusa (o raio pedido e o limite),
// e `f32` não é `Eq` por causa do NaN. Guardar os números vale mais do que a igualdade total —
// uma recusa que diz *"0,08 não cabe em 0,06"* poupa a próxima pessoa de ir medir.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldError {
    /// A arena está vazia, ou a raiz aponta para fora dela.
    BadRoot,
    /// Um filho tem índice ≥ o do pai — a invariante topológica (ver o doc da crate).
    ForwardReference { parent: u32, child: u32 },
    /// Uma operação sem filhos não tem o que combinar.
    EmptyCombine { node: u32 },
    /// Dimensão não-positiva (raio, altura, escala).
    NonPositive { node: u32, what: &'static str },
    /// O arredondamento não cabe na forma: a fonte encolhida ficaria negativa.
    RoundTooLarge { node: u32, round: f32, limit: f32 },
    /// Escala não-uniforme, ou não-finita (ver [`Xform::scale`]).
    BadScale { node: u32 },
    /// O perfil de um [`Primitive::Revolve`] tem ponto com `x < 0` — a superfície de revolução
    /// auto-intersecta e o campo deixa de ser uma distância.
    ProfileCrossesAxis { node: u32, min_x: f32 },
    /// Uma escultura sem nome não pode ser resolvida contra registo nenhum.
    EmptySampledKey { node: u32 },
    /// ⚠️ Modificadores sobre uma escultura — ver a nota de [`NodeKind::Sampled`] na validação.
    ModsOnSampled { node: u32 },
}

impl FieldDoc {
    /// Constrói e **valida**. Só há esta porta: um `FieldDoc` que exista está válido.
    ///
    /// # Errors
    /// Ver [`FieldError`].
    pub fn new(nodes: Vec<Node>, root: NodeId) -> Result<Self, FieldError> {
        let doc = Self {
            version: FIELD_DOC_VERSION,
            nodes,
            root,
        };
        doc.validate()?;
        Ok(doc)
    }

    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    #[must_use]
    pub fn root(&self) -> NodeId {
        self.root
    }

    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0 as usize)
    }

    /// Une vários documentos num só — **uma cena É a união dos seus objetos**.
    ///
    /// As arenas são concatenadas com deslocamento de índice e um nó de combinação novo recebe as
    /// raízes. ⚠️ **A invariante topológica sobrevive de graça**: cada arena já vem ordenada, o
    /// deslocamento preserva a ordem relativa, e a raiz nova é o último índice — logo todo filho
    /// continua vindo antes do pai, sem precisar de ordenação nem de verificação extra.
    ///
    /// Devolve `None` para uma lista vazia: uma cena sem objetos não tem campo, e um documento
    /// vazio inventado aqui seria uma forma que ninguém pediu.
    ///
    /// # Errors
    /// Só se o resultado violar a validação — o que, dadas entradas válidas, não pode acontecer;
    /// o `Result` existe para que isso seja verificado e não assumido.
    pub fn union_all(docs: &[FieldDoc], blend: Blend) -> Option<Result<Self, FieldError>> {
        match docs.len() {
            0 => return None,
            1 => return Some(Ok(docs[0].clone())),
            _ => {}
        }
        let mut nodes: Vec<Node> = Vec::new();
        let mut roots: Vec<NodeId> = Vec::new();
        for doc in docs {
            let base = nodes.len() as u32;
            for node in &doc.nodes {
                let mut node = node.clone();
                if let NodeKind::Combine { children, .. } = &mut node.kind {
                    for c in children.iter_mut() {
                        c.0 += base;
                    }
                }
                nodes.push(node);
            }
            roots.push(NodeId(doc.root.0 + base));
        }
        // ⚠️ **A raiz adotada perde o verbo dela**, e isto é decisão e não zelo: um verbo autorado
        // dentro de uma peça fala dos **irmãos dela**, e aqui ele passaria a falar das **outras
        // peças** da cena — uma peça inteira a subtrair-se de outra sem ninguém o ter pedido. Esta
        // porta chama-se `union_all`; a união é o contrato, e não uma omissão a herdar.
        for r in &roots {
            nodes[r.0 as usize].verb = None;
        }
        let root = NodeId(nodes.len() as u32);
        nodes.push(Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Combine {
                op: Op::Union(blend),
                children: roots,
            },
            mods: Vec::new(),
            verb: None,
        });
        Some(Self::new(nodes, root))
    }

    fn validate(&self) -> Result<(), FieldError> {
        if self.nodes.is_empty() || self.root.0 as usize >= self.nodes.len() {
            return Err(FieldError::BadRoot);
        }
        for (i, node) in self.nodes.iter().enumerate() {
            let idx = i as u32;
            if !node.xform.scale.is_finite() || node.xform.scale <= 0.0 {
                return Err(FieldError::BadScale { node: idx });
            }
            match &node.kind {
                NodeKind::Combine { children, .. } => {
                    if children.is_empty() {
                        return Err(FieldError::EmptyCombine { node: idx });
                    }
                    for c in children {
                        // A invariante topológica: filho SEMPRE antes do pai.
                        if c.0 >= idx {
                            return Err(FieldError::ForwardReference {
                                parent: idx,
                                child: c.0,
                            });
                        }
                    }
                }
                NodeKind::Leaf(p) => validate_primitive(idx, p)?,
                NodeKind::Sampled { key } => {
                    if key.is_empty() {
                        return Err(FieldError::EmptySampledKey { node: idx });
                    }
                    // ⚠️ **A pilha de modificadores NÃO corre sobre uma escultura, e recusar é a
                    // única resposta honesta.** Aplicá-la exigiria a casca, a matriz e a inclinação
                    // escritas uma segunda vez em números — cada uma com o gate de paridade que a
                    // segure. Deixá-la passar em silêncio daria um botão que não faz nada, que é o
                    // modo de falha que nenhum smoke apanha.
                    if !node.mods.is_empty() {
                        return Err(FieldError::ModsOnSampled { node: idx });
                    }
                }
            }
            // ⚠️ **A pilha valida com a MESMA porta que a escreve** (`Unary::set_value`), e não com
            // uma segunda cópia das regras aqui: duas listas de *"o que é um número aceitável"*
            // divergem na primeira variante nova, e a que fica errada é sempre a que ninguém lê.
            for m in &node.mods {
                for (field, d) in m.dims().into_iter().enumerate() {
                    let mut probe = *m;
                    probe.set_dim(idx, field as u8, d.value)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

/// ⭐ O que cada forma recusa — ver [`validate_primitive`].
#[path = "validate_primitive.rs"]
mod validate_primitive;
use validate_primitive::validate_primitive;
