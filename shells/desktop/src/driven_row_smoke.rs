//! **A cena do smoke da ROW DIRIGIDA** (`PH2D_DRIVEN_ROW_SMOKE=1`, doc 88 B3).
//!
//! Um param puxado por um fio não é editável — o número vem de fora. Até esta wave a row
//! dizia só isso, com um acento: um valor que não obedece ao dedo e nenhuma palavra sobre a
//! procedência. Agora ela mostra o **elo** e o **nome do card** que a dirige.
//!
//! ```text
//!   motion.grid → motion.scale → motion.oscillator → motion.move → motion.output
//!                                     ↑ amplitude        ↑ frequency
//!                 value.lfo "Batida" ─┘                  │
//!                 value.lfo ─→ value.gain ─→ value.map_range ─┘
//! ```
//!
//! ⚠️ **A cadeia do segundo fio é a que a doc do `value.gain` PRESCREVE**, e a v1 desta cena
//! a ignorou: o gain é o shaper de Schlick, opera na banda `[0,1]` e **CLAMPA a entrada** —
//! um LFO em `[1,2]` colapsa todo em `1.0`, que é ponto fixo das duas formas, então a saída
//! ficava **constante e imune ao `strength`** (medido: `1.000` em toda strength e em todo
//! instante — o fio não movia a cena, e o Enio reportou exatamente isso). O LFO entra
//! normalizado em `[0,1]`, o gain molda, e o `value.map_range` **põe o resultado na faixa
//! que a frequência quer** (0,6 a 2,4 Hz).
//!
//! **Um nó, três estados de row na MESMA tela** — é isto que a cena existe para pôr lado a
//! lado, porque o valor da feature é a DIFERENÇA:
//!
//! - **`Amplitude`** é dirigida por um nó que o artista NOMEOU → a row lê **Batida**;
//! - **`Frequency`** é dirigida por um nó sem nome próprio → a row lê **Map Range**, que é o
//!   que o card dele diz (o degrau do tipo);
//! - **`Wave`, `Phase`, `Offset`…** seguem knobs comuns → o CONTROLE. Sem elas na mesma tela,
//!   "a row dirigida é diferente" é uma afirmação sem contraparte.
//!
//! E as duas dirigidas se veem no canvas: a fileira pulsa (a amplitude respira no ritmo da
//! Batida) e acelera e desacelera (a frequência passeia pela cadeia do Gain). Um fio que não
//! muda nada provaria a UI e não o produto.
//!
//! TESTE:
//! 1. **Play.** A fileira ondula: a altura respira e o ritmo passeia.
//! 2. O `Oscillator` já está selecionado. No painel de params: `Amplitude` e `Frequency` não
//!    têm knob — têm o número em destaque, o **elo** e um NOME à direita (**Batida** e
//!    **Map Range**). As outras rows continuam com slider: é a diferença que se julga.
//! 3. **O nome segue o rename.** Clique no card `Batida` no grafo, `F2`, chame de `Coracao`,
//!    Enter. A row `Amplitude` passa a ler **Coracao** — sem tocar no inspector.
//! 4. **O knob do DRIVER move a cena.** Clique no card `Gain` e arraste o `Strength`: em `-1`
//!    o ritmo fica UNIFORME (o passeio achata no meio), em `+1` ele ALTERNA entre nitidamente
//!    lento e nitidamente rápido (o passeio vai aos extremos). O número da row `Frequency` do
//!    oscilador acompanha ao vivo.
//! 5. ⚠️ **Se alguma das duas rows mostrar um slider, PARE**: um knob que gira enquanto um fio
//!    decide o valor é um controle que mente uma vez por arrasto.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O nome que o ARTISTA deu ao driver da amplitude — o degrau de cima da escada do
/// `card_title`. A cena já o aplica para que os DOIS degraus estejam na tela de uma vez;
/// o passo 3 do roteiro troca este nome e olha a row.
const BEAT_NAME: &str = "Batida";

