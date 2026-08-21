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

pub mod dims;
pub mod profile;
pub mod radius;
pub mod xform;

pub use dims::{Dim, Param, clamp_round, dims, scale_primitive, set_dim};
pub use profile::{FillRule, Profile, ProfileError};
pub use radius::{Bound, characteristic_size, round_limit, set_shape_radius};
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
/// [`CLAUDE.md §5.0`]: ../../../CLAUDE.md
pub const FIELD_DOC_VERSION: u32 = 2;

/// Índice de um nó na arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

/// As primitivas. Cada uma é **distância exata** — dentro e fora.
///
/// ⚠️ O `round` de uma primitiva é o **arredondamento da aresta convexa** dela, e ele é feito por
/// **deslocamento da superfície** com a fonte encolhida na mesma medida (ADR-0161 §3). É por isso
/// que ele vive na primitiva e não numa operação: arredondar a aresta de uma caixa não envolve
/// segunda forma nenhuma.
// ⚠️ **Sem `Copy` desde a v2**, e a razão é `Extrude`: um perfil é uma lista de pontos, e um tipo
// que se copia por bit não pode conter um `Vec`. A alternativa — pôr os perfis numa segunda arena e
// referi-los por índice — foi recusada: ela mantinha o `Copy` e comprava, em troca, uma segunda
// classe inteira de erro (índice pendente), que é exatamente o que a arena de nós existe para
// tornar impossível. Uma invariante, um lugar.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Primitive {
    /// Caixa de meias-extensões `half`, com as 12 arestas arredondadas em `round`.
    Box { half: [f32; 3], round: f32 },
    /// Esfera. Não tem aresta, logo não tem `round`.
    Sphere { radius: f32 },
    /// Cilindro no eixo **Z** (outro eixo se obtém pela rotação do nó), com o aro das tampas
    /// arredondado em `round`.
    Cylinder {
        radius: f32,
        half_height: f32,
        round: f32,
    },
    /// Toro no plano XY: `major` é o raio do anel, `minor` a espessura do tubo.
    Torus { major: f32, minor: f32 },
    /// **O perfil puxado ao longo de Z**, de `−half_height` a `+half_height`, com o **aro** (a
    /// aresta entre a parede e a tampa) arredondado em `round`.
    ///
    /// ⚠️ As arestas **verticais** — as quinas do próprio contorno — não são assunto deste `round`:
    /// elas são o que o perfil desenhou. Quem as quer redondas arredonda a quina **no editor
    /// vetorial**, e o raio vivo de lá chega aqui já cozido. *Uma quina, um dono.*
    Extrude {
        profile: Profile,
        half_height: f32,
        round: f32,
    },
    /// **O perfil girado em torno do eixo Y.** O `x` do perfil é a distância ao eixo e o `y` é a
    /// altura.
    ///
    /// ⚠️ **Y, e não Z — de propósito, e ao contrário do [`Primitive::Cylinder`] e do
    /// [`Primitive::Torus`], que são simétricos em Z.** A regra que manda aqui não é a coerência
    /// entre primitivas, é a coerência com o **plano de desenho**: o perfil vem do editor vetorial,
    /// que desenha em XY, e o eixo de uma revolução tem de estar **dentro** do plano do perfil. A
    /// extrusão sai do plano (por Z), a revolução gira em torno de uma reta do plano (o Y). Quem
    /// quiser outro eixo roda o nó — é para isso que o [`Xform`] existe.
    ///
    /// ⚠️ O perfil **não pode cruzar o eixo** (`x < 0`): a superfície de revolução de um contorno
    /// que cruza o eixo auto-intersecta, e o campo que sai disso deixa de ser uma distância. O
    /// documento recusa ([`FieldError::ProfileCrossesAxis`]) em vez de produzir a forma errada.
    Revolve { profile: Profile },
}

