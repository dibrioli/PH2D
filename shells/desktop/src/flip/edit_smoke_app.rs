//! **A metade que precisa da `App`** do `edit_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::edit_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::edit_smoke::*;
use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_edit_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        match FRAME.fetch_add(1, Ordering::Relaxed) {
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));
                let oid = gfx.flip.push_object("Edit Smoke");
                let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
                obj.fps = 12.0;
                let l = obj.add_layer("L");
                if let Some(d) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) {
                    let dr = obj.drawing_mut(d).expect("desenho");
                    dr.strokes.push(wave(24));
                    // Um quadrado PREENCHIDO ao lado — para ver que mover TODOS os
                    // pontos dele leva o miolo junto, e que o pick de ponto não o pega
                    // quando o clique mira a senoide.
                    let mut sq = FlipStroke::new();
                    for p in [
                        Vec2::new(-2.0, -3.0),
                        Vec2::new(0.0, -3.0),
                        Vec2::new(0.0, -1.5),
                        Vec2::new(-2.0, -1.5),
                    ] {
                        sq.push_point(Point {
                            pos: p,
                            width: 5.0,
                            opacity: 1.0,
                            color: Rgba::new(0.85, 0.2, 0.7, 1.0),
                        });
                    }
                    sq.closed = true;
                    sq.fill = Some(ph2d_flip::Fill {
                        color: Rgba::new(0.95, 0.8, 0.2, 1.0),
                        opacity: 1.0,
                    });
                    dr.strokes.push(sq);
                }
                self.playhead.pause();
            }
            // Entra no Edit + domínio Point pela porta REAL (os mesmos eventos dos pills).
            8 => {
                if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    for id in [
                        ph2d_editor::ids::FLIP_MODE_EDIT,
                        ph2d_editor::ids::FLIP_EDIT_DOM_POINT,
                    ] {
                        hero.bus
                            .push(ph2d_editor::action_bus::EditorAction::ToolPanelEvent(
                                ph2d_editor::tool::PanelEvent::Click(id),
                            ));
                    }
                }
                eprintln!(
                    "[edit-smoke] senoide de 24 ancoras + quadrado preenchido; modo Edit, \
                     dominio POINT (ancoras na tela). Roteiro: clique numa ancora (so ela) · \
                     Shift alterna · marquee pega as de dentro · arrastar move a selecao · \
                     Delete dissolve · All/None por ponto · Sculpt com meia senoide \
                     selecionada alisa SO a metade · dominio Stroke volta a pegar o traco."
                );
            }
            9 => self.any_input_this_frame = true, // arma o baseline do undo
            _ => {}
        }
    }
}
