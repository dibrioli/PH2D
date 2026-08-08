//! **A FONTE — a cena para julgar o curso de `Rate` e `Max Particles`** (`PH2D_EMITTER_SMOKE=1`,
//! doc 88 §11.3).
//!
//! A wave da régua baixou os dois tetos SOFT do `motion.emitter` (`rate` 12.000 → 1.200,
//! `max` 4.194.304 → 4.096) e mandou o teto de antes para o `ParamHardMax`, onde a CAIXA o
//! digita. Nada ficou inalcançável — mas *"o arrasto ficou curto demais para uma cena densa?"*
//! é uma pergunta que **só a mão do artista responde**, e ela precisa de uma fonte na tela.
//!
//! ```text
//! motion.emitter -> motion.integrate -> motion.tint -> motion.output
//!                        ^        |
//!                        |        v (delayed: o estado do tick anterior)
//!                   force.wind (gravidade)
//! ```
//!
//! **A cena é uma FONTE porque os dois knobs se leem em coisas diferentes nela:** o `Rate` é a
//! DENSIDADE do jato (quantas partículas por segundo) e o `Max Particles` é o TETO DA PISCINA
//! (quantas podem estar vivas de uma vez). Um jato ralo não mostra o segundo, e um teto folgado
//! não mostra o primeiro — por isso o roteiro pede o `Rate` primeiro.
//!
//! ⚠️ **Os defaults aqui são os DEFAULTS DO NÓ** (`rate = 40`, `life` encurtada só para a
//! partícula morrer dentro do quadro): a cena tem de abrir mostrando o que um emitter recém-
//! colocado mostra, senão ela julga um ajuste que o artista não tem.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Quanto tempo uma partícula vive. **Não é o default do nó (3 s)**, e a razão é
/// enquadramento: com a gravidade abaixo, uma partícula de 3 s cai muito além da moldura e
/// dois terços do jato ficam fora da tela — o `Rate` pareceria menos denso do que é.
const LIFE_S: f32 = 2.6;

/// O teto da piscina com que a cena ABRE: o topo do slider depois desta wave. Começar no topo
/// é o que deixa o `Rate` ser lido sozinho no TESTE 1 (nada é ceifado), e o que dá ao TESTE 2
/// um lugar de onde descer.
const POOL: f32 = 4_096.0;

