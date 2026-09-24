//! `ph2d-field-eval` — a **ponte** entre o documento autorado e o motor de avaliação ([ADR-0161]).
//!
//! Documento (`ph2d-field`) → árvore de avaliação → malha (`ph2d-mesh`).
//!
//! # ⚠️ Esta é a única crate do repositório que nomeia o motor
//!
//! Não é arrumação: é o preço combinado por entrar num motor que o próprio autor chama de
//! experimental. Se ele mudar, morrer ou precisar de fork, o que se reescreve é **uma** crate — e
//! **nenhum arquivo salvo pelo utilizador quebra**, porque o documento não sabe que ele existe.
//!
//! # A compilação é uma passagem só, sem recursão
//!
//! A arena de `ph2d-field` garante que **todo filho tem índice menor que o do pai**. Isso não é
//! detalhe de arrumação: é o que permite compilar de baixo para cima num `for`, sem pilha de
//! visitados, sem detecção de ciclo e sem estouro de pilha numa árvore funda.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

pub mod bounds;
/// ⭐ A caixa da peça DOBRADA — ver [`bounds_bend`].
pub(crate) mod bounds_bend;
/// ⭐ A caixa que a MARCHA percorre — ver [`bounds_clip`].
pub mod bounds_clip;
/// ⭐ Intervalos com derivada — o substrato do bound da composição.
pub(crate) mod bounds_iv;
/// ⭐ O bound da COMPOSIÇÃO — ver [`bounds_lip`].
pub(crate) mod bounds_lip;
/// ⭐ O que um MODIFICADOR faz ao bordo — ver [`bounds_mods`].
pub(crate) mod bounds_mods;
/// ⭐⭐⭐ **A peça para o DISPOSITIVO, com a escultura dentro** — ver [`device`].
pub mod device;
pub mod extract;
pub mod hybrid;
pub mod ops;
/// ⭐ A família da SETA — ver [`ops_arrows`].
pub mod ops_arrows;
/// ⭐ Os BALÕES — ver [`ops_balloons`].
pub mod ops_balloons;
pub mod ops_bool;
pub mod ops_box;
/// ⭐ As EXACTAS do catálogo — ver [`ops_exact`].
/// ⭐ A BEZIER quadrática — ver [`ops_curve`].
pub mod ops_curve;
pub mod ops_exact;
/// ⭐ O FLUXOGRAMA — ver [`ops_flowchart`].
pub mod ops_flowchart;
/// ⭐ A SUPERFÓRMULA de Gielis — ver [`ops_gielis`].
pub mod ops_gielis;
pub mod ops_joint;
/// ⭐ O NÓ DE TORO — ver [`ops_knot`].
pub mod ops_knot;
/// ⭐ O GYROID e a família das redes — ver [`ops_lattice`].
pub mod ops_lattice;
pub(crate) mod ops_norm;
/// ⭐ Os blocos 2D partilhados por toda chapa — ver [`ops_plate2d`].
pub(crate) mod ops_plate2d;
pub mod ops_plates;
pub mod ops_slab;
pub mod ops_solids;
/// ⭐ A ESPIRAL por fórmula — ver [`ops_spiral`].
pub mod ops_spiral;
/// ⭐ A SUPERQUADRÁTICA — ver [`ops_super`].
pub mod ops_super;
/// ⭐ Os SÍMBOLOS — ver [`ops_symbols`].
pub mod ops_symbols;
/// ⭐ A ROSCA e o SERRILHADO — ver [`ops_thread`].
pub mod ops_thread;
/// ⭐ O TRIÂNGULO de vértices quaisquer — ver [`ops_triangle`].
pub mod ops_triangle;
/// ⭐⭐⭐ **De quem é este ponto** — ver [`owners`].
pub mod owners;
pub mod profile;
/// ⭐⭐⭐ A lei do ARCO, numa porta só — ver [`profile_arc`].
pub(crate) mod profile_arc;
/// ⭐⭐ O perfil como CONSULTA (W56) — a cura do custo linear nas arestas.
pub mod profile_index;
/// ⭐ A pilha de modificadores de um nó — ver [`stack`].
pub(crate) mod stack;
pub(crate) mod stack_bend;
pub(crate) mod stack_mirror;
pub(crate) mod stack_taper;

