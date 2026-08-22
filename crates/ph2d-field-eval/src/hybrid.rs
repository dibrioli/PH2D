//! **O avaliador HÍBRIDO** — uma folha do documento pode ser uma árvore analítica *ou* um campo
//! **amostrado**, e a booleana entre as duas continua a ser `min`/`max`.
//!
//! # ⚠️ Por que ele existe: a álgebra da `fidget` é FECHADA
//!
//! `TreeOp` é `Input(Var)` · `Const` · `Binary` · `Unary` · remapeamentos. **Não há consulta a
//! dados.** Uma escultura, que só existe como grade de voxels, não é exprimível ali — e o caminho
//! que restaria (um termo de árvore por triângulo) custa ~10 nós por triângulo.
//!
//! ⭐ **O número que autoriza este desenho está medido**: uma amostra trilinear custa **1,39×** uma
//! avaliação da árvore com JIT (7,6 ms contra 5,5 ms por milhão de pontos). Misturar uma folha
//! amostrada custa aproximadamente o mesmo que uma folha analítica a mais.
//!
//! # ⭐ O documento SEM folha amostrada continua a ser uma árvore só
//!
//! A compilação funde: um `Combine` cujos filhos são **todos** analíticos volta a ser analítico, com
//! a mesma expressão de sempre. Logo um documento sem escultura produz **uma** fita de JIT e o
//! caminho rápido não muda um bit — é o gate `an_all_analytic_document_stays_one_tape` que o prende.
//!
//! # ⚠️ Duas implementações da MESMA lei, e o gate que as segura
//!
//! As booleanas existem duas vezes: como árvore ([`crate::ops`]) e como aritmética `f32` aqui. Não
//! há como fugir — um `min` entre uma fita de JIT e uma grade de voxels tem de acontecer fora das
//! duas. O que torna isso seguro em vez de fatal é o gate de **paridade**
//! (`the_numeric_law_is_the_same_law_as_the_tree`): as duas formas avaliadas no mesmo documento
//! analítico, ponto a ponto. *Dois motores, uma lei — e a lei tem um juiz.*

use std::sync::Arc;

use fidget::context::Tree;
use fidget::shape::EzShape;
use ph2d_field::{Blend, FieldDoc, NodeKind, Op};

use crate::{Engine, MeshError};

/// **Uma folha que se AMOSTRA em vez de se avaliar** — o que a `ph2d-field-mesh` implementa.
///
/// ⚠️ **É um trait, e não o tipo concreto, por causa da direção da dependência.** O campo amostrado
/// nasce de uma malha, e a malha traz consigo a `ph2d-sdf`, que é do módulo de escultura. Se esta
/// crate conhecesse aquele tipo, o avaliador do campo implícito passaria a depender da escultura — e
/// a promessa de que apagar a escultura não apaga mais nada morreria no mesmo commit.
pub trait Sampled: Send + Sync {
    /// A distância com sinal em `p`, em coordenadas **do campo** (a pose do nó já foi desfeita).
    fn at(&self, p: [f32; 3]) -> f32;

    /// ⭐ **O raio de uma esfera na origem que contém a escultura inteira** (W33).
    ///
    /// ⚠️ **Método exigido, sem `default`**, e é de propósito: um default devolveria um número que
    /// **cabe compilar e não cabe a peça**, e o sintoma seria a escultura sair cortada da
    /// exportação sem uma palavra. Quem tem uma grade sabe a caixa dela; quem não souber tem de
    /// dizê-lo à mão.
    fn bounding_radius(&self) -> f32;
}

/// Nome → campo amostrado.
///
/// ⚠️ **`BTreeMap` e não `HashMap`**: a ordem de iteração entra na compilação, e o determinismo
/// (HR-5) é o que faz o `replay-hash` do CI significar alguma coisa.
pub type Registry = std::collections::BTreeMap<String, Arc<dyn Sampled>>;

