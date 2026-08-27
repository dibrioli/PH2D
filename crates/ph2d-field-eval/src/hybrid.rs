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

/// A fita float compilada, **sem** o avaliador — ver [`RegionTape`].
type FloatTape = fidget::shape::ShapeTape<
    <<fidget::jit::JitFunction as fidget::eval::Function>::FloatSliceEval as
        fidget::eval::BulkEvaluator>::Tape,
>;

type Tapes = (
    fidget::shape::ShapeBulkEval<
        <fidget::jit::JitFunction as fidget::eval::Function>::FloatSliceEval,
    >,
    FloatTape,
);

/// ⭐⭐⭐ **UMA FITA JÁ COMPILADA, PRONTA A SER PARTILHADA** (W82) — o que uma cache entre quadros
/// guarda.
///
/// # Por que ela existe
///
/// ⚠️ **Compilar é a parede do traçado, e ela não escala.** Medido (W81, `docs/3DModeling` §82.9):
/// as `242` fitas de um quadro de movimento custam `~130 ms` em série e **`~14 ms` a partir de 16
/// threads — de 16 para 32 o ganho é `1 %`**. Um JIT mapeia memória **executável**, e
/// `mmap`/`mprotect` são recursos do kernel: a compilação em parte **serializa-se**, e núcleos a
/// mais não a atravessam. Num quadro de `~24 ms` isso é metade do relógio, repetido inteiro a cada
/// quadro enquanto a mão mexe.
///
/// # ⭐⭐ O que a torna partilhável
///
/// A fita da `fidget` é um **`Arc<Mmap>`** por dentro: cloná-la é um incremento de contador. O que
/// **não** se partilha é o avaliador (`ShapeBulkEval`), que é o estado mutável — e esse nasce por
/// uso, de graça. ⇒ *uma fita serve todas as threads; um avaliador serve uma.*
///
/// ⚠️ **Ela carrega a árvore ao lado da fita** porque o [`Hybrid`] a guarda para o
/// [`Hybrid::fork`] — e a `Tree` da `fidget` também é um `Arc` por dentro.
#[derive(Clone)]
pub struct RegionTape {
    tree: Tree,
    tape: FloatTape,
}

/// ⭐⭐⭐ **A fita atravessa threads, e isto é um GATE — não um comentário.**
///
/// Toda a razão de existir da [`RegionTape`] é ser guardada uma vez e usada por todas as threads de
/// um quadro. Se um campo futuro (um `Rc`, um `Cell`) lhe tirar o `Send + Sync`, a linha abaixo
/// **deixa de compilar** — e não há como o descobrir mais tarde, num sítio pior.
const _: () = {
    const fn is_send_sync<T: Send + Sync>() {}
    is_send_sync::<RegionTape>();
};

impl RegionTape {
    /// **Compila** — é aqui, e só aqui, que o JIT corre para uma região.
    #[must_use]
    pub fn compile(tree: Tree) -> Self {
        let shape = Engine::from(tree.clone());
        FLOAT_TAPES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let tape = shape.ez_float_slice_tape();
        Self { tree, tape }
    }
}

/// ⭐⭐ **Quantas fitas FLOAT foram compiladas** — o instrumento da lei *«uma fita monta-se uma vez
/// por quem a avalia»* (W70).
///
/// ⚠️ **Ele existe porque a montagem é o quadro inteiro** e era invisível: `132` especializações
/// num traçado a `640×360`, cada uma a compilar uma fita — e a marcha **forkava** a que acabara de
/// construir, dobrando a conta sem que gate nenhum pudesse vê-lo (a imagem é idêntica; só o relógio
/// muda). *Um custo que nenhuma sonda conta é um custo que nenhuma mutação mata.*
#[doc(hidden)]
pub static FLOAT_TAPES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// ⭐⭐⭐ **Quantas fitas de GRADIENTE foram compiladas** — ver [`Hybrid::grad`].
///
/// ⚠️ A lei que ele defende é *«o traçado não compila fita de gradiente nenhuma»*: o consumidor
/// dela é a **extração**, e a normal do traçado sai por diferença central na fita float.
#[doc(hidden)]
pub static GRAD_TAPES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// ⭐ **O documento pronto a avaliar em lote** — a porta única do traçado e da extração.
pub struct Hybrid {
    plan: Plan,
    /// As árvores por trás das fitas — guardadas para o [`Hybrid::fork`].
    trees: Vec<Tree>,
    tapes: Vec<Tapes>,
    sampled: Vec<SampledLeaf>,
    /// ⭐ **A fita de GRADIENTE, só quando o documento é uma árvore só — e só quando alguém a pede.**
    ///
    /// ⚠️ Um campo amostrado **não tem gradiente analítico** — ele é uma grade. Quando há escultura,
    /// a normal sai por diferença central, e isso tem um preço nomeado: numa **quina viva** a
    /// diferença central devolve a média dos dois lados em vez de um deles, e o QEF deixa de a
    /// prender. É a razão de o caminho exato não ser abandonado quando ele existe.
    ///
    /// ⭐⭐⭐ **Ela é PREGUIÇOSA desde a W70, e o motivo é uma contagem de consumidores:**
    /// `Hybrid::gradients` tem **um** chamador em toda a árvore — a extração de malha
    /// (`extract.rs`), que corre na **exportação**. O traçado nunca lhe toca: a normal dele sai de
    /// **seis amostras na fita float** (`march::normals_into`). Construí-la sempre custava
    /// `1,47 ms` por especialização (a fita float custa `1,37`), e o traçado especializa **132**
    /// árvores num quadro a `640×360` ⇒ *metade da montagem de um quadro era uma fita que ninguém
    /// avaliava*.
    ///
    /// ⚠️ **`None` deixou de significar «não há gradiente exato»** — passou a significar *«ainda
    /// não foi pedido, ou não há»*. Quem decide é [`Hybrid::grad_is_exact`], e a distinção vive
    /// dentro de `gradients`, que é o único sítio onde ela importa.
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