/// ⭐ **A porta da SONDA para a torção** — ela existe porque a lei tem de ser medida **antes** de o
/// modificador nascer: a constante de segurança dela é o que decide se a peça fura.
///
/// ⚠️ `probe_*` e `#[doc(hidden)]`: é interno exposto para ser medido, e não API.
#[doc(hidden)]
#[must_use]
pub fn probe_twist(tree: &Tree, k: f64, safety: f64) -> Tree {
    stack::twist_with(tree, k, safety)
}

/// A irmã da [`probe_twist`] com a lei que SHIPA — banda e divisor constante.
#[doc(hidden)]
#[must_use]
pub fn probe_twist_band(tree: &Tree, k: f64, lower: f64, upper: f64, reach: f64) -> Tree {
    stack::twist(tree, k, lower, upper, 0.0)
        / Tree::constant(stack::twist_sigma(k.abs() * reach.abs()))
}

/// O tecto espectral da torção — ver `stack::twist`.
#[doc(hidden)]
#[must_use]
pub fn probe_twist_sigma(t: f64) -> f64 {
    stack::twist_sigma(t)
}

use fidget::context::Tree;
use ph2d_field::{Blend, FieldDoc, Node, NodeKind, Op, Primitive, Unary, Xform};
pub(crate) use stack::stacked;

/// O motor de avaliação. Ver a nota do `Cargo.toml` sobre o `jit` estar ligado por medição.
pub type Engine = fidget::jit::JitShape;

/// Compila o documento numa árvore de avaliação.
#[must_use]
pub fn compile(doc: &FieldDoc) -> Tree {
    compile_with(doc, &hybrid::Registry::default())
}

/// A mesma compilação com o REGISTO à mão — a porta que o avaliador híbrido e as sondas partilham.
///
/// ⚠️ Ele entra porque o bordo de cada nó ([`bounds::local_balls`]) é hoje um **insumo** da pilha de
/// modificadores: a torção tira dele o divisor que a mantém uma distância honesta. Sem registo, uma
/// escultura lê como espaço vazio — exactamente o que a árvore já fazia com ela.
#[must_use]
pub fn compile_with(doc: &FieldDoc, reg: &hybrid::Registry) -> Tree {
    let balls = bounds::local_balls(doc, reg);
    let mut built: Vec<Tree> = Vec::with_capacity(doc.nodes().len());
    for (i, node) in doc.nodes().iter().enumerate() {
        // Seguro pela invariante da arena: todo filho já foi construído.
        let inner = match &node.kind {
            // ⭐ O divisor da ARESTA vive DENTRO do [`primitive_tree::primitive`], que é a única
            // porta que baixa uma forma — ver o doc dela e o report do Enio de 2026-08-30.
            NodeKind::Leaf(p) => primitive(p),
            NodeKind::Combine { op, children } => combine(*op, children, doc.nodes(), &built),
            // ⚠️ **Uma escultura NÃO é exprimível numa árvore** — ver [`hybrid`]. Aqui ela lê como
            // espaço vazio, que é o degenerado seguro: numa união some, numa subtração não corta.
            // Quem quiser a escultura de facto compila pelo [`hybrid::Hybrid`], e é ele que a
            // produção usa; esta porta serve às sondas e aos gates analíticos.
            NodeKind::Sampled { .. } => Tree::constant(f64::from(hybrid::ABSENT)),
        };
        // ⭐ **A pilha corre entre o que o nó É e onde ele ESTÁ**, e a ordem das duas metades é a
        // lei: em local, a espessura de uma casca é um número do nó, e a pose de um ancestral
        // escala-a junto com tudo o mais do nó — exatamente como a largura de uma caixa. Aplicá-la
        // depois de `place` faria o único número deste módulo que **não** obedece à cadeia.
        built.push(place(
            &stacked(&inner, &node.mods, balls[i].unwrap_or(bounds::Ball::EMPTY)),
            node.xform,
        ));
    }
    built[doc.root().0 as usize].clone()
}

