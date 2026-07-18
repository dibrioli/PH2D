//! `PH2D_NEST_SMOKE=1` — uma cena PRONTA PARA VER do **nesting** (ADR-0133).
//!
//! Sobe o app com **um objeto**, um clip que o faz dar UM passo, e um **container** ("Walk")
//! que toca esse passo — instanciado **duas vezes** na timeline, a segunda deslocada. Tocando
//! em loop: o objeto dá o passo, para, e dá o passo de novo — a mesma peça, usada duas vezes,
//! que é a frase inteira do nesting.
//!
//! É o antídoto da montagem-na-mão ([[feedback_ready_to_smoke_example]]): abra o painel com
//! **L**, clique a aba **Arrange**, e as duas instâncias já estão lá.
//!
//! # O que olhar
//!
//! - **As duas strips "Walk"** na lane: a mesma peça, dois lugares. Arraste uma — ela leva o
//!   conteúdo junto, porque o container é a fonte e a strip é só onde ela toca.
//! - **Entrar:** botão direito numa strip "Walk" → **Enter Container**. A tela troca para o
//!   INTERIOR dele, e aparece a trilha `[ Scene ][ Walk ]` na barra de transporte.
//! - ⚠️ **A régua muda de relógio ao entrar.** Lá fora ela mede a timeline; lá dentro mede os
//!   segundos DO CONTAINER — a 2ª instância começa em 4 s na cena, mas por dentro ela também
//!   começa em 0. É por isso que a marca do playhead cai sobre o conteúdo, em vez de meio
//!   traço fora dele. Passe o playhead pelas duas instâncias e repare que o interior é o
//!   MESMO nos dois lugares.
//! - **Sair:** clique **Scene** na trilha.
//! - **Aninhar de novo:** dentro do Walk, o botão **+ Container** faz outro, um nível abaixo.
//!   A trilha cresce. (Não há teto de profundidade — o ADR mediu e não achou recurso que o
//!   justificasse; o que tem teto é a TRILHA, em 8 segmentos.)
//! - **Ciclo:** não há gesto que o crie — `try_nest` recusa antes de aceitar o link.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, StackHost, StripSource};

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
    pub(crate) fn nest_smoke(&mut self) {
        if self.nest_smoke_done || std::env::var_os("PH2D_NEST_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // ainda sem mundo; tenta no próximo frame
        }
        self.nest_smoke_done = true;

        let bits = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, 0.0)),
                    Sprite::atlas(0, [1.0, 1.0], [0.25, 0.65, 1.0, 1.0]),
                    Name::new("NestDemo"),
                ))
                .id()
                .to_bits()
        };

        let doc = &mut self.timeline.doc;

        // 1. O clip: UM passo. Sobe, avança, desce — 2 s.
        doc.rename_clip(0, "Step".to_string());
        key(doc, bits, PropKind::TranslationX, 0.0, -2.0);
        key(doc, bits, PropKind::TranslationX, 2.0, 0.0);
        key(doc, bits, PropKind::TranslationY, 0.0, 0.0);
        key(doc, bits, PropKind::TranslationY, 1.0, 1.0);
        key(doc, bits, PropKind::TranslationY, 2.0, 0.0);
        let step = doc.clip_end_seconds(0); // 2 s

        // 2. O CONTAINER: uma lane, o passo dentro dele. É a "peça".
        let walk = doc.add_container("Walk".to_string());
        let inner = doc
            .add_lane_in(StackHost::Container(walk), "Steps".to_string())
            .expect("lane interna");
        doc.add_strip_to(
            StackHost::Container(walk),
            inner,
            StripSource::Clip(0),
            0.0,
            step,
        )
        .expect("o passo dentro do container");

        // 3. Duas INSTÂNCIAS na timeline, separadas por uma pausa. **A mesma peça, dois
        //    lugares** — é isto que o nesting compra, e é o que a composição de clips (que só
        //    faz a transição entre dois estados) não comprava.
        let lane = doc.add_lane("Timeline".to_string()).expect("lane");
        let src = StripSource::Container(u16::try_from(walk).expect("cabe"));
        doc.add_strip_to(StackHost::Document, lane, src, 0.0, step)
            .expect("1a instância");
        doc.add_strip_to(StackHost::Document, lane, src, 4.0, 4.0 + step)
            .expect("2a instância");

        // 4. Toca em loop sobre as duas, para a repetição ficar óbvia sem ninguém apertar nada.
        self.playhead.rewind();
        self.playhead.set_loop(0.0, 4.0 + step);
        self.playhead.play();

        eprintln!(
            "[nest-smoke] NestDemo: container \"Walk\" (um passo de {step} s) instanciado em \
             [0,{step}) e [4,{}). Abra L -> aba Arrange. Botao direito numa strip Walk -> \
             \"Enter Container\" para entrar; clique \"Scene\" na trilha para sair.",
            4.0 + step
        );
    }
}
