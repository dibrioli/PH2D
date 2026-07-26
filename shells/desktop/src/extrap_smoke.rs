//! **Smoke da extrapolação por-track** (joia da coroa §6). `PH2D_EXTRAP_SMOKE=1`:
//!
//! UM objeto com uma track de X keyada SÓ NO MEIO (uma rampa 0 -> 3 m em
//! `[1.5, 2.5]`), a timeline ABERTA e TOCANDO em loop `[0, 4]`. As keys no meio
//! abrem as DUAS zonas de extrapolação numa cena só:
//!   - **`[0, 1.5)` = zona do PRE** (antes da 1ª key);
//!   - **`(2.5, 4]` = zona do POST** (depois da última).
//! Por padrão (Hold/Hold) o objeto fica em 0 na entrada, sobe 0 -> 3 no meio, e
//! congela em 3 no fim. Pre e Post são INDEPENDENTES.
//!
//! O controle vive no menu de BOTÃO DIREITO da LABEL da track (a coluna de nomes à
//! esquerda), em duas cascatas: **Extrapolation Pre ▸** e **Extrapolation Post ▸**,
//! cada uma abrindo Hold / Loop / Ping-Pong / Continue.
//!
//! O que provar (playhead correndo em `[0, 4]`; R-click na LABEL da track):
//! 1. **POST -> Loop**: em `(2.5, 4]` o objeto CICLA (salta e repete) em vez de parar.
//! 2. **POST -> Ping-Pong**: em `(2.5, 4]` ele vai-e-volta (reflete).
//! 3. **POST -> Continue**: em `(2.5, 4]` ele CONTINUA subindo pela reta (slope 3/s).
//! 4. **PRE -> Continue**: em `[0, 1.5)` ele já CHEGA em movimento (entra na 1ª key
//!    com velocidade, em vez de esperar parado em 0).
//! 5. **PRE -> Loop / Ping-Pong**: em `[0, 1.5)` o ciclo já roda ANTES do trecho keyado.
//! 6. Pre e Post ligados juntos (ex.: os dois Loop) = oscilação infinita nas duas pontas.
//! 7. Nenhum strip/fade se mexe (é edit de KEY, via edit/settle) — e uma track de
//!    **Time Remap** não oferece o controle (o menu dela é só Delete Track).
//!
//! ⚠️ Se a linha `[extrap-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, TimelineDoc};

/// A short ramp `0 -> 3` keyed in the MIDDLE of the loop (`[1.5, 2.5]s`), so the
/// `[0, 4]` loop has a PRE zone (`[0, 1.5)`, before the first key) AND a POST zone
/// (`(2.5, 4]`, after the last) — both extrapolation, visible in one scene.
fn author_ramp(doc: &mut TimelineDoc, bits: u64) {
    let s = RationalTime::from_seconds;
    doc.insert_key(
        bits,
        PropKind::TranslationX,
        s(1.5),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        bits,
        PropKind::TranslationX,
        s(2.5),
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

        // Open the timeline, loop the transport over [0, 4] and PLAY — the keys sit
        // in [1.5, 2.5], so [0, 1.5) is the PRE zone and (2.5, 4] the POST zone.
        // Default Hold/Hold freezes both; the artist right-clicks the track label to
        // pick Pre / Post extrapolation independently.
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
            "[extrap-smoke] 1 track (X, rampa 0->3 keyada so em [1.5,2.5]), timeline aberta \
             tocando em loop [0,4]. Zona PRE = [0,1.5), zona POST = (2.5,4]. Por padrao \
             (Hold/Hold) o objeto fica em 0 na entrada e congela em 3 no fim. R-CLICK na \
             LABEL da track -> Extrapolation Pre/Post -> Loop/Ping-Pong/Continue, cada lado \
             independente. Nenhum strip/fade se mexe; Time Remap nao oferece o controle."
        );
    }
}