fn blended(b: Blend) -> ops::Blended {
    match b {
        Blend::Sharp => ops::Blended::Sharp,
        Blend::Exact { radius } => ops::Blended::Exact(f64::from(radius)),
        Blend::Chamfer { radius } => ops::Blended::Chamfer(f64::from(radius)),
        // ⭐ **A CALIBRAÇÃO entra aqui, e só aqui** — o documento guarda o raio ENTREGUE e o operador
        // cru quer o alcance `k`. Ver [`Blend::ORGANIC_REACH`]: sem esta linha, trocar de carácter
        // com o mesmo número na tela mudaria o tamanho da peça.
        Blend::Organic { radius } => {
            ops::Blended::Organic(f64::from(radius * Blend::ORGANIC_REACH))
        }
        // ⭐ **A mesma calibração, com o grau dentro** (W145) — ver [`Blend::SOFT_REACH`].
        Blend::Soft { radius } => ops::Blended::Soft(f64::from(radius * Blend::SOFT_REACH)),
        // ⚠️ **As três seguintes NÃO são calibradas, e a diferença é o que o número significa.** O
        // `Organic` e o `Soft` precisam de conversão porque o alcance cru deles não é um raio; aqui
        // o número **é** a grandeza (a espessura do cordão, a profundidade do sulco, a altura do
        // friso), e multiplicá-lo por um factor faria o slider mentir.
        Blend::Bead { radius } => ops::Blended::Bead(f64::from(radius)),
        Blend::Groove { radius } => ops::Blended::Groove(f64::from(radius)),
        Blend::Ridge { radius, width } => ops::Blended::Ridge {
            height: f64::from(radius),
            width: f64::from(width),
        },
        Blend::Bevel { radius, bias } => ops::Blended::Bevel {
            recess: f64::from(radius),
            bias: f64::from(bias),
        },
    }
}

fn combine(op: Op, children: &[ph2d_field::NodeId], nodes: &[Node], built: &[Tree]) -> Tree {
    let kids: Vec<(Option<Op>, Tree)> = children
        .iter()
        .map(|c| (nodes[c.0 as usize].verb, built[c.0 as usize].clone()))
        .collect();
    combine_trees(op, &kids)
}

/// A mesma combinação, já sobre as árvores — a porta que o avaliador híbrido partilha.
///
/// ⭐ **A dobra sempre foi esta**; o que mudou em 2026-08-28 é que o verbo deixou de ser constante:
/// cada filho traz o dele, ou herda o do pai ([`ph2d_field::fold_verb`], onde a lei está escrita).
///
/// ⚠️ **E a MISTURA vem junto com o verbo**, porque ela vive dentro dele ([`Op`] carrega o
/// [`ph2d_field::Blend`]). É por isso que um raio por objeto sai desta mesma linha: quem traz o
/// verbo traz o raio da junção que ele faz.
pub(crate) fn combine_trees(parent: Op, kids: &[(Option<Op>, Tree)]) -> Tree {
    // ⚠️ O verbo de `kids[0]` **não é perguntado**: ele semeia o acumulado. Ver [`fold_verb`].
    let mut acc = kids[0].1.clone();
    for (verb, rhs) in &kids[1..] {
        let op = ph2d_field::fold_verb(parent, *verb);
        let b = blended(op.blend());
        let rhs = rhs.clone();
        acc = match op {
            Op::Union(_) => ops::union(&acc, &rhs, b),
            Op::Intersection(_) => ops::intersection(&acc, &rhs, b),
            // O acumulado menos este filho.
            Op::Difference(_) => ops::difference(&acc, &rhs, b),
        };
    }
    acc
}

