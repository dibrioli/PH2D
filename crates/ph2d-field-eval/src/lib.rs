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
pub mod profile;
/// ⭐⭐ O perfil como CONSULTA (W56) — a cura do custo linear nas arestas.
pub mod profile_index;

use fidget::context::Tree;
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
        built.push(place(&stacked(&inner, &node.mods), node.xform));
    }
    built[doc.root().0 as usize].clone()
}

pub(crate) fn primitive(p: &Primitive) -> Tree {
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
pub(crate) fn stacked(inner: &Tree, mods: &[Unary]) -> Tree {
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
            Unary::Radial { count } => radial(&acc, count),
            Unary::Taper { slope } => taper(&acc, f64::from(slope)),
        };
    }
    acc
}

/// ⭐ **A inclinação (draft/taper)** — e o **primeiro operador deste módulo que não é exato**.
///
/// A secção transversal escala por `k(y) = 1 + slope·y`: o ponto vai para o espaço não-inclinado
/// (`x/k`, `y`, `z/k`) e o valor volta multiplicado por `k` — a mesma receita de duas metades que a
/// [`place`] usa para a escala uniforme, e pela mesma razão (sem a segunda metade o campo deixa de
/// ser uma distância).
///
/// # ⚠️ Por que ele não pode ser exato, e o que se paga em vez disso
///
/// A escala **varia com `y`**, e é essa variação que estraga: `∇g` ganha um termo de ordem
/// `slope·f` que a multiplicação por `k` não cancela. Perto da superfície (`f ≈ 0`) o erro
/// desaparece — que é onde a marcha mais precisa dele —, mas longe ele **superestima**, e
/// superestimar é o erro que faz o raio saltar por cima da peça.
///
/// A cura é dividir por `1 + |slope|`, o que torna o campo um **bound conservador**: ele nunca
/// passa da distância verdadeira, e a marcha continua correta. O preço é o número de passos, e ele
/// está medido em `measure_taper_cost` — é dali que sai o
/// [`ph2d_field::mods::MAX_TAPER_SLOPE`].
///
/// ⚠️ **O piso em `k` impede a inversão.** Em `y = −1/slope` a secção colapsa e, passando disso,
/// ela **vira do avesso** — a peça sairia com o interior para fora. Preso a [`TAPER_FLOOR`], o que
/// acontece além do ápice é a secção ficar congelada nele, que é uma forma e não um defeito.
fn taper(inner: &Tree, slope: f64) -> Tree {
    if slope == 0.0 || !slope.is_finite() {
        return inner.clone();
    }
    let k = (Tree::constant(1.0) + Tree::y() * Tree::constant(slope)).max(TAPER_FLOOR);
    let shrunk = inner.remap_xyz(Tree::x() / k.clone(), Tree::y(), Tree::z() / k.clone());
    shrunk * k / Tree::constant(1.0 + TAPER_SAFETY * slope.abs())
}

/// O menor fator de secção que a inclinação admite — ver [`taper`].
///
/// ⚠️ Não é um épsilon de gosto: abaixo dele o `x/k` explode e o campo passa a devolver números que
/// a marcha lê como "muito longe" dentro da própria peça. Um centésimo é duas ordens de grandeza
/// abaixo da secção nominal, o que põe o ápice bem fora de qualquer peça enquadrada.
const TAPER_FLOOR: f64 = 0.01;

/// Quanto o divisor da inclinação cresce por unidade de declive — **medido, e a primeira tentativa
/// estava errada**.
///
/// ⚠️ A conta que eu escrevi primeiro dividia por `1 + |slope|`, derivada à mão. A sonda
/// `measure_taper_cost` **refutou-a**: `‖∇f‖` continuava acima de 1 em todo o alcance, ou seja o
/// campo **superestimava** — exatamente a falha que a divisão existe para evitar.
///
/// | declive | `‖∇f‖` máx com `1 + s` | com `1 + 2s` |
/// |---|---|---|
/// | 0,25 | **1,12** ⛔ | 0,93 ✅ |
/// | 0,50 | **1,20** ⛔ | 0,90 ✅ |
/// | 1,00 | **1,30** ⛔ | 0,87 ✅ |
/// | 2,00 | **1,40** ⛔ | 0,84 ✅ |
///
/// *Uma derivação à mão é uma hipótese; a tabela é o facto.* O `2` é o degrau que a medição deu —
/// com ele `‖∇f‖ ≤ 1` em todo o alcance, que é a condição de a marcha não atravessar a peça.
const TAPER_SAFETY: f64 = 2.0;

/// ⭐ **A matriz radial**: `count` cópias em coroa, em torno do **Z**.
///
/// A conta é a mesma ideia da linear numa coordenada diferente: em vez de dobrar o `x`, dobra-se o
/// **ângulo**. Leva-se o ponto para a fatia dele (`θ − Δ·k`, com `Δ = 2π/count`) e avalia-se **uma**
/// forma — uma coroa de 32 custa o mesmo que uma de 2.
///
/// ⚠️ **Duas fatias**, pelo mesmíssimo motivo da linear: com uma só, uma forma que transborde a
/// fatia faz o campo **superestimar**, e superestimar é o que faz a marcha de raios saltar por cima
/// da superfície. Ver [`array`], onde o mecanismo está escrito por extenso.
///
/// ⚠️ **No eixo (`x = y = 0`) não há ângulo**, e é por isso que a conta não divide por `r`: ela
/// reconstrói o ponto por `r·cos θ'` / `r·sin θ'`, e em `r = 0` isso é a origem — a resposta certa,
/// sem caso especial e sem `NaN`.
fn radial(inner: &Tree, count: u32) -> Tree {
    if count <= 1 {
        return inner.clone();
    }
    let step = std::f64::consts::TAU / f64::from(count);
    let d = Tree::constant(step);
    let r = crate::ops::safe_sqrt(Tree::x().square() + Tree::y().square());
    let theta = Tree::y().atan2(Tree::x());
    let raw = (theta.clone() / d.clone()).round();
    // A fatia vizinha é a do lado para onde o ponto pende — mesma lei da linear.
    let toward = theta.clone() / d.clone() - raw.clone();
    let other = raw.clone() + toward.compare(Tree::constant(0.0));
    let wedge = |k: Tree| {
        let t = theta.clone() - d.clone() * k;
        inner.remap_xyz(r.clone() * t.clone().cos(), r.clone() * t.sin(), Tree::z())
    };
    wedge(raw).min(wedge(other))
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
    let trees: Vec<Tree> = children
        .iter()
        .map(|c| built[c.0 as usize].clone())
        .collect();
    combine_trees(op, &trees)
}

/// A mesma combinação, já sobre as árvores — a porta que o avaliador híbrido partilha.
pub(crate) fn combine_trees(op: Op, trees: &[Tree]) -> Tree {
    let b = blended(op.blend());
    let mut acc = trees[0].clone();
    for rhs in &trees[1..] {
        let rhs = rhs.clone();
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

#[cfg(test)]
mod tests;
