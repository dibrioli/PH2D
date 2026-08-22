//! **A pilha de FX raster como COMPONENTE** — o embrulho que põe o degrau no documento.
//!
//! ⚠️ **O degrau em si mudou de casa em 2026-08-21**: o `FxOp` e o catálogo que o descreve vivem
//! na folha [`ph2d_fx_op`], porque passaram a ter um terceiro consumidor que não pode ver o ECS —
//! a POSE de um estado de UI (`ph2d_ui_state::ObjectPose`), para um blur ou um glow poderem
//! diferir entre *Default* e *Hover*. É o precedente literal da `ph2d-stroke-width`, que existe
//! pela mesma razão: quando um canal precisa de estar no documento **e** numa pose, a casa do
//! tipo é uma folha e o componente é só o embrulho.
//!
//! O que fica aqui é o que é de facto ECS — a pilha que uma entidade carrega, e as perguntas que
//! só fazem sentido sobre ela (o teto, se desenha alguma coisa, reordenar).

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;
pub use ph2d_fx_op::{FxKindSpec, FxOp};

/// **A pilha de FX raster de uma forma.** A entidade que a carrega também tem um
/// [`crate::VecPathRef`]: o `VecPath` dela continua a curva AUTORADA (o modo Node a edita); o
/// resultado filtrado é DESENHO, que a shell produz por frame e injeta no z da fonte.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VecFilter {
    /// Os degraus, **na ordem em que se aplicam** (o primeiro recebe a forma nua). A ordem é a
    /// feature: `Shadow → Blur` e `Blur → Shadow` desenham coisas diferentes.
    pub ops: Vec<FxOp>,
}

impl SimComponent for VecFilter {}

impl VecFilter {
    /// O teto de degraus numa pilha.
    ///
    /// **O recurso que aperta é a TELA do painel, não a GPU** — e isso está MEDIDO, não suposto
    /// (`ph2d-render/tests/fx_stack_gpu.rs::the_cost_of_a_stack_is_linear_in_the_number_of_ops`,
    /// RTX, 512×512, sigma 8 px): `0 degraus 0,082 ms · 1 → 0,084 · 2 → 0,149 · 3 → 0,220 ·
    /// 4 → 0,336 · 6 → **0,429 ms**`. O custo é linear, ~0,07 ms por degrau, e uma pilha CHEIA
    /// custa **2,6 % de um frame de 60 fps**. Cada degrau, em compensação, é um card de 4-6 linhas
    /// no painel — seis já enchem a coluna. É a mesma razão do `MAX_PATH_EFFECTS` da pilha de
    /// geometria, com o número um degrau acima porque aqui o card é mais raso.
    pub const MAX_OPS: usize = 6;

    /// Uma pilha com um degrau — o que o 1º "Add" produz.
    #[must_use]
    pub fn single(op: FxOp) -> Self {
        Self { ops: vec![op] }
    }

    /// A pilha desenha alguma coisa? Vazia (ou toda desligada) = a forma sai nua, e a shell não
    /// produz imagem nenhuma. **Porta única**: quem coze e quem decide se há FX perguntam aqui.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.ops.iter().any(|o| o.is_active())
    }

    /// Há espaço para mais um degrau?
    #[must_use]
    pub fn has_room(&self) -> bool {
        self.ops.len() < Self::MAX_OPS
    }

    /// Troca `row` com o vizinho de cima. `false` (e nada muda) na primeira linha.
    pub fn move_up(&mut self, row: usize) -> bool {
        if row == 0 || row >= self.ops.len() {
            return false;
        }
        self.ops.swap(row - 1, row);
        true
    }

    /// Troca `row` com o vizinho de baixo. `false` (e nada muda) na última linha.
    pub fn move_down(&mut self, row: usize) -> bool {
        if row + 1 >= self.ops.len() {
            return false;
        }
        self.ops.swap(row, row + 1);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(kinds: &[u8]) -> VecFilter {
        VecFilter {
            ops: kinds.iter().map(|k| FxOp::new(*k)).collect(),
        }
    }

    /// **Reordenar troca DOIS vizinhos, e as pontas são no-ops** — subir na primeira linha e descer
    /// na última não fazem nada, e o painel nem desenha essas setas. Aqui prova-se que, mesmo se
    /// alguém as despachasse, a pilha não se deforma (um `swap` fora de faixa entraria em pânico).
    #[test]
    fn reordering_swaps_neighbours_and_the_ends_are_no_ops() {
        let mut f = stack(&[FxOp::BLUR, FxOp::GLOW, FxOp::DROP_SHADOW]);
        assert!(f.move_down(0));
        assert_eq!(
            f.ops.iter().map(|o| o.kind).collect::<Vec<_>>(),
            vec![FxOp::GLOW, FxOp::BLUR, FxOp::DROP_SHADOW]
        );
        assert!(f.move_up(2));
        assert_eq!(
            f.ops.iter().map(|o| o.kind).collect::<Vec<_>>(),
            vec![FxOp::GLOW, FxOp::DROP_SHADOW, FxOp::BLUR]
        );
        let before = f.clone();
        assert!(!f.move_up(0), "subir na primeira linha não faz nada");
        assert!(!f.move_down(2), "descer na última não faz nada");
        assert!(!f.move_down(9), "nem uma linha que não existe");
        assert_eq!(f, before, "e nenhuma delas pode deformar a pilha");
    }

    /// **Uma pilha só desenha se algum degrau estiver LIGADO** — a porta única que o produtor e a
    /// remoção do componente perguntam. Vazia e toda-desligada são o mesmo fato para quem desenha.
    #[test]
    fn a_stack_is_active_only_while_some_op_is_enabled() {
        assert!(!VecFilter::default().is_active(), "vazia não desenha nada");
        let mut f = stack(&[FxOp::BLUR, FxOp::GLOW]);
        assert!(f.is_active());
        f.ops[0].enabled = false;
        assert!(f.is_active(), "um degrau ligado basta");
        f.ops[1].enabled = false;
        assert!(!f.is_active(), "toda desligada é o mesmo que vazia");
    }

    /// O teto é respondido pela pilha, não contado no chamador.
    #[test]
    fn the_ceiling_is_the_stacks_own_answer() {
        let mut f = VecFilter::default();
        while f.has_room() {
            f.ops.push(FxOp::new(FxOp::BLUR));
        }
        assert_eq!(f.ops.len(), VecFilter::MAX_OPS);
    }
}
