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
}
