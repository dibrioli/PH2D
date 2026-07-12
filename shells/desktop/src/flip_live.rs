//! **O alvo VIVO** — o que você acabou de criar continua respondendo aos controles.
//!
//! Enquanto não existe seleção de traço (Enio 2026-07-12), o alvo dos ajustes do painel é
//! **a última coisa que você fez**: o traço recém-desenhado, ou o preenchimento
//! recém-aplicado. Mexer em Size, Color, Smoothing, Gap, Grow ou Precision **refaz** esse
//! alvo na hora, em vez de só valer para o próximo. É a diferença entre um painel de
//! *configuração* (que exige adivinhar antes) e um de *ajuste* (que deixa ver e corrigir).
//!
//! ## As duas coisas que isto exige, e que não são óbvias
//!
//! **1. Guardar o INSUMO, não o resultado.** Um traço assado já perdeu as amostras cruas —
//! e sem elas, mudar o Smoothing é impossível (ele filtra as amostras, não o traço). Um
//! preenchimento já perdeu o desenho de antes — e sem ele, mudar o Gap não teria de onde
//! partir. Então o alvo vivo guarda o insumo de cada um: as amostras (mundo + pressão) do
//! traço, e a lista de traços PRISTINA de antes do fill.
//!
//! **2. Re-aplicar sempre a partir do insumo — nunca do resultado anterior.** É o mesmo
//! erro do *offset acumulador* do Painter (`docs/Painter/09`): reaplicar sobre o que já
//! está lá faz os parâmetros se COMPOREM em vez de se substituírem, e arrastar o slider
//! para a frente e para trás não volta ao mesmo lugar. Aqui o Gap de um segundo fill
//! recriaria os fechamentos do primeiro por cima dos que já existem.
//!
//! O alvo morre no instante em que deixa de ser "a última coisa": outro traço, uma
//! borracha, outro quadro, outra camada, outro modo, um undo.

use ph2d_core::Vec2;
use ph2d_flip::{DrawingId, FlipObjectId, FlipStroke, Frame, LayerId};
use ph2d_tool_flip::{FlipMode, FlipStyleSnapshot};
use ph2d_vec_scene::Xform;

/// Onde o alvo vivo mora e o que ele precisa para ser refeito.
pub(crate) struct FlipLive {
    pub(crate) oid: FlipObjectId,
    pub(crate) did: DrawingId,
    /// O quadro e a camada em que ele nasceu — sair de qualquer um dos dois o mata (o
    /// alvo é *aquele* desenho, não "o desenho atual").
    pub(crate) frame: Frame,
    pub(crate) layer: Option<LayerId>,
    /// O modo da tool quando ele nasceu: um traço não responde ao Gap do balde.
    pub(crate) mode: FlipMode,
    /// O estilo com que ele está AGORA na tela. Se o do painel divergir, refaz.
    pub(crate) applied: FlipStyleSnapshot,
    /// A câmera e o afim do objeto no momento da criação — o insumo tem de ser
    /// re-convertido com os MESMOS números, senão refazer moveria a geometria.
    pub(crate) px_to_world: f32,
    pub(crate) w2l: Xform,
    pub(crate) kind: LiveKind,
}

/// O insumo de cada tipo de alvo.
pub(crate) enum LiveKind {
    /// O último TRAÇO: as amostras CRUAS (mundo + pressão) e onde ele está na pilha.
    /// Sem as amostras, o Smoothing seria inajustável — ele filtra o insumo, não a saída.
    Stroke {
        index: usize,
        points: Vec<Vec2>,
        pressures: Vec<f32>,
    },
    /// O último PREENCHIMENTO: o desenho PRISTINO de antes dele (a base), e o clique.
    /// Re-preencher parte SEMPRE daqui — nunca do resultado anterior, senão os
    /// fechamentos de gap se acumulariam a cada mexida no slider.
    Fill { base: Vec<FlipStroke>, click: Vec2 },
}

