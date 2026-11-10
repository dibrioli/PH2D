//! O smoke `PH2D_NEST_SMOKE`. Gestos e o porquê: `docs/Timeline/smoke_nest.md`.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, StackHost, StripLoop, StripSource, TimelineDoc};

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
            Some(n @ ("4" | "5")) => self.nest_scene_montagem(n == "5"),
            _ => self.nest_scene_walk(),
        }
    }

    /// Cenas `=4` (Arrange, a tocar) e `=5` (a LISTA de Containers): a montagem da maquete B.
    fn nest_scene_montagem(&mut self, lista: bool) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let bits = gfx
            .sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                Sprite::atlas(0, [1.0, 1.0], [0.95, 0.7, 0.3, 1.0]),
                Name::new("Hero"),
            ))
            .id()
            .to_bits();
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("timeline", true);
        }
        build_montagem(&mut self.timeline.doc, bits);
        self.playhead.rewind();
        self.playhead.set_loop(0.0, MONTAGEM_END);
        let aba = if lista {
            ph2d_panel_timeline::state::request_containers_tab();
            "Containers: a LISTA, cada linha com \"used 2x\" / \"not placed\""
        } else {
            ph2d_panel_timeline::state::request_arrange_tab();
            self.playhead.play();
            "Arrange, a tocar [0,6): as sobreposicoes sao mesclas"
        };
        eprintln!(
            "[nest-smoke] Hero: clips idle walk jump swing shake; Run & jump 2x na cena, Idle \
             pack nao posto, Step 1x dentro do Idle pack; faixas Base 1.0, Upper body 0.6, FX \
             MUDA (o olho solta o shake de +-0.6 X em [2.75, 3.75)). Abre em {aba}."
        );
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
        let host = StackHost::Container(walk);
        let inner = doc.add_lane_in(host, "Steps".into()).expect("lane interna");
        doc.add_strip_to(host, inner, StripSource::Clip(0), 0.0, step)
            .expect("o passo dentro do container");

        // 3. Duas INSTÂNCIAS separadas por uma pausa: a mesma peça, dois lugares.
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

    /// Cena `=3` — **a BIBLIOTECA**: três containers, um DENTRO do outro, um VAZIO. O que olhar:
    /// `docs/Timeline/smoke_nest.md`.
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

        // O loop mora no playhead (o relógio) E no doc (o que o avaliador lê).
        self.timeline
            .doc
            .set_active_loop_for(false, Some((0.0, LIBRARY_END)));
        self.playhead.rewind();
        self.playhead.set_loop(0.0, LIBRARY_END);
        self.playhead.play();

        eprintln!(
            "[nest-smoke] LibraryDemo: Step (1.0s) · Walk (3.0s, CONTEM o Step 3x) · Pause \
             (VAZIO, 2.0s). Cena = Walk [0,3) + Pause [3,5) + Step [5,6). Abra L -> aba \
             Containers: TRES barras. Gestos: docs/Timeline/smoke_nest.md."
        );
    }
}

/// Quanto a cena `=3` dura — o fim da última instância.
pub(crate) const LIBRARY_END: f64 = 6.0;

