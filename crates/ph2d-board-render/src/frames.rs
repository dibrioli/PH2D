//! **Os frames** (W5): o fundo e a borda por TRÁS de tudo (o Miro: *«Frames are always placed behind
//! board objects»*), o conteúdo RECORTADO pela borda (oráculo Excalidraw A10: o filho que atravessa
//! a borda corta-se nela — no editor, no PNG e no SVG), e o título por cima, num tamanho FIXO de ecrã
//! (o do oráculo: 14 px a 3 px acima do frame, à esquerda) — é o que se lê de longe.
//!
//! O **nível de detalhe** ([`Detail`]): um frame pequeno no ecrã mostra só o fundo e o título; uma
//! nota de letra ilegível é um bloco da cor dela. Os limiares são parâmetros do desenho para as
//! réguas compararem as variantes no MESMO processo (`tests/it/measure_encode_cost.rs`).

use std::collections::BTreeSet;

use ph2d_board_model::{Board, BoardDoc, Element, ElementId, RichText};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Point, Rect, Shape as _, Stroke, VectorScene};

use crate::{RenderCache, doc_color, token};

/// ⭐ **O nível de detalhe** — os limiares do desenho de longe, em px de ECRÃ.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Detail {
    /// Abaixo desta ALTURA no ecrã um frame mostra só o fundo e o título (o conteúdo não se
    /// desenha). `0` = sempre o conteúdo.
    pub frame_title_only_px: f64,
    /// Uma nota cuja letra fica abaixo do `READ_PX` vira um BLOCO da cor dela: sem sombra nem
    /// riscas (a queixa aberta do Miro: de longe, um brainstorm é uma mancha de cores).
    pub note_blocks: bool,
}

impl Default for Detail {
    fn default() -> Self {
        Self {
            frame_title_only_px: FRAME_TITLE_ONLY_PX,
            note_blocks: true,
        }
    }
}

/// O limiar de nascença de [`Detail::frame_title_only_px`]. ⚠️ Medido na 2.ª rodada da W5 (ver a
/// tabela no handoff W5 e na régua).
pub const FRAME_TITLE_ONLY_PX: f64 = 96.0;

/// A caixa de `el` no ecrã.
pub(crate) fn screen_box(board: &Board, area: [f64; 4], el: &Element) -> Rect {
    let a = board.camera.to_screen(area, [el.x, el.y]);
    let b = board.camera.to_screen(area, [el.x + el.w, el.y + el.h]);
    Rect::new(a[0], a[1], b[0], b[1])
}

/// Os frames à vista, na ordem de desenho, com a caixa deles no ecrã.
pub(crate) fn visible<'a>(board: &'a Board, area: [f64; 4], clip: Rect) -> Vec<(&'a Element, Rect)> {
    board
        .doc
        .frames_in_draw_order()
        .into_iter()
        .map(|f| (f, screen_box(board, area, f)))
        .filter(|(_, r)| r.intersect(clip).area() > 0.0)
        .collect()
}

/// O fundo e a borda de cada frame à vista; devolve os que, de tão pequenos, mostram só o título.
pub(crate) fn paint_backs(
    scene: &mut VectorScene,
    frames: &[(&Element, Rect)],
    theme: Theme,
    detail: Detail,
) -> BTreeSet<ElementId> {
    let hairline = Stroke::new(f64::from(StrokeToken::Hairline.px()));
    let mut far = BTreeSet::new();
    for (f, r) in frames {
        let st = f.style();
        if let Some(c) = st.fill {
            scene.fill_rect(*r, doc_color(c));
        }
        // A borda: o contorno do estilo do frame (a barra de cores serve-o como às formas), à
        // escala do quadro; sem contorno, o fio da interface que o separa do fundo.
        let (line, brush) = match st.stroke.filter(|_| st.stroke_width > 0.0) {
            Some(c) => {
                let px = st.stroke_width * r.width() / f.w.max(f64::EPSILON);
                (Stroke::new(px), Brush::Solid(doc_color(c)))
            }
            None => (hairline.clone(), Brush::Solid(token(ColorToken::Border, theme))),
        };
        scene
            .inner_mut()
            .stroke(&line, Affine::IDENTITY, &brush, None, &r.to_path(0.1));
        if r.height() < detail.frame_title_only_px {
            far.insert(f.id);
        }
    }
    far
}

