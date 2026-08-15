//! **O depósito de uma forma SÓLIDA** — a metade de tool do `Style: Solid` (plano 38 §1.1, W7).
//!
//! O motor (`ph2d_painter_brush::solid`) responde *que cobertura este caminho fechado tem*; aqui
//! decide-se *o que fazer com ela*: onde escrever, com que cor, e — a parte que importa — **pela
//! mesma porta de proteção que os dabs atravessam** (`gate_scoped`).
//!
//! ⚠️ **O caminho é acumulado pelo TOOL, não pela lista de dabs**, e a razão é medível: a lista de
//! dabs de um evento é o pedaço do gesto desde o último ponteiro, não o gesto — preencher o polígono
//! dela daria a mancha de um fragmento. O que uma forma sólida cerca é o percurso INTEIRO.
//!
//! # ⚠️ O Solid deixou de ESCONDER o traço (W7, ordem do Enio 2026-08-15)
//!
//! Até aqui `Solid` era *a região **EM VEZ** do traço*: os dabs eram suprimidos e a §1.1 do plano
//! registrava a decisão de então — *"para forma sólida a espessura da linha passa a não ser
//! considerada"*. O pedido novo é o oposto e é do mesmo autor: ***"Solid deve usar o pincel com o
//! falloff e espessura do traço como no modo flip"*.**
//!
//! No Flip um traço tem `fill: Option<Fill>` e **continua a ter a largura e a dureza dele** — o
//! preenchimento é a região, e o contorno é o pincel. É esse o modelo agora: a mancha é a região
//! CERCADA, e o traço é carimbado como em qualquer outro gesto. Isso entrega as três metades do
//! pedido de uma vez, porque as três eram a mesma:
//!
//!   1. o **falloff e a espessura** voltam — eles são o pincel, e o pincel voltou a carimbar;
//!   2. **todo tipo de linha** passa a funcionar sob Solid (Speed, Sketchy, Wire, Ribbon, Rough
//!      DECORAM o traço, e não havia traço para decorar);
//!   3. **Symmetry e Tiling** alcançam o traço **de graça** — eles moram na porta do dab —, e esta
//!      wave só teve de os levar ao PREENCHIMENTO e aos FIOS.
//!
//! # ⚠️ A ordem entre a mancha e o traço NÃO é observável, e é por isso que as duas famílias podem
//! usar transações diferentes
//!
//! As duas tintas são a **mesma cor**, e o `over` de duas fontes da mesma cor é **comutativo** em
//! cor e em alfa: `a₁ ⊕ a₂ = a₁ + a₂ − a₁a₂` é simétrico, e a cor sai `c` nos dois casos. Logo
//! *fill-por-baixo* e *fill-por-cima* dão a MESMA imagem (ao arredondamento de `u8`), e cada família
//! de depósito pode usar a transação que já tem:
//!
//! - **métodos de RE-CARIMBO** (Drag Dot / Anchored / Line e os cinco shape editors): o
//!   preenchimento viaja **dentro** do `stamp_drag_preview`, entre o `save` e o carimbo — porque ali
//!   os dabs também são transitórios, e duas transações encadeadas no mesmo slot deixariam só a
//!   última de pé;
//! - **métodos CUMULATIVOS** (mão livre, Airbrush, Dots, Grid Stamp): os dabs são permanentes, então
//!   o preenchimento é uma transação PRÓPRIA, e o `stamp_dabs` a **descasca antes** do lote e a
//!   **re-escreve depois** ([`Self::freehand_solid_fill_live`]). O snapshot tem de conter todo dab e
//!   nenhum fill — se ele envelhecer, o restore do quadro seguinte apaga tinta do artista.

use super::Region;
use crate::tool::PainterTool;
use ph2d_painter_brush::solid;

impl PainterTool {
    /// O gesto está sendo depositado como forma SÓLIDA?
    ///
    /// ⚠️ **Porta única, e ela pergunta ao MODO também** — o Solid escreve pigmento pela rota própria,
    /// então onde o depósito não é pigmento (Sculpt, Smear/Blur/Clone, a máscara, a aquarela, o
    /// fluido) ele não tem o que preencher, e deixá-lo entrar seria o botão fazendo outra coisa em
    /// cada ferramenta. `false` ⇒ tudo é byte-idêntico ao mundo sem esta feature.
    pub(super) fn solid_owns_the_gesture(&self) -> bool {
        self.paint.brush.style_solid
            && matches!(self.paint.paint_mode, super::PaintMode::Paint)
            && !self.paint.eraser
            && !self.paint.brush.watercolor
            && !self.paint.wetpaint.armed
    }

