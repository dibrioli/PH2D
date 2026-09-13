//! **Fase do quadro: O MODO IMAGE TOOLS E AS PILLS** — o *Image Tools* desligado manda na ferramenta activa, o id dela espelhado no hero e o
//! estado das pills (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_image_tools_mode_and_pills(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            tools, hero_screen, ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // Image Tools OFF is AUTHORITATIVE over the active tool. The
        // TopBar Image Tools toggle (`image_edit.mode_on`) and the
        // ToolRegistry's active tool are otherwise decoupled: a
        // stateful image tool (Bg Removal / Padding) activated while
        // the mode was on stays active — panel + on-canvas preview and
        // all — after the mode is toggled off, since nothing
        // deactivated it. Reconcile here every frame, BEFORE the
        // panel/preview bridges run: when the mode is off, no
        // image-edit tool may remain active, so switch back to the
        // default tool and drop the Bg-Removal preview. This is the
        // single invariant that makes "Image Tools off ⟹ every image
        // tool off & inaccessible" hold no matter how the tool became
        // active (toggle-off, a stale path, the Digit3 shortcut).
        if !hero.image_edit.mode_on {
            let active_is_image_tool = tools
                .active()
                .map(|t| crate::is_image_edit_tool(&t.id()))
                .unwrap_or(false);
            if active_is_image_tool
                && let Some(default_id) = tools.default_tool_id()
                && tools.set_active(&default_id)
            {
                self.bgremoval_preview = None;
                self.last_bgremoval_pushed_entity = None;
                self.title_dirty = true;
            }
        }
        // Mirror the active image-edit tool's canonical id into the hero
        // state so editor-core chrome (the left rail's Painter face) can
        // react without a dependency on the concrete tool crates (ADR-0040).
        // Runs AFTER the mode-off reconciliation above, so it reflects the
        // frame's final active tool. `ToolId` holds a runtime `String`; the
        // rail only needs to recognise the Painter, so intern to the
        // `&'static str` literal the `ActivateTool { tool_id: "painter" }`
        // action already uses. Gated on `mode_on` (no image tool is
        // reachable with Image Tools off).
        // ⛔⛔ **O espelho passou a servir TODA ferramenta, e a lista à mão morreu.** Ele
        // internava contra um `match` de **um** literal (`"painter"`) e filtrava por
        // `mode_on` — o que era verdade enquanto o único leitor era o trilho do Painter.
        // Deixou de ser em 2026-08-30: os toggles de `vector`/`motion`/`flip` precisam de
        // saber se a ferramenta DELES está activa para escolher entre activar e cancelar, e
        // com o espelho cego eles liam sempre *«não está»* — o segundo clique reactivava.
        //
        // ⭐ A internagem vem do **registry**: os `manifest.id` já são `&'static str`, então
        // procurar o manifesto cujo id bate com o id vivo devolve o `&'static` certo sem
        // alocar e **sem lista escrita à mão** — uma ferramenta nova entra sozinha.
        //
        // ⚠️ **E o filtro `mode_on` saiu**: ele pertence a quem pergunta pelo Painter, e
        // `offers::rail_shows_painter_tools` já o exige (`mode_on && == Some("painter")`).
        // Aqui ele apagava a resposta para as ferramentas que não são de imagem.
        let live = tools.active().map(|t| t.id());
        hero.image_edit.active_tool_id =
            crate::active_tool_mirror::intern_active_tool(live.as_ref().map(|i| i.0.as_str()));
        // Reconcile Image Tools pill ButtonState ↔ active tool. Each pill
        // whose manifest id matches `tools.active()` is forced to Pressed;
        // pills holding a stale Pressed (tool no longer active) drop back
        // to Normal. Hovered/click-transient states are preserved (we only
        // touch the Normal↔Pressed transitions).
        //
        // Data-driven via `installed_registry().cluster("image_tools")` —
        // zero hardcoded tool id (anti-padrão Image Tools Bugs §2.b
        // fechado em T1.2). New tools dropped via fan-out drop-crate
        // inherit the highlight wiring automatically.
        {
            let active_id_string: Option<String> = tools.active().map(|t| t.id().0.clone());
            if let Some(reg) = ph2d_editor_core::installed_registry() {
                // W1.T1.7 R3: iterate both image_tools (existing)
                // AND vector_tools (Pen pill ship) so the Pressed-
                // highlight reconcile picks up the Pen pill when
                // the Vector Pen tool activates. Each pill's
                // NodeId is computed via `hash_node_id(manifest.id)`
                // — for Pen this matches `TOPBAR_VECTOR_PEN` only
                // because `TOPBAR_VECTOR_PEN = hash_node_id("vector_pen")`
                // (image-action pill convention; see ids.rs).
                for cluster_name in ["image_tools", "vector_tools"] {
                    for manifest in reg.cluster(cluster_name) {
                        let pill_id = ph2d_tool_registry::hash_node_id(manifest.id);
                        let should_press = active_id_string.as_deref() == Some(manifest.id);
                        if let Some(ph2d_editor_core::InteractiveState::Button { state }) =
                            hero.store.get_mut(pill_id)
                        {
                            use ph2d_editor_core::widget::ButtonState;
                            match (*state, should_press) {
                                (ButtonState::Normal, true) => *state = ButtonState::Pressed,
                                (ButtonState::Pressed, false) => *state = ButtonState::Normal,
                                _ => {} // preserve Hovered + already-consistent
                            }
                        }
                    }
                }
            }
        }
    }
}