/// Monta a fonte. Devolve `(sink, [emitter, integrate])` — o emitter é o herói.
fn chain(g: &mut Graph) -> (NodeId, [NodeId; 2]) {
    let em = g.add_node("motion.emitter");
    let ig = g.add_node("motion.integrate");
    let grav = g.add_node("force.wind");
    let tint = g.add_node("motion.tint");
    let out = g.add_node("motion.output");

    // O DEFAULT do nó, de propósito (ver o ⚠️ do cabeçalho).
    g.set_param(em, "rate", 40.0);
    g.set_param(em, "life", LIFE_S);
    g.set_param(em, "max", POOL);
    g.set_param(em, "speed", 2.4);
    g.set_param(em, "angle", 90.0); // para cima (mundo Y-up)
    g.set_param(em, "spread", 40.0);
    g.set_param(em, "x", 0.0);
    g.set_param(em, "y", -1.0); // lança de baixo, para dentro do quadro
    g.set_param(em, "size", 0.03); // grãos: 2.000 quadrados grandes viram uma chapa
    g.set_param(em, "seed", 3.0);

    // Gravidade: constante (`gust = 0`), reta para baixo (270° => o par `(cos, sin)` lê
    // `(0, −1)`). O ápice `v²/2g` fica ~1,8 unidade acima do bico e a vida acaba na DESCIDA,
    // então o jato ocupa a moldura inteira em vez de só a metade de baixo (medido).
    g.set_param(grav, "angle", 270.0);
    g.set_param(grav, "strength", 1.6);
    g.set_param(grav, "gust", 0.0);

    // Cor por IDADE (os ids do emitter sobem do mais velho para o mais novo), então o jato sai
    // quente no bico e frio nas pontas — é o que torna a DENSIDADE legível quando o `Rate` sobe.
    g.set_param(tint, "mode", 1.0); // Gradient
    g.set_param(tint, "r", 1.0);
    g.set_param(tint, "g", 0.84);
    g.set_param(tint, "b", 0.42);
    g.set_param(tint, "a", 1.0);
    g.set_param(tint, "r2", 0.18);
    g.set_param(tint, "g2", 0.36);
    g.set_param(tint, "b2", 0.95);
    g.set_param(tint, "a2", 1.0);

    for (from, to, port, delayed) in [
        (em, ig, 0, false),
        // O fio de retorno que o artista nunca desenha: o estado do tick anterior entra na
        // cabeça do laço (ADR-0135; no editor ele aparece como o badge de portal ⊙).
        (ig, grav, 0, true),
        (grav, ig, 1, false),
        (ig, tint, 0, false),
        (tint, out, 0, false),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed,
        })
        .expect("emitter-smoke edge");
    }
    (out, [em, ig])
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_EMITTER_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `units_smoke`. No-op sem a env.
    pub(crate) fn emitter_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let (sink, heroes) = chain(&mut gfx.motion.doc.graph);
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &heroes);
        gfx.motion.sinks.push(sink);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        // O emitter já selecionado: `Rate` e `Max Particles` na tela no 1º frame.
        ph2d_panel_motion_graph::request_graph_selection(vec![heroes[0].0]);

        // ⚠️ Os números vêm das consts, não de literais na prosa: uma mensagem que repete o
        // número à mão mente no dia em que a const se mexe, com a suíte verde.
        let alive_default = (40.0 * LIFE_S) as u32;
        let alive_full = (1_200.0 * LIFE_S) as u32;
        eprintln!(
            "[emitter smoke] fonte montada: emitter -> integrate -> tint -> output, com \
             force.wind de gravidade.\n  \
             O no 'Emitter' ja esta selecionado. APERTE PLAY -- uma fonte so existe no tempo.\n  \
             Ela abre nos DEFAULTS do no: Rate = 40/s, vida {LIFE_S} s => ~{alive_default} \
             particulas vivas, um fio ralo.\n  \
             TESTE 1 (o curso do RATE, a pergunta desta wave): arraste 'Rate' ate o FIM do \
             slider (1.200/s => ~{alive_full} vivas). O fio vira um jato denso.\n    \
             >> A PERGUNTA: 1.200 e denso o bastante para a sua cena? Se nao for, DIGITE \
             12000 na caixa -- ela aceita ate 4.000.000, entao nada se perdeu; o que quero \
             saber e se o DEDO precisa chegar mais longe.\n  \
             TESTE 2 (o teto da PISCINA): com o Rate no alto, arraste 'Max Particles' de \
             {POOL} para baixo. Abaixo de ~{alive_full} o jato e CEIFADO (as mais velhas \
             somem) -- e isso que o teto da piscina faz. Depois DIGITE 100000 na caixa: o \
             jato volta inteiro.\n  \
             TESTE 3 (o nudge, que era impossivel): com o Rate em 40, empurre o slider UM \
             passo. Ele anda ~8/s. Antes desta wave um pixel andava 78/s -- de 40 voce \
             pulava para 118 e nao havia como pedir 50."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_node_registry::NodeRegistry;
    use ph2d_nodegraph::cook::Cook;

    fn registry() -> NodeRegistry {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
        reg
    }

    /// Roda a cena do tick 0 até `t` e devolve `(vivas, caixa de x, caixa de y)`.
    ///
    /// ⚠️ **`cook` sozinho NÃO é o caminho do produto, e a sonda pegou isso antes do smoke:**
    /// o `dt` que o integrador lê é `playhead − prev_playhead`, e o `prev_playhead` só existe
    /// depois de um [`Cook::advance_tick`] — que é também o que carrega as arestas `pre`. Sem
    /// ele **toda partícula fica no bico** (medido: a caixa inteira colapsava em `(0, −1)`),
    /// e os gates de CONTAGEM ficariam verdes sobre uma cena imóvel, porque o `motion.emitter`
    /// é stateless e conta certo mesmo com o laço parado.
    fn drive(rate: f32, max: f32, t: f32) -> (usize, [f32; 2], [f32; 2]) {
        let reg = registry();
        let mut g = Graph::new();
        let (out, heroes) = chain(&mut g);
        g.set_param(heroes[0], "rate", rate);
        g.set_param(heroes[0], "max", max);
        g.validate(&reg).expect("cena bem-tipada");

        let mut cook = Cook::new();
        let mut tick = 0.0f64;
        let (mut n, mut x, mut y) = (0, [0.0; 2], [0.0; 2]);
        while tick <= f64::from(t) + 1e-3 {
            if let Ok(set) = cook.cook(&g, &reg, out, tick)
                && let Some(s) = set.iter().next()
            {
                let st = s.as_stream();
                n = st.count();
                if let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = st.get("P") {
                    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
                    for v in p {
                        for k in 0..2 {
                            lo[k] = lo[k].min(v[k]);
                            hi[k] = hi[k].max(v[k]);
                        }
                    }
                    (x, y) = ([lo[0], hi[0]], [lo[1], hi[1]]);
                }
            }
            cook.advance_tick(&g, &reg, tick).expect("advance");
            tick += 1.0 / 60.0;
        }
        (n, x, y)
    }

    /// Quantas partículas vivas a cena tem em `t`, com `rate`/`max` sobrepostos.
    fn alive_at(rate: f32, max: f32, t: f32) -> usize {
        drive(rate, max, t).0
    }

    /// **SONDA — a cena cabe na moldura?** Um smoke cujo jato sai do quadro é um smoke em que
    /// o artista julga um terço do que a mensagem descreve, e eu não vejo a tela. As outras
    /// cenas deste painel vivem em torno de `|x|,|y| ≲ 1,5` (o echo orbita a 0,9), então é
    /// contra ISSO que a caixa desta fonte é lida.
    ///
    /// `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop emitter_smoke -- --ignored --nocapture`
    #[test]
    #[ignore = "sonda de diagnostico"]
    fn measure_what_the_fountain_occupies() {
        for (rate, t) in [(40.0, 1.0), (40.0, LIFE_S), (1_200.0, 1.5)] {
            let (n, x, y) = drive(rate, POOL, t);
            println!(
                "  rate {rate:>7} @ t={t}s -> {n:>5} vivas   \
                 x [{:+.2}, {:+.2}]   y [{:+.2}, {:+.2}]",
                x[0], x[1], y[0], y[1]
            );
        }
    }

    /// A cena cozinha e tem partículas — um smoke cuja cena não cozinha mostra tela vazia,
    /// que é o que o artista reportaria como *"o smoke está quebrado"*.
    #[test]
    fn the_fountain_cooks_and_is_alive() {
        let n = alive_at(40.0, POOL, 1.0);
        assert!(n > 0, "a fonte esta vazia em t=1s -- tela em branco");
    }

    /// **A FONTE VOA — e este gate existe porque ela não voava.**
    ///
    /// A 1ª versão da sonda cozinhava sem [`Cook::advance_tick`], e **toda partícula ficava
    /// no bico**: a caixa inteira colapsava no ponto de lançamento. ⚠️ Os três gates de
    /// CONTAGEM ficavam VERDES sobre essa cena imóvel, porque o `motion.emitter` é stateless
    /// e conta certo com o laço parado — quem estava morto era o `motion.integrate`, e nada
    /// que contasse partículas podia perceber. O oráculo tem de ser o LUGAR delas.
    ///
    /// Ele também é o gate de ENQUADRAMENTO: as outras cenas deste painel vivem em torno de
    /// `|x|,|y| ≲ 1,5`, e um jato que sai da moldura faz o artista julgar um terço do que a
    /// mensagem descreve.
    #[test]
    fn the_fountain_flies_and_stays_in_frame() {
        let (_, x, y) = drive(40.0, POOL, LIFE_S);
        assert!(
            y[1] > -0.5,
            "o jato tem de SUBIR acima do bico (y=-1.0); o topo dele esta em {:.2} -- \
             uma fonte parada no lançamento e a cena imovel que este gate existe para pegar",
            y[1]
        );
        assert!(
            x[1] - x[0] > 1.0,
            "o cone tem de ABRIR: a largura medida e {:.2}",
            x[1] - x[0]
        );
        for (lo, hi, eixo) in [(x[0], x[1], "x"), (y[0], y[1], "y")] {
            assert!(
                lo > -3.0 && hi < 3.0,
                "o jato sai da moldura no eixo {eixo}: [{lo:.2}, {hi:.2}]"
            );
        }
    }

    /// **O TESTE 1 da mensagem tem de ser VERDADE:** arrastar o `Rate` até o fim do slider
    /// tem de DENSIFICAR o jato, e não bater no teto da piscina — senão o artista estaria
    /// julgando o `max` achando que julga o `rate`.
    #[test]
    fn dragging_the_rate_to_its_ceiling_densifies_without_hitting_the_pool() {
        let sparse = alive_at(40.0, POOL, 1.5);
        let dense = alive_at(1_200.0, POOL, 1.5);
        assert!(
            dense > sparse * 5,
            "arrastar o Rate ate 1.200 tem de densificar: {sparse} -> {dense}"
        );
        assert!(
            (dense as f32) < POOL,
            "o Rate no topo ({dense} vivas) nao pode encostar no teto da piscina ({POOL}) -- \
             se encostar, o TESTE 1 estaria medindo o `max`"
        );
    }

    /// **O TESTE 2 tem de ser VERDADE:** baixar o `Max Particles` CEIFA o jato. Sem isto a
    /// mensagem manda o artista arrastar um slider que não muda nada na tela.
    #[test]
    fn lowering_the_pool_cap_clips_the_fountain() {
        let full = alive_at(1_200.0, POOL, 1.5);
        let capped = alive_at(1_200.0, 256.0, 1.5);
        assert!(
            capped < full,
            "baixar Max Particles tem de ceifar: {full} -> {capped}"
        );
        assert!(
            capped <= 256,
            "o teto e um TETO: {capped} vivas sob um max de 256"
        );
    }

    /// **E a caixa alcança o que o slider não alcança** — a promessa inteira da wave. Um
    /// `rate` de 12.000 (o teto SOFT de antes) tem de continuar produzindo o que produzia.
    #[test]
    fn typing_past_the_slider_still_reaches_the_old_ceiling() {
        let reg = registry();
        let hard = reg
            .param_hard_max(
                ph2d_nodegraph::node::NodeTypeId::of("motion.emitter"),
                "rate",
            )
            .expect("o rate tem teto duro");
        assert!(
            hard >= 12_000.0,
            "a caixa tem de alcancar o teto SOFT de antes (12.000), e ela para em {hard}"
        );
        let typed = alive_at(12_000.0, 100_000.0, 1.5);
        let dragged = alive_at(1_200.0, 100_000.0, 1.5);
        assert!(
            typed > dragged * 5,
            "digitar 12.000 tem de valer mais que arrastar ate 1.200: {dragged} -> {typed}"
        );
    }
}
