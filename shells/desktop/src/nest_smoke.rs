//! `PH2D_NEST_SMOKE=1|2|3` — cenas PRONTAS PARA VER do **nesting** (ADR-0133).
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
//! **`=3`** é a **BIBLIOTECA** (pedido do Enio, 2026-07-22): **três** containers de uma vez —
//! um aninhado DENTRO de outro, e um VAZIO. As duas primeiras cenas têm um container cada, e
//! por isso não alcançam o que a aba **Containers** passou a ser em 2026-07-21: uma LISTA de
//! assets, com barras de tamanhos diferentes, aninhamento real (trilha de 3 níveis) e uma
//! lixeira que apaga em **CASCATA**. Ver a doc de [`App::nest_scene_library`].
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

        match scene.to_str() {
            Some("2") => self.nest_scene_jump(),
            Some("3") => self.nest_scene_library(),
            _ => self.nest_scene_walk(),
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

    /// Cena `=3` — **a BIBLIOTECA**: três containers, um DENTRO do outro, um VAZIO.
    ///
    /// As cenas `=1`/`=2` têm **um** container cada, então nenhuma delas alcança o que a aba
    /// **Containers** virou em 2026-07-21 — uma LISTA de assets. Esta monta a lista inteira:
    ///
    /// | asset | interior | onde aparece |
    /// |---|---|---|
    /// | **Step** (0) | 1 lane, 1 clip, 1,0 s | 3× dentro do Walk **e** 1× na cena |
    /// | **Walk** (1) | 2 lanes, 3,0 s — **contém o Step** | 1× na cena |
    /// | **Pause** (2) | vazio | 1× na cena |
    ///
    /// # O que olhar
    ///
    /// - **A lista tem TRÊS barras de tamanhos diferentes** (1 s · 3 s · 2 s), e a do Pause
    ///   mede 2 s **sem ter conteúdo nenhum**: é a `container_bar_seconds`, a porta única que
    ///   um container vazio responde. O bug latente que ela consertou está nesta cena — a
    ///   instância de Pause na cena tem janela REAL; encha o Pause depois e ela toca o que
    ///   você pôs, em vez de tocar nada para sempre.
    /// - **Aninhamento de verdade:** entre no Walk (duplo-clique na barra dele) e as strips
    ///   lá dentro são **containers**, não clips. Entre de novo, numa delas: a trilha fica
    ///   `[ Scene ][ Walk ][ Step ]`, e clicar **Walk** volta UM nível — não salta para a cena.
    /// - **O dropdown de FONTE tem as duas famílias** (2 clips + 3 containers): os glifos são
    ///   de desenho distinto — **folha** para clip, **caixa** para container. Escolher fonte
    ///   **não navega**.
    /// - **Canais esparsos, e é por isso que o passo anda:** o clip `Step` keya só **Y** e o
    ///   `Glide` só **X**, então dentro do Walk as duas lanes SOMAM — três quiques no lugar
    ///   viram três passos atravessando a tela. Um `TranslationX` dentro do Step não faria
    ///   isso: a posição é ABSOLUTA, e as três repetições voltariam ao mesmo x.
    /// - ⚠️ **No vão do Pause (3 s a 5 s) o objeto PARA e isso é o certo** — o container está
    ///   vazio, então não há curva dirigindo nada ali. A barra existe, a instância existe, e o
    ///   conteúdo é que não existe ainda.
    /// - **A CASCATA da lixeira** (o gesto que só esta cena expõe): apague o **Step** na lista.
    ///   Ele some de DOIS lugares — a instância solta na cena **e** as três de dentro do Walk —
    ///   e o Walk, que era o índice 1, desce para o 0 com a instância dele **ainda tocando o
    ///   Walk**, não o vizinho que escorregou para o slot. Ctrl+Z devolve tudo.
    fn nest_scene_library(&mut self) {
        let bits = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, 0.0)),
                    Sprite::atlas(0, [1.0, 1.0], [0.45, 0.85, 0.45, 1.0]),
                    Name::new("LibraryDemo"),
                ))
                .id()
                .to_bits()
        };

        build_library(&mut self.timeline.doc, bits);

        // O loop mora em DOIS lugares e os dois têm de concordar (o do playhead rebobina o
        // relógio; o do DOC é o que o avaliador lê).
        self.timeline
            .doc
            .set_active_loop_for(false, Some((0.0, LIBRARY_END)));
        self.playhead.rewind();
        self.playhead.set_loop(0.0, LIBRARY_END);
        self.playhead.play();

        eprintln!(
            "[nest-smoke] LibraryDemo: 3 containers -- Step (1.0s, 1 lane) · Walk (3.0s, 2 \
             lanes, CONTEM o Step 3x) · Pause (VAZIO, barra de 2.0s). Cena = Walk [0,3) + \
             Pause [3,5) + Step [5,6). Abra L -> aba Containers: sao TRES barras, de tamanhos \
             diferentes.\n\
             [nest-smoke]   ANINHAMENTO: duplo-clique na barra do Walk entra nele; as strips \
             de la dentro sao CONTAINERS (Step). Entre numa delas: a trilha vira \
             [ Scene ][ Walk ][ Step ] e clicar \"Walk\" volta UM nivel.\n\
             [nest-smoke]   CASCATA: apague o Step na lista (lixeira). Ele some da cena E de \
             dentro do Walk; o Walk desce de indice 1 para 0 e a instancia dele continua \
             tocando o WALK. Ctrl+Z devolve.\n\
             [nest-smoke]   O objeto PARA entre 3s e 5s -- o Pause esta vazio, e e' isso que \
             uma barra sem conteudo significa. Ponha algo dentro dele e a instancia passa a \
             tocar (era este o bug que a porta unica de tamanho consertou).\n\
             [nest-smoke]   FONTE: o dropdown lista 2 clips (folha) + 3 containers (caixa), \
             glifos de desenho distinto; escolher fonte nunca navega."
        );
    }
}

