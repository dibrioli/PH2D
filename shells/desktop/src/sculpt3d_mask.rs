//! **A MÁSCARA** — as quatro operações que agem na malha inteira.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados; o corte é
//! entre *o que um TRAÇO de máscara faz* (o pincel, no pai — ela é um verbo como
//! qualquer outro) e *o que se faz com a máscara INTEIRA* (aqui). São gestos
//! diferentes: um pinta, os outros quatro agem sobre tudo que já foi pintado.

use super::{MASK_OP_PASSES, Sculpt3dScene, StrokeUndo};

/// As quatro operações de máscara — ver [`Sculpt3dScene::mask_op`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MaskOp {
    Clear,
    Invert,
    Blur,
    Sharpen,
}

impl MaskOp {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Clear => "limpa",
            Self::Invert => "inverte",
            Self::Blur => "borra",
            Self::Sharpen => "afia",
        }
    }
}

impl Sculpt3dScene {
    /// **Uma operação de máscara**, com o undo e o upload que ela implica.
    ///
    /// ⚠️ **A entrada de undo é a MÁSCARA INTEIRA, e não a janela de um traço.**
    /// Estas operações agem na malha toda por definição (o `blur` alcança todo
    /// vértice cuja vizinhança tem máscara), então uma janela seria uma mentira
    /// sobre o que mudou — e o que ela custa é `4 B × vértices`, o mesmo que o
    /// plano que ela desfaz.
    ///
    /// ⚠️ E a GPU tem de re-ler a malha INTEIRA: o `dirty` incremental é a
    /// janela de um dab, e aqui não houve dab.
    pub(super) fn mask_op(&mut self, op: MaskOp) {
        let before = self.mesh().masks().map(<[f32]>::to_vec);
        match op {
            MaskOp::Clear => {
                if !ph2d_sculpt3d::mask_ops::clear(self.mesh_mut()) {
                    return;
                }
            }
            MaskOp::Invert => ph2d_sculpt3d::mask_ops::invert(self.mesh_mut()),
            MaskOp::Blur => ph2d_sculpt3d::mask_ops::blur(self.mesh_mut(), MASK_OP_PASSES),
            MaskOp::Sharpen => ph2d_sculpt3d::mask_ops::sharpen(self.mesh_mut(), MASK_OP_PASSES),
        }
        self.record(StrokeUndo::Mask {
            level: self.obj().stack.level(),
            before,
        });
        self.obj_mut().uploaded = false;
        self.edits += 1;
    }
}
