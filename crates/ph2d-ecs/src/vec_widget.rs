//! **A forma VESTE um widget** — o degrau 2 do §2 do plano UI/UX (W6.2).
//!
//! Irmão de [`crate::VecOffset`] / [`crate::VecTextPath`] / [`crate::VecFilter`] no padrão que esta
//! linha já usou muitas vezes: o componente guarda a **relação** (*esta forma é um botão*) e a
//! aparência é uma **função pura** dela, re-cozida pela shell a cada frame. Consequência de graça:
//! **undo e save cobrem a pele sem uma linha a mais** — os dois capturam o mundo ECS, e este
//! componente está registrado no `ComponentRegistry`.
//!
//! # UM campo, e a razão é uma lei desta casa
//!
//! O plano escrevia `{ kind: u16, key: String }`. O `key` **não existe aqui**: o rótulo que o
//! widget mostra é o **`Name` da entidade** — o nome que o artista já digita na Hierarquia. Um
//! segundo lugar para digitar o nome de uma coisa é um segundo lugar que discorda do primeiro, e a
//! W5c acabou de usar exactamente esta porta para os eixos de variant.
//!
//! # `u16` e não um enum, e isso é o canal de compatibilidade
//!
//! O catálogo de widgets vive na `ph2d-editor-core`, que esta crate **não alcança** (seria a seta
//! errada: o modelo de mundo não pode depender da UI). Mas a razão principal é outra e é uma
//! CAPACIDADE: um documento autorado por um build mais novo carrega um código que este não conhece,
//! e a resposta certa é **desenhar a forma** — nunca recusar o arquivo, nunca pintar um retângulo
//! vazio. `WidgetKind::from_code` devolve `None`, e o `None` é o caminho suportado.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **Que widget do catálogo esta forma veste.**
///
/// A entidade que o carrega também tem um [`crate::VecPathRef`]: o `VecPath` dela continua sendo a
/// geometria **AUTORADA** (é ela que o modo Node edita, que o gizmo move e que o hit-test pega), e
/// o widget é DESENHO — a shell o pinta no lugar da forma, no z dela.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecWidget {
    /// O código do tipo, estável para sempre (`ph2d_editor_core::widget::WidgetKind::code`).
    ///
    /// ⚠️ Nunca a ordem de um enum: o número viaja no documento, e reordenar o catálogo
    /// re-pintaria em silêncio toda arte já salva.
    pub kind: u16,
}

impl SimComponent for VecWidget {}
