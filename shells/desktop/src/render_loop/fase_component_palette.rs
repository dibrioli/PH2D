//! **Fase do quadro: A PALETA DE COMPONENTES** — abrir quando o `+` do Inspector a pede, reconstruir no
//! *Show all* e anexar o componente escolhido (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Corre DEPOIS do dreno do pivot do joint: esteve no meio do par `let joint_pivot_commit` → `if let`,
//! e um gate reprovou.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_component_palette(&mut self, add_component_for: Option<u64>) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        // A Animação de um objecto vive na Timeline, não no mundo (doc ObjectsAndComponents 01 §6).
        let doc = &self.timeline.doc;
        let has_animation = |b: u64| doc.has_animation(b);
        #[cfg(feature = "panel-inspector")]
        ph2d_panel_inspector::recibo::vigia(self.undo.depth());
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            toasts,
            hero_screen,
            component_registry,
            component_palette_target,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let add_component_for = add_component_for
            .or_else(|| super::fase_component_add_smoke::abre(hero, sim, component_registry));
        // ⚠️ DEPOIS do dreno do pivot do joint (o gate `the_position_commit_reseats_the_anchor_
        // through_the_door` lê a janela de 3000 bytes a seguir à captura; ver o cabeçalho).
        // ⭐ **O `+` do Inspector, as DUAS pontas** (ADR-0166 / F3) — abrir a paleta para quem
        // pediu, e anexar o que ela escolheu. Irmã por assunto (`component_attach`), como a
        // biblioteca do Motion é irmã do `motion_bridge`.
        ph2d_app_components::component_attach::open_palette_if_asked(
            hero,
            sim,
            component_registry,
            add_component_for,
            component_palette_target,
            &has_animation,
        );
        // ⭐ **A caixa *Show all*** — o widget vira o estado dele e avisa; quem reconstrói o
        // modelo é quem abriu a paleta (só ele sabe o que «mostrar tudo» quer dizer).
        ph2d_app_components::component_attach::refresh_palette_on_toggle(
            hero,
            sim,
            component_registry,
            *component_palette_target,
            &has_animation,
        );
        // ⚠️ O pick chega **noutro quadro** (a paleta fica aberta), e por isso o alvo vive no
        // `AppGfx` em vez de num local deste laço.
        super::fase_component_add_smoke::escolhe(hero);
        let animation_for = ph2d_app_components::component_attach::take_animation_pick(
            hero,
            component_palette_target,
        );
        let picked =
            ph2d_app_components::component_attach::route_pick(hero, component_palette_target);
        // ⭐ A COMPOSIÇÃO entrega as sementes das famílias donas (auditoria A1): a da física e a cerca da
        // câmera no contorno do nível (GEF 20 C6).
        if picked.is_some() || hero.store.has_component_resets() {
            let seeds = [
                ph2d_app_physics::physics_seed::COMPONENT_SEEDS,
                &[("ph2d::ecs::CameraLimits", ph2d_camera_frame::semeia_limites)],
            ]
            .concat();
            // ⭐ O recibo (plano GEF 25 §2.4): o que veio junto, com Undo, no Inspector.
            let _recibo = ph2d_app_components::component_attach::attach_picked(
                picked.as_ref(),
                sim,
                component_registry,
                &seeds,
                toasts,
            );
            #[cfg(feature = "panel-inspector")]
            if let Some((bits, frase)) = _recibo {
                ph2d_panel_inspector::recibo::mostra(bits, frase);
            }
            // O ↺ dos cartões (T60) repõe com as mesmas sementes.
            ph2d_app_components::component_attach::reset_asked(
                hero,
                sim,
                component_registry,
                &seeds,
                toasts,
            );
        }
        if let Some(bits) = animation_for {
            use ph2d_timeline::{AnimationIntent, TimelineIntent};
            self.timeline_intents
                .push(TimelineIntent::Animation(AnimationIntent::Add(bits)));
        }
    }
}
