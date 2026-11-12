//! **Uma lane da pilha como o painel a desenha** — a ÚNICA construção de [`StripView`]: a pilha
//! aberta (`TimelineViewSnapshot::rebuild`) e o container aberto no lugar (`crate::no_lugar`)
//! passam ambos por aqui, então a fita de dentro e a de fora não podem divergir.

use crate::doc::TimelineDoc;
use crate::snapshot::{LaneView, StripView};

/// `lane` com as janelas de mistura perguntadas à LANE (o mesmo número que o avaliador pesa) e a
/// costura resolvida sob o loop `(range, ping)` que governa a pilha onde ela mora.
pub(crate) fn lane_view(
    doc: &TimelineDoc,
    lane: &crate::ClipLane,
    range: Option<(f64, f64)>,
    ping: bool,
) -> LaneView {
    // ⚠️ **A costura é resolvida por LANE, uma vez** — e só sob um loop que ENVOLVE:
    // sob ping-pong o playhead reflete, não há travessia da volta a desenhar.
    let ctx = crate::stack::FadeCtx {
        seam: if ping { None } else { lane.seam(range) },
        reversed: doc.reverse_play(),
    };
    let seam = ctx.split_seam();
    let mut strips = Vec::with_capacity(lane.strips.len());
    for (i, st) in lane.strips.iter().enumerate() {
        strips.push(StripView {
            id: st.id,
            // The strip's source names itself, whichever list it lives in — a
            // container strip that painted a blank label would read as a corrupt
            // clip strip rather than as what it is (ADR-0133).
            container: st.source.container_index().map(usize::from),
            clip_name: match st.source {
                crate::StripSource::Clip(i) => doc
                    .clips()
                    .get(i as usize)
                    .map_or_else(String::new, |c| c.name.clone()),
                crate::StripSource::Container(i) => doc
                    .containers()
                    .get(i as usize)
                    .map_or_else(String::new, |c| c.name.clone()),
            },
            t_start: st.t_start,
            t_end: st.t_end,
            blend_in: lane.blend_in(i),
            blend_out: lane.blend_out(i),
            lead_in: st.lead_in,
            lead_out: st.lead_out,
            marks: st.marks,
            ease_locked_in: lane.neighbour_reach_in(i) > 0.0,
            ease_locked_out: lane.neighbour_reach_out(i) > 0.0,
            // ⚠️ **A curva PUBLICADA é a efetiva do frame** — inclusive a direção:
            // com o relógio andando para trás o desenho decorativo tem de mostrar o
            // espelho, que é o que a pose faz. É por isso que o painel não precisa
            // saber da direção (Enio, 2026-08-01).
            curve_in: ctx.shape(lane.effective_curve(i, 0)),
            curve_out: ctx.shape(lane.effective_curve(i, 1)),
            seam: seam.and_then(|sm| sm.slice_for(i)),
            loop_mode: st.loop_mode,
            speed: st.speed,
            volta: st.volta(),
        });
    }
    LaneView {
        name: lane.name.clone(),
        muted: lane.muted,
        weight: lane.weight,
        mode: lane.mode,
        strips,
    }
}
