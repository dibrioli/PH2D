//! `PH2D_NEST_SMOKE=1|2` — cenas PRONTAS PARA VER do **nesting** (ADR-0133).
//!
//! **`=1`** sobe o app com **um objeto**, um clip que o faz dar UM passo, e um **container**
//! ("Walk") que toca esse passo — instanciado **duas vezes** na timeline, a segunda deslocada.
//! Tocando em loop: o objeto dá o passo, para, e dá o passo de novo — a mesma peça, usada duas
//! vezes, que é a frase inteira do nesting.
//!
//! **`=2`** é o exemplo COMPLETO (pedido do Enio, 2026-07-20): um pulo feito de **três clips
//! em duas lanes DENTRO do container** ("Jump"), instanciado **três vezes** na cena — a do
//! meio **esticada** (o `add_strip_to` deriva `speed = slice/span`, então esticar É câmera
//! lenta), as travessias entre instâncias ligadas por **lead-in** e o fim cruzando o **loop**
//! por **lead-out** (as fades desta linha). Ver a doc de [`App::nest_scene_jump`].
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
//! - **DOIS relógios na tela ao mesmo tempo, e o texto que os liga:** o chip de segundos da
//!   barra segue mostrando o tempo da CENA (ele nunca some), a régua mostra o do interior, e
//!   ao lado da trilha o readout diz **`plays 0.00 - 2.00`** / **`plays 4.00 - 6.00`** — a
//!   janela da instância em segundos da cena. É a diferença entre os dois números.
//! - ⚠️ **Arraste a régua lá dentro.** Ela busca o segundo certo da CENA (era este o bug:
//!   arrastar até o segundo 1 do interior buscava o segundo 1 da timeline, fora da instância).
//!   Arraste até as pontas: o mapa GRUDA nas bordas da instância em vez de sair dela.
//! - **Leve o playhead para fora das duas instâncias** (entre 2 s e 4 s): a marca some da
//!   régua — porque o container não toca ali — e o readout **diz** `not playing here`. As
//!   lanes seguem editáveis: você está autorando a peça, não a cena.
//! - **Sair:** clique **Scene** na trilha.
//! - **Aninhar de novo:** dentro do Walk, o botão **+ Container** faz outro container JÁ
//!   instanciado ali dentro. Botão direito nele → Enter Container: a trilha fica
//!   `[ Scene ][ Walk ][ Container 2 ]`, e clicar **Walk** volta UM nível (não salta para a
//!   cena). (Não há teto de profundidade — o ADR mediu e não achou recurso que o
//!   justificasse; o que tem teto é a TRILHA, em 8 segmentos, e ela elide por FORA: a raiz e
//!   onde você está nunca somem.)
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
        if self.nest_smoke_done {
            return;
        }
        let Some(scene) = std::env::var_os("PH2D_NEST_SMOKE") else {
            return;
        };
        if self.gfx.is_none() {
            return; // ainda sem mundo; tenta no próximo frame
        }
        self.nest_smoke_done = true;

        if scene == "2" {
            self.nest_scene_jump();
        } else {
            self.nest_scene_walk();
        }
    }

    /// Cena `=1`: o mínimo do nesting — um passo, um container, duas instâncias.
    fn nest_scene_walk(&mut self) {
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

    /// Cena `=2` — **"o mesmo pulo, três vezes"**: o exemplo completo do diferencial.
    ///
    /// # O que olhar
    ///
    /// - **A cena tem UMA lane com três strips "Jump"** — e cada strip é um pulo INTEIRO
    ///   (subida + queda com quique + deslocamento lateral). O emaranhado de clips que o
    ///   compõe não está na cena: está DENTRO do container, uma vez.
    /// - **Entrar:** botão direito numa strip "Jump" → **Enter Container**. Lá dentro há
    ///   **duas lanes e três strips de clip**: a lane "Body" toca `Rise` e depois `Fall`,
    ///   **sobrepostos por 0,2 s** — a sobreposição É o crossfade do ápice; a lane "Drift"
    ///   toca `Drift` (só X) por cima — canais esparsos: `Rise`/`Fall` keyam só Y, `Drift`
    ///   só X, então as lanes se SOMAM em vez de brigar.
    /// - **A instância do meio é CÂMERA LENTA e nenhum keyframe mudou:** a strip [3,7)
    ///   tem o dobro do span da peça (2 s), e `add_strip_to` deriva `speed = slice/span`
    ///   = 0,5. Retime é da INSTÂNCIA, não da fonte — as outras duas seguem normais.
    /// - **Edite o interior UMA vez e veja os TRÊS mudarem:** dentro do Jump, arraste o
    ///   key do pico de `Rise` (Y 1,2) para mais alto e saia — os três pulos da cena
    ///   ficam mais altos. É isto que lanes soltas não fazem: lá seriam três cópias.
    /// - **As travessias entre instâncias são leads** (as fades desta linha): o objeto
    ///   viaja da pose final de um pulo até a pose inicial do próximo DURANTE o vão
    ///   (`lead_in` 0,6 s), e o fim da última strip cruza a costura do **loop** de volta
    ///   ao primeiro pulo (`lead_out` 0,5 s) — sem nenhum salto no ciclo inteiro.
    fn nest_scene_jump(&mut self) {
        let bits = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, 0.0)),
                    Sprite::atlas(0, [1.0, 1.0], [1.0, 0.62, 0.25, 1.0]),
                    Name::new("JumpDemo"),
                ))
                .id()
                .to_bits()
        };

        let doc = &mut self.timeline.doc;

        // 1. TRÊS clips — as peças do pulo. Os keys caem no clip ATIVO, então cada
        //    lote troca a seleção antes de escrever.
        doc.rename_clip(0, "Rise".to_string());
        key(doc, bits, PropKind::TranslationY, 0.0, 0.0);
        key(doc, bits, PropKind::TranslationY, 1.1, 1.2);

        let fall = doc.add_clip("Fall".to_string());
        doc.set_active(fall);
        key(doc, bits, PropKind::TranslationY, 0.0, 1.2);
        key(doc, bits, PropKind::TranslationY, 0.9, 0.0);
        key(doc, bits, PropKind::TranslationY, 1.0, 0.18); // o quique da aterrissagem
        key(doc, bits, PropKind::TranslationY, 1.1, 0.0);

        let drift = doc.add_clip("Drift".to_string());
        doc.set_active(drift);
        key(doc, bits, PropKind::TranslationX, 0.0, -1.5);
        key(doc, bits, PropKind::TranslationX, 2.0, 1.5);
        doc.set_active(0);

        // 2. O CONTAINER "Jump": duas lanes, três strips — a montagem que, sem
        //    container, teria de ser re-feita (ou re-colada) a cada uso.
        let jump = doc.add_container("Jump".to_string());
        let host = StackHost::Container(jump);
        let clip_of = |i: usize| StripSource::Clip(u16::try_from(i).expect("cabe"));
        let body = doc.add_lane_in(host, "Body".to_string()).expect("lane Body");
        doc.add_strip_to(host, body, clip_of(0), 0.0, 1.1)
            .expect("Rise no Body");
        // Sobreposto 0,2 s ao Rise: a sobreposição É o crossfade (ADR-0115).
        doc.add_strip_to(host, body, clip_of(fall), 0.9, 2.0)
            .expect("Fall no Body");
        let side = doc
            .add_lane_in(host, "Drift".to_string())
            .expect("lane Drift");
        doc.add_strip_to(host, side, clip_of(drift), 0.0, 2.0)
            .expect("Drift na lane de cima");

        // 3. TRÊS instâncias na cena — a peça inteira é UMA strip cada vez. A do meio
        //    ocupa o dobro do span, e o `add_strip_to` deriva `speed = slice/span`:
        //    esticar a strip É a câmera lenta, sem tocar um keyframe.
        let lane = doc.add_lane("Jumps".to_string()).expect("lane da cena");
        let src = StripSource::Container(u16::try_from(jump).expect("cabe"));
        doc.add_strip_to(StackHost::Document, lane, src, 0.0, 2.0)
            .expect("1a instância");
        let slow = doc
            .add_strip_to(StackHost::Document, lane, src, 3.0, 7.0)
            .expect("2a instância, esticada");
        let last = doc
            .add_strip_to(StackHost::Document, lane, src, 8.0, 10.0)
            .expect("3a instância");

        // 4. As travessias: lead-in atravessa o vão até a pose inicial da instância, e o
        //    lead-out da última cruza a costura do loop de volta ao primeiro pulo.
        if let Some(s) = doc.strip_in_mut(StackHost::Document, lane, slow) {
            s.lead_in = 0.6;
        }
        if let Some(s) = doc.strip_in_mut(StackHost::Document, lane, last) {
            s.lead_in = 0.6;
            s.lead_out = 0.5;
        }

        // O loop mora em DOIS lugares e os dois têm de concordar: o do playhead rebobina o
        // relógio, o do DOC é o que o avaliador da costura lê (`seam_source`) — sem ele o
        // lead-out da última strip não teria costura para cruzar.
        doc.set_active_loop_for(false, Some((0.0, 10.6)));
        self.playhead.rewind();
        self.playhead.set_loop(0.0, 10.6);
        self.playhead.play();

        eprintln!(
            "[nest-smoke] JumpDemo: container \"Jump\" = 3 clips (Rise/Fall/Drift) em 2 lanes, \
             instanciado 3x em [0,2) [3,7) [8,10) — a do meio esticada (speed 0.5 = camera \
             lenta). Abra L -> aba Arrange. Botao direito numa strip Jump -> \"Enter \
             Container\": edite o pico do Rise UMA vez e os TRES pulos mudam."
        );
    }
}
