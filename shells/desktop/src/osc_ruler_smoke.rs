//! **A cena do smoke da RÉGUA DE TEMPO e do LOOP** (`PH2D_OSC_RULER_SMOKE=1`, doc 88 B3).
//!
//! Duas fileiras, uma por feature, para as duas serem julgadas na MESMA tela:
//!
//! ```text
//! motion.grid → motion.scale → motion.oscillator(BPM) → motion.output   (de cima)
//! motion.grid → motion.scale → motion.noise(loop)     → motion.output   (de baixo)
//! ```
//!
//! **A de cima é o BPM.** O oscilador ganhou `Time Mode: Seconds | BPM` — a MESMA frequência
//! noutra régua. A fileira anda a **120 BPM = 2 Hz**, e a prova de que a régua é uma unidade e
//! não um segundo multiplicador é que trocar para `Seconds` e digitar `2` dá exatamente o mesmo
//! movimento. ⚠️ Só a régua ESCOLHIDA aparece no painel: em BPM o slider de Hz não é oferecido
//! (os dois na tela seriam dois números discordando sobre um valor).
//!
//! **A de baixo é o LOOP do ruído.** `Loop Length = 3 s` faz o campo fechar o ciclo: aos 3
//! segundos ele volta a ser exatamente o que era no zero. Sem isso o ruído evolui para sempre
//! e um take nunca emenda consigo mesmo.
//!
//! TESTE (a timeline abre junto — o relógio é a régua das duas):
//! 1. **Play.** A fileira de cima oscila no ritmo; a de baixo ondula.
//! 2. Selecione o `Oscillator` → seção **TIMING** → troque `Time Mode` para **Seconds** e
//!    digite `2` em Frequency: o movimento é o MESMO. Volte para BPM e suba para 240: dobra.
//! 3. Selecione o `Noise` → seção **TIMING** → `Loop Length`. Com **3**, pare o playhead em
//!    0,0 e depois em 3,0: a onda de baixo está na MESMA forma. Com **0**, os dois instantes
//!    são diferentes — o ruído nunca volta.
//! 4. ⚠️ **Nada aqui pode congelar.** Se alguma fileira parar sozinha com o relógio andando,
//!    PARE: é a doença que o `fade` do oscilador tinha (um controle que expira).

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Os BPM da cena: 120 é o meio do slider e vale exatamente 2 Hz — o número que torna o
/// passo 2 do roteiro uma comparação, não uma impressão.
const BPM: f32 = 120.0;
/// O ciclo do ruído, em segundos. Curto o bastante para o artista ver a volta sem esperar.
const LOOP_LEN: f32 = 3.0;

