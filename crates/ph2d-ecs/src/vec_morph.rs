//! **O Morph vivo** — a forma única que É o caminho entre duas outras, parada num `t`.
//!
//! O irmão animável do [`crate::VecBlend`], e a diferença entre os dois é o que cada um entrega:
//! o Blend mostra os **N passos de uma vez** (o Blend do Illustrator, uma transição vista de fora);
//! o Morph mostra **UM**, e o `t` dele é um número que a timeline keya. Blend é ilustração; Morph
//! é animação.
//!
//! Espelha o padrão do [`crate::VecConnector`]: o componente guarda a **relação** (quais duas
//! formas, e onde no caminho), e a aparência é **função pura** dela, re-cozida a cada frame pela
//! shell. Ninguém desenha um morph — move-se uma forma-fonte, ou mexe-se no `t`, e a forma se
//! refaz.
//!
//! Consequência de graça (a mesma do conector e do blend): **undo e save cobrem o morph sem uma
//! linha a mais** — os dois capturam o mundo ECS + a cena vetorial, e este componente está
//! registrado no `ComponentRegistry`.
//!
//! # As fontes são `VecPathId`, nunca bits de entidade
//!
//! `Entity::to_bits()` é um id de **alocação**, e o undo restaura o mundo **respawnando** as
//! entidades — com bits novos. Guardar as fontes por bits significaria que o usuário desfaz
//! qualquer coisa e todo morph se solta das formas dele. (Mesma razão, palavra por palavra, do
//! [`crate::VecBlend`] e do [`crate::VecConnector`].)

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **O Morph Object.** A entidade que o carrega também tem um [`crate::VecPathRef`], e o `VecPath`
/// dela é a **forma morfada** — geometria de verdade, na cena, re-escrita *em lugar* a cada frame.
///
/// É aqui que ele difere do [`crate::VecBlend`], cujo `VecPath` é o *spine* invisível e cujos
/// passos são overlay virtual: o morph é UMA forma, e ela está na cena porque é ela que o artista
/// quer — para pintar, exportar, animar.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecMorph {
    /// Os `VecPathId` das duas formas-fonte, na ordem `A → B`. Um id que não resolve (forma
    /// apagada) **congela** a forma onde ela está, em vez de a fazer sumir — a mesma escolha do
    /// conector, e a única que preserva o trabalho do artista.
    pub sources: [u64; 2],
    /// **Onde no caminho.** `0` = a forma A; `1` = a forma B; `0,5` = o meio. É este número que a
    /// timeline keya (`PropKind::Morph`), e é o motivo de o Morph existir.
    ///
    /// Fora de `[0, 1]` o motor faz clamp ([`ph2d_vec_blend::Plan::at`]), então uma curva animada
    /// com overshoot (um ease com back/elastic) não quebra a forma — ela encosta na ponta e para.
    pub t: f32,
}

impl SimComponent for VecMorph {}

impl VecMorph {
    /// Um morph novo entre `a` e `b`, nascendo no meio do caminho.
    ///
    /// **No meio, e não em `0`**: um morph que nasce em `t=0` é uma cópia exata da forma A, em cima
    /// da forma A — o artista clica "Morph" e não vê nada acontecer. Nascendo no meio, o objeto
    /// novo **se anuncia**: é uma forma que ele nunca desenhou, entre as duas que ele escolheu.
    #[must_use]
    pub fn new(a: u64, b: u64) -> Self {
        Self {
            sources: [a, b],
            t: 0.5,
        }
    }
}
