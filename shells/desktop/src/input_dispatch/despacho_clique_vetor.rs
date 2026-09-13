//! **O clique, a FERRAMENTA VETORIAL** — ramos do `on_mouse_input` ([`super`]): o guarda do ADR-0112 (Node e os
//! modos de desenho capturam o canvas; o Select não), o blur do campo de texto, o picker que se fecha, e os
//! braços do `match` — o Shift que alterna ou abre a região, e o direito que aborta. Os corpos MUDARAM-SE
//! verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da chamada, pela mesma ordem.
//!
//! ⚠️ Os braços que SEMPRE devolvem (o Shift e o direito) são ramos SEM sinal: o `return;` final ficou no
//! braço que os chama, e um `return;` de dentro sai do ramo para cair exactamente nesse `return;`.

use super::*;

impl crate::App {
    /// O botão direito no canvas: aborta o Picker, o conector ou a alça dele, apaga a parada do Width, cancela
    /// o lápis, e fecha a caneta ou cancela a forma.
    pub(super) fn ramo_vetor_direito_premido(&mut self) {
        // O botão direito ABORTA o gesto em curso — a mesma tecla de fuga que já vale para
        // a caneta, a forma e o conector. Um Picker armado é um gesto: o direito desiste
        // dele (o clique esquerdo no vazio também, mas o direito é o "cancela" universal).
        if self.vec.path_pick.take().is_some() {
            return;
        }
        // O botão direito ABORTA o conector em construção (a linha some) — o
        // mesmo que ele já faz com a caneta e a ferramenta de forma. E abortar o
        // arrasto de uma ALÇA devolve a ponta ao lugar de onde ela saiu (o
        // vínculo original, intacto): desistir não pode desligar a linha.
        if self.conn_handle_cancel() || self.connector_cancel() {
            return;
        }
        // **O Width Tool**: o direito APAGA a parada sob o cursor — o verbo de
        // remoção da ferramenta, não um cancelamento (não há gesto em curso a
        // abortar; um clique é um clique). Abaixo de duas paradas o perfil inteiro
        // sai e o traço volta ao uniforme, que é o neutro-é-ausência das outras
        // rotas. O passo de undo é o da fila global, por diff.
        if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Width
            && let Some(pid) = self.vec.pen.selected()
            && let Some(world) = self.vec_world_at(self.last_pointer)
        {
            let hit_r = HANDLE_HIT_PX * self.vec_px_to_world();
            if let Some(gfx) = self.gfx.as_mut() {
                let scene = &gfx.vec_scene;
                crate::width_handles::remove(
                    &mut gfx.sim,
                    scene,
                    &self.vec.entities,
                    pid,
                    world,
                    hit_r,
                );
            }
            return;
        }
        // **O lápis** desiste pelo direito também: o traço vivo some sem deixar
        // rastro e o passo de undo pendente é cancelado.
        if self.vec.pencil.is_active() {
            if let Some(gfx) = self.gfx.as_mut() {
                self.vec.pencil.cancel(&mut gfx.vec_scene);
            }
            return;
        }
        if shape_kind_for_mode(&self.vec.draw_config).is_none() {
            self.vec.pen.finish();
        } else {
            if let Some(gfx) = self.gfx.as_mut() {
                self.vec.shape.cancel(&mut gfx.vec_scene);
            }
        }
    }

    /// O Shift no canvas: alterna o PONTO do Node sob o cursor, ou o OBJECTO (o grupo inteiro), ou abre a região.
    pub(super) fn ramo_vetor_shift_premido(&mut self) {
        // **Modo Node: Shift+clique num PONTO alterna-o na multi-seleção de pontos**
        // (Enio 2026-07-15). Tentado ANTES do toggle de OBJETO: no Node é no ponto que
        // se mexe, e o Shift sobre a forma (que cobre o ponto) alternava o objeto —
        // somar pontos a dedo era impossível (só o retângulo). Mesmo raio do grab do
        // Node (`10 px`), então o que se agarra é o que se alterna. Sem ponto sob o
        // cursor, cai no comportamento de sempre (objeto / marquee).
        if self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Node
            && let Some(wp) = self.vec_world_at(self.last_pointer)
        {
            let hit_r = 10.0 * self.vec_px_to_world();
            // `gfx.vec_scene` e `vec_pen` são campos DISJUNTOS de `self`.
            if let Some(gfx) = self.gfx.as_ref()
                && self.vec.pen.toggle_vert_at(&gfx.vec_scene, wp, hit_r)
            {
                return;
            }
        }
        let hit = self.gfx.as_ref().and_then(|gfx| {
            let win = gfx.surface.size();
            let w = gfx.camera.screen_to_world(self.last_pointer, win);
            let w0 = gfx.camera.screen_to_world((0.0, 0.0), win);
            let w1 = gfx.camera.screen_to_world((1.0, 0.0), win);
            let px = (((w1[0] - w0[0]).powi(2) + (w1[1] - w0[1]).powi(2)).sqrt()) as f64;
            self.vec
                .pen
                .path_at(&gfx.vec_scene, [w[0] as f64, w[1] as f64], 10.0 * px)
        });
        if let Some(id) = hit {
            // Um grupo entra e sai da seleção INTEIRO (a árvore é a
            // Hierarquia — o ancestral de topo diz quem vem junto).
            let members = self.vec_object_selection_for(id);
            self.vec.pen.toggle_object_members(&members);
            // Object selection changed → drop any gradient-handle selection.
            self.vec.grad_selected = None;
            self.vec.grad_drag = None;
            return;
        }
        self.vec.marquee = Some(crate::vec_marquee::VecMarquee::open(
            self.marquee_shape_for_press(),
            self.last_pointer,
        ));
    }

