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

use crate::{ItemId, SimComponent};
pub use ph2d_fx_op::{FxKindSpec, FxOp};

/// **A pilha de FX raster de uma forma.** A entidade que a carrega também tem um
/// [`crate::VecPathRef`]: o `VecPath` dela continua a curva AUTORADA (o modo Node a edita); o
/// resultado filtrado é DESENHO, que a shell produz por frame e injeta no z da fonte.
///
/// ⭐ Cada degrau tem um [`ItemId`] (F31 §B2): a chave de animação de um filtro é o id, não a
/// posição. `ops` e `ids` são privados e só mudam de comprimento JUNTOS — não há como divergirem.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "Gravado")]
pub struct VecFilter {
    /// Os degraus, **na ordem em que se aplicam** (o primeiro recebe a forma nua). A ordem é a
    /// feature: `Shadow → Blur` e `Blur → Shadow` desenham coisas diferentes.
    ops: Vec<FxOp>,
    /// O id de cada degrau (`NONE` até ao passe [`crate::item_id::assign_missing_item_ids`]).
    ids: Vec<ItemId>,
}

/// A forma no fio; o comprimento dos ids acerta-se ao dos degraus ao ler (bytes à mão).
#[derive(Deserialize)]
struct Gravado {
    ops: Vec<FxOp>,
    #[serde(default)]
    ids: Vec<ItemId>,
}

impl From<Gravado> for VecFilter {
    fn from(g: Gravado) -> Self {
        let mut ids = g.ids;
        ids.resize(g.ops.len(), ItemId::NONE);
        Self { ops: g.ops, ids }
    }
}

impl SimComponent for VecFilter {}

impl crate::item_id::ComItens for VecFilter {
    fn ids(&self) -> Vec<ItemId> {
        self.ids.clone()
    }
    fn com_ids(&mut self, ids: &[ItemId]) {
        for (x, id) in self.ids.iter_mut().zip(ids) {
            *x = *id;
        }
    }
}

impl VecFilter {
    /// O teto de degraus numa pilha.
    ///
    /// **O recurso que aperta é a TELA do painel, não a GPU** — e isso está MEDIDO, não suposto
    /// (`ph2d-render/tests/it/fx_stack_gpu.rs::the_cost_of_a_stack_is_linear_in_the_number_of_ops`,
    /// RTX, 512×512, sigma 8 px): `0 degraus 0,082 ms · 1 → 0,084 · 2 → 0,149 · 3 → 0,220 ·
    /// 4 → 0,336 · 6 → **0,429 ms**`. O custo é linear, ~0,07 ms por degrau, e uma pilha CHEIA
    /// custa **2,6 % de um frame de 60 fps**. Cada degrau, em compensação, é um card de 4-6 linhas
    /// no painel — seis já enchem a coluna. É a mesma razão do `MAX_PATH_EFFECTS` da pilha de
    /// geometria, com o número um degrau acima porque aqui o card é mais raso.
    pub const MAX_OPS: usize = 6;

    /// Uma pilha com um degrau — o que o 1º "Add" produz.
    #[must_use]
    pub fn single(op: FxOp) -> Self {
        Self::from_ops(vec![op])
    }

    /// Uma pilha com estes degraus, ids por atribuir.
    #[must_use]
    pub fn from_ops(ops: Vec<FxOp>) -> Self {
        let ids = vec![ItemId::NONE; ops.len()];
        Self { ops, ids }
    }

    /// Os degraus, na ordem em que se aplicam.
    #[must_use]
    pub fn ops(&self) -> &[FxOp] {
        &self.ops
    }

    /// Os valores dos degraus (a estrutura só muda pelos métodos abaixo).
    pub fn ops_mut(&mut self) -> &mut [FxOp] {
        &mut self.ops
    }

    /// O id de cada degrau, na ordem de [`Self::ops`].
    #[must_use]
    pub fn ids(&self) -> &[ItemId] {
        &self.ids
    }

    /// A posição do degrau com o id `id` (`NONE` nunca casa).
    #[must_use]
    pub fn posicao(&self, id: ItemId) -> Option<usize> {
        if id.is_none() {
            return None;
        }
        self.ids.iter().position(|x| *x == id)
    }

