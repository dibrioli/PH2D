//! **A cena pronta para o smoke do gizmo de POSE** (`PH2D_FLIP_POSE_SMOKE=1`, W7.5).
//!
//! O Enio não monta cena (feedback_ready_to_smoke_example): o app abre com 1 objeto
//! Flip cuja chave 0 desenha um quadrado, a chave 12 é uma **INSTÂNCIA** dele já
//! movida (pose ≠ identidade), o playhead parado NA instância e a tool Flip em modo
//! **Edit** — o gizmo da pose já visível enquadrando a arte posada.
//!
//! Roteiro: arrastar uma **quina** gira (anel de hover) / escala; uma **borda**
//! escala num eixo; arrastar a ARTE move (o gesto de sempre). Conferir que a chave 0
//! e o objeto **não se mexem**, que Ctrl+Z desfaz o gesto inteiro, e que voltar o
//! playhead à chave 0 (arte exclusiva) **não** mostra gizmo de pose.

use ph2d_core::Vec2;
use ph2d_flip::{DupMode, FlipStroke, Hold, KeyKind, Point, Pose, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

/// O frame corrente do roteiro (mesmo padrão do `build_smoke`).
static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_POSE_SMOKE").is_some())
}

/// Um quadrado de traço grosso (px de tela) entre `a` e `b`, fechado.
fn square(a: Vec2, b: Vec2, color: Rgba) -> FlipStroke {
    let mut s = FlipStroke::new();
    for p in [
        Vec2::new(a.x, a.y),
        Vec2::new(b.x, a.y),
        Vec2::new(b.x, b.y),
        Vec2::new(a.x, b.y),
    ] {
        s.push_point(Point {
            pos: p,
            width: 6.0,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s
}

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
