//! **O vínculo de um texto a um caminho-guia** — [`VecTextPath`].
//!
//! Presença = o texto **cavalga** um caminho; ausência = texto reto, que é o caminho de hoje
//! e é **byte-idêntico por construção** (não por um `None` que alguém pode esquecer de ler).
//!
//! # Por que um componente OPCIONAL e não campos em `VecTextParams`
//!
//! O plano previa apender `on_path`/`start_offset`/`flip` ao `VecTextParams`. Isso está errado
//! e o preço é grande: o blob de um componente é postcard **posicional**, então apender campo
//! obriga a bumpar o `PROJECT_SCHEMA` — e **um bump RECUSA todo projeto já salvo**. Um
//! componente NOVO cunha a própria blob-key (`stable_type_id`) e **não move nada**: é o mesmo
//! raciocínio que fez o `PhysicsJoint` não bumpar e o `GravityScale` nascer opcional em vez de
//! campo do `RigidBody`.
//!
//! Fica ainda melhor de graça: *"texto reto é o que não tem o componente"* é uma afirmação que
//! o compilador ajuda a manter, enquanto *"texto reto é `on_path: None`"* é uma que cada leitor
//! tem de lembrar.
//!
//! # O texto vinculado vive na IDENTIDADE, como o conector
//!
//! A geometria de um texto em caminho é **MUNDO** (o caminho já traz a pose dele), então uma
//! pose por cima a deslocaria. O `connector_live` já estabeleceu o precedente e o escreveu:
//!
//! > *"Ele vive na identidade, e é isso que o torna (corretamente) não-arrastável pelo gizmo:
//! > arrastar um conector não quer dizer nada."*
//!
//! Vale palavra por palavra aqui: **mover um texto em caminho não quer dizer nada — o que se
//! move é o caminho.** É também o que o Illustrator faz (o texto e o caminho são um objeto só).

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// O vínculo texto → caminho-guia.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecTextPath {
    /// O caminho-guia. É o `ph2d_vec_scene::VecPathId`, que é um `u64` — o ECS não depende do
    /// vetor (a mesma razão pela qual `VecTextParams::align` é um `u8` e não um `TextAlign`).
    pub path: u64,
    /// Onde a 1ª linha começa, em **fração do comprimento total** do caminho.
    ///
    /// Fração e não distância absoluta porque é o idioma do `startOffset` do SVG *e* porque é o
    /// que sobrevive a editar o caminho: esticar a curva mantém o texto onde o artista o pôs,
    /// em vez de o deixar para trás.
    pub start_offset: f32,
    /// Texto do outro lado do caminho, a ler no sentido oposto.
    pub flip: bool,
}

impl SimComponent for VecTextPath {}
