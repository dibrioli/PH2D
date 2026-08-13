//! **Que rows o painel de params PINTA** — os gates de visibilidade condicional.
//!
//! O assunto é uma pergunta só: *este controle é lido por alguma rota agora?* Se
//! não é, ele não se pinta (a lei do `amount_y` do `motion.scale`), e o painel a
//! responde por duas famílias de side-metadata — o `ParamGate` (a condição é o
//! valor de outro param **f32**) e o `ParamGateText` (a condição é a PRESENÇA de
//! um param de TEXTO, como o nome de uma forma desenhada).
//!
//! Cortado do irmão `motion_bridge_param_tests.rs` no teto de LOC do shell, por
//! ASSUNTO: lá mora *que NÚMERO uma row carrega* (a faixa que contém o valor, a
//! unidade, o canal), aqui *se a row EXISTE*. Declarado pelo pai como um
//! `#[path]`, então `super` é `render_loop::motion_bridge`.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;

/// **Os oito sliders de polígono de controle somem quando o artista escolhe a
/// forma que desenhou** (`ParamGateText`, smoke de 2026-08-12).
///
/// ⚠️ **PRESENÇA e AUSÊNCIA, e a ausência é a metade que prende:** um gate que só
/// pedisse *"com forma escolhida as oito somem"* ficaria verde com as oito
/// removidas do nó, com o polígono de controle inalcançável para sempre. As duas
/// perguntas são independentes e as duas estão aqui.
#[test]
fn the_control_polygon_rows_hide_when_a_drawn_shape_is_named() {
    use ph2d_panel_motion_params::ParamRow;
    const COORDS: [&str; 8] = ["p0x", "p0y", "p1x", "p1y", "p2x", "p2y", "p3x", "p3y"];
    let mut motion = MotionState::new();
    let sw = motion.doc.graph.add_node("motion.spline_wrap");
    ph2d_panel_motion_graph::set_graph_selection(vec![sw.0]);

    let names = |m: &MotionState| -> Vec<String> {
        build_params_snapshot(m, ProjectSettings::default())
            .expect("o no resolve")
            .rows
            .iter()
            // ⚠️ O nome do param sai de CADA variante, não só da escalar: a row da
            // forma é `Source` e o `follow_rotation` é `Toggle`, e uma fixture que
            // as formatasse como `{:?}` procuraria um nome que nunca casa —
            // reprovando sobre um produto certo (foi o que a primeira versão fez).
            .map(|r| match r {
                ParamRow::Scalar(s) => s.name.to_string(),
                ParamRow::Text(t) => t.name.to_string(),
                ParamRow::Source(s) => s.param.to_string(),
                ParamRow::Toggle(t) => t.name.to_string(),
                other => format!("{other:?}"),
            })
            .collect()
    };

    // SEM forma: as oito estão lá, e é assim que o nó sempre foi autorado.
    let before = names(&motion);
    for c in COORDS {
        assert!(
            before.iter().any(|n| n == c),
            "sem forma escolhida, `{c}` tem de ser pintavel: {before:?}"
        );
    }
    // E a row da FORMA existe nos dois estados — é por ela que se sai daqui.
    assert!(
        before.iter().any(|n| n == "path"),
        "a row da forma e a porta, e nunca some: {before:?}"
    );

    // COM forma: as oito somem, e o resto dos controles fica.
    motion.doc.graph.set_text_param(sw, "path", "Track");
    let after = names(&motion);
    for c in COORDS {
        assert!(
            !after.iter().any(|n| n == c),
            "com forma escolhida, `{c}` nao e lido por rota nenhuma -- nao se pinta: {after:?}"
        );
    }
    for live in [
        "path",
        "follow_rotation",
        "height_scale",
        "offset",
        "from",
        "to",
    ] {
        assert!(
            after.iter().any(|n| n == live),
            "`{live}` vale na curva desenhada tambem: {after:?}"
        );
    }

    // ⚠️ E um nome em BRANCO não é uma forma: apagar o texto devolve as oito.
    motion.doc.graph.set_text_param(sw, "path", "   ");
    assert_eq!(names(&motion), before, "um nome vazio nao esconde nada");
}
