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

pub mod ops;
pub mod profile;

use fidget::context::Tree;
use fidget::mesh::{Octree, Settings};
use ph2d_field::{Blend, FieldDoc, Node, NodeKind, Op, Primitive, Unary, Xform};

/// O motor de avaliação. Ver a nota do `Cargo.toml` sobre o `jit` estar ligado por medição.
pub type Engine = fidget::jit::JitShape;

/// Compila o documento numa árvore de avaliação.
#[must_use]
pub fn compile(doc: &FieldDoc) -> Tree {
    let mut built: Vec<Tree> = Vec::with_capacity(doc.nodes().len());
    for node in doc.nodes() {
        // Seguro pela invariante da arena: todo filho já foi construído.
        let inner = match &node.kind {
            NodeKind::Leaf(p) => primitive(p),
            NodeKind::Combine { op, children } => combine(*op, children, &built),
        };
        // ⭐ **A pilha corre entre o que o nó É e onde ele ESTÁ**, e a ordem das duas metades é a
        // lei: em local, a espessura de uma casca é um número do nó, e a pose de um ancestral
        // escala-a junto com tudo o mais do nó — exatamente como a largura de uma caixa. Aplicá-la
        // depois de `place` faria o único número deste módulo que **não** obedece à cadeia.
        built.push(place(&stacked(&inner, &node.mods), node.xform));
    }
    built[doc.root().0 as usize].clone()
}

fn primitive(p: &Primitive) -> Tree {
    match *p {
        Primitive::Box { half, round } => ops::sd_box(
            [f64::from(half[0]), f64::from(half[1]), f64::from(half[2])],
            f64::from(round),
        ),
        Primitive::Sphere { radius } => ops::sd_sphere(f64::from(radius)),
        Primitive::Cylinder {
            radius,
            half_height,
            round,
        } => ops::sd_cylinder(f64::from(radius), f64::from(half_height), f64::from(round)),
        Primitive::Torus { major, minor } => ops::sd_torus(f64::from(major), f64::from(minor)),
        Primitive::Extrude {
            ref profile,
            half_height,
            round,
        } => profile::sd_extrude(profile, f64::from(half_height), f64::from(round)),
        Primitive::Revolve { ref profile } => profile::sd_revolve(profile),
    }
}

/// ⭐ **A pilha de modificadores de um nó**, aplicada na ordem em que ela está.
///
/// ⚠️ **A ordem importa e é por isso que ela é uma lista**: encascar-e-afastar não é afastar-e-
/// encascar. `|f| − t` seguido de `− d` dá uma parede mais grossa; `f − d` seguido de `| | − t` dá
/// uma parede da mesma espessura noutro sítio. Um conjunto sem ordem teria de escolher uma em
/// silêncio.
fn stacked(inner: &Tree, mods: &[Unary]) -> Tree {
    let mut acc = inner.clone();
    for m in mods {
        acc = match *m {
            // ⭐ A casca inteira: o módulo de uma distância É a distância à mesma superfície vista
            // dos dois lados, e afastá-la meia espessura para cada lado dá a parede.
            Unary::Shell { thickness } => ops::offset(&acc.abs(), f64::from(thickness) * 0.5),
            Unary::Offset { distance } => ops::offset(&acc, f64::from(distance)),
            // ⭐ **Dobra do domínio**: `x → |x|`. O que existe de um lado passa a existir dos dois, e
            // o campo continua uma distância exata — não há costura a fechar, que é o mesmo motivo
            // de a booleana e a casca não poderem falhar.
            Unary::Mirror => acc.remap_xyz(Tree::x().abs(), Tree::y(), Tree::z()),
            Unary::Array { count, spacing } => array(&acc, count, f64::from(spacing)),
        };
    }
    acc
}

