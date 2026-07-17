//! ADR-0114 §4.B — **o domínio SEGMENT do Edit Mode** no shell, módulo-irmão de
//! `flip_select` (cap de LOC do HR-18). O motor (cortes → pedaços) mora no MODELO
//! (`ph2d_flip::segment`); aqui fica o que só o shell sabe: **quem corta quem**.
//!
//! # O que o shell resolve: o QUADRO
//!
//! O `02_referencia §11` manda cortar contra *"o BVH 2D do **frame**"*, e é o que a
//! referência faz: o corte é *"por interseção VISUAL"*, então quem corta é **tudo que
//! está na tela** — não só o desenho que se está editando. Aqui isso é
//! [`frame_cutters`]: cada camada **visível** contribui o desenho que ela mostra no
//! playhead, pela **pose daquela chave**.
//!
//! **O espaço comum é o do OBJETO.** Cada camada tem a sua pose por chave, e a arte é
//! local a ela — duas camadas não se falam em coords de arte. A cadeia é
//! `arte → pose_da_chave → objeto`, e o objeto é onde todas se encontram. Não é
//! preciso subir até a TELA (como a referência sobe): cruzamento é **invariante afim**, e
//! a fração λ do corte também — subir seria arredondamento de graça.
//!
//! # Três recusas, cada uma por um motivo
//!
//! - **camada invisível não corta**: não está na tela, e o corte é visual. (Travada
//!   corta: você a VÊ. A referência exclui a travada junto com a invisível porque lá o
//!   conjunto que corta é o conjunto EDITÁVEL — uma coincidência da arquitetura dela, e
//!   o preço seria trancar uma camada mudar em silêncio onde os pedaços do vizinho
//!   começam.)
//! - **traço sem tinta não corta** (`hide_stroke`): uma região do balde é um anel
//!   INVISÍVEL que costeia a arte-linha (BUGS #14: o balde ancora no EIXO da linha) —
//!   deixá-la cortar picaria a linha em dezenas de pedaços que o artista não desenhou.
//! - **os 3 vizinhos não cortam**: é do motor, no modelo (senão todo segmento cortaria
//!   nas pontas que compartilha com o vizinho).

use std::ops::Range;

use ph2d_core::Vec2;
use ph2d_flip::{Cutter, FlipDrawing, FlipObject, Frame, LayerId};

use crate::flip_select_pick::Where;
use crate::flip_select_points::DownPoints;

/// Os segmentos que **cortam** neste quadro, em espaço de OBJETO, mais o mapa de onde os
/// segmentos de cada traço da camada ATIVA caíram (o `tree_data_range` da referência).
pub(crate) struct FrameCutters {
    segs: Vec<Cutter>,
    /// Indexado pelo `si` do desenho ativo. `None` = o traço não contribui tinta
    /// (`hide_stroke`) ⇒ não tem segmento na lista ⇒ **não tem corte**: é um pedaço só.
    own: Vec<Option<Range<usize>>>,
}

impl FrameCutters {
    /// **Os cortes do traço `si`** — a porta única. Traço sem tinta não está na lista de
    /// cortadores: nada o cruza, e um `cuts` vazio é exatamente isso (o
    /// [`ph2d_flip::piece_of_point`] devolve um pedaço só).
    #[must_use]
    fn cuts_of(&self, drawing: &FlipDrawing, si: usize) -> Vec<Option<f32>> {
        match self.own.get(si).cloned().flatten() {
            Some(range) => ph2d_flip::cuts(&self.segs, range, drawing.strokes[si].closed),
            None => Vec::new(),
        }
    }

    /// **O mapa de pedaços do traço `si`**: `dono[p]` = o id do pedaço do ponto `p`. O
    /// pick, o marquee e o colapso saem todos DESTA função — não têm como divergir.
    #[must_use]
    pub(crate) fn piece_map(&self, drawing: &FlipDrawing, si: usize) -> Vec<u32> {
        let s = &drawing.strokes[si];
        ph2d_flip::piece_of_point(&self.cuts_of(drawing, si), s.len(), s.closed)
    }