    /// ⭐⭐ **Um avaliador a partir de uma ÁRVORE já compilada** — a porta da especialização por
    /// região (W56, [`crate::compile_in_region`]).
    ///
    /// ⚠️ **Ela preserva o gradiente exacto**, e não por sorte: o `grad` só é possível com
    /// `sampled.is_empty() && trees.len() == 1`, que é exactamente a forma que esta porta produz.
    /// Era essa a propriedade que a rota da folha nativa perdia, e é a razão de a especialização ter
    /// ficado **dentro** da árvore.
    #[must_use]
    pub fn from_tree(tree: Tree) -> Self {
        Self::from_region_tape(&RegionTape::compile(tree))
    }

    /// ⭐⭐⭐ **Um avaliador sobre uma fita que JÁ ESTÁ COMPILADA** (W82) — a porta da cache entre
    /// quadros, e a única que não corre o JIT.
    ///
    /// ⚠️ **A fita é clonada e o avaliador é novo**, e essa assimetria é o desenho inteiro: a fita é
    /// um `Arc<Mmap>` (partilhável, imutável) e o avaliador é o estado mutável de uma thread. Ver
    /// [`RegionTape`].
    ///
    /// ⚠️ **A cerca da região viaja com quem chama.** Uma fita especializada só responde certo
    /// **dentro** da região para que foi construída ([`crate::compile_in_region`]) — esta porta não
    /// tem como saber qual era, e quem a guarda é quem tem de a comparar.
    #[must_use]
    pub fn from_region_tape(t: &RegionTape) -> Self {
        Self {
            plan: Plan::Analytic(0),
            trees: vec![t.tree.clone()],
            tapes: vec![(Engine::new_float_slice_eval(), t.tape.clone())],
            sampled: Vec::new(),
            grad: None,
            leaves: vec![Vec::new()],
            out: Vec::new(),
        }
    }

    /// ⭐⭐⭐ **Um avaliador NOVO sobre as MESMAS FITAS** — o que uma marcha paralela precisa, e
    /// **sem compilar nada** (W83).
    ///
    /// ⚠️ **O avaliador da `fidget` tem estado mutável**, então partilhá-lo entre threads exigiria
    /// trava; a marcha resolve isso construindo um por lote. ⭐ Mas a **fita** é imutável e é um
    /// `Arc<Mmap>` por dentro: cloná-la é um incremento de contador. *O que se partilha é o código;
    /// o que se duplica é o rascunho.*
    ///
    /// ⛔⛔ **Até à W83 este `fork` RECOMPILAVA**, e o preço estava escondido no sítio mais caro
    /// que havia: a 2.ª passagem do traçado (o anti-serrilhado) constrói um avaliador por **lote de
    /// 64 pixels de borda**, e cada um compilava a árvore **inteira** — a mais cara que existe, sem
    /// especialização nenhuma. Medido (`docs/3DModeling/06` §84): num assentar a `640×360` são
    /// `1 762` pixels de borda ⇒ `28` lotes ⇒ **`29` das `29` fitas do quadro**, com a passagem
    /// primária a `100 %` de acerto na cache.
    ///
    /// ⚠️ **A W70 mediu «reaproveitar o avaliador entre lotes» e achou-o NEUTRO** — e a nota dela
    /// dizia porquê: *«o quadro tem `917` regiões especializadas nesse tamanho: as dezenas de fitas
    /// desta passagem são ruído ao lado delas»*. ⭐ **A W82 apagou aquele `917`**, e com ele a
    /// premissa. *Quem move o número que sustenta uma nota tem de reconferir a nota.*
    #[must_use]
    pub fn fork(&self) -> Self {
        let n = self.tapes.len() + self.sampled.len();
        Self {
            plan: self.plan.clone(),
            trees: self.trees.clone(),
            tapes: self
                .tapes
                .iter()
                .map(|(_, tape)| (Engine::new_float_slice_eval(), tape.clone()))
                .collect(),
            sampled: self.sampled.clone(),
            grad: None,
            leaves: vec![Vec::new(); n],
            out: Vec::new(),
        }
    }

    fn from_parts(plan: Plan, trees: Vec<Tree>, sampled: Vec<SampledLeaf>) -> Self {
        let mut tapes = Vec::with_capacity(trees.len());
        for t in &trees {
            let shape = Engine::from(t.clone());
            FLOAT_TAPES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            tapes.push((Engine::new_float_slice_eval(), shape.ez_float_slice_tape()));
        }
        let n = tapes.len() + sampled.len();
        Self {
            plan,
            trees,
            tapes,
            sampled,
            // ⭐⭐⭐ **A fita de gradiente NASCE VAZIA** (W70) — ver [`Hybrid::grad`].
            grad: None,
            leaves: vec![Vec::new(); n],
            out: Vec::new(),
        }
    }

    /// **O documento é uma árvore só?** — a condição de existir gradiente exato, perguntada onde ela
    /// é usada em vez de decidida na construção.
    fn grad_is_exact(&self) -> bool {
        self.sampled.is_empty() && self.trees.len() == 1
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
        // ⭐⭐⭐ **A fita de gradiente monta-se AQUI, no primeiro pedido** (W70) — e é o único sítio
        // onde `grad.is_none()` ainda não quer dizer *«este documento não tem gradiente exato»*.
        if self.grad.is_none() && self.grad_is_exact() {
            let shape = Engine::from(self.trees[0].clone());
            GRAD_TAPES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.grad = Some((Engine::new_grad_slice_eval(), shape.ez_grad_slice_tape()));
        }
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
