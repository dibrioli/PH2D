//! ⭐⭐⭐ **A PEÇA INTEIRA PARA O DISPOSITIVO, com a ESCULTURA dentro.**
//!
//! # ⛔⛔ O que a fita de sempre perde, e porquê
//!
//! O [`crate::compile`] traduz uma [`ph2d_field::NodeKind::Sampled`] para
//! `Tree::constant(ABSENT)` — **espaço vazio** — porque a álgebra da `fidget` é fechada: `TreeOp`
//! é `Input` · `Const` · `Binary` · `Unary` · remapeamentos, e **não há consulta a dados**. É por
//! isso que o [`crate::hybrid`] existe do lado da CPU, e era por isso que uma peça com escultura
//! ficava fora do dispositivo: mandá-la na fita de sempre faria a escultura **desaparecer em
//! silêncio, com o resto da peça perfeito**.
//!
//! # ⭐⭐⭐ A cura: a escultura entra como uma VARIÁVEL, e a álgebra fica intacta
//!
//! A `fidget` tem variáveis genéricas ([`fidget::var::Var::V`]), e um remapeamento de eixos **não
//! lhes toca**. ⇒ cada escultura vira uma variável, a árvore continua a ser **uma só** — com todos
//! os filetes, chanfros e booleanas onde sempre estiveram — e o gerador de WGSL traduz a variável
//! numa **chamada** (`escultura_k(p)`), cujo corpo este módulo escreve.
//!
//! ⚠️⚠️ **É isto que evita uma TERCEIRA cópia da lei das booleanas.** Ela já existe duas vezes (a
//! árvore e o [`crate::hybrid_law`], com o gate `the_numeric_law_is_the_same_law_as_the_tree` a
//! segurá-las); portá-la para WGSL seria uma terceira, com uma chance a mais de divergir. *Aqui o
//! dispositivo corre a MESMA árvore que a CPU analítica corre — a escultura é que é uma folha.*
//!
//! # ⚠️ A pose e a pilha de um `Combine` MISTO não correm — e isso é PARIDADE, não omissão
//!
//! O [`crate::hybrid`] declara-o por escrito: *«um `Combine` com escultura por baixo é avaliado na
//! pose dele próprio»*. Reproduzir esse limite é o que faz as duas imagens baterem; «corrigi-lo» só
//! aqui poria o dispositivo a desenhar outra peça que a CPU.

use crate::hybrid::{ABSENT, Registry, Sampled, SampledGrid};
use fidget::context::Tree;
use fidget::var::Var;
use ph2d_field::{FieldDoc, NodeKind};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Uma escultura, com a pose já invertida — o gémeo do `hybrid::SampledLeaf`.
#[derive(Clone)]
pub struct DeviceSculpt {
    /// De onde a grade vem. ⚠️ O ponteiro dele é a **identidade** para um cache de dispositivo.
    pub field: Arc<dyn Sampled>,
    /// `p' = R⁻¹(p − t)/s` — a mesma conta do [`crate::place`], em números.
    pub inv_rot: [[f64; 3]; 3],
    pub translation: [f32; 3],
    pub scale: f32,
}

impl DeviceSculpt {
    /// A grade desta folha. `None` quando a folha não é uma grade densa.
    #[must_use]
    pub fn grid(&self) -> Option<SampledGrid<'_>> {
        self.field.grid()
    }
}

/// A peça compilada para o dispositivo: a árvore (com variáveis) e as esculturas que elas nomeiam.
pub struct DeviceField {
    tape: crate::point_tape::PointTape,
    sculpts: Vec<DeviceSculpt>,
}

impl DeviceField {
    /// ⭐⭐⭐ **Compila `doc` para o dispositivo.**
    ///
    /// `None` quando **alguma** escultura viva não souber entregar a grade — e nesse caso o
    /// chamador fica na CPU, que sabe desenhá-la. ⛔ Tudo ou nada de propósito: uma escultura em
    /// falta não é «uma folha a menos», é a peça **com um buraco** onde havia matéria.
    #[must_use]
    pub fn new(doc: &FieldDoc, reg: &Registry) -> Option<Self> {
        let (tree, sculpts, vars) = arvore(doc, reg)?;
        let mut ctx = fidget::context::Context::new();
        let raiz = ctx.import(&tree);
        let tape = crate::point_tape::PointTape::build_with_vars(&ctx, raiz, &vars);
        Some(Self { tape, sculpts })
    }

