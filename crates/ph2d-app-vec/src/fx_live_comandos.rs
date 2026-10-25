//! **Os comandos da pilha de filtros** sobre as formas seleccionadas — a paragem do gradiente
//! arrastada e os comandos da pilha: acrescentar, tirar, reordenar, esconder, o modo e a mistura.
//! A shell só recolhe os pedidos do quadro e chama [`aplica`].

use ph2d_ecs::SimWorld;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::VecPathId;

use crate::fx_live::FilterHit;

/// Aplica a paragem arrastada `(linha, paragem, x)` e o comando `cmd` às formas `sel`.
pub fn aplica(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    sel: &[VecPathId],
    pending_filter_stop: Option<(usize, u8, f32)>,
    pending_filter_cmd: Option<FilterHit>,
) {
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
        crate::fx_live::edit(sim, map, sel, |f| {
            if let Some(op) = f.ops_mut().get_mut(row)
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
                    f.len(),
                    f.ops().get(row).map(|o| o.stop_count)
                );
            }
        });
    }
    if let Some(cmd) = pending_filter_cmd {
        match cmd {
            // Um "Add" numa forma SEM pilha cria a pilha; com pilha, empilha no fim
            // (o topo visual). O degrau nasce com defaults VISÍVEIS.
            FilterHit::Add(kind) => {
                for id in sel {
                    let one = std::slice::from_ref(id);
                    match crate::fx_live::spec_of(sim, map, *id) {
                        Some(mut f) if f.has_room() => {
                            f.push(ph2d_ecs::FxOp::new(kind));
                            crate::fx_live::set_filter(sim, map, one, Some(f));
                        }
                        Some(_) => {}
                        None => {
                            crate::fx_live::set_filter(
                                sim,
                                map,
                                one,
                                Some(ph2d_ecs::VecFilter::single(ph2d_ecs::FxOp::new(kind))),
                            );
                        }
                    }
                }
            }
            FilterHit::Remove(row) => {
                crate::fx_live::edit(sim, map, sel, |f| {
                    f.remove(row);
                });
            }
            FilterHit::Up(row) => {
                crate::fx_live::edit(sim, map, sel, |f| {
                    f.move_up(row);
                });
            }
            FilterHit::Down(row) => {
                crate::fx_live::edit(sim, map, sel, |f| {
                    f.move_down(row);
                });
            }
            FilterHit::Hide(row) => {
                crate::fx_live::edit(sim, map, sel, |f| {
                    if let Some(op) = f.ops_mut().get_mut(row) {
                        op.enabled = !op.enabled;
                    }
                });
            }
            // O MODO é a LEI do degrau, não a intensidade dele — e um clique num modo
            // que o TIPO não oferece é recusado aqui (o painel não o pinta, mas a
            // recusa mora onde o valor é escrito).
            FilterHit::Mode(row, mode) => {
                crate::fx_live::edit(sim, map, sel, |f| {
                    if let Some(op) = f.ops_mut().get_mut(row)
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
                crate::fx_live::edit(sim, map, sel, |f| {
                    if let Some(op) = f.ops_mut().get_mut(row)
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
                crate::fx_live::edit(sim, map, sel, |f| {
                    if let Some(op) = f.ops_mut().get_mut(row) {
                        crate::fx_live::add_stop(op);
                    }
                });
            }
            FilterHit::StopRemove(row) => {
                let sel_stop = usize::from(ph2d_panel_vector::selected_stop(row));
                crate::fx_live::edit(sim, map, sel, |f| {
                    if let Some(op) = f.ops_mut().get_mut(row) {
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
}