    /// **O preenchimento é uma transação PRÓPRIA neste gesto?** — a testemunha que faz o `stamp_dabs`
    /// bracketar o lote (descascar antes, re-escrever depois).
    ///
    /// ⚠️ Ela pergunta pelo **caminho acumulado** e pelo **método**, nunca por uma lista de sítios: o
    /// ciclo de traço carimba dabs em **seis** lugares (pen-down, move, o tick do airbrush, o settle
    /// do estabilizador, o finish e o pen-up), e bracketar os que eu lembrei é como o sétimo nasce
    /// apagando a tinta do artista — o restore de um snapshot velho não *deixa de desenhar*, ele
    /// **desfaz**. O caminho só é semeado pelo `paint_begin`, e os cinco shape editors nem passam
    /// por ele (`canvas_pointer` os desvia antes), então só um gesto cumulativo cai aqui.
    ///
    /// ⚠️ **`is_incremental` é o que separa as duas transações da §doc do módulo:** Drag Dot,
    /// Anchored e Line também semeiam o caminho, mas os dabs deles são transitórios e o
    /// preenchimento viaja dentro do `stamp_drag_preview`.
    pub(super) fn freehand_solid_fill_live(&self) -> bool {
        !self.paint.solid_path.is_empty()
            && self.paint.brush.stroke_method.is_incremental()
            && self.solid_owns_the_gesture()
    }

    /// **A PORTA ÚNICA de *"que laços este gesto preenche AGORA?"*** — já replicados pela Symmetry e
    /// pelo Tiling, prontos para uma única passada de `fill_coverage`.
    ///
    /// A cena de FORMAS responde primeiro (um shape editor autora a geometria diretamente,
    /// `super::solid_shapes`); na ausência dela, o caminho acumulado do gesto à mão livre.
    ///
    /// ⚠️ **Uma passada só sobre o conjunto INTEIRO, e não é economia — é CORREÇÃO:** o
    /// preenchimento é `nonzero` sobre o conjunto, então formas que se sobrepõem (as cópias de
    /// simetria inclusive) fundem sem costura. Preenchê-las uma a uma comporia a borda anti-aliased
    /// **duas vezes** e deixaria uma linha escura exatamente onde elas se tocam.
    pub(super) fn solid_fill_loops(&self) -> Vec<Vec<[f32; 2]>> {
        if !self.solid_owns_the_gesture() {
            return Vec::new();
        }
        let mut loops = self.solid_loops();
        if loops.is_empty() && self.paint.solid_path.len() >= 2 {
            loops.push(self.paint.solid_path.clone());
        }
        if loops.is_empty() {
            return loops;
        }
        // ⚠️ A ordem é SYMMETRY e depois TILING, a mesma que a lista de dabs percorre: o motor
        // espelha na emissão (`push_symmetric`) e o `stamp_dabs_routed` envolve as cópias na costura
        // depois. Invertida, uma cópia espelhada que caísse fora da tela não seria envolvida, e a
        // mancha e o traço passariam a discordar sobre onde a tile continua.
        let mirrored =
            ph2d_painter_brush::symmetry::symmetric_loops(&loops, &self.paint.brush.symmetry);
        if self.paint.tiling[0] || self.paint.tiling[1] {
            super::tiling::tiled_loops(&mirrored, self.source_size, self.paint.tiling)
        } else {
            mirrored
        }
    }

