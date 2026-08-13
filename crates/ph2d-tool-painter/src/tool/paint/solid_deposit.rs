//! **O depósito de uma forma SÓLIDA** — a metade de tool do `Style: Solid` (plano 38 §1.1).
//!
//! O motor (`ph2d_painter_brush::solid`) responde *que cobertura este caminho fechado tem*; aqui
//! decide-se *o que fazer com ela*: onde escrever, com que cor, e — a parte que importa — **pela
//! mesma porta de proteção que os dabs atravessam** (`gate_scoped`).
//!
//! ⚠️ **O caminho é acumulado pelo TOOL, não pela lista de dabs**, e a razão é medível: a lista de
//! dabs de um evento é o pedaço do gesto desde o último ponteiro, não o gesto — preencher o polígono
//! dela daria a mancha de um fragmento. O que uma forma sólida cerca é o percurso INTEIRO.

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

    /// **O gesto sólido está EM VOO?** — a testemunha que suprime o carimbo de dabs.
    ///
    /// ⚠️ Ela pergunta pelo **caminho acumulado**, não por uma lista de sítios: o ciclo de traço
    /// carimba dabs em **seis** lugares (pen-down, move, o tick do airbrush, o settle do
    /// estabilizador, o finish e o pen-up), e guardar os que eu lembrei é como o sétimo nasce
    /// carimbando a linha por baixo da mancha — que foi exactamente o defeito que o gate da espessura
    /// pegou, com dois dos três guardados. O caminho só é semeado pelo `paint_begin` de um gesto
    /// sólido, então nenhum shape editor e nenhum outro modo cai aqui por acidente.
    pub(super) fn solid_suppresses_dabs(&self) -> bool {
        !self.paint.solid_path.is_empty() && self.solid_owns_the_gesture()
    }

    /// Preenche o caminho acumulado, restaurando o preview do quadro anterior — a MESMA dança do
    /// `stamp_drag_preview` (restaurar → medir → salvar → escrever), porque um Solid é um re-carimbo
    /// por construção: a cada ponto novo o polígono INTEIRO muda de forma.
    pub(super) fn stamp_solid_preview(&mut self) {
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || self.paint.solid_path.len() < 2 {
            return;
        }
        let loops = [std::mem::take(&mut self.paint.solid_path)];
        let bb = solid::loops_bbox(&loops, w as usize, h as usize);
        if let Some([bx, by, bw, bh]) = bb {
            #[allow(clippy::cast_possible_truncation)]
            let rect = Region {
                x: bx as u32,
                y: by as u32,
                w: bw as u32,
                h: bh as u32,
            };
            let pixels = self.save_region(&rect);
            self.stamp_solid(&loops, rect);
            self.paint.drag_preview = Some(super::DragPreview { rect, pixels });
        }
        let [path] = loops;
        self.paint.solid_path = path;
    }

    /// Escreve a região **pela porta do gate**, exactamente como um lote de dabs: a proteção e a
    /// seleção são um fator por texel aplicado UMA vez sobre a tinta acumulada livre, e é isso que
    /// impede o Solid de ser a segunda semântica de proteção do módulo.
    fn stamp_solid(&mut self, loops: &[Vec<[f32; 2]>], rect: Region) {
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
        // ⚠️ A opacidade do pincel entra AQUI e a espessura NÃO entra em lugar nenhum — é a frase do
        // pedido em código (*"para forma sólida a espessura da linha passa a não ser considerada"*).
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