/// Monta a cena. Devolve `(sink, [oscillator, driver_da_amplitude])` — o herói é o oscilador
/// (é o nó cujo painel se julga) e o segundo é o card que o passo 3 renomeia.
fn scene(g: &mut Graph) -> (NodeId, [NodeId; 2]) {
    let grid = g.add_node("motion.grid");
    let scale = g.add_node("motion.scale");
    let osc = g.add_node("motion.oscillator");
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    // Os dois drivers. Ambos com a porta `in` SOLTA de propósito: um `value.lfo` sem entrada
    // devolve um valor global, que é exatamente o que um param quer (um número, não um campo
    // por elemento).
    let beat = g.add_node("value.lfo");
    let wobble = g.add_node("value.lfo");
    let gain = g.add_node("value.gain");
    let fit = g.add_node("value.map_range");

    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 28.0);
    g.set_param(grid, "gap_x", 0.55);
    g.set_param(scale, "amount", 0.22);
    g.set_param(osc, "channel", 1.0); // Y
    g.set_param(osc, "phase_stagger", 0.06);

    // A BATIDA: respira entre 0,2 e 2,0 de amplitude, num ciclo de 2 s — lento o bastante
    // para o olho seguir, rápido o bastante para não esperar.
    g.set_param(beat, "period", 2.0);
    g.set_param(beat, "amplitude", 0.9);
    g.set_param(beat, "offset", 1.1);
    g.set_label(beat, BEAT_NAME);

    // O PASSEIO. O LFO entra **NORMALIZADO em `[0,1]`** — é a banda em que o gain vive, e
    // alimentá-lo fora dela é o que matava este fio (ver o ⚠️ do topo).
    // ⚠️ A banda é `[0,05, 0,95]`, **não** `[0,1]`: os extremos `0` e `1` são PONTOS FIXOS das
    // duas formas de Schlick, então um LFO que os toca escapa do achatamento e crava um
    // solavanco de um quadro a cada ciclo — correto no modelo, e num smoke lê como defeito.
    g.set_param(wobble, "period", 5.0);
    g.set_param(wobble, "amplitude", 0.45);
    g.set_param(wobble, "offset", 0.5);
    // O GAIN nasce NEUTRO (`strength = 0` é a identidade bit-exata) — a cena entrega o fio
    // vivo e o knob intocado, então o PRIMEIRO gesto do artista é o discriminante. Semeá-lo
    // já torcido esconderia justamente a diferença que ele existe para mostrar.
    g.set_param(gain, "strength", 0.0);
    // E o MAP RANGE põe o `[0,1]` moldado na faixa que a frequência quer: 0,6 a 2,4 Hz —
    // lento e rápido o bastante para o olho separar, e **sem passar por zero** (frequência 0
    // é a fileira parada, que lê como defeito).
    g.set_param(fit, "out_lo", 0.6);
    g.set_param(fit, "out_hi", 2.4);

    for (from, to) in [(grid, scale), (scale, osc), (osc, mv), (mv, out)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .expect("driven-row-smoke edge");
    }
    for (from, to) in [(wobble, gain), (gain, fit)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .expect("driven-row-smoke value edge");
    }
    g.drive_param(osc, "amplitude", (beat, 0))
        .expect("a batida dirige a amplitude");
    g.drive_param(osc, "frequency", (fit, 0))
        .expect("o map range dirige a frequencia");
    (out, [osc, beat])
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_DRIVEN_ROW_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `gradient_smoke`. No-op sem a env.
    pub(crate) fn driven_row_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let (sink, heroes) = scene(&mut gfx.motion.doc.graph);
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.push(sink);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        // O oscilador já selecionado: o painel de params É o objeto do smoke.
        ph2d_panel_motion_graph::request_graph_selection(vec![heroes[0].0]);
        eprintln!(
            "[driven-row smoke] Uma fileira de 28 pontos, e UM no com os TRES estados de row \
             na mesma tela.\n  \
             1) Play: a altura respira (Amplitude dirigida pela '{BEAT_NAME}') e o ritmo \
             passeia (Frequency dirigida pelo 'Map Range').\n  \
             2) O 'Oscillator' ja esta selecionado. No painel: Amplitude e Frequency NAO tem \
             knob -- tem o numero em destaque, o elo, e o NOME de quem as dirige ('{BEAT_NAME}' \
             e 'Map Range'). As outras rows seguem com slider: e essa a diferenca que se julga.\n  \
             3) O nome SEGUE o rename: clique no card '{BEAT_NAME}' no grafo, F2, chame de \
             'Coracao', Enter -- a row Amplitude passa a ler Coracao, sem tocar no inspector.\n  \
             4) O knob do DRIVER move a cena: clique no card 'Gain' e arraste o Strength. Em \
             -1 o ritmo fica UNIFORME; em +1 ele ALTERNA entre lento e rapido. O numero da row \
             Frequency acompanha ao vivo.\n  \
             5) Se alguma das duas mostrar um SLIDER, pare: um knob que gira enquanto um fio \
             decide o valor mente uma vez por arrasto."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion_state::MotionState;
    use crate::render_loop::motion_bridge::params::{
        build_params_snapshot, driven_value_for_probe,
    };
    use ph2d_editor::ProjectSettings;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_panel_motion_params::ParamRow;

    /// A cena é bem-tipada e cozinha — um smoke cuja cena não cozinha mostra tela vazia, que é
    /// o que o artista reportaria como "o smoke está quebrado".
    #[test]
    fn the_driven_row_scene_cooks() {
        let st = MotionState::new();
        let mut g = Graph::new();
        let (out, _) = scene(&mut g);
        g.validate(&st.registry).expect("a cena é bem-tipada");
        let mut cook = Cook::new();
        let set = cook
            .cook(&g, &st.registry, out, 0.5)
            .expect("a cena cozinha");
        assert!(
            set.iter().next().is_some_and(|s| s.as_stream().count() > 0),
            "a fileira existe em t=0,5s — um conjunto vazio é uma tela em branco"
        );
    }

    /// **E o fio MUDA a cena** — a metade que impede este smoke de provar a UI em vez do
    /// produto.
    ///
    /// Se a amplitude dirigida não movesse nada, o painel poderia estar perfeito sobre um fio
    /// inerte e o roteiro ("a altura respira") seria uma frase sem contraparte na tela.
    /// Medido no ciclo de 2 s da Batida: a extensão em Y da fileira vai de **0,396 a 3,961**
    /// — **10×**. A barra é folgada de propósito (um fator 3), porque o número exato é do
    /// ajuste da cena e não da feature; o que não pode acontecer é a fileira ficar parada.
    #[test]
    fn the_wire_actually_moves_the_scene() {
        let st = MotionState::new();
        let mut g = Graph::new();
        let (out, _) = scene(&mut g);
        let mut cook = Cook::new();
        let extent = |cook: &mut Cook, t: f64| -> f32 {
            let set = cook.cook(&g, &st.registry, out, t).expect("cozinha");
            let stream = set.iter().next().expect("um sink").as_stream();
            let ys: Vec<f32> = match stream.get("P") {
                Some(ph2d_nodegraph::attr::Column::Vec2(p)) => p.iter().map(|v| v[1]).collect(),
                _ => Vec::new(),
            };
            ys.iter().copied().fold(f32::NEG_INFINITY, f32::max)
                - ys.iter().copied().fold(f32::INFINITY, f32::min)
        };
        // O pico e o vale da respiração, meio ciclo apartados.
        let (hi, lo) = (extent(&mut cook, 0.5), extent(&mut cook, 1.5));
        assert!(
            hi > lo * 3.0,
            "a amplitude dirigida tem de RESPIRAR: pico {hi:.3} contra vale {lo:.3} — um fio \
             que não muda a cena provaria o painel, não o produto"
        );
    }

    /// A trajetória que o fio de fato entrega na `frequency`, um valor por instante ao longo
    /// de um ciclo do passeio — lida pela porta do **PRODUTO** (`driven_value`, o memo do
    /// cook), que é exatamente a que a row dirigida mostra ao artista.
    fn frequency_trajectory(strength: f32) -> Vec<f32> {
        let mut motion = MotionState::new();
        let (sink, _) = scene(&mut motion.doc.graph);
        let find = |ty: &str| {
            motion
                .doc
                .graph
                .nodes()
                .iter()
                .find(|n| n.type_name == ty)
                .unwrap_or_else(|| panic!("a cena tem um {ty}"))
                .id
        };
        let (gain, osc) = (find("value.gain"), find("motion.oscillator"));
        motion.doc.graph.set_param(gain, "strength", strength);
        (1..=12)
            .map(|k| {
                let t = f64::from(k) * 0.25;
                let (graph, reg) = (&motion.doc.graph, &motion.registry);
                motion.pump.cook.cook(graph, reg, sink, t).expect("cozinha");
                driven_value_for_probe(&motion, osc, "frequency").map_or(f32::NAN, |(v, _)| v)
            })
            .collect()
    }

    /// **O fio da FREQUÊNCIA anda** — a metade que faltava, e a que deixou um fio MORTO
    /// atravessar o commit.
    ///
    /// ⚠️ O irmão `the_wire_actually_moves_the_scene` mede a **extensão em Y**, que só o fio da
    /// AMPLITUDE move; ele ficou verde enquanto a `frequency` entregue era **1,000 constante em
    /// todo instante** (medido). *Um gate que mede uma metade fica verde sobre a outra morta.*
    /// A causa era a cena, não o produto: o `value.gain` clampa em `[0,1]` e um LFO em `[1,2]`
    /// colapsa todo no ponto fixo `1.0`.
    #[test]
    fn the_frequency_wire_walks_over_the_cycle() {
        let f = frequency_trajectory(0.0);
        let lo = f.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = f.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        // Medido: 1,024 a 2,310 = 1,286 de excursão. A barra é folgada de propósito (o número
        // exato é do ajuste da cena); o que não pode acontecer é a frequência ficar PARADA.
        assert!(
            hi - lo > 0.5,
            "o ritmo tem de PASSEAR: a frequência dirigida foi de {lo:.3} a {hi:.3} — um fio \
             que entrega sempre o mesmo número não move a cena"
        );
        assert!(
            lo > 0.0,
            "e nunca chega a zero, que é a fileira parada: {lo:.3}"
        );
    }

    /// **E o knob do DRIVER dirige esse fio** — o report do Enio (2026-08-07: *"mudar os
    /// parâmetros de gain não parecem ter nenhum efeito"*), afirmado no número que a row mostra.
    ///
    /// É pergunta distinta da do irmão acima: um `value.gain` inerte com o `value.map_range`
    /// vivo passaria lá e falharia aqui. Medido: entre `strength ±1` o mesmo instante muda até
    /// **0,900** (o ritmo sai de uniforme a 1,5 Hz para alternar entre 0,6 e 2,4).
    #[test]
    fn the_drivers_knob_steers_the_wire() {
        let (flat, sharp) = (frequency_trajectory(-1.0), frequency_trajectory(1.0));
        let worst = flat
            .iter()
            .zip(&sharp)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            worst > 0.3,
            "o Strength do Gain tem de MUDAR o que o fio entrega: a maior diferença entre \
             strength -1 e +1 foi {worst:.3} — um knob de driver que não move o número é o \
             controle que o artista reportou como morto"
        );
    }

    /// **A SONDA DO REPORT** (*"mudar os parâmetros de gain não parecem ter nenhum efeito"*):
    /// varre a `strength` do `value.gain` e imprime o número que o fio de fato põe na
    /// `frequency` — pela porta do PRODUTO (`driven_value`, o memo do cook), nunca por uma
    /// segunda avaliação.
    /// `cargo test -p ph2d-host-desktop --bins measure_the_gain_knob -- --ignored --nocapture`
    #[test]
    #[ignore = "sonda de medição, não gate"]
    fn measure_the_gain_knob() {
        println!("\n=== strength do Gain -> frequency dirigida (t = 0,25 .. 3,0) ===");
        for &s in &[-1.0f32, -0.5, 0.0, 0.5, 1.0] {
            let mut motion = MotionState::new();
            let (sink, _) = scene(&mut motion.doc.graph);
            let gain = motion
                .doc
                .graph
                .nodes()
                .iter()
                .find(|n| n.type_name == "value.gain")
                .expect("a cena tem um value.gain")
                .id;
            let osc = motion
                .doc
                .graph
                .nodes()
                .iter()
                .find(|n| n.type_name == "motion.oscillator")
                .expect("a cena tem um oscilador")
                .id;
            motion.doc.graph.set_param(gain, "strength", s);
            let mut row = format!("strength {s:+.2} -> ");
            for k in 1..=12 {
                let t = f64::from(k) * 0.25;
                let (graph, reg) = (&motion.doc.graph, &motion.registry);
                motion.pump.cook.cook(graph, reg, sink, t).expect("cozinha");
                let f = driven_value_for_probe(&motion, osc, "frequency");
                row.push_str(&format!("{:.3} ", f.map_or(f32::NAN, |(v, _)| v)));
            }
            println!("{row}");
        }
    }

    /// A SONDA: quanto a altura de fato RESPIRA, e quanto o ritmo passeia.
    /// `cargo test -p ph2d-host-desktop --bins measure_the_driven_scene -- --ignored --nocapture`
    #[test]
    #[ignore = "sonda de medição, não gate"]
    fn measure_the_driven_scene() {
        let st = MotionState::new();
        let mut g = Graph::new();
        let (out, _) = scene(&mut g);
        let mut cook = Cook::new();
        println!("\n=== t, extensao em Y da fileira ===");
        for k in 0..=12 {
            let t = f64::from(k) * 0.25;
            let set = cook.cook(&g, &st.registry, out, t).expect("cozinha");
            let stream = set.iter().next().expect("um sink").as_stream();
            let ys: Vec<f32> = match stream.get("P") {
                Some(ph2d_nodegraph::attr::Column::Vec2(p)) => p.iter().map(|v| v[1]).collect(),
                _ => Vec::new(),
            };
            let lo = ys.iter().copied().fold(f32::INFINITY, f32::min);
            let hi = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            println!("{t:5.2}  {:.3}", hi - lo);
        }
    }

    /// **Os NÚMEROS que a mensagem promete são os que o painel pinta** — os três estados de
    /// row, na porta do produto.
    ///
    /// Um smoke que descreve a tela é um smoke que envelhece: mude um default e a prosa passa a
    /// mentir com a suíte verde. Este lê as rows pelo caminho REAL do bridge e confere as três
    /// afirmações do roteiro, inclusive a do CONTROLE — que é a única que afirma uma AUSÊNCIA
    /// (uma row que continua sendo um knob).
    #[test]
    fn the_message_promises_the_rows_the_panel_paints() {
        let mut motion = MotionState::new();
        let (sink, heroes) = scene(&mut motion.doc.graph);
        motion.sinks.push(sink);
        // ⚠️ **A fixture tem de COZINHAR primeiro.** A row dirigida mostra o número que o fio
        // está pondo ali, e esse número sai do MEMO do cook (nunca de uma segunda avaliação —
        // doc 58). Num estado recém-construído o memo está vazio, então o param ainda não
        // tem dono: o painel mostra o override, honestamente. No app um frame sempre cozinha
        // antes de o painel pintar; aqui isso é dito.
        let (graph, reg) = (&motion.doc.graph, &motion.registry);
        motion
            .pump
            .cook
            .cook(graph, reg, sink, 0.5)
            .expect("a cena cozinha");
        ph2d_panel_motion_graph::set_graph_selection(vec![heroes[0].0]);
        let rows = build_params_snapshot(&motion, ProjectSettings::default())
            .expect("o oscilador resolve")
            .rows;
        let driver_of = |name: &str| -> Option<String> {
            rows.iter().find_map(|r| match r {
                ParamRow::Scalar(s) if s.name == name => s.driven_by.clone(),
                _ => None,
            })
        };
        assert_eq!(
            driver_of("amplitude").as_deref(),
            Some(BEAT_NAME),
            "a Amplitude é dirigida pelo card que o artista nomeou"
        );
        assert_eq!(
            driver_of("frequency").as_deref(),
            Some("Map Range"),
            "e a Frequency pelo card sem nome próprio, que diz o que o nó É"
        );
        // O CONTROLE: alguma row do MESMO nó continua sendo um knob. Sem isto o gate ficaria
        // verde num painel em que TODA row virou dirigida, que é o oposto do que a cena mostra.
        assert!(
            rows.iter().any(|r| matches!(
                r,
                ParamRow::Scalar(s) if s.driven_by.is_none() && s.name != "amplitude"
            )),
            "as demais rows seguem editáveis — é a diferença que a cena existe para mostrar"
        );
        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    }
}
