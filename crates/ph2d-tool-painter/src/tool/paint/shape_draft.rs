//! **O meio caro renderiza em REPOUSO** — pedido do Enio de 2026-08-07: *"ligar o pigmento apenas no
//! mouse up quando o usuário estiver em repouso; se o usuário mover a shape a pintura some e só volta
//! no mouse up"*.
//!
//! ## O número que abriu a frente
//!
//! Um move de figura viva, 4096², elipse de 400 px, pincel r=96
//! (`measure_shape_system::measure_shape_system_by_medium`):
//!
//! ```text
//! meio        move (ms)   do qual: carimbo
//! Digital          4,52   4,01  (89%)
//! Impasto         99,10   98,38 (99,3%)
//! Aquarela        74,27   -
//! ```
//!
//! ⚠️ **A máquina de shape custa 0,25 ms** — geometria, restore, save. Quem cobra é o CARIMBO, e ele
//! não é ineficiente: ele é **repetido**, porque o shape editor re-carimba a figura INTEIRA a cada
//! quadro do arrasto (`stamp_drag_preview` restaura a região salva e re-lança o lote todo).
//!
//! ## A escolha do desenho, medida em vez de opinada
//!
//! Duas variantes respondem *"o que o artista vê durante o arrasto?"*
//! (`measure_what_a_flat_preview_would_cost`):
//!
//! ```text
//! meio        caro    RASCUNHO   só o guia
//! Impasto   101,17      4,38       0,39
//! Aquarela   73,78      4,38       0,32
//! ```
//!
//! **Apagar** deixa só o guia amarelo (perímetro + alças, que a shell desenha no vector scene, fora do
//! carimbo) e custa 0,39 ms. **O rascunho** custa 4,38 — 26% de um quadro de 60 fps — e mantém sob a
//! mão a COR, a **largura real do traço** (o pincel passa do guia), a textura do pincel e **o resultado
//! do booleano**, nenhum dos quais o guia mostra. Ajustar uma figura é ajustar até *ficar bom*: apagar
//! o que se julga durante o gesto tira justamente a informação do julgamento, por 4 ms.
//!
//! ⚠️ **E não é desenho novo: é o padrão que o doc 21 já shipa no Wet Paint** (autoria com preview
//! flat estático, o meio caro entrando num commit), smokado e aprovado. O que muda aqui é a
//! FRONTEIRA — lá o caro entra no *commit* (Enter/Apply), aqui entra **em repouso** (o Up), então
//! ajustar → soltar → olhar → ajustar funciona sem esperar o Apply.
//!
//! ## O preço, nomeado
//!
//! Sob Impasto o **relevo** some durante o arrasto (forma, cor e largura ficam); sob Aquarela, o
//! sangramento. É o mesmo trade que o Wet Paint já faz, e o pisca ao pegar de novo é real — o doc 21
//! o chama de *"o esboço derrete"*.

use super::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

impl PainterTool {
    /// **Este lote renderiza um RASCUNHO do meio caro?**
    ///
    /// A porta única que Impasto e Aquarela perguntam. Ela responde sobre o LOTE, nunca sobre o meio
    /// estar armado — ⚠️ e essa distinção é load-bearing: o painel lê *"a aquarela está armada?"* para
    /// esconder a row **Accumulate**, e se ele lesse esta as rows apareceriam e sumiriam a cada
    /// arrasto. Duas perguntas, dois predicados (ver `watercolor_armed`).
    pub(super) fn draft_stamp(&self) -> bool {
        self.paint.shape_draft
    }

    /// A rota dos editores de figura, embrulhada pela lei do repouso.
    ///
    /// ⚠️ **A bandeira cai ANTES da rota no Up**, não depois: assim o re-carimbo que o próprio editor
    /// faz ao soltar JÁ é o final, e o gesto paga **exatamente um** carimbo caro. Cair depois custaria
    /// dois (um rascunho e um final) pelo mesmo resultado.
    pub(super) fn route_shape_draft(&mut self, ev: CanvasPointer) -> bool {
        let up = ev.phase == PointerPhase::Up;
        self.paint.shape_draft = !up;
        let before = self.paint.restamp_seq;
        let used = self.route_shape_pointer_multi(ev);
        // ⚠️ O Up tem de deixar a figura CARA na tela, e nem todo ramo de Up de todo editor re-carimba
        // (um clique que não pegou nada, um ramo que sai cedo). Perguntar ao FATO — *alguém
        // re-carimbou?* — em vez de enumerar os ramos é o que impede a figura de ficar plana para
        // sempre no dia em que um deles ganhar um early-return.
        if up && self.paint.restamp_seq == before {
            self.refill_live_shape();
        }
        used
    }

    /// Re-carimba a figura VIVA, seja ela qual for — a porta agnóstica de editor.
    ///
    /// Espelha a cascata que o `restore_model` já fazia à mão (`shape_snapshot`), agora com um dono só:
    /// um editor novo entra aqui e os dois chamadores o ganham juntos.
    pub(super) fn refill_live_shape(&mut self) {
        if self.paint.curve.is_some() {
            self.curve_refill();
        } else if self.paint.ellipse.is_some() {
            self.ellipse_refill();
        } else if self.paint.polygon.is_some() {
            self.polygon_refill();
        } else if self.paint.line.is_some() {
            self.line_refill();
        } else if self.has_parked_shapes() {
            // Sem editor ativo as figuras estacionadas ainda são a imagem — elas também saíram
            // rascunhadas durante o gesto.
            self.restamp_shapes_preview(&[]);
        }
    }
}
