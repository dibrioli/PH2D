//! **Sob a mão, o GIZMO é o preview** — decisão do Enio de 2026-08-07, depois do smoke:
//! *"mesmo o preview plano é extremamente custoso; num 4096, 4 círculos com boolean +, o fps cai para
//! 2 ou menos. Minha ideia é deixar só as linhas do gizmo."*
//!
//! Enquanto um gesto de figura está em voo a tinta **some** — o `drag_preview` é descascado e nenhum
//! carimbo roda. Ao soltar, um re-carimbo final devolve o desenho com o meio de verdade.
//!
//! ## A atribuição, e por que ela derrubou a primeira versão desta lei
//!
//! A v1 desarmava só o **MEIO CARO** (o corpo do Impasto, a lavagem da Aquarela) e media 16× num move
//! de figura única. O smoke reprovou, e a fixture do report explica: ela usava **UMA** elipse de 400 px
//! e a cena real tem **quatro** círculos de 900 px com Operation Add num 4096
//! (`measure_boolean_cost::measure_the_scene_the_report_describes`):
//!
//! ```text
//! formas  raio    EVENTO   geom (boolean)   carimbo
//!      1   120     90,79        79,16        11,64
//!      4   120    308,16       284,10 (92%)  24,06 (8%)
//!      4    40    289,23       280,85         8,38
//! ```
//!
//! ⚠️ **A tinta era 8% do problema.** Os 92% são o composite booleano — rasterizar as quatro figuras
//! num buffer supersampleado (142 ms) e traçar os contornos (124). E ele **quase não responde ao
//! pincel**: r=120 dá 308 ms, r=40 dá 289. Desarmar o meio caro levaria 308 para ~300.
//!
//! ⛔ **MEDIDO E REJEITADO — não refaça: rascunhar o composite em resolução menor.** `SS` de 3 para 1
//! leva a cena a **60 ms** (5×, ainda 16 fps) e **muda o desenho sob a mão** (o contorno cai de 30.884
//! para 10.291 pontos, visivelmente mais grosseiro). Não há ponto de operação nessa direção.
//!
//! ⚠️ **O composite é `O(área da união × SS²)`** — a wave de 06/08 já o havia levado ao piso do método
//! (*"a janela É a figura — irredutível para este método"*) e nomeou a saída: a **rota analítica** do
//! [doc 35 §4](../../../../docs/Painter/35_boolean_o_que_o_vector_ensina.md), o booleano sobre CURVAS
//! em vez de sobre pixels, que é `O(segmentos)` e não `O(área)`. Ela muda o desenho e é decisão do
//! Enio — e **é ela que tornaria o composite vivo sob a mão de novo**. Esta lei é o que torna a cena
//! usável enquanto aquela decisão não é tomada.
//!
//! ## O que o artista vê
//!
//! O guia amarelo da figura ativa (perímetro + alças) e os badges de Operation das parqueadas — tudo
//! isso a shell desenha no vector scene, **fora do carimbo**, e portanto de graça. A arte já commitada
//! fica; o que some é o preview das figuras abertas, que é literalmente *"a pintura some e só volta no
//! mouse up"*.

use super::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

impl PainterTool {
    /// **Um gesto de figura está em voo?** — a porta única que o re-carimbo pergunta.
    ///
    /// Verdadeira, o `restamp_shapes_preview` descasca o preview e volta sem carimbar nada: o gizmo
    /// carrega a forma sozinho.
    pub(super) fn draft_stamp(&self) -> bool {
        self.paint.shape_draft
    }

    /// A rota dos editores de figura, embrulhada pela lei do repouso.
    ///
    /// ⚠️ **A bandeira cai ANTES da rota no Up**, então o re-carimbo que o próprio editor faz ao soltar
    /// JÁ é o final e o gesto paga **exatamente um** carimbo. O `settle` do fim é para o ramo que NÃO
    /// re-carimba — o `editing` do `ellipse_up`/`curve_up` fecha a transação de undo e sai por
    /// `return true` — e é no-op quando o editor já pagou.
    pub(super) fn route_shape_draft(&mut self, ev: CanvasPointer) -> bool {
        self.paint.shape_draft = ev.phase != PointerPhase::Up;
        let used = self.route_shape_pointer_multi(ev);
        if ev.phase == PointerPhase::Up {
            // ⚠️ **MUTAÇÃO SOBREVIVENTE, mantida com o motivo.** Hoje ela é no-op: todo Up dos quatro
            // editores passa por `commit_shape_txn`, que assenta antes de capturar — varrido pelo
            // `no_editor_leaves_the_canvas_owing_at_rest`, e a mutação *"tire esta linha"* não sangra.
            // Ela fica porque as duas portas existem por razões DIFERENTES: aquela é da CORREÇÃO do
            // undo (um snapshot não pode descrever uma tela que ninguém viu), esta é da FEATURE (o
            // artista tem de ver a figura ao soltar). Delegar a segunda à primeira faz a lei depender
            // de todo Up abrir transação — exatamente a enumeração que este desenho evita —, e o modo
            // de falha é o pior que há: a figura fica INVISÍVEL até o próximo evento.
            self.settle_shape_draft();
        }
        used
    }

    /// **Devolve as figuras à tela, se elas estiverem devendo.** No-op quando não estão.
    ///
    /// ⚠️ **Toda captura de undo passa por aqui, e é isso que torna a lei correta em vez de só rápida.**
    /// Um `ModelSnapshot` guarda o `drag_preview` como `preview_patch`, então um snapshot tirado com a
    /// tela rascunhada descreve um estado que o artista nunca viu — e a consequência medida não foi
    /// cosmética: escrever a figura de volta **depois** do commit fazia a escrita ser absorvida pelo
    /// passo anterior (`undo_absorb`), e o redo de um arrasto de curva devolvia uma cena **sem a
    /// curva**. Dois gates de undo nasceram vermelhos com isso, e é por isso que o `commit_shape_txn`
    /// chama esta porta ANTES de capturar.
    pub(crate) fn settle_shape_draft(&mut self) {
        if self.paint.shape_stale {
            self.paint.shape_draft = false;
            self.refill_live_shape();
        }
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
            // Sem editor ativo as figuras estacionadas ainda são a imagem — elas também sumiram
            // durante o gesto.
            self.restamp_shapes_preview(&[]);
        }
    }
}
