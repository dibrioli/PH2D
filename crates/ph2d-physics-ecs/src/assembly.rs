//! [`JointAssembly`] — **uma montagem é um OBJETO** (ADR-0201): «Car 1» é uma entidade nomeada, na
//! Hierarchy, que diz de que gabarito veio e que objeto ocupa cada papel. As juntas apontam para
//! ela ([`crate::PhysicsJoint::assembly`]); ela nunca aponta para as juntas (uma fonte só).
//!
//! O gabarito em si (papéis, regras, números de fábrica) é dado puro em `ph2d-joint-templates`;
//! aqui só viaja o `id` dele, pinado lá.

use ph2d_ecs::{Component, SimComponent};
use serde::{Deserialize, Serialize};

/// Um papel ocupado. `body == 0` = **o papel é o MUNDO** (o batente da porta pregado no cenário) —
/// só vale nos papéis que o gabarito declara `may_be_world`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssemblyPart {
    /// O índice do papel no gabarito (`Template::roles[role]`).
    pub role: u16,
    /// O `StableId` do objeto. Num papel de vários (as tábuas da ponte) há uma entrada por objeto,
    /// na ordem da corrente.
    pub body: u64,
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JointAssembly {
    /// O `TemplateId` do gabarito (pinado em `ph2d-joint-templates`, append-only).
    pub template: u16,
    pub parts: Vec<AssemblyPart>,
    /// Bit `g` ligado = as juntas do grupo `g` do gabarito são **iguais entre si** (editar uma
    /// edita todas — «Same as the rear wheel»). Nasce tudo ligado.
    pub linked: u32,
}

impl Default for JointAssembly {
    fn default() -> Self {
        Self {
            template: 0,
            parts: Vec::new(),
            linked: u32::MAX,
        }
    }
}

impl JointAssembly {
    /// Os objetos de um papel, na ordem em que entraram.
    pub fn bodies_of(&self, role: u16) -> impl Iterator<Item = u64> + '_ {
        self.parts
            .iter()
            .filter(move |p| p.role == role)
            .map(|p| p.body)
    }

    #[must_use]
    pub fn is_linked(&self, group: u8) -> bool {
        group < 32 && self.linked & (1 << group) != 0
    }
}

impl SimComponent for JointAssembly {}
