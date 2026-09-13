//! **O clique, o PREMIR da ferramenta vetorial** — ramos do `on_mouse_input` ([`super`]): a cadeia de prioridade
//! do press no canvas (a região do Node, as alças de gradiente, Text, Build, lápis, Connect, Blend, Width) · o
//! Corte, o Trim, o Balde e o Osso · as quinas (Fillet/Chamfer) · e a caneta/forma com o snap. Os corpos
//! MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da chamada, pela mesma ordem.
//!
//! ⚠️ Cada ramo devolve `true` onde o braço fazia `return;`; o braço chama-os em sequência e devolve ao primeiro
//! `true` — que é a cadeia de early-returns que o braço era.

use super::*;

impl crate::App {
    /// A caneta e a forma: os alvos de snap do gesto, a gaiola do Envelope e o nó do modo Node (congelando a
    /// receita viva), a lâmina do Corte, a forma com o canto encaixado, e os alvos refeitos pelo que foi agarrado.
    pub(super) fn ramo_vetor_premido_caneta(&mut self) -> bool {
        let shape_kind = shape_kind_for_mode(&self.vec.draw_config);
        // Alt held → the Pen breaks the tangent when grabbing a handle.
        let alt = self.modifiers.alt_key();
        // Snap targets for THIS gesture: the whole scene as it stands.
        // Rebuilt right after the press, once we know what got grabbed.
        self.vec_rebuild_snap_targets(&[], &[]);
        let cfg = self.vec_snap_cfg(self.vec_px_to_world());
        let targets = std::mem::take(&mut self.vec.snap_targets);
        if let Some(gfx) = self.gfx.as_mut() {
            let win = gfx.surface.size();
            let w = gfx.camera.screen_to_world(self.last_pointer, win);
            // world-units por pixel (delta de 1px) → limiar/traço em px.
            let w0 = gfx.camera.screen_to_world((0.0, 0.0), win);
            let w1 = gfx.camera.screen_to_world((1.0, 0.0), win);
            let px_to_world = (((w1[0] - w0[0]).powi(2) + (w1[1] - w0[1]).powi(2)).sqrt()) as f64;
            // ADR-0129 Fatia 3: o alvo do gesto de gaiola é o CONTAINER do envelope (sem
            // path), não a forma selecionada — os bits dele estão na seleção do gizmo
            // (regra seleciona-só-o-container). Copy, lido ANTES do borrow mutável de
            // `hero_screen` abaixo. `press` devolve `false` se não for um `VecEnvelope`.
            let env_container = gfx.hero_screen.as_ref().and_then(|h| h.gizmo.selection);
            // `hero_screen` e `vec_scene` são campos IRMÃOS de `AppGfx`: a
            // grade pode ser consultada enquanto o Pen muta a cena.
            let mut hero = gfx.hero_screen.as_mut();
            let mut snap = |p: [f64; 2]| {
                let mut grid = |q: [f64; 2]| {
                    let h = hero.as_mut()?;
                    crate::vec_snap::ask_grid(&mut h.grid.snap_state, q)
                };
                ph2d_vec_edit::snap::snap(&[p], &targets, cfg, Some(&mut grid)).apply(p)
            };
            let node_mode = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Node;
            match shape_kind {
                // Node edita nós e NUNCA cria (ADR-0112). Não encaixa
                // tampouco: o snap serve a quem POSICIONA um ponto novo.
                None if node_mode => {
                    // ADR-0129 Fatia 1: os cantos da gaiola do Envelope são
                    // alças PRÓPRIAS no modo Node (§3.3). Hit-testa-os
                    // PRIMEIRO; um acerto arma o arrasto do canto e PULA o pen
                    // — que agarraria uma âncora da forma COZIDA, revertida
                    // pelo recook do frame seguinte. Um erro cai no
                    // `on_press_node` de sempre (seleção / edição de âncora).
                    //
                    // A alça do TEXTO EM CAMINHO (W5) NÃO vive aqui — ela é do
                    // modo **Select** (a bolinha se perdia no meio das âncoras
                    // do Node; Enio, smoke). Vive ao lado das alças do conector,
                    // mais acima neste arquivo.
                    if crate::envelope_gesture::press(
                        &mut gfx.sim,
                        &gfx.vec_scene,
                        &self.vec.live_drawn,
                        &self.vec.view_derived,
                        env_container,
                        [w[0] as f64, w[1] as f64],
                        px_to_world,
                        self.modifiers.alt_key(),
                        &mut self.vec.envelope_drag,
                    ) {
                        // canto agarrado — o pen fica de fora
                    } else {
                        // **A forma VIVA congela a receita AQUI**, exatamente como no
                        // par Fillet/Chamfer acima — e pelo mesmo motivo: o
                        // `recook_into` reescreve `path.verts` INTEIRO, então um nó
                        // arrastado numa Live Shape sobrevive até o instante em que o
                        // artista encosta num slider de parâmetro, e some **sem erro
                        // nenhum** (o modo de falha que o `corner_handles` descreve;
                        // medido em `vec_node_freeze_tests`).
                        //
                        // ⚠️ Só quando o press vai de fato EDITAR geometria: o
                        // `on_press_node` devolve `Grabbed` tanto ao agarrar um vértice
                        // como ao apenas SELECIONAR a forma pelo preenchimento, então a
                        // pergunta é feita ANTES, à porta que faz a MESMA busca
                        // (`node_edit_hit_at`) — congelar num clique que só seleciona
                        // expandiria a forma sem ninguém pedir.
                        if let Some(pid) = self.vec.pen.node_edit_hit_at(
                            &gfx.vec_scene,
                            [w[0] as f64, w[1] as f64],
                            px_to_world,
                        ) {
                            crate::vec_convert::freeze_shape_recipe(
                                &mut gfx.sim,
                                &self.vec.entities,
                                pid,
                            );
                        }
                        // Node edita âncoras/handles. Arredondar/chanfrar quina não é
                        // mais deste modo — virou o par Fillet/Chamfer (o hit-test aqui
                        // não agarra alça de raio nenhuma).
                        self.vec.pen.on_press_node(
                            &mut gfx.vec_scene,
                            [w[0] as f64, w[1] as f64],
                            px_to_world,
                            alt,
                        );
                    }
                }
                None => {
                    let click = self.vec.pen.on_press(
                        &mut gfx.vec_scene,
                        [w[0] as f64, w[1] as f64],
                        px_to_world,
                        alt,
                        &mut snap,
                    );
                    // **O modo Corte só muda o que a caneta PRODUZ.** Um caminho
                    // começado aqui em modo Cut é a LÂMINA, e não desenho: fica
                    // pendente até o `sync` lhe dar entidade, e aí recebe o
                    // `VecCutPath` (o padrão exato do conector e do blend).
                    if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Cut
                        && click == ph2d_vec_edit::PenClick::Started
                        && let Some(id) = self.vec.pen.selected()
                    {
                        self.vec.cut_pending = Some(id);
                    }
                }
                Some(kind) => {
                    // A ferramenta de forma não faz hit-test: o canto pode
                    // ser encaixado antes de entrar.
                    let p = snap([w[0] as f64, w[1] as f64]);
                    // Os parâmetros cruzam a fronteira de unidade AQUI: a
                    // tool os guarda como o usuário os digita (px nos
                    // raios), a geometria só fala mundo.
                    let values = ph2d_tool_vector::shapes::to_world(
                        kind,
                        &self.vec.draw_config.values,
                        px_to_world,
                    );
                    self.vec.shape.on_press(
                        &mut gfx.vec_scene,
                        kind,
                        values,
                        p,
                        px_to_world,
                        shape_constraint(self.modifiers),
                    );
                }
            }
            self.vec.snap_targets = targets;
            // Tocar um filho seleciona o GRUPO (a árvore é a Hierarquia).
            // Depois do press, porque só agora sabemos o que foi agarrado.
            if let Some(primary) = self.vec.pen.selected() {
                let members = self.vec_object_selection_for(primary);
                self.vec.pen.set_object_selection(&members);
            }
            // Agora sabemos o que o press agarrou: o que se move sai dos
            // alvos (uma âncora não pode encaixar em si mesma; a forma em
            // desenho não é referência de nada).
            match (self.vec.pen.dragging_anchors(), self.vec.shape.selected()) {
                // ⚠️ Os pares vêm PRONTOS do pen: ele passou a guardar o dono de cada
                // nó, então a re-montagem que morava aqui (`map(|&v| (pid, v))`) some
                // — e com ela o pressuposto de que todas as âncoras em movimento
                // pertencem à MESMA forma, que um arrasto multi-forma quebra.
                (Some(moving), _) => self.vec_rebuild_snap_targets(&[], &moving),
                (None, Some(sid)) => self.vec_rebuild_snap_targets(&[sid], &[]),
                (None, None) => {}
            }
            return true;
        }
        self.vec.snap_targets = targets;
        false
    }

