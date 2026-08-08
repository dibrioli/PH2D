//! **A cena da família TRANSFORM** (`PH2D_TRANSFORM_SMOKE=1`, doc 88 §B3).
//!
//! A varredura PRO por família começa aqui, e o censo (`param_census`, na
//! `ph2d-node-registry-init`) é quem escolheu a família: dos 118 nós, dois desta
//! **contradiziam a própria coluna que escrevem**.
//!
//! - **`motion.scale`** escrevia `size`, que é uma coluna **Vec2**, a partir de **UM**
//!   número. *Squash & stretch* — o primeiro dos doze princípios da animação — não era
//!   difícil no grafo: era **inexprimível**, porque nó nenhum sabia esticar um eixo e não
//!   o outro. Agora o `Uniform` é o *link* de corrente do AE/Cavalry/Figma, e destravá-lo
//!   revela o `Scale Y`.
//! - **`motion.mirror`** pregava a linha de espelho no **centroide** do layout, então a
//!   simetria só sabia acontecer contra si mesma — encostá-la numa parede era
//!   igualmente inexprimível. O `Axis Offset` move a linha.
//!
//! ⚠️ **Os dois defaults são byte-idênticos ao que já shipava**, e é o que torna a wave
//! segura: a demo de boot sozinha põe treze `motion.scale`. O `Uniform` nasce ligado e o
//! `Axis Offset` nasce em zero — cada um reduzindo LITERALMENTE à expressão antiga.
//!
//! ```text
//! motion.grid -> motion.scale -> motion.mirror -> motion.output
//! ```

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A largura e a altura autoradas do `motion.scale` da cena — esticado e achatado, a
/// razão de aspecto que UM número jamais produz.
const STRETCH_X: f32 = 2.2;
const STRETCH_Y: f32 = 0.45;
/// O deslocamento da linha de espelho, em METROS (o documento é sempre métrico; a face
/// do artista lê `120 px` na escala de fábrica).
const MIRROR_OFFSET: f32 = 1.2;

