//! **A cena do GESTO do Pattern Along Path (W3)** — `PH2D_BUILD_SMOKE=24`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `text_path_smoke`. A mesa fica
//! posta: um **motivo** (seta azul) SELECIONADO e um **guia** (arco cinza) à espera, e a ferramenta
//! Vector ativa. O painel Vector já deve mostrar a seção **Pattern on Path** com o botão **Pick
//! Path** — o gesto de duas mãos (Enio 2026-07-23: aperta e clica o guia). Daí em diante é o
//! `pattern_live::recook` + o `dispatch` que desenham as cópias (a fonte, que o Node edita, nunca é
//! tocada).
//!
//! # O roteiro (impresso no terminal)
//!
//! 1. **Pick Path** — com a seta selecionada, aperta; o realce acende a forma sob o cursor.
//! 2. **Clique no arco** — a seta some do lugar e reaparece **repetida ao longo dele**, cada cópia
//!    **girada para a tangente** dali (é o que separa isto do Repeater). Vazio/direito desiste.
//! 3. **Spacing** — arrastar o slider muda quão densas as cópias povoam a curva; a **contagem é
//!    automática** (mais espaçamento ⇒ menos cópias).
//! 4. **Start / End** — pelo slider OU pelas **duas fichas âmbar na curva** (modo Select): as
//!    cópias caem só no trecho `[Start, End]` do arco.
//! 5. **Slide** — desliza o trecho inteiro (as duas âncoras juntas) pela curva.
//! 6. **Offset** — empurra as cópias para fora/dentro da curva (perpendicular).
//! 7. **Side: Other side** — o padrão passa para o outro lado (e as fichas viram junto).
//! 8. **Detach from Path** — o motivo volta a ser uma forma solta e **o guia fica**.
//!
//! (Alternativa: selecionar a seta *e* o arco mostra **Pattern on Path**, a auto-ligação por dois.)

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
    // Só o MOTIVO selecionado — é a seleção que o **Picker** exige (a fonte à espera do clique do
    // guia). Selecionar os DOIS mostraria a auto-ligação ("Pattern on Path") em vez do Picker; o
    // Picker é a porta explícita e mais correta (Enio 2026-07-23), então é ela que a cena encena.
    app.vec_pen.select(Some(motif));
    let sel = app.vec_pen.selected_paths().to_vec();
    let ready = sel.len() == 1
        && sel[0] == motif
        && app
            .gfx
            .as_ref()
            .is_some_and(|g| g.vec_scene.path(guide).is_some());

    eprintln!(
        "[smoke] W4 pattern along path (Picker) -- a mesa esta' posta: uma SETA (motivo) \
         SELECIONADA e um ARCO (guia) a` espera. Picker pronto: {ready}.\n\
         [smoke]   1. No painel Vector, secao PATTERN ON PATH -> clique \"Pick Path\".\n\
         [smoke]      O motivo continua selecionado; o realce acende a forma sob o cursor.\n\
         [smoke]   2. Clique no ARCO -- a seta some do lugar e reaparece REPETIDA ao longo dele,\n\
         [smoke]      cada copia girada para a tangente dali (o que separa isto do Repeater).\n\
         [smoke]      (Clique no vazio, ou o botao DIREITO, DESISTE do pick.)\n\
         [smoke]   3. Arraste Spacing -- muda quao densas as copias povoam a curva; a CONTAGEM e'\n\
         [smoke]      automatica (mais espacamento => menos copias).\n\
         [smoke]   4. Arraste Start/End (slider) OU as duas FICHAS ambar na curva (modo Select).\n\
         [smoke]   5. Arraste Slide -- desliza o trecho INTEIRO (as duas ancoras juntas) pela curva.\n\
         [smoke]   6. Arraste Offset -- empurra as copias para fora/dentro da curva (perpendicular).\n\
         [smoke]   7. Side: \"Other side\" -- o padrao passa para o outro lado, e as FICHAS viram junto.\n\
         [smoke]   8. \"Detach from Path\" -- o motivo volta a ser forma solta, e o ARCO fica.\n\
         [smoke]   (Alternativa: selecione a SETA *e* o ARCO -> a secao mostra \"Pattern on Path\", a\n\
         [smoke]    auto-ligacao por dois selecionados -- o guia e' o de maior extensao.)"
    );
    if !ready {
        eprintln!(
            "[smoke] !! a mesa NAO esta' posta -- o painel nao vai oferecer o Picker, e o resto do \
             smoke nao significa nada. PARE e reporte."
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
        assert!(
            copies.len() >= 8,
            "o arco cabe várias setas, veio {}",
            copies.len()
        );

        // As setas GIRAM: a direção ponta−centroide varia mais de 60° ao longo do arco (senão a
        // cena demonstra o pattern sem mostrar a rotação — a diferença do Repeater).
        let dir = |c: &VecPath| {
            let n = c.verts.len() as f64;
            let ctr = c
                .verts
                .iter()
                .fold([0.0, 0.0], |a, v| [a[0] + v.anchor[0], a[1] + v.anchor[1]]);
            let tip = c.verts[1].anchor; // a ponta da seta é o índice 1
            (tip[1] - ctr[1] / n)
                .atan2(tip[0] - ctr[0] / n)
                .to_degrees()
        };
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for c in &copies {
            let a = dir(c);
            lo = lo.min(a);
            hi = hi.max(a);
        }
        assert!(
            hi - lo > 60.0,
            "as setas mal giram ({lo:.0} a {hi:.0} graus)"
        );
    }
}
