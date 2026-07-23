//! **A cena do Pattern Along Path (W2)** — `PH2D_BUILD_SMOKE=24`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `text_path_smoke`. Prova o
//! **pipeline VIVO** (o que o texto smoke 22 NÃO faz: aquele monta a geometria à mão): um motivo e
//! um guia entram na cena, o [`crate::pattern_live::link`] os vincula, e a partir daí é o
//! `pattern_live::recook` + o `dispatch` do render loop que desenham as cópias — a fonte nunca é
//! tocada. Se as cópias aparecem, o componente, o cozimento e a fusão na `LiveGeometry` funcionam.
//!
//! # O que julgar
//!
//! - O **motivo** (uma seta azul) some do lugar onde foi posto e reaparece **repetido ao longo da
//!   curva**, cada cópia **girada para a tangente** dali (é o que separa isto do Repeater).
//! - O **guia** (cinza) fica desenhado por baixo — as cópias andam por cima dele.
//! - O espaçamento entre as cópias é constante em ARCO (não aperta nas dobras).

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
    let Some(gfx) = app.gfx.as_mut() else { return };
    let linked = crate::pattern_live::link(&mut gfx.sim, &app.vec_entities, motif, guide);

    // ⚠️ MEÇA antes de escrever a mensagem (a lição de física): quantas cópias o `recook` vai de
    // facto desenhar? Resolve o guia pela MESMA porta do render loop e roda o MESMO motor.
    let n = crate::vec_guide::guide_arc(&gfx.sim, &gfx.vec_scene, &app.vec_entities, guide)
        .and_then(|arc| gfx.vec_scene.path(motif).map(|m| (arc, m)))
        .map_or(0, |(arc, m)| {
            let cooked = m.cooked();
            ph2d_vec_scene::pattern_path::pattern_along(
                &cooked,
                &arc,
                &ph2d_vec_scene::pattern_path::PatternSpec {
                    start_offset: 0.0,
                    spacing: 1.0,
                    ..Default::default()
                },
            )
            .len()
        });

    eprintln!(
        "[smoke] W2 pattern along path -- vinculado: {linked} · {n} copia(s) na curva.\n\
         [smoke]   A SETA azul foi posta de lado e agora aparece REPETIDA ao longo do arco,\n\
         [smoke]   cada copia girada para a tangente dali (nao um angulo unico -- isso e' o que\n\
         [smoke]   separa o pattern do Repeater). O guia cinza fica por baixo.\n\
         [smoke]   REPROVE se: as setas nao giram com a curva, ou o motivo continua desenhado\n\
         [smoke]   solto no lugar onde foi posto (o vinculo/cozimento nao pegou)."
    );
    if !linked || n == 0 {
        eprintln!(
            "[smoke] !! a mesa NAO esta' posta (vinculado: {linked}, copias: {n}) -- as copias nao \
             vao aparecer, e o resto do smoke nao significa nada. PARE e reporte."
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
