//! **A cena pronta para o smoke das UNIDADES de param** (`PH2D_UNITS_SMOKE=1`, doc 88).
//!
//! Até esta wave todo número do painel de Motion era um número NU. Um `Gap X` de `0.9`
//! não dizia se eram nove décimos de quê, e a resposta — nove décimos de METRO, ou seja
//! 90 px — não estava em lugar nenhum da tela. A wave dá a cada param uma unidade
//! DECLARADA e resolve a face na fronteira do painel, uma vez.
//!
//! ```text
//! motion.emitter -> motion.twist -> motion.output
//! ```
//!
//! Os dois nós são heróis de propósito, porque juntos exercitam as três faces:
//!
//! - **`motion.emitter`** — `Origin X` / `Origin Y` / `Size` em **px** (comprimentos de
//!   mundo, guardados em metros), `Life` em **s** (segundos são guardados na unidade em
//!   que são mostrados, então `pixels_per_meter` nunca os alcança) e `Rate` / `Speed`
//!   NUS. ⚠️ O `Speed` está nu **de propósito**: ele é metros por segundo, e este
//!   vocabulário não tem velocidade — `px` num ritmo seria um rótulo que ensina algo
//!   falso.
//! - **`motion.twist`** — `Angle` em **deg** e `Pivot X` / `Pivot Y` em **px**, o par que
//!   mostra que a unidade é por-PARAM e não por-nó.
//!
//! E o `Life` é onde o **slider duplo** aparece: o arrasto começa em `0.1 s` (a faixa
//! útil de uma fonte) e a CAIXA alcança `0.001 s` (a faísca) — o piso duro que não
//! existia antes desta wave, porque só havia o teto.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Monta `emitter -> twist -> output`. Devolve `(sink, [emitter, twist])`.
///
/// Os valores são os que o artista LÊ na face de fábrica (Pixels, 100 px/m), escritos
/// aqui em METROS porque o documento é sempre métrico: `0.5 m` de origem lê `50 px`,
/// `0.15 m` de tamanho lê `15 px`.
fn chain(g: &mut Graph) -> (NodeId, [NodeId; 2]) {
    let em = g.add_node("motion.emitter");
    let tw = g.add_node("motion.twist");
    let out = g.add_node("motion.output");

    g.set_param(em, "rate", 40.0);
    g.set_param(em, "life", 3.0);
    g.set_param(em, "speed", 1.5);
    g.set_param(em, "x", -0.5);
    g.set_param(em, "y", 0.0);
    g.set_param(em, "size", 0.15);
    g.set_param(tw, "angle", 90.0);

    for (from, to) in [(em, tw), (tw, out)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .expect("units-smoke edge");
    }
    (out, [em, tw])
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_UNITS_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `gradient_smoke`. No-op sem a env.
    pub(crate) fn units_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let (sink, heroes) = chain(&mut gfx.motion.doc.graph);
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.push(sink);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        // O emitter já selecionado: as três faces (px / s / nu) na tela no 1º frame.
        ph2d_panel_motion_graph::request_graph_selection(vec![heroes[0].0]);
        // ⚠️ A escala do projeto é IMPRESSA, não assumida: ela viaja no arquivo desde
        // esta wave, então um projeto carregado pode ter outra, e todo numero abaixo
        // é derivado dela. Um smoke que afirmasse "100" seria um smoke que mente no
        // dia em que o artista mexer no Settings.
        let ppm = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map_or(100.0, |h| h.project.pixels_per_meter);
        eprintln!(
            "[units smoke] emitter -> twist -> output, com a escala do projeto em \
             {ppm} px/m.\n  \
             O no 'Emitter' ja esta selecionado. No painel de params (a direita) as rows \
             tem de LER:\n    \
             Origin X = -50 px   Origin Y = 0 px   Size = 15 px   Life = 3 s\n    \
             Rate = 40   Speed = 1.5   (NUS de proposito: um ritmo nao e um comprimento)\n  \
             TESTE 1 (a face segue o PROJETO): abra Settings -> Display unit -> Meters. \
             Toda row de px vira 'm' e o numero divide por {ppm} (Origin X = -0.5 m), \
             e A CENA NAO SE MEXE -- so a leitura mudou. Volte para Pixels.\n  \
             TESTE 2 (o slider DUPLO): o arrasto do 'Life' comeca em 0.1 s, mas DIGITE \
             0.001 na caixa -- ela aceita. O piso duro nao existia antes desta wave.\n  \
             TESTE 3 (a unidade e por-PARAM): clique o no 'Twist'. 'Angle' le em 'deg' e \
             'Pivot X'/'Pivot Y' leem em 'px', no MESMO no."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_node_registry::NodeRegistry;
    use ph2d_nodegraph::cook::Cook;

    /// A cena é bem-tipada e cozinha — um smoke cuja cena não cozinha mostra tela vazia,
    /// que é o que o artista reportaria como "o smoke está quebrado".
    #[test]
    fn the_units_smoke_scene_cooks() {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
        let mut g = Graph::new();
        let (out, _) = chain(&mut g);
        g.validate(&reg).expect("smoke scene is well-typed");
        let mut cook = Cook::new();
        let set = cook.cook(&g, &reg, out, 1.0).expect("scene cooks");
        assert!(
            set.iter().next().is_some_and(|s| s.as_stream().count() > 0),
            "the emitter is alive at t=1s — an empty set is a blank screen"
        );
    }

    /// **Os números que a mensagem PROMETE são os que o painel pinta.**
    ///
    /// Um smoke que descreve a tela é um smoke que envelhece: mude um default e a prosa
    /// passa a mentir, com a suíte verde. Este gate lê as rows pela porta do PRODUTO e
    /// confere cada número citado — inclusive o `Speed` NU, que é a única linha da
    /// mensagem que afirma uma AUSÊNCIA.
    #[test]
    fn the_message_promises_the_numbers_the_panel_paints() {
        use crate::motion_state::MotionState;
        use crate::render_loop::motion_bridge::params::build_params_snapshot;
        use ph2d_editor::ProjectSettings;
        use ph2d_panel_motion_params::ParamRow;

        let mut motion = MotionState::new();
        let (_, heroes) = chain(&mut motion.doc.graph);
        ph2d_panel_motion_graph::set_graph_selection(vec![heroes[0].0]);
        let rows = build_params_snapshot(&motion, ProjectSettings::default())
            .expect("the emitter is resolvable")
            .rows;
        let row = |name: &str| {
            rows.iter()
                .find_map(|r| match r {
                    ParamRow::Scalar(s) if s.name == name => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("no scalar row `{name}`"))
        };
        for (name, want, suffix) in [
            ("x", -50.0, "px"),
            ("y", 0.0, "px"),
            ("size", 15.0, "px"),
            ("life", 3.0, "s"),
            ("rate", 40.0, ""),
            ("speed", 1.5, ""),
        ] {
            let r = row(name);
            assert!(
                (r.value - want).abs() <= 1e-4,
                "{name}: panel shows {}, the message promises {want}",
                r.value
            );
            assert_eq!(r.display.suffix, suffix, "{name}'s face");
        }
        // TESTE 2: o arrasto começa em 0.1 s e a caixa alcança 0.001 — o slider duplo.
        let life = row("life");
        assert!((life.min - 0.1).abs() <= 1e-6 && (life.hard_min - 0.001).abs() <= 1e-6);

        // TESTE 3: a unidade é por-param DENTRO de um nó.
        ph2d_panel_motion_graph::set_graph_selection(vec![heroes[1].0]);
        let tw = build_params_snapshot(&motion, ProjectSettings::default())
            .expect("the twist is resolvable")
            .rows;
        let face = |name: &str| {
            tw.iter()
                .find_map(|r| match r {
                    ParamRow::Scalar(s) if s.name == name => Some(s.display.suffix),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("no scalar row `{name}`"))
        };
        assert_eq!(face("angle"), "deg");
        assert_eq!(face("pivot_x"), "px");
        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    }
}
