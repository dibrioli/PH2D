//! **O que esta tela OFERECE agora** — as portas únicas de *«esta superfície está viva?»*.
//!
//! Irmão do `hero.rs` pela mesma linha de corte do [`live`](super::live): aquele diz o que uma tela
//! **É** (os campos, os painéis, a selecção) e o que ela **FAZ** por quadro; isto diz o que ela
//! **OFERECE** — perguntas de SIM ou NÃO que decidem se um chrome existe neste instante.
//!
//! # Por que as duas moram juntas, e por que são portas
//!
//! As duas nasceram do mesmo defeito, com dois anos de distância: **uma condição composta perguntada
//! por dois consumidores em cópias separadas**. Cada uma tem DUAS metades e nenhuma basta sozinha, e
//! é exactamente aí que uma segunda cópia diverge — o dia em que a condição ganha um terceiro termo,
//! um dos leitores fica para trás e o app passa a **desenhar o que não responde**, ou a responder
//! onde não desenha. Este arquivo é o sítio onde esse terceiro termo se escreve **uma vez**.

use super::HeroScreen;

impl HeroScreen {
    /// **A coluna mostra as ferramentas de PINTURA?** — a porta única da pergunta.
    ///
    /// São duas condições e nenhuma basta sozinha: o modo Image-Tools ligado **e** o Painter em
    /// mãos. Ela existe porque a pergunta ganhou um segundo consumidor — o `paint` (para desenhar o
    /// rail) e a [`global_palette`](super::global_palette) (para oferecer os mesmos comandos) — e
    /// duas cópias divergiriam no dia em que a condição ganhasse um terceiro termo, com a paleta a
    /// oferecer ferramentas que a coluna não mostra.
    ///
    /// ⭐ **Desde a F2 do spec/06 o Painter entra pelo MODO Paint da imagem**, sem o IMG: o 3.º
    /// termo é `ou o modo Paint`. (Com o IMG ligado ele continua a chegar pela aba do Painter.)
    #[must_use]
    pub fn rail_shows_painter_tools(&self) -> bool {
        self.image_edit.active_tool_id == Some("painter")
            && (self.image_edit.mode_on
                || self.gizmo.mode.current() == crate::object_mode::ObjectMode::Paint)
    }

    /// **As réguas estão vivas neste frame?** — a PORTA ÚNICA da W6.2, perguntada pelo paint
    /// (para desenhar as faixas) e pelo gesto (para decidir se um press nelas cria uma guia).
    ///
    /// **Duas condições, numa porta só: o interruptor do artista** (`view.rulers_visible`, que é
    /// também o *lock* das guias) **e a aba activa ser a CENA** — um quadro (MiroClone) não tem
    /// réguas nem guias. ⛔ Report do dono (07/10, *«o Select é inseleccionável; toda a barra tem
    /// problemas»*): a 2.ª condição entrou em 05/10 só no PAINT; o gesto da shell continuava a ver
    /// faixas e guias INVISÍVEIS sobre a área do quadro — e a barra de ferramentas dele mora
    /// encostada a essa borda: um clique no botão criava (ou agarrava) uma guia e nunca chegava ao
    /// botão. O portão que devia apanhá-lo só conferia que as duas metades CITAVAM esta porta.
    ///
    /// ⛔⛔ **Havia uma segunda — «a ferramenta vetorial em mãos» — e ela CAIU em 2026-08-30, por
    /// ordem do Enio:** *«as réguas devem funcionar em todos os modos e layouts, e não apenas
    /// para vector».*
    ///
    /// ⭐ **A cerca era legítima e o substrato dela dissolveu-se no mesmo dia.** Ela existia por
    /// duas razões, e as duas deixaram de valer:
    ///
    /// 1. *«a faixa OCUPA a borda do canvas e o gesto corre antes de toda ferramenta ⇒ uma régua
    ///    permanente comeria o pen-down do Painter nos 20 px de cima»*. O defeito real ali era a
    ///    faixa ser **invisível** — ela nascia debaixo do trilho e da barra, e o artista carregava
    ///    no que parecia um botão e recebia uma guia. Desde que a régua é uma **região da área de
    ///    desenho** ([`crate::screens::layout::HeroLayout::draw_area`]) a faixa está **à vista**,
    ///    e carregar numa régua visível para criar uma guia é o comportamento de todo DCC. ⇒ o que
    ///    sobra não é um roubo, é a régua a fazer o que uma régua faz.
    /// 2. *«só o snap vetorial consome guias, logo a faixa noutra ferramenta é custo sem
    ///    contrapartida»* — juízo de **produto**, e o dono do produto decidiu ao contrário.
    ///
    /// ⚠️ **O preço que fica, nomeado:** com as réguas ligadas, os 20 px de cima e da esquerda da
    /// área de desenho deixam de ser pintáveis em **qualquer** ferramenta. É o mesmo preço que o
    /// Photoshop e o Blender cobram, a área é pannable, e o interruptor desliga-o.
    ///
    /// ⚠️ E o invariante que importa continua: *visível ⇔ vivo* — as duas metades perguntam a
    /// **esta** função, e a faixa em si tem a porta [`crate::ruler::live_bands`].
    #[must_use]
    pub fn rulers_live(&self) -> bool {
        self.view.rulers_visible && self.documents.active().is_none()
    }
}