/// O **caráter** do arredondamento de uma operação.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Blend {
    /// Aresta viva.
    Sharp,
    /// Raio **constante de verdade** — o *look* de produto, e o default do módulo.
    /// Medido: entrega o raio pedido com **0,00 %** de erro (ADR-0161 §3).
    Exact { radius: f32 },
    /// Transição contínua ("derretida").
    ///
    /// ⚠️ **`k` NÃO é um raio.** Medido: entrega **exatamente 3/4** do número, em todos os raios
    /// testados. Quem o mostrar na UI com a etiqueta "raio" mente 25 % ao utilizador, sempre — ou
    /// calibra (×4/3), ou lhe dá outro nome.
    Organic { k: f32 },
}

impl Blend {
    /// O raio (ou alcance) desta mistura, ou `0.0` se for viva.
    #[must_use]
    pub fn amount(self) -> f32 {
        match self {
            Blend::Sharp => 0.0,
            Blend::Exact { radius } => radius,
            Blend::Organic { k } => k,
        }
    }
}

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    Leaf(Primitive),
    Combine { op: Op, children: Vec<NodeId> },
}

impl NodeKind {
    /// O que este nó **é**, sem os filhos. Ver [`NodeShape`].
    #[must_use]
    pub fn shape(&self) -> NodeShape {
        match self {
            NodeKind::Leaf(p) => NodeShape::Leaf(p.clone()),
            NodeKind::Combine { op, .. } => NodeShape::Combine(*op),
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub xform: Xform,
    pub kind: NodeKind,
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
        let root = NodeId(nodes.len() as u32);
        nodes.push(Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Combine {
                op: Op::Union(blend),
                children: roots,
            },
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
            }
        }
        Ok(())
    }
}

fn validate_primitive(idx: u32, p: &Primitive) -> Result<(), FieldError> {
    let positive = |v: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v <= 0.0 {
            Err(FieldError::NonPositive { node: idx, what })
        } else {
            Ok(())
        }
    };
    let round_fits = |round: f32, limit: f32| -> Result<(), FieldError> {
        if !round.is_finite() || round < 0.0 || round >= limit {
            Err(FieldError::RoundTooLarge {
                node: idx,
                round,
                limit,
            })
        } else {
            Ok(())
        }
    };
    match *p {
        Primitive::Box { half, round } => {
            for h in half {
                positive(h, "half")?;
            }
            // ⚠️ O limite é a MENOR meia-extensão: a receita do arredondamento encolhe a caixa em
            // `round` nos três eixos, e uma delas ficando ≤ 0 não é "quase" — é uma caixa que
            // deixou de existir naquele eixo, e o campo que sai disso não é uma distância.
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Sphere { radius } => positive(radius, "radius"),
        Primitive::Cylinder {
            radius,
            half_height,
            round,
        } => {
            positive(radius, "radius")?;
            positive(half_height, "half_height")?;
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Torus { major, minor } => {
            positive(major, "major")?;
            positive(minor, "minor")
        }
        Primitive::Extrude {
            profile: _,
            half_height,
            round,
        } => {
            positive(half_height, "half_height")?;
            // ⚠️ O limite é a meia-altura, e **só** ela. Um `round` maior do que a meia-largura do
            // perfil não é um erro: a receita (encolher a fonte, depois deslocar) é uma **abertura
            // morfológica**, e o que ela faz a um pescoço mais fino que `2·round` é exatamente o
            // que arredondar com esse raio deveria fazer — o pescoço desaparece. O campo continua a
            // ser um limite conservador de distância; a forma é a certa.
            //
            // Na altura não é assim: com `round ≥ half_height` o termo axial inverte de sinal e o
            // sólido deixa de existir — isso não é abertura, é uma forma que ninguém pediu.
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Revolve { ref profile } => {
            let min_x = profile.bounds().0[0];
            if min_x < 0.0 {
                return Err(FieldError::ProfileCrossesAxis { node: idx, min_x });
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;