    /// O retângulo do canvas que este preenchimento escreve, ou `None` se ele não toca a tela.
    pub(super) fn solid_fill_rect(&self, loops: &[Vec<[f32; 2]>]) -> Option<Region> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || loops.is_empty() {
            return None;
        }
        let [bx, by, bw, bh] = solid::loops_bbox(loops, w as usize, h as usize)?;
        #[allow(clippy::cast_possible_truncation)]
        Some(Region {
            x: bx as u32,
            y: by as u32,
            w: bw as u32,
            h: bh as u32,
        })
    }

    /// **A transação do preenchimento SOZINHO** — restaurar → medir → salvar → escrever, para os
    /// métodos CUMULATIVOS (a §doc do módulo diz por que só eles).
    ///
    /// Chamada pelo `stamp_dabs` **depois** do lote, com o preview já descascado: o snapshot que ela
    /// guarda contém todo dab do gesto e nenhum fill, que é o invariante inteiro.
    pub(super) fn stamp_solid_preview(&mut self) {
        let loops = self.solid_fill_loops();
        self.stamp_solid_loops(&loops);
    }

    /// Preenche os laços dados, restaurando o preview do quadro anterior — a MESMA dança do
    /// `stamp_drag_preview` (restaurar → medir → salvar → escrever), porque um Solid é um re-carimbo
    /// por construção: a cada ponto novo o polígono INTEIRO muda de forma.
    pub(super) fn stamp_solid_loops(&mut self, loops: &[Vec<[f32; 2]>]) {
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        let Some(rect) = self.solid_fill_rect(loops) else {
            return;
        };
        let pixels = self.save_region(&rect);
        self.stamp_solid(loops, rect);
        self.paint.drag_preview = Some(super::DragPreview { rect, pixels });
    }

    /// Escreve a região **pela porta do gate**, exactamente como um lote de dabs: a proteção e a
    /// seleção são um fator por texel aplicado UMA vez sobre a tinta acumulada livre, e é isso que
    /// impede o Solid de ser a segunda semântica de proteção do módulo.
    pub(super) fn stamp_solid(&mut self, loops: &[Vec<[f32; 2]>], rect: Region) {
        let mask_gate = self.mask_protection_active();
        let sel_gate = self.selection_restricts_paint();
        if mask_gate || sel_gate {
            self.gate_scoped(rect, mask_gate, sel_gate, |t| t.write_solid(loops, rect));
        } else {
            self.write_solid(loops, rect);
        }
    }

    /// A escrita crua: `over` da cor do pincel pesada pela cobertura exata.
    fn write_solid(&mut self, loops: &[Vec<[f32; 2]>], rect: Region) {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        #[allow(clippy::cast_precision_loss)]
        let origin = [rect.x as f32, rect.y as f32];
        let cov = solid::fill_coverage(loops, rect.w as usize, rect.h as usize, origin);
        let col = self.paint.brush.color;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let rgb = [
            (col[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u32,
            (col[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u32,
            (col[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u32,
        ];
        // ⚠️ A opacidade do pincel entra AQUI. A ESPESSURA não entra nesta mancha e nunca entrará:
        // ela é o que o TRAÇO desenha, e desde a W7 o traço é carimbado ao lado dela (§doc do
        // módulo). Fazer a mancha crescer meia-espessura sozinha seria uma SEGUNDA resposta a *"que
        // borda este pincel tem"*, divergindo do falloff no dia em que alguém afinasse um dos dois.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let strength = (self.paint.brush.strength.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
        // ⚠️ A janela DECLARADA é o retângulo que este preenchimento escreve — é ela que faz o commit
        // de undo usar a janela em vez de varrer os planos (doc 28 §5.17), e é por isso que a porta
        // de fork a recebe em vez de a redescobrir.
        let width_px = self.source_size.0;
        let canvas = super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            width_px,
            Some(rect),
        );
        for row in 0..rect.h as usize {
            let y = rect.y as usize + row;
            if y >= h {
                break;
            }
            for cx in 0..rect.w as usize {
                let x = rect.x as usize + cx;
                if x >= w {
                    break;
                }
                let a = u32::from(cov[row * rect.w as usize + cx]) * strength / 255;
                if a == 0 {
                    continue;
                }
                let p = (y * w + x) * 4;
                for ch in 0..3 {
                    let d = u32::from(canvas[p + ch]);
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        canvas[p + ch] = ((rgb[ch] * a + d * (255 - a)) / 255) as u8;
                    }
                }
                let da = u32::from(canvas[p + 3]);
                #[allow(clippy::cast_possible_truncation)]
                {
                    canvas[p + 3] = (a + da * (255 - a) / 255) as u8;
                }
            }
        }
        self.declare_wrote(Some(rect));
        self.mark_dirty(rect);
    }
}
