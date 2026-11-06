//! **A política de quem responde por um Ctrl+Z**, sem `App`: a shell só a consulta (`shells/desktop/src/undo_route.rs`).

/// **Quem responde por um Ctrl+Z** (ou pelo botão Undo/Redo da barra — são a MESMA
/// pergunta, e é por isso que a política mora numa função só).
///
/// O undo do editor não é um: são cinco. Um editor modal focado (o Audio, o Painter) é o
/// dono do seu próprio Ctrl+Z — se ele cedesse o atalho, um passo a mais **saltaria a cena
/// inteira** (o undo global restaura o `WorldSnapshot` e limpa a seleção), e a fronteira
/// entre as duas filas é invisível para o usuário. Foi assim que o Ctrl+Z do Áudio virou
/// "às vezes não funciona, às vezes faz coisa estranha" (auditoria 2026-07-11, A1).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UndoOwner {
    /// O Audio Editor, com o painel da onda aberto e um clipe carregado.
    Audio,
    /// Um QUADRO (MiroClone) na aba activa: o desfazer dele, por quadro.
    Board,
    /// O Painter (a fila de traços dele).
    Painter,
    /// O **Colorize do Flip com rabiscos pendentes** (7º smoke, "undo/redo ruim"): os
    /// rabiscos são SEMENTES transientes — não entram no `ProjectState` —, então sem este
    /// dono um Ctrl+Z no meio do rabisco caía no Global e **apagava a última edição REAL
    /// do documento** (o traço de line-art sumia enquanto os rabiscos ficavam intactos).
    /// Ctrl+Z remove o último rabisco; Ctrl+Shift+Z o devolve. Sem rabisco pendente (e sem
    /// rabisco removido, no redo) ele NÃO é dono — o atalho cai no Global, que é quem
    /// desfaz o Apply.
    Colorize,
    /// A fila GLOBAL de objetos/hierarquia/canvas (ADR-0110+). O caso comum.
    Global,
    /// O undo de image-edit (single-level: Trim / Make Square / Bg Removal). É o
    /// **fallback**, e é o que emite o toast "Nothing to undo" quando não há nada.
    ImageEdit,
    /// MiroClone B5(4) — a barra de ABAS: na cena, a última operação de abas (apagar, criar, renomear,
    /// arrastar um quadro) quando é mais recente que o último passo da cena.
    Tabs,
}