/// A distância que uma folha amostrada **ausente** devolve.
///
/// ⚠️ **Ausente é VAZIO, e não sólido.** Um nome que o registo não conhece (um projeto carregado
/// antes de a escultura ser regenerada) tem de ler como espaço vazio: numa união some, numa
/// subtração não corta nada. O oposto — ler como sólido — encheria a cena de um bloco que ninguém
/// autorizou, e o artista não teria como o apagar.
pub(crate) const ABSENT: f32 = 1.0e9;

/// O plano de avaliação: uma árvore de operações sobre folhas já resolvidas.
#[derive(Clone)]
enum Plan {
    /// Índice numa fita de JIT.
    Analytic(usize),
    /// Índice num campo amostrado, com a pose do nó já invertida.
    Sampled(usize),
    Combine(Op, Vec<Plan>),
}

#[derive(Clone)]
struct SampledLeaf {
    field: Arc<dyn Sampled>,
    /// `p' = R⁻¹(p − t)/s` — a mesma conta do [`crate::place`], em números.
    inv_rot: [[f64; 3]; 3],
    translation: [f32; 3],
    scale: f32,
}

impl SampledLeaf {
    fn at(&self, p: [f32; 3]) -> f32 {
        let q = [
            (p[0] - self.translation[0]) / self.scale,
            (p[1] - self.translation[1]) / self.scale,
            (p[2] - self.translation[2]) / self.scale,
        ];
        let l = |r: usize| {
            (f64::from(q[0]).mul_add(
                self.inv_rot[r][0],
                f64::from(q[1]).mul_add(self.inv_rot[r][1], f64::from(q[2]) * self.inv_rot[r][2]),
            )) as f32
        };
        // ⚠️ **A segunda metade da pose**: o valor volta multiplicado pela escala. Sem ela o campo
        // deixa de ser uma distância assim que houver escala, e todo raio de filete mente.
        self.field.at([l(0), l(1), l(2)]) * self.scale
    }
}

type GradTapes = (
    fidget::shape::ShapeBulkEval<
        <fidget::jit::JitFunction as fidget::eval::Function>::GradSliceEval,
    >,
    fidget::shape::ShapeTape<
        <<fidget::jit::JitFunction as fidget::eval::Function>::GradSliceEval as
            fidget::eval::BulkEvaluator>::Tape,
    >,
);

type Tapes = (
    fidget::shape::ShapeBulkEval<
        <fidget::jit::JitFunction as fidget::eval::Function>::FloatSliceEval,
    >,
    fidget::shape::ShapeTape<
        <<fidget::jit::JitFunction as fidget::eval::Function>::FloatSliceEval as
            fidget::eval::BulkEvaluator>::Tape,
    >,
);

/// ⭐ **O documento pronto a avaliar em lote** — a porta única do traçado e da extração.
pub struct Hybrid {
    plan: Plan,
    /// As árvores por trás das fitas — guardadas para o [`Hybrid::fork`].
    trees: Vec<Tree>,
    tapes: Vec<Tapes>,
    sampled: Vec<SampledLeaf>,
    /// ⭐ **A fita de GRADIENTE, só quando o documento é uma árvore só.**
    ///
    /// ⚠️ Um campo amostrado **não tem gradiente analítico** — ele é uma grade. Quando há escultura,
    /// a normal sai por diferença central, e isso tem um preço nomeado: numa **quina viva** a
    /// diferença central devolve a média dos dois lados em vez de um deles, e o QEF deixa de a
    /// prender. É a razão de o caminho exato não ser abandonado quando ele existe.
    grad: Option<GradTapes>,
    /// Um buffer por folha, reaproveitado entre lotes.
    leaves: Vec<Vec<f32>>,
    out: Vec<f32>,
}

impl Hybrid {
    /// Compila o documento contra o registo de campos amostrados.
    #[must_use]
    pub fn new(doc: &FieldDoc, reg: &Registry) -> Self {
        let mut b = Builder {
            reg,
            trees: Vec::new(),
            sampled: Vec::new(),
        };
        let expr = b.build(doc);
        Self::from_parts(expr, b.trees, b.sampled)
    }

