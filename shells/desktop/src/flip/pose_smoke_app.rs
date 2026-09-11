//! **A metade que precisa da `App`** do `pose_smoke` (W2/L5, 2026-09-11).
//!
//! O resto do módulo vive em [`ph2d_app_flip::pose_smoke`] — as leis e a cena, que não
//! precisam da shell. Aqui fica só o que toca os agregados DELA: `AppGfx` (o `FlipDoc`
//! vivo, o registo de ferramentas), o `HeroScreen` (o barramento dos painéis) e o
//! relógio. É a lista que o substrato da `ph2d-app-host` tem de cobrir para isto
//! também sair (W2 Fase B).

#[allow(unused_imports)]
use ph2d_app_flip::pose_smoke::*;
use ph2d_core::Vec2;
use ph2d_flip::{DupMode, Hold, KeyKind, Pose, Rgba};
use std::sync::atomic::Ordering;

impl crate::App {
    /// Roda no prólogo do frame (ao lado do `build_smoke`). No-op sem a env.
    pub(crate) fn flip_pose_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        match FRAME.fetch_add(1, Ordering::Relaxed) {
            // A cena: 1 objeto, chave 0 = quadrado, chave 12 = INSTÂNCIA movida.
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));
                let oid = gfx.flip.push_object("Pose Smoke");
                let obj = gfx.flip.object_mut(oid).expect("objeto recém-criado");
                obj.fps = 12.0;
                let l = obj.add_layer("L");
                if let Some(d) = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe) {
                    obj.drawing_mut(d).expect("desenho").strokes.push(square(
                        Vec2::new(-1.0, -1.0),
                        Vec2::new(1.0, 1.0),
                        Rgba::new(0.85, 0.2, 0.7, 1.0),
                    ));
                }
                assert!(
                    obj.duplicate_frame(l, 0, 12, DupMode::Instance),
                    "a instância do smoke tem de nascer"
                );
                // A instância já MOVIDA — pose ≠ identidade, o alvo do gizmo.
                obj.set_frame_pose(l, 12, Pose::from_translation(Vec2::new(2.5, 0.8)));
            }
            // O estado em que o Enio começa: playhead NA instância, modo Edit.
            8 => {
                self.playhead.pause();
                self.playhead.seek_frame(12, 12.0);
                if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    // O MESMO evento que o pill Edit do painel emite — o modo troca
                    // pela porta real (`FlipTool::handle_panel_event`).
                    hero.bus
                        .push(ph2d_editor::action_bus::EditorAction::ToolPanelEvent(
                            ph2d_editor::tool::PanelEvent::Click(ph2d_editor::ids::FLIP_MODE_EDIT),
                        ));
                }
                eprintln!(
                    "[pose-smoke] chave 0 = quadrado; chave 12 = INSTÂNCIA movida (+2.5, +0.8); \
                     playhead no 12, modo Edit. O gizmo da pose enquadra a arte posada: \
                     quina = rotate (anel)/scale, borda = scale-1-eixo, arrastar a arte = move. \
                     Confira: a chave 0 e o objeto NÃO se mexem; Ctrl+Z desfaz o gesto inteiro; \
                     na chave 0 (arte exclusiva) o gizmo de pose NÃO aparece."
                );
            }
            // Arma o baseline do undo (a cena nasceu sem input; sem isto o 1º gesto
            // arrastaria a criação para dentro do mesmo passo — igual ao build_smoke).
            9 => self.any_input_this_frame = true,
            _ => {}
        }
    }
}