/// A política, sem `App`: quem responde, dado quem está aberto e quem tem passo.
///
/// Pura de propósito — é a única parte disto que dá para gatear sem uma janela, e é onde o
/// bug moraria (um dono a mais na cadeia e o atalho some).
#[must_use]
pub fn undo_owner(
    board_active: bool,
    audio_owns: bool,
    painter_active: bool,
    colorize_owns: bool,
    tabs_newer: bool,
    global_has: bool,
) -> UndoOwner {
    // Um QUADRO activo (MiroClone) desfaz-se a si mesmo — nunca a cena escondida por baixo
    // (plano §1.4).
    if board_active {
        UndoOwner::Board
    } else if audio_owns {
        UndoOwner::Audio
    } else if painter_active {
        UndoOwner::Painter
    } else if colorize_owns {
        UndoOwner::Colorize
    } else if tabs_newer {
        UndoOwner::Tabs
    } else if global_has {
        UndoOwner::Global
    } else {
        UndoOwner::ImageEdit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Quem responde por um Ctrl+Z — e pelo BOTÃO Undo/Redo, que é a mesma pergunta.**
    ///
    /// O botão da barra tinha caminho próprio: levantava `UndoImageEdit` (o desfazer de
    /// IMAGEM, single-level) enquanto o atalho desfazia o projeto — então mover uma forma e
    /// clicar em Undo não fazia nada. E o botão **Redo não despachava coisa alguma**. Os dois
    /// entram por esta política agora, e é por isso que ela existe separada: dois caminhos
    /// para a mesma pergunta divergem.
    #[test]
    fn a_focused_modal_editor_owns_the_undo_chord_and_the_button_agrees() {
        // O caso comum: o global, quando há passo.
        assert_eq!(
            undo_owner(false, false, false, false, false, true),
            UndoOwner::Global
        );
        // Sem passo global, cai no image-edit (que é quem emite "Nothing to undo").
        assert_eq!(
            undo_owner(false, false, false, false, false, false),
            UndoOwner::ImageEdit
        );
        // O Painter é dono da fila dele.
        assert_eq!(
            undo_owner(false, false, true, false, false, true),
            UndoOwner::Painter
        );
        // E o Audio ganha do Painter E do global: um editor modal FOCADO não pode ceder o
        // atalho — um passo a mais saltaria a cena inteira (auditoria 2026-07-11, A1).
        assert_eq!(
            undo_owner(false, true, true, false, false, true),
            UndoOwner::Audio
        );
        assert_eq!(
            undo_owner(false, true, false, false, false, false),
            UndoOwner::Audio
        );
    }

    /// **O Colorize com rabisco pendente é dono do Ctrl+Z** (7º smoke, "undo/redo ruim"):
    /// os rabiscos são transientes (fora do `ProjectState`), então sem este dono o atalho
    /// caía no Global e apagava a última edição REAL do documento — o artista rabiscava,
    /// errava, pedia Ctrl+Z e via o traço de LINE-ART sumir com os rabiscos intactos. Sem
    /// rabisco na direção pedida, ele cede — e o Global é quem desfaz o Apply.
    #[test]
    fn pending_scribbles_own_the_undo_chord_and_yield_when_empty() {
        assert_eq!(
            undo_owner(false, false, false, true, false, true),
            UndoOwner::Colorize
        );
        assert_eq!(
            undo_owner(false, false, false, true, false, false),
            UndoOwner::Colorize
        );
        // Vazio na direção pedida: cede ao Global (o Apply é dele).
        assert_eq!(
            undo_owner(false, false, false, false, false, true),
            UndoOwner::Global
        );
        // Um editor modal focado ainda ganha (o Colorize é modo de tool, não painel modal).
        assert_eq!(
            undo_owner(false, true, false, true, false, true),
            UndoOwner::Audio
        );
        assert_eq!(
            undo_owner(false, false, true, true, false, true),
            UndoOwner::Painter
        );
    }

    /// B5(4) — na cena, a operação de abas mais recente que o último passo dela responde (apagar o único
    /// quadro deixa a cena aberta, e o `Ctrl+Z` tem de o devolver); um editor modal focado continua dono.
    #[test]
    fn on_the_scene_a_newer_tab_step_answers_before_the_global_queue() {
        assert_eq!(
            undo_owner(false, false, false, false, true, true),
            UndoOwner::Tabs
        );
        assert_eq!(
            undo_owner(false, false, false, false, true, false),
            UndoOwner::Tabs
        );
        assert_eq!(
            undo_owner(false, false, false, false, false, true),
            UndoOwner::Global
        );
        assert_eq!(
            undo_owner(false, true, false, false, true, true),
            UndoOwner::Audio
        );
        assert_eq!(
            undo_owner(false, false, true, false, true, true),
            UndoOwner::Painter
        );
        // Com um quadro aberto quem compara as horas é o quadro (`board_keys::tabs_first`).
        assert_eq!(
            undo_owner(true, false, false, false, true, true),
            UndoOwner::Board
        );
    }

    /// ⭐ Com um QUADRO na aba activa, o Ctrl+Z (e o botão) é dele — mesmo com passo global, com o
    /// Painter activo ou o áudio aberto: a cena está escondida por baixo (plano MiroClone §1.4).
    #[test]
    fn an_active_board_owns_the_undo_before_everyone() {
        assert_eq!(
            undo_owner(true, true, true, true, false, true),
            UndoOwner::Board
        );
        assert_eq!(
            undo_owner(true, false, false, false, false, false),
            UndoOwner::Board
        );
    }
}