    /// ⭐ **Um avaliador NOVO sobre o mesmo plano** — o que uma marcha paralela precisa.
    ///
    /// ⚠️ **O avaliador da `fidget` tem estado mutável**, então partilhá-lo entre threads exigiria
    /// trava; a marcha já resolvia isso construindo um por lote. Aqui o que se copia é a **árvore**
    /// (um `Arc` por dentro) e o `Arc` de cada escultura: o que custa é a fita, e ela custa o mesmo
    /// que a marcha já pagava.
    #[must_use]
    pub fn fork(&self) -> Self {
        Self::from_parts(self.plan.clone(), self.trees.clone(), self.sampled.clone())
    }

    fn from_parts(plan: Plan, trees: Vec<Tree>, sampled: Vec<SampledLeaf>) -> Self {
        let mut tapes = Vec::with_capacity(trees.len());
        for t in &trees {
            let shape = Engine::from(t.clone());
            tapes.push((Engine::new_float_slice_eval(), shape.ez_float_slice_tape()));
        }
        // Só o caso puro — uma árvore, nenhuma escultura — tem gradiente exato.
        let grad = (sampled.is_empty() && trees.len() == 1).then(|| {
            let shape = Engine::from(trees[0].clone());
            (Engine::new_grad_slice_eval(), shape.ez_grad_slice_tape())
        });
        let n = tapes.len() + sampled.len();
        Self {
            plan,
            trees,
            tapes,
            sampled,
            grad,
            leaves: vec![Vec::new(); n],
            out: Vec::new(),
        }
    }

    /// ⭐ **O gradiente em cada ponto** — exato quando o documento é uma árvore só, por diferença
    /// central quando há escultura.
    ///
    /// `eps` é o passo da diferença; quem chama sabe a escala da grade e ninguém mais sabe.
    ///
    /// # Errors
    /// Ver [`Hybrid::eval`].
    pub fn gradients(
        &mut self,
        xs: &[f32],
        ys: &[f32],
        zs: &[f32],
        eps: f32,
        out: &mut Vec<[f32; 3]>,
    ) -> Result<(), MeshError> {
        out.clear();
        if let Some((eval, tape)) = &mut self.grad {
            let (gx, gy, gz) = dual_inputs(xs, ys, zs);
            let g = eval
                .eval(tape, &gx, &gy, &gz)
                .map_err(|e| MeshError::Rejected(format!("gradiente em lote: {e}")))?;
            out.extend(g.iter().map(|g| [g.dx, g.dy, g.dz]));
            return Ok(());
        }
        out.resize(xs.len(), [0.0; 3]);
        let mut shifted = vec![0.0f32; xs.len()];
        for axis in 0..3 {
            let mut both = [Vec::new(), Vec::new()];
            for (side, delta) in [(0usize, -eps), (1, eps)] {
                let src = match axis {
                    0 => xs,
                    1 => ys,
                    _ => zs,
                };
                shifted.clear();
                shifted.extend(src.iter().map(|v| v + delta));
                let v = match axis {
                    0 => self.eval(&shifted, ys, zs),
                    1 => self.eval(xs, &shifted, zs),
                    _ => self.eval(xs, ys, &shifted),
                }?;
                both[side] = v.to_vec();
            }
            for (i, slot) in out.iter_mut().enumerate() {
                slot[axis] = (both[1][i] - both[0][i]) / (2.0 * eps);
            }
        }
        Ok(())
    }

    /// Quantas fitas de JIT o documento produziu. ⭐ **Um documento sem escultura tem de dar 1** —
    /// é o que prova que a fusão funcionou e que o caminho rápido não regrediu.
    #[must_use]
    pub fn tape_count(&self) -> usize {
        self.tapes.len()
    }

    /// Quantas folhas amostradas o documento tem.
    #[must_use]
    pub fn sampled_count(&self) -> usize {
        self.sampled.len()
    }