/// Aplica a pose ao campo de um nó.
///
/// ⚠️ **A conta tem DUAS metades, e esquecer a segunda é o erro clássico.** O ponto vai para o
/// espaço local — `p' = R⁻¹(p − t) / s` — e o **valor** volta multiplicado por `s`. Sem essa
/// multiplicação o campo deixa de ser uma distância assim que houver escala: um raio de 1 mm num
/// nó escalado 2× mediria 0,5 mm, e a `f − r` da casca mentiria junto.
pub(crate) fn place(inner: &Tree, x: Xform) -> Tree {
    let s = f64::from(x.scale);
    let [tx, ty, tz] = x.translation.map(f64::from);
    let m = inverse_rotation_matrix(x.rotation);

    let px = (Tree::x() - Tree::constant(tx)) / Tree::constant(s);
    let py = (Tree::y() - Tree::constant(ty)) / Tree::constant(s);
    let pz = (Tree::z() - Tree::constant(tz)) / Tree::constant(s);

    let lx = px.clone() * Tree::constant(m[0][0])
        + py.clone() * Tree::constant(m[0][1])
        + pz.clone() * Tree::constant(m[0][2]);
    let ly = px.clone() * Tree::constant(m[1][0])
        + py.clone() * Tree::constant(m[1][1])
        + pz.clone() * Tree::constant(m[1][2]);
    let lz =
        px * Tree::constant(m[2][0]) + py * Tree::constant(m[2][1]) + pz * Tree::constant(m[2][2]);

    inner.remap_xyz(lx, ly, lz) * Tree::constant(s)
}

/// A matriz da rotação **inversa** (transposta, porque a rotação é ortonormal), a partir do
/// quaternion `(x, y, z, w)`.
pub(crate) fn inverse_rotation_matrix(q: [f32; 4]) -> [[f64; 3]; 3] {
    let [x, y, z, w] = q.map(f64::from);
    let n = (x * x + y * y + z * z + w * w).sqrt();
    // Quaternion nulo não define rotação nenhuma; a identidade é a única resposta honesta, e o
    // documento já recusa o resto (`FieldDoc::new`).
    if n <= 0.0 || !n.is_finite() {
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }
    let (x, y, z, w) = (x / n, y / n, z / n, w / n);
    // R (direta); a inversa é a transposta, e é ela que se devolve.
    let r = [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y - z * w),
            2.0 * (x * z + y * w),
        ],
        [
            2.0 * (x * y + z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z - x * w),
        ],
        [
            2.0 * (x * z - y * w),
            2.0 * (y * z + x * w),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ];
    [
        [r[0][0], r[1][0], r[2][0]],
        [r[0][1], r[1][1], r[2][1]],
        [r[0][2], r[1][2], r[2][2]],
    ]
}

/// Erros da extração de malha.
///
/// ⚠️ **A malha é o artefato de EXPORTAÇÃO, não o que o artista vê** (ADR-0161 §2) — a tela é o
/// campo traçado, e por isso não passa por aqui. Quem extrai é [`extract::extract`], e o *porquê*
/// de ele ser da casa está no doc-comment daquele módulo.
#[derive(Debug)]
pub enum MeshError {
    /// A malha saiu, mas a validação da `ph2d-mesh` a recusou.
    Rejected(String),
}

/// Atalho de leitura para quem monta um documento à mão.
#[must_use]
pub fn leaf(p: Primitive, xform: Xform) -> Node {
    Node::new(xform, NodeKind::Leaf(p))
}

mod affine;
/// ⭐⭐⭐ O documento avaliado ponto a ponto — ver [`field`].
pub mod field;
pub use field::Field;
/// ⏱️ A poda por região (Keeter 2020) — instrumento; ver o módulo.
#[doc(hidden)]
pub mod poda;
pub mod point_tape;
/// ⭐⭐⭐ A ordem da fita — ver [`tape_schedule`].
///
/// ⚠️ **O módulo é `pub` e os itens dele não são**, e isso é deliberado: a `schedule` recebe
/// `Instr`, que é interno, mas a **lei** dele é citada por nome em cinco sítios de duas outras
/// crates (o tecto do dispositivo, o pintor, as sondas). *Uma lei que outros têm de citar precisa
/// de endereço* — e sem o módulo público esses `[`…`]` não resolvem.
pub mod tape_schedule;
pub mod wgsl;
use affine::Affine;
/// ⭐⭐ **O contador de fitas de PONTO** — ver [`point_tape::POINT_TAPES`]. Público porque o
/// instrumento de um custo só serve a quem o paga, e quem o paga é o consumidor (a `Owners`, a
/// tabela de materiais do modelador).
pub use point_tape::POINT_TAPES;
mod hull;
pub use hull::{probe_hull_uv, probe_in_hull};
/// ⭐⭐⭐ Os cascos de uma região, por folha — ver [`region_hulls`].
mod region_hulls;
pub use region_hulls::RegionHulls;

