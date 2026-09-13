//! **O clique, o SOLTAR da ferramenta vetorial** — ramos do `on_mouse_input` ([`super`]), corpos verbatim pela mesma
//! ordem: o osso do arrasto, os gestos que se fecham (Build, conector, gradiente, gaiola, região, Width, lápis) e o
//! release da caneta e da forma, com a solda. Um Up que nenhum ramo consome cai no chrome, como caía.

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

    /// O Up apaga as guias de snap, e o osso nasce do arrasto: a mesma leitura que a pré-visualização desenhou,
    /// o pai que o press apontou, a emenda da corrente solta e a memória do revelar-ao-focar.
    pub(super) fn ramo_vetor_solto_osso(&mut self) -> bool {
        // Fim de gesto: as guias de snap não sobrevivem ao Up.
        self.vec_clear_snap_guides();
        // ⭐⭐⭐ **O OSSO nasce aqui** (estudo 42 item 5): origem no press, comprimento e
        // ângulo no arrasto, PAI = o osso seleccionado — e o novo fica seleccionado, que
        // é o que faz arrasto-arrasto-arrasto ser uma cadeia.
        //
        // ⚠️ **Consome SÓ com o gesto VIVO** (a origem marcada), pela lei que o
        // `shape_up_consumes` documenta: soltar sobre um botão do painel neste modo não
        // pode engolir o clique.
        if let Some(nascimento) = self.skeleton.bone_drag.take() {
            let px = self.vec_px_to_world();
            let mut nasceu = None;
            if let Some(solto) = self.vec_world_at(self.last_pointer) {
                // ⭐⭐⭐ **A MESMA leitura que a pré-visualização desenhou**
                // ([`crate::bone_gesture::drag_now`]): a emenda, a ponta encaixada e o
                // limiar. O artista viu o osso saltar para aquela bolinha, e é
                // exactamente ali que ele nasce.
                //
                // ⚠️ **A EMENDA** (ordem do dono, 2026-09-09): se o arrasto acaba na
                // BASE de uma corrente solta, a ponta do osso novo encaixa nela e essa
                // corrente passa a pendurar-se nele — duas correntes viram uma.
                let Some(agora) = self
                    .gfx
                    .as_ref()
                    .map(|g| crate::bone_gesture::drag_now(&g.sim, nascimento, solto, px))
                else {
                    return true;
                };
                let (ponta, emenda) = (agora.tip, agora.splice);
                if agora.armed
                    && let Some(gfx) = self.gfx.as_mut()
                {
                    // ⭐⭐⭐ **O PAI é o que o PRESS apontou** (ordem do dono,
                    // 2026-09-09) — ⛔ nunca a selecção, que era a lei que ele mandou
                    // tirar. Ele ainda é filtrado porque um osso pode ter sido apagado
                    // entre o press e o release, e um pai morto não tem espaço local.
                    let pai = nascimento
                        .parent
                        .and_then(ph2d_ecs::Entity::try_from_bits)
                        .filter(|e| gfx.sim.world().get::<ph2d_skeleton_ecs::Bone>(*e).is_some());
                    nasceu =
                        crate::bone_gesture::create(&mut gfx.sim, pai, nascimento.origin, ponta);
                    // ⭐⭐⭐ **E a corrente solta passa a pendurar-se no osso novo.**
                    //
                    // ⚠️ **Depois do `create`, nunca antes:** o pai só existe agora, e
                    // adoptar antes dele nascer não tem onde pendurar. ⚠️ E o `connect`
                    // preserva a pose de MUNDO do adoptado — sem isso o esqueleto
                    // inteiro saltaria pela pose do osso novo.
                    if let (Some(novo), Some((alvo, _))) = (nasceu, emenda) {
                        crate::bone_gesture::connect(&mut gfx.sim, alvo, novo);
                    }
                    if let Some(bits) = nasceu
                        && let Some(hero) = gfx.hero_screen.as_mut()
                    {
                        hero.gizmo.selection = Some(bits);
                        hero.gizmo.extra_selection.clear();
                    }
                }
            }
            // ⭐⭐⭐ **UM OSSO ACABADO DE NASCER NÃO É UM OSSO ESCOLHIDO** (report do
            // dono, 2026-09-09: *«cada vez que se cria um osso o modo Transform é
            // selecionado»*).
            //
            // ⛔ O osso novo fica aceso — é assim que o artista vê qual é — e no quadro
            // seguinte a aresta do foco lia isso como *«o artista escolheu um osso»* e
            // armava *Transform*, arrancando-o do verbo em que ele estava. A memória
            // absorve-o AQUI, onde se sabe que ele nasceu de um arrasto e não de uma
            // escolha. *A aresta continua a valer; o que mudou é quem a alimenta.*
            if let Some(bits) = nasceu {
                crate::skeleton_reveal::on_birth(&mut self.skeleton.osso_revelado, bits);
            }
            return true;
        }
        false
    }
}
