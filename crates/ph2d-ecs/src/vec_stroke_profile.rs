//! **A LARGURA VIVA** de um traço — o perfil que varia ao longo do caminho, sem que a curva
//! autorada mude (ADR-0148).
//!
//! Irmão do [`crate::VecOffset`] no padrão que esta linha já usou várias vezes: o componente
//! guarda a **relação** (que largura, onde) e a aparência é uma **função pura** dela, re-cozida
//! pela shell a cada frame. Consequência de graça: **undo e save cobrem o perfil sem uma linha a
//! mais** — os dois capturam o mundo ECS, e este componente está no `ComponentRegistry`.
//!
//! # Por que um componente, e não um campo do `StrokeSpec`
//!
//! O `StrokeSpec` é `Serialize` e a `VecScene` viaja EMBUTIDA no `ProjectState`, então um campo
//! novo ali bumparia `VEC_SCENE_SCHEMA_VERSION` **e** `PROJECT_SCHEMA` — e um schema divergente
//! **RECUSA o arquivo inteiro**, isto é, todo projeto já salvo. Um componente cunha
//! `stable_type_id = blake3(NOME)[..8]` próprio e não move o layout posicional de nada: **zero
//! bump**. É a mesma conta que decidiu os sete componentes vivos anteriores deste módulo, e a
//! razão está escrita em cada um: *jogar fora trabalho real para evitar um componente é o trade
//! errado*.
//!
//! # O que ele guarda é a LISTA, e a lista tem uma casa só
//!
//! [`ph2d_stroke_width::WidthStops`] — `(posição de arco, multiplicador)`. O preset de quatro
//! números que o painel desenha é uma **face** dela ([`ph2d_stroke_width::WidthProfile`]), e é
//! a face que CONSTRÓI a lista; nada lê as duas tentando reconciliá-las. A casa é uma folha
//! porque este crate (fundação) e a `ph2d-vec-scene` (o motor) não se veem — o precedente
//! exato da [`ph2d_warp_style`].
//!
//! # Multiplicadores, e o neutro é a AUSÊNCIA
//!
//! A largura absoluta continua sendo a do `StrokeSpec`; o perfil diz o que acontece com ELA. Um
//! perfil uniforme não é guardado: a shell **REMOVE** o componente (a mesma lei do `VecOffset`
//! com `d = 0`), senão um documento acumularia relações invisíveis que não desenham nada.

use bevy_ecs::component::Component;
use ph2d_stroke_width::WidthStops;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **O perfil de largura vivo de uma forma.** A entidade que o carrega também tem um
/// [`crate::VecPathRef`]: o `VecPath` dela continua sendo a curva **AUTORADA** (é ela que o modo
/// Node edita), e a fita de largura variável é DESENHO — a shell a coze por frame e o passe de
/// render a desenha no lugar da fonte, no z dela.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VecStrokeProfile {
    /// As paradas, em fração de arco. Vazia não acontece: a shell remove o componente em vez de
    /// guardar um perfil que não desenha nada.
    pub stops: WidthStops,
}

impl SimComponent for VecStrokeProfile {}
