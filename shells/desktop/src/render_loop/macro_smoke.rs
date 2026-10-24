//! `PH2D_MACRO_SMOKE` — as cenas de [`ph2d_app_macros::CENAS_DO_SMOKE`]. A família monta o objecto e
//! grava a macro; aqui monta-se o clipe (`add_macro_clip`, o laço `[0, D]`, o Autoplay) e a cena corre.

use super::super::fase_macros::atlas_da_cena;
use ph2d_app_painter::macro_track::encaixe_vivo::MontaTela;
use ph2d_editor_core::action_bus::{EditorAction::Hierarchy, HierRequest::Delete};
use ph2d_editor_core::macro_edits::MacroRequest;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_timeline::TimelineIntent;

static FEITO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// A tela da cena (`=6` conduz-se sobre ela).
static TELA: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

impl crate::App {
    fn com_tela<T>(&mut self, monta: MontaTela<T>) -> Option<T> {
        let gfx = self.gfx.as_mut()?;
        super::super::fase_macros::atlas_da_cena!(gfx).monta(&mut gfx.sim, monta)
    }

    pub(crate) fn macro_smoke(&mut self) {
        let Ok(nivel) = std::env::var("PH2D_MACRO_SMOKE") else {
            return;
        };
        match nivel.as_str() {
            "5" | "6" => self.caminho_do_dono(nivel == "6"),
            "7" => return self.cena_da_biblioteca(),
            _ => {}
        }
        if self.gfx.as_ref().is_none_or(|g| g.hero_screen.is_none())
            || FEITO.swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            return; // ainda sem ecrã; tenta no próximo quadro
        }
        let montada = match nivel.as_str() {
            "1" => self.com_tela(ph2d_app_painter::macro_track::smoke::monta),
            "5" => self.com_tela(ph2d_app_painter::macro_track::smoke::monta_artista),
            // A mesma cena; o caminho do dono conduz-se em `caminho_do_dono`.
            "6" => self.com_tela(ph2d_app_painter::macro_track::smoke::monta_artista),
            "2" => self
                .with_motion_scene(ph2d_app_motion::macro_track::smoke::monta)
                .flatten(),
            // O tutorial «fazer»: sem clipe — a macro não toca, o aluno faz.
            "3" => {
                let bits = self
                    .with_motion_scene(ph2d_app_motion::macro_track::smoke::monta_fazer)
                    .flatten();
                if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    hero.gizmo.replace_selection(bits);
                    hero.set_panel_visible("hierarchy", true);
                }
                return;
            }
            // O tutorial «fazer» do Painter: a tela em modo Paint (o Wet Paint arma-se no bind).
            "4" => {
                let bits = self.com_tela(ph2d_app_painter::macro_track::smoke::monta_fazer);
                if let (Some(hero), Some(b)) =
                    (self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()), bits)
                {
                    ph2d_app_painter::composite_smoke::arm_hero(hero, b);
                    hero.set_panel_visible("hierarchy", true);
                }
                return;
            }
            _ => ph2d_app_macros::cena_desconhecida(&nivel),
        };
        let Some((bits, duracao)) = montada else {
            return;
        };
        let _ = TELA.set(bits);
        let parada = ph2d_app_macros::a_cena_abre_parada(&nivel);
        if !ph2d_app_macros::instala::clipe_da_cena(&mut self.timeline.doc, bits, duracao, parada) {
            return;
        }
        if !parada {
            self.timeline_intents
                .push(ph2d_timeline::TimelineIntent::RunScene);
        }
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(bits));
            if parada {
                ph2d_app_painter::composite_smoke::arm_hero(hero, bits);
            }
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("hierarchy", true);
            // As etapas à vista: o painel MACROS (lista e propriedades) e a linha «Macro steps».
            hero.set_panel_visible("macros", true);
        }
    }
    /// `PH2D_MACRO_SMOKE=7` — a biblioteca, o caminho inteiro ([`ph2d_app_macros::cena_da_biblioteca`]):
    /// a família faz cada passo pelas portas do `HeroScreen`; aqui o ponteiro, o `Delete` e a imagem maior.
    fn cena_da_biblioteca(&mut self) {
        use ph2d_app_macros::cena_da_biblioteca::{self as cena, Passo};
        if self.gfx.as_ref().is_none_or(|g| g.hero_screen.is_none()) {
            return;
        }
        for p in cena::vencidos().1 {
            let Some(g) = self.gfx.as_mut() else { return };
            let (sim, toasts) = (&g.sim, &mut g.toasts);
            let Some(p) = g
                .hero_screen
                .as_mut()
                .and_then(|h| cena::faz(p, h, sim, toasts))
            else {
                continue;
            };
            let sel = g.hero_screen.as_ref().and_then(|h| h.gizmo.selection);
            let quem = match p {
                Passo::Apaga => cena::gravado(),
                Passo::ImagemMaior => {
                    cena::nasce_a_maior(&mut g.sim, |s, i, n| atlas_da_cena!(g).nasce(s, i, n))
                }
                _ => None,
            };
            let row = quem.and_then(|b| g.hero_live.as_ref()?.bridge.node_for(b));
            if let Some(h) = g.hero_screen.as_mut().filter(|_| quem.is_some()) {
                h.gizmo.replace_selection(quem);
                (row.filter(|_| p == Passo::Apaga))
                    .map(|row| h.bus.push(Hierarchy(Delete { row })));
            }
            let Some((px, py)) = sel
                .zip(p.uv())
                .and_then(|(b, uv)| self.ponto_da_tela(b, uv))
            else {
                if p == Passo::Sobe {
                    self.painter_canvas_up();
                }
                continue;
            };
            self.last_pointer = (px, py);
            if matches!(p, Passo::Desce(_)) {
                self.painter_canvas_down(px, py, 1.0);
            } else {
                self.painter_canvas_move(px, py, 1.0);
            }
        }
    }

    /// `PH2D_MACRO_SMOKE=6` — **o caminho do dono** ([`ph2d_app_painter::macro_track::caminho`]): os
    /// gestos pelas portas de sempre (o painel MACROS, o ponteiro da tela, o transporte da Keys) e a
    /// linha de diagnóstico a cada décimo de segundo. Na `=5` (`conduz = false`) só o registo: o dono
    /// faz o caminho à mão.
    fn caminho_do_dono(&mut self, conduz: bool) {
        use ph2d_app_painter::macro_track::caminho::{self, Gesto, passo, regista};
        let Some(&bits) = TELA.get() else {
            return;
        };
        let (ms, gestos, diag) = passo(conduz);
        // Os pedidos à Timeline deste quadro (os do painel chegam aqui antes do dreno).
        if !self.timeline_intents.is_empty() {
            let mut i = format!("{:?}", self.timeline_intents);
            i.truncate(400);
            regista(&format!("{ms} ms: intents {i}"));
        }
        for g in gestos {
            regista(&format!("{ms} ms: {g:?}"));
            match g {
                Gesto::Grava => ph2d_app_macros::com(|m| m.pede(MacroRequest::Grava)),
                Gesto::Para => ph2d_app_macros::com(|m| m.pede(MacroRequest::Para)),
                Gesto::Rewind | Gesto::Arrasta(_) => {
                    self.timeline_intents
                        .push(TimelineIntent::Scrub(g.segundos()));
                    self.timeline_reveal_after_apply = true;
                }
                // A porta do clique no losango: o pedido que `macro_steps::apply` levanta.
                Gesto::Losango(k) => self.timeline_intents.extend(
                    (self.timeline_view.macro_etapas.get(k))
                        .map(|e| TimelineIntent::ScrubExact(e.fim)),
                ),
                // A porta do losango largado: o pedido que `macro_steps::apply` levanta (W16).
                Gesto::ArrastaLosango(k, d) => {
                    if let Some(e) = self.timeline_view.macro_etapas.get(k) {
                        let r = ph2d_panel_timeline::pedido_ao_largar(e, d);
                        ph2d_app_macros::com(|m| m.pede(r));
                    }
                }
                Gesto::Chaves => self.timeline_intents.extend(caminho::CHAVES_S.map(|s| {
                    TimelineIntent::AddKey {
                        entity: bits,
                        prop: ph2d_timeline::PropKind::Opacity,
                        t: ph2d_anim::RationalTime::from_seconds(s),
                        value: ph2d_anim::AnimValue::Float(1.0),
                        interp: ph2d_anim::Interp::Linear,
                    }
                })),
                Gesto::Play => self.timeline_intents.push(TimelineIntent::TogglePlay),
                // A porta do botão: o `EditorAction` que o clique levanta.
                Gesto::PlayDeCima => {
                    if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                        hero.bus
                            .push(ph2d_editor_core::action_bus::EditorAction::Transport(
                                ph2d_editor_core::action_bus::TransportCmd::Play,
                            ));
                    }
                }
                Gesto::Desce(uv) | Gesto::Move(uv) => {
                    let Some((px, py)) = self.ponto_da_tela(bits, uv) else {
                        continue;
                    };
                    self.last_pointer = (px, py);
                    let ok = if matches!(g, Gesto::Desce(_)) {
                        self.painter_canvas_down(px, py, 1.0)
                    } else {
                        self.painter_canvas_move(px, py, 1.0)
                    };
                    if !ok {
                        regista(&format!("the canvas refused {g:?} at ({px:.0}, {py:.0})"));
                    }
                }
                Gesto::Sobe => self.painter_canvas_up(),
                Gesto::CeFDesce
                | Gesto::CeFMove(_)
                | Gesto::CeFSobe
                | Gesto::EscolheCor(_)
                | Gesto::FechaSeletor
                | Gesto::FillPronto => {
                    if let Some(falha) = self.gesto_do_cef(bits, g) {
                        regista(&format!("{g:?}: {falha}"));
                    }
                }
                Gesto::MeioWet => {
                    if let Some(p) = self.gfx.as_mut().and_then(|g| {
                        ph2d_app_painter::painter_bridge_queries::o_painter(&mut g.tools)
                    }) {
                        ph2d_app_painter::macro_track::smoke::arm_wet(p);
                    }
                }
            }
        }
        let Some(quadros) = diag else {
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let painter = ph2d_app_painter::painter_bridge_queries::o_painter(&mut gfx.tools);
        let r = ph2d_app_painter::macro_track::smoke::Relogio {
            keys: self.timeline.keys_mode,
            playing: self.playhead.is_playing(),
            playhead: self.playhead.time(),
            clip: self.clip_playhead.time(),
            laco: self.timeline.doc.active_loop_for(self.timeline.keys_mode),
            etapas: self
                .timeline_view
                .macro_etapas
                .iter()
                .map(|e| e.fim)
                .collect(),
            gravando: format!("{:?}", ph2d_app_macros::com(|m| m.gravando())),
        };
        let w = gfx.sim.world();
        (gfx.painter_macros).regista_o_quadro((bits, ms, quadros), painter.as_deref(), w, &r);
    }

    /// Os gestos do C&F (W11) pelas portas da janela: o Down no botão, o arrasto, o Up; o valor do
    /// seletor e o fecho dele; o Done do modal pelo evento do chrome. `Some` = o que falhou.
    fn gesto_do_cef(
        &mut self,
        bits: u64,
        g: ph2d_app_painter::macro_track::caminho::Gesto,
    ) -> Option<String> {
        use ph2d_app_painter::macro_track::caminho::Gesto;
        use ph2d_editor_core::ids;
        match g {
            Gesto::CeFDesce => {
                let hero = self.gfx.as_ref()?.hero_screen.as_ref()?;
                let r = hero.hit_index.rect_for(ids::PAINTER_RAIL_FILL)?;
                let r = hero.escala().rect_to_physical(r);
                let (px, py) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
                self.last_pointer = (px, py);
                (!self.arm_fill_drag_if_on_button(px, py)).then(|| "C&F did not arm".to_owned())
            }
            Gesto::CeFMove(uv) => {
                let (px, py) = self.ponto_da_tela(bits, uv)?;
                self.last_pointer = (px, py);
                self.fill_drag_move(px, py);
                None
            }
            Gesto::CeFSobe => {
                self.fill_drag_up();
                None
            }
            g @ (Gesto::EscolheCor(_) | Gesto::FechaSeletor | Gesto::FillPronto) => {
                let hero = self.gfx.as_mut()?.hero_screen.as_mut()?;
                ph2d_app_painter::macro_track::caminho::no_seletor(g, hero)
            }
            _ => None,
        }
    }

    /// O ponto do ecrã de `(u, v)` na tela `bits` (a conta vive em `ph2d_app_painter`).
    fn ponto_da_tela(&self, bits: u64, uv: [f32; 2]) -> Option<(f32, f32)> {
        let g = self.gfx.as_ref()?;
        ph2d_app_painter::macro_track::smoke::ponto_da_tela(
            g.sim.world(),
            &g.camera,
            g.scene_window(),
            bits,
            uv,
        )
    }
}