    /// As ferramentas de quina (Fillet/Chamfer): o caminho sob o cursor, a receita viva congelada só com a quina
    /// ACERTADA, o diagnóstico `PH2D_CORNER_LOG`, e o press da quina fora dos hosts de relação.
    pub(super) fn ramo_vetor_premido_quina(&mut self) -> bool {
        if self.vec.draw_config.mode.is_corner_tool() {
            let chamfer = self.vec.draw_config.mode.corner_is_chamfer();
            let px_to_world = self.vec_px_to_world();
            if let Some(world) = self.vec_world_at(self.last_pointer)
                && let Some(gfx) = self.gfx.as_mut()
            {
                let hit_r = 12.0 * px_to_world;
                // (re)seleciona o path sob o cursor num acerto — o gesto vale sem
                // pré-selecionar; num erro mantém a seleção (uma quina do path já
                // selecionado ainda pega).
                if let Some(pid) = self.vec.pen.path_at(&gfx.vec_scene, world, hit_r) {
                    self.vec.pen.select(Some(pid));
                }
                // **A forma VIVA congela a receita AQUI** (Enio: *"fillet e chanfer
                // nao funciona diretamente nos vertex das shapes"*). Um raio
                // por-vértice não sobrevive ao `recook_into`, então antes a
                // ferramenta RECUSAVA a forma — o que lê como "não funciona". Agora
                // ela faz, dentro do gesto, o "Convert to Curves" que o artista faria
                // à mão. Só com a quina de fato ACERTADA: congelar num clique que
                // erra expandiria a forma sem ninguém pedir.
                if let Some(pid) = self.vec.pen.selected()
                    && self
                        .vec
                        .pen
                        .corner_hit_at(&gfx.vec_scene, world, px_to_world)
                {
                    crate::vec_convert::freeze_shape_recipe(&mut gfx.sim, &self.vec.entities, pid);
                }
                // DIAGNÓSTICO (`PH2D_CORNER_LOG=1`): os raios do path NO INSTANTE do
                // press. Serve para partir em dois o report *"a 1ª operação é
                // apagada"*: se os raios anteriores já vêm ZERADOS aqui, quem apagou
                // foi algo ENTRE os gestos (um passe por-frame); se vêm inteiros e
                // somem depois, foi o gesto. O motor e o recook da forma viva já
                // estão provados limpos por gate, então o eraser está fora deles.
                if std::env::var_os("PH2D_CORNER_LOG").is_some()
                    && let Some(pid) = self.vec.pen.selected()
                {
                    // `shape` = a receita ainda esta' pendurada? Se ela reaparece
                    // entre gestos, o `recook_into` reescreve `verts` e zera TODOS
                    // os raios de uma vez -- o unico mecanismo que casa com "so' um
                    // raio vivo por vez, com a contagem de vertices intacta".
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
                    eprintln!("[corner] PRESS chamfer={chamfer} shape={shape} radii={radii:?}");
                }
                // Os hosts de RELAÇÃO (conector, morph, blend, envelope) seguem
                // recusados: ali a geometria é uma relação, e soltá-la sem o artista
                // pedir destruiria o que ele construiu. A forma viva já saiu acima.
                let derived = self.vec.pen.selected().is_some_and(|pid| {
                    crate::corner_handles::has_derived_verts(&gfx.sim, &self.vec.entities, pid)
                });
                if !derived {
                    self.vec
                        .pen
                        .on_press_corner(&mut gfx.vec_scene, world, px_to_world, chamfer);
                }
            }
            return true;
        }
        false
    }