#[path = "step.rs"]
mod step;
#[doc(hidden)]
pub use bounds_lip::stack_lipschitz as stack_lipschitz_probe;
#[doc(hidden)]
pub use stack::stack_divisor_factors;
#[doc(hidden)]
pub use stack_bend::{bend_curvature, bend_reach};
#[doc(hidden)]
pub use stack_taper::taper_floor;
pub use step::{field_shrink, gradient_bound, safe_march_step};

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "verb_tests.rs"]
mod verb_tests;

/// ⭐⭐⭐ **O DOCUMENTO COMPILADO PARA UMA REGIÃO DO MUNDO** (W56) — a ponte entre a especialização
/// do perfil e quem a vai consumir.
///
/// A [`profile::sd_profile_in_region`] especializa **um** perfil numa região do plano dele. Um
/// documento tem poses, pilhas e booleanas por cima; esta função é a que leva uma caixa do **mundo**
/// até ao plano de cada perfil e devolve a árvore inteira, com a mesma lei e uma fração das arestas.
///
/// ⚠️ **A árvore devolvida só vale DENTRO de `[lo, hi]`** — fora, a distância pode sair **maior** que
/// a verdadeira, e uma esfera-marcha que sobre-estima o passo **atravessa a peça**. Quem chama é quem
/// sabe onde a vai avaliar.
///
/// # ⛔ Onde ela DESISTE, e por quê
///
/// Quatro modificadores **remapeiam coordenadas** — `Mirror` (`x → |x|`), `Array` e `Radial`
/// (dobram o domínio) e `Taper` (escala com `y`). Debaixo de qualquer um deles, a caixa do mundo
/// **não** mapeia para uma caixa no plano do perfil: uma matriz dobra meio espaço numa célula.
/// Calcular a pré-imagem de cada um é possível e é uma wave própria; até lá o perfil por baixo deles
/// é baixado **inteiro** — correcto, só não mais rápido. *Uma especialização que erra a pré-imagem
/// não fica lenta: fura a peça.*
///
/// ⚠️ `Shell` e `Offset` **não** remapeiam (agem no valor), então não desistem.
#[must_use]
pub fn compile_in_region(doc: &FieldDoc, lo: [f32; 3], hi: [f32; 3]) -> Tree {
    RegionCompiler::new(doc).compile(doc, lo, hi)
}

/// ⚠️ Só para a prova de mutação: os oito cantos de uma caixa.
#[doc(hidden)]
#[must_use]
pub fn probe_box_corners(lo: [f32; 3], hi: [f32; 3]) -> Vec<[f32; 3]> {
    box_corners(lo, hi)
}

/// Os oito cantos de uma caixa — a região «forma real» de quem só tem a caixa.
fn box_corners(lo: [f32; 3], hi: [f32; 3]) -> Vec<[f32; 3]> {
    (0..8u8)
        .map(|k| {
            [
                if k & 1 == 0 { lo[0] } else { hi[0] },
                if k & 2 == 0 { lo[1] } else { hi[1] },
                if k & 4 == 0 { lo[2] } else { hi[2] },
            ]
        })
        .collect()
}

/// ⭐⭐ **O compilador de regiões, com os índices já construídos** (W56).
///
/// ⚠️ **Ele existe por uma medição:** construir a [`profile_index::ProfileIndex`] de um contorno de
/// 168 arestas custa **0,2 ms**, e um quadro pede uma região por ladrilho — dezenas delas.
/// Reconstruir o índice por região pagaria mais do que a especialização poupa. *Um índice é do
/// CONTORNO, não da região.*
pub struct RegionCompiler {
    /// Índice por nó, só para os nós que são forma de perfil.
    idx: std::collections::BTreeMap<usize, profile_index::ProfileIndex>,
    /// ⭐⭐⭐⭐ **A ÁRVORE DA FÓRMULA por nó, ajustada UMA vez** — pela mesma razão que o índice ao
    /// lado, e com um número dez vezes maior.
    ///
    /// ⛔⛔ Medido em 2026-09-23: ajustar o torno da cena `5` custa **`0,0748 ms`**, e o
    /// `specialised_profile` corre **por ladrilho × fatia** — `750` regiões a `1920×1080` com
    /// ladrilho `64`, `39 406` com ladrilho `8`. ⇒ **`56 ms` a `2 948 ms` por quadro só a
    /// ajustar**, contra `0,0280 ms` que montar a árvore exacta da peça inteira custa. *O A/B de
    /// CPU leu `90,17` contra `88,23 ms` porque dois números grandes se cancelavam: a marcha
    /// poupava o que a montagem gastava.*
    ///
    /// ⭐ **E a resposta da região é a MESMA árvore** — uma fórmula não depende da região —, logo o
    /// que aqui se guarda serve toda região por clonagem de um ponteiro.
    formula: std::collections::BTreeMap<usize, Tree>,
    /// ⭐ O mapa mundo→local de cada nó — do documento, e não da região, então compõe-se **uma** vez
    /// (W148). Ver [`affine::local_maps`].
    maps: Vec<Option<Affine>>,
}

