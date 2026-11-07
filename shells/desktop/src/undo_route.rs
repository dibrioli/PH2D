//! **Quem responde por um Ctrl+Z** — o roteamento do desfazer, módulo irmão de
//! [`crate::undo`] (que é a FILA).
//!
//! O undo do editor não é um: são quatro (o Áudio, o Painter, o global de objetos, e o
//! image-edit single-level). Esta é a política que escolhe entre eles — e ela mora sozinha
//! porque **duas portas para a mesma pergunta divergem**: enquanto o botão Undo da barra
//! tinha caminho próprio, ele levantava o desfazer de IMAGEM enquanto o Ctrl+Z desfazia o
//! projeto (mover uma forma e clicar em Undo não fazia nada), e o botão **Redo não
//! despachava coisa alguma** — pintado, clicável, órfão. Agora os dois entram por aqui.

use ph2d_editor_core::undo_route::{UndoOwner, undo_owner};
use ph2d_i18n::tr;

impl crate::App {
    /// ⭐⭐⭐ **ESTAÇÃO ZERO DO Ctrl+Z — a tecla chegou à janela.**
    ///
    /// ⚠️ **Um Ctrl+Z que nunca alcança o [`Self::undo_or_redo`] e um Ctrl+Z que nunca foi
    /// premido imprimem exactamente a mesma coisa: NADA.** O log do report de 2026-09-07 acaba
    /// assim — o arrasto de linha aparece inteiro (as três estações), e depois silêncio —, e as
    /// duas leituras levam a obras opostas.
    ///
    /// Entre a janela e a fila global há ~20 `return`: a escuta do Input Map, o `F9`, a paleta de comandos, o peek do Flip, o texto vectorial, o modal, os
    /// acordes de ficheiro, o bloco do Vector, o do Motion, o da timeline, e por fim o
    /// `handle_editor_key` — que ainda tem o seu próprio `return` de campo de texto focado. Cada
    /// um deles engole em **silêncio**, que é o desenho certo para um roteador e o pior possível
    /// para um diagnóstico.
    ///
    /// ⇒ **três estações, e falhar ENTRE duas delas nomeia o troço:** `RECEBIDA` (aqui, no topo
    /// do `key_input`), `SOBREVIVEU A CADEIA` (no `key_tail`, imediatamente antes do
    /// `handle_editor_key`) e `respondido por` (o log que o [`Self::undo_or_redo`] já tinha). O
    /// retrato das guardas vai junto porque a pergunta seguinte é sempre *qual delas*.
    pub(crate) fn diag_undo_chord(&self, estacao: &str) {
        if !Self::undo_log_on() {
            return;
        }
        let paleta = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.command_palette_model().is_some());
        eprintln!(
            "[undo] Ctrl+Z {estacao} · guardas: paleta={} texto_vectorial={} \
             campo_de_texto={} vector={} motion={} timeline={} (desfaz={}) rato_sobre_timeline={}",
            paleta,
            self.vec_text_editing(),
            self.text_entry_focused(),
            self.vector_keys_live(),
            self.motion_keys_live(),
            self.timeline_panel_open(),
            self.timeline.history.can_undo(),
            self.cursor_over_timeline(),
        );
    }

    /// **O único desfazer do editor.** O Ctrl+Z e os botões Undo/Redo da barra entram por
    /// aqui — e é por isso que existem: enquanto o botão tinha caminho próprio, ele
    /// despachava o undo de IMAGEM (single-level) enquanto o atalho desfazia o projeto, e o
    /// botão **Redo não despachava nada** (era pintado e órfão). Dois caminhos para a mesma
    /// pergunta divergem; um só, não.
    pub(crate) fn undo_or_redo(&mut self, redo: bool) {
        // Uma aba de FAMÍLIA activa (o editor de scripts) desfaz o texto dela, nunca a cena.
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut())
            && ph2d_editor_core::screens::hero::surface_input::undo(h, redo).is_some()
        {
            return;
        }
        let Some((dono, audio_owns, painter_active, colorize_owns, global_has)) =
            self.dono_do_desfazer(redo)
        else {
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else { return };
        // ⭐ **O log diz QUEM respondeu** (2026-09-04) — sem esta linha um Ctrl+Z que vai parar ao
        // Painter ou ao image-edit é indistinguível, no log, de um que nunca chegou: nenhum dos
        // dois imprime nada, e a fila global lê-se como «não fez nada».
        if Self::undo_log_on() {
            eprintln!(
                "[undo] Ctrl+{} respondido por {dono:?} (audio={audio_owns} painter={painter_active} \
                 colorize={colorize_owns} global={global_has})",
                if redo {
                    tr("shell.undo_route.shift_z")
                } else {
                    "Z"
                }
            );
        }
        match dono {
            UndoOwner::Board => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    let c = if redo {
                        ph2d_editor_core::screens::hero::board_keys::Command::Redo
                    } else {
                        ph2d_editor_core::screens::hero::board_keys::Command::Undo
                    };
                    ph2d_editor_core::screens::hero::board_keys::board_command(
                        hero,
                        &mut gfx.text_system,
                        c,
                    );
                }
            }
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
                    self.flip_state.colorize.redo_scribble();
                } else {
                    self.flip_state.colorize.undo_scribble();
                }
            }
            // O passo é APLICADO no `post_frame_undo` (fim do frame, `self` livre e o
            // estado já reconciliado pelo `sync`) — aqui só se arma o pedido.
            UndoOwner::Global => self.undo_request = Some(redo),
            UndoOwner::Tabs => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    ph2d_editor_core::screens::hero::document_tabs_menu::undo_tab(hero, redo);
                }
            }
            UndoOwner::ImageEdit => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    hero.bus
                        .push(ph2d_editor_core::action_bus::EditorAction::UndoImageEdit);
                }
            }
        }
    }

    /// O recibo do Remove da física só diz «Undo» enquanto o próximo Desfazer é o passo dele: publica o
    /// degrau do topo e se a fila global é quem responderia, pela mesma política do [`Self::undo_or_redo`].
    #[cfg(feature = "panel-inspector")]
    pub(crate) fn publica_o_topo_do_desfazer(&mut self) {
        let aba = self
            .gfx
            .as_mut()
            .and_then(|g| g.hero_screen.as_mut())
            .is_some_and(|h| {
                !h.text_entry_focused()
                    && h.documents
                        .active_surface_mut()
                        .is_some_and(|(f, _)| !f.undo_is_the_projects())
            });
        let global = !aba
            && self
                .dono_do_desfazer(false)
                .is_some_and(|d| d.0 == UndoOwner::Global);
        ph2d_panel_inspector::recibo_fisica::publica_o_topo(self.undo.top_at(false), global);
    }

    /// Quem responderia agora, e os factos que o decidiram: audio, painter, colorize e o global tem passo.
    fn dono_do_desfazer(&self, redo: bool) -> Option<(UndoOwner, bool, bool, bool, bool)> {
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
                self.flip_state.colorize.can_redo_scribble()
            } else {
                self.flip_state.colorize.can_undo_scribble()
            };
        let gfx = self.gfx.as_ref()?;
        let painter_active = gfx
            .tools
            .active()
            .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("painter"));
        let global_has = if redo {
            self.undo.can_redo()
        } else {
            self.undo.can_undo()
        };
        let board_active = gfx
            .hero_screen
            .as_ref()
            .is_some_and(|h| h.documents.active().is_some());
        // B5(4) — a operação de abas mais recente que o último passo da cena é a que a cena desfaz.
        let tabs_newer = gfx.hero_screen.as_ref().is_some_and(|h| {
            let tab = if redo {
                h.documents.tab_redo_at()
            } else {
                h.documents.tab_undo_at()
            };
            tab > self.undo.top_at(redo)
        });
        let dono = undo_owner(
            board_active,
            audio_owns,
            painter_active,
            colorize_owns,
            tabs_newer,
            global_has,
        );
        Some((dono, audio_owns, painter_active, colorize_owns, global_has))
    }
}
