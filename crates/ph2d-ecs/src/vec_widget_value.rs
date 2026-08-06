//! **A POSIÇÃO de um controle autorado** — onde o slider está, se o toggle está ligado
//! (plano UI/UX W8b.4).
//!
//! A W8b.3 fez a row DIRIGIR a arte e nomeou o próprio buraco: o valor vivia só no `WidgetStore`,
//! que é de runtime ⇒ o artista punha a forma a 30%, salvava, reabria, e **a arte voltava a
//! 100%**. Não é um luxo perdido: é o documento não fazendo round-trip visual.
//!
//! # Um número, e o que ele NÃO é
//!
//! ⚠️ **Isto não é a arte** — a tinta autorada continua intocada, e o que este componente guarda é
//! *onde o CONTROLE está*. São dois fatos diferentes e é isso que mantém a lei da W8b.3 de pé
//! (*a row modula a VISTA; o documento é do artista*): a vista é derivada de um controle cuja
//! posição, essa sim, é autorada.
//!
//! ⚠️ **E ele existe mesmo sem vínculo:** um slider que o artista ainda não prendeu a forma nenhuma
//! tem posição, e perdê-la seria perder trabalho que ele fez. É por isso que este componente é
//! irmão do [`crate::VecWidgetBind`] e não um campo dele.
//!
//! # Por que componente, e não um campo do `VecWidget`
//!
//! Blob-key própria (`stable_type_id` do NOME) ⇒ **zero bump de `PROJECT_SCHEMA`**, e um controle
//! que nunca foi tocado simplesmente não tem o componente — o precedente exato do
//! [`crate::VecWidgetBind`] e do [`crate::VecCutPath`].

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **A posição autorada deste controle.**
///
/// O número é o que o tipo do widget carrega, normalizado: um slider guarda `0..=1`, um toggle e
/// um checkbox guardam `0` ou `1`. ⚠️ Quem traduz é a porta única do lado da shell
/// (`vec_widget_value::{value_of, seed_state}`), e as duas metades **têm de fazer round-trip** —
/// uma tradução que não volta é um controle que muda de posição sozinho ao reabrir o arquivo.
#[derive(Component, Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct VecWidgetValue {
    /// A posição, na unidade do tipo.
    pub value: f32,
}

impl SimComponent for VecWidgetValue {}
