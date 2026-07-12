//! `PaintedDoc` — a entidade tem um **documento do Painter** (camadas, pixels, relevo).
//!
//! O Painter é dono do documento (`PainterTool`: `layers` + `images` + `heights` + `covers`); esta
//! componente é a única ponte, e carrega só a **identidade estável** dele.
//!
//! Por que ela existe: o Painter chaveia seus documentos por `Entity::to_bits()`, que é o id de
//! ALOCAÇÃO do ECS — ele muda a cada spawn, então um save/load (que despawna tudo e recria) deixa
//! aquela chave morta. Sem uma identidade que sobreviva ao restore, não há como devolver a um sprite
//! o documento que era dele: as camadas, o relevo e a possibilidade de continuar editando iriam
//! embora, e o que restaria seria um bake achatado.
//!
//! O `u32` é caller-supplied (a shell aloca o próximo livre), no mesmo espírito da célula de atlas —
//! e, como toda componente, viaja no `WorldSnapshot`, logo o **undo** também a preserva de graça.
//!
//! O que ela NÃO faz: não põe pixels no ECS. O `PainterTool` continua dono, e a shell mantém o ciclo
//! de vida (documento ⇒ id; sprite carregado ⇒ documento re-instalado).

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Identidade estável do documento do Painter desta entidade.
///
/// Um `u32` cru (e não um tipo do `ph2d-tool-painter`) para manter `ph2d-ecs` sem dependência da
/// ferramenta — a direção da seta importa: a shell conhece os dois, nenhum dos dois conhece o outro.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaintedDoc(pub u32);

impl SimComponent for PaintedDoc {}