/// **Monta a biblioteca da cena `=3` no `doc`**, sem janela: o gate irmão dirige ESTA função, não
/// um espelho (uma cena que afirma o que não montou é indistinguível de feature quebrada).
pub(crate) fn build_library(doc: &mut TimelineDoc, bits: u64) {
    // 1. DUAS peças em canais DISJUNTOS: as lanes somam (ADR-0115, o keyado É a máscara).
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

    // 3. O container COMPOSTO: o Step TRÊS vezes (aninhado) + a travessia numa 2ª lane.
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

    // 4. O container VAZIO: mede 2 s pela `container_bar_seconds` (a porta única).
    let pause = doc.add_container("Pause".to_string());

    // 5. A CENA: o composto, o vazio e a folha SOLTA (a cascata da lixeira em dois lugares).
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

/// Quanto a montagem das cenas `=4`/`=5` dura.
pub(crate) const MONTAGEM_END: f64 = 6.0;

type Clipe = (&'static str, f64, [(PropKind, f64, f32); 3]);
type Faixa<'a> = (StackHost, &'a str, f64, bool, &'a [(StripSource, f64, f64)]);
/// **Monta as cenas `=4`/`=5`** (a maquete B), sem janela: 5 clips; `Run & jump` 2× na cena,
/// `Idle pack` em lado nenhum, `Step` 1× dentro do `Idle pack`. Um clip toca a velocidade 1.
pub(crate) fn build_montagem(doc: &mut TimelineDoc, bits: u64) {
    use PropKind::{Rotation as R, TranslationX as X, TranslationY as Y};
    #[rustfmt::skip]
    let clips: [Clipe; 5] = [
        ("idle", 2.0, [(Y, 0.0, 0.0), (Y, 1.0, 0.1), (Y, 2.0, 0.0)]),
        ("walk", 0.75, [(X, 0.0, -1.0), (Y, 0.4, 0.15), (X, 0.75, 1.0)]),
        ("jump", 1.0, [(Y, 0.0, 0.0), (Y, 0.5, 1.2), (Y, 1.0, 0.0)]),
        ("swing", 0.5, [(R, 0.0, 0.0), (R, 0.25, 0.6), (R, 0.5, 0.0)]),
        ("shake", 0.25, [(X, 0.0, -0.6), (X, 0.125, 0.6), (X, 0.25, -0.6)]),
    ];
    let mut ix = [0_u16; 5];
    for (i, (name, len, keys)) in clips.into_iter().enumerate() {
        let c = if i == 0 { 0 } else { doc.add_clip(name.into()) };
        doc.rename_clip(c, name.into());
        doc.set_active(c);
        for (prop, t, v) in keys {
            key(doc, bits, prop, t, v);
        }
        doc.set_clip_length_override(c, Some(len));
        ix[i] = u16::try_from(c).expect("cabe");
    }
    doc.set_active(0);
    doc.set_scene_length(Some(MONTAGEM_END)); // senão o véu de 4 s listra [4, 6)
    doc.set_active_loop_for(false, Some((0.0, MONTAGEM_END)));
    let [idle, walk, jump, swing, shake] = ix.map(StripSource::Clip);
    let [run, pack, step] =
        ["Run & jump", "Idle pack", "Step"].map(|n| doc.add_container(n.into()));
    let src = |c: usize| StripSource::Container(u16::try_from(c).expect("cabe"));
    let (scene, inside) = (StackHost::Document, StackHost::Container);
    #[rustfmt::skip]
    let lanes: [Faixa<'_>; 7] = [
        (inside(run), "Base", 1.0, false, &[(walk, 0.0, 1.25), (jump, 1.0, 2.0)]),
        (inside(run), "Upper body", 1.0, false, &[(swing, 1.25, 1.75)]),
        (inside(step), "Base", 1.0, false, &[(walk, 0.0, 0.75)]),
        (inside(pack), "Base", 1.0, false, &[(idle, 0.0, 2.0), (src(step), 2.0, 2.75)]),
        (scene, "Base", 1.0, false, &[(idle, 0.0, 1.33), (src(run), 1.0, 3.0),
                                      (idle, 2.75, 4.25), (src(run), 4.0, 6.0)]),
        (scene, "Upper body", 0.6, false, &[(swing, 1.67, 2.17)]),
        (scene, "FX", 1.0, true, &[(shake, 2.75, 3.75)]),
    ];
    for (host, name, weight, muted, strips) in lanes {
        let l = doc.add_lane_in(host, name.into()).expect("lane");
        for &(source, t0, t1) in strips {
            let id = doc.add_strip_to(host, l, source, t0, t1).expect("peca");
            let s = doc.strip_in_mut(host, l, id).expect("peca");
            if source.clip_index().is_some() {
                // Cortado ao vão, ou em loop quando o vão é maior que o clip.
                if s.src_out < t1 - t0 {
                    s.loop_mode = StripLoop::Loop;
                }
                (s.src_out, s.speed) = (s.src_out.min(t1 - t0), 1.0);
            }
        }
        let ln = &mut doc.host_stack_mut(host).expect("host")[l];
        (ln.weight, ln.muted) = (weight, muted);
    }
}

#[cfg(test)]
#[path = "nest_smoke_tests.rs"]
mod tests;

impl crate::App {
    /// Cena `=2` — **"o mesmo pulo, três vezes"**: o exemplo completo do diferencial. O que
    /// olhar: `docs/Timeline/smoke_nest.md`.
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

        // 1. TRÊS clips; os keys caem no clip ATIVO, então cada lote troca a seleção antes.
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

        // 2. O CONTAINER "Jump": duas lanes, três strips, montado UMA vez.
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

        // 3. TRÊS instâncias; a do meio tem o dobro do span: `speed = slice/span` = 0,5.
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

        // 4. As travessias: lead-in pelo vão; o lead-out da última cruza a costura do loop.
        if let Some(s) = doc.strip_in_mut(StackHost::Document, lane, slow) {
            s.lead_in = 0.6;
        }
        if let Some(s) = doc.strip_in_mut(StackHost::Document, lane, last) {
            s.lead_in = 0.6;
            s.lead_out = 0.5;
        }

        // O loop no doc é o que a costura lê (`seam_source`): sem ele o lead-out não a cruza.
        doc.set_active_loop_for(false, Some((0.0, 10.6)));
        self.playhead.rewind();
        self.playhead.set_loop(0.0, 10.6);
        self.playhead.play();

        eprintln!(
            "[nest-smoke] JumpDemo: container \"Jump\" = 3 clips (Rise/Fall/Drift) em 2 lanes, \
             instanciado 3x em [0,2) [3,7) [8,10) — a do meio esticada (speed 0.5 = camera \
             lenta). Abra L: [Keys | Containers | Arrange]. Gestos: docs/Timeline/smoke_nest.md."
        );
    }
}
