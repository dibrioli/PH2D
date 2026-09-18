//! **Os métodos do despacho: modos e alças** — `impl App` ([`super`]): que ferramenta está na mão e que teclas
//! estão vivas, o undo do grafo, as alças do texto e das fichas, o motion path e o pick do caminho-guia.
//! Mudados VERBATIM (`line/input-dispatch`, 2026-09-13); os privados passam a `pub(super)`.

impl crate::App {
    pub(crate) fn vector_tool_active(&self) -> bool {
        self.gfx.as_ref().is_some_and(|g| {
            g.tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("vector"))
        })
    }

    /// **O barro está na tela?** — a pergunta que o PONTEIRO da cena 3D já fazia.
    ///
    /// Delega ao papel da forma (`FormRole::draws_clay`), que é o mesmo fato que
    /// decide o passe de cor e a posse do clique no canvas. Sem cena (ou sem a
    /// feature) ela é `false`, e o resto do app nem sabe que este módulo existe.
    #[cfg(feature = "sculpt3d")]
    pub(crate) fn sculpt3d_clay_on_screen(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.sculpt3d.as_ref())
            .is_some_and(ph2d_app_sculpt3d::Sculpt3dScene::clay_on_screen)
    }

    /// **As teclas da ESCULTURA estão vivas?** — o irmão de [`Self::motion_keys_live`]
    /// e [`Self::vector_keys_live`], e a cura do report do Enio (2026-08-17:
    /// *"depois de abrir outros módulos como Sculpt, o Motion não consegue usar os
    /// atalhos nem digitar um número"*).
    ///
    /// ⚠️ **A pergunta ERRADA era *«existe uma cena?»***. O teclado do 3D consome
    /// os **dez dígitos** e ~26 letras (o próprio módulo o escreve: *"com uma cena
    /// armada este teclado consome quase toda letra"*), e o portão dele era
    /// `sculpt3d_scene_mut().is_some()` — que fica verdadeiro **para sempre** depois
    /// do primeiro clique no pill, porque **sair do modo nunca destrói a cena**. A
    /// partir dali toda letra e todo dígito eram comidos ANTES do `handler.on_key`,
    /// então o painel do Motion não via nem atalho nem texto. Medido, o que morria
    /// era exatamente o conjunto NU do grafo (`F` Fit · `A` Add · `H` Bypass ·
    /// `K` Knife · `P` Probe) mais os dígitos — os `Ctrl+…` já passavam, porque o
    /// braço de `ctrl` só reclama o `Ctrl+Z`.
    ///
    /// ⚠️ **É uma assimetria entre duas portas que respondem à MESMA pergunta:** o
    /// ponteiro já cedia (ele pergunta `draws_clay`), o teclado não. Agora os dois
    /// perguntam o mesmo — *o barro está na tela?* —, que é o que "estou esculpindo"
    /// significa neste módulo, do mesmo jeito que *ter a ferramenta em mãos* é o que
    /// significa nos irmãos.
    /// ⚠️ **E o barro sozinho não bastava**, porque *sair do modo* não é o único jeito de
    /// ir trabalhar noutro lugar: pegar a ferramenta **Motion** no rail deixa o barro na
    /// tela e põe o artista num PAINEL, que foi o segundo caso do report (*"faça com que
    /// os atalhos motion funcionem no painel motion logo que ele for aberto"*). Uma
    /// ferramenta EM MÃOS ganha as teclas nuas — ver [`Self::a_tool_owns_the_bare_keys`].
    #[cfg(feature = "sculpt3d")]
    pub(crate) fn sculpt3d_keys_live(&self) -> bool {
        self.sculpt3d_keys_dead_reason().is_empty()
    }

    /// ⭐⭐⭐ **POR QUE as teclas da escultura estão mortas** — vazio quando estão vivas.
    ///
    /// ⛔⛔ **Ela existe por um report** (Enio, 2026-09-04: *«corrija o deletar com a tecla
    /// del»*): a tecla morria num de três guardas e **nenhum deles dizia nada**. O diagnóstico
    /// custou uma sessão de leitura de código para chegar a uma frase que esta função imprime.
    ///
    /// ⚠️ **Ela é a FONTE do [`Self::sculpt3d_keys_live`]**, e não uma segunda opinião: duas
    /// respostas à mesma pergunta divergem no dia em que alguém acrescenta um quarto guarda a
    /// só uma delas.
    #[cfg(feature = "sculpt3d")]
    pub(crate) fn sculpt3d_keys_dead_reason(&self) -> &'static str {
        if !self.sculpt3d_clay_on_screen() {
            return "nao ha' barro na tela (o pill SCULPT esta' fora, ou a forma nao e' barro)";
        }
        if self.text_entry_focused() {
            return "um campo de texto esta' FOCADO -- clique fora dele e tente outra vez";
        }
        if self.a_tool_owns_the_bare_keys() {
            return "a ferramenta Motion/Vector esta' EM MAOS e reivindica as teclas nuas";
        }
        ""
    }

    /// **Uma FERRAMENTA está em mãos reivindicando as teclas NUAS?**
    ///
    /// ⚠️ **Por que uma lista, e onde ela apodrece — dito na cara:** a cena 3D **não é uma
    /// `Tool`** (ADR-0150: a navegação orbital mora no shell justamente para manter a
    /// superfície congelada `Tool=12` fora do caminho), então ela **não participa** da
    /// arbitragem normal de ferramenta ativa — não há `ToolId` dela para o rail comparar. E
    /// o `Tool` é contrato CONGELADO (§6), logo não dá para perguntar à ferramenta *"você
    /// quer o teclado?"* sem um ADR.
    ///
    /// Enquanto isso, a precedência é expressa contra as ferramentas que de facto reclamam
    /// tecla NUA — hoje o Motion (`F`/`A`/`H`/`K`/`P`) e o Vector. ⚠️ **Uma terceira nasce
    /// fora desta lista**, e o sintoma será o mesmo report: atalhos mudos com o barro na
    /// tela. A cura definitiva é o 3D virar camada do documento (`docs/3D/05.2`), quando a
    /// pergunta deixa de precisar de lista.
    #[cfg(feature = "sculpt3d")]
    pub(super) fn a_tool_owns_the_bare_keys(&self) -> bool {
        self.motion_tool_active() || self.vector_tool_active()
    }

    /// Motion Nodes M1: is the Motion Nodes tool the active tool? Gates the graph
    /// undo/redo chord (mirror of `vector_tool_active`).
    pub(crate) fn motion_tool_active(&self) -> bool {
        self.gfx.as_ref().is_some_and(|g| {
            g.tools
                .active()
                .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("motion"))
        })
    }

    /// **As teclas do Vector estão vivas?** (BUGS #25)
    ///
    /// ⚠️ **Não é a mesma pergunta que [`Self::vector_tool_active`]**, e é essa
    /// distinção que o bug era: *ter a ferramenta em mãos* governa o PONTEIRO
    /// (clicar o canvas com um campo focado é justamente como se sai dele), mas
    /// uma TECLA pertence a quem tem o foco do teclado. Digitar `Update` no rename
    /// da Hierarquia disparava Union · Difference · modo Texto, e `Backspace`
    /// apagava um vértice em vez de uma letra.
    ///
    /// ⚠️ **A guarda sempre existiu e sempre esteve certa** — o que apodreceu foi
    /// o modo de aplicá-la: ela era composta **à mão** em três dos oito blocos de
    /// tecla, e os outros cinco nasceram sem. *Uma condição que enumera os seus
    /// leitores apodrece*; por isso ela é uma PORTA, e o arch-gate
    /// `the_vector_key_blocks_ask_whether_the_keys_are_live` recusa
    /// `vector_tool_active()` cru na família `keyboard*.rs`.
    pub(crate) fn vector_keys_live(&self) -> bool {
        self.vector_tool_active() && !self.text_entry_focused()
    }

    /// O espelho do Motion — mesma lei, mesmo motivo (o acorde Ctrl+Z do grafo
    /// roubava o undo de um campo de texto focado). Duas portas e não uma porque
    /// *qual ferramenta está em mãos* é a metade que difere; a metade do foco é a
    /// MESMA função, então não há duas respostas para *"há texto sob o cursor de
    /// teclado?"*.
    pub(crate) fn motion_keys_live(&self) -> bool {
        self.motion_tool_active() && !self.text_entry_focused()
    }

    /// Motion Nodes M1 Phase 1b-3: undo the last graph edit (Ctrl/Cmd+Z). The
    /// `MotionHistory` stack is populated by the graph-edit intents (add / delete
    /// / connect / disconnect = one step each; a node drag is one bracketed step).
    /// Restoring the doc changes the cook, so re-cook via `mark_dirty`.
    pub(super) fn motion_undo(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let m = &mut gfx.motion;
        if let Some(prev) = m.history.undo(&m.doc) {
            m.doc = prev;
            m.pump.mark_dirty();
        }
    }

    /// Motion Nodes M1 Phase 1b-3: redo (Ctrl/Cmd+Shift+Z / Ctrl+Y). Mirror of
    /// [`Self::motion_undo`].
    pub(super) fn motion_redo(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let m = &mut gfx.motion;
        if let Some(next) = m.history.redo(&m.doc) {
            m.doc = next;
            m.pump.mark_dirty();
        }
    }

    /// ADR-0108: enquanto o Pen arrasta um handle, projeta o cursor pra world e
    /// puxa os handles Bézier do último vértice. No-op barato quando não há
    /// arrasto — chamado a cada CursorMoved.
    ///
    /// O snap é entregue como closure porque o Pen sabe o que é ÂNCORA (encaixa) e
    /// o que é handle (não encaixa); a shell só sabe a posição do cursor.
    /// Arrasta o canto da gaiola do Envelope agarrado no press (ADR-0129 Fatia 1) para a
    /// posição do cursor, respeitando a convexidade (o canto para na fronteira, não sai
    /// dela). No-op (false) sem um arrasto vivo — a mesma disciplina do `vec_pen_drag_move`.
    pub(super) fn vec_textpath_handle_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vec.textpath_handle_drag {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.scene_window();
        let w = gfx.camera.screen_to_world((x, y), win);
        crate::vec_text_ride::handle::drag(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            &self.vec.entities,
            self.vec.pen.selected_paths(),
            [f64::from(w[0]), f64::from(w[1])],
            self.vec.textpath_handle_drag,
        )
    }

    /// **Pressão no modo Select sobre a alça do texto em caminho** (W5): se a alça está sob o
    /// cursor, arma o arrasto e devolve `true` — o host então PULA o picking/gizmo. Irmã do
    /// `conn_handle_down`: no Select a tool não captura o canvas, e o gizmo é inócuo sobre um
    /// texto vinculado (identidade), então a alça precisa deste arm para o dedo a pegar.
    pub(super) fn vec_textpath_handle_down(&mut self, world: [f64; 2]) -> bool {
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let radius = self.vec_px_to_world() * crate::vec_text_ride::HANDLE_R_PX;
        crate::vec_text_ride::handle::press(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec.entities,
            self.vec.pen.selected_paths(),
            world,
            radius,
            &mut self.vec.textpath_handle_drag,
        )
    }

    /// **Pressão sobre uma ÂNCORA do motion path** (ADR-0141, Fatia 3): se há uma sob o
    /// cursor, arma o arrasto, abre UM passo de undo e devolve `true` — o host então PULA
    /// o picking/gizmo.
    ///
    /// ⚠️ **Não é gateada por ferramenta**, ao contrário das alças do vetor: a trajetória
    /// é do documento de ANIMAÇÃO, e o artista a molda com qualquer ferramenta na mão. O
    /// que a gateia é ela estar VISÍVEL — a mesma pergunta que o desenho faz, e ela só é
    /// verdadeira para o objeto selecionado com um binding Position.
    ///
    /// ⚠️ **Consequência honesta:** a âncora do primeiro key costuma cair EM CIMA do
    /// sprite, onde o gizmo mora, então apertar exatamente ali agarra a âncora e não o
    /// objeto. O alvo tem 7 px de raio e nada mais muda — é o mesmo trade que toda alça
    /// deste app faz, e o desenho (um quadrado, a forma universal de "isto se arrasta")
    /// é o que o anuncia.
    ///
    /// ⚠️ **Agarra a âncora OU uma alça de tangente** (`motion_path_hit`) — as duas são a
    /// mesma pergunta ("o que está sob o cursor?"), e o gesto de move despacha sobre o
    /// que veio.
    pub(super) fn motion_path_anchor_down(&mut self, x: f32, y: f32) -> bool {
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let selected = gfx
            .hero_screen
            .as_ref()
            .and_then(|h| h.gizmo.iter_selected().next());
        let Some(hit) = ph2d_app_motion::motion_path_overlay::motion_path_hit(
            self.timeline.keys_mode,
            &self.timeline.doc,
            selected,
            &gfx.camera,
            gfx.scene_window(),
            x,
            y,
        ) else {
            return false;
        };
        // UM passo de undo por GESTO, não por frame de arrasto: o `commit_if_changed` do
        // release fecha o que este `begin` abriu.
        self.timeline.history.begin(&self.timeline.doc);
        self.motion_shell.path_drag = Some(hit);
        true
    }

    /// Secondary Down sobre uma **âncora** da trajetória: abre o menu de tipo de alça
    /// (Corner / Smooth / Symmetric) no cursor (ADR-0141) — o espelho do menu de ponto de
    /// curva do Painter. Devolve `true` (consumindo) só quando uma âncora foi atingida; uma
    /// ponta de tangente ou tela vazia cai fora (pan / outros handlers).
    ///
    /// ⚠️ A identidade da âncora VIAJA no `ContextMenuKind` (`{target, i}`), porque uma
    /// âncora de caminho não tem seleção persistente que o shell possa recuperar depois — o
    /// chrome a lê de volta do `last_context_menu` e o drain (`render_loop`) a converte.
    pub(super) fn motion_path_open_anchor_menu(&mut self, x: f32, y: f32) -> bool {
        use ph2d_app_motion::motion_path_overlay::{MotionPathGrab, motion_path_hit};
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let selected = gfx
            .hero_screen
            .as_ref()
            .and_then(|h| h.gizmo.iter_selected().next());
        // Só a ÂNCORA (o quadrado) abre o menu — a ponta de tangente é para arrastar, e o
        // tipo de alça é propriedade do NÓ, não da alça.
        let Some(MotionPathGrab::Anchor { target, i }) = motion_path_hit(
            self.timeline.keys_mode,
            &self.timeline.doc,
            selected,
            &gfx.camera,
            gfx.scene_window(),
            x,
            y,
        ) else {
            return false;
        };
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        hero.store
            .open_context_menu(ph2d_editor_core::interaction::ContextMenuRequest {
                x,
                y,
                kind: ph2d_editor_core::interaction::ContextMenuKind::MotionPathAnchor {
                    target: target.get(),
                    i: i as u32,
                },
            });
        true
    }

    /// Janela do duplo-clique no canvas (a mesma do texto) e a folga de posição.
    const MOTION_PATH_DCLICK_MS: u128 = 350;
    const MOTION_PATH_DCLICK_SLOP_PX: f32 = 5.0;

    /// **Duplo-clique no CAMINHO insere um ponto ali** (ADR-0141) — o "adicionar ponto" de
    /// um editor de vetor. Um clique SIMPLES cairia toda hora perto da trajetória (que fica
    /// sempre visível para o objeto selecionado); o duplo é deliberado. A forma da curva e o
    /// compasso do objeto são PRESERVADOS (`insert_path_anchor_at` divide por de Casteljau e
    /// keya no tempo exato em que o objeto passa ali). `true` = inseriu (o chamador consome).
    ///
    /// ⚠️ O canvas não emite `DoubleClick` (é evento por-widget do chrome), então o par
    /// (instante, posição) é rastreado aqui — o mesmo recurso do `vec_text_double_click`.
    pub(super) fn motion_path_curve_double_click(&mut self, x: f32, y: f32) -> bool {
        let now = std::time::Instant::now();
        let is_double = self
            .motion_shell
            .path_last_click
            .is_some_and(|(t, (px, py))| {
                now.duration_since(t).as_millis() <= Self::MOTION_PATH_DCLICK_MS
                    && (x - px).abs() <= Self::MOTION_PATH_DCLICK_SLOP_PX
                    && (y - py).abs() <= Self::MOTION_PATH_DCLICK_SLOP_PX
            });
        self.motion_shell.path_last_click = Some((now, (x, y)));
        if !is_double {
            return false;
        }
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let selected = gfx
            .hero_screen
            .as_ref()
            .and_then(|h| h.gizmo.iter_selected().next());
        let Some((target, d)) = ph2d_app_motion::motion_path_overlay::motion_path_curve_hit(
            self.timeline.keys_mode,
            &self.timeline.doc,
            selected,
            &gfx.camera,
            gfx.scene_window(),
            x,
            y,
        ) else {
            return false;
        };
        // Um passo de undo próprio, como o arrasto de âncora.
        self.timeline.history.begin(&self.timeline.doc);
        let ok = self.timeline.doc.insert_path_anchor_at(target, d);
        self.timeline.history.commit_if_changed(&self.timeline.doc);
        ok
    }

    /// Leva o que foi agarrado (âncora ou alça) para o cursor. No-op (`false`) sem um
    /// arrasto vivo — a mesma disciplina de early-return das alças do vetor.
    ///
    /// Escreve pela porta ÚNICA de cada gesto: `move_path_anchor` translada a curva (e
    /// re-suaviza as âncoras `auto`, senão a curva quebra), `move_path_tangent` a molda.
    /// As duas reescrevem as distâncias que as keys guardam na MESMA operação.
    pub(super) fn motion_path_anchor_move(&mut self, x: f32, y: f32) -> bool {
        use ph2d_app_motion::motion_path_overlay::MotionPathGrab;
        let Some(grab) = self.motion_shell.path_drag else {
            return false;
        };
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        let w = gfx.camera.screen_to_world((x, y), gfx.scene_window());
        match grab {
            MotionPathGrab::Anchor { target, i } => {
                let Some(mut a) = self.timeline.doc.path_anchor(target, i) else {
                    return false;
                };
                a.anchor = [w[0], w[1]];
                self.timeline.doc.move_path_anchor(target, i, a)
            }
            MotionPathGrab::Tangent { target, i, out } => {
                self.timeline
                    .doc
                    .move_path_tangent(target, i, out, [w[0], w[1]])
            }
        }
    }

    /// Arrasta a alça do PATTERN (Start/End, W4) armada para o cursor — no-op sem uma armada.
    /// Irmã do `vec_textpath_handle_move`, mesma disciplina de early-return.
    pub(super) fn vec_patternpath_handle_move(&mut self, x: f32, y: f32) -> bool {
        if self.vec.patternpath_handle.is_none() {
            return false;
        }
        let armed = self.vec.patternpath_handle;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.scene_window();
        let w = gfx.camera.screen_to_world((x, y), win);
        crate::pattern_live::handle::drag(
            &mut gfx.sim,
            &gfx.vec_scene,
            &self.vec.entities,
            self.vec.pen.selected_paths(),
            [f64::from(w[0]), f64::from(w[1])],
            armed,
        )
    }

    /// **Pressão no modo Select sobre uma alça do PATTERN** (W4): se uma está sob o cursor, arma o
    /// arrasto DELA e devolve `true` (o host então PULA o picking/gizmo). Irmã do
    /// `vec_textpath_handle_down`; o raio é o MESMO da ficha do texto (as duas são a mesma ficha).
    pub(super) fn vec_patternpath_handle_down(&mut self, world: [f64; 2]) -> bool {
        let radius = self.vec_px_to_world() * crate::vec_text_ride::HANDLE_R_PX;
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        crate::pattern_live::handle::press(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec.entities,
            self.vec.pen.selected_paths(),
            world,
            radius,
            &mut self.vec.patternpath_handle,
        )
    }

    /// **O clique do Picker de guia** (Enio 2026-07-23): com um pick armado, resolve o caminho sob o
    /// cursor e PRENDE — o motivo/texto capturado à fonte, o clicado ao guia. Clique no vazio desiste;
    /// clicar a própria fonte é ignorado (fica armado). Consome sempre o press (o guard já filtrou por
    /// `vec_path_pick.is_some()`), então o clique nunca cai no picking/gizmo enquanto o pick corre.
    pub(super) fn vec_path_pick_click(&mut self, world: [f64; 2]) {
        let Some(pick) = self.vec.path_pick else {
            return;
        };
        // LITERAL-PX-OK: o MESMO raio/resolvedor do realce do hover, para o que se clica ser o que se vê.
        let hit_r = 10.0 * self.vec_px_to_world();
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(guide) = self.vec.pen.path_at(&gfx.vec_scene, world, hit_r) else {
            self.vec.path_pick = None; // clique no vazio = desiste
            return;
        };
        if guide == pick.source() {
            return; // clicou a própria fonte — não é um guia; continua armado
        }
        let done = match pick {
            crate::vec_pick::PathPick::PatternMotif(motif) => {
                crate::pattern_live::link(&mut gfx.sim, &self.vec.entities, motif, guide)
            }
            crate::vec_pick::PathPick::TextObject(text) => crate::vec_text_ride::link_explicit(
                &mut gfx.sim,
                &mut gfx.vec_scene,
                &self.vec.entities,
                text,
                guide,
            ),
            // ⭐⭐⭐ **O SEGUNDO clique do conta-gotas do *Swap Prefab*** — a cópia `inst` passa a
            // ser uma cópia do prefab que o clique apontou.
            //
            // ⚠️ **O clicado NÃO tem de ser o prefab** — no modelo geral a receita está escondida
            // do canvas, então o alvo é *uma cópia dele* (ou a receita, quando aberta). Quem
            // resolve é a mesma porta que os outros verbos usam.
            //
            // ⚠️ **Falhar deixa o pick ARMADO de propósito:** desarmar aqui faria um clique fora
            // do alvo parecer que a troca aconteceu.
            //
            // ⛔ Aqui viveu um `if armed() { … } else { … }` (F4.6c): a resolução pelo motor
            // `VecInstance` morreu com ele. *Armar por um motor e resolver pelo outro trocava o
            // prefab pela porta errada, e o sintoma era uma cópia que muda de desenho e mantém o
            // elo antigo* — hoje só há uma porta, e a classe inteira do defeito com ela.
            crate::vec_pick::PathPick::InstanceMain(inst) => self
                .vec
                .entities
                .get(&inst)
                .copied()
                .zip(self.vec.entities.get(&guide).copied())
                .is_some_and(|(src, dst)| {
                    crate::vec_component_general::swap_by_pick(
                        &mut gfx.sim,
                        &mut self.instance_echo,
                        &mut gfx.toasts,
                        ph2d_ecs::Entity::from_bits(src),
                        ph2d_ecs::Entity::from_bits(dst),
                    )
                }),
            // ⭐ **A ARTE de um padrão** (plano 33 W7): a fonte é a forma COM o padrão, o clicado
            // é a forma que passa a ser o desenho que se repete. ⚠️ O `guide == pick.source()` logo
            // acima já barra o ciclo — e a `source_shape` do memo barra-o outra vez, porque o
            // documento pode chegar lá por outro caminho (um save, um replay).
            crate::vec_pick::PathPick::TexturePatternArt(host, slot) => {
                // ⭐⭐⭐ **A FORMA ESCOLHIDA TRAZ O TAMANHO DELA** (report do Enio, 2026-08-30: um
                // grupo alto virava um padrão achatado). O padrão nasceu sem arte, logo com um
                // `size` QUADRADO — e um quadrado não é uma escolha, é um marcador.
                //
                // ⚠️ Pela porta que ASSA (`art_dims` -> `bake_dims`) e com a MESMA expansão de
                // objecto, senão o ladrilho tem um aspecto e a colocação tem outro.
                let fonte = ph2d_vec_scene::PatternSource::Shape(guide);
                let xf = ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
                let arte = crate::texture_pattern_pick::art_dims(
                    &gfx.asset_db,
                    &gfx.vec_scene,
                    &xf,
                    &self.vec.live_drawn,
                    host,
                    &fonte,
                    &|id| {
                        ph2d_vec_entities::entities::object_selection_for(
                            &gfx.sim,
                            &gfx.vec_scene,
                            &self.vec.entities,
                            id,
                        )
                    },
                );
                let (size, _) =
                    crate::texture_pattern_pick::default_placement(&gfx.vec_scene, host, arte);
                crate::texture_pattern_edit::set_source(&mut gfx.vec_scene, host, slot, fonte, size)
            }
            // ⭐⭐⭐ **A ARTE de um PINCEL** (plano 36, W4): a fonte é a forma COM o pincel, o clicado
            // é a forma que passa a ser o motivo repetido ao longo do contorno dela.
            //
            // ⚠️ O `guide == pick.source()` logo acima já barra o ciclo — e a `brush_live::art_of`
            // barra-o outra vez, porque o documento pode chegar lá por outro caminho (um save, um
            // replay). *Duas metades porque as duas portas existem.*
            crate::vec_pick::PathPick::BrushArt(host) => {
                // ⚠️ A recusa é sobre PERTENÇA (a arte pode ser um GRUPO), e por isso a porta
                // precisa da expansão de objecto — a MESMA que a resolução usa.
                //
                // ⚠️ A expansão é medida ANTES do empréstimo mutável — a porta só pergunta pelo
                // `guide` (é ele a arte), e o `&mut scene` da escrita não coexiste com o `&scene`
                // que a expansão lê.
                let membros = ph2d_vec_entities::entities::object_selection_for(
                    &gfx.sim,
                    &gfx.vec_scene,
                    &self.vec.entities,
                    guide,
                );
                crate::vec_stroke_paint::set_art(&mut gfx.vec_scene, host, guide, &|_| {
                    membros.clone()
                })
            }
            // **O vínculo da row** (W8b.3): a fonte é o WIDGET, o clicado é a forma dirigida.
            crate::vec_pick::PathPick::WidgetBind(widget) => {
                crate::vec_widget_edit::bind(&mut gfx.sim, &self.vec.entities, widget, guide)
            }
        };
        if done {
            self.vec.path_pick = None;
            eprintln!("[ph2d-vec] pick: preso ao caminho-guia");
        }
    }
}
