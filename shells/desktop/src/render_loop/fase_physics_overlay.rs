//! **Fase do quadro: A SOBREPOSIÇÃO DA FÍSICA** — o contorno dos colliders e tudo o que a física sabe e a tela não mostrava: juntas, rodas,
//! gravidade, sensores, contatos, flashes, água e sondas (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    /// ⛔ **Ela já não recebe a janela, e a ausência é a lei:** desde 19/09 todo mapeamento
    /// mundo↔tela desta fase passa pela BANDA da cena ([`crate::scene_mapping`]), logo um segundo
    /// tamanho em alcance seria a porta por onde o defeito voltava — *a régua que compila é a que
    /// está à mão*.
    pub(super) fn fase_physics_overlay(&mut self, viewport: EditorRect) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // ⭐ **A janela da CENA, não a do quadro** — sob um split do centro a cena desenha num
        // sub-rectângulo e a projecção MUDA (ver [`crate::scene_mapping`]). Fora do split é a
        // janela inteira, bit a bit.
        let janela_da_cena = gfx.scene_window();
        let FrameGfx {
            sim,
            camera,
            theme,
            vector_scene,
            text_system,
            hero_screen,
            physics,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };
        // O contorno dos colliders: um sprite é um QUAD e um collider é
        // invisível, então sem isto "que forma isto tem, fisicamente?"
        // não tem resposta na tela (Enio, 2026-07-18). No-op sem corpos.
        // Joints too — a joint is a RELATIONSHIP with no geometry at all,
        // so two objects pinned together look exactly like two that merely
        // touch. The anchors come from the solver, not from the joint
        // entity's Transform (see `ph2d_app_physics::overlay::outline::joint_marks`).
        //
        // W-J1: a VIEW, not just the anchor pair — the drawing reads the
        // very `JointDesc` the solver was handed (kind, limits, motor,
        // length) plus the live poses, so the glyph cannot describe a joint
        // the solver is not enforcing.
        let joint_views: Vec<ph2d_physics_ecs::JointView> = physics.joint_views().collect();
        // A arena que as faixas das views indexam (W-Pulley W1) — a MESMA
        // fatia que o solver está usando neste frame.
        let joint_wheels = physics.pulley_wheel_arena().to_vec();
        // E o ângulo de cada uma — o giro que faz uma roda parecer uma roda.
        let joint_spins = physics.pulley_wheel_spins().to_vec();
        // A corda frouxa pendura para onde as coisas caem — a MESMA fonte
        // que decide a superfície de uma poça (W-Buoyancy).
        let joint_gravity = {
            let s = physics.settings();
            [s.gravity_x, s.gravity_y]
        };
        // Sensors that have a body inside them THIS frame — the overlay
        // lights them up. A sensor with nothing reading its overlaps would
        // be a dead flag (W7), so the visible reaction lives here.
        let triggered = physics.triggered_sensors();
        // The initial-velocity arrow is only truthful before the sim steps:
        // once a body has moved, its live velocity is no longer the authored
        // launch. `last_stepped() == 0` is exactly "the bodies are at their
        // authored rest", the same fact the bridge uses to decide a respawn.
        let velocity_at_rest = physics.last_stepped() == 0;
        // Where bodies actually TOUCH, and how hard they press (W-Contacts). A
        // contact exists only while two shapes meet, and nothing else on screen
        // says whether two objects are resting on each other or just overlapping
        // in the artist's eye.
        let contacts = physics.contacts().to_vec();
        // Onde o cursor está, em mundo — a âncora da mira das ferramentas de
        // ponto (W-Hand). Derivada aqui e não guardada: o `last_pointer` é a
        // única fonte, e uma cópia dela desenharia a mira onde o mouse ESTAVA.
        let pointer_world = camera.screen_to_world(self.last_pointer, janela_da_cena);
        // The begin-flashes (`×`) — the visible half of the contact-events channel,
        // a separate list from the standing `+` crosses because a flash marks a
        // BEGINNING and outlives the tick it was born in (W-TickContacts).
        let flashes = physics.contact_flashes().to_vec();
        // Onde a água está. O empuxo calcula essa superfície todo frame e, até
        // isto, nada na tela a mostrava — o artista posicionava o que boia no olho.
        let waterlines = physics.waterlines();
        // W-Probes: o que os sensores do player olharam no ULTIMO tique, do
        // UNICO dono do fato (a ponte). Ate isto, nada na tela dizia onde a
        // perna, o flanco, a quina ou o teto do agachar procuram.
        // ⭐ E os RAIOS AUTORADOS (suplente #21, W5) na MESMA lista, porque o pintor é UM só: um
        // `ProbeShape::Ray` já desenha a linha, a ponta do alcance e o tique do acerto, e um segundo
        // pintor seria a segunda resposta a *como se desenha um raio*. As duas listas ficam
        // separadas na PONTE (dois escritores) e juntam-se AQUI, que é o que a shell é.
        let probes = [physics.player_probe_marks(), physics.ray_marks()].concat();
        ph2d_app_physics::overlay::outline::draw(
            self.show_colliders,
            velocity_at_rest,
            sim,
            &joint_views,
            &joint_wheels,
            &joint_spins,
            joint_gravity,
            // W-J3: o limite que o arrasto está posando AGORA, para o
            // fantasma de B. Lido do componente (o arrasto já escreveu nele
            // neste frame), então a silhueta e o arco mostram o mesmo número.
            self.physics
                .joint_anchor_drag
                .and_then(|d| d.posed_limit(sim)),
            // W-J4: a banda elástica, se um gesto de criar está em voo (e o
            // corpo A ainda existe — apagá-lo sob o gesto o invalida).
            ph2d_app_physics::joint_draw::body_alive(sim, self.physics.joint_draw)
                .then(|| ph2d_app_physics::joint_draw::band(self.physics.joint_draw))
                .flatten(),
            // W-Grab: a mola da mão, lida do ÚNICO dono do fato (a ponte);
            // o ponto de pega é derivado da pose VIVA do corpo, então o
            // zigzag acompanha o que a mola está de fato puxando.
            physics.grab_marks(),
            // W-Hand: a MIRA da ferramenta de ponto em mãos. `aim_radius` é
            // `None` para a mão; e o gesto só é oferecido com o relógio
            // ANDANDO e a física ARMADA, então a mira honra as MESMAS duas
            // condições que `body_grab::poke_at` — uma mira que promete o que
            // o clique não faz é pior que mira nenhuma.
            (self.playhead.is_playing() && self.timeline.flags.simulate_physics)
                .then(|| self.physics.interaction.aim_radius())
                .flatten()
                .map(|r| (pointer_world, r)),
            // O campo VIVO, do ÚNICO dono do fato (a ponte).
            physics.attract_marks(),
            // E o último estouro, enquanto o flash dura.
            self.blast_flash.map(|(c, r, _)| (c, r)),
            &contacts,
            &flashes,
            &waterlines,
            &probes,
            &triggered,
            // W20: a descida em curso, do ÚNICO dono do fato (a ponte). Sem
            // isto uma prancha fantasma é indistinguível de uma sólida, que
            // é a forma como toda esta classe de defeito ficou silenciosa.
            physics.any_player_is_dropping(),
            // W-J7b: o joint selecionado ganha readout mesmo sem teto armado
            // — é preciso ler a carga ANTES de escolher um número.
            hero.gizmo
                .iter_selected()
                .map(ph2d_ecs::Entity::from_bits)
                .find(|e| {
                    sim.world()
                        .get::<ph2d_physics_ecs::PhysicsJoint>(*e)
                        .is_some()
                }),
            camera,
            // ⛔⛔⛔ **A JANELA DA CENA, e nunca a da janela — a QUINTA vez que esta lei é paga**
            // ([`crate::scene_mapping`], que lista as outras quatro).
            //
            // ⚠️ **MEDIDO pela foto da cena `PH2D_RAY_SMOKE=1` (19/09), e a aritmética fecha antes
            // do código:** com a ferramenta MOTION activa — que o `~/.ph2d/layout.txt` do dono
            // **reactiva no quadro 1** — a cena desenha num sub-rectângulo `[0, 0, w, h·t]` e a
            // projecção MUDA (não é um recorte). O [`ph2d_render::Camera2d::world_to_screen`]
            // deriva os **dois** eixos de `h`, logo com a janela inteira tudo sai `h/(h·t)` vezes
            // maior e o zero vertical fica em `h/2` em vez de `h·t/2`.
            //
            // A conta da foto: janela `1930×1012`, banda `≈557`, `height_world ≈ 11,14 m` ⇒ o raio
            // do olho, que nasce em `y = +0,35`, tinha de ser desenhado a `≈261 px` e apareceu a
            // **`≈474`** — `~210 px` abaixo e `1,8×` mais comprido. *Os corpos estavam no sítio
            // certo e a física que se desenha por cima deles não.*
            //
            // ⛔ **E isto NÃO é da timeline estar aberta:** o único escritor de um `CenterSplit` é
            // o ramo da ferramenta Motion — a correcção que a wave do HUD pagou um dia antes, por
            // ler *onde o defeito aterrava* como *o que o causava*.
            //
            // ⭐ **Fora do split é BYTE-IDÊNTICO** (`CenterSplit::None` devolve a janela inteira),
            // logo esta linha não muda um pixel de nenhuma das cenas que já shipavam.
            janela_da_cena,
            vector_scene,
            // ⚠️ Reborrow por `paint_ctx`, não o binding cru: o `text_system`
            // já está emprestado por ele desde o começo do frame, e um
            // segundo empréstimo direto não compila. O reborrow morre com a
            // chamada, que é exatamente o tempo de vida que o rótulo precisa.
            paint_ctx.text,
        );
    }
}
