//! **Fase do quadro: OS VERBOS DO MORPH** (plano 32) — fazer o conjunto, os três verbos de mundo (Play ·
//! Desconectar · Desfazer tudo) e a tecla de uma forma, pela cadeia `if … else if …` que os separa (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! **Corte um nível abaixo:** o bloco do painel vectorial é UM statement de 467 linhas; esta fase é um grupo dos statements do CORPO dele, e a chamada fica dentro do bloco, no sítio do corpo.

use super::*;

/// O pedido de verbo do Morph que o dreno do barramento recolheu neste quadro.
pub(super) struct MorphVerbIntents {
    pub(super) pending_morph_arrow: Option<crate::vec_morph_edit::MorphCmd>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_morph_verbs(
        &mut self,
        intents: MorphVerbIntents,
        sel: Vec<ph2d_vec_scene::VecPathId>,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            ui_states,
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let MorphVerbIntents {
            pending_morph_arrow,
        } = intents;
        // ⭐⭐ **FAZER O CONJUNTO** (plano 32 W8) — as formas escolhidas viram um objecto
        // com todas as transições ligadas. ⚠️ **Porta própria e não o `apply`**: ele age
        // sobre o componente de um Morph que aqui ainda **não existe**, e é o `sync` do
        // quadro seguinte que faz nascer a entidade — daí o pendente.
        if pending_morph_arrow == Some(crate::vec_morph_edit::MorphCmd::MakeSet) {
            if let Some(p) = ph2d_vec_entities::morph_set::create(
                sim,
                vec_scene,
                &self.vec.entities,
                &sel,
                ph2d_panel_vector::ids::MAX_MORPH_STATES,
            ) {
                eprintln!(
                    "[ph2d-vec] morph states: {} formas, {} transicoes",
                    p.members.len(),
                    p.members.len() * (p.members.len() - 1)
                );
                // ⭐⭐ **O OBJECTO NOVO FICA SELECCIONADO** — a mesma escolha do botão
                // Morph ao lado, e aqui ela é load-bearing por uma razão a mais: sem isto
                // a selecção continuaria a ser as formas-membro, que acabaram de ficar
                // **ocultas e filhas do conjunto** — e a seção voltaria a oferecer
                // *"Make Morph States"* sobre elas, prometendo um segundo conjunto por
                // cima do primeiro.
                self.vec.pen.select_many(&[p.path]);
                self.vec.morph_set_pending = Some(p);
                // ⭐ **COMMIT** — criar o conjunto muda o documento de vez, e é exactamente
                // o gesto que uma confirmação pelo ouvido serve (a lei do D1).
                self.pending_ui_sound = Some(crate::ui_sound::UiSound::Commit);
            }
        }
        // ⭐⭐ **OS TRÊS VERBOS DE MUNDO** (plano 32 W11b) — Play · Desconectar · Desfazer
        // tudo. Eles reparentam, mostram e apagam entidades, então **não** cabem no `apply`
        // (que só tem o componente).
        else if let Some(cmd) = pending_morph_arrow
            && matches!(
                cmd,
                crate::vec_morph_edit::MorphCmd::Play { .. }
                    | crate::vec_morph_edit::MorphCmd::Disconnect { .. }
                    | crate::vec_morph_edit::MorphCmd::Dissolve
            )
            && let Some(host) =
                crate::vec_morph_edit::morph_of_selection(sim, &self.vec.entities, &sel)
        {
            // ⚠️ **Cada braço deriva o que precisa DENTRO da porta dele** — a lista de
            // formas era derivada aqui e passada aos três, e era ela que convidava a
            // escrever a lógica de cada verbo neste `match` (onde nenhum gate a alcança).
            // ⚠️ **Os dois verbos que APAGAM o conjunto saem pela MESMA porta** — o
            // `Dissolve` sempre, e o `Disconnect` quando tira a penúltima forma (abaixo de
            // `MIN_STATES` um conjunto deixa de ser uma relação). Cada um a remover o path
            // por si seriam duas respostas a *"o que é apagar um conjunto"*.
            let removed = match cmd {
                // ⭐ **PLAY: liga a pré-visualização se estiver desligada.** A máquina só
                // anda dentro do modo (é ele que tem o relógio), e um Play que não tocasse
                // nada seria um botão morto com nome de verbo.
                crate::vec_morph_edit::MorphCmd::Play { row } => {
                    self.morph_preview = true;
                    // ⚠️ **Pela porta**, e não por um `get_mut` aqui: este braço corre
                    // DEPOIS do `tick`, que esvazia o mapa fora do modo — o `get_mut`
                    // encontrava-o vazio e o botão só ligava a pré-visualização (report do
                    // Enio, 2026-08-26). A porta abre a máquina semeada pelo mundo.
                    crate::morph_machine_drive::play(
                        &mut self.morph_machines,
                        sim,
                        &self.vec.entities,
                        host,
                        row,
                    );
                    None
                }
                // ⚠️ **Pela porta**, e não pelas metades aqui: a segunda — a forma solta
                // LEVAR as poses dela — não é alcançável de um teste escrita neste braço, e
                // foi assim que ela ficou por escrever uma wave inteira.
                crate::vec_morph_edit::MorphCmd::Disconnect { row } => {
                    // ⚠️ **UMA linha por CLIQUE** (`PH2D_MORPH_LOG=1`) — e ela imprime as
                    // duas coisas que decidem se a arrumação alcança a tabela: o
                    // `VecPathId` do conjunto, e as CHAVES que a tabela de States tem.
                    // Se as duas não baterem, a arrumação sai cedo e nada acontece.
                    if crate::morph_machine_drive::log_on() {
                        eprintln!(
                            "[morph] CLIQUE ⊘ row={row} conjunto={:?} \
                                     chaves-da-tabela={:?} formas={:?}",
                            ph2d_vec_entities::morph_set::path_of(&self.vec.entities, host),
                            ui_states.hosts().collect::<Vec<_>>(),
                            ph2d_vec_entities::morph_set::graph_of(sim, &self.vec.entities, host)
                                .shapes(),
                        );
                        // ⚠️ **O INVENTÁRIO da cena**, porque a linha acima disse que a
                        // tabela está sob um id que não é o do conjunto — e a pergunta
                        // seguinte é *o que é aquele id*. Sem isto, a resposta era mais
                        // uma corrida do Enio.
                        for p in vec_scene.paths() {
                            let e = ph2d_vec_entities::morph_set::path_of(&self.vec.entities, host)
                                .filter(|h| *h == p.id);
                            let ent = self
                                .vec
                                .entities
                                .get(&p.id)
                                .map(|&b| ph2d_ecs::Entity::from_bits(b));
                            let (morph, machine, name, pai) =
                                ent.map_or((false, false, String::new(), None), |en| {
                                    let w = sim.world();
                                    (
                                        w.get::<ph2d_ecs::VecMorph>(en).is_some(),
                                        w.get::<ph2d_ecs::VecMorphMachine>(en).is_some(),
                                        w.get::<ph2d_ecs::Name>(en)
                                            .map(|n| n.0.clone())
                                            .unwrap_or_default(),
                                        w.get::<ph2d_ecs::ChildOf>(en).and_then(|c| {
                                            ph2d_vec_entities::morph_set::path_of(
                                                &self.vec.entities,
                                                c.parent(),
                                            )
                                        }),
                                    )
                                });
                            eprintln!(
                                "[morph]   path {} nome={name:?} morph={morph} \
                                         maquina={machine} pai={pai:?} verts={} e-o-conjunto={}",
                                p.id,
                                p.verts.len(),
                                e.is_some(),
                            );
                        }
                    }
                    ph2d_vec_entities::morph_set::disconnect_row(sim, &self.vec.entities, host, row)
                }
                crate::vec_morph_edit::MorphCmd::Dissolve => {
                    ph2d_vec_entities::morph_set::dissolve(sim, &self.vec.entities, host)
                }
                _ => None,
            };
            if let Some(path) = removed {
                vec_scene.remove_path(path);
                self.vec.pen.clear();
            }
            self.pending_ui_sound = Some(crate::ui_sound::UiSound::Commit);
        }
        // ⭐ **A TECLA de uma forma** (plano 32 W4). As acções são as MESMAS que o menu
        // mostrou (as do Input Map do projecto): resolver o índice contra uma segunda
        // leitura poria o nome escolhido a apontar para outro.
        else if let Some(cmd) = pending_morph_arrow
            && let Some(e) =
                crate::vec_morph_edit::morph_of_selection(sim, &self.vec.entities, &sel)
        {
            let actions: Vec<String> = hero
                .input_map
                .actions()
                .iter()
                .map(|a| a.name.clone())
                .collect();
            crate::vec_morph_edit::apply(sim, &self.vec.entities, e, cmd, &actions);
        }
    }
}
