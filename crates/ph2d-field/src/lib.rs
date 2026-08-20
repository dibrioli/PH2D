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

pub mod profile;

pub use profile::{FillRule, Profile, ProfileError};

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

/// Pose de um nó: translação, rotação e escala **uniforme**.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Xform {
    pub translation: [f32; 3],
    /// Quaternion `(x, y, z, w)`.
    pub rotation: [f32; 4],
    /// ⛔ **UNIFORME de propósito.** Escala não-uniforme **destrói a propriedade de distância**
    /// (‖∇f‖ = 1), que é a fundação de tudo neste módulo: sem ela o raio deixa de ser o raio, a
    /// casca perde a espessura e a marcha de raios atravessa a superfície. Quem quer um elipsoide
    /// usa uma primitiva de elipsoide — não uma esfera esticada (ADR-0161 §6).
    pub scale: f32,
}

impl Default for Xform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Xform {
    pub const IDENTITY: Self = Self {
        translation: [0.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: 1.0,
    };

    #[must_use]
    pub fn at(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: [x, y, z],
            ..Self::IDENTITY
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    Leaf(Primitive),
    Combine { op: Op, children: Vec<NodeId> },
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

    /// **O raio EDITÁVEL de um nó** — `None` quando não há nenhum.
    ///
    /// ⭐ É a promessa central do módulo virada em função: *o raio fica editável para sempre*,
    /// porque é parâmetro da operação e não geometria assada. Para uma combinação é o raio da
    /// mistura; para uma primitiva, o `round` da aresta convexa dela.
    #[must_use]
    pub fn radius_of(&self, node: NodeId) -> Option<f32> {
        match &self.node(node)?.kind {
            NodeKind::Combine { op, .. } => Some(op.blend().amount()),
            NodeKind::Leaf(p) => match p {
                Primitive::Box { round, .. }
                | Primitive::Cylinder { round, .. }
                | Primitive::Extrude { round, .. } => Some(*round),
                _ => None,
            },
        }
    }

    /// Até onde esse raio pode ir, e de que natureza é o limite. Ver [`RadiusBound`].
    #[must_use]
    pub fn radius_bound(&self, node: NodeId) -> Option<RadiusBound> {
        match &self.node(node)?.kind {
            // Um raio de mistura não tem limite de VALIDADE: o campo continua a ser uma distância
            // com qualquer raio. O que existe é escala — e ela vem da menor peça sob este nó,
            // porque um filete maior do que ela engole-a.
            NodeKind::Combine { .. } => Some(RadiusBound::Soft(self.subtree_scale(node))),
            NodeKind::Leaf(p) => round_limit(p).map(RadiusBound::Hard),
        }
    }

    /// A menor peça sob um nó — a escala que dá sentido a um raio de mistura.
    ///
    /// ⚠️ Uma passagem **de baixo para cima**, sem recursão: a invariante da arena (todo filho antes
    /// do pai) já garante que os filhos foram vistos quando se chega ao pai.
    fn subtree_scale(&self, node: NodeId) -> f32 {
        let mut scale = vec![f32::INFINITY; self.nodes.len()];
        for (i, n) in self.nodes.iter().enumerate() {
            scale[i] = match &n.kind {
                NodeKind::Leaf(p) => characteristic_size(p) * n.xform.scale,
                NodeKind::Combine { children, .. } => children
                    .iter()
                    .map(|c| scale[c.0 as usize])
                    .fold(f32::INFINITY, f32::min),
            };
        }
        let s = scale[node.0 as usize];
        if s.is_finite() && s > 0.0 { s } else { 1.0 }
    }

    /// **Muda o raio de um nó**, e **revalida**.
    ///
    /// ⚠️ A revalidação é a razão de esta ser a única porta: a invariante da crate é *um documento
    /// que existe está válido*, e um `set` que a quebrasse produziria a forma errada em silêncio, e
    /// não um erro. Quando ela recusa, o documento fica **como estava** — um documento meio-mudado
    /// seria pior do que a recusa.
    ///
    /// Numa mistura viva (`Blend::Sharp`), um raio positivo acorda-a como [`Blend::Exact`]: a aresta
    /// viva é o raio zero, e não um modo à parte.
    ///
    /// # Errors
    /// Ver [`FieldError`]. `BadRoot` se o nó não existe; `NonPositive` para um raio não-finito.
    pub fn set_radius(&mut self, node: NodeId, radius: f32) -> Result<(), FieldError> {
        let idx = node.0 as usize;
        if idx >= self.nodes.len() {
            return Err(FieldError::BadRoot);
        }
        if !radius.is_finite() || radius < 0.0 {
            return Err(FieldError::NonPositive {
                node: node.0,
                what: "radius",
            });
        }
        let previous = self.nodes[idx].clone();
        match &mut self.nodes[idx].kind {
            NodeKind::Combine { op, .. } => {
                let blend = match (*op).blend() {
                    // O caráter ORGÂNICO é uma escolha de produto e sobrevive a mudar o número;
                    // trocá-lo aqui seria decidir por quem só mexeu num slider.
                    Blend::Organic { .. } => Blend::Organic { k: radius },
                    _ if radius <= 0.0 => Blend::Sharp,
                    _ => Blend::Exact { radius },
                };
                *op = match *op {
                    Op::Union(_) => Op::Union(blend),
                    Op::Intersection(_) => Op::Intersection(blend),
                    Op::Difference(_) => Op::Difference(blend),
                };
            }
            NodeKind::Leaf(p) => match p {
                Primitive::Box { round, .. }
                | Primitive::Cylinder { round, .. }
                | Primitive::Extrude { round, .. } => *round = radius,
                _ => {
                    return Err(FieldError::NonPositive {
                        node: node.0,
                        what: "radius",
                    });
                }
            },
        }
        if let Err(e) = self.validate() {
            self.nodes[idx] = previous;
            return Err(e);
        }
        Ok(())
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

/// **Até onde o `round` desta primitiva pode ir** — `None` se ela não tem `round`.
///
/// ⭐ **É a MESMA função que a validação usa.** Um painel que calculasse o próprio teto ofereceria
/// valores que o documento recusa, e o utilizador veria o controle parar sem explicação — a forma
/// clássica de dois lados divergirem sobre a mesma regra.
#[must_use]
pub fn round_limit(p: &Primitive) -> Option<f32> {
    match p {
        // A MENOR meia-extensão: a receita encolhe a caixa em `round` nos três eixos, e uma delas
        // ficando ≤ 0 não é "quase" — é uma caixa que deixou de existir naquele eixo.
        Primitive::Box { half, .. } => Some(half[0].min(half[1]).min(half[2])),
        Primitive::Cylinder {
            radius,
            half_height,
            ..
        } => Some(radius.min(*half_height)),
        // Só a meia-altura: um `round` maior que a meia-largura do perfil é uma ABERTURA, não um
        // erro (ver a nota de [`Primitive::Extrude`]).
        Primitive::Extrude { half_height, .. } => Some(*half_height),
        Primitive::Sphere { .. } | Primitive::Torus { .. } | Primitive::Revolve { .. } => None,
    }
}

/// **O tamanho característico de uma primitiva** — a menor dimensão que a define.
///
/// É o que dá escala a um raio de mistura: um filete maior do que a peça menor que ele junta
/// engole-a. Não é uma regra de validade (não existe nenhuma), é a escala do documento.
#[must_use]
fn characteristic_size(p: &Primitive) -> f32 {
    match p {
        Primitive::Box { half, .. } => half[0].min(half[1]).min(half[2]),
        Primitive::Sphere { radius } => *radius,
        Primitive::Cylinder {
            radius,
            half_height,
            ..
        } => radius.min(*half_height),
        Primitive::Torus { minor, .. } => *minor,
        Primitive::Extrude {
            profile,
            half_height,
            ..
        } => {
            let (min, max) = profile.bounds();
            half_height.min((max[0] - min[0]).min(max[1] - min[1]) * 0.5)
        }
        Primitive::Revolve { profile } => {
            let (min, max) = profile.bounds();
            (max[0] - min[0]).min(max[1] - min[1]) * 0.5
        }
    }
}

/// Até onde um raio pode ir, e **de que natureza é esse limite**.
///
/// ⚠️ A distinção não é decorativa e por isso está no tipo, em vez de num comentário: um limite de
/// **validade** é uma parede (o documento recusa), e um de **escala** é uma sugestão (a forma
/// continua correta, só deixa de ser útil). Um controle que os pintasse igual mentiria numa das
/// duas direções — ou proibiria o que é legítimo, ou ofereceria o que vai ser recusado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RadiusBound {
    /// O documento **recusa** acima disto.
    Hard(f32),
    /// Não há limite de validade. Este é o alcance **útil**, derivado do tamanho da peça.
    Soft(f32),
}

impl RadiusBound {
    #[must_use]
    pub fn value(self) -> f32 {
        match self {
            RadiusBound::Hard(v) | RadiusBound::Soft(v) => v,
        }
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