    /// Os pontos que um clique em `hit` acende no traço `si`, **e o ponto-sonda** que
    /// representa o pedaço (o alvo do colapso adiado).
    ///
    /// `Where::Whole` (o miolo de um preenchimento, ou um traço de um ponto só) acende o
    /// traço INTEIRO **ignorando os cortes**: o `fill` é do anel todo, então quem aponta o
    /// miolo apontou a forma e não uma aresta dela — não existe "um pedaço do
    /// preenchimento" para escolher.
    #[must_use]
    fn piece_points(&self, drawing: &FlipDrawing, si: usize, hit: Where) -> (Vec<usize>, usize) {
        let s = &drawing.strokes[si];
        let Where::Ink { i, t } = hit else {
            return ((0..s.len()).collect(), 0);
        };
        let cuts = self.cuts_of(drawing, si);
        let map = ph2d_flip::piece_of_point(&cuts, s.len(), s.closed);
        // A SONDA, não o `i` cru: um clique além do corte que parte o segmento `i`
        // pertence ao pedaço que o corte ABRIU, e esse começa no `i+1`.
        let probe = ph2d_flip::probe_point(&cuts, s.len(), i, t);
        let want = map[probe];
        ((0..s.len()).filter(|&p| map[p] == want).collect(), probe)
    }
}

/// **Os cortadores do quadro** — ver o doc do módulo para as três recusas.
///
/// `frame` é o quadro do playhead; `active` é a camada que está sendo editada (é dela que
/// o mapa `own` fala). A pose de cada camada sai do [`ph2d_flip::FlipLayer::pose_at_cycled`]
/// — o par exato do `drawing_at_cycled`: amostrar a arte pelo ciclo e o LUGAR pelo quadro
/// cru poria a arte da 2ª volta do Loop na pose errada
/// ([[feedback_derived_coordinate_seed_must_match_sample]]).
#[must_use]
pub(crate) fn frame_cutters(obj: &FlipObject, frame: Frame, active: LayerId) -> FrameCutters {
    let mut segs: Vec<Cutter> = Vec::new();
    let mut own: Vec<Option<Range<usize>>> = Vec::new();
    for layer in obj.layers() {
        if !layer.visible {
            continue; // não está na tela ⇒ não corta
        }
        let Some(did) = layer.drawing_at_cycled(frame) else {
            continue;
        };
        let Some(drawing) = obj.drawing(did) else {
            continue;
        };
        let pose = layer.pose_at_cycled(frame);
        let is_active = layer.id == active;
        if is_active {
            own = vec![None; drawing.strokes.len()];
        }
        for (si, s) in drawing.strokes.iter().enumerate() {
            if s.hide_stroke {
                continue; // anel invisível do balde ⇒ não corta
            }
            let start = segs.len();
            segs.extend(s.segments().map(|(_, a, b)| (pose.apply(a), pose.apply(b))));
            if is_active && segs.len() > start {
                own[si] = Some(start..segs.len());
            }
        }
    }
    FrameCutters { segs, own }
}