    /// `f` em cada ponto. As três fatias têm de ter o mesmo comprimento.
    ///
    /// # Errors
    /// Se o avaliador recusar o lote (comprimentos diferentes, ou variável livre na árvore).
    pub fn eval(&mut self, xs: &[f32], ys: &[f32], zs: &[f32]) -> Result<&[f32], MeshError> {
        // — Passo 1: todas as folhas, cada uma no seu caminho.
        for (i, (eval, tape)) in self.tapes.iter_mut().enumerate() {
            let v = eval
                .eval(tape, xs, ys, zs)
                .map_err(|e| MeshError::Rejected(format!("avaliação em lote: {e}")))?;
            self.leaves[i].clear();
            self.leaves[i].extend_from_slice(v);
        }
        let base = self.tapes.len();
        for (i, leaf) in self.sampled.iter().enumerate() {
            let buf = &mut self.leaves[base + i];
            buf.clear();
            buf.extend(
                xs.iter()
                    .zip(ys)
                    .zip(zs)
                    .map(|((x, y), z)| leaf.at([*x, *y, *z])),
            );
        }
        // — Passo 2: as operações, já sobre números.
        self.out.clear();
        self.out.resize(xs.len(), 0.0);
        reduce(&self.plan, base, &self.leaves, &mut self.out);
        Ok(&self.out)
    }
}

/// A entrada **dual** do avaliador de gradiente: cada eixo entra com a sua derivada a 1.
///
/// ⚠️ Passar `f32` cru compila (há `From<f32>`) e devolve gradiente **zero** — o modo de falha
/// silencioso deste avaliador.
fn dual_inputs(
    xs: &[f32],
    ys: &[f32],
    zs: &[f32],
) -> (
    Vec<fidget::types::Grad>,
    Vec<fidget::types::Grad>,
    Vec<fidget::types::Grad>,
) {
    use fidget::types::Grad;
    (
        xs.iter().map(|v| Grad::new(*v, 1.0, 0.0, 0.0)).collect(),
        ys.iter().map(|v| Grad::new(*v, 0.0, 1.0, 0.0)).collect(),
        zs.iter().map(|v| Grad::new(*v, 0.0, 0.0, 1.0)).collect(),
    )
}

/// A avaliação das operações, ponto a ponto, sobre as folhas já resolvidas.
fn reduce(plan: &Plan, base: usize, leaves: &[Vec<f32>], out: &mut [f32]) {
    match plan {
        Plan::Analytic(i) => out.copy_from_slice(&leaves[*i]),
        Plan::Sampled(i) => out.copy_from_slice(&leaves[base + *i]),
        Plan::Combine(op, kids) => {
            reduce(&kids[0], base, leaves, out);
            let mut rhs = vec![0.0f32; out.len()];
            for k in &kids[1..] {
                reduce(k, base, leaves, &mut rhs);
                for (a, b) in out.iter_mut().zip(&rhs) {
                    *a = apply(*op, *a, *b);
                }
            }
        }
    }
}

/// ⭐ **A lei da booleana, em números** — a mesma de [`crate::ops`], escrita uma segunda vez porque
/// um `min` entre uma fita de JIT e uma grade de voxels não pode acontecer dentro de nenhuma das
/// duas.
///
/// ⚠️ **É a única duplicação de lei deste módulo, e ela tem juiz**: o gate
/// `the_numeric_law_is_the_same_law_as_the_tree` avalia as duas formas no mesmo documento e compara
/// ponto a ponto. Sem ele, as duas divergiriam na primeira wave que mexesse numa e esquecesse a
/// outra — e o sintoma seria um filete com outro raio, não um erro.
#[must_use]
pub fn apply(op: Op, a: f32, b: f32) -> f32 {
    match op {
        Op::Union(blend) => union(a, b, blend),
        // De Morgan, exatamente como a árvore faz.
        Op::Intersection(blend) => -union(-a, -b, blend),
        Op::Difference(blend) => -union(-a, b, blend),
    }
}

fn union(a: f32, b: f32, blend: Blend) -> f32 {
    match blend {
        Blend::Sharp => a.min(b),
        Blend::Exact { radius } if radius > 0.0 => {
            let ux = (radius - a).max(0.0);
            let uy = (radius - b).max(0.0);
            a.min(b).max(radius) - ux.hypot(uy)
        }
        Blend::Organic { k } if k > 0.0 => {
            let h = 0.5f32.mul_add((b - a) / k, 0.5).clamp(0.0, 1.0);
            let mixed = (a - b).mul_add(h, b);
            mixed - k * h * (1.0 - h)
        }
        // ⚠️ Raio zero é união DURA, e não uma fórmula com um zero dentro — é o mesmo ramo que a
        // árvore toma, e a razão é a mesma: com `r = 0` as duas são algebricamente idênticas.
        Blend::Exact { .. } | Blend::Organic { .. } => a.min(b),
    }
}