    /// A ferramenta vetorial no canvas (ADR-0112: só Node e os modos de desenho, nunca o Select): o blur do campo de
    /// texto, o picker que se fecha, e os braços do `match` — Shift, premir, soltar e o direito.
    pub(super) fn ramo_ferramenta_vetorial(
        &mut self,
        mapped_button: ph2d_host::PointerButton,
        kind: PointerKind,
        on_canvas: bool,
        evt: PointerEvent,
        menu_open_before: bool,
    ) -> bool {
        // ADR-0112: no modo **Select** a ferramenta não captura o canvas — o clique
        // cai no caminho de sempre (picking de sprite + gizmo), e é assim que uma
        // forma vetorial se transforma. Só Node e os modos de desenho entram aqui.
        if self.vector_tool_active()
            && self.vec.draw_config.mode != ph2d_tool_vector::DrawMode::Select
            && !menu_open_before
        {
            // A canvas press while a text field is focused must blur it (commit the
            // edit) — the pen/shape arms below consume the press and bypass the
            // chrome dispatch that normally does this. Route it explicitly (only a
            // primary press with a field actually focused, so normal draw clicks
            // don't churn dispatch and a right-click can't open a menu here).
            if mapped_button == ph2d_host::PointerButton::Primary
                && kind == PointerKind::Down
                && on_canvas
                && self.text_entry_focused()
            {
                let _ = forward_to_hero(self.gfx.as_mut(), evt);
            }
            // A canvas press dismisses an open colour picker (click-outside closes
            // it, mirroring the chrome light-dismiss). `on_canvas` already excludes
            // the picker rect, so any press reaching here is genuinely outside it —
            // done BEFORE the grad/pen/shape arms so the picker's colour is never
            // applied to the handle the press then selects (Enio 2026-07-08).
            if mapped_button == ph2d_host::PointerButton::Primary
                && kind == PointerKind::Down
                && on_canvas
                && let Some(gfx) = self.gfx.as_mut()
                && let Some(hero) = gfx.hero_screen.as_mut()
                && hero.store.picker_target().is_some()
            {
                hero.store.set_picker_target(None);
            }
            match (mapped_button, kind) {
                // Shift+Down on a PATH → toggle it in the object multi-selection
                // (Align/Distribute); Shift+Down on empty canvas → vertex marquee.
                // Tried first so Shift diverts the press from the pen/shape draw.
                (ph2d_host::PointerButton::Primary, PointerKind::Down)
                    if on_canvas && self.modifiers.shift_key() =>
                {
                    self.ramo_vetor_shift_premido();
                    return true;
                }
                (ph2d_host::PointerButton::Primary, PointerKind::Down) if on_canvas => {
                    if self.ramo_vetor_premido_modos() {
                        return true;
                    }
                    if self.ramo_vetor_premido_corte_balde_osso() {
                        return true;
                    }
                    if self.ramo_vetor_premido_quina() {
                        return true;
                    }
                    if self.ramo_vetor_premido_caneta() {
                        return true;
                    }
                }
                (ph2d_host::PointerButton::Primary, PointerKind::Up) => {
                    if self.ramo_vetor_solto_osso() {
                        return true;
                    }
                    if self.ramo_vetor_solto_gestos() {
                        return true;
                    }
                    if self.ramo_vetor_solto_caneta() {
                        return true;
                    }
                }
                (ph2d_host::PointerButton::Secondary, PointerKind::Down) if on_canvas => {
                    self.ramo_vetor_direito_premido();
                    return true;
                }
                _ => {}
            }
        }
        false
    }
}
