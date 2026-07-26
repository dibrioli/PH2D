//! **Smoke da extrapolação por-track** (joia da coroa §6). `PH2D_EXTRAP_SMOKE=1`:
//!
//! UM objeto com uma track de X keyada SÓ no primeiro segundo (uma rampa 0 -> 3 m
//! em `[0, 1]`), a timeline ABERTA e TOCANDO em loop `[0, 4]`. Por padrão a track
//! flat-clampa (Hold): o objeto desliza 0 -> 3 no 1º segundo e depois **para** em
//! 3 até o loop voltar. A extrapolação por-track é o loopOut / cycle / pingpong /
//! continue do After Effects: o que a track faz ALÉM das keys, Pre e Post
//! independentes.
//!
//! O controle vive no menu de BOTÃO DIREITO da LABEL da track (a coluna de nomes à
//! esquerda), em duas cascatas: **Extrapolation Pre ▸** e **Extrapolation Post ▸**,
//! cada uma abrindo Hold / Loop / Ping-Pong / Continue.
//!
//! O que provar na tela (com o playhead correndo em `[0, 4]`):
//! 1. Como está (Post = Hold): o objeto anda 0 -> 3 em 1 s e CONGELA em 3 até o loop.
//! 2. R-click na LABEL da track -> **Extrapolation Post ▸ -> Loop**: agora o objeto
//!    CICLA (0 -> 3, salta pra 0, repete) durante todo o `[0, 4]`.
//! 3. Troque para **Ping-Pong**: ele vai-e-volta (0 -> 3 -> 0), refletindo.
//! 4. Troque para **Continue**: ele passa de 3 e CONTINUA subindo pela reta (slope 3/s).
//! 5. **Pre** é independente (scrube para antes de 0 para ver, ou anime o loop).
//! 6. Nenhum strip/fade se mexe (é edit de KEY, via edit/settle) — e uma track de
//!    **Time Remap** não oferece o controle (o menu dela é só Delete Track).
//!
//! ⚠️ Se a linha `[extrap-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, TimelineDoc};

/// A short ramp `0 -> 3` over `[0, 1]s` — keyed only in the first second, so the
/// rest of the `[0, 4]` loop is ALL extrapolation: Hold holds, Loop cycles,
/// Continue keeps climbing.
fn author_ramp(doc: &mut TimelineDoc, bits: u64) {
    let s = RationalTime::from_seconds;
    doc.insert_key(
        bits,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        bits,
        PropKind::TranslationX,
        s(1.0),
        AnimValue::Float(3.0),
        Interp::Linear,
    );
}

impl crate::App {
    /// In the frame prologue, once. No-op without the env.
    pub(crate) fn extrap_smoke(&mut self) {
        if self.extrap_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_EXTRAP_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // no world yet; try next frame
        }
        self.extrap_smoke_done = true;

        let bits = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, 0.0)),
                    Sprite::atlas(0, [0.5, 0.5], [1.0, 0.7, 0.3, 1.0]),
                    Name::new("Loop Me"),
                ))
                .id()
                .to_bits()
        };
        author_ramp(&mut self.timeline.doc, bits);

        // Open the timeline, loop the transport over [0, 4] and PLAY — the keys end
        // at 1 s, so [1, 4] is all extrapolation. Default Hold freezes there; the
        // artist right-clicks the track label to pick Loop / Ping-Pong / Continue.
        self.timeline
            .doc
            .set_active_loop_for(false, Some((0.0, 4.0)));
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(bits));
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.seek(0.0);
        self.playhead.play();

        eprintln!(
            "[extrap-smoke] 1 track (X, rampa 0->3 keyada so em [0,1]), timeline aberta \
             tocando em loop [0,4]. Por padrao (Hold) o objeto para em 3 apos 1s. R-CLICK \
             na LABEL da track -> Extrapolation Post -> Loop/Ping-Pong/Continue para ve-lo \
             ciclar/refletir/continuar alem das keys. Nenhum strip/fade se mexe; Time Remap \
             nao oferece o controle."
        );
    }
}
