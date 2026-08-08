//! **A cena da família ECHO** (`PH2D_ECHO_SMOKE=1`, doc 88 §B3).
//!
//! A segunda família da varredura. O censo mediu `motion.trail` com **3 params** contra
//! os **8** que a referência traz, e o que faltava não era enfeite: um fantasma por TICK
//! é um rastro **contínuo**, que a 60 fps lê como borrão. O *sprite echo* — o eco
//! discreto, que o catálogo de referência traz com `spacing 2` no default DELE — era
//! **inexprimível**, porque o motor promovia a cabeça a fantasma em todo tick.
//!
//! ⚠️ **A cena mostra os DOIS lado a lado**, porque o valor do param é uma COMPARAÇÃO:
//! um rastro sozinho na tela não diz se está espaçado — ele só diz que existe.
//!
//! ```text
//! grid -> move -> orbit -> trail(spacing 1) -> output   (em cima)
//! grid -> move -> orbit -> trail(spacing 4) -> output   (embaixo)
//! ```
//!
//! ⚠️ **O `spacing` não custou estado novo** — a coluna `trail_age` já sabe há quantos
//! ticks o último eco foi deixado, então a promoção é uma pergunta ao ESTADO e não a um
//! contador. É por isso que `spacing = 1` volta a ser, **byte a byte**, o motor que sempre
//! shipou: a faixa que ele consulta fica vazia e a promoção acontece sempre.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O espaçamento da esteira de BAIXO — o eco discreto que a wave destrava.
const SPACED: f32 = 4.0;
/// Quantos ecos cada rastro carrega (os dois iguais: só o espaçamento difere).
const ECHOES: f32 = 6.0;

/// Uma esteira: `grid -> move -> orbit -> trail -> output`, com o `pre` self-loop que o
/// nó sequencial exige. Devolve `(sink, trail)`.
fn lane(g: &mut Graph, y: f32, spacing: f32) -> (NodeId, NodeId) {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let orbit = g.add_node("motion.orbit");
    let trail = g.add_node("motion.trail");
    let out = g.add_node("motion.output");

    // Um ponto só: o rastro dele é a coisa inteira que se olha.
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 1.0);
    // Fora do pivô, senão a órbita gira em torno do próprio ponto e nada anda.
    g.set_param(mv, "dx", 0.9);
    g.set_param(mv, "dy", y);
    g.set_param(orbit, "pivot_x", 0.0);
    g.set_param(orbit, "pivot_y", y);
    g.set_param(orbit, "speed", 140.0);

    g.set_param(trail, "length", ECHOES);
    g.set_param(trail, "fade", 0.8);
    g.set_param(trail, "shrink", 0.93);
    g.set_param(trail, "spacing", spacing);

    for (from, to) in [(grid, mv), (mv, orbit), (orbit, trail), (trail, out)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .expect("echo-smoke edge");
    }
    // O anel: a saída do tick anterior É o buffer. Sem esta aresta o rastro nunca
    // acumula e a cena mostra um ponto solto — o que se reportaria como "quebrado".
    g.connect(Edge {
        from: (trail, 0),
        to: (trail, 1),
        delayed: true,
    })
    .expect("echo-smoke self loop");
    (out, trail)
}

