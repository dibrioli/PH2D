//! **A cena pronta para o smoke do domínio POINT** (`PH2D_FLIP_EDIT_SMOKE=1`, W8).
//!
//! O app abre com 1 objeto Flip (uma senoide de 24 âncoras + um quadrado preenchido),
//! a tool Flip em modo **Edit** e o domínio já em **Point** — as âncoras na tela
//! (dim; selecionadas em acento).
//!
//! Roteiro: clicar numa âncora seleciona SÓ ela · Shift+clique alterna · marquee pega
//! as de dentro · arrastar uma selecionada move a seleção (as outras ficam) · Delete
//! dissolve · **All/None** do painel agem por ponto · trocar pro **Sculpt** com meia
//! senoide selecionada: o Smooth alisa SÓ a metade (máscara fina) · voltar ao domínio
//! **Stroke**: um clique volta a pegar o traço inteiro (a seleção promove por `any`).

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_EDIT_SMOKE").is_some())
}

/// Uma senoide com `n` âncoras — pontos de sobra para o pick/marquee/dissolve.
fn wave(n: usize) -> FlipStroke {
    let mut s = FlipStroke::new();
    for k in 0..n {
        let t = k as f32 / (n - 1) as f32;
        s.push_point(Point {
            pos: Vec2::new(-3.0 + 6.0 * t, libm::sinf(t * 12.0)),
            width: 5.0,
            opacity: 1.0,
            color: Rgba::new(0.9, 0.9, 0.95, 1.0),
        });
    }
    s
}

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
