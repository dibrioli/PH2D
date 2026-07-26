//! **Smoke dos sinais da timeline** (ADR-0143). `PH2D_SIGNAL_SMOKE=1`:
//!
//! Um objeto ATRAVESSA a tela (X, 0..4 s) sob um LOOP `[0, 4)`, TOCANDO. Três markers na
//! régua: dois carregam SINAL (`footstep` @1,0 · `beat` @2,5) e um é ANOTAÇÃO pura
//! (`chapter` @3,5, sem sinal). A cada volta do loop, o play cruza os markers e dispara
//! **um toast por sinal** (`Signal: footstep`, `Signal: beat`) — a anotação NÃO dispara.
//!
//! O que provar na tela:
//! - **Toast a cada passagem** dos markers com sinal, e só deles (o round-trip do canal
//!   desacoplado, ADR-0075).
//! - **Glifo distinto**: abra a timeline (`L`) — os markers com sinal têm um FURO no
//!   galhardete; a anotação não (como o Signal Emitter da Unity).
//! - **Pausar para o toast** (Space): parado, nada dispara. Arraste a régua (scrub): nada
//!   dispara — sinal é evento de play PARA FRENTE.
//! - **Autoria**: `Shift`+duplo-clique num marker abre o editor de SINAL (borda na cor do
//!   marker); duplo-clique simples edita o label.
//!
//! ⚠️ Se a linha `[signal-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, TimelineDoc};

/// A moving object so there is motion to sync the signals against: X across the
/// screen, 0..4 s, ease-in-out (the loop replays it each pass).
fn author_mover(doc: &mut TimelineDoc, bits: u64) {
    let s = RationalTime::from_seconds;
    doc.insert_key(
        bits,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(-6.0),
        Interp::Bezier {
            x1: 0.8,
            y1: 0.0,
            x2: 0.2,
            y2: 1.0,
        },
    );
    doc.insert_key(
        bits,
        PropKind::TranslationX,
        s(4.0),
        AnimValue::Float(6.0),
        Interp::Linear,
    );
}

impl crate::App {
    /// In the frame prologue, once. No-op without the env.
    pub(crate) fn signal_smoke(&mut self) {
        if self.signal_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_SIGNAL_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // no world yet; try next frame
        }
        self.signal_smoke_done = true;

        let mover = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(-6.0, 0.0)),
                    Sprite::atlas(0, [1.4, 0.4], [1.0, 0.55, 0.15, 1.0]),
                    Name::new("Mover"),
                ))
                .id()
                .to_bits()
        };
        author_mover(&mut self.timeline.doc, mover);
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(mover));
        }

        // Two markers that EMIT, one that is a pure annotation.
        let doc = &mut self.timeline.doc;
        let foot = doc.add_marker(RationalTime::from_seconds(1.0), "step");
        doc.set_marker_signal(foot, Some("footstep".to_string()));
        let beat = doc.add_marker(RationalTime::from_seconds(2.5), "beat");
        doc.set_marker_signal(beat, Some("beat".to_string()));
        doc.add_marker(RationalTime::from_seconds(3.5), "chapter"); // no signal

        // Loop [0, 4) and PLAY: signals are a forward-play event, so the scene has to
        // be running for the toasts to fire — each pass crosses the two signal markers.
        //
        // ⚠️ Arm the loop through the DOCUMENT, then sync the playhead FROM it — never poke
        // `playhead.set_loop` directly. The doc is the truth the transport's Loop toggle
        // reads (`snap.loop_range.is_some()`); poking only the playhead loops playback while
        // the toggle paints OFF, which reads as "está fazendo loop sem eu marcar loop" (Enio,
        // 2026-07-26). This is the exact single door `sync_transport_loop` exists for, and it
        // sets the loop MODE (Wrap here) too, so the toggle can never disagree with the clock.
        self.timeline
            .doc
            .set_active_loop_for(false, Some((0.0, 4.0)));
        ph2d_timeline::sync_transport_loop(&self.timeline.doc, &mut self.playhead, false);
        self.playhead.seek(0.0);
        self.playhead.play();

        eprintln!(
            "[signal-smoke] 3 markers em [1,0 / 2,5 / 3,5] s; 2 com SINAL \
             (footstep@1,0 · beat@2,5), 1 anotacao pura (chapter@3,5). Loop [0,4) + PLAY \
             armados. Cada volta dispara um toast por sinal cruzado; a anotacao NAO."
        );
        eprintln!(
            "[signal-smoke] Pause (Space): nada dispara. Abra a timeline (L): os markers \
             com sinal tem FURO no galhardete. Shift+duplo-clique num marker edita o SINAL."
        );
    }
}

#[cfg(test)]
#[path = "signal_smoke_tests.rs"]
mod tests;