impl RegionCompiler {
    /// Constrói os índices dos perfis do documento — uma vez.
    #[must_use]
    pub fn new(doc: &FieldDoc) -> Self {
        let mut idx = std::collections::BTreeMap::new();
        let mut formula = std::collections::BTreeMap::new();
        for (i, node) in doc.nodes().iter().enumerate() {
            // ⚠️ **O POLÍGONO entra aqui**, e esquecê-lo não daria erro nenhum: ele só perderia
            // a especialização por ladrilho e ficaria mais lento em silêncio. *Um `if let` não
            // fecha buraco nenhum* — o que fecha é o gate que mede o custo dele.
            if let NodeKind::Leaf(
                Primitive::Extrude { profile, .. }
                | Primitive::Polygon { profile, .. }
                | Primitive::Revolve { profile },
            ) = &node.kind
            {
                idx.insert(i, profile_index::ProfileIndex::build(profile));
                // ⭐ Só o TORNO desce por fórmula (a silhueta de um extrude é uma curva fechada,
                // não uma função da altura) — ver [`profile_formula`].
                if matches!(&node.kind, NodeKind::Leaf(Primitive::Revolve { .. }))
                    && let Some(t) = profile::torno_por_formula(profile)
                {
                    formula.insert(i, t);
                }
            }
        }
        Self {
            idx,
            formula,
            maps: affine::local_maps(doc),
        }
    }

    /// ⭐ **A árvore da fórmula deste nó**, ou `None` se ele não desce por fórmula — ver
    /// [`Self::formula`].
    #[must_use]
    pub(crate) fn formula_de(&self, i: usize) -> Option<&Tree> {
        self.formula.get(&i)
    }

    /// **Este documento tem alguma forma de perfil que a especialização ENCURTE?** — se não,
    /// especializar não compra nada, e o consumidor fica com a marcha de sempre.
    ///
    /// ⭐⭐⭐⭐ **E um torno por FÓRMULA não conta** (2026-09-23): a árvore dele não tem arestas para
    /// cortar, logo a região é a **identidade** — e ladrilhar por causa dela faz o quadro pagar a
    /// montagem por região sem poupar um passo. ⇒ uma peça feita só de tornos por fórmula corre a
    /// marcha de sempre, com **uma** árvore.
    #[must_use]
    pub fn is_worth_it(&self) -> bool {
        self.idx.keys().any(|i| !self.formula.contains_key(i))
    }

    /// A árvore do documento, especializada para a caixa de mundo `[lo, hi]`. Ver
    /// [`compile_in_region`].
    #[must_use]
    pub fn compile(&self, doc: &FieldDoc, lo: [f32; 3], hi: [f32; 3]) -> Tree {
        self.compile_at(doc, lo, hi, &box_corners(lo, hi))
    }

