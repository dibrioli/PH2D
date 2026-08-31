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
pub mod extract;
pub mod hybrid;
pub mod ops;
pub mod ops_bool;
pub mod ops_box;
pub mod ops_joint;
pub(crate) mod ops_norm;
pub mod ops_plates;
pub mod ops_solids;
pub mod profile;
/// ⭐⭐ O perfil como CONSULTA (W56) — a cura do custo linear nas arestas.
pub mod profile_index;
/// ⭐ A pilha de modificadores de um nó — ver [`stack`].
pub(crate) mod stack;

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

/// Um documento avaliado ponto a ponto — a porta que o traçado e as sondas usam.
pub struct Field {
    ctx: fidget::context::Context,
    root: fidget::context::Node,
}

impl Field {
    #[must_use]
    pub fn new(doc: &FieldDoc) -> Self {
        Self::from_tree(&compile(doc))
    }

    #[must_use]
    pub fn from_tree(tree: &Tree) -> Self {
        let mut ctx = fidget::context::Context::new();
        let root = ctx.import(tree);
        Self { ctx, root }
    }

    /// `f(x, y, z)`. `NaN` se a árvore não puder ser avaliada ali.
    #[must_use]
    pub fn at(&self, x: f64, y: f64, z: f64) -> f64 {
        self.ctx.eval_xyz(self.root, x, y, z).unwrap_or(f64::NAN)
    }

    /// `‖∇f‖` por diferença central — a medida de quanto o campo ainda é uma **distância**.
    #[must_use]
    pub fn gradient_norm(&self, x: f64, y: f64, z: f64, eps: f64) -> f64 {
        let gx = (self.at(x + eps, y, z) - self.at(x - eps, y, z)) / (2.0 * eps);
        let gy = (self.at(x, y + eps, z) - self.at(x, y - eps, z)) / (2.0 * eps);
        let gz = (self.at(x, y, z + eps) - self.at(x, y, z - eps)) / (2.0 * eps);
        (gx * gx + gy * gy + gz * gz).sqrt()
    }
}

/// Atalho de leitura para quem monta um documento à mão.
#[must_use]
pub fn leaf(p: Primitive, xform: Xform) -> Node {
    Node::new(xform, NodeKind::Leaf(p))
}

mod affine;
use affine::Affine;
mod hull;
use hull::hull_uv;
pub use hull::{probe_hull_uv, probe_in_hull};

#[path = "step.rs"]
mod step;
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
}

impl RegionCompiler {
    /// Constrói os índices dos perfis do documento — uma vez.
    #[must_use]
    pub fn new(doc: &FieldDoc) -> Self {
        let mut idx = std::collections::BTreeMap::new();
        for (i, node) in doc.nodes().iter().enumerate() {
            if let NodeKind::Leaf(
                Primitive::Extrude { profile, .. } | Primitive::Revolve { profile },
            ) = &node.kind
            {
                idx.insert(i, profile_index::ProfileIndex::build(profile));
            }
        }
        Self { idx }
    }

    /// **Este documento tem alguma forma de perfil?** — se não, especializar não compra nada, e o
    /// consumidor fica com a marcha de sempre.
    #[must_use]
    pub fn is_worth_it(&self) -> bool {
        !self.idx.is_empty()
    }

    /// A árvore do documento, especializada para a caixa de mundo `[lo, hi]`. Ver
    /// [`compile_in_region`].
    #[must_use]
    pub fn compile(&self, doc: &FieldDoc, lo: [f32; 3], hi: [f32; 3]) -> Tree {
        compile_in_region_with(self, doc, lo, hi, &box_corners(lo, hi))
    }

    /// ⭐⭐⭐ **A MESMA especialização, com a região a ser um CONJUNTO DE PONTOS** (W59).
    ///
    /// ⚠️ **A caixa continua a viajar, e não é redundância:** ela é o que o `Revolve` usa (o `(u, v)`
    /// dele é `√(x²+z²)`, e a região ali é um rectângulo por construção), o que o sinal e a âncora
    /// consomem, e o que `Affine::box_of` sabe mapear. O que os **pontos** acrescentam é a forma
    /// real, e só a **distância** de um `Extrude` a consome.
    ///
    /// ⚠️ Os pontos são os cantos do tubo da região, **crus** (sem a folga da sonda da normal): quem
    /// a soma de volta é [`hull_uv`], que a lê da caixa.
    #[must_use]
    pub fn compile_at(
        &self,
        doc: &FieldDoc,
        lo: [f32; 3],
        hi: [f32; 3],
        corners: &[[f32; 3]],
    ) -> Tree {
        compile_in_region_with(self, doc, lo, hi, corners)
    }
}

fn compile_in_region_with(
    rc: &RegionCompiler,
    doc: &FieldDoc,
    lo: [f32; 3],
    hi: [f32; 3],
    corners: &[[f32; 3]],
) -> Tree {
    // Passo 1 — o mapa mundo→local de cada nó. A arena tem os filhos ANTES dos pais, então o
    // percurso é de cima para baixo a partir da raiz.
    let n = doc.nodes().len();
    let mut to_local = vec![None::<Affine>; n];
    let root = doc.root().0 as usize;
    to_local[root] = Some(Affine::of(doc.nodes()[root].xform));
    // Da raiz para trás: um filho tem índice menor que o pai, logo descer por índices decrescentes
    // visita todo pai antes dos filhos dele.
    for i in (0..n).rev() {
        let Some(parent) = to_local[i] else {
            continue;
        };
        if let NodeKind::Combine { children, .. } = &doc.nodes()[i].kind {
            for c in children {
                let ci = c.0 as usize;
                to_local[ci] = Some(Affine::of(doc.nodes()[ci].xform).after(parent));
            }
        }
    }

    let balls = bounds::local_balls(doc, &hybrid::Registry::default());
    let mut built: Vec<Tree> = Vec::with_capacity(n);
    for (i, node) in doc.nodes().iter().enumerate() {
        let inner = match &node.kind {
            NodeKind::Leaf(p) => to_local[i]
                .filter(|_| !node.mods.iter().any(remaps_coordinates))
                .zip(rc.idx.get(&i))
                .and_then(|(m, idx)| {
                    // ⭐⭐ **Os CANTOS da região, mapeados** (W59) — o casco em `(u, v)` sai deles, e
                    // não da caixa. Um mapa afim leva canto a canto, então os oito bastam.
                    let pts = m.points_of(corners);
                    specialised_profile(p, idx, m.box_of(lo, hi), &pts)
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
    built[root].clone()
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
        Unary::Mirror
        | Unary::MirrorY
        | Unary::MirrorZ
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
    // Os cantos da região, já em espaço LOCAL — ver `hull_uv`.
    pts: &[[f32; 3]],
) -> Option<Tree> {
    let (lo, hi) = local;
    match p {
        Primitive::Extrude {
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
            let hull = hull_uv(pts, [lo[0], lo[1]], [hi[0], hi[1]]);
            let flat = profile::sd_profile_in_region(
                profile,
                idx,
                &Tree::x(),
                &Tree::y(),
                [lo[0], lo[1]],
                [hi[0], hi[1]],
                false,
                (hull.len() >= 3).then_some(&hull[..]),
            );
            Some(profile::extrude_from(
                &flat,
                f64::from(*half_height),
                f64::from(*round),
                f64::from(*chamfer),
            ))
        }
        Primitive::Revolve { profile } => {
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

/// ⭐ Qual fórmula cada forma usa — ver [`primitive_tree`].
#[path = "primitive_tree.rs"]
mod primitive_tree;
pub(crate) use primitive_tree::primitive;
