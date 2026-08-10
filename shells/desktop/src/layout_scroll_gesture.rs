//! **A roda que rola uma moldura** — a metade de GESTO da rolagem (o item 3 do estudo dos
//! contêineres). O motor é o [`crate::layout_live::scroll`].
//!
//! # Por que este ficheiro existe separado do handler da roda
//!
//! Pela mesma lei que o `sculpt3d_wheel` escreve no dele: **quem decide de quem é o gesto é o
//! módulo do assunto, não o roteador**. O `on_mouse_wheel` já tem quatro inquilinos (painel ·
//! escultura · o alcance do Gap Closure · o zoom), e a quinta pergunta escrita lá dentro seria a
//! quinta cópia de *"e se…"* num sítio onde ninguém consegue ler a ordem.
//!
//! # A régua: 16 px de tela por linha de roda, convertidos para MUNDO
//!
//! ⚠️ **Converter é obrigatório, e é o que faz a rolagem parecer certa em qualquer zoom.** Um
//! número de mundo fixo faria a lista voar com a câmera afastada e mal se mexer de perto; o que o
//! artista espera é que a roda ande *o mesmo tanto na tela*. A conversão sai da MESMA câmera que
//! desenha (`screen_to_world` de dois pontos), e não de uma segunda régua.

use ph2d_ecs::Entity;

impl crate::App {
    /// Rola a moldura sob o cursor. Devolve `true` se consumiu a roda — e o chamador conta com
    /// isso para deixar o zoom acontecer quando não há nada a rolar.
    pub(crate) fn wheel_scrolls_a_frame(&mut self, dx: f32, dy: f32) -> bool {
        let Some(gfx) = self.gfx.as_ref() else {
            return false;
        };
        // ⚠️ **Só com a ferramenta VECTOR na mão.** A moldura é do documento vetorial, e a roda é o
        // zoom de todo o resto do app — tomá-la enquanto o artista pinta seria roubar o gesto num
        // módulo que nem sabe o que é uma moldura. É a mesma cerca que a régua das guias carrega.
        if !self.vector_tool_active() {
            return false;
        }
        let size = gfx.surface.size();
        let p = gfx.camera.screen_to_world(self.last_pointer, size);
        let Some(frame) = self
            .layout_live
            .scrollable_frame_at([f64::from(p[0]), f64::from(p[1])])
        else {
            return false;
        };
        let d = self.wheel_delta_world(frame, dx, dy);
        if !self.layout_live.scroll_by(frame, d) {
            // ⚠️ **Já no fim da lista, a roda continua a ser CONSUMIDA.** Deixá-la passar faria a
            // câmera dar um salto de zoom no instante em que a lista acaba — o pior momento
            // possível, porque é quando o artista ainda está a girar a roda.
            return true;
        }
        self.any_input_this_frame = true;
        true
    }

    /// O deslocamento em unidades de MUNDO, no eixo que de facto transborda.
    ///
    /// ⚠️ **O eixo é derivado do excedente, e não da direção do fluxo.** São perguntas diferentes:
    /// um `RowWrap` flui em X e transborda em Y (as linhas empilham), e escolher pelo fluxo faria a
    /// roda empurrar a lista contra uma parede. Quando os dois transbordam vale o universal —
    /// a roda anda em Y, `Shift` anda em X.
    fn wheel_delta_world(&self, frame: Entity, dx: f32, dy: f32) -> [f64; 2] {
        let over = self.layout_live.overflow_of(frame);
        let (Some(gfx), shift) = (self.gfx.as_ref(), self.modifiers.shift_key()) else {
            return [0.0; 2];
        };
        // Quantas unidades de mundo mede um pixel de tela AGORA — a régua sai da própria câmera.
        let size = gfx.surface.size();
        let a = gfx.camera.screen_to_world((0.0, 0.0), size);
        let b = gfx.camera.screen_to_world((0.0, 1.0), size);
        let per_px = f64::from((b[1] - a[1]).abs()).max(f64::EPSILON);
        // ⚠️ O sinal: roda para CIMA (dy > 0) mostra o conteúdo de CIMA, ou seja diminui o
        // deslocamento. O eixo do deslocamento é o do motor (`y` para baixo).
        let step = -f64::from(dy) * per_px;
        let horizontal = shift || (over[1] <= 0.0 && over[0] > 0.0);
        if horizontal {
            // `dx` existe em trackpads; num rato ele é zero e o `dy` é que anda, com Shift.
            let h = if dx.abs() > f32::EPSILON {
                -f64::from(dx) * per_px
            } else {
                step
            };
            [h, 0.0]
        } else {
            [0.0, step]
        }
    }
}

#[cfg(test)]
#[path = "layout_scroll_gesture_tests.rs"]
mod tests;
