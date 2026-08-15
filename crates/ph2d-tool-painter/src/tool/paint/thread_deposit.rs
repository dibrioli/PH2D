//! **O depósito dos FIOS** — a metade de tool dos tipos que costuram (`Sketchy`, W3 · `Wire`, W4).
//!
//! O motor costura (`ph2d_painter_brush::stroke::threads`) e responde *que alfa este feixe deixa*
//! (`ph2d_painter_brush::thread_raster`); aqui decide-se **com que cor**, **onde**, e — a parte que
//! importa — **pela mesma porta de proteção que os dabs atravessam** (`gate_scoped`). É o desenho do
//! irmão [`super::solid_deposit`], e de propósito: um canal novo de tinta que não passasse pelo gate
//! seria a segunda semântica de proteção do módulo.

use super::Region;
use crate::tool::PainterTool;
use ph2d_painter_brush::Stroke;
use ph2d_painter_brush::thread_raster::{ThreadInk, threads_alpha, threads_bbox};

impl PainterTool {
    /// **A PORTA ÚNICA por onde o traço volta para o tool.** Ela substitui os quatro
    /// `self.paint.stroke = Some(stroke)` do ciclo de traço (begin · extend · tick · finish).
    ///
    /// ⚠️ **Ela existe porque os fios saem por um canal PRÓPRIO** (um fio é um segmento, não um dab)
    /// e nada no compilador liga esse canal ao carimbo. Drená-los "nos sítios que eu lembrei" é
    /// exactamente a enumeração que este módulo já viu apodrecer três vezes (o `smears()`, os
    /// leitores de `Dab::dir`, os escritores de canvas).
    ///
    /// ⚠️ **E o modo de falha de esquecer esta porta é ALTO, não silencioso:** quem não a chamar não
    /// devolve o traço, o evento seguinte encontra `None` e o pincel **para de pintar** no meio do
    /// gesto. Um dreno esquecido, em compensação, seria uma teia que some — sem erro nenhum.
    pub(super) fn park_stroke(&mut self, mut stroke: Stroke) {
        // Drena SEMPRE, escreve só quando a tinta é nossa: sem o dreno a memória do motor cresceria
        // durante o traço inteiro num modo que não a desenha.
        let mut threads = std::mem::take(&mut self.paint.pending_threads);
        stroke.take_threads(&mut threads);
        let owns = self.threads_own_the_gesture();
        // O par cosido/carimbado é o que separa *o motor não costurou* de *o depósito recusou* —
        // ver o doc do `ribbon_diag`, onde essa confusão já custou uma hora.
        super::ribbon_diag::note_threads(threads.len(), if owns { threads.len() } else { 0 });
        if !threads.is_empty() && owns {
            self.stamp_threads(&threads);
        }
        threads.clear();
        self.paint.pending_threads = threads;
        self.paint.stroke = Some(stroke);
    }

    /// O gesto está costurando fios?
    ///
    /// ⚠️ **Porta única, e ela pergunta ao TIPO e ao MÉTODO.** Ao tipo pela `sews_threads`, que é
    /// quem sabe que a família tem dois membros — um `match` escrito aqui seria a segunda lista a
    /// apodrecer no terceiro. Ao método porque a memória do traço
    /// só é laid UMA vez nos métodos **incrementais**; os de re-carimbo (Line / Drag Dot / Anchored
    /// e os shape editors) re-emitem a figura INTEIRA a cada quadro, então a memória cresceria por
    /// quadro e a teia adensaria enquanto o artista apenas olha — a mesma doença que o `pre`
    /// congelado do sculpt existe para impedir. O que falta para eles é a wave dos shape editors,
    /// **nomeada** no handoff em vez de ficar como uma teia que engorda parada.
    pub(super) fn threads_own_the_gesture(&self) -> bool {
        self.paint.brush.sews_threads()
            && self.paint.brush.stroke_method.is_incremental()
            && matches!(self.paint.paint_mode, super::PaintMode::Paint)
            && !self.paint.eraser
            && !self.paint.brush.watercolor
            && !self.paint.wetpaint.armed
            && !self.solid_owns_the_gesture()
    }

