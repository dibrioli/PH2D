//! **Fase do quadro: OS VERBOS DE RECEITA E DE ASSETS** — abrir o navegador de assets, o *Aplicar* por degrau, a
//! peça acrescentada, a troca de variante e os verbos de catálogo (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos de receita e de assets que o dreno do barramento recolheu neste quadro.
pub(super) struct RecipeVerbIntents {
    pub(super) catalog_verbs: Vec<ph2d_editor_core::action_bus::CatalogVerb>,
    pub(super) swap_variant: Option<(u64, u64)>,
    pub(super) apply_added: Option<u64>,
    pub(super) apply_to_level: Option<(u64, u64)>,
    pub(super) open_asset_browser: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_recipe_and_asset_verbs(&mut self, intents: RecipeVerbIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            toasts,
            vec_scene,
            hero_screen,
            asset_catalogs,
            component_registry,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let RecipeVerbIntents {
            catalog_verbs,
            swap_variant,
            apply_added,
            apply_to_level,
            open_asset_browser,
        } = intents;
        // ⭐⭐⭐ **A troca de VARIANTE, aplicada** (ADR-0164 / F5, critério 2).
        //
        // ⚠️ **Aqui, e não no dreno do Inspector**, porque é aqui que o `sim` e o **eco** estão
        // os dois à mão — a troca tem de o esquecer, senão o passe seguinte lê a diferença
        // contra o mestre NOVO como *«a instância mexeu-se»* e congela a cópia com o valor do
        // mestre VELHO ([`ph2d_app_components::instance_variant::swap`]).
        // ⭐⭐⭐ **RENOMEAR O VALOR de uma propriedade** (report do Enio, 2026-08-31).
        //
        // ⚠️ **O sujeito é a RECEITA, e o gesto nasceu sobre a CÓPIA.** É a razão de existir:
        // autorar o valor obrigava a seleccionar outro objecto do que aquele que se está a
        // olhar — e ele tentou pelo nome da cópia quatro vezes, com o modelo a ignorá-lo
        // correctamente as quatro.
        //
        if open_asset_browser {
            <_ as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
                hero,
                ph2d_panel_asset_browser::PANEL_ID,
                true,
            );
        }
        // ⭐⭐⭐ **APLICAR num degrau da escada** (ADR-0164 / F5, critério 4).
        //
        // ⚠️ **O toast diz a QUE receita** — os degraus ficam um debaixo do outro no cartão e a
        // diferença entre eles só se vê no gesto SEGUINTE (mexer noutra cópia daquele mestre).
        // Uma confirmação igual para os dois deixaria o artista sem saber qual carregou.
        if let Some((entity_bits, master)) = apply_to_level {
            let name = inspector_instance::master_named(sim, master)
                .unwrap_or_else(|| "prefab".to_string());
            match ph2d_app_components::instance_apply_deep::apply_to_level(
                sim,
                component_registry,
                &mut self.instance_echo,
                ph2d_ecs::Entity::from_bits(entity_bits),
                master,
                &mut ph2d_app_components::instance_docs::OwnedDocs {
                    vec_scene,
                    vec_entities: &mut self.vec.entities,
                },
            ) {
                Ok(done) if done.changed == 0 && done.left == 0 => {
                    toasts.push(Toast::info("Nothing overridden here"));
                }
                Ok(done) => {
                    // ⚠️ **O que ficou por aplicar é DITO** — uma excepção cuja escada não
                    // alcança aquela receita fica onde está, e um número que desaparece em
                    // silêncio lê-se como trabalho perdido.
                    toasts.push(Toast::success(if done.left > 0 {
                            format!(
                                "Applied {} change(s) to \u{201c}{name}\u{201d} \u{2014} {} left (not part of it)",
                                done.changed, done.left
                            )
                        } else {
                            format!("Applied {} change(s) to \u{201c}{name}\u{201d}", done.changed)
                        }));
                    self.title_dirty = true;
                }
                Err(_) => {
                    toasts.push(Toast::warning("Not part of an instance"));
                }
            }
        }
        // ⭐⭐⭐ **APLICAR uma peça acrescentada** (F5.11) — ela entra na receita e o passe
        // estrutural leva-a às irmãs no quadro seguinte.
        //
        // ⚠️ **O sujeito resolve-se por `StableId`**, e não pelos bits que o cartão viu: entre
        // o clique e este ponto pode ter corrido um Ctrl+Z, que respawna tudo com bits novos.
        if let Some(piece) = apply_added {
            let mut docs = ph2d_app_components::instance_docs::OwnedDocs {
                vec_scene,
                vec_entities: &mut self.vec.entities,
            };
            let subject = ph2d_app_components::instance_verbs::entity_for_stable_id(sim, piece)
                .map(ph2d_ecs::Entity::from_bits);
            match subject
                .ok_or(ph2d_app_components::instance_added::AddRefusal::NotAdded)
                .and_then(|e| {
                    ph2d_app_components::instance_added::promote(
                        sim,
                        component_registry,
                        &mut docs,
                        e,
                    )
                }) {
                Ok(p) => {
                    let name = crate::render_loop::inspector_instance::master_named(sim, p.master)
                        .unwrap_or_else(|| "the prefab".to_string());
                    toasts.push(Toast::success(format!(
                        "Added {} piece(s) to \u{201c}{name}\u{201d} \u{2014} every copy gets them",
                        p.pieces
                    )));
                    self.title_dirty = true;
                }
                // ⚠️ **Todo caminho negativo fala** — a lei do menu dos verbos. Um botão que
                // come o clique em silêncio é pior que um ausente.
                Err(ph2d_app_components::instance_added::AddRefusal::NotAdded) => {
                    toasts.push(Toast::warning(
                        "That piece came from the component \u{2014} it is already in it",
                    ));
                }
                Err(_) => {
                    toasts.push(Toast::warning("That is not a piece of a copy"));
                }
            }
        }
        if let Some((root_bits, master)) = swap_variant {
            match ph2d_app_components::instance_variant::swap(
                sim,
                &mut self.instance_echo,
                ph2d_ecs::Entity::from_bits(root_bits),
                master,
                // ⚠️ **A fileira de versões nunca adivinha.** Ali os mestres são aparentados
                // por construção; uma queda para heurística seria a operação automática que o
                // plano F5 proíbe. Quem pede um dos três modos é o menu da biblioteca.
                ph2d_app_components::instance_swap_match::WhenUnrelated::Refuse,
            ) {
                Ok(r) => {
                    toasts.push(Toast::success(if r.dropped > 0 {
                        format!(
                            "Switched variant \u{2014} {} override(s) kept, {} piece(s) unused",
                            r.overrides_kept, r.dropped
                        )
                    } else {
                        format!(
                            "Switched variant \u{2014} {} override(s) kept",
                            r.overrides_kept
                        )
                    }));
                    self.title_dirty = true;
                }
                // ⚠️ **Todo caminho negativo fala.** Um chip que come o clique em silêncio é
                // pior que um ausente — a mesma lei que o menu dos verbos paga.
                Err(ph2d_app_components::instance_variant::SwapRefusal::Already) => {}
                Err(ph2d_app_components::instance_variant::SwapRefusal::Unrelated) => {
                    toasts.push(Toast::warning(
                            "These components are not related \u{2014} switching would lose every override",
                        ));
                }
                Err(_) => {
                    toasts.push(Toast::warning("That is not a copy of a prefab"));
                }
            }
        }
        // ⭐⭐ **Os verbos de catálogo** (wave A3). ⚠️ Eles correm ANTES do `hierarchy::dispatch`
        // de propósito: a taxonomia não é o mundo, e misturá-los no mesmo bloco daria a
        // impressão de que um catálogo é um objecto da cena.
        for v in &catalog_verbs {
            if crate::asset_catalog_verbs::drain(v, asset_catalogs, toasts) {
                self.title_dirty = true;
            }
        }
    }
}
