//! **Smoke do onion da timeline** (ADR-0142). Duas cenas:
//!
//! - `PH2D_ONION_SMOKE=1` — modo **Frames**: um objeto que ATRAVESSA a tela com ease (X) e
//!   GIRA, pausado no MEIO. Fantasmas do passado (verdes) atrás e do futuro (azuis) à
//!   frente, a `t ± k` quadros, desvanecendo com a distância; o espaçamento conta o ritmo.
//! - `PH2D_ONION_SMOKE=2` — modo **Keys** (o default): um objeto keyado em 5 POSES
//!   distintas (zigue-zague); os fantasmas caem NAS keyframes vizinhas — o pose-a-pose.
//!
//! Em ambos, um segundo objeto animado NÃO selecionado prova o escopo (*só o selecionado
//! ganha fantasma*). ⚠️ Se a linha `[onion-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, TimelineDoc};

use crate::render_loop::timeline_onion::OnionMode;

/// Modo Frames: X com ease-in-out (o espaçamento conta o ritmo) + rotação linear, 0..4 s.
fn author_mover(doc: &mut TimelineDoc, bits: u64, x0: f32, x1: f32, rot: f32) {
    let s = RationalTime::from_seconds;
    doc.insert_key(
        bits,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(x0),
        Interp::Bezier { x1: 0.8, y1: 0.0, x2: 0.2, y2: 1.0 },
    );
    doc.insert_key(bits, PropKind::TranslationX, s(4.0), AnimValue::Float(x1), Interp::Linear);
    doc.insert_key(bits, PropKind::Rotation, s(0.0), AnimValue::Float(0.0), Interp::Linear);
    doc.insert_key(bits, PropKind::Rotation, s(4.0), AnimValue::Float(rot), Interp::Linear);
}

/// Modo Keys: uma POSE autorada por instante (x, y, rot) — o zigue-zague que faz cada
/// fantasma vizinho ser visivelmente distinto.
fn author_poses(doc: &mut TimelineDoc, bits: u64, poses: &[(f64, f32, f32, f32)]) {
    let s = RationalTime::from_seconds;
    for &(t, x, y, rot) in poses {
        doc.insert_key(bits, PropKind::TranslationX, s(t), AnimValue::Float(x), Interp::Linear);
        doc.insert_key(bits, PropKind::TranslationY, s(t), AnimValue::Float(y), Interp::Linear);
        doc.insert_key(bits, PropKind::Rotation, s(t), AnimValue::Float(rot), Interp::Linear);
    }
}

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn timeline_onion_smoke(&mut self) {
        if self.timeline_onion_smoke_done {
            return;
        }
        let Some(mode_env) = std::env::var_os("PH2D_ONION_SMOKE") else {
            return;
        };
        if self.gfx.is_none() {
            return; // ainda sem mundo; tenta no próximo frame
        }
        self.timeline_onion_smoke_done = true;

        let mut spawn = |x: f32, y: f32, tint: [f32; 4], name: &str| -> u64 {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(x, y)),
                    Sprite::atlas(0, [1.4, 0.4], tint),
                    Name::new(name),
                ))
                .id()
                .to_bits()
        };
        let mover = spawn(-6.0, -1.0, [1.0, 0.55, 0.15, 1.0], "Mover");
        let bystander = spawn(-6.0, 3.0, [0.5, 0.55, 0.7, 1.0], "Bystander");

        let keys_scene = mode_env == "2";
        if keys_scene {
            // 5 poses distintas (zigue-zague) ⇒ os fantasmas por-key são bem separados.
            let poses = [
                (0.0, -6.0, -3.0, 0.0),
                (1.0, -3.0, 1.0, 0.5),
                (2.0, 0.0, -2.0, -0.4),
                (3.0, 3.0, 2.0, 0.9),
                (4.0, 6.0, -1.0, 0.2),
            ];
            author_poses(&mut self.timeline.doc, mover, &poses);
            author_poses(&mut self.timeline.doc, bystander, &poses);
            self.onion.mode = OnionMode::Keys;
        } else {
            author_mover(&mut self.timeline.doc, mover, -6.0, 6.0, 1.2);
            author_mover(&mut self.timeline.doc, bystander, -6.0, 6.0, 1.2);
            self.onion.mode = OnionMode::Frames;
        }

        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(mover));
        }

        self.onion.enabled = true;
        self.onion.frames_before = if keys_scene { 2 } else { 3 };
        self.onion.frames_after = if keys_scene { 2 } else { 3 };
        self.onion.opacity = 0.6;
        self.onion.fps = self.timeline.doc.fps_display;

        // Pausado no MEIO (2 s de 4), para os dois lados do onion aparecerem.
        self.playhead.seek(2.0);
        self.playhead.pause();

        if keys_scene {
            eprintln!(
                "[onion-smoke] modo KEYS: Mover keyado em 5 POSES; playhead a 2,00 s. \
                 Fantasmas VERDES nas 2 keyframes anteriores, AZUIS nas 2 seguintes — \
                 pose-a-pose. Bystander (animado, nao selecionado) NAO tem fantasma."
            );
        } else {
            eprintln!(
                "[onion-smoke] modo FRAMES: fantasmas VERDES atras / AZUIS a frente a t±k \
                 quadros ({} de cada lado), fps {:.1}, playhead a 2,00 s. O espacamento \
                 conta o ease. Bystander (nao selecionado) NAO tem fantasma.",
                self.onion.frames_before, self.onion.fps
            );
        }
        eprintln!("[onion-smoke] Clique o Bystander: o onion migra para ele (so o selecionado).");
    }
}

#[cfg(test)]
#[path = "timeline_onion_smoke_tests.rs"]
mod timeline_onion_smoke_tests;
