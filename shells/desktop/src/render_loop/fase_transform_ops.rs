//! **Fase do quadro: AS OPERAÇÕES DE TRANSFORMAÇÃO** — o afim de cada caminho, a mistura e a opacidade por objecto, os campos X/Y/W/H, o nó pela
//! porta das setas, o preset da moldura e rodar (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct TransformOpsIntents {
    pub(super) pending_frame_preset: Option<ph2d_tool_vector::frames::DevicePreset>,
    pub(super) pending_vec_opacity: Option<f64>,
    pub(super) pending_vec_blend: Option<u8>,
    pub(super) pending_paint_verb: Option<crate::vec_paint_stack::StackVerb>,
    pub(super) pending_paint_width: Option<f64>,
    pub(super) pending_paint_dx: Option<f64>,
    pub(super) pending_paint_dy: Option<f64>,
    pub(super) pending_paint_dilate: Option<f64>,
    pub(super) pending_paint_join: Option<u8>,
    pub(super) pending_paint_opacity: Option<f64>,
    pub(super) pending_paint_blend: Option<u8>,
    pub(super) pending_vec_transform: Option<(crate::input_dispatch::VecTransformField, f64)>,
    pub(super) pending_vec_vert: Option<(bool, f64)>,
    pub(super) pending_vec_rotate_by: Option<f64>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_transform_ops(
        &mut self,
        intents: TransformOpsIntents,
    ) -> Option<ph2d_vec_scene::VecXforms> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let TransformOpsIntents {
            pending_frame_preset,
            pending_vec_opacity,
            pending_vec_blend,
            pending_paint_verb,
            pending_paint_width,
            pending_paint_dx,
            pending_paint_dy,
            pending_paint_dilate,
            pending_paint_join,
            pending_paint_opacity,
            pending_paint_blend,
            pending_vec_transform,
            pending_vec_vert,
            pending_vec_rotate_by,
        } = intents;
        // O afim de cada path, para as operações que falam MUNDO (align, distribute,
        // campos X/Y/W/H). O mapa é o do frame passado — os paths envolvidos já
        // existem, então basta.
        let vec_xf_ops = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
        // ⚠️ **A VOLTA da fronteira de display, e ela mora AQUI e não dentro do
        // `apply_vec_transform`.** O `target` é o número que o artista DIGITOU, logo está na
        // unidade dele; a operação fala mundo. Converter dentro dela quebraria o outro
        // chamador logo abaixo — o preset de dispositivo devolve **unidades de DOCUMENTO**
        // (`DevicePreset::size`, o aspecto do aparelho normalizado ao `LONG_SIDE`), que é dado
        // AUTORADO e não um número da face do artista: ele tem de atravessar intocado.
        // ⭐⭐⭐ **A APARÊNCIA do objecto entra no DOCUMENTO** (estudo 42 item 2).
        //
        // ⚠️ **Escreve na selecção INTEIRA e lê do primário** — a lei desta janela, e a porta é
        // uma só (`vec_appearance`). ⚠️ E o passo de undo sai de graça: o registo é por DIFF, e
        // um arrasto de slider não regista por quadro (um gesto em curso suprime a captura),
        // então a corrida inteira colapsa em UM passo ao soltar.
        {
            let sel = self.vec.pen.selected_paths().to_vec();
            if let Some(track) = pending_vec_opacity {
                #[allow(clippy::cast_possible_truncation)]
                crate::vec_appearance::set_opacity(vec_scene, &sel, track as f32);
            }
            if let Some(code) = pending_vec_blend {
                crate::vec_appearance::set_blend(vec_scene, &sel, code);
            }
            // ⭐⭐⭐ **E A PILHA** (item 4). ⚠️ O verbo primeiro, as propriedades depois: um
            // clique em «apagar» e um arrasto de slider não chegam no mesmo frame, mas se
            // chegassem, escrever numa camada que o verbo acabou de remover seria um `None`
            // silencioso — e a ordem torna isso impossível de acontecer ao contrário.
            if let Some(v) = pending_paint_verb {
                crate::vec_paint_stack::apply(vec_scene, &sel, v);
            }
            // ⚠️ **O índice é o da camada ABERTA no painel** — a única que mostra estes três
            // controlos. Sem camada aberta não há sujeito, e escrever seria adivinhar.
            // ⭐⭐⭐ **A COR de uma camada** — o picker partilhado escreve nela.
            //
            // ⛔ Sem este braço a swatch de uma camada abre o picker, o artista escolhe uma
            // cor e **nada acontece** — o `clippy` apanhou-o como `set_color` never used, que
            // é a assinatura do controlo morto que o §5.0 nomeia.
            if let Some(i) = crate::vec_paint_stack::layer_of_picker_target(&hero.store)
                && let Some((value, _, _, _)) = hero
                    .store
                    .blender_picker(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
            {
                // ⚠️ O picker já entrega bytes (`rgba`), como as swatches de base — converter
                // aqui seria a segunda régua de *"que cor é esta?"*.
                let c = value.rgba;
                crate::vec_paint_stack::set_color(
                    vec_scene,
                    &sel,
                    i,
                    ph2d_vec_scene::Rgba8::new(c[0], c[1], c[2], c[3]),
                );
            }
            if let Some(i) = ph2d_panel_vector::state::open_layer_index() {
                if let Some(w) = pending_paint_width {
                    crate::vec_paint_stack::set_width(vec_scene, &sel, i, w);
                }
                // ⭐ ONDE ela desenha (v21). ⚠️ O eixo que NAO comitou le-se do documento, e
                // nao de um default: escrever `0` no gemeo apagaria o valor que o artista
                // acabou de por na outra caixa.
                if pending_paint_dx.is_some() || pending_paint_dy.is_some() {
                    // ⛔ `sel.first()`, nunca `sel[0]`: a camada aberta é estado de VISTA e
                    // sobrevive a um quadro em que a selecção esvaziou.
                    let atual = sel
                        .first()
                        .and_then(|id| vec_scene.path(*id))
                        .and_then(|p| p.paints.get(i).map(|e| e.offset))
                        .unwrap_or([0.0, 0.0]);
                    let novo = [
                        pending_paint_dx.unwrap_or(atual[0]),
                        pending_paint_dy.unwrap_or(atual[1]),
                    ];
                    crate::vec_paint_stack::set_offset(vec_scene, &sel, i, novo);
                }
                if let Some(t) = pending_paint_opacity {
                    #[allow(clippy::cast_possible_truncation)]
                    crate::vec_paint_stack::set_opacity(vec_scene, &sel, i, t as f32);
                }
                if let Some(code) = pending_paint_blend {
                    crate::vec_paint_stack::set_blend(vec_scene, &sel, i, code);
                }
                if let Some(d) = pending_paint_dilate {
                    crate::vec_paint_stack::set_dilate(vec_scene, &sel, i, d);
                }
                if let Some(j) = pending_paint_join {
                    crate::vec_paint_stack::set_dilate_join(vec_scene, &sel, i, j);
                }
            }
        }
        if let Some((field, target)) = pending_vec_transform {
            let target = ph2d_editor_core::LengthDisplay::of(&hero.project).to_world(target);
            crate::input_dispatch::apply_vec_transform(
                sim,
                &self.vec.entities,
                vec_scene,
                &self.vec.pen,
                &vec_xf_ops,
                field,
                target,
            );
        }
        // **O NÓ ANDA PELA PORTA DAS SETAS.**
        //
        // ⚠️ O que o dreno aplica é um **DESLOCAMENTO**, nunca uma posição: `PenTool::nudge`
        // é a porta que o teclado já usa, e ela move âncora **e handles** e converte
        // mundo→local **por FORMA** (`delta_to_local`) — duas formas de escalas diferentes
        // andariam distâncias diferentes sob uma conversão só. Um `set_vertex_position` seria
        // a segunda resposta a *"como um nó se move?"*, e as duas divergiriam no dia em que
        // uma delas ganhasse um caso especial (o `nudge` já tem um).
        //
        // ⚠️ E o número digitado atravessa a MESMA fronteira de display do Transform: ele sai
        // da face do artista e volta pela mesma porta.
        if let Some((is_y, target)) = pending_vec_vert {
            let target = ph2d_editor_core::LengthDisplay::of(&hero.project).to_world(target);
            if let Some(now) = self.vec.pen.selected_anchor_world(vec_scene) {
                let (dx, dy) = if is_y {
                    (0.0, target - now[1])
                } else {
                    (target - now[0], 0.0)
                };
                self.vec.pen.nudge(vec_scene, dx, dy);
            }
        }
        // **O preset de dispositivo da MOLDURA** (plano UI/UX W0) — dois números pela porta
        // que os campos W/H já usam. Um preset não é um caminho novo: é o mesmo pedido feito
        // de outra forma, e é por isso que ele herda o undo e o clamp de dimensão degenerada.
        if let Some(p) = pending_frame_preset {
            let (pw, ph) = p.size();
            for (field, target) in [
                (crate::input_dispatch::VecTransformField::W, pw),
                (crate::input_dispatch::VecTransformField::H, ph),
            ] {
                crate::input_dispatch::apply_vec_transform(
                    sim,
                    &self.vec.entities,
                    vec_scene,
                    &self.vec.pen,
                    &vec_xf_ops,
                    field,
                    target,
                );
            }
        }
        if let Some(deg) = pending_vec_rotate_by {
            crate::input_dispatch::apply_vec_rotate_by(vec_scene, &self.vec.pen, deg);
        }
        Some(vec_xf_ops)
    }
}