/// ⭐ **A matriz linear**: `count` cópias espaçadas de `spacing` no X, **sem N cópias da árvore**.
///
/// A conta é a dobra do domínio: leva-se o ponto para a célula dele (`x − s·k`, com `k` o índice da
/// célula preso a `[0, count−1]`) e avalia-se **uma** forma. É a razão de uma matriz de 64 custar o
/// mesmo que uma de 2 — numa malha ela custaria 64 vezes a geometria.
///
/// # ⚠️ Por que DUAS células, e não uma
///
/// A receita clássica (`opRepLim`) olha só a célula do ponto, e ela **superestima** a distância
/// quando a forma transborda a célula: existe uma cópia vizinha mais perto do que a da célula, e o
/// campo não a vê. Superestimar é o erro **caro** numa marcha de raios — o passo salta por cima da
/// superfície, e o sintoma é a peça com buracos, não um erro.
///
/// Olhar a célula do ponto **e a vizinha do lado para onde ele pende** custa duas avaliações da
/// subárvore e devolve a distância exata enquanto a forma couber em **1,5 células**. ⛔ Acima disso
/// o bound volta, e a cura é olhar três — que é o dobro do custo por um caso que o nascimento da
/// matriz (espaçamento = 2× a peça) já põe fora de alcance.
fn array(inner: &Tree, count: u32, spacing: f64) -> Tree {
    if count <= 1 || spacing <= 0.0 || !spacing.is_finite() {
        return inner.clone();
    }
    let s = Tree::constant(spacing);
    let last = f64::from(count - 1);
    // O índice da célula, preso à matriz: `clamp(round(x/s), 0, count−1)`.
    let raw = (Tree::x() / s.clone()).round();
    let k = raw.max(Tree::constant(0.0)).min(Tree::constant(last));
    // ⚠️ **A vizinha é a do lado para onde o ponto PENDE**, e não uma fixa: com o sinal errado a
    // segunda avaliação cai na mesma célula metade das vezes e o gate passaria sem nada a defender.
    let toward = Tree::x() / s.clone() - k.clone();
    let neighbour = (k.clone() + toward.compare(Tree::constant(0.0)))
        .max(Tree::constant(0.0))
        .min(Tree::constant(last));
    let cell = |idx: Tree| inner.remap_xyz(Tree::x() - s.clone() * idx, Tree::y(), Tree::z());
    cell(k).min(cell(neighbour))
}

fn blended(b: Blend) -> ops::Blended {
    match b {
        Blend::Sharp => ops::Blended::Sharp,
        Blend::Exact { radius } => ops::Blended::Exact(f64::from(radius)),
        Blend::Organic { k } => ops::Blended::Organic(f64::from(k)),
    }
}

fn combine(op: Op, children: &[ph2d_field::NodeId], built: &[Tree]) -> Tree {
    let child = |i: usize| built[children[i].0 as usize].clone();
    let b = blended(op.blend());
    let mut acc = child(0);
    for i in 1..children.len() {
        let rhs = child(i);
        acc = match op {
            Op::Union(_) => ops::union(&acc, &rhs, b),
            Op::Intersection(_) => ops::intersection(&acc, &rhs, b),
            // `children[0]` menos todos os seguintes.
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
fn place(inner: &Tree, x: Xform) -> Tree {
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
fn inverse_rotation_matrix(q: [f32; 4]) -> [[f64; 3]; 3] {
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
#[derive(Debug)]
pub enum MeshError {
    /// A malhagem foi cancelada.
    Cancelled,
    /// A malha saiu, mas a validação da `ph2d-mesh` a recusou.
    Rejected(String),
}

/// Extrai a malha do documento, na profundidade de octree pedida.
///
/// ⚠️ **A malha é o artefato de EXPORTAÇÃO, não o que o artista vê** (ADR-0161 §2). O extrator
/// atual **serrilha aresta viva** — medido, com o mecanismo nomeado
/// (`docs/3DModeling/01_resultados_spike.md` §2). Isso é aceitável para exportar e **não** para
/// desenhar a tela, e é por isso que a tela não passa por aqui.
///
/// # Errors
/// Ver [`MeshError`].
pub fn mesh(doc: &FieldDoc, depth: u8) -> Result<ph2d_mesh::Mesh, MeshError> {
    let tree = compile(doc);
    let shape = Engine::from(tree);
    let bound = shape
        .try_into()
        .map_err(|_| MeshError::Rejected("a árvore tem variáveis livres".into()))?;
    let settings = Settings {
        depth,
        ..Default::default()
    };
    let octree = Octree::build(&bound, &settings).ok_or(MeshError::Cancelled)?;
    let out = octree.walk_dual();

    let positions: Vec<[f32; 3]> = out.vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
    let faces: Vec<ph2d_mesh::Face> = out
        .triangles
        .iter()
        .map(|t| ph2d_mesh::Face::tri(t.x as u32, t.y as u32, t.z as u32))
        .collect();
    ph2d_mesh::Mesh::from_parts(positions, faces).map_err(|e| MeshError::Rejected(format!("{e:?}")))
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

#[cfg(test)]
mod tests;
