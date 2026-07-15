//! **O Blend Object vivo** (ADR-0122) — o objeto único que interpola 2..=5 formas.
//!
//! Espelha o padrão do [`crate::VecConnector`] e da *Live Shape* ([`crate::VecShape`]): o
//! componente guarda a **relação** (quais formas, na ordem, e quantos passos entre cada par); a
//! aparência — os passos intermediários — é uma **função pura** dela, re-cozida a cada frame pela
//! shell. Ninguém "desenha" um passo do blend: move-se uma forma-fonte, e a transição se refaz.
//!
//! Consequência de graça (a mesma do conector): **undo e save cobrem o blend sem uma linha a
//! mais** — os dois capturam o mundo ECS + a cena vetorial, e este componente está registrado no
//! `ComponentRegistry`.
//!
//! # A decisão que não é de gosto: as fontes são `VecPathId`, nunca bits de entidade
//!
//! `Entity::to_bits()` é um id de **alocação**. O undo restaura o mundo **respawnando** as
//! entidades — com bits novos. Guardar as fontes por bits significaria que o usuário desfaz
//! qualquer coisa e **todo blend se solta das formas dele**. O `VecPathId` do documento vetorial
//! sobrevive ao clone (undo) e ao postcard (save) — é a única identidade estável que existe hoje.
//! (Mesma razão, palavra por palavra, do [`crate::VecConnector`].)

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **O Blend Object.** A entidade que o carrega também tem um [`crate::VecPathRef`]: o componente
/// é a relação, e o `VecPath` dela é o **spine** (a linha que une as fontes, editável no modo
/// Node — ADR-0122). Os passos intermediários NÃO entram na cena: são desenho, gerado por um
/// passe de render que lê este componente. É o que torna o blend **um objeto**, e não N.
#[derive(Component, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecBlend {
    /// Os `VecPathId` das formas-fonte, **na ordem da cadeia** (a ordem de z, como o "Make" do
    /// Illustrator). A linha aceita `2..=5`. Um id que não resolve (forma apagada) é **pulado**
    /// no recook — a cadeia não quebra por causa de um elo.
    pub sources: Vec<u64>,
    /// Passos intermediários **por elo** (entre cada par consecutivo). `chain` liga
    /// `fonte[0]→fonte[1]`, `fonte[1]→fonte[2]`, … com `steps` passos em cada par. Não para em 12
    /// (o teto do Illustrator) — vai a centenas; o clamp mora no painel (ADR-0122, Fase C).
    pub steps: u32,
}

impl SimComponent for VecBlend {}

impl VecBlend {
    /// Um blend novo sobre `sources` (na ordem de z), com `steps` passos por elo.
    #[must_use]
    pub fn new(sources: Vec<u64>, steps: u32) -> Self {
        Self { sources, steps }
    }
}
