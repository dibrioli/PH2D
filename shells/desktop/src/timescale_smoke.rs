//! **Smoke do time-scale de seleção** (joias da coroa §4). `PH2D_TIMESCALE_SMOKE=1`:
//!
//! Um objeto com uma track de X em ZIGUE-ZAGUE (5 keys em 0..4 s), com TODAS as keys
//! JÁ SELECIONADAS e a timeline ABERTA — então a caixa de seleção com as duas alças de
//! TEMPO aparece de cara. Arraste uma alça e a seleção ESTICA/ENCOLHE no tempo em torno
//! da borda oposta (o pivô). É o verbo de retiming mais amado (AE/Maya/Unreal).
//!
//! O que provar na tela:
//! - **Caixa + duas alças** nas bordas de tempo da seleção (as barras de accent logo FORA
//!   dos diamantes das pontas, ligadas por uma linha no topo).
//! - **Arrastar a alça DIREITA** estica/encolhe a seleção mantendo a borda ESQUERDA parada;
//!   a alça ESQUERDA faz o espelho (pivô = borda direita).
//! - **Um Ctrl+Z** desfaz o arrasto inteiro (um bracket).
//! - ⚠️ É edit de KEY: NÃO toca strip/fade nenhum (a caixa vive no dope-sheet, não na régua
//!   de strips).
//!
//! ⚠️ Se a linha `[timescale-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, SelectedKey, TimelineDoc};

/// Author a zig-zag X track (5 keys, 0..4 s) so scaling is visible as the pattern
/// compresses/expands.
fn author_zigzag(doc: &mut TimelineDoc, bits: u64) {
    let s = RationalTime::from_seconds;
    for (i, v) in [(-6.0_f32), 6.0, -3.0, 3.0, 0.0].into_iter().enumerate() {
        doc.insert_key(
            bits,
            PropKind::TranslationX,
            s(i as f64),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
}

impl crate::App {
    /// In the frame prologue, once. No-op without the env.
    pub(crate) fn timescale_smoke(&mut self) {
        if self.timescale_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_TIMESCALE_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // no world yet; try next frame
        }
        self.timescale_smoke_done = true;

        let mover = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(-6.0, 0.0)),
                    Sprite::atlas(0, [1.4, 0.4], [0.4, 0.7, 1.0, 1.0]),
                    Name::new("Bouncer"),
                ))
                .id()
                .to_bits()
        };
        author_zigzag(&mut self.timeline.doc, mover);
        // Two markers INSIDE the 0..4 span so the box carries them: they scale with
        // the keys (Enio, 2026-07-26). A third would sit outside and stay put.
        self.timeline
            .doc
            .add_marker(RationalTime::from_seconds(1.0), "beat");
        self.timeline
            .doc
            .add_marker(RationalTime::from_seconds(2.5), "hit");

        // Select EVERY key so the time-scale box shows on open. The selection lives
        // in `TimelineState.selection`; the snapshot derives `KeyView.selected` from
        // it, and `scale_drag::selection_extent` reads that.
        let keys: Vec<u64> = self
            .timeline
            .doc
            .binding_for(mover, PropKind::TranslationX)
            .and_then(|b| {
                self.timeline
                    .doc
                    .active_clip()
                    .track(b.target)
                    .map(|t| (b.target, t))
            })
            .map(|(target, track)| {
                let tgt = target.get();
                for id in track.ids() {
                    self.timeline.selection.add(SelectedKey::new(tgt, id.get()));
                }
                track.ids().iter().map(|id| id.get()).collect()
            })
            .unwrap_or_default();

        // Open the timeline (the box lives in the dope-sheet) and park at 0, paused —
        // scaling is authoring, not playback.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(mover));
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "[timescale-smoke] 1 track (X zig-zag), {} keys em 0..4 s, TODAS selecionadas, \
             + 2 markers na regua (beat@1,0 · hit@2,5) DENTRO da caixa. A timeline abre com a \
             CAIXA de selecao + 2 alcas de tempo (a esquerda separada do divisor). Arraste a \
             alca direita: a selecao E OS MARKERS esticam/encolhem mantendo a borda esquerda \
             parada; a esquerda espelha. Ctrl+Z desfaz o arrasto inteiro.",
            keys.len()
        );
    }
}
