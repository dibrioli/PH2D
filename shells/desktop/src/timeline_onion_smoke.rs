//! **Smoke do onion da timeline** (ADR-0142, W1). `PH2D_ONION_SMOKE=1`.
//!
//! Encena um objeto que ATRAVESSA a tela com um ease (X) e GIRA (Rotation), o seleciona,
//! arma o onion e pausa o playhead no MEIO — então os fantasmas do PASSADO (verdes, atrás)
//! e do FUTURO (azuis, à frente) aparecem dos dois lados, desvanecendo com a distância, e
//! o espaçamento deles conta o RITMO (juntos onde o ease é lento, esparramados no meio).
//!
//! Um segundo objeto animado, NÃO selecionado, prova o escopo: *só o selecionado ganha
//! fantasma* (o onion é do que está na mão, ADR-0142).
//!
//! ⚠️ Se a linha `[onion-smoke]` de prólogo não aparecer, PARE: a cena não montou (sem
//! isso, uma tela sem fantasmas é indistinguível da feature quebrada).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, TimelineDoc};

/// Keya um objeto que vai de `x0` a `x1` (ease-in-out) girando de 0 a `rot`, em 0..4 s.
fn author_mover(doc: &mut TimelineDoc, bits: u64, x0: f32, x1: f32, rot: f32) {
    let s = RationalTime::from_seconds;
    // X com ease-in-out: o espaçamento dos fantasmas conta o ritmo (a lição da fita).
    doc.insert_key(
        bits,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(x0),
        Interp::Bezier { x1: 0.8, y1: 0.0, x2: 0.2, y2: 1.0 },
    );
    doc.insert_key(bits, PropKind::TranslationX, s(4.0), AnimValue::Float(x1), Interp::Linear);
    // Rotação linear: os fantasmas mostram a MUDANÇA DE POSE, não só de posição.
    doc.insert_key(bits, PropKind::Rotation, s(0.0), AnimValue::Float(0.0), Interp::Linear);
    doc.insert_key(bits, PropKind::Rotation, s(4.0), AnimValue::Float(rot), Interp::Linear);
}

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn timeline_onion_smoke(&mut self) {
        if self.timeline_onion_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_ONION_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // ainda sem mundo; tenta no próximo frame
        }
        self.timeline_onion_smoke_done = true;

        // O MOVER (selecionado) e o BYSTANDER (animado, não selecionado). Finos e compridos
        // para o giro se ver.
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

        author_mover(&mut self.timeline.doc, mover, -6.0, 6.0, 1.2);
        author_mover(&mut self.timeline.doc, bystander, -6.0, 6.0, 1.2);

        // Seleciona o Mover — só o selecionado ganha fantasma.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(mover));
        }

        // Arma o onion: 3 quadros de cada lado, no fps do doc, opacidade generosa para o
        // smoke. Estado de VISTA — a UI do painel é W3; aqui a cena o liga.
        self.onion.enabled = true;
        self.onion.frames_before = 3;
        self.onion.frames_after = 3;
        self.onion.opacity = 0.6;
        self.onion.fps = self.timeline.doc.fps_display;

        // Pausado no MEIO (2 s de 4), para os dois lados do onion aparecerem.
        self.playhead.seek(2.0);
        self.playhead.pause();

        eprintln!(
            "[onion-smoke] Mover selecionado + Bystander (animado, NAO selecionado). \
             onion: {} fantasmas antes / {} depois, fps {:.1}, playhead a 2,00 s.",
            self.onion.frames_before, self.onion.frames_after, self.onion.fps
        );
        eprintln!(
            "[onion-smoke] Olhe o Mover: fantasmas VERDES atras (passado) e AZUIS a frente \
             (futuro), desvanecendo com a distancia; o espacamento conta o ease. O Bystander \
             NAO tem fantasma (so o selecionado). Clique o Bystander: o onion migra para ele."
        );
    }
}

#[cfg(test)]
#[path = "timeline_onion_smoke_tests.rs"]
mod timeline_onion_smoke_tests;