    /// As esculturas que a fita nomeia, na ordem dos índices dela.
    #[must_use]
    pub fn sculpts(&self) -> &[DeviceSculpt] {
        &self.sculpts
    }

    /// ⭐⭐ **O retrato da fita** — ver [`crate::point_tape::TapeShape`]. O `vivos` é o scratch **por
    /// thread**, e numa placa ele é o que decide a OCUPAÇÃO: quantas threads cabem num
    /// multiprocessador ao mesmo tempo.
    #[must_use]
    pub fn tape_shape(&self) -> Option<crate::point_tape::TapeShape> {
        self.tape.shape()
    }

    /// ⭐ **A fita em WGSL.** As chamadas a `escultura_k` ficam por resolver — quem escreve o corpo
    /// delas é quem liga a grade ([`crate::wgsl::ESCULTURA`]).
    #[must_use]
    pub fn tape_wgsl(&self) -> Option<crate::wgsl::TapeWgsl> {
        self.tape.to_wgsl()
    }
}

/// A árvore com variáveis, as esculturas e o mapa `Var → índice`.
fn arvore(doc: &FieldDoc, reg: &Registry) -> Option<(Tree, Vec<DeviceSculpt>, BTreeMap<Var, u32>)> {
    let balls = crate::bounds::local_balls(doc, reg);
    let mut built: Vec<Tree> = Vec::with_capacity(doc.nodes().len());
    // ⚠️ **Quem tem escultura por baixo não leva pose nem pilha** — ver a nota do módulo.
    let mut misto: Vec<bool> = Vec::with_capacity(doc.nodes().len());
    let mut sculpts: Vec<DeviceSculpt> = Vec::new();
    let mut vars: BTreeMap<Var, u32> = BTreeMap::new();

    for (i, node) in doc.nodes().iter().enumerate() {
        let (inner, este_misto) = match &node.kind {
            NodeKind::Leaf(p) => (crate::primitive(p), false),
            NodeKind::Combine { op, children } => {
                let m = children.iter().any(|c| misto[c.0 as usize]);
                (crate::combine(*op, children, doc.nodes(), &built), m)
            }
            NodeKind::Sampled { key } => match reg.get(key) {
                Some(field) => {
                    // ⛔ Sem grade não há caminho de dispositivo — a peça inteira cai na CPU.
                    field.grid()?;
                    let v = Var::new();
                    #[allow(clippy::cast_possible_truncation)]
                    vars.insert(v, sculpts.len() as u32);
                    sculpts.push(DeviceSculpt {
                        field: Arc::clone(field),
                        inv_rot: crate::inverse_rotation_matrix(node.xform.rotation),
                        translation: node.xform.translation,
                        scale: node.xform.scale,
                    });
                    (Tree::from(v), true)
                }
                // ⚠️ **Nome desconhecido é espaço VAZIO, e não um erro** — a mesma resposta do
                // `hybrid`: o documento continua a abrir, e na união a folha some.
                None => (Tree::constant(f64::from(ABSENT)), false),
            },
        };
        // ⚠️ **A folha amostrada não passa pela pilha nem pelo `place`**: a pose viaja com ela e é
        // desfeita na amostragem. E um `Combine` misto herda a mesma abstinência — ver o módulo.
        let colocado = if este_misto {
            inner
        } else {
            crate::place(
                &crate::stacked(
                    &inner,
                    &node.mods,
                    balls[i].unwrap_or(crate::bounds::Ball::EMPTY),
                ),
                node.xform,
            )
        };
        built.push(colocado);
        misto.push(este_misto);
    }
    Some((built[doc.root().0 as usize].clone(), sculpts, vars))
}

#[cfg(test)]
#[path = "device_tests.rs"]
mod tests;
