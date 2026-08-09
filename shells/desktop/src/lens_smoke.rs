//! **A cena da família DEFORMERS** (`PH2D_LENS_SMOKE=1`, doc 88 §9.1).
//!
//! A terceira família da varredura. O censo mediu `motion.spherize` com **UM** param
//! (`radius`) e o centro da lente **preso ao centroide** do layout — a única coisa
//! desta família que o grafo **não podia exprimir por composição nenhuma**: um
//! `motion.move` a montante leva o centroide junto, e o campo `falloff` mascara a
//! MISTURA e não o CENTRO. Os três irmãos (`bend`, `twist`, `kaleidoscope`) carregam
//! `pivot_x`/`pivot_y` desde que foram escritos: a família já nomeava a capacidade e a
//! lente era a única que não a tinha.
//!
//! ⚠️ **A cena mostra as DUAS lado a lado**, porque o valor do param é uma COMPARAÇÃO:
//! uma lente sozinha na tela não diz onde ela está — ela só diz que existe.
//!
//! ```text
//! grid -> move(y = +Y) -> spherize(offset 0)          -> tint(âmbar) -> output  (em cima)
//! grid -> move(y = -Y) -> spherize(offset_x = OFFSET)  -> tint(ciano) -> output  (embaixo)
//! ```
//!
//! ⚠️ **O offset é do CENTROIDE, não uma coordenada de mundo** — `c + (0,0)` **é** `c`,
//! então a esteira de cima é, byte a byte, a lente que sempre shipou; e uma lente sobre
//! um conjunto VIVO segue o assunto quando ele anda.

use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Quanto a lente de baixo sai do centroide (unidades de mundo).
const OFFSET: f32 = 1.0;
/// O raio da lente — MENOR que a grade de propósito: é o aro que torna visível *onde*
/// ela está. Uma lente maior que o assunto cobre tudo e a posição some.
const RADIUS: f32 = 1.2;
/// Meia-distância vertical entre as duas esteiras.
const LANE_Y: f32 = 1.6;
/// Lado da grade (ímpar: há um elemento exatamente no meio, que é o ponto fixo da
/// lente quando ela está no centroide).
const SIDE: f32 = 11.0;
/// Espaçamento da grade.
const GAP: f32 = 0.28;

/// Uma esteira: `grid -> move -> spherize -> tint -> output`. Devolve
/// `(sink, spherize, move)` — o `move` sai porque ele é a entrada da lente, e o gate
/// **cozinha** o "antes" ali em vez de reconstruir a grade à mão (uma segunda resposta
/// a *onde o `motion.grid` põe os pontos* divergiria no dia em que a grade mudasse).
fn lane(g: &mut Graph, y: f32, offset_x: f32, warm: bool) -> (NodeId, NodeId, NodeId) {
    let grid = g.add_node("motion.grid");
    let mv = g.add_node("motion.move");
    let sph = g.add_node("motion.spherize");
    let tint = g.add_node("motion.tint");
    let out = g.add_node("motion.output");

    g.set_param(grid, "rows", SIDE);
    g.set_param(grid, "cols", SIDE);
    g.set_param(grid, "gap_x", GAP);
    g.set_param(grid, "gap_y", GAP);
    // A grade nasce na origem; o `move` só separa as duas esteiras em Y — ele NÃO
    // move a lente, e é justamente isso que a wave existe para tornar possível.
    g.set_param(mv, "dx", 0.0);
    g.set_param(mv, "dy", y);
    g.set_param(sph, "radius", RADIUS);
    g.set_param(sph, "offset_x", offset_x);
    g.set_param(sph, "offset_y", 0.0);
    // Âmbar em cima (o controle), ciano embaixo (a lente deslocada).
    let (r, gr, b) = if warm {
        (0.95, 0.62, 0.15)
    } else {
        (0.20, 0.75, 0.95)
    };
    g.set_param(tint, "r", r);
    g.set_param(tint, "g", gr);
    g.set_param(tint, "b", b);

    for (from, to, port) in [
        (grid, mv, 0u16),
        (mv, sph, 0),
        (sph, tint, 0),
        (tint, out, 0),
    ] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .expect("lens-smoke edge");
    }
    (out, sph, mv)
}

