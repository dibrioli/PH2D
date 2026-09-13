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
}