/// Monta as duas fileiras. Devolve `(sinks, heroes)` — os dois `motion.output` e os dois nós a
/// selecionar (o oscilador primeiro, que é o do passo 2).
fn scene(g: &mut Graph) -> (Vec<NodeId>, Vec<NodeId>) {
    let mut row = |y: f32, hero_ty: &str, tune: &dyn Fn(&mut Graph, NodeId)| {
        let grid = g.add_node("motion.grid");
        let scale = g.add_node("motion.scale");
        let hero = g.add_node(hero_ty);
        let mv = g.add_node("motion.move");
        let out = g.add_node("motion.output");
        g.set_param(grid, "rows", 1.0);
        g.set_param(grid, "cols", 28.0);
        g.set_param(grid, "gap_x", 0.55);
        g.set_param(scale, "amount", 0.22);
        g.set_param(mv, "dy", y);
        tune(g, hero);
        for (from, to) in [(grid, scale), (scale, hero), (hero, mv), (mv, out)] {
            g.connect(Edge {
                from: (from, 0),
                to: (to, 0),
                delayed: false,
            })
            .expect("osc-ruler-smoke edge");
        }
        (out, hero)
    };

    // De cima: o oscilador na régua de BPM.
    let (out_a, osc) = row(2.2, "motion.oscillator", &|g, n| {
        g.set_param(n, "channel", 1.0); // Y
        g.set_param(n, "amplitude", 1.4);
        g.set_param(n, "phase_stagger", 0.06);
        g.set_param(n, "time_mode", 1.0); // BPM
        g.set_param(n, "bpm", BPM);
    });
    // De baixo: o ruído com o ciclo fechado.
    let (out_b, noise) = row(-2.2, "motion.noise", &|g, n| {
        g.set_param(n, "channel", 1.0); // Y
        g.set_param(n, "amplitude", 1.4);
        g.set_param(n, "scale", 0.55);
        g.set_param(n, "speed", 1.0);
        g.set_param(n, "loop_len", LOOP_LEN);
    });
    (vec![out_a, out_b], vec![osc, noise])
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_OSC_RULER_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `gradient_smoke`. No-op sem a env.
    pub(crate) fn osc_ruler_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let (sinks, heroes) = scene(&mut gfx.motion.doc.graph);
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.extend(sinks);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        // A RÉGUA. As duas features desta cena são sobre TEMPO, e o roteiro manda parar o
        // playhead em 0,0 e depois em 3,0 -- sem a timeline o artista não tem onde fazer
        // isso. O `Espaco` toca sem painel nenhum; o que só a timeline oferece é a RÉGUA
        // (a mesma linha que o prólogo do `physics_smoke` carrega, pelo mesmo motivo).
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("timeline", true);
        }
        // O oscilador já selecionado: a seção TIMING é o passo 2 do roteiro.
        ph2d_panel_motion_graph::request_graph_selection(vec![heroes[0].0]);
        eprintln!(
            "[osc-ruler smoke] Duas fileiras de 28 pontos. EM CIMA o Oscillator na regua de \
             BPM ({BPM} BPM = 2 Hz exatos); EMBAIXO o Noise com o ciclo FECHADO em \
             {LOOP_LEN} s.\n  \
             1) Play: a de cima oscila no ritmo, a de baixo ondula.\n  \
             2) O 'Oscillator' ja esta selecionado -> secao TIMING. Troque Time Mode para \
             Seconds e digite 2 em Frequency: o movimento e o MESMO (a regua e uma UNIDADE, \
             nao um segundo multiplicador). Volte para BPM e suba para 240: dobra. Note que \
             so a regua ESCOLHIDA aparece -- em BPM o slider de Hz nao e oferecido.\n  \
             3) Selecione o 'Noise' -> secao TIMING -> Loop Length. Com {LOOP_LEN}, pare o \
             playhead em 0,0 e depois em {LOOP_LEN},0: a onda de baixo esta na MESMA forma. \
             Com 0, os dois instantes sao diferentes -- o ruido nunca volta.\n  \
             4) ATENCAO: nada aqui pode CONGELAR. Se uma fileira parar sozinha com o relogio \
             andando, PARE e reporte."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_node_registry::NodeRegistry;
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;

    fn ys(g: &Graph, reg: &NodeRegistry, sink: NodeId, t: f64) -> Vec<f32> {
        let mut cook = Cook::new();
        let out = cook.cook(g, reg, sink, t).expect("smoke scene cooks");
        match out[0].as_stream().get("P").expect("P") {
            Column::Vec2(v) => v.iter().map(|p| p[1]).collect(),
            _ => panic!("P is Vec2"),
        }
    }

    /// **A cena AFIRMA duas coisas na tela, e as duas são cozidas aqui antes de a mensagem
    /// existir** — a política do plano (*"cena com números MEDIDOS, sonda headless ANTES da
    /// mensagem"*). Uma cena que promete um movimento que ela não produz é pior que cena
    /// nenhuma: o artista reporta a feature como quebrada.
    #[test]
    fn the_smoke_scene_really_shows_the_ruler_and_the_loop() {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
        let mut g = Graph::new();
        let (sinks, _heroes) = scene(&mut g);
        g.validate(&reg).expect("a cena e bem-tipada");

        // (0) A PREMISSA, declarada. O ciclo é a régua da metade de baixo deste gate, e com
        // `LOOP_LEN = 0` a asserção do loop compararia `t` com `t + 0` -- o campo contra ELE
        // MESMO, verde por construção sobre uma cena que não fecha ciclo nenhum. Medido: essa
        // mutação passava. Um gate cujo oráculo se dissolve com o valor da fixture tem de
        // declarar a fixture.
        const {
            assert!(
                LOOP_LEN > 0.5,
                "a cena tem de ter um ciclo que o artista consegue ver: baixar LOOP_LEN a zero \
                 faria o gate do loop comparar o campo com ELE MESMO"
            );
        }

        // (1) A fileira de cima se MOVE — sem isto o resto da cena não diz nada.
        let a0 = ys(&g, &reg, sinks[0], 0.0);
        let a1 = ys(&g, &reg, sinks[0], 0.12);
        assert_ne!(a0, a1, "a fileira do oscilador tem de oscilar");

        // (2) O LOOP fecha: a fileira de baixo em `t` e em `t + LOOP_LEN` é a mesma. É o
        // passo 3 do roteiro, medido — e ele falharia com o `loop_len` desligado.
        let l = f64::from(LOOP_LEN);
        for k in 0..8 {
            let t = f64::from(k) * 0.25;
            let b0 = ys(&g, &reg, sinks[1], t);
            let b1 = ys(&g, &reg, sinks[1], t + l);
            let dev = b0
                .iter()
                .zip(&b1)
                .map(|(x, y)| (x - y).abs())
                .fold(0.0f32, f32::max);
            assert!(
                dev < 5e-6,
                "o ciclo do ruido nao fecha em t={t}: desvio {dev}"
            );
        }
        // E dentro do ciclo ele MUDA (um campo congelado fecharia o ciclo trivialmente).
        assert_ne!(ys(&g, &reg, sinks[1], 0.0), ys(&g, &reg, sinks[1], 1.0));

        // O CONTROLE, e é ele que torna a asserção acima uma medição em vez de aritmética:
        // com o ciclo DESLIGADO o mesmo par de instantes tem de DIFERIR. É o passo 3 do
        // roteiro (`Loop Length` em 0 => o ruído nunca volta), e sem ele um gate que
        // comparasse qualquer coisa consigo mesma passaria igual.
        let mut off = Graph::new();
        let (sinks_off, heroes_off) = scene(&mut off);
        off.set_param(heroes_off[1], "loop_len", 0.0);
        let (c0, c1) = (
            ys(&off, &reg, sinks_off[1], 0.0),
            ys(&off, &reg, sinks_off[1], l),
        );
        let dev_off = c0
            .iter()
            .zip(&c1)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max);
        assert!(
            dev_off > 1e-3,
            "sem loop o campo nao pode voltar ao mesmo lugar em t+{LOOP_LEN}: desvio {dev_off}"
        );

        // (3) A régua é uma UNIDADE: 120 BPM e 2 Hz produzem o MESMO movimento, que é
        // exatamente o que o passo 2 manda o artista conferir na tela.
        let mut g2 = Graph::new();
        let (sinks2, heroes2) = scene(&mut g2);
        g2.set_param(heroes2[0], "time_mode", 0.0);
        g2.set_param(heroes2[0], "frequency", BPM / 60.0);
        for k in 0..8 {
            let t = f64::from(k) * 0.13;
            assert_eq!(
                ys(&g, &reg, sinks[0], t),
                ys(&g2, &reg, sinks2[0], t),
                "120 BPM tem de ser 2 Hz em t={t} -- o passo 2 do roteiro pede isso na tela"
            );
        }
    }
}
