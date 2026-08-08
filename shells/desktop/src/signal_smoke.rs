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
//!
//! # `PH2D_SIGNAL_SMOKE=2` — **as DUAS fontes na MESMA saída** (R0)
//!
//! A cena acima mais a **cena 73 da física** (`build_signal_scene`, a porta e o sino): a
//! timeline grita `footstep`/`beat` ao cruzar os markers e a física grita `door`/`bell` quando
//! a bola chega, e **os dois nomes saem pelo mesmo dreno**. Antes desta wave havia dois `for`
//! de toast escritos à mão, a oitenta linhas um do outro, cada um decidindo por conta o que um
//! sinal faz.
//!
//! ⚠️ **Ligue `PH2D_SIGNAL_LOG=1` junto** — é o SEGUNDO consumidor, com cursor próprio, e é o
//! que torna a propriedade visível em vez de inferida: no terminal cada linha diz a ORIGEM
//! (`<- timeline @ 1.000s` · `<- fisica, 4294967296 tocou 8589934592`), enquanto os toasts,
//! lidos por um cursor DIFERENTE, sobem na tela em paralelo. Dois consumidores, um canal,
//! nenhum deles consome o sinal do outro.

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

/// Que cena o valor de `PH2D_SIGNAL_SMOKE` pede.
///
/// ⚠️ **`=1` é a cena APROVADA da timeline e não se move** — `=2` é ela MAIS a metade da física.
/// Um valor que não parseia cai em **1**, e é deliberado: o default de um smoke tem de ser a
/// cena que já existia, não a mais nova (`PH2D_SIGNAL_SMOKE=sim` não pode virar a demo de R0
/// por acidente de digitação).
fn smoke_level(raw: &std::ffi::OsStr) -> u32 {
    raw.to_str()
        .and_then(|s| s.trim().parse().ok())
        .filter(|n| *n >= 1)
        .unwrap_or(1)
}

impl crate::App {
    /// In the frame prologue, once. No-op without the env.
    pub(crate) fn signal_smoke(&mut self) {
        if self.signal_smoke_done {
            return;
        }
        let Some(raw) = std::env::var_os("PH2D_SIGNAL_SMOKE") else {
            return;
        };
        if self.gfx.is_none() {
            return; // no world yet; try next frame
        }
        let level = smoke_level(&raw);
        self.signal_smoke_done = true;

        // No `=2` o mover sobe acima das plataformas da cena da física (que ficam em y=2 sobre
        // um chão em y=0): ele não tem corpo, então não colidiria, mas atravessaria o chão na
        // tela — e uma cena confusa é uma cena que o smoke julga errado.
        let mover_y = if level >= 2 { 3.5 } else { 0.0 };
        let mover = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(-6.0, mover_y)),
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

        if level >= 2 {
            // A metade da FÍSICA — a MESMA `build_signal_scene` da cena 73, chamada, não
            // recopiada: duas cenas que respondem *"o que a física grita?"* divergiriam, e a
            // que o Enio aprovou é aquela.
            let gfx = self.gfx.as_mut().expect("gfx");
            crate::physics_smoke_signal::build_signal_scene(gfx.sim.world_mut());
            gfx.camera.center = [0.0, 1.8];
            gfx.camera.height_world = 11.0;
            // Sem isto o solver não roda: o relógio tem DOIS consumidores e quem decide se
            // ele alcança o mundo rapier é este toggle, DESMARCADO por default (W4b).
            self.timeline.flags.simulate_physics = true;
            eprintln!(
                "[signal-smoke 2] AS DUAS FONTES, UMA SAIDA. A timeline grita\n  \
                 'footstep'@1,0s e 'beat'@2,5s; a fisica grita 'door' (a bola atravessa o\n  \
                 SENSOR rosa) e 'bell' (a bola BATE na plataforma ambar). A terceira\n  \
                 plataforma, cinza, e' o CONTROLE: sem componente, e ela nao grita.\n\n  \
                 O que provar na tela: os QUATRO toasts sobem pelo mesmo canto, e nenhum\n  \
                 codigo decide duas vezes o que um sinal faz -- ha um dreno so'.\n\n  \
                 (!) Rode com PH2D_SIGNAL_LOG=1 junto: e' o SEGUNDO consumidor, com cursor\n      \
                     proprio. No terminal cada linha diz a ORIGEM ('<- timeline @ 1.000s'\n      \
                     ou '<- fisica, A tocou B') enquanto os toasts sobem na tela por um\n      \
                     cursor DIFERENTE -- e nenhum dos dois consome o sinal do outro.\n"
            );
        }

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
