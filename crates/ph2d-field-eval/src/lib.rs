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
        Primitive::Cone {
            bottom,
            top,
            half_height,
            round,
        } => ops::sd_cone(
            f64::from(bottom),
            f64::from(top),
            f64::from(half_height),
            f64::from(round),
        ),
        Primitive::Capsule {
            radius,
            half_height,
        } => ops::sd_capsule(f64::from(radius), f64::from(half_height)),
        Primitive::Prism {
            sides,
            radius,
            half_height,
            round,
        } => ops::sd_prism(
            sides,
            f64::from(radius),
            f64::from(half_height),
            f64::from(round),
        ),
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
            // ⭐ Os outros dois eixos, pela MESMA lei — ver [`ph2d_field::Unary::MirrorZ`] para a
            // cerca que caiu.
            Unary::MirrorY => acc.remap_xyz(Tree::x(), Tree::y().abs(), Tree::z()),
            Unary::MirrorZ => acc.remap_xyz(Tree::x(), Tree::y(), Tree::z().abs()),
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
pub use step::{inflation_depth, safe_march_step};

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
        built.push(place(&stacked(&inner, &node.mods), node.xform));
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
        | Unary::Taper { .. } => true,
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
