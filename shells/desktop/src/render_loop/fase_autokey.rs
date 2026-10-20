//! **Fase do quadro: O AUTOKEY** — o ponto ÚNICO que grava chaves de animação, DEPOIS de toda escrita de
//! Transform/opacidade da UI no quadro (gizmo, Inspector, reset da Hierarquia) (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_autokey(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            toasts,
            hero_screen,
            vec_scene,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // AutoKey (W4.T1/T2) — the single choke point. Runs HERE, after every
        // UI Transform/opacity write for the frame (gizmo early, Inspector +
        // Hierarchy reset just above) so it reads the settled pose of each
        // selected sprite and keys only what left its curve. Placed after the
        // apply pass too, so an undo/paste/scrub — which the apply writes back
        // to the world — reads world == curve and keys nothing.
        // The pass authors on the clock the APPLY drove — the same `ViewClock` the drain
        // picked. Handing it the wrong clock while a solo drives the pose is how one strip in
        // a lane killed auto-key (2026-07-22).
        let autokey_clock = self.timeline.view_clock().pick(
            &self.playhead,
            &self.clip_playhead,
            &self.container_playhead,
        );
        // ⭐ O que o documento vetorial vale agora, para quem o compara com a curva (plano 35): a
        // população do autokey e todo objecto que tem um campo animado.
        let mut publicar = autokey_pass::populacao(hero, &self.skeleton, sim, &self.preview_drive);
        publicar.extend(
            self.timeline
                .doc
                .bindings()
                .iter()
                .filter(|b| matches!(b.prop, ph2d_timeline::PropKind::Field(_)))
                .map(|b| b.entity),
        );
        publicar.sort_unstable();
        publicar.dedup();
        ph2d_app_vec::canais::publicar(sim, vec_scene, &publicar);
        autokey_pass::run(
            &mut self.timeline,
            autokey_clock,
            &mut self.autokey,
            toasts,
            hero,
            sim,
            vec_scene,
            &self.preview_drive,
            // ⭐ A SEGUNDA mão: a ferramenta Bone a posar um osso é um arrasto como o do gizmo, e é
            // isso que faz o gesto inteiro ser UM passo de undo (ver `autokey_pass::run`).
            &self.skeleton,
        );
        // ⭐ O LOSANGO ao lado de cada campo (plano 35 §5): o clique desde o último quadro, depois o
        // estado de cada canal no instante do autor para a pintura deste quadro.
        let sel: Vec<u64> = hero.gizmo.iter_selected().collect();
        let (solo, t_raw) = (self.timeline.keys_mode, autokey_clock.time());
        let inst = ph2d_timeline::Instante {
            t_cut: self
                .timeline
                .doc
                .author_cut(solo, self.timeline.container_open, t_raw),
            solo,
        };
        let mundo = timeline_bridge::Mundo::com_vetor(sim.world(), vec_scene);
        let valor = |e, p| match timeline_bridge::sample_prop_value(mundo, e, p) {
            Some(ph2d_anim::AnimValue::Float(v)) => Some(v),
            _ => None,
        };
        let ctx = ph2d_app_vec::marcas::Contexto::do_painel();
        let tl = &self.timeline.doc;
        if let Some(c) = ph2d_editor_core::widget::key_mark::take_key_mark_click() {
            let lidos = |e| {
                autokey_pass::campos_lidos(sim, e)
                    .into_iter()
                    .map(|(k, _)| k)
                    .collect()
            };
            let k = |e, p| {
                timeline_bridge::chave_do_losango(mundo, &self.timeline, e, p, t_raw, solo)
            };
            let mut intents =
                ph2d_app_vec::marcas::clique(tl, ctx, &c.fields, &sel, inst, lidos, valor, k);
            if intents.is_empty() {
                intents = ph2d_app_components::marcas_inspector::clique(
                    tl, &c.fields, &sel, inst, valor, k,
                );
            }
            self.timeline_intents.extend(intents);
        }
        let mut marcas = ph2d_app_vec::marcas::marcas(tl, ctx, &sel, inst, valor);
        marcas.extend(ph2d_app_components::marcas_inspector::marcas(
            tl, &sel, inst, valor,
        ));
        ph2d_editor_core::widget::key_mark::set_key_marks(marcas);
    }
}