/// Quanto a cena `=3` dura — o fim da última instância.
pub(crate) const LIBRARY_END: f64 = 6.0;

/// **Monta a biblioteca da cena `=3` no `doc`** — a metade que não precisa de janela.
///
/// Separada do [`App::nest_scene_library`] para que o gate irmão dirija **esta** função, e não
/// um espelho dela: uma cena que afirma no `eprintln` o que não montou é indistinguível de
/// feature quebrada (a lição que o smoke do Colorize pagou), e o eprintln aqui afirma seis
/// números.
pub(crate) fn build_library(doc: &mut ph2d_timeline::TimelineDoc, bits: u64) {
    {
        // 1. DUAS peças, em canais DISJUNTOS — é o que deixa as lanes somarem em vez de
        //    brigarem (ADR-0115, canais esparsos: o keyado É a máscara).
        doc.rename_clip(0, "Step".to_string()); // só Y: o quique
        key(doc, bits, PropKind::TranslationY, 0.0, 0.0);
        key(doc, bits, PropKind::TranslationY, 0.5, 0.8);
        key(doc, bits, PropKind::TranslationY, 1.0, 0.0);

        let glide = doc.add_clip("Glide".to_string()); // só X: a travessia
        doc.set_active(glide);
        key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
        key(doc, bits, PropKind::TranslationX, 3.0, 3.0);
        doc.set_active(0);

        // 2. O container FOLHA: um quique, 1 s. É o asset mais fundo da árvore.
        let step = doc.add_container("Step".to_string());
        let step_host = StackHost::Container(step);
        let bounce = doc
            .add_lane_in(step_host, "Bounce".to_string())
            .expect("lane do Step");
        doc.add_strip_to(step_host, bounce, StripSource::Clip(0), 0.0, 1.0)
            .expect("o quique dentro do Step");

        // 3. O container COMPOSTO: contém o Step TRÊS vezes (aninhamento — um container
        //    referenciando outro) + a travessia por cima, numa 2ª lane.
        let walk = doc.add_container("Walk".to_string());
        let walk_host = StackHost::Container(walk);
        let steps = doc
            .add_lane_in(walk_host, "Steps".to_string())
            .expect("lane Steps");
        let step_src = StripSource::Container(u16::try_from(step).expect("cabe"));
        for i in 0..3 {
            let t = f64::from(i);
            doc.add_strip_to(walk_host, steps, step_src, t, t + 1.0)
                .expect("passo aninhado");
        }
        let glide_lane = doc
            .add_lane_in(walk_host, "Glide".to_string())
            .expect("lane Glide");
        doc.add_strip_to(
            walk_host,
            glide_lane,
            StripSource::Clip(u16::try_from(glide).expect("cabe")),
            0.0,
            3.0,
        )
        .expect("a travessia por cima dos passos");

        // 4. O container VAZIO. Nasce medindo 2 s pela `container_bar_seconds` — e a instância
        //    dele na cena, abaixo, é o caso que a porta única conserta.
        let pause = doc.add_container("Pause".to_string());

        // 5. A CENA: o composto, o vazio, e a folha SOLTA — a folha aparecer aqui E dentro do
        //    Walk é o que torna a cascata da lixeira visível em dois lugares de uma vez.
        let lane = doc.add_lane("Scene".to_string()).expect("lane da cena");
        let walk_src = StripSource::Container(u16::try_from(walk).expect("cabe"));
        let pause_src = StripSource::Container(u16::try_from(pause).expect("cabe"));
        doc.add_strip_to(StackHost::Document, lane, walk_src, 0.0, 3.0)
            .expect("o Walk na cena");
        doc.add_strip_to(StackHost::Document, lane, pause_src, 3.0, 5.0)
            .expect("o Pause vazio na cena");
        doc.add_strip_to(StackHost::Document, lane, step_src, 5.0, LIBRARY_END)
            .expect("o Step solto na cena");
    }
}