/// ⭐ **O recorte dos frames** enquanto se desenha em ordem de z: a pilha de recortes acompanha a
/// cadeia de frames de cada elemento (do de fora para dentro), abrindo e fechando só o que muda.
#[derive(Default)]
pub(crate) struct Clips {
    open: Vec<ElementId>,
}

impl Clips {
    /// Põe o recorte de `chain` (do de FORA para DENTRO).
    pub(crate) fn enter(
        &mut self,
        scene: &mut VectorScene,
        board: &Board,
        area: [f64; 4],
        chain: &[ElementId],
    ) {
        let keep = self
            .open
            .iter()
            .zip(chain)
            .take_while(|(a, b)| a == b)
            .count();
        while self.open.len() > keep {
            self.open.pop();
            scene.pop_layer();
        }
        for id in &chain[keep..] {
            if let Some(f) = board.doc.get(*id) {
                scene.push_clip(&screen_box(board, area, f));
            }
            self.open.push(*id);
        }
    }

    pub(crate) fn close(&mut self, scene: &mut VectorScene) {
        for _ in self.open.drain(..) {
            scene.pop_layer();
        }
    }
}

/// ⭐ **As cadeias de todos os frames, UMA vez por imagem** — e os que escondem o que vive dentro
/// deles (fora de vista: o recorte esconde-o; de longe: só o título), herdado pelos de dentro.
/// ⛔ Medido (07/10, 1 000 frames × 100 formas): a cadeia pedida por ELEMENTO custava ~6 ms dos 12
/// com os frames todos só em título.
#[derive(Default)]
pub(crate) struct Chains {
    /// Frame → a cadeia de quem vive nele, do de FORA para DENTRO (ele incluído, no fim).
    of: std::collections::BTreeMap<ElementId, Vec<ElementId>>,
    hidden: BTreeSet<ElementId>,
}

impl Chains {
    pub(crate) fn new(doc: &BoardDoc, shown: &BTreeSet<ElementId>, far: &BTreeSet<ElementId>) -> Self {
        let mut c = Chains::default();
        for f in doc.frames() {
            let mut chain = doc.frame_chain(f.id);
            chain.reverse();
            chain.push(f.id);
            if chain.iter().any(|x| !shown.contains(x) || far.contains(x)) {
                c.hidden.insert(f.id);
            }
            c.of.insert(f.id, chain);
        }
        c
    }

    /// A cadeia de `el` (vazia num solto) — `None` se um frame dela o esconde.
    pub(crate) fn of(&self, el: &Element) -> Option<&[ElementId]> {
        match el.frame {
            None => Some(&[]),
            Some(f) if self.hidden.contains(&f) => None,
            // Um ponteiro para um frame apagado: solto (a cadeia do modelo pára aí).
            Some(f) => Some(self.of.get(&f).map_or(&[][..], Vec::as_slice)),
        }
    }
}

/// O título de cada frame à vista, por cima de tudo, no tamanho FIXO de ecrã, recortado à largura
/// do frame.
pub(crate) fn paint_titles(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    board_id: u64,
    frames: &[(&Element, Rect)],
    chains: &Chains,
    theme: Theme,
) {
    let size = TypeToken::Sm.px();
    let gap = f64::from(Spacing::Xs.px());
    let ink = token(ColorToken::Text2, theme);
    for (f, r) in frames {
        let Some(frame) = f.as_frame() else {
            continue;
        };
        // Dentro de um frame visto de longe (só o título dele) o título de dentro não se pinta.
        if frame.name.is_empty() || r.width() <= 0.0 || chains.of(f).is_none() {
            continue;
        }
        let text = RichText::plain(&frame.name);
        let layout = cache
            .text
            .get_rich(ts, (board_id, f.id.0), &text, size, f32::MAX, false);
        let h = f64::from(layout.height());
        let band = Rect::new(r.x0, r.y0 - gap - h, r.x1, r.y0);
        scene.push_clip(&band);
        let at = Affine::translate(Point::new(r.x0, r.y0 - gap - h).to_vec2());
        ph2d_board_layout::paint(scene, layout, at, ink);
        scene.pop_layer();
    }
}

