//! **Smoke do stagger / distribute** (joias da coroa §3). `PH2D_STAGGER_SMOKE=1`:
//!
//! TRÊS objetos, cada um com uma track de X cujas keys estão AMONTOADAS perto do
//! começo ({0, 0.3, 0.6, 2.0}), com TODAS as keys JÁ SELECIONADAS e a timeline
//! ABERTA. Dois verbos, ambos só sobre KEYS (nunca strips — o fade é intocável):
//!
//! - **Quick-Offset (Alt+arrastar OU Ctrl+arrastar uma key):** a seleção CASCATEIA
//!   — o 1º objeto fica, cada seguinte desloca `rank·passo` (a cascata de
//!   motion-graphics). O passo É a distância do arrasto; solte e a cascata pousa em
//!   um Ctrl+Z. ⚠️ Use **Ctrl+arrastar** no KDE/Linux: o compositor rouba o
//!   Alt+arrastar (gesto de mover-janela) e o app nunca vê o arrasto.
//! - **Distribute (Ctrl+E):** as keys de cada track se re-espaçam UNIFORMEMENTE
//!   entre a primeira e a última ({0, 0.3, 0.6, 2.0} -> {0, 0.667, 1.333, 2.0}) —
//!   as pontas ficam, os miolos deslizam. Por track, independente.
//!
//! O que provar na tela:
//! - Ctrl+arrastar (ou Alt, fora do KDE) uma das keys ESCALONA as três linhas (a
//!   de cima fica parada).
//! - Ctrl+E espalha os miolos de cada linha uniformemente.
//! - Um Ctrl+Z desfaz o gesto inteiro.
//! - Nenhum strip/fade se mexe (os dois verbos vivem no dope-sheet).
//!
//! ⚠️ Se a linha `[stagger-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, SelectedKey, TimelineDoc};

/// Author a track of four X keys bunched near the start, so both verbs read: the
/// stagger cascade shifts the whole clump; distribute spreads it out.
fn author_bunched(doc: &mut TimelineDoc, bits: u64) {
    let s = RationalTime::from_seconds;
    for (t, v) in [(0.0_f64, -6.0_f32), (0.3, 6.0), (0.6, -3.0), (2.0, 3.0)] {
        doc.insert_key(
            bits,
            PropKind::TranslationX,
            s(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
}

impl crate::App {
    /// In the frame prologue, once. No-op without the env.
    pub(crate) fn stagger_smoke(&mut self) {
        if self.stagger_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_STAGGER_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // no world yet; try next frame
        }
        self.stagger_smoke_done = true;

        // Three named movers on distinct rows so the cascade is visible top-to-bottom.
        let mut movers = Vec::new();
        {
            let gfx = self.gfx.as_mut().expect("gfx");
            for (i, name) in ["Alpha", "Bravo", "Charlie"].into_iter().enumerate() {
                let y = 2.0 - i as f32 * 2.0;
                let bits = gfx
                    .sim
                    .world_mut()
                    .spawn((
                        Transform::from_translation(Vec2::new(-6.0, y)),
                        Sprite::atlas(0, [1.4, 0.4], [0.4, 0.7, 1.0, 1.0]),
                        Name::new(name),
                    ))
                    .id()
                    .to_bits();
                movers.push(bits);
            }
        }
        for &bits in &movers {
            author_bunched(&mut self.timeline.doc, bits);
        }

        // Select EVERY key across all three tracks — both verbs act on the whole
        // multi-track selection.
        let mut total = 0usize;
        for &bits in &movers {
            if let Some(b) = self.timeline.doc.binding_for(bits, PropKind::TranslationX) {
                let target = b.target;
                if let Some(track) = self.timeline.doc.active_clip().track(target) {
                    let tgt = target.get();
                    for id in track.ids() {
                        self.timeline.selection.add(SelectedKey::new(tgt, id.get()));
                        total += 1;
                    }
                }
            }
        }

        // Open the timeline (both verbs live in the dope-sheet) and park at 0, paused —
        // stagger and distribute are authoring, not playback.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(movers.first().copied());
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "[stagger-smoke] 3 tracks (X amontoado em 0..2 s), {total} keys, TODAS \
             selecionadas. Ctrl+ARRASTAR (ou Alt fora do KDE) uma key CASCATEIA as 3 \
             linhas (a de cima fica); Ctrl+E DISTRIBUI os miolos de cada linha \
             uniformemente. Um Ctrl+Z desfaz. Nenhum strip/fade se mexe."
        );
    }
}