/// **O plano do pen-DOWN no domínio Segment** — o espelho de
/// [`crate::flip_select_points::plan_down_points`], com o PEDAÇO no lugar do ponto.
///
/// Reusa o `DownPoints` de propósito: o Segment **não é um gesto novo**, é uma política de
/// PICK. O que ele acende é um conjunto de pontos, e daí para a frente (arrastar, o slop,
/// o colapso, a recusa de instância) tudo é o domínio Point do W8, sem uma linha nova.
///
/// O **colapso adiado** aponta o ponto-sonda: soltar sem arrastar re-pergunta ao MESMO
/// pick (ver [`collapse_to_piece`]) — sem arrasto o cursor não saiu do lugar, então a
/// resposta é a mesma, e guardar o pedaço seria um 2º dono da pergunta.
pub(crate) fn plan_down_segment(
    drawing: &mut FlipDrawing,
    cutters: &FrameCutters,
    hit: Option<(usize, Where)>,
    shift: bool,
    in_box: bool,
) -> DownPoints {
    match (hit, shift) {
        (None, false) if in_box => DownPoints::Move { collapse_to: None },
        (None, shift) => {
            if !shift {
                drawing.clear_selection();
            }
            DownPoints::Marquee { additive: shift }
        }
        (Some((si, w)), shift) => {
            let (pts, probe) = cutters.piece_points(drawing, si, w);
            let already = pts.iter().all(|&p| drawing.strokes[si].point_selected(p));
            if shift {
                // Shift alterna o pedaço INTEIRO: ele acende se ALGUM ponto dele estava
                // apagado (senão alternar um pedaço meio-aceso o apagaria pela metade).
                for &p in &pts {
                    drawing.strokes[si].set_point_selected(p, !already);
                }
                return DownPoints::Click;
            }
            if !already {
                drawing.clear_selection();
            }
            for &p in &pts {
                drawing.strokes[si].set_point_selected(p, true);
            }
            // Já estava todo aceso ⇒ o gesto é ambíguo (colapsar × arrastar o grupo), e o
            // colapso fica ADIADO para o pen-up decidir — a regra do domínio Point.
            DownPoints::Move {
                collapse_to: already.then_some((si, probe)),
            }
        }
    }
}

/// **O colapso adiado do domínio Segment**: soltou sem arrastar em cima de um pedaço que
/// JÁ estava todo aceso ⇒ "agora só este". Acende o pedaço do ponto-sonda e apaga o resto.
pub(crate) fn collapse_to_piece(
    drawing: &mut FlipDrawing,
    cutters: &FrameCutters,
    si: usize,
    probe: usize,
) -> bool {
    if si >= drawing.strokes.len() || probe >= drawing.strokes[si].len() {
        return false;
    }
    let map = cutters.piece_map(drawing, si);
    let want = map[probe];
    let mut changed = drawing.clear_selection();
    for (p, &owner) in map.iter().enumerate() {
        if owner == want {
            changed |= drawing.strokes[si].set_point_selected(p, true);
        }
    }
    changed
}

/// **O marquee no domínio Segment** — o pós-processo da referência
/// (`apply_mask_as_segment_selection`): a caixa produz uma máscara de PONTOS e o modo a
/// **expande** para os pedaços que ela tocou. Encostar num ponto de um pedaço acende o
/// pedaço inteiro; é o que separa "arrastar uma caixa por cima de um traço" de "recortar
/// o traço na borda da caixa".
pub(crate) fn apply_marquee_segments(
    drawing: &mut FlipDrawing,
    cutters: &FrameCutters,
    min: Vec2,
    max: Vec2,
    additive: bool,
) -> bool {
    let mut changed = false;
    if !additive {
        changed |= drawing.clear_selection();
    }
    for si in 0..drawing.strokes.len() {
        // Os pontos que a caixa pegou. **Perguntar isto ANTES dos cortes é o que mantém o
        // custo honesto**: os cortes são O(segmentos do traço × segmentos do quadro), e uma
        // caixa costuma tocar poucos traços — computá-los para os traços que ela nem
        // encostou pagaria o N² do quadro inteiro por nada (medido: 12,6 ms num quadro de
        // 2940 segmentos, contra ~1 ms quando só os tocados entram).
        let inside: Vec<usize> = (0..drawing.strokes[si].len())
            .filter(|&p| {
                let q = drawing.strokes[si].positions()[p];
                q.x >= min.x && q.x <= max.x && q.y >= min.y && q.y <= max.y
            })
            .collect();
        if inside.is_empty() {
            continue;
        }
        let map = cutters.piece_map(drawing, si);
        let touched: std::collections::BTreeSet<u32> = inside.iter().map(|&p| map[p]).collect();
        for (p, owner) in map.iter().enumerate() {
            if touched.contains(owner) {
                changed |= drawing.strokes[si].set_point_selected(p, true);
            }
        }
    }
    changed
}

#[cfg(test)]
#[path = "flip_select_segment_tests.rs"]
mod tests;
