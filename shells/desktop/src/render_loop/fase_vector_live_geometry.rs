//! **Fase do quadro: A GEOMETRIA VIVA** — a simetria de desenho, o lápis, o perfil, o mapa de geometria viva
//! fundido e a booleana viva com os operandos absorvidos (OBRA 2 da `line/render-loop`, 2026-09-13).
//!
//! ⚠️ **O corte pára ANTES do bloco do painel vectorial:** esse bloco já só chama fases, e a cola delas
//! (`let … else { return; };`) é saída do QUADRO — mora no orquestrador, nunca dentro de outra fase.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_live_geometry(
        &mut self,
        window_size: ph2d_host::WindowSize,
        drawing: Vec<u64>,
        mut vec_view: ph2d_vec_scene::VecViewState,
        vec_xf: ph2d_vec_scene::VecXforms,
    ) -> Option<(
        ph2d_vec_render::LiveGeometry,
        ph2d_vec_scene::VecViewState,
        ph2d_vec_scene::VecXforms,
    )> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            camera,
            vec_scene,
            tags,
            ..
        } = FrameGfx::of(gfx);
        // ── A SIMETRIA de DESENHO (plano 25 W6.3) ────────────────────────────
        // *"A linha deve aparecer logo que se aperta o botão e não quando se inicia o desenho.
        // A simetria funciona apenas para formas que serão desenhadas com a tool ligada … com
        // o botão checado pode-se fazer quantos desenhos desejar que a linha permanece no
        // lugar"* (Enio, 2026-08-01).
        //
        // ⚠️ **Aqui e não na malha de ações**, e por duas razões que se somam: o `sync` já
        // correu (uma forma recém-desenhada já tem entidade, senão o componente não teria
        // onde pousar) e o `settle_origins` já assentou o pivô do gesto que acabou — que é o
        // frame em que a captura do eixo sela. É o mesmo sítio, e pela mesma razão, em que o
        // LÁPIS pendura o perfil dele logo abaixo.
        //
        // ⚠️ A adopção olha para `drawing` — quem está EM GESTO —, **nunca** para a seleção.
        // É essa ausência que cumpre *"não deve fazer simetria de formas que já existem
        // previamente"*.
        {
            let style = self.vec.draw_config.symmetry;
            let live = if style.on {
                // A semeadura acontece UMA vez, na aresta desligado→ligado: *"a tela é a
                // referência para a posição inicial da linha"*. Re-semear por frame faria a
                // linha seguir a câmera, e panhar o canvas arrastaria o eixo junto.
                let origin = *self.vec.symmetry_origin.get_or_insert_with(|| {
                    let (w, h) = (window_size.width as f32, window_size.height as f32);
                    let c = camera.screen_to_world((w * 0.5, h * 0.5), window_size);
                    [f64::from(c[0]), f64::from(c[1])]
                });
                self.symmetry_live.adopt(
                    sim,
                    &self.vec.entities,
                    vec_scene,
                    &vec_xf,
                    style,
                    origin,
                    &drawing,
                )
            } else {
                // Desligado, o eixo de sessão morre: a próxima ligação re-semeia no centro do
                // ecrã, que é o que o artista pede ao ligar. Os COMPONENTES ficam — desarmar
                // esconde as cópias, não as destrói.
                self.vec.symmetry_origin = None;
                0
            };
            // O painel só oferece o **Apply** quando há o que consolidar — e "o que se vê" é
            // vazio com o modo desligado, então um Apply ali consolidaria coisa invisível.
            ph2d_panel_vector::state_symmetry::set_symmetry_live_count(live);
        }
        // A SIMETRIA VIVA (plano 25 W6.3): as cópias do modo simétrico, cozidas aqui e
        // desenhadas no z da fonte — que entra na lista junto com elas.
        self.symmetry_live.recook(
            vec_scene,
            sim,
            &self.vec.entities,
            &vec_xf,
            self.vec.draw_config.symmetry.on,
        );
        // **O LÁPIS pendura o perfil que o GESTO pede** (W1d) — ao vivo, a cada frame em que
        // o traço está aberto. É aqui e não no `input_dispatch` porque o armamento precisa do
        // mundo ECS, e porque é o único lugar que corre entre o `sync` (que dá entidade ao
        // path recém-nascido) e o cozimento logo abaixo: o artista vê a espessura enquanto
        // desenha, que é a promessa do lápis desde o W1a (*"o ajuste é AO VIVO"*).
        if let Some(id) = self.vec.pencil.active_path() {
            let stops = self
                .vec
                .pencil
                .width_stops(self.vec.draw_config.pencil_width_source);
            crate::profile_live::arm(sim, &self.vec.entities, &[id], &stops);
        }
        // A largura VIVA (ADR-0148): a fita de largura variável, cozida aqui e desenhada no
        // z da fonte — que continua sendo a curva autorada que o modo Node edita.
        self.profile_live
            .recook(vec_scene, sim, &self.vec.entities, &vec_xf);
        // O `dispatch` recebe UMA `LiveGeometry`. Uma forma é offset OU pattern OU contour
        // (nunca dois — cada um é um componente próprio e o painel oferece um de cada vez),
        // então fundir é seguro; começa do offset (em cena típica dos outros, vazio ⇒ clone
        // trivial) e junta os demais por cima.
        let mut vec_live = self.offset_live.live().clone();
        // ⭐⭐⭐ **O 10.º produtor: o texto que o JOGO muda** (TOP-20 #20). Ele entra AQUI, entre os
        // que estendem o mapa, porque um rótulo de HUD é uma forma como as outras — e sai vazio na
        // cena típica (só um `UiLabel` com fonte derivada e número diferente do autorado o enche).
        // ⛔ O documento não é tocado: ver o cabeçalho do [`crate::hud_label_live`].
        vec_live.extend(crate::hud_label_live::cook(
            sim,
            vec_scene,
            &self.vec.entities,
            &vec_xf,
            tags,
        ));
        vec_live.extend(
            self.pattern_live
                .live()
                .iter()
                .map(|(id, v)| (*id, v.clone())),
        );
        vec_live.extend(
            self.contour_live
                .live()
                .iter()
                .map(|(id, v)| (*id, v.clone())),
        );
        vec_live.extend(
            self.symmetry_live
                .live()
                .iter()
                .map(|(id, v)| (*id, v.clone())),
        );
        vec_live.extend(
            self.profile_live
                .live()
                .iter()
                .map(|(id, v)| (*id, v.clone())),
        );
        // **A BOOLEANA VIVA roda DEPOIS dos cinco e ANTES do alinhamento**, e a ordem é a lei
        // da wave — trocar dois destes termos dá arte diferente sem nenhum gate vermelho:
        //
        // - depois dos cinco, porque ela consome *o que os filhos de fato desenham* (um
        //   operando com offset vivo tem de entrar deslocado);
        // - antes do alinhamento, porque o alinhamento é um campo do `StrokeSpec` do
        //   RESULTADO — alinhar os operandos e só então os combinar responderia outra
        //   pergunta;
        // - e ela TRANSFORMA o mapa (não o estende) pela mesma razão do alinhamento: é um
        //   componente do PAI, então convive com o offset de cada filho.
        self.bool_live.recook(
            vec_scene,
            sim,
            &self.vec.entities,
            &vec_xf,
            &self.ui_cooked.bool_morphs,
            &mut vec_live,
        );
        // **QUEM FOI ABSORVIDO**, publicado no mesmo fôlego em que a absorção acontece. Sem
        // isto o operando consumido — que recebe uma lista VAZIA logo acima — fica
        // indistinguível de uma forma ANIQUILADA por um offset, e a lei *nada desenhado, nada
        // pego* torna-o inalcançável pelo canvas (Enio, 2026-08-22).
        vec_view.absorbed = self.bool_live.absorbed();
        Some((vec_live, vec_view, vec_xf))
    }
}
