//! **Fase do quadro: OS COMANDOS DA PILHA DE FILTROS** — a paragem do gradiente de um filtro e os comandos da pilha: acrescentar, tirar,
//! reordenar, esconder, o modo e a mistura (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct FilterCommandsIntents {
    pub(super) pending_filter_cmd: Option<crate::fx_live::FilterHit>,
    pub(super) pending_filter_stop: Option<(usize, u8, f32)>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_filter_commands(
        &mut self,
        intents: FilterCommandsIntents,
    ) -> Option<Vec<ph2d_vec_scene::VecPathId>> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { sim, .. } = FrameGfx::of(gfx);
        let FilterCommandsIntents {
            pending_filter_cmd,
            pending_filter_stop,
        } = intents;
        use crate::fx_live::FilterHit;
        let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
        // O punho arrastado move APENAS a posição — a cor é da swatch, e o índice é o de
        // AUTORIA (quem ordena é o consumidor), então arrastar por cima do vizinho não
        // re-liga o dedo a outro stop.
        if let Some((row, idx, x)) = pending_filter_stop {
            let diag = std::env::var_os("PH2D_FX_RAMP_DIAG").is_some();
            if diag {
                eprintln!(
                    "[ramp] shell aplica: {} forma(s) selecionada(s), linha {row} stop {idx}",
                    sel.len()
                );
            }
            crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                if let Some(op) = f.ops.get_mut(row)
                    && usize::from(idx) < usize::from(op.stop_count)
                    && let Some(slot) = op.stop_pos.get_mut(usize::from(idx))
                {
                    *slot = x.clamp(0.0, 1.0);
                    if diag {
                        eprintln!("[ramp] stop_pos escrito: {:?}", op.stop_pos);
                    }
                } else if diag {
                    eprintln!(
                        "[ramp] RECUSADO: ops={} stop_count={:?}",
                        f.ops.len(),
                        f.ops.get(row).map(|o| o.stop_count)
                    );
                }
            });
        }
        if let Some(cmd) = pending_filter_cmd {
            match cmd {
                // Um "Add" numa forma SEM pilha cria a pilha; com pilha, empilha no fim
                // (o topo visual). O degrau nasce com defaults VISÍVEIS.
                FilterHit::Add(kind) => {
                    for id in &sel {
                        let one = std::slice::from_ref(id);
                        match crate::fx_live::spec_of(sim, &self.vec.entities, *id) {
                            Some(mut f) if f.has_room() => {
                                f.ops.push(ph2d_ecs::FxOp::new(kind));
                                crate::fx_live::set_filter(sim, &self.vec.entities, one, Some(f));
                            }
                            Some(_) => {}
                            None => {
                                crate::fx_live::set_filter(
                                    sim,
                                    &self.vec.entities,
                                    one,
                                    Some(ph2d_ecs::VecFilter::single(ph2d_ecs::FxOp::new(kind))),
                                );
                            }
                        }
                    }
                }
                FilterHit::Remove(row) => {
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                        if row < f.ops.len() {
                            f.ops.remove(row);
                        }
                    });
                }
                FilterHit::Up(row) => {
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                        f.move_up(row);
                    });
                }
                FilterHit::Down(row) => {
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                        f.move_down(row);
                    });
                }
                FilterHit::Hide(row) => {
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                        if let Some(op) = f.ops.get_mut(row) {
                            op.enabled = !op.enabled;
                        }
                    });
                }
                // O MODO é a LEI do degrau, não a intensidade dele — e um clique num modo
                // que o TIPO não oferece é recusado aqui (o painel não o pinta, mas a
                // recusa mora onde o valor é escrito).
                FilterHit::Mode(row, mode) => {
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                        if let Some(op) = f.ops.get_mut(row)
                            && (mode as usize) < ph2d_ecs::FxOp::spec(op.kind).modes.len()
                        {
                            op.mode = mode;
                        }
                    });
                }
                // A LEI DE MISTURA — *como a cor deste degrau encosta na que já está ali*.
                // Mesma recusa do MODO, pela porta única `FxOp::takes_blend`: um clique
                // numa lei que o TIPO não toma é recusado ONDE o valor é escrito, e não só
                // onde ele é pintado. (O popover nem chega a existir num tipo que não a
                // oferece — mas a segunda metade é o que impede um arquivo, ou um teste,
                // de instalar um número órfão.)
                FilterHit::Blend(row, blend) => {
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                        if let Some(op) = f.ops.get_mut(row)
                            && op.takes_blend()
                            && blend < ph2d_ecs::FxOp::BLEND_KINDS
                        {
                            op.blend = blend;
                        }
                    });
                }
                // A swatch só ABRE o picker (o `register_picker_swatch` faz isso); a cor
                // é lida abaixo, do alvo do picker.
                // O trilho da rampa: `+` põe um stop no maior vão com a cor que já está
                // ali (não muda o desenho), `−` tira o SELECIONADO com piso em dois.
                FilterHit::StopAdd(row) => {
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                        if let Some(op) = f.ops.get_mut(row) {
                            crate::fx_live::add_stop(op);
                        }
                    });
                }
                FilterHit::StopRemove(row) => {
                    let sel_stop = usize::from(ph2d_panel_vector::selected_stop(row));
                    crate::fx_live::edit(sim, &self.vec.entities, &sel, |f| {
                        if let Some(op) = f.ops.get_mut(row) {
                            crate::fx_live::remove_stop(op, sel_stop);
                        }
                    });
                }
                FilterHit::Color(_)
                | FilterHit::StopColor(_)
                | FilterHit::ColorB(_)
                | FilterHit::Radius(_)
                | FilterHit::OffX(_)
                | FilterHit::OffY(_)
                | FilterHit::Opacity(_)
                | FilterHit::Scale(_)
                | FilterHit::Detail(_)
                | FilterHit::Seed(_)
                | FilterHit::Grow(_)
                | FilterHit::Hue(_)
                | FilterHit::Sat(_)
                | FilterHit::Bright(_) => {}
            }
        }
        Some(sel)
    }
}