/// Monta `grid -> scale -> mirror -> output`. Devolve `(sink, [grid, scale, mirror])`.
fn chain(g: &mut Graph) -> (NodeId, [NodeId; 3]) {
    let grid = g.add_node("motion.grid");
    let scale = g.add_node("motion.scale");
    let mirror = g.add_node("motion.mirror");
    let out = g.add_node("motion.output");

    // Uma FILEIRA de quatro, larga o bastante para o espelho cair ao lado e não em cima.
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 4.0);
    g.set_param(grid, "gap_x", 0.45);

    // O link DESTRAVADO: a razão de aspecto sai de 1 e é isso que se vê.
    g.set_param(scale, "uniform", 0.0);
    g.set_param(scale, "amount", STRETCH_X);
    g.set_param(scale, "amount_y", STRETCH_Y);

    // Eixo vertical, linha deslocada do centroide — o gêmeo nasce à direita, separado.
    g.set_param(mirror, "axis", 0.0);
    g.set_param(mirror, "offset", MIRROR_OFFSET);

    for (from, to) in [(grid, scale), (scale, mirror), (mirror, out)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .expect("transform-smoke edge");
    }
    (out, [grid, scale, mirror])
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_TRANSFORM_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `units_smoke`. No-op sem a env.
    pub(crate) fn transform_family_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let (sink, heroes) = chain(&mut gfx.motion.doc.graph);
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.push(sink);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        // O `Scale` já selecionado: o `Uniform` destravado e o `Scale Y` visível no 1º
        // frame, que é o que a wave entrega.
        ph2d_panel_motion_graph::request_graph_selection(vec![heroes[1].0]);
        eprintln!(
            "[transform smoke] grid(1x4) -> scale -> mirror -> output.\n  \
             MONTOU: 4 pontos, esticados {STRETCH_X}x em X e {STRETCH_Y}x em Y, \
             espelhados numa linha a {MIRROR_OFFSET} m do centroide -> 8 instancias.\n  \
             Na tela: quadrados ACHATADOS E LARGOS (nao quadrados), em dois grupos \
             separados. Se os oito forem quadrados, PARE -- o eixo Y nao chegou.\n  \
             O no 'Scale' ja esta selecionado.\n  \
             TESTE 1 (o link): marque 'Uniform'. O 'Scale Y' SOME da lista (um controle \
             que nao faz nada nao e pintado) e as formas viram quadrados de {STRETCH_X}x. \
             Desmarque: o 'Scale Y' volta com o valor que estava.\n  \
             TESTE 2 (squash & stretch): com 'Uniform' desmarcado, arraste o 'Scale Y' \
             de 0.2 ate 3. As formas ESPREMEM e ESTICAM sem mudar de largura -- era isto \
             que nao existia no grafo.\n  \
             TESTE 3 (a linha de espelho anda): clique o no 'Mirror' e arraste o \
             'Axis Offset'. O grupo espelhado desliza; em 0 os dois grupos se encostam \
             no centroide, que era o UNICO lugar possivel ate esta wave.\n  \
             TESTE 4 (o eixo): no 'Mirror', troque 'Axis' para 'Horizontal'. O gemeo \
             passa a nascer acima/abaixo, e o mesmo 'Axis Offset' agora anda em Y."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_node_registry::NodeRegistry;
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;

    fn cooked() -> ph2d_nodegraph::attr::Stream {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
        let mut g = Graph::new();
        let (out, _) = chain(&mut g);
        g.validate(&reg).expect("smoke scene is well-typed");
        let mut cook = Cook::new();
        let set = cook.cook(&g, &reg, out, 0.0).expect("scene cooks");
        set.iter().next().expect("one stream").as_stream().clone()
    }

    /// **A cena MOSTRA o que a mensagem promete** — as duas metades, medidas na saída.
    ///
    /// ⚠️ Um smoke que descreve a tela é um smoke que envelhece: mude um default e a
    /// prosa passa a mentir com a suíte verde. O oráculo aqui não são valores crus mas as
    /// duas GRANDEZAS que a wave nomeia — a razão de aspecto (que nenhum fator uniforme
    /// move) e a posição da linha de espelho (que o centroide sozinho não alcança).
    #[test]
    fn the_transform_smoke_scene_shows_both_halves() {
        let s = cooked();
        assert_eq!(s.count(), 8, "4 pontos espelhados = 8 instancias");

        let Some(Column::Vec2(size)) = s.get("size") else {
            panic!("size")
        };
        let aspect = size[0][0] / size[0][1];
        let want = STRETCH_X / STRETCH_Y;
        assert!(
            (aspect - want).abs() < 1e-4,
            "as formas tinham de sair com razao de aspecto {want} e sairam com {aspect} \
             — em 1.0 o eixo Y nao chegou e a cena mostra quadrados"
        );

        let Some(Column::Vec2(p)) = s.get("P") else {
            panic!("P")
        };
        // A linha de espelho e o ponto medio entre um original e o seu gemeo (o grid tem
        // 4 pontos, entao o gemeo de `i` esta em `i + 4`).
        let line = (p[0][0] + p[4][0]) / 2.0;
        let centroid = p[..4].iter().map(|q| q[0]).sum::<f32>() / 4.0;
        assert!(
            (line - (centroid + MIRROR_OFFSET)).abs() < 1e-4,
            "a linha de espelho tinha de estar a {MIRROR_OFFSET} do centroide \
             ({centroid}) e o cook a poe em {line} — em cima do centroide o offset foi \
             descartado"
        );
    }

    /// **O painel OFERECE o `Scale Y` só com o link destravado** — o TESTE 1 da mensagem.
    ///
    /// A cena nasce destravada, então a row tem de existir; travar o `uniform` tem de
    /// removê-la. É a metade que só o `ParamGate` responde, e ela é lida pela porta do
    /// PRODUTO (o snapshot que o painel pinta), nunca pela tabela de gates.
    #[test]
    fn the_second_axis_row_appears_only_when_the_link_is_off() {
        use crate::motion_state::MotionState;
        use crate::render_loop::motion_bridge::params::build_params_snapshot;
        use ph2d_editor::ProjectSettings;
        use ph2d_panel_motion_params::ParamRow;

        let mut motion = MotionState::new();
        let (_, heroes) = chain(&mut motion.doc.graph);
        let scale = heroes[1];
        ph2d_panel_motion_graph::set_graph_selection(vec![scale.0]);

        let has_row = |motion: &MotionState| {
            build_params_snapshot(motion, ProjectSettings::default())
                .expect("the scale is resolvable")
                .rows
                .iter()
                .any(|r| matches!(r, ParamRow::Scalar(s) if s.name == "amount_y"))
        };
        assert!(has_row(&motion), "destravado, o `Scale Y` e pintado");

        motion.doc.graph.set_param(scale, "uniform", 1.0);
        assert!(
            !has_row(&motion),
            "travado, o `Scale Y` nao e lido — e um controle que nao faz nada nao e \
             pintado"
        );
        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    }
}
