//! **A orientação do motivo sobre a curva** — [`VecPatternRotation`], o par opcional do
//! [`crate::VecPatternPath`] (Pattern Along Path, plano 23).
//!
//! Gira cada cópia dentro do referencial da guia: `90` põe o motivo de pé, atravessado na curva.
//! É **constante** em todas as cópias e **relativa à tangente**, não ao mundo — numa curva as
//! cópias continuam a acompanhar a guia e só ganham uma orientação fixa em cima dela. O giro que
//! **progride** de cópia para cópia (um leque) é outra coisa e tem outro dono: o Repeater
//! (`fx_repeat`), dirigido por índice e não pela guia.
//!
//! # Por que um componente SEPARADO, e não mais um campo no `VecPatternPath`
//!
//! Porque o blob de um componente é postcard **POSICIONAL**: apender um campo ao
//! [`crate::VecPatternPath`] mudaria o layout dele e obrigaria a bumpar o `PROJECT_SCHEMA` — e um
//! bump **RECUSA todo projeto já salvo**. Um componente NOVO cunha a própria blob-key
//! (`stable_type_id`) e **não move nada**.
//!
//! Este é o critério que a `line/physics` fixou depois de o pagar: a W-Offset apendou campo ao
//! `Collider` e bumpou 28→29, e as três waves seguintes da MESMA área (`AreaDrag`, `AreaBuoyancy`,
//! `AreaTorque`) reverteram o critério e cada uma nasceu componente próprio, com o mesmo racional
//! escrito nas três — *"jogar fora trabalho real para evitar um 2º componente é o trade errado"*.
//! O wrapper de runtime junta os dois num `PatternSpec` (que não é serializado); o ECS os mantém
//! separados. Aqui é a mesma forma, pela mesma razão.
//!
//! # Ausência É "sem rotação"
//!
//! O componente só existe quando o artista autorou um ângulo ≠ 0 (a ponte o **destaca** no
//! neutro). Isso torna *"um pattern sem este componente é um pattern não-girado"* um fato que o
//! compilador ajuda a manter, e faz todo documento salvo antes desta wave carregar **inalterado**
//! — não há campo em falta a inventar no load.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A orientação do motivo sobre a guia, em **GRAUS**.
///
/// A unidade está no nome porque um ângulo sem unidade declarada é o bug que não dá erro em lado
/// nenhum: a fronteira do painel fala graus, o motor converte para o rotor uma única vez por
/// re-cook, e nenhum dos dois precisa lembrar qual é qual.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecPatternRotation(pub f32);

impl SimComponent for VecPatternRotation {}

impl Default for VecPatternRotation {
    fn default() -> Self {
        Self(0.0)
    }
}
