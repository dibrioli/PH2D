//! **O ECO DE UMA LARGADA** — irmão do [`super`] pelo tecto de LOC (700) e por RESPONSABILIDADE:
//! ali mora o ESTADO do módulo, aqui uma grandeza transitória da interface com vida própria.

/// **O eco de uma largada, ANTES de virar intensidade** — *«Após a troca o conjunto linha e nó
/// piscam e se acentam»* (ordem do dono, 2026-09-19).
///
/// ⚠️ **Ele guarda o INSTANTE e não o `t`**, ao contrário do [`ph2d_panel_motion_graph::Piscada`]
/// que a shell publica: a conversão é feita uma vez por quadro, com o relógio de PAREDE
/// (`MotionState::ui_now`), e é ela que mantém o painel sem relógio nenhum. *Dois tipos porque
/// são duas perguntas — «quando começou» e «quanto resta».*
#[derive(Clone, Debug, PartialEq)]
pub struct PiscadaPendente {
    pub nos: Vec<u32>,
    pub fios: Vec<(u32, u16)>,
    pub inicio: f32,
}
