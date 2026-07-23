//! **A cena do GESTO do Pattern Along Path (W3)** — `PH2D_BUILD_SMOKE=24`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `text_path_smoke`. A mesa fica
//! posta: um **motivo** (seta azul) e um **guia** (arco cinza), os DOIS selecionados com o motivo
//! como PRIMÁRIO, e a ferramenta Vector ativa. O painel Vector já deve mostrar a seção **Pattern on
//! Path** com o botão de prender — e daí em diante é o `pattern_live::recook` + o `dispatch` que
//! desenham as cópias (a fonte, que o Node edita, nunca é tocada).
//!
//! # O roteiro (impresso no terminal)
//!
//! 1. **Pattern on Path** — a seta some do lugar e reaparece **repetida ao longo do arco**, cada
//!    cópia **girada para a tangente** dali (é o que separa isto do Repeater).
//! 2. **Spacing** — arrastar o slider muda quão densas as cópias povoam a curva.
//! 3. **Start / End** — pelo slider OU pelas **duas fichas âmbar na curva** (W4, no modo Select):
//!    as cópias caem só no trecho `[Start, End]` do arco.
//! 4. **Offset** — empurra as cópias para fora/dentro da curva (perpendicular).
//! 5. **Side: Other side** — o padrão passa para o outro lado.
//! 6. **Detach from Path** — o motivo volta a ser uma forma solta e **o guia fica**.

use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecPathId, VecVertex, VertexKind};

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // Só no frame seguinte: o `sync` do render loop é que dá entidade aos paths, e sem
        // entidade o `link` não acha o motivo (a mesma razão da cena 23).
        4 => arm(app),
        _ => {}
    }
}

/// O motivo (seta azul) + o guia (arco cinza), à espera das entidades que o `sync` lhes dá.
static PENDING: std::sync::Mutex<Option<(VecPathId, VecPathId)>> = std::sync::Mutex::new(None);

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else { return };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));

    // O motivo: uma seta assimétrica que aponta em +x, para a rotação por-cópia ser óbvia. Posta
    // de lado (o guia é que decide onde ela cai) — as cópias substituem este desenho.
    let motif = gfx.vec_scene.push_path(VecPath {
        verts: [[0.0, -0.18], [0.5, 0.0], [0.0, 0.18]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(90, 150, 220, 255))),
        stroke: None,
        ..Default::default()
    });
    // O guia: um arco largo e raso, aberto — onde a rotação por-cópia se vê sem as setas se
    // amontoarem umas nas outras.
    let guide = gfx.vec_scene.push_path(VecPath {
        verts: arc(),
        closed: false,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(120, 120, 130, 255), 0.02)),
        ..Default::default()
    });
    PENDING.lock().expect("smoke lock").replace((motif, guide));
}

fn arm(app: &mut crate::App) {
    let Some((motif, guide)) = PENDING.lock().expect("smoke lock").take() else {
        return;
    };
    // Os DOIS selecionados, com o motivo como PRIMÁRIO (o `select_many` faz o ÚLTIMO ser o
    // primário) — é a seleção que o gesto exige, e a cena existe para a pôr pronta.
    app.vec_pen.select_many(&[guide, motif]);
    let sel = app.vec_pen.selected_paths().to_vec();
    let primary = app.vec_pen.selected();
    let can = crate::pattern_live::link_candidate(&sel, primary);

    eprintln!(
        "[smoke] W3 pattern along path -- a mesa esta' posta: uma SETA (motivo) e um ARCO (guia), \
         os DOIS selecionados (motivo primario), gesto oferecido: {}.\n\
         [smoke]   1. No painel Vector, secao PATTERN ON PATH -> clique \"Pattern on Path\".\n\
         [smoke]      A seta some do lugar e reaparece REPETIDA ao longo do arco, cada copia\n\
         [smoke]      girada para a tangente dali (nao um angulo unico -- isso separa do Repeater).\n\
         [smoke]   2. Arraste Spacing -- muda quao densas as copias povoam a curva.\n\
         [smoke]   3. Arraste Start/End (slider) OU as duas FICHAS ambar na curva (W4, modo\n\
         [smoke]      Select) -- as copias caem so no trecho [Start, End] do arco.\n\
         [smoke]   4. Arraste Offset -- empurra as copias para fora/dentro da curva (perpendicular).\n\
         [smoke]   5. Side: \"Other side\" -- o padrao passa para o outro lado.\n\
         [smoke]   6. \"Detach from Path\" -- o motivo volta a ser forma solta, e o ARCO fica.",
        can.is_some()
    );
    if can.is_none() {
        eprintln!(
            "[smoke] !! a mesa NAO esta' posta (gesto: false) -- o painel nao vai oferecer o botao, \
             e o resto do smoke nao significa nada. PARE e reporte."
        );
    }
}

/// Um arco largo e raso, aberto, da esquerda para a direita (o mesmo formato da cena 23).
fn arc() -> Vec<VecVertex> {
    let pt = |x: f64, y: f64, hx: f64| VecVertex {
        anchor: [x, y],
        in_handle: [x - hx, y],
        out_handle: [x + hx, y],
        kind: VertexKind::Smooth,
        corner_radius: 0.0,
    };
    vec![pt(-3.4, 0.0, 1.1), pt(0.0, 1.7, 1.4), pt(3.4, 0.0, 1.1)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::arc_path::ArcPath;
    use ph2d_vec_scene::pattern_path::{PatternSpec, pattern_along};

    /// **A cena mostra o que a mensagem diz** (a política de física): as cópias tilam o arco E
    /// giram com ele. A sonda roda ANTES de a mensagem ser escrita, e fica.
    #[test]
    fn the_scene_tiles_the_arc_and_the_copies_turn() {
        let guide = ArcPath::from_contour(&arc(), false).expect("arco");
        let motif = VecPath {
            verts: [[0.0, -0.18], [0.5, 0.0], [0.0, 0.18]]
                .map(VecVertex::corner)
                .to_vec(),
            closed: true,
            ..VecPath::default()
        };
        let copies = pattern_along(&motif, &guide, &PatternSpec::default());
        assert!(copies.len() >= 8, "o arco cabe várias setas, veio {}", copies.len());

        // As setas GIRAM: a direção ponta−centroide varia mais de 60° ao longo do arco (senão a
        // cena demonstra o pattern sem mostrar a rotação — a diferença do Repeater).
        let dir = |c: &VecPath| {
            let n = c.verts.len() as f64;
            let ctr = c
                .verts
                .iter()
                .fold([0.0, 0.0], |a, v| [a[0] + v.anchor[0], a[1] + v.anchor[1]]);
            let tip = c.verts[1].anchor; // a ponta da seta é o índice 1
            (tip[1] - ctr[1] / n).atan2(tip[0] - ctr[0] / n).to_degrees()
        };
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for c in &copies {
            let a = dir(c);
            lo = lo.min(a);
            hi = hi.max(a);
        }
        assert!(hi - lo > 60.0, "as setas mal giram ({lo:.0} a {hi:.0} graus)");
    }
}