    /// ⭐⭐⭐ **A MESMA especialização, com a região a ser um CONJUNTO DE PONTOS** (W59).
    ///
    /// ⚠️ **A caixa continua a viajar, e não é redundância:** ela é o que o `Revolve` usa (o `(u, v)`
    /// dele é `√(x²+z²)`, e a região ali é um rectângulo por construção), o que o sinal e a âncora
    /// consomem, e o que `Affine::box_of` sabe mapear. O que os **pontos** acrescentam é a forma
    /// real, e só a **distância** de um `Extrude` a consome.
    ///
    /// ⚠️ Os pontos são os cantos do tubo da região, **crus** (sem a folga da sonda da normal): quem
    /// a soma de volta é o `hull_uv`, que a lê da caixa.
    ///
    /// ⭐ **Desde a W148 esta porta é a [`Self::compile_hulled`] com os cascos calculados na hora** — a
    /// cache de fitas guarda-os ao lado da fita, e os dois caminhos consomem a MESMA conta.
    #[must_use]
    pub fn compile_at(
        &self,
        doc: &FieldDoc,
        lo: [f32; 3],
        hi: [f32; 3],
        corners: &[[f32; 3]],
    ) -> Tree {
        self.compile_hulled(doc, lo, hi, &self.hulls(doc, lo, hi, corners))
    }
}

fn compile_in_region_with(
    rc: &RegionCompiler,
    doc: &FieldDoc,
    lo: [f32; 3],
    hi: [f32; 3],
    // ⭐⭐ Os cascos da região, por folha (W59, W148) — ver [`RegionHulls`]. O mapa mundo→local de
    // cada nó mora no compilador, porque é do documento e não da região.
    hulls: &RegionHulls,
) -> Tree {
    let n = doc.nodes().len();
    let balls = bounds::local_balls(doc, &hybrid::Registry::default());
    let mut built: Vec<Tree> = Vec::with_capacity(n);
    for (i, node) in doc.nodes().iter().enumerate() {
        let inner = match &node.kind {
            // ⚠️ **Quem é especializado decide-o UMA função** ([`RegionCompiler::specialised_leaf`]),
            // que os cascos guardados pela cache também perguntam.
            // ⭐⭐⭐⭐ **A ÁRVORE DA FÓRMULA vem do mapa e não de um ajuste novo** — ver
            // [`RegionCompiler::formula`]: ajustá-la aqui custaria `0,0748 ms` **por região**, e
            // um quadro pede centenas delas.
            NodeKind::Leaf(p) => rc
                .formula_de(i)
                .cloned()
                .or_else(|| {
                    rc.specialised_leaf(node, i).and_then(|(m, idx)| {
                        specialised_profile(p, idx, m.box_of(lo, hi), hulls.hull_of(i))
                    })
                })
                .unwrap_or_else(|| primitive(p)),
            NodeKind::Combine { op, children } => combine(*op, children, doc.nodes(), &built),
            NodeKind::Sampled { .. } => Tree::constant(f64::from(hybrid::ABSENT)),
        };
        built.push(place(
            &stacked(&inner, &node.mods, balls[i].unwrap_or(bounds::Ball::EMPTY)),
            node.xform,
        ));
    }
    built[doc.root().0 as usize].clone()
}

/// A mesma pergunta, aberta ao gate — ver [`remaps_coordinates`].
#[cfg(test)]
pub(crate) fn remaps_coordinates_for_test(m: &Unary) -> bool {
    remaps_coordinates(m)
}

/// Este modificador mexe nas **coordenadas** (e não só no valor)? Ver [`compile_in_region`].
fn remaps_coordinates(m: &Unary) -> bool {
    match m {
        Unary::Shell { .. } | Unary::Offset { .. } => false,
        Unary::Mirror { .. }
        | Unary::MirrorY { .. }
        | Unary::MirrorZ { .. }
        | Unary::Array { .. }
        | Unary::Radial { .. }
        | Unary::Taper { .. }
        // ⚠️ Os dois deformadores remapeiam o domínio: uma especialização que errasse a pré-imagem
        // **fura**.
        | Unary::Twist { .. }
        | Unary::Bend { .. } => true,
    }
}

