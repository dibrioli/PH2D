//! `PH2D_STACK_SMOKE=1` — uma cena PRONTA PARA VER da composição de clips (ADR-0115).
//!
//! Sobe o app com **um objeto**, **duas clips** que animam o MESMO X (uma segura o objeto à
//! esquerda, a outra à direita) e **uma lane com dois strips que se SOBREPÕEM**. Tocando em loop:
//! o objeto fica à esquerda, **atravessa** para a direita durante a sobreposição (o crossfade
//! sai só do blend — ninguém digitou duração), e fica à direita.
//!
//! É o antídoto da montagem-na-mão ([[feedback_ready_to_smoke_example]]): abra o painel com **L**
//! e o sistema já está rodando. Para experimentar:
//!
//! - **arraste a borda lateral** de um strip → **TRIM** (muda QUAIS frames tocam; a taxa fica).
//! - **Cmd/Ctrl + a mesma borda** → **STRETCH** (dilata o TEMPO; a mesma animação, mais devagar).
//! - **a barrinha na quina de cima** → **FADE** do próprio strip (só aparece onde não há vizinho).
//! - **arraste um strip para dentro do outro** → a sobreposição É o crossfade.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::PropKind;

/// Uma key de `TranslationX`/`Y` na clip ATIVA, para o objeto `bits`.
fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, prop: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        prop,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn stack_smoke(&mut self) {
        if self.stack_smoke_done || std::env::var_os("PH2D_STACK_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // ainda sem mundo; tenta no próximo frame
        }
        self.stack_smoke_done = true;

        // 1. O objeto: grande, no centro, SEM `Velocity` — só a timeline o move (o resto da cena
        //    de boot quica; este fica parado até a animação pegá-lo).
        let bits = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, 0.0)),
                    Sprite::atlas(0, [1.0, 1.0], [1.0, 0.45, 0.15, 1.0]),
                    Name::new("StackDemo"),
                ))
                .id()
                .to_bits()
        };

        // 2. Duas clips que animam o MESMO alvo (o X do objeto), para valores diferentes — é isto
        //    que faz a sobreposição ser um crossfade VISÍVEL: X vai do valor de uma para o da
        //    outra. Um leve sobe-desce em Y mantém as duas vivas.
        let doc = &mut self.timeline.doc;

        // Clip 0 ("Left"): segura à esquerda, balançando.
        doc.rename_clip(0, "Left".to_string());
        key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
        key(doc, bits, PropKind::TranslationX, 3.0, -3.0);
        key(doc, bits, PropKind::TranslationY, 0.0, 0.0);
        key(doc, bits, PropKind::TranslationY, 1.5, 1.2);
        key(doc, bits, PropKind::TranslationY, 3.0, 0.0);

        // Clip 1 ("Right"): segura à direita, balançando ao contrário.
        let right = doc.add_clip("Right".to_string());
        doc.set_active(right);
        key(doc, bits, PropKind::TranslationX, 0.0, 3.0);
        key(doc, bits, PropKind::TranslationX, 3.0, 3.0);
        key(doc, bits, PropKind::TranslationY, 0.0, 0.0);
        key(doc, bits, PropKind::TranslationY, 1.5, -1.2);
        key(doc, bits, PropKind::TranslationY, 3.0, 0.0);
        doc.set_active(0);

        // 3. Uma lane, dois strips que se SOBREPÕEM. Cada strip nasce do comprimento REAL da sua
        //    clip (a última key = 3 s), a speed 1 — a correção do bug das "duas portas". A
        //    sobreposição de 1 s ([2, 3)) é o crossfade, e ninguém digitou duração nenhuma.
        let lane = doc.add_lane("Lane 1".to_string()).expect("lane");
        let len0 = doc.clip_end_seconds(0); // 3 s
        let len1 = doc.clip_end_seconds(right); // 3 s
        doc.add_strip(lane, 0, 0.0, len0); // Left  : [0, 3)
        doc.add_strip(lane, right, 2.0, 2.0 + len1); // Right : [2, 5) — 1 s de overlap

        // 4. Toca em loop sobre a cena inteira, para o crossfade acontecer sem parar.
        self.playhead.rewind();
        self.playhead.set_loop(0.0, 2.0 + len1);
        self.playhead.play();

        eprintln!(
            "[stack-smoke] StackDemo: Left[0,3) x Right[2,5), overlap [2,3) = crossfade. Abra L."
        );
    }
}
