//! **Fase do quadro: OS ESPELHOS DA FERRAMENTA VECTORIAL E DA FÍSICA** — o modo e as formas da
//! ferramenta vectorial para o input e o painel de mundo da física (OBRA 2 da `line/render-loop`,
//! 2026-09-13). ⛔ O objecto inicial do Flip que nascia aqui ao activar a ferramenta saiu (spec/06
//! F3): um desenho nasce pelo menu Add, e o modo nunca cria um objecto (§6.5).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_tool_mirrors(
        &mut self,
        vec_cfg: ph2d_tool_vector::VectorDrawConfig,
    ) -> Option<ph2d_tool_vector::VectorDrawConfig> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            hero_screen,
            physics,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // Mirror the tool's mode + shape params for the input dispatch's
        // pen-vs-shape routing (the downcast lives in the bridge).
        self.vec.draw_config = vec_cfg;
        // ⭐⭐⭐ **A SONDA DA FOTOGRAFIA arma o verbo AQUI, e o sítio é a lei dela**
        // (`PH2D_VEC_WEIGHT_PROBE=1`, ver [`crate::vec_bone_smoke::sonda_do_peso`]).
        //
        // ⛔⛔ **Escolher um osso FORÇA o verbo a `Transform`** (a porta de aresta da
        // `fase_selection_mirror_bone_focus`, com razão: o artista que acabou de escolher um osso
        // quer transformá-lo). Uma sonda que arme o verbo no prólogo da cena é reposta no quadro
        // seguinte — *duas fotos seguidas mostraram-me o painel do `Transform` e eu li as duas como
        // «o pincel não desenha nada»*.
        //
        // ⇒ ela mora na ÚNICA linha que escreve o espelho, logo ganha a toda a porta de aresta e a
        // tudo o que lê `draw_config` a jusante — o painel e o overlay.
        if crate::vec_bone_smoke::sonda_do_peso().is_some() {
            self.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Bone;
            self.vec.draw_config.bone_action = ph2d_tool_vector::BoneAction::Weight;
        }

        // ADR-0114 W2: espelha o estado da tool Flip (ativa + estilo de brush)
        // pro input_dispatch decidir/assar o desenho sem downcast (o downcast
        // vive no flip_bridge, allowlistado).
        // Physics WORLD panel (ADR-0131 D8 / W2b). Same phase as the other
        // panel bridges — after the ActivateTool drain, before paint — but
        // deliberately NOT tool-gated: this panel belongs to the document,
        // so the artist owns its visibility and this call never writes it.
        //
        // ⚠️ Distinct from `ph2d_app_physics::bridge::dispatch::dispatch` far above, which steps
        // the SIMULATION at the Playhead tick. Two bridges, two phases.
        self.show_colliders = ph2d_app_physics::panel_bridge::dispatch(
            hero,
            physics,
            self.show_colliders,
            &mut self.physics.interaction,
            // W25: a corrida gravada é um fato do DOCUMENTO, e este é o
            // painel do documento. A §14 mostra o mesmo par de números; as
            // duas vistas caem na mesma porta (`run_stash`).
            ph2d_app_physics::panel_bridge::RunTapes {
                live: &mut self.player_tape,
                stash: &mut self.discarded_run,
                fixed_dt: self.fixed_step.fixed_dt(),
            },
        );
        Some(vec_cfg)
    }
}