/// Os atributos que decidem a forma de um TRAÇO. (Mudar o Gap não refaz um traço.)
fn same_stroke_style(a: &FlipStyleSnapshot, b: &FlipStyleSnapshot) -> bool {
    a.stroke == b.stroke
        && a.width_px.to_bits() == b.width_px.to_bits()
        && a.hardness.to_bits() == b.hardness.to_bits()
        && a.opacity.to_bits() == b.opacity.to_bits()
        && a.smoothing.to_bits() == b.smoothing.to_bits()
}

/// Os atributos que decidem a forma de um PREENCHIMENTO.
fn same_fill_style(a: &FlipStyleSnapshot, b: &FlipStyleSnapshot) -> bool {
    a.fill_color == b.fill_color
        && a.fill_mode == b.fill_mode
        && a.gap_px.to_bits() == b.gap_px.to_bits()
        && a.grow.to_bits() == b.grow.to_bits()
        && a.precision.to_bits() == b.precision.to_bits()
}

impl crate::App {
    /// Mata o alvo vivo. Chamado por tudo que deixa de ser "a última coisa que você fez":
    /// borracha, undo, carregar projeto.
    pub(crate) fn flip_live_clear(&mut self) {
        self.flip_live = None;
    }

    /// **Refaz o alvo vivo se o painel mudou.** Roda uma vez por frame, depois de o
    /// `flip_bridge` publicar o estilo.
    pub(crate) fn flip_live_refresh(&mut self) {
        let Some(live) = self.flip_live.as_ref() else {
            return;
        };
        let Some(style) = self.flip_style else {
            return;
        };
        // O alvo é *aquele* desenho: mudar de quadro, de camada ou de modo o solta.
        let frame_now = self
            .gfx
            .as_ref()
            .and_then(|g| g.flip.object(live.oid).map(|o| o.frame_at(&self.playhead)));
        if style.mode != live.mode
            || self.flip_active_layer != live.layer
            || frame_now != Some(live.frame)
        {
            self.flip_live = None;
            return;
        }
        let dirty = match live.kind {
            LiveKind::Stroke { .. } => !same_stroke_style(&style, &live.applied),
            LiveKind::Fill { .. } => !same_fill_style(&style, &live.applied),
        };
        if !dirty {
            return;
        }
        self.flip_live_reapply(&style);
    }

    /// Reconstrói o alvo a partir do INSUMO (nunca do resultado anterior).
    fn flip_live_reapply(&mut self, style: &FlipStyleSnapshot) {
        let Some(live) = self.flip_live.as_mut() else {
            return;
        };
        let (px_to_world, w2l) = (live.px_to_world, live.w2l);
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(drawing) = gfx
            .flip
            .object_mut(live.oid)
            .and_then(|o| o.drawing_mut(live.did))
        else {
            self.flip_live = None; // o desenho sumiu (delete/undo): o alvo morreu com ele
            return;
        };

        match &live.kind {
            LiveKind::Stroke {
                index,
                points,
                pressures,
            } => {
                let Some(slot) = drawing.strokes.get_mut(*index) else {
                    self.flip_live = None;
                    return;
                };
                *slot = crate::flip_draw::stroke_from_samples(
                    style,
                    points,
                    pressures,
                    px_to_world,
                    &w2l,
                );
            }
            LiveKind::Fill { base, click } => {
                // Parte da base PRISTINA. Se o novo ajuste não produzir fill (baixar o Gap
                // abaixo do necessário, por exemplo), fica o ÚLTIMO resultado bom — o
                // preenchimento não some da tela por causa de um valor intermediário
                // enquanto o slider passa por ele.
                let last_good = std::mem::replace(&mut drawing.strokes, base.clone());
                if crate::flip_fill::fill_click(drawing, style, *click, px_to_world, &w2l).is_err()
                {
                    drawing.strokes = last_good;
                    return; // `applied` não avança: tenta de novo no próximo ajuste
                }
            }
        }
        live.applied = *style;
    }
}
