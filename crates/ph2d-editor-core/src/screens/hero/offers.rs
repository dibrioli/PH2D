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
    #[must_use]
    pub fn rail_shows_painter_tools(&self) -> bool {
        self.image_edit.mode_on && self.image_edit.active_tool_id == Some("painter")
    }

    /// **As réguas estão vivas neste frame?** — a PORTA ÚNICA da W6.2, perguntada pelo paint
    /// (para desenhar as faixas) e pelo gesto (para decidir se um press nelas cria uma guia).
    ///
    /// São DUAS condições e nenhuma basta sozinha:
    /// - o interruptor do artista (`view.rulers_visible`), que é também o *lock* das guias;
    /// - **a ferramenta vetorial estar em mãos.**
    ///
    /// ⚠️ **A segunda condição é uma CORREÇÃO, não uma restrição de escopo.** A faixa da régua
    /// **ocupa** a borda do canvas (o modelo de sobreposição), e o gesto dela corre antes de
    /// toda ferramenta — então uma régua permanente comeria o pen-down do PAINTER nos 20 px de
    /// cima: o artista pincelaria ali e nasceria uma guia. Hoje quem consome guias é só o snap
    /// vetorial, então uma faixa presente noutra ferramenta seria custo sem contrapartida.
    ///
    /// ⚠️ E ela **preserva o invariante que importa**: *visível ⇔ vivo*. Uma faixa que
    /// aparecesse sem responder — ou que respondesse sem aparecer — é a forma exata do chrome
    /// morto sob o mouse que esta codebase varre a cada wave.
    ///
    /// O dia em que o gizmo de sprite consumir guias, esta função é o único lugar a mudar.
    #[must_use]
    pub fn rulers_live(&self) -> bool {
        self.view.rulers_visible && self.is_panel_visible("vector")
    }
}