/// A forma de perfil especializada para a caixa **local**, ou `None` se não for uma forma de perfil.
fn specialised_profile(
    p: &Primitive,
    idx: &profile_index::ProfileIndex,
    local: ([f32; 3], [f32; 3]),
    // O casco em `(u, v)` desta folha, JÁ CALCULADO — ver `RegionCompiler::hulls`. Vazio (degenerado)
    // = a distância corta pela caixa.
    hull: &[[f32; 2]],
) -> Option<Tree> {
    let (lo, hi) = local;
    match p {
        Primitive::Extrude {
            profile,
            half_height,
            round,
            chamfer,
        }
        | Primitive::Polygon {
            profile,
            half_height,
            round,
            chamfer,
        } => {
            // ⭐⭐⭐ **A região do EXTRUDE é o casco, não a caixa dele** (W59): o `(u, v)` dele é
            // `(x, y)`, então a pegada real do tubo no plano do perfil é o **polígono** dos cantos
            // projectados. ⛔ O `Revolve` fica de fora e não é esquecimento: o `u` dele é
            // `√(x² + z²)`, e a região em `(u, v)` é um **rectângulo** por construção — não há
            // polígono a apertar.
            //
            // ⚠️ **Desde a W148 o casco chega calculado**, porque a cache de fitas o guarda ao lado
            // da fita e o compara: a conta que especializa e a que decide servir têm de ser UMA.
            let flat = profile::sd_profile_in_region(
                profile,
                idx,
                &Tree::x(),
                &Tree::y(),
                [lo[0], lo[1]],
                [hi[0], hi[1]],
                false,
                (hull.len() >= 3).then_some(hull),
            );
            Some(profile::extrude_from(
                &flat,
                f64::from(*half_height),
                f64::from(*round),
                f64::from(*chamfer),
            ))
        }
        Primitive::Revolve { profile } => {
            // ⚠️ **Quem responde por um torno de FÓRMULA é o chamador**, pela árvore que o
            // [`RegionCompiler`] ajustou uma vez — ajustá-la aqui custaria `0,0748 ms` por região.
            // ⛔ E chegar aqui com um torno que desce por fórmula é o defeito que três gates
            // apanharam: o todo pela fórmula e a região a cortar o contorno DESENHADO são duas
            // leis. Ver [`profile::torno_por_formula`] e [`RegionCompiler::formula_de`].
            debug_assert!(
                profile::torno_por_formula(profile).is_none(),
                "um torno por fórmula não passa por aqui — a região dele é a árvore guardada"
            );
            // ⚠️ `u = √(x² + z²)`: a caixa local vira um **anel** em `u`, e o mínimo é a distância do
            // eixo à caixa no plano `xz` — zero quando ela o contém.
            let du = axis_gap(lo[0], hi[0]);
            let dv = axis_gap(lo[2], hi[2]);
            let u_lo = du.hypot(dv);
            let u_hi = lo[0]
                .abs()
                .max(hi[0].abs())
                .hypot(lo[2].abs().max(hi[2].abs()));
            // ⚠️ **O `u` do torno é `√(x² + z²)`, não `x`** — a árvore especializada recebe o raio,
            // como a [`profile::sd_revolve`] faz, senão ela mediria o perfil no plano errado.
            let r = ops::safe_sqrt(Tree::x().square() + Tree::z().square());
            Some(profile::sd_profile_in_region(
                profile,
                idx,
                &r,
                &Tree::y(),
                [u_lo, lo[1]],
                [u_hi, hi[1]],
                true,
                None,
            ))
        }
        _ => None,
    }
}

/// A distância do zero ao intervalo `[lo, hi]` — zero quando ele o contém.
fn axis_gap(lo: f32, hi: f32) -> f32 {
    if lo > 0.0 {
        lo
    } else if hi < 0.0 {
        -hi
    } else {
        0.0
    }
}

#[path = "primitive_tree.rs"]
mod primitive_tree;
/// ⭐ As formas por FÓRMULA — ver [`primitive_tree_formula`].
mod primitive_tree_formula;
/// ⭐ As formas cujos VÉRTICES o artista autora — ver [`primitive_tree_vertices`].
mod primitive_tree_vertices;
/// ⭐ Qual fórmula cada forma usa — ver [`primitive_tree`].
/// ⏱️⭐⭐⭐⭐ **O perfil por FÓRMULA** — ver o cabeçalho do [`profile_formula`].
#[path = "profile_formula_probe.rs"]
pub mod profile_formula;

/// ⭐⭐⭐⭐ **Os gates do torno por fórmula** — ver o cabeçalho do [`profile_formula_tests`].
#[cfg(test)]
#[path = "profile_formula_tests.rs"]
mod profile_formula_tests;
pub(crate) use primitive_tree::primitive;
