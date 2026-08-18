//! **O GESTO DO FILTRO** — o verbo corrente na malha INTEIRA, com o arrasto
//! horizontal a dar a força.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados; o corte é o
//! do [`super::transform`], o vizinho de que este módulo é a cópia estrutural: a
//! LEI mora no kernel ([`ph2d_sculpt3d::SculptStroke::filter`]) e o que mora
//! aqui é *o que a mão na tela quer dizer*.
//!
//! # O gesto é o da referência, e o número também
//!
//! O `sculpt_filter_mesh.cc:2299` mede
//! `len = prev_press_mouse[0] - mouse[0]` e depois
//! `strength = start_strength * -len * 0.001`, ou seja **o arrasto para a
//! DIREITA é positivo** e a régua é `0,001` por pixel. Os dois vêm de lá; o que
//! **não** vem é o `start_strength`, e a divergência está declarada no doc do
//! driver: aqui a força é o arrasto e nada mais.
//!
//! # Por que não há `close_filter`
//!
//! Porque o [`super::Sculpt3dScene::close_stroke`] **já é** a porta. O
//! [`ph2d_sculpt3d::SculptStroke::filter_begin`] preenche exactamente os dois
//! arrays que o fecho de um traço grava (`touched` e `base_positions`), então
//! uma porta própria seria a segunda resposta a *"como se desfaz um punhado de
//! vértices deslocados"* — o mesmo argumento que fez o transform reusar a
//! [`super::StrokeUndo::Stroke`] em vez de inventar um variant.
//!
//! ⚠️ E o braço da MÁSCARA do `close_stroke` é inerte aqui por construção: ele
//! pergunta `verb.paints_mask()`, e nenhum verbo que filtra pinta máscara.

use super::{FILTER_DRAG_PER_PX, Sculpt3dScene};

impl Sculpt3dScene {
    /// **O filtro está armado E vivo?**
    ///
    /// ⚠️ **A pergunta inclui o VERBO, e é o que mantém *visível ⇔ vivo*.** O
    /// painel só oferece o interruptor a um verbo que filtra, então um arm que
    /// sobrevivesse a `1` (pegar o Draw) ficaria **aceso e invisível**: o botão
    /// esquerdo pararia de esculpir sem nada na tela dizendo por quê. Perguntar
    /// as duas metades numa porta só é o que impede o painel e o gesto de
    /// discordarem — e é por isso que o flag **não** é apagado na troca de
    /// verbo: voltar ao Smooth devolve a escolha que o artista fez.
    pub(crate) fn filter_arm(&self) -> bool {
        self.filter_arm && self.brush.verb.filters_mesh()
    }

    /// **Arma ou desarma** — clicar o aceso desliga, a lei de todo toggle deste
    /// app.
    ///
    /// ⚠️ **Ele DESARMA o transform**, e a exclusão mora nas duas portas de
    /// armar em vez de num `enum` de modo: os dois reivindicam o MESMO botão
    /// esquerdo, e um gesto que significasse as duas coisas não teria como
    /// escolher. Um `enum` seria a resposta mais limpa e obrigaria a reescrever
    /// os cinco leitores do `transform_arm`; a exclusão por porta custa duas
    /// linhas e tem gate.
    pub(crate) fn arm_filter(&mut self) -> bool {
        self.filter_arm = !self.filter_arm;
        if self.filter_arm {
            self.transform_arm = None;
        }
        self.filter_arm
    }

    /// **Congela a foto e abre o gesto.** `false` quando o verbo em mãos não
    /// filtra — e a recusa é REPORTADA pelo chamador, como a do transform: um
    /// gesto que não faz nada e não diz nada é indistinguível de um botão que
    /// não chegou.
    pub(super) fn begin_filter(&mut self, x: f32) -> bool {
        if !self.filter_arm() {
            return false;
        }
        let mesh = self.mesh().clone();
        self.stroke.filter_begin(&mesh);
        self.filter_from_x = x;
        true
    }

    /// O dedo andou: re-roda a lei sobre a pose CONGELADA com a força de agora.
    ///
    /// ⚠️ **A força é o arrasto TOTAL desde o pen-down, nunca um incremento**, e
    /// é o que faz do gesto um passo só: o driver repõe a pose congelada a cada
    /// chamada, então voltar com o dedo desfaz — e um filtro que compusesse
    /// incrementos dependeria de quantos eventos o rato mandou, a lei que esta
    /// casa já pagou quatro vezes no relevo do Painter.
    pub(super) fn filter_at(&mut self, x: f32) {
        if !self.filter_arm() {
            return;
        }
        let amount = (x - self.filter_from_x) * FILTER_DRAG_PER_PX;
        // ⚠️ **O pincel é o AUTORADO, e não o [`Sculpt3dScene::armed_brush`]**:
        // aquele resolve o raio contra o ACERTO e carimba o estêncil da vista, e
        // um filtro não tem cursor nem pegada. O que a lei lê daqui é o verbo, a
        // referência e os dois `hc_*` — todos autorados —, então pedir um pincel
        // ancorado num ponto seria inventar o ponto que o gesto não tem.
        let brush = self.brush.clone();
        let mesh = self.objects[self.active].stack.mesh_mut();
        let moved = self.stroke.filter(mesh, &brush, amount);
        if moved == 0 {
            return;
        }
        let touched = self.stroke.touched().to_vec();
        Self::mesh_changed(
            &mut self.objects[self.active].dirty,
            &mut self.edits,
            &touched,
        );
    }
}

#[cfg(test)]
#[path = "sculpt3d_filter_tests.rs"]
mod tests;