    /// Acrescenta um degrau no fim (id por atribuir).
    pub fn push(&mut self, op: FxOp) {
        self.ops.push(op);
        self.ids.push(ItemId::NONE);
    }

    /// Insere um degrau em `at` (id por atribuir). Pânico se `at > len`, como o `Vec`.
    pub fn insert(&mut self, at: usize, op: FxOp) {
        self.ops.insert(at, op);
        self.ids.insert(at, ItemId::NONE);
    }

    /// Tira o degrau `row` (com o id dele). `None` fora da faixa.
    pub fn remove(&mut self, row: usize) -> Option<FxOp> {
        if row >= self.ops.len() {
            return None;
        }
        self.ids.remove(row);
        Some(self.ops.remove(row))
    }

    /// Põe os valores de `ops` POR POSIÇÃO (a pose de um estado de UI): a posição que fica guarda
    /// o id; a que sobra sai com o dele; a que entra nasce por atribuir. Sem mudança, nada se escreve.
    pub fn troca_ops(&mut self, ops: &[FxOp]) {
        if self.ops.as_slice() == ops {
            return;
        }
        self.ops.clear();
        self.ops.extend_from_slice(ops);
        self.ids.resize(ops.len(), ItemId::NONE);
    }

    /// Quantos degraus.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Sem degraus?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
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
        self.ids.swap(row - 1, row);
        true
    }

    /// Troca `row` com o vizinho de baixo. `false` (e nada muda) na última linha.
    pub fn move_down(&mut self, row: usize) -> bool {
        if row + 1 >= self.ops.len() {
            return false;
        }
        self.ops.swap(row, row + 1);
        self.ids.swap(row, row + 1);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(kinds: &[u8]) -> VecFilter {
        VecFilter::from_ops(kinds.iter().map(|k| FxOp::new(*k)).collect())
    }

    /// **Reordenar troca DOIS vizinhos, e as pontas são no-ops** — subir na primeira linha e descer
    /// na última não fazem nada, e o painel nem desenha essas setas. Aqui prova-se que, mesmo se
    /// alguém as despachasse, a pilha não se deforma (um `swap` fora de faixa entraria em pânico).
    #[test]
    fn reordering_swaps_neighbours_and_the_ends_are_no_ops() {
        let mut f = stack(&[FxOp::BLUR, FxOp::GLOW, FxOp::DROP_SHADOW]);
        assert!(f.move_down(0));
        assert_eq!(
            f.ops().iter().map(|o| o.kind).collect::<Vec<_>>(),
            vec![FxOp::GLOW, FxOp::BLUR, FxOp::DROP_SHADOW]
        );
        assert!(f.move_up(2));
        assert_eq!(
            f.ops().iter().map(|o| o.kind).collect::<Vec<_>>(),
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
        f.ops_mut()[0].enabled = false;
        assert!(f.is_active(), "um degrau ligado basta");
        f.ops_mut()[1].enabled = false;
        assert!(!f.is_active(), "toda desligada é o mesmo que vazia");
    }

    /// A pose de um estado de UI põe os valores POR POSIÇÃO: quem fica guarda o id (a chave de
    /// animação não muda a cada Show), o que sobra sai, o que entra espera o passe.
    #[test]
    fn a_pose_troca_os_valores_e_guarda_os_ids_por_posicao() {
        use crate::item_id::ComItens;
        let mut f = stack(&[FxOp::BLUR, FxOp::GLOW]);
        f.com_ids(&[ItemId(3), ItemId(4)]);
        let mut pose = f.ops().to_vec();
        pose[1].radius = 9.0;
        pose.push(FxOp::new(FxOp::BEVEL));
        f.troca_ops(&pose);
        assert_eq!(f.ops(), pose.as_slice());
        assert_eq!(f.ids(), &[ItemId(3), ItemId(4), ItemId::NONE]);
        f.troca_ops(&pose[..1]);
        assert_eq!(f.ids(), &[ItemId(3)]);
    }

    /// O teto é respondido pela pilha, não contado no chamador.
    #[test]
    fn the_ceiling_is_the_stacks_own_answer() {
        let mut f = VecFilter::default();
        while f.has_room() {
            f.push(FxOp::new(FxOp::BLUR));
        }
        assert_eq!(f.len(), VecFilter::MAX_OPS);
    }
}