    /// O Trim (apaga o pedaço que o realce mostra), o Balde (deposita a face acesa) e o Osso (agarra, aponta ou marca
    /// a origem) — os três consomem o press SEMPRE que estão na mão. O Corte não tem braço: desenha pela caneta.
    pub(super) fn ramo_vetor_premido_corte_balde_osso(&mut self) -> bool {
        // **Modo Corte** (W4): NÃO há early return aqui, e é o desenho inteiro — a
        // linha de corte é desenhada pela CANETA, que é o caminho por onde este press
        // cai adiante. Uma rota própria seria uma segunda resposta a *"como se desenha
        // uma curva?"*, e ela divergiria da caneta no primeiro refino (handles,
        // fechamento, snap, continuar por um endpoint — tudo isto sai de graça).
        //
        // O que o modo muda é só o que a caneta PRODUZ: o caminho nasce marcado como
        // lâmina (`vec_cut_line::adopt_new_path`, depois do `sync`).
        // ⭐⭐⭐ **APARAR** (plano 38): o clique apaga o pedaço que o realce está a
        // mostrar. ⚠️ **O pedaço vem do estado do QUADRO** (`vec_trim_hit`), e não de
        // um cálculo feito aqui: o que o artista vê a vermelho é literalmente o que
        // some. Recalcular no clique abriria a porta para o cursor ter andado um pixel
        // entre o desenho e o gesto — e numa ferramenta destrutiva isso é apagar outra
        // coisa.
        //
        // ⚠️ **A forma VIVA congela a receita AQUI**, como no Fillet/Chamfer: um corte
        // não sobrevive ao `recook_into`, então sem isto o pedaço voltaria no quadro
        // seguinte e a ferramenta leria como *"não funciona"*.
        if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Trim {
            if let Some(hit) = self.vec.trim_hit
                && let Some(gfx) = self.gfx.as_mut()
            {
                crate::vec_convert::freeze_shape_recipe(&mut gfx.sim, &self.vec.entities, hit.path);
                if crate::vec_trim::apply(&mut gfx.vec_scene, &hit) {
                    // A selecção pode ter deixado de existir (a peça toda saiu).
                    if gfx.vec_scene.path(hit.path).is_none() {
                        self.vec.pen.select(None);
                    }
                }
                self.vec.trim_hit = None;
                self.vec.trim_piece.clear();
            }
            // ⛔ Consome o press SEMPRE que a ferramenta está na mão: um clique no
            // vazio não pode cair na cadeia de baixo e começar a desenhar uma forma.
            return true;
        }
        // ⭐⭐⭐ **O BALDE** (plano 40): o clique deposita a face que o realce está a
        // mostrar. ⚠️ **A geometria vem do estado do QUADRO** (`vec_bucket_face`), como
        // no Trim: o que o artista vê aceso é literalmente o que fica.
        //
        // ⛔ Consome o press SEMPRE, pela razão do Trim: um clique no vazio não pode
        // cair na cadeia de baixo e começar a desenhar uma forma.
        if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Bucket {
            // ⚠️ A guarda de fora é a de sempre: o `apply_bucket` pergunta a tinta ANTES das guardas
            // dele (e avisa se ela for transparente), então chamá-lo sem face nem `gfx` imprimiria um
            // aviso que este clique nunca imprimiu.
            if self.vec.bucket_face.is_some() && self.gfx.is_some() {
                self.apply_bucket();
            }
            return true;
        }
        // ⭐⭐⭐ **O OSSO** (estudo 42 item 5, doc 47 §2.6): apontar um osso
        // SELECCIONA-o (é assim que se ramifica); o vazio marca a ORIGEM, e o `release`
        // faz o osso dali até onde a mão soltou.
        //
        // ⛔ Consome o press SEMPRE que a ferramenta está na mão, pela razão do Trim e
        // do Balde: um clique no vazio não pode cair na cadeia de baixo e começar a
        // desenhar uma forma.
        // ⭐⭐⭐ **O OSSO** (estudo 42 item 5, doc 47 §2.6): a DECISÃO vive na porta
        // única `bone_gesture::press` — aqui ficam só os efeitos. ⚠️ Foi tê-la dentro
        // deste ficheiro que escondeu a metade que faltava (report do Enio,
        // 2026-09-06: *"o bind não funciona"* — apontar uma forma nunca a
        // seleccionava, e o botão só sabia recusar).
        //
        // ⛔ Consome o press SEMPRE que a ferramenta está na mão, pela razão do Trim e
        // do Balde: um clique no vazio não pode cair na cadeia de baixo e começar a
        // desenhar uma forma.
        if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Bone {
            if let Some(world) = self.vec_world_at(self.last_pointer) {
                let px = self.vec_px_to_world();
                let sel = self.selected_bone_bits();
                // ⭐ O VERBO do arrasto, que o grupo alternável da seção SKELETON diz.
                let acao = self.vec.draw_config.bone_action;
                let decisao = self.gfx.as_ref().map(|g| {
                    crate::bone_gesture::press(
                        &g.sim,
                        &g.vec_scene,
                        &self.vec.pen,
                        world,
                        px,
                        sel,
                        acao,
                    )
                });
                match decisao {
                    Some(crate::bone_gesture::BonePress::Grab { bone, part }) => {
                        // Agarrar o osso é o gesto de o POSAR (o gizmo de sprite não
                        // serve — ver `bone_pose::pose`), e também o que o
                        // selecciona: o pai do próximo osso é o que está aceso.
                        self.skeleton.bone_pose = Some((bone, part));
                        if let Some(gfx) = self.gfx.as_mut()
                            && let Some(hero) = gfx.hero_screen.as_mut()
                        {
                            hero.gizmo.selection = Some(bone);
                            hero.gizmo.extra_selection.clear();
                        }
                    }
                    // ⭐ Em *Transformar*, um press fora de osso aponta a forma e mais
                    // nada — o *Bind* precisa do sujeito, e nenhum osso nasce aqui.
                    Some(crate::bone_gesture::BonePress::Pick { path: Some(pid) }) => {
                        self.vec.pen.select(Some(pid));
                    }
                    // ⛔ Sem forma sob o cursor, um press em *Transformar* não faz
                    // NADA — nem cria, nem DESMARCA: desmarcar tiraria o sujeito do
                    // `Bind` a cada clique no vazio, e o artista clica no vazio o tempo
                    // todo.
                    Some(crate::bone_gesture::BonePress::Pick { path: None }) => {}
                    Some(crate::bone_gesture::BonePress::Start { birth, pick }) => {
                        self.skeleton.bone_drag = Some(birth);
                        // ⚠️ **O clique que SELECCIONA e o arrasto que faz osso são o
                        // MESMO press**, e é de propósito: um clique curto (< 12 px)
                        // não faz osso nenhum, então apontar uma forma é só apontar —
                        // e é assim que o *Bind* passa a ter sujeito.
                        if let Some(pid) = pick {
                            self.vec.pen.select(Some(pid));
                        }
                    }
                    None => {}
                }
            }
            return true;
        }
        false
    }
}
