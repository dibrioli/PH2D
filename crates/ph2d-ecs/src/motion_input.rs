//! ⭐ **O componente `Motion Input`** (ciclo 19, doc 128 §3.6) — um objecto que publica uma entrada
//! para o grafo do Motion (o `value.input` com o mesmo nome a lê).
//!
//! É CONFIG (registado, gravado, com override por instância e variante pelo sistema de
//! componentes). O valor de cada quadro vai para o recurso [`crate::MotionInputs`], pela ponte da
//! `ph2d-app-components`.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// **O que o objecto publica.** ⚠️ A ordem é a do seletor do Inspector; acrescentar é no fim.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionSource {
    /// O número do campo `value`.
    #[default]
    Value,
    /// A cor do campo `color`.
    Color,
    /// A vida do objecto, de `0` (morto) a `1` (cheio) — o objecto precisa de `Health`.
    Health,
    /// Um DISPARO cada vez que a vida desce (um gatilho).
    Damage,
    /// O módulo da velocidade do corpo do objecto From, em unidades do mundo por segundo.
    Speed,
    /// `1` no chão, `0` no ar — o personagem do objecto From.
    Grounded,
    /// A força `0..1` da acção `action` do Input Map (a mesma leitura do Luau `action_strength`).
    Action,
    /// Um campo numérico (`property` = `canonical/serde_key`) de um componente do objecto From.
    Property,
}

impl MotionSource {
    pub const ALL: [Self; 8] = [
        Self::Value,
        Self::Color,
        Self::Health,
        Self::Damage,
        Self::Speed,
        Self::Grounded,
        Self::Action,
        Self::Property,
    ];

    #[must_use]
    pub fn index(self) -> u8 {
        match self {
            Self::Value => 0,
            Self::Color => 1,
            Self::Health => 2,
            Self::Damage => 3,
            Self::Speed => 4,
            Self::Grounded => 5,
            Self::Action => 6,
            Self::Property => 7,
        }
    }
}

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MotionInput {
    /// O nome da entrada (o do `value.input`).
    pub name: String,
    pub source: MotionSource,
    pub value: f32,
    /// RGBA linear.
    pub color: [f32; 4],
    /// De que objecto ler (Health, Damage, Speed, Grounded, Property), pelo `StableId`; `0` = este.
    pub from: u64,
    /// O nome da acção do Input Map (fonte `Action`).
    pub action: String,
    /// `"<canonical>/<serde_key>"` do campo lido (fonte `Property`).
    pub property: String,
    /// Leva o número de `remap_from..remap_to` a `0..1` (com clamp), e depois a `curve`.
    pub remap: bool,
    pub remap_from: f32,
    pub remap_to: f32,
    /// Texto de `ph2d-curve` aplicado depois do remap; `""` = linear.
    pub curve: String,
    /// Meia-vida da suavização, em segundos (a lei do `value.follow` Smooth); `0` = desligada.
    pub smooth: f32,
}

impl Default for MotionInput {
    fn default() -> Self {
        Self {
            name: "input".to_string(),
            source: MotionSource::Value,
            value: 0.0,
            color: [1.0; 4],
            from: 0,
            action: String::new(),
            property: String::new(),
            remap: false,
            remap_from: 0.0,
            remap_to: 1.0,
            curve: String::new(),
            smooth: 0.0,
        }
    }
}

/// O dono de uma entrada (MI-10): o PAI do objeto «Motion Input»; na raiz, o próprio.
#[must_use]
pub fn dono(
    world: &bevy_ecs::world::World,
    e: bevy_ecs::entity::Entity,
) -> bevy_ecs::entity::Entity {
    world
        .get::<crate::ChildOf>(e)
        .map_or(e, crate::ChildOf::parent)
}

/// O rótulo da linha da Hierarquia: `«<base> · <nome>»` (`base` = «Motion Input», traduzido por quem chama).
#[must_use]
pub fn rotulo(base: &str, nome: &str) -> String {
    format!("{base} \u{b7} {}", nome.trim())
}

/// A linha da Hierarquia segue o nome da entrada: todo objeto com `MotionInput` cujo `Name` ainda começa
/// por `base` (o artista não o renomeou) passa a `rotulo(base, nome)`. Só escreve quando muda.
pub fn rotula_entradas(world: &mut bevy_ecs::world::World, base: &str) {
    let mut q = world.query::<(bevy_ecs::entity::Entity, &MotionInput, Option<&crate::Name>)>();
    let trocar: Vec<_> = q
        .iter(world)
        .filter_map(|(e, m, n)| {
            let quer = rotulo(base, &m.name);
            let atual = n.map_or("", |n| n.as_str());
            (atual != quer && (atual.is_empty() || atual.starts_with(base))).then_some((e, quer))
        })
        .collect();
    for (e, quer) in trocar {
        world.entity_mut(e).insert(crate::Name::new(quer));
    }
}