#[cfg(test)]
#[path = "nest_smoke_tests.rs"]
mod tests;

impl crate::App {
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
        let body = doc
            .add_lane_in(host, "Body".to_string())
            .expect("lane Body");
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
             lenta). Abra L: sao TRES abas agora, [Keys | Containers | Arrange].\n\
             [nest-smoke]   Arrange   = a CENA (as 3 instancias). Botao direito numa strip \
             Jump -> \"Enter Container\" leva voce para dentro dela.\n\
             [nest-smoke]   Containers = a LISTA dos containers do documento: uma strip em \
             branco por container, do tamanho DELE (o Jump mede 2s; um container vazio \
             nasce medindo 2s). A barra nao redimensiona, nao arrasta, nao corta. Tres \
             coisas -- o LAPIS renomeia, a LIXEIRA apaga (o asset E as instancias dele, um \
             undo so), e o DUPLO-CLIQUE na barra entra.\n\
             [nest-smoke]   O unico botao da coluna ali e' \"+ Container\" (cria um NOVO e \
             vazio, sem despejar nada na cena e sem entrar nele). DENTRO de um container o \
             botao vira \"+ Lane\". Edite o pico do Rise UMA vez e os TRES pulos mudam.\n\
             [nest-smoke]   Para VOLTAR a lista: toque a aba Containers estando nela (ou vá \
             pela migalha \"Scene\", que leva ao Arrange).\n\
             [nest-smoke]   Para COLOCAR: escolha a fonte no dropdown (clips com folha, \
             containers com CAIXA) e clique no \"+\" da lane -- vale no Arrange e dentro de \
             um container (e' assim que se aninha). Arraste o CORPO de uma strip para \
             CIMA/BAIXO para troca-la de lane."
        );
    }
}
