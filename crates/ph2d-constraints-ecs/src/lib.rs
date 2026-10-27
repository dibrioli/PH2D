#![forbid(unsafe_code)]
//! **A pilha de constraints de uma entidade** — objecto ou osso (o osso é uma entidade com
//! `Transform`). O componente [`Constraints`] grava a pilha; [`evaluate`] corre uma vez por quadro e
//! escreve a pose pelo `PreviewDrive` (`Driver::ConstraintPose`): *o que a pilha escreve vê-se, não
//! se guarda*. A lei de cada tipo é da `ph2d-constraints`.

mod avalia;
pub mod cola;
mod curva;
mod ik;
pub mod ik_item;
mod ik_novo;
mod ordem;
pub mod relogio;

pub use avalia::{Adotar, Caminhos, Estado, Estados, Pontos, com_autorado, evaluate};
pub use cola::{COLAS_TYPE_NAME, Colas, EstadosDaCola, colar};
pub use ik_item::{
    Ancora, ancora_de, ancoras, com_ancora, com_ik, converter_ik_de_um_osso, converter_ik_goals,
    ik_de, iks, por_ik, tirar_ancora, tirar_ik,
};
pub use ik_novo::{add_ik, add_look_at, aponta};
pub use ordem::ordem;
pub use relogio::{Accoes, Amostra, Laco, Molas, Simulador};

use bevy_ecs::component::Component;
use ph2d_constraints::Kind;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{SimComponent, StableId};
use serde::{Deserialize, Serialize};

/// O nome canónico gravado (o `stable_type_id` sai dele — nunca o mude).
pub const CONSTRAINTS_TYPE_NAME: &str = "ph2d::constraints::Constraints";

/// Um item da pilha.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// Id estável dentro da pilha, nunca reusado: é o endereço do canal animável e sobrevive a
    /// reordenar.
    pub id: u32,
    pub kind: Kind,
    /// O alvo por identidade durável (sobrevive a gravar, desfazer e instanciar).
    pub target: Option<StableId>,
    /// A força, `0..=1`.
    pub influence: f64,
    pub enabled: bool,
}

/// A pilha, de cima para baixo — a ordem é a ordem do cálculo, sem excepções.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Constraints {
    pub items: Vec<Item>,
    /// O próximo [`Item::id`]; só sobe.
    pub next_id: u32,
}

impl SimComponent for Constraints {}

impl Constraints {
    /// Acrescenta no FIM e devolve o id. O offset (se o tipo o guarda ao nascer) é do chamador, que
    /// conhece as poses — [`avalia::bind_on_create`].
    pub fn push(&mut self, kind: Kind, target: Option<StableId>) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.items.push(Item {
            id,
            kind,
            target,
            influence: 1.0,
            enabled: true,
        });
        id
    }

    #[must_use]
    pub fn item(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn item_mut(&mut self, id: u32) -> Option<&mut Item> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    /// Move o item `id` para a posição `to` (aparada ao fim). `false` se não existe.
    pub fn move_to(&mut self, id: u32, to: usize) -> bool {
        let Some(from) = self.items.iter().position(|i| i.id == id) else {
            return false;
        };
        let it = self.items.remove(from);
        self.items.insert(to.min(self.items.len()), it);
        true
    }

    pub fn remove(&mut self, id: u32) -> bool {
        let n = self.items.len();
        self.items.retain(|i| i.id != id);
        self.items.len() != n
    }
}

pub use avalia::bind_on_create;

/// Regista os componentes desta crate. Sem isto o `WorldSnapshot` (undo + gravar) descarta a pilha
/// em silêncio.
pub fn register_constraint_components(reg: &mut ComponentRegistry) {
    reg.register_default::<Constraints>(CONSTRAINTS_TYPE_NAME);
    reg.register_default::<Colas>(COLAS_TYPE_NAME);
}
