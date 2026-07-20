//! **Quem responde por um Ctrl+Z** — o roteamento do desfazer, módulo irmão de
//! [`crate::undo`] (que é a FILA).
//!
//! O undo do editor não é um: são quatro (o Áudio, o Painter, o global de objetos, e o
//! image-edit single-level). Esta é a política que escolhe entre eles — e ela mora sozinha
//! porque **duas portas para a mesma pergunta divergem**: enquanto o botão Undo da barra
//! tinha caminho próprio, ele levantava o desfazer de IMAGEM enquanto o Ctrl+Z desfazia o
//! projeto (mover uma forma e clicar em Undo não fazia nada), e o botão **Redo não
//! despachava coisa alguma** — pintado, clicável, órfão. Agora os dois entram por aqui.

/// **Quem responde por um Ctrl+Z** (ou pelo botão Undo/Redo da barra — são a MESMA
/// pergunta, e é por isso que a política mora numa função só).
///
/// O undo do editor não é um: são cinco. Um editor modal focado (o Audio, o Painter) é o
/// dono do seu próprio Ctrl+Z — se ele cedesse o atalho, um passo a mais **saltaria a cena
/// inteira** (o undo global restaura o `WorldSnapshot` e limpa a seleção), e a fronteira
/// entre as duas filas é invisível para o usuário. Foi assim que o Ctrl+Z do Áudio virou
/// "às vezes não funciona, às vezes faz coisa estranha" (auditoria 2026-07-11, A1).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum UndoOwner {
    /// O Audio Editor, com o painel da onda aberto e um clipe carregado.
    Audio,
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
}

/// A política, sem `App`: quem responde, dado quem está aberto e quem tem passo.
///
/// Pura de propósito — é a única parte disto que dá para gatear sem uma janela, e é onde o
/// bug moraria (um dono a mais na cadeia e o atalho some).
#[must_use]
pub(crate) fn undo_owner(
    audio_owns: bool,
    painter_active: bool,
    colorize_owns: bool,
    global_has: bool,
) -> UndoOwner {
    if audio_owns {
        UndoOwner::Audio
    } else if painter_active {
        UndoOwner::Painter
    } else if colorize_owns {
        UndoOwner::Colorize
    } else if global_has {
        UndoOwner::Global
    } else {
        UndoOwner::ImageEdit
    }
}

impl crate::App {
    /// **O único desfazer do editor.** O Ctrl+Z e os botões Undo/Redo da barra entram por
    /// aqui — e é por isso que existem: enquanto o botão tinha caminho próprio, ele
    /// despachava o undo de IMAGEM (single-level) enquanto o atalho desfazia o projeto, e o
    /// botão **Redo não despachava nada** (era pintado e órfão). Dois caminhos para a mesma
    /// pergunta divergem; um só, não.
    pub(crate) fn undo_or_redo(&mut self, redo: bool) {
        #[cfg(feature = "panel-audio-editor")]
        let audio_owns = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.is_panel_visible("audio_editor"))
            && self.audio.as_ref().is_some_and(|a| a.editor_loaded());
        #[cfg(not(feature = "panel-audio-editor"))]
        let audio_owns = false;

        // O Colorize só é dono enquanto tem o que responder NESTA direção: rabisco
        // pendente pro undo, rabisco removido pro redo. Vazio dos dois lados, o atalho
        // passa direto ao Global (que é quem desfaz/refaz o Apply).
        let colorize_owns = self.flip_wants_colorize()
            && if redo {
                self.flip_colorize.can_redo_scribble()
            } else {
                self.flip_colorize.can_undo_scribble()
            };
        let Some(gfx) = self.gfx.as_mut() else { return };
        let painter_active = gfx
            .tools
            .active()
            .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("painter"));
        let global_has = if redo {
            self.undo.can_redo()
        } else {
            self.undo.can_undo()
        };
        match undo_owner(audio_owns, painter_active, colorize_owns, global_has) {
            UndoOwner::Audio =>
            {
                #[cfg(feature = "panel-audio-editor")]
                if let Some(a) = self.audio.as_mut() {
                    let can = if redo {
                        a.editor_can_redo()
                    } else {
                        a.editor_can_undo()
                    };
                    if can {
                        a.editor_apply(if redo {
                            ph2d_panel_audio_editor::AudioEditCmd::Redo
                        } else {
                            ph2d_panel_audio_editor::AudioEditCmd::Undo
                        });
                    }
                }
            }
            UndoOwner::Painter => {
                if redo {
                    self.painter_redo_requested = true;
                } else {
                    self.painter_undo_requested = true;
                }
            }
            UndoOwner::Colorize => {
                if redo {
                    self.flip_colorize.redo_scribble();
                } else {
                    self.flip_colorize.undo_scribble();
                }
            }
            // O passo é APLICADO no `post_frame_undo` (fim do frame, `self` livre e o
            // estado já reconciliado pelo `sync`) — aqui só se arma o pedido.
            UndoOwner::Global => self.undo_request = Some(redo),
            UndoOwner::ImageEdit => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.bus
                        .push(ph2d_editor::action_bus::EditorAction::UndoImageEdit);
                }
            }
        }
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
        assert_eq!(undo_owner(false, false, false, true), UndoOwner::Global);
        // Sem passo global, cai no image-edit (que é quem emite "Nothing to undo").
        assert_eq!(undo_owner(false, false, false, false), UndoOwner::ImageEdit);
        // O Painter é dono da fila dele.
        assert_eq!(undo_owner(false, true, false, true), UndoOwner::Painter);
        // E o Audio ganha do Painter E do global: um editor modal FOCADO não pode ceder o
        // atalho — um passo a mais saltaria a cena inteira (auditoria 2026-07-11, A1).
        assert_eq!(undo_owner(true, true, false, true), UndoOwner::Audio);
        assert_eq!(undo_owner(true, false, false, false), UndoOwner::Audio);
    }

    /// **O Colorize com rabisco pendente é dono do Ctrl+Z** (7º smoke, "undo/redo ruim"):
    /// os rabiscos são transientes (fora do `ProjectState`), então sem este dono o atalho
    /// caía no Global e apagava a última edição REAL do documento — o artista rabiscava,
    /// errava, pedia Ctrl+Z e via o traço de LINE-ART sumir com os rabiscos intactos. Sem
    /// rabisco na direção pedida, ele cede — e o Global é quem desfaz o Apply.
    #[test]
    fn pending_scribbles_own_the_undo_chord_and_yield_when_empty() {
        assert_eq!(undo_owner(false, false, true, true), UndoOwner::Colorize);
        assert_eq!(undo_owner(false, false, true, false), UndoOwner::Colorize);
        // Vazio na direção pedida: cede ao Global (o Apply é dele).
        assert_eq!(undo_owner(false, false, false, true), UndoOwner::Global);
        // Um editor modal focado ainda ganha (o Colorize é modo de tool, não painel modal).
        assert_eq!(undo_owner(true, false, true, true), UndoOwner::Audio);
        assert_eq!(undo_owner(false, true, true, true), UndoOwner::Painter);
    }
}