/// Monta as duas esteiras. Devolve `(sinks, [trail_continuo, trail_espacado])`.
fn scene(g: &mut Graph) -> ([NodeId; 2], [NodeId; 2]) {
    let (out_a, trail_a) = lane(g, 0.6, 1.0);
    let (out_b, trail_b) = lane(g, -0.6, SPACED);
    ([out_a, out_b], [trail_a, trail_b])
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_ECHO_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `transform_family_smoke`. No-op sem a env.
    pub(crate) fn echo_family_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let (sinks, trails) = scene(&mut gfx.motion.doc.graph);
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &trails);
        for s in sinks {
            gfx.motion.sinks.push(s);
        }
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        // O rastro ESPAÇADO já selecionado: o param novo na tela no 1º frame.
        ph2d_panel_motion_graph::request_graph_selection(vec![trails[1].0]);
        eprintln!(
            "[echo smoke] duas esteiras iguais em orbita, com {ECHOES} ecos cada.\n  \
             MONTOU: em CIMA 'Spacing = 1' (o rastro continuo, o motor que sempre \
             shipou); embaixo 'Spacing = {SPACED}'.\n  \
             DE PLAY na timeline. Na tela: em cima um borrao continuo colado no ponto; \
             embaixo ecos SEPARADOS, cobrindo {SPACED}x mais arco com o mesmo numero de \
             copias. Se os dois rastros forem iguais, PARE -- o espacamento nao chegou.\n  \
             O no 'Trail' de baixo ja esta selecionado.\n  \
             TESTE 1 (a cadencia): arraste o 'Spacing' de 1 ate 16. Em 1 os dois rastros \
             ficam IDENTICOS -- e a prova de que o default nao mexeu em nada.\n  \
             TESTE 2 (o alcance): com 'Spacing' alto, aumente o 'Length'. A cauda cobre \
             a orbita inteira sem custar um eco por quadro.\n  \
             TESTE 3 (a cabeca): o ponto VIVO nunca pisca -- ele e desenhado todo tick, \
             espacado ou nao; o que espaca sao os FANTASMAS."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_node_registry::NodeRegistry;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::Cook;

    /// Roda a cena por `ticks` e devolve as duas saídas (contínua, espaçada).
    fn run(ticks: u32) -> (Stream, Stream) {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
        let mut g = Graph::new();
        let (sinks, _) = scene(&mut g);
        g.validate(&reg).expect("smoke scene is well-typed");
        let mut cook = Cook::new();
        let (mut a, mut b) = (Stream::new(0), Stream::new(0));
        for t in 0..ticks {
            let t = f64::from(t);
            a = cook.cook(&g, &reg, sinks[0], t).expect("lane A cooks")[0]
                .as_stream()
                .clone();
            b = cook.cook(&g, &reg, sinks[1], t).expect("lane B cooks")[0]
                .as_stream()
                .clone();
            cook.advance_tick(&g, &reg, t).expect("the ring advances");
        }
        (a, b)
    }

    fn ages(s: &Stream) -> Vec<f32> {
        let mut v = match s.get("trail_age") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => vec![],
        };
        v.sort_by(f32::total_cmp);
        v
    }

    /// **A CENA DE FATO DESBOTA E ENCOLHE** — o gate do defeito de 2026-08-08.
    ///
    /// ⚠️ O smoke reportou *"Fade e Shrink não têm efeito algum"* e a sonda mediu a causa
    /// na cena REAL: as colunas eram `["Count", "Index", "P", "trail_age"]` — um
    /// `motion.grid` não carrega `tint` nem `size`, e os dois knobs multiplicavam o que
    /// não existia. Este gate mede o que o artista VÊ (a rampa de alfa e de tamanho ao
    /// longo da cauda), não a presença das colunas: presença é o mecanismo, e o mecanismo
    /// pode mudar.
    #[test]
    fn the_echo_scene_actually_fades_and_shrinks() {
        let (a, _) = run(12);
        let alpha = |s: &Stream| match s.get("tint") {
            Some(Column::Vec4(v)) => v.iter().map(|c| c[3]).collect::<Vec<_>>(),
            _ => panic!("a cena tem de carregar `tint` — sem ela o Fade e um no-op"),
        };
        let al = alpha(&a);
        assert!(
            al.windows(2).all(|w| w[0] < w[1]),
            "o alfa tinha de subir do eco mais velho ate a cabeca viva, e mediu {al:?}"
        );
        assert!(
            *al.last().expect("a cabeca") > 0.99 && al[0] < 0.5,
            "a cabeca e opaca e a cauda quase sumiu: {al:?}"
        );
        let sz = match a.get("size") {
            Some(Column::Vec2(v)) => v.iter().map(|s| s[0]).collect::<Vec<_>>(),
            _ => panic!("a cena tem de carregar `size` — sem ela o Shrink e um no-op"),
        };
        assert!(
            sz.windows(2).all(|w| w[0] < w[1]),
            "o tamanho tinha de crescer do eco mais velho ate a cabeca, e mediu {sz:?}"
        );
    }

    /// **A cena MOSTRA a diferença que a mensagem promete.**
    ///
    /// ⚠️ O oráculo é a CADÊNCIA das idades e o ARCO coberto — não a contagem: as duas
    /// esteiras carregam o mesmo número de ecos **de propósito**, então uma contagem
    /// seria igual nas duas e o gate ficaria verde sobre dois rastros idênticos.
    #[test]
    fn the_echo_smoke_shows_a_continuous_trail_against_a_spaced_one() {
        let (a, b) = run(24);
        assert_eq!(
            ages(&a),
            vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
            "a esteira de cima e o rastro continuo"
        );
        assert_eq!(
            ages(&b),
            vec![0.0, 3.0, 7.0, 11.0, 15.0, 19.0],
            "a de baixo deixa um eco a cada {SPACED} ticks"
        );
        // ⚠️ MESMO numero de linhas nas duas: `length` conta linhas em todo espaçamento,
        // e um gate que medisse a CONTAGEM ficaria verde sobre dois rastros iguais.
        assert_eq!(a.count(), b.count());

        // E o arco coberto: a distancia entre o eco mais velho e a cabeca viva.
        let span = |s: &Stream| match s.get("P") {
            Some(Column::Vec2(v)) => {
                let (first, last) = (v[0], v[v.len() - 1]);
                ((first[0] - last[0]).powi(2) + (first[1] - last[1]).powi(2)).sqrt()
            }
            _ => panic!("P"),
        };
        assert!(
            span(&b) > span(&a) * 1.5,
            "o rastro espacado tinha de cobrir muito mais arco: {} contra {}",
            span(&b),
            span(&a)
        );
    }

    /// **O TESTE 1 da mensagem é verdade** — em `spacing = 1` as duas esteiras são a
    /// MESMA coisa, que é a prova de que o default desta wave não moveu nada.
    #[test]
    fn at_spacing_one_the_two_lanes_are_the_same_trail() {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
        let mut g = Graph::new();
        let (sinks, trails) = scene(&mut g);
        g.set_param(trails[1], "spacing", 1.0);
        let mut cook = Cook::new();
        let (mut a, mut b) = (Stream::new(0), Stream::new(0));
        for t in 0..24 {
            let t = f64::from(t);
            a = cook.cook(&g, &reg, sinks[0], t).unwrap()[0]
                .as_stream()
                .clone();
            b = cook.cook(&g, &reg, sinks[1], t).unwrap()[0]
                .as_stream()
                .clone();
            cook.advance_tick(&g, &reg, t).unwrap();
        }
        assert_eq!(ages(&a), ages(&b));
        assert_eq!(a.count(), b.count());
    }
}
