//! **O clique, o SOLTAR da ferramenta vetorial** — ramos do `on_mouse_input` ([`super`]): o osso que nasce do
//! arrasto, os gestos que se fecham (Build, conector, gradiente, gaiola, região, Width, lápis) e o release da
//! caneta e da forma, com a solda. Os corpos MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no
//! sítio da chamada, pela mesma ordem.
//!
//! ⚠️ Cada ramo devolve `true` onde o braço fazia `return;` — e o braço, ao receber `true`, devolve. Um Up que
//! nenhum consome cai no chrome, como caía.

use super::*;

impl crate::App {
    /// O release da caneta (e o diagnóstico das quinas) ou da forma em arrasto — com a solda dos endpoints e a
    /// forma nova seleccionada.
    pub(super) fn ramo_vetor_solto_caneta(&mut self) -> bool {
        if shape_kind_for_mode(&self.vec.draw_config).is_none() {
            // Pen: the release ends a handle drag / grab.
            let consumed = self.vec.pen.on_release();
            // DIAGNÓSTICO (`PH2D_CORNER_LOG=1`): os raios LOGO APÓS o gesto. Com o
            // log do press, parte o report em dois — se aqui os raios anteriores já
            // sumiram, foi o GESTO; se estão inteiros e somem até o press seguinte,
            // foi um passe POR-FRAME entre os dois.
            if std::env::var_os("PH2D_CORNER_LOG").is_some()
                && self.vec.draw_config.mode.is_corner_tool()
                && let Some(gfx) = self.gfx.as_ref()
                && let Some(pid) = self.vec.pen.selected()
            {
                let shape = self.vec.entities.get(&pid).is_some_and(|&b| {
                    gfx.sim
                        .world()
                        .get::<ph2d_ecs::VecShape>(ph2d_ecs::Entity::from_bits(b))
                        .is_some()
                });
                let radii: Vec<f64> = gfx
                    .vec_scene
                    .path(pid)
                    .map(|p| p.verts_all().map(|v| v.corner_radius).collect())
                    .unwrap_or_default();
                eprintln!("[corner] RELEASE shape={shape} radii={radii:?}");
            }
            if consumed {
                return true;
            }
        } else if shape_up_consumes(self.vec.draw_config.mode, self.vec.shape.is_active()) {
            // A shape drag is in progress → finalize it. Commit if the
            // drag spanned a real size, else discard the stray click
            // (cancel the pending undo so it doesn't record a spurious
            // `next_id`-only step). ONLY consume the Up when a shape is
            // actually being drawn — otherwise (e.g. releasing over a
            // panel button while in a shape mode) the Up MUST fall
            // through to the chrome dispatch, else every panel click
            // (mode switch, boolean, close) is silently swallowed.
            let committed = if let Some(gfx) = self.gfx.as_mut() {
                let c = self.vec.shape.on_release(&mut gfx.vec_scene);
                if c {
                    // Solda os endpoints da forma recém-criada com nós
                    // vizinhos: basta ficarem próximos para se fundirem, e
                    // várias linhas/arcos fecham numa forma (Enio
                    // 2026-07-09). A forma nova ainda não tem entidade
                    // (o sync roda depois), então está na identidade; a
                    // geometria PRÉ-existente nunca se mexe (só a nova
                    // snapa nela). Ao fechar num laço, recebe o fill do
                    // estilo atual — como uma região desenhada pela pen.
                    if let Some(new_id) = self.vec.shape.selected() {
                        let fill = self.vec.pen.style().fill;
                        let fill_on_close =
                            (fill.a != 0).then(|| ph2d_vec_scene::Paint::solid(fill));
                        let xforms =
                            ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
                        let win = gfx.surface.size();
                        let tol = crate::vec_gizmo_view::stroke_hit_r(&gfx.camera, win) * 1.5;
                        gfx.vec_scene
                            .weld_new_shape(new_id, &xforms, tol, fill_on_close);
                    }
                }
                c
            } else {
                false
            };
            if committed {
                // Seleciona a forma nova para edição imediata — a menos que
                // o weld a tenha fundido noutro objeto (o id sumiu).
                let sel = self.vec.shape.selected().filter(|id| {
                    self.gfx
                        .as_ref()
                        .is_some_and(|g| g.vec_scene.paths().iter().any(|p| p.id == *id))
                });
                self.vec.pen.select(sel);
            }
            return true;
        }
        // Shape mode but no active drag → fall through to chrome so the
        // panel buttons receive their Up.
        false
    }

