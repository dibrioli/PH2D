//! ⏱️⭐⭐⭐ **A PODA POR REGIÃO — quantas instruções da peça sobram dentro de uma caixa do espaço.**
//!
//! O estado da arte para campos implícitos complexos no dispositivo (Keeter, *Massively Parallel
//! Rendering of Complex Closed-Form Implicit Surfaces*, 2020) avalia a peça por **intervalos** sobre
//! uma região, anota que ramo de cada `min`/`max` ganha ali, e desenha a região com uma fita
//! **podada** — só as instruções que ainda podem decidir o valor. A `fidget` é a biblioteca dele e
//! é a que já compila o nosso documento, logo a poda é a dela e não uma segunda lei.
//!
//! ⚠️ **Hoje é INSTRUMENTO, não produto:** responde *«quanto compraria»* antes de alguém construir a
//! fita por ladrilho no dispositivo (`docs/Render3d/03_o_plano.md` §W9, «onde os passos
//! acontecem»). ⛔ Nada no produto a chama.

use fidget::context::Tree;
use fidget::types::Interval;
use fidget::vm::VmShape;

/// O que a poda diz de uma caixa.
#[derive(Clone, Debug, PartialEq)]
pub enum Poda {
    /// O intervalo prova que a caixa NÃO toca a superfície: o campo fica acima de `min` em todo o
    /// lado dela (ou abaixo de `−min`, dentro da peça). Um raio atravessa-a sem avaliar a árvore.
    Vazia { min: f32 },
    /// A caixa pode tocar a superfície, e dentro dela bastam estas instruções.
    ///
    /// `rasto` é a escolha de cada `min`/`max` (um byte por escolha): **duas caixas com o mesmo
    /// rasto têm a MESMA fita podada** — é a impressão digital que conta quantas fitas diferentes um
    /// quadro pede.
    Fita { instrucoes: usize, rasto: Vec<u8> },
}

/// ⏱️ A peça pronta a ser podada — construída UMA vez, perguntada por caixa.
pub struct Podador {
    forma: VmShape,
    fita: fidget::shape::ShapeTape<<<fidget::vm::VmFunction as fidget::eval::Function>::IntervalEval as fidget::eval::TracingEvaluator>::Tape>,
    avaliador: fidget::shape::ShapeTracingEval<<fidget::vm::VmFunction as fidget::eval::Function>::IntervalEval>,
}

impl Podador {
    /// A peça, pela mesma compilação que o [`crate::Field`] usa.
    #[must_use]
    pub fn new(arvore: &Tree) -> Self {
        let forma = VmShape::from(arvore.clone());
        let fita = forma.interval_tape(Default::default());
        Self {
            forma,
            fita,
            avaliador: VmShape::new_interval_eval(),
        }
    }

    /// Instruções da peça INTEIRA — o denominador de toda a razão.
    #[must_use]
    pub fn tamanho(&self) -> usize {
        self.forma.size()
    }

    /// ⭐ A poda sobre a caixa `[lo, hi]`.
    ///
    /// # Panics
    /// Se a `fidget` recusar a própria fita (não acontece com uma fita que ela construiu).
    pub fn na_caixa(&mut self, lo: [f32; 3], hi: [f32; 3]) -> Poda {
        let x = Interval::new(lo[0], hi[0]);
        let y = Interval::new(lo[1], hi[1]);
        let z = Interval::new(lo[2], hi[2]);
        let (v, rasto) = self
            .avaliador
            .eval(&self.fita, x, y, z)
            .expect("a fita da fidget");
        if v.lower() > 0.0 {
            return Poda::Vazia { min: v.lower() };
        }
        if v.upper() < 0.0 {
            return Poda::Vazia { min: -v.upper() };
        }
        let Some(rasto) = rasto else {
            return Poda::Fita {
                instrucoes: self.tamanho(),
                rasto: Vec::new(),
            };
        };
        let assinatura: Vec<u8> = rasto.as_slice().iter().map(|c| *c as u8).collect();
        let podada = self
            .forma
            .simplify(rasto, Default::default(), &mut Default::default())
            .expect("o rasto é desta fita");
        Poda::Fita {
            instrucoes: podada.size(),
            rasto: assinatura,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Poda, Podador};
    use fidget::context::Tree;

    /// ⭐ Duas esferas afastadas: numa caixa à volta de UMA, a poda deita fora a outra; numa caixa
    /// longe das duas, a caixa é vazia. ⚠️ O CONTROLO é a caixa que abraça as duas, onde nada cai.
    #[test]
    fn a_poda_deita_fora_a_esfera_longe() {
        let esfera = |cx: f64| {
            let (x, y, z) = (Tree::x() - cx, Tree::y(), Tree::z());
            (x.square() + y.square() + z.square()).sqrt() - 0.5
        };
        let duas = esfera(-2.0).min(esfera(2.0));
        let mut p = Podador::new(&duas);
        let inteira = p.tamanho();
        let Poda::Fita {
            instrucoes: uma, ..
        } = p.na_caixa([1.2, -0.6, -0.6], [2.8, 0.6, 0.6])
        else {
            panic!("a caixa à volta de uma esfera toca a superfície");
        };
        assert!(
            uma < inteira,
            "a poda não deitou nada fora ({uma} contra {inteira})"
        );
        let Poda::Fita {
            instrucoes: ambas, ..
        } = p.na_caixa([-3.0, -1.0, -1.0], [3.0, 1.0, 1.0])
        else {
            panic!("a caixa das duas toca a superfície");
        };
        assert_eq!(
            ambas, inteira,
            "CONTROLO: a caixa que abraça as duas não pode podar nada"
        );
        assert!(matches!(
            p.na_caixa([-0.4, 3.0, -0.4], [0.4, 3.5, 0.4]),
            Poda::Vazia { .. }
        ));
    }
}