struct Builder<'a> {
    reg: &'a Registry,
    trees: Vec<Tree>,
    sampled: Vec<SampledLeaf>,
}

impl Builder<'_> {
    fn build(&mut self, doc: &FieldDoc) -> Plan {
        // Cada nó vira **ou** uma árvore (fundível) **ou** um plano já misto.
        enum Built {
            Tree(Tree),
            Mixed(Plan),
        }
        let mut built: Vec<Built> = Vec::with_capacity(doc.nodes().len());
        for node in doc.nodes() {
            let b = match &node.kind {
                NodeKind::Sampled { key } => {
                    // ⚠️ Uma folha amostrada **não passa** pela pilha de modificadores nem pelo
                    // `place` da árvore: a pose viaja com ela e é desfeita na amostragem.
                    let field = self.reg.get(key).cloned();
                    match field {
                        Some(field) => {
                            self.sampled.push(SampledLeaf {
                                field,
                                inv_rot: crate::inverse_rotation_matrix(node.xform.rotation),
                                translation: node.xform.translation,
                                scale: node.xform.scale,
                            });
                            Built::Mixed(Plan::Sampled(self.sampled.len() - 1))
                        }
                        // Nome desconhecido: espaço vazio, e o documento continua a abrir.
                        None => Built::Tree(Tree::constant(f64::from(ABSENT))),
                    }
                }
                NodeKind::Leaf(p) => Built::Tree(crate::place(
                    &crate::stacked(&crate::primitive(p), &node.mods),
                    node.xform,
                )),
                NodeKind::Combine { op, children } => {
                    let all_analytic = children
                        .iter()
                        .all(|c| matches!(built[c.0 as usize], Built::Tree(_)));
                    if all_analytic {
                        // ⭐ **A FUSÃO**: nada de amostrado por baixo, então isto volta a ser uma
                        // árvore só — e o documento sem escultura produz uma fita, como antes.
                        let trees: Vec<Tree> = children
                            .iter()
                            .map(|c| match &built[c.0 as usize] {
                                Built::Tree(t) => t.clone(),
                                Built::Mixed(_) => unreachable!("acabou de se verificar"),
                            })
                            .collect();
                        Built::Tree(crate::place(
                            &crate::stacked(&crate::combine_trees(*op, &trees), &node.mods),
                            node.xform,
                        ))
                    } else {
                        // ⚠️ **A pose e os modificadores de um `Combine` MISTO ainda não correm**,
                        // e a nota fica aqui em vez de num `TODO`: aplicá-los exigiria a casca, a
                        // matriz e a inclinação escritas uma segunda vez em números, cada uma com o
                        // gate de paridade que a segure. Enquanto isso, um `Combine` com escultura
                        // por baixo é avaliado **na pose dele próprio**, e a pilha é recusada pelo
                        // documento (`FieldError::ModsOnSampled` cobre a folha; o `Combine` misto é
                        // o item aberto nomeado no §22).
                        let mut kids = Vec::with_capacity(children.len());
                        for c in children {
                            let taken = std::mem::replace(
                                &mut built[c.0 as usize],
                                Built::Tree(Tree::constant(0.0)),
                            );
                            kids.push(match taken {
                                Built::Tree(t) => {
                                    self.trees.push(t);
                                    Plan::Analytic(self.trees.len() - 1)
                                }
                                Built::Mixed(p) => p,
                            });
                        }
                        Built::Mixed(Plan::Combine(*op, kids))
                    }
                }
            };
            built.push(b);
        }
        match built.swap_remove(doc.root().0 as usize) {
            Built::Tree(t) => {
                self.trees.push(t);
                Plan::Analytic(self.trees.len() - 1)
            }
            Built::Mixed(p) => p,
        }
    }
}