    /// **O `Connection Line` do Wire está DESLIGADO?** — então o traço em si não é pintado e sobra
    /// só o arame (o *Paint connection line* do Krita, verbatim: *"which toggles the visibility of
    /// the connection line"*).
    ///
    /// ⚠️ **A supressão mora na MESMA porta do `Style: Solid`** (`stamp_dabs`), e não nos sítios do
    /// ciclo de traço — o doc daquela porta já diz por quê, e uma segunda casa para *"não deposite
    /// dabs"* é como as duas passam a discordar num modo que só uma delas conhece.
    ///
    /// ⚠️ E ela **não** desliga a memória: os fios continuam a ser costurados a partir dos mesmos
    /// pontos, porque o arame é função do caminho, não da tinta que ele deixou.
    pub(super) fn wire_suppresses_dabs(&self) -> bool {
        !self.paint.brush.wire_connection_line
            && self.paint.brush.wire_active()
            && self.threads_own_the_gesture()
    }

    /// A tinta com que ESTE pincel desenha um fio.
    ///
    /// ⚠️ **Só a espessura e a opacidade.** O `Reach` e o `Magnetify` decidem QUE PARES o motor
    /// costura (`ph2d_painter_brush::stroke::threads`), e um fio que chegou até aqui já é um par
    /// escolhido — reler os dois no depósito seria a segunda resposta à mesma pergunta.
    fn thread_ink(&self) -> ThreadInk {
        let b = &self.paint.brush;
        ThreadInk {
            width_px: b.thread_width_px,
            opacity: b.thread_opacity,
        }
    }

    /// Rasteriza o feixe e o escreve — **uma janela, uma escrita**, seja qual for o número de fios.
    ///
    /// ⚠️ Não há restore/re-carimbo aqui, e não é esquecimento: um fio é tinta CUMULATIVA (como um
    /// dab de método incremental), não um preview que se refaz. É por isso que o depósito é um
    /// `over` sobre o que já está lá em vez da dança do `drag_preview`.
    fn stamp_threads(&mut self, threads: &[ph2d_painter_brush::stroke::threads::Thread]) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let ink = self.thread_ink();
        let Some([bx, by, bw, bh]) = threads_bbox(threads, ink.width_px, w as usize, h as usize)
        else {
            return;
        };
        #[allow(clippy::cast_possible_truncation)]
        let rect = Region {
            x: bx as u32,
            y: by as u32,
            w: bw as u32,
            h: bh as u32,
        };
        let mask_gate = self.mask_protection_active();
        let sel_gate = self.selection_restricts_paint();
        if mask_gate || sel_gate {
            self.gate_scoped(rect, mask_gate, sel_gate, |t| {
                t.write_threads(threads, rect)
            });
        } else {
            self.write_threads(threads, rect);
        }
    }

    /// A escrita crua: `over` da cor do pincel pesada pelo alfa do feixe.
    fn write_threads(
        &mut self,
        threads: &[ph2d_painter_brush::stroke::threads::Thread],
        rect: Region,
    ) {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        #[allow(clippy::cast_precision_loss)]
        let origin = [rect.x as f32, rect.y as f32];
        let ink = self.thread_ink();
        let alpha = threads_alpha(threads, ink, rect.w as usize, rect.h as usize, origin);
        let col = self.paint.brush.color;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let rgb = [
            (col[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u32,
            (col[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u32,
            (col[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u32,
        ];
        // ⚠️ A janela DECLARADA é a que este feixe escreve — é ela que faz o commit de undo usar a
        // janela em vez de varrer os planos (doc 28 §5.17).
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
                let a = u32::from(alpha[row * rect.w as usize + cx]);
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