/// Monta as duas esteiras. Devolve `(sinks, [lente_no_centroide, lente_deslocada],
/// [entrada_de_cima, entrada_de_baixo])`.
fn scene(g: &mut Graph) -> ([NodeId; 2], [NodeId; 2], [NodeId; 2]) {
    let (out_a, sph_a, mv_a) = lane(g, LANE_Y, 0.0, true);
    let (out_b, sph_b, mv_b) = lane(g, -LANE_Y, OFFSET, false);
    ([out_a, out_b], [sph_a, sph_b], [mv_a, mv_b])
}

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_LENS_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `echo_family_smoke`. No-op sem a env.
    pub(crate) fn lens_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let (sinks, lenses, _) = scene(&mut gfx.motion.doc.graph);
        crate::smoke_layout::arrange_and_mark(&mut gfx.motion.doc, &lenses);
        for s in sinks {
            gfx.motion.sinks.push(s);
        }
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        // A lente DESLOCADA já selecionada: o param novo na tela no 1º frame.
        ph2d_panel_motion_graph::request_graph_selection(vec![lenses[1].0]);
        eprintln!(
            "[lens smoke] duas grades {SIDE:.0}x{SIDE:.0} iguais, cada uma sob uma lente \
             de raio {RADIUS}.\n  \
             MONTOU: em CIMA 'Offset X = 0' (a lente no centroide, o motor que sempre \
             shipou); embaixo 'Offset X = {OFFSET}'.\n  \
             Na tela: em cima o inchaco no MEIO da grade; embaixo o MESMO inchaco \
             empurrado para a DIREITA. Se os dois estiverem no meio, PARE -- o offset \
             nao chegou.\n  \
             O no 'Spherize' de baixo ja esta selecionado.\n  \
             TESTE 1 (a lente ANDA): arraste 'Offset X' de -10 ate 10. O inchaco varre a \
             grade e sai por fora dela -- longe demais, a grade volta ao repouso, que e \
             o comportamento honesto de uma lente que nao esta mais sobre o assunto.\n  \
             TESTE 2 (o default nao mexeu em NADA): ponha 'Offset X' de volta em 0. As \
             duas grades ficam IDENTICAS. Esse e o contrato: 'c + 0' e 'c'.\n  \
             TESTE 3 (o eixo Y tambem): arraste 'Offset Y'. O inchaco sobe e desce.\n  \
             TESTE 4 (a lente SEGUE o assunto): selecione o no 'Move' da esteira de \
             baixo e arraste o 'dx'. A grade inteira anda -- e o inchaco anda JUNTO, \
             mantendo a mesma posicao relativa. E por isso que o numero e um \
             deslocamento do centroide e nao uma coordenada de mundo: um centro \
             absoluto ficaria para tras enquanto a grade vai embora.\n  \
             TESTE 5 (o raio e o aro): arraste o 'Radius'. Fora dele a grade fica \
             intocada -- e a borda do inchaco que diz onde a lente acaba."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_node_registry::NodeRegistry;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::Cook;

    /// O que uma esteira entrega ao gate: as posições ANTES da lente e DEPOIS dela.
    type Lane = (Vec<[f32; 2]>, Vec<[f32; 2]>);

    /// Cozinha a cena e devolve, por esteira, o par `(antes da lente, depois dela)`.
    /// O "antes" sai do nó `move`, cozido — nunca de uma grade redigitada aqui.
    fn run() -> [Lane; 2] {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
        let mut g = Graph::new();
        let (sinks, _, feeds) = scene(&mut g);
        g.validate(&reg).expect("smoke scene is well-typed");
        let mut cook = Cook::new();
        let at = |cook: &mut Cook, n: NodeId| {
            positions(cook.cook(&g, &reg, n, 0.0).expect("cooks")[0].as_stream())
        };
        [0usize, 1].map(|i| {
            let before = at(&mut cook, feeds[i]);
            let after = at(&mut cook, sinks[i]);
            (before, after)
        })
    }

    fn positions(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        }
    }

    /// Onde a lente ESTÁ, lido dos pixels em vez do param: um bulge quadrático
    /// desloca mais os elementos de um ANEL em torno do centro, e o anel é simétrico
    /// nele — então a média de X sobre os mais deslocados estima o centro da lente.
    ///
    /// ⚠️ O estimador não conhece `offset_x`: ele mede o DESENHO. Um oráculo que
    /// somasse o param ao centroide seria a função sob teste computando o que espera.
    fn lens_x(before: &[[f32; 2]], after: &[[f32; 2]]) -> f32 {
        let disp: Vec<f32> = before
            .iter()
            .zip(after)
            .map(|(p, q)| ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt())
            .collect();
        let peak = disp.iter().copied().fold(0.0f32, f32::max);
        assert!(peak > 1e-3, "a cena nao deformou nada (peak {peak})");
        let (sum, n) = before
            .iter()
            .zip(&disp)
            .filter(|(_, d)| **d > peak * 0.5)
            .fold((0.0f32, 0u32), |(s, n), (p, _)| (s + p[0], n + 1));
        sum / n as f32
    }

    /// A cena montou o que a mensagem afirma: a lente de baixo está DESLOCADA e a de
    /// cima não. Lido do desenho — se a mensagem mentir, este gate sangra.
    #[test]
    fn the_scene_puts_one_lens_on_the_centroid_and_one_beside_it() {
        let [(before_a, after_a), (before_b, after_b)] = run();
        assert_eq!(after_a.len(), (SIDE * SIDE) as usize, "a grade de cima");
        assert_eq!(after_b.len(), (SIDE * SIDE) as usize, "a grade de baixo");

        let on_centroid = lens_x(&before_a, &after_a);
        let beside = lens_x(&before_b, &after_b);

        assert!(
            on_centroid.abs() < 0.15,
            "a lente de cima tem de estar no centroide (x ~ 0), medido {on_centroid:.3}"
        );
        assert!(
            (beside - OFFSET).abs() < 0.25,
            "a lente de baixo tem de estar em x ~ {OFFSET}, medido {beside:.3}"
        );
        assert!(
            beside - on_centroid > 0.6,
            "as duas lentes tem de estar em lugares DIFERENTES: {on_centroid:.3} vs {beside:.3}"
        );
    }
}
