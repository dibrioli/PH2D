//! **Fase do quadro: O AUTOKEY** — o ponto ÚNICO que grava chaves de animação, DEPOIS de toda escrita de
//! Transform/opacidade da UI no quadro (gizmo, Inspector, reset da Hierarquia) (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;
use ph2d_app_audio::marcas_mesa;
use ph2d_app_physics::marcas_mundo;

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
            flip,
            component_registry,
            physics,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // AutoKey — the single choke point, AFTER every UI write of the frame and the apply (an
        // undo/scrub reads world == curve: no key), on the clock the apply drove (`ViewClock`).
        let autokey_clock = self.timeline.view_clock().pick(
            &self.playhead,
            &self.clip_playhead,
            &self.container_playhead,
        );
        // O que cada documento vale agora (plano 35): a população do autokey e todo campo animado.
        let mut publicar = autokey_pass::populacao(hero, &self.skeleton, sim, &self.preview_drive);
        publicar.extend(
            self.timeline
                .doc
                .bindings()
                .iter()
                .filter(|b| matches!(b.prop, ph2d_timeline::PropKind::Field(_)))
                .map(|b| b.entity),
        );
        if let Some(l) = sim.world().get_resource::<ph2d_ecs::LeitoresDosCanais>() {
            publicar.extend(l.todas());
        }
        // O desenho Flip em edição: o painel mostra as camadas dele, selecionado ou não.
        publicar.extend(ph2d_app_flip::marcas_camada::alvo(&self.flip_state, flip).map(|a| a.0));
        publicar.sort_unstable();
        publicar.dedup();
        // A entrada nascida neste quadro já publica o canal (a rede do undo é a do `main.rs`).
        ph2d_app_vec::ids_da_pilha::passe(sim, vec_scene);
        ph2d_app_vec::canais::publicar(sim, vec_scene, &self.vec.conducao, &publicar);
        ph2d_app_components::canais_objeto::publicar(sim, &publicar);
        ph2d_app_components::canais_componente::publicar(sim, component_registry, &publicar);
        ph2d_app_physics::publicar_canais(sim, physics, &publicar);
        ph2d_app_flip::canais_camada::publicar(sim, flip, &publicar);
        ph2d_app_constraints::canais::publicar(sim, &publicar);
        ph2d_app_audio::canais_mesa::publicar(sim);
        let sel: Vec<u64> = hero.gizmo.iter_selected().collect();
        let (solo, t_raw) = (self.timeline.keys_mode, autokey_clock.time());
        let inst = ph2d_timeline::Instante {
            t_cut: self
                .timeline
                .doc
                .author_cut(solo, self.timeline.container_open, t_raw),
            solo,
        };
        // As chaves do cartão das constraints (pegar/largar, Bake) — ANTES do Auto Key (`chaves`).
        let (pd, ph) = (&self.preview_drive, &self.playhead);
        let (painel, ak) = (
            hero.is_panel_visible("timeline"),
            timeline_bridge::default_interp(),
        );
        ph2d_app_constraints::chaves::do_quadro(&mut self.timeline, sim, pd, ph, inst, painel, ak);
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
        let mundo = timeline_bridge::Mundo::com_vetor(sim.world(), vec_scene);
        let valor = |e, p| match timeline_bridge::sample_prop_value(mundo, e, p) {
            Some(ph2d_anim::AnimValue::Float(v)) => Some(v),
            _ => None,
        };
        let mostrada = ph2d_app_vec::marcas::ids_mostrados(sim, &self.vec);
        let ctx = ph2d_app_vec::marcas::Contexto::do_painel(mostrada);
        let tl = &self.timeline.doc;
        let camada = ph2d_app_flip::marcas_camada::alvo(&self.flip_state, flip);
        if let Some(c) = ph2d_editor_core::widget::key_mark::take_key_mark_click() {
            let lidos = |e| {
                autokey_pass::campos_lidos(sim, e)
                    .into_iter()
                    .map(|(k, _)| k)
                    .collect()
            };
            let k =
                |e, p| timeline_bridge::chave_do_losango(mundo, &self.timeline, e, p, t_raw, solo);
            let mut intents =
                ph2d_app_vec::marcas::clique(tl, ctx, &c.fields, &sel, inst, lidos, valor, k);
            if intents.is_empty() {
                intents = ph2d_app_components::marcas_inspector::clique(
                    tl, &c.fields, &sel, inst, lidos, valor, k,
                );
            }
            if intents.is_empty() {
                intents = ph2d_app_constraints::marcas::clique(
                    tl, &c.fields, sim, &sel, inst, lidos, valor, k,
                );
            }
            if intents.is_empty()
                && let Some((e, obj)) = camada
            {
                intents = ph2d_app_flip::marcas_camada::clique(
                    tl, &c.fields, e, obj, inst, lidos, valor, k,
                );
            }
            if intents.is_empty() {
                intents = marcas_mundo::clique(tl, &c.fields, sim.world(), inst, lidos, valor, k);
            }
            if intents.is_empty() {
                intents = marcas_mesa::clique(tl, &c.fields, sim.world(), inst, lidos, valor, k);
            }
            self.timeline_intents.extend(intents);
        }
        let mut marcas = ph2d_app_vec::marcas::marcas(tl, ctx, &sel, inst, valor);
        marcas.extend(ph2d_app_components::marcas_inspector::marcas(
            tl, &sel, inst, valor,
        ));
        if let Some((e, obj)) = camada {
            marcas.extend(ph2d_app_flip::marcas_camada::marcas(
                tl, e, obj, inst, valor,
            ));
        }
        marcas.extend(ph2d_app_constraints::marcas::marcas(
            tl, sim, &sel, inst, valor,
        ));
        marcas.extend(marcas_mundo::marcas(tl, sim.world(), inst, valor));
        marcas.extend(marcas_mesa::marcas(tl, sim.world(), inst, valor));
        ph2d_editor_core::widget::key_mark::set_key_marks(marcas);
    }
}