    /// Os gestos que se fecham no Up, cada um só com o gesto VIVO: o Build, o conector, o gradiente, o canto da
    /// gaiola, a região (caixa ou laço), a alça do Width e o traço do lápis.
    pub(super) fn ramo_vetor_solto_gestos(&mut self) -> bool {
        // Shape Builder: o Up materializa as faces pintadas. Consome SÓ com o
        // arrasto VIVO, pela mesma razão que o conector documenta abaixo —
        // um Up sobre um botão do painel não pode ser engolido pelo modo.
        if self.vec.build.as_ref().is_some_and(|s| s.dragging) {
            self.build_up();
            return true;
        }
        // Conector: o Up prende a 2ª ponta (na forma sob o cursor, ou solta ali).
        // Consome SÓ com gesto vivo — senão soltar sobre um botão do painel no
        // modo Connect engoliria o clique (a armadilha do `shape_up_consumes`).
        if self.vec.connect.is_some() {
            let w = self.vec_world_at(self.last_pointer);
            if let Some(w) = w {
                self.connector_up(w);
            } else {
                self.connector_cancel();
            }
            return true;
        }
        // Gradient group 3b: end a gradient-handle drag.
        if self.vec.grad_drag.take().is_some() {
            return true;
        }
        // ADR-0129 Fatia 1: fim de um arrasto de canto da gaiola. O
        // `VecEnvelope` alterado vira UM passo no diff global do undo ao
        // soltar — o `held_button` suprimiu os frames intermediários
        // (`post_frame_undo`), então não há `commit_if_changed` a chamar aqui
        // (esse é o histórico do PEN; o envelope viaja no `WorldSnapshot`).
        // Consome só quando havia um canto vivo.
        if self.vec.envelope_drag.take().is_some() {
            return true;
        }
        // (A alça do texto em caminho é do modo Select — o Up dela mora lá em cima,
        // ao lado do Up do conector; não aqui, que é o caminho de Node.)
        // Fim do gesto de REGIÃO → selecciona as âncoras dentro dela.
        if let Some(m) = self.vec.marquee.take() {
            // **Shift SOMA** (o retângulo de todo app); sem ele, substitui. E uma
            // região de tamanho zero é um CLIQUE no vazio: ela desseleciona, em vez
            // de fazer um select que não apanha nada e deixa a seleção intacta.
            let additive = self.modifiers.shift_key();
            let (start, cur) = (m.start, m.cur);
            let moved = (start.0 - cur.0).abs() > 1.0 || (start.1 - cur.1).abs() > 1.0;
            if let Some(gfx) = self.gfx.as_mut() {
                if moved {
                    let win = gfx.surface.size();
                    let to_world = |p: (f32, f32)| {
                        let w = gfx.camera.screen_to_world(p, win);
                        [w[0] as f64, w[1] as f64]
                    };
                    match m.shape {
                        MarqueeShape::Box => self.vec.pen.box_select_with(
                            &gfx.vec_scene,
                            to_world(start),
                            to_world(cur),
                            additive,
                        ),
                        // ⚠️ O polígono é convertido a MUNDO ponto a ponto, e é aqui
                        // que o LAÇO tem de o ser: as âncoras que ele julga sobem
                        // pelo afim de cada forma (ADR-0111), então a pergunta só faz
                        // sentido no espaço que as duas partilham.
                        MarqueeShape::Lasso => {
                            let poly: Vec<[f64; 2]> =
                                m.closed_path().into_iter().map(to_world).collect();
                            self.vec
                                .pen
                                .lasso_select_with(&gfx.vec_scene, &poly, additive);
                        }
                    }
                } else if !additive {
                    self.vec.pen.select(None);
                }
            }
            return true;
        }
        // **O WIDTH TOOL solta.** A alça é largada e o passo de undo fecha — o
        // MESMO par begin/commit do lápis e das ferramentas de quina.
        //
        // ⚠️ **Arm próprio, e ANTES da cadeia de modo**, pela razão que o lápis
        // pagou logo abaixo: `shape_kind_for_mode(..).is_none()` é verdadeiro no modo
        // Width, então um ramo posto no `else` dele seria código morto no único modo
        // capaz de o alcançar — e a alça ficaria agarrada ao dedo depois de solta.
        if let Some(grab) = self.vec.width_grab.take() {
            if let Some(gfx) = self.gfx.as_mut() {
                // Um clique que não moveu nada não pediu nada: a parada que o press
                // criou é desfeita, e o desenho fica como estava (ver `Grab::created`
                // — os 13,1% da re-parametrização nunca chegam à tela).
                crate::width_handles::discard_if_untouched(&mut gfx.sim, &self.vec.entities, grab);
            }
            return true;
        }
        // **O LÁPIS solta.** O traço vira documento (ou desaparece, se o gesto
        // foi um clique perdido) e a forma nova fica SELECIONADA — o artista
        // acabou de a desenhar, então é nela que ele vai mexer.
        //
        // ⚠️ **Arm PRÓPRIO, e antes da cadeia de modo.** Ele nasceu no `else` de
        // `shape_kind_for_mode(..).is_none()`, que é **verdadeiro em modo Pencil** (o
        // lápis não é um `ShapeKind`) ⇒ a primeira metade ganhava sempre e este
        // ramo era **código morto no único modo capaz de o alcançar**. O preço eram
        // dois defeitos que o Enio viu como um: o `active` nunca era limpo, então o
        // lápis **continuava a desenhar com o botão em cima** (todo move seguinte
        // entrava no traço) e o press seguinte **apagava o traço anterior**
        // (`on_press` remove o path que encontra vivo).
        //
        // ⚠️ O guard é `is_active()`, não "o modo é Pencil": soltar sobre um botão
        // do painel enquanto o lápis está armado mas ocioso TEM de cair no chrome,
        // senão todo clique de painel morre em silêncio (a lição que o
        // `shape_up_consumes` documenta ao lado).
        if self.vec.pencil.is_active() {
            let committed = if let Some(gfx) = self.gfx.as_mut() {
                self.vec.pencil.on_release(&mut gfx.vec_scene)
            } else {
                false
            };
            if committed {
                let sel = self.vec.pencil.selected();
                self.vec.pen.select(sel);
            }
            return true;
        }
        false
    }
}
