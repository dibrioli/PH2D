//! **Fase do quadro: A ACTIVAÇÃO DA FERRAMENTA DE IMAGEM** — a activação da ferramenta de imagem pedida pelo barramento (o hero não alcança `gfx.tools`) (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;
use ph2d_i18n::tr_with;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct ImageToolActivationIntents {
    pub(super) pending_image_tool_activation: Option<&'static str>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_image_tool_activation(&mut self, intents: ImageToolActivationIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            toasts,
            tools,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let ImageToolActivationIntents {
            mut pending_image_tool_activation,
        } = intents;
        // Drain the `EditorAction::ActivateTool { tool_id: "bgremoval" }`
        // intent raised by clicking the Bg Removal pill. The hero can't reach
        // `gfx.tools` so the activation round-trips via the bus.
        // Same force-refresh of the snapshot push state as the
        // Digit3 shortcut below so the next snapshot push fires
        // against the current selection.
        // Data-driven activation of any stateful image-tool (audit F1
        // 2026-05-26 — substitui 6 drain blocks hardcoded per-tool).
        // Gated on `mode_on`: image tools are only reachable while Image
        // Tools toggle is on (the pills only exist then; the Digit3
        // shortcut must also respect the mode). The reconcile below is
        // the safety net, but gating here avoids a 1-frame
        // activate→deactivate flicker + a spurious toast.
        //
        // Cluster lookup via `installed_registry()` resolve o handler kind
        // (Stateful vs OneShot) e o label canônico (`Tool::label()`); zero
        // hardcoded id no dispatch. Tools dropped via fan-out drop-crate
        // (incluindo Painter T1.1) flow pelo mesmo canal automaticamente.
        //
        // Legacy débito: `last_bgremoval_pushed_entity = None` reset é
        // bgremoval-specific shell cache. Em T-N.X (refactor cache-per-tool
        // map) substituído por `HashMap<ToolId, ShellCache>` ou hook em
        // `Tool::on_activate` (ADR-0041). Por hoje, mantido inline.
        if let Some(tool_id) = pending_image_tool_activation.take() {
            // Look up the activating tool's cluster + Stateful gate.
            // W1.T1.7 generalization: was "image_tools" only; now also
            // accepts "vector_tools" (Pen tool ship). When a third
            // cluster appears, add it here OR extract a generic
            // `find_activatable_stateful_tool` helper.
            let activating_cluster: Option<&'static str> = ph2d_editor_core::installed_registry()
                .and_then(|reg| {
                    ["image_tools", "vector_tools", "motion_tools", "flip_tools"]
                        .into_iter()
                        .find(|&cluster_name| {
                            reg.cluster(cluster_name).iter().any(|m| {
                                m.id == tool_id
                                    && matches!(
                                        m.handler,
                                        ph2d_tool_registry::ToolHandler::Stateful { .. }
                                    )
                            })
                        })
                });
            // Per-cluster activation gate. "image_tools" requires
            // the IMG mode toggle; "vector_tools" / "motion_tools" have no
            // toggle so they're always-on (the pill is direct-activate).
            let gate_on = match activating_cluster {
                Some("image_tools") => hero.image_edit.mode_on,
                Some("vector_tools") | Some("motion_tools") | Some("flip_tools") => true,
                _ => false,
            };
            // O pill de um cluster direct-activate ALTERNA: clicar na ferramenta
            // já ativa sai dela e volta para a default (move). É o que faz uma
            // forma vetorial voltar a se comportar como qualquer objeto — o
            // gizmo de sprite a move, o clique a seleciona (ADR-0111). Os
            // `image_tools` ficam de fora: quem manda neles é o toggle IMG.
            let already_active = tools.active().map(ph2d_editor_core::Tool::id)
                == Some(ph2d_editor_core::ToolId::new(tool_id));
            let toggles_off = matches!(
                activating_cluster,
                Some("vector_tools" | "motion_tools" | "flip_tools")
            );
            if gate_on && already_active && toggles_off {
                tools.activate_default();
                self.title_dirty = true;
                if let Some(active) = tools.active() {
                    toasts.push(Toast::info(tr_with(
                        "shell.fase_image_tool_activation.tool",
                        &[("label", &(active.label()))],
                    )));
                }
            } else if gate_on && tools.set_active(&ph2d_editor_core::ToolId::new(tool_id)) {
                // **ENTRAR NO PAINTER COLAPSA A SELEÇÃO À ÚLTIMA** (Enio, 2026-08-19: *"se o
                // usuário estiver com múltiplas imagens selecionadas e entrar no painter,
                // selecione a última selecionada e desselecione as outras antes de entrar"*).
                //
                // ⚠️ **Antes de entrar, e não depois:** o Painter lê a seleção ao ativar-se
                // para saber que documento abrir. Colapsar depois deixá-lo-ia um quadro com o
                // estado que a trava existe para impedir — e um quadro chega para ele ligar a
                // prévia à sprite errada.
                if tool_id == "painter" {
                    let dropped = ph2d_app_painter::painter_lock::collapse_to_last(hero);
                    if dropped > 0 {
                        toasts.push(Toast::info(tr_with(
                            "shell.fase_image_tool_activation.painter_kept_the_last",
                            &[("dropped", &dropped)],
                        )));
                    }
                }
                self.title_dirty = true;
                if tool_id == "bgremoval" {
                    self.last_bgremoval_pushed_entity = None;
                }
                if let Some(active) = tools.active() {
                    toasts.push(Toast::info(tr_with(
                        "shell.fase_image_tool_activation.tool",
                        &[("label", &(active.label()))],
                    )));
                }
                // (R4: Pen activation no longer needs a sprite —
                // network IS the asset, world-coords throughout.)
            }
        }
    }
}
