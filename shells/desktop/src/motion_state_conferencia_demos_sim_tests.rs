//! Os gates da cena `=75` — o pin que rasga e o bando que desvia.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::NodeId;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn scene() -> (MotionDoc, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_sim_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipado");
    (doc, sinks)
}

fn nodes_of(doc: &MotionDoc, ty: &str) -> Vec<NodeId> {
    doc.graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == ty)
        .map(|n| n.id)
        .collect()
}

fn param(doc: &MotionDoc, id: NodeId, name: &str) -> f32 {
    doc.graph
        .node_param_overrides(id)
        .and_then(|m| m.get(name).copied())
        .unwrap_or(f32::NAN)
}

/// **DUAS LINHAS, QUATRO NUVENS, E AS PEDRAS DESENHADAS.**
#[test]
fn the_scene_is_two_rows_with_its_marks_drawn() {
    let (doc, sinks) = scene();
    assert_eq!(sinks.len(), 6, "quatro nuvens + duas pedras");
    assert_eq!(
        nodes_of(&doc, "motion.soft_body").len(),
        2,
        "a linha 1 é TECIDO"
    );
    assert_eq!(nodes_of(&doc, "motion.boids").len(), 2);
    assert_eq!(nodes_of(&doc, "motion.pin_constraint").len(), 2);
}

/// **A CORTINA É UM TECIDO, e o pin vive na cadeia de estado dele.**
///
/// ⚠️ **Duas correcções de smoke numa só linha.** (1) *"porque usar grid se temos nós de
/// tecido?"* — uma nuvem de pontos soltos não mostra um pano a rasgar. (2) A v1 pôs o
/// pin no laço do `motion.integrate`, que lê o `inv_mass` do `rest` e não do `state`, e
/// nada ficava pinado. O `motion.soft_body` lê **os dois** da cadeia de estado, então
/// aqui o pin cabe dentro dela e o `in` dele já traz a carga.
#[test]
fn the_curtain_is_a_cloth_and_the_pin_lives_in_its_state_chain() {
    let (doc, _) = scene();
    let edges = doc.graph.edges();
    let cloths = nodes_of(&doc, "motion.soft_body");
    assert_eq!(cloths.len(), 2, "uma cortina de TECIDO por metade");
    assert!(
        nodes_of(&doc, "motion.integrate").is_empty(),
        "o tecido é o próprio solver — um integrador aqui seria uma segunda física"
    );
    for ((cloth, wind), pin) in cloths
        .iter()
        .zip(nodes_of(&doc, "force.wind"))
        .zip(nodes_of(&doc, "motion.pin_constraint"))
    {
        assert_eq!(
            param(&doc, *cloth, "pin"),
            0.0,
            "o pin INTRÍNSECO tem de estar desligado — senão a folha fica presa de \
             qualquer maneira e o par sai mudo"
        );
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == *cloth && e.to == (wind, 0) && e.delayed),
            "o `pre` do tecido abre a cadeia de estado"
        );
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == wind && e.to == (pin, 0) && !e.delayed),
            "o vento tem de estar A MONTANTE do pin — é assim que a carga lhe chega"
        );
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == pin && e.to == (*cloth, 2) && !e.delayed),
            "e o pin devolve o `inv_mass` pela porta `state` do tecido"
        );
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == pin && e.to == (pin, 1) && e.delayed),
            "mais a própria memória, senão ele CEDE em vez de rasgar"
        );
    }
    assert_eq!(nodes_of(&doc, "force.wind").len(), 2, "um vento por metade");
}

/// **A LINHA 1 DIFERE SÓ NO `break_above`, e o pin selecciona a MESMA fileira.**
#[test]
fn the_tear_row_changes_only_the_threshold() {
    let (doc, _) = scene();
    let pins = nodes_of(&doc, "motion.pin_constraint");
    let (first, count) = pinned_run();
    for p in &pins {
        assert_eq!(param(&doc, *p, "first"), first, "a mesma fileira");
        assert_eq!(param(&doc, *p, "count"), count);
        assert_eq!(param(&doc, *p, "strength"), 1.0);
    }
    assert_eq!(param(&doc, pins[0], "break_above"), 0.0, "esquerda: nunca");
    assert_eq!(param(&doc, pins[1], "break_above"), BREAK_ABOVE);
    // E os dois ventos são o mesmo vento — mas a força deles é DIRIGIDA por um fio, não
    // um param, então é a rampa que se compara.
    //
    // ⚠️ **A versão anterior deste gate comparava `param(wind, "strength")` e passou a
    // comparar `NaN` com `NaN` no dia em que o param passou a ser dirigido** — um gate
    // que deixou de medir sem ficar vermelho. É a mesma classe do knob morto: um
    // `unwrap_or(NaN)` responde a uma pergunta que já não faz sentido.
    let ramps = nodes_of(&doc, "value.map_range");
    assert_eq!(ramps.len(), 2, "uma rampa de vento por metade");
    for r in &ramps {
        assert_eq!(param(&doc, *r, "in_hi"), RAMP_SECS, "a mesma subida");
        assert_eq!(param(&doc, *r, "out_hi"), WIND_TOP, "e o mesmo topo");
    }
    for (w, r) in nodes_of(&doc, "force.wind").iter().zip(&ramps) {
        let src = doc
            .graph
            .param_sources(*w)
            .and_then(|m| m.get("strength").copied());
        assert_eq!(
            src,
            Some((*r, 0)),
            "a força do vento tem de vir da rampa, por fio"
        );
    }
}

/// **O LIMIAR CAI DENTRO DA RAMPA DO VENTO — nem antes dela, nem depois.**
///
/// ⚠️ **As duas metades desta asserção vieram de dois smokes reprovados.** Abaixo do
/// início: a carga já passa o limiar no primeiro quadro e o pano nasce a voar — foi o
/// *"não rasga"* (não se vê rasgar, vê-se um painel vazio). Acima do topo: nunca rasga.
/// O limiar tem de ficar no MEIO da subida, e a margem é o que garante que o pano
/// pendura um bocado antes de soltar.
#[test]
fn the_tear_threshold_lands_inside_the_wind_ramp() {
    let (limiar, topo) = (
        std::hint::black_box(BREAK_ABOVE),
        std::hint::black_box(WIND_TOP),
    );
    assert!(
        limiar > topo * 0.25,
        "o limiar ({limiar}) tem de ficar bem depois do início da rampa, senão o pano \
         rasga antes de se ver que ele estava pendurado"
    );
    assert!(
        limiar < topo * 0.9,
        "e antes do topo ({topo}), senão ele nunca rasga"
    );
}

/// **A FILEIRA PINADA É A DE CIMA, e o índice é DERIVADO da grelha.**
///
/// ⚠️ A `motion.grid` é row-major da MENOR altura para cima, então as últimas `cols`
/// peças são o topo. Escrever `30` à mão aqui seria um número que deixa de significar o
/// topo assim que alguém mexer na grelha.
#[test]
fn the_pinned_run_is_the_top_row_derived_from_the_grid() {
    let (cols, rows, ..) = CURTAIN;
    let (first, count) = pinned_run();
    assert_eq!(count, cols, "uma fileira inteira");
    assert_eq!(first, (rows - 1.0) * cols, "a ÚLTIMA fileira = o topo");
    assert!(first + count <= rows * cols, "e ela cabe na grelha");
}

/// **A LINHA 2 DIFERE SÓ NO `avoid`, E SÓ A DIREITA TEM PEDRAS LIGADAS — na porta 3.**
#[test]
fn the_avoid_row_changes_only_the_steer() {
    let (doc, _) = scene();
    let flocks = nodes_of(&doc, "motion.boids");
    assert_eq!(param(&doc, flocks[0], "avoid"), 0.0, "esquerda: atravessa");
    assert_eq!(param(&doc, flocks[1], "avoid"), AVOID);
    for k in ["count", "seed", "radius", "separation", "seek", "lookahead"] {
        assert_eq!(
            param(&doc, flocks[0], k),
            param(&doc, flocks[1], k),
            "`{k}` tem de ser o mesmo — senão a linha mede duas coisas"
        );
    }
    let edges = doc.graph.edges();
    let rings = nodes_of(&doc, "motion.distribute_radial");
    assert_eq!(rings.len(), 2, "as duas metades DESENHAM as pedras");
    assert!(
        edges
            .iter()
            .any(|e| e.from.0 == rings[1] && e.to == (flocks[1], 3) && !e.delayed),
        "só a direita LIGA as pedras à porta `obstacle`"
    );
    assert!(
        !edges.iter().any(|e| e.to == (flocks[0], 3)),
        "e a esquerda não pode tê-las ligadas — é o controlo"
    );
    for f in &flocks {
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == *f && e.to == (*f, 2) && e.delayed),
            "todo bando precisa do próprio estado"
        );
    }
}

/// **AS PEDRAS SÃO DESENHADAS NOS DOIS LADOS** — senão a metade da esquerda pareceria
/// não ter obstáculo nenhum, e o par mediria *"há pedra?"* em vez de *"ele desvia?"*.
#[test]
fn both_halves_draw_the_rocks() {
    let (doc, _) = scene();
    let rings = nodes_of(&doc, "motion.distribute_radial");
    for r in &rings {
        assert_eq!(param(&doc, *r, "count"), ROCKS);
        assert_eq!(param(&doc, *r, "radius"), ROCK_RING);
    }
}

/// **O RAIO DO DESVIO ALCANÇA O ESPAÇAMENTO DAS PEDRAS** — senão o bando passa PELOS
/// VÃOS e o desvio nunca se vê.
///
/// ⚠️ A corda entre duas pedras vizinhas de um anel de `n` é `2·R·sin(π/n)`; para `n`
/// grande ela tende a `2πR/n`, e a aproximação **subestima** a corda, o que torna a
/// asserção conservadora. Derivada do anel, não escrita à mão.
#[test]
fn the_avoid_radius_covers_the_gap_between_rocks() {
    let vao = 2.0 * core::f32::consts::PI * ROCK_RING / ROCKS;
    assert!(
        AVOID_RADIUS >= vao * 0.5,
        "o raio ({AVOID_RADIUS}) tem de cobrir meio vão ({:.2})",
        vao * 0.5
    );
}

/// **O DIAGNOSER DA CASA NÃO ACHA BURACO NESTA CENA.**
#[test]
fn the_house_diagnoser_finds_no_hole_in_this_scene() {
    let (doc, _) = scene();
    let reg = registry();
    let d = ph2d_motion_diagnose::diagnose(&doc.graph, &reg);
    assert!(d.is_empty(), "a cena não encena defeito nenhum: {d:?}");
}

/// **A FOLHA DA ESQUERDA FICA PENDURADA; A DA DIREITA SOLTA-SE E VAI-SE EMBORA.**
///
/// ⚠️ **Este é o gate que faltava, e é a terceira vez que esta linha paga a mesma lei.**
/// O smoke voltou com *"tudo foi levado pelo vento, nada rasgou"* e os NOVE gates desta
/// cena estavam verdes — porque todos mediam a FORMA do grafo, e a forma que eu escrevera
/// era a que eu ACREDITAVA estar certa. Um gate que corre a simulação e olha quem ficou
/// não tem opinião nenhuma.
///
/// ⚠️ **As três barras são DERIVADAS do passo do tecido**, nunca escritas: um pano mais
/// fechado ou mais aberto move números diferentes, e uma constante à mão deixaria de
/// significar *"saiu"* no dia em que alguém mexesse na malha.
#[test]
fn the_left_cloth_hangs_and_the_right_one_tears_free() {
    let (doc, sinks) = scene();
    let reg = registry();
    let (_, _, spacing, _) = CURTAIN;
    let (first, count) = pinned_run();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "índices"
    )]
    let (first, count) = (first as usize, count as usize);

    // ⚠️ **Quatro segundos, e o número é DERIVADO da rampa:** a carga só cruza o limiar
    // lá pelos 2 s (a rampa leva `RAMP_SECS` a chegar ao topo), então uma janela de 2 s
    // mediria o pano AINDA pendurado dos dois lados — verde sobre uma cena que não
    // mostra nada. Medido: o limiar 4,5 é cruzado a **2,00 s**.
    let ticks = (RAMP_SECS * 60.0) as u32 + 120;
    let run = |sink: NodeId| -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
        let mut cook = Cook::new();
        let (mut inicio, mut fim) = (Vec::new(), Vec::new());
        for k in 0..ticks {
            let t = f64::from(k) / 60.0;
            let out = cook.cook(&doc.graph, &reg, sink, t).expect("cozinha");
            if let Some(Column::Vec2(p)) = out[0].as_stream().get("P") {
                if k == 0 {
                    inicio = p.clone();
                }
                fim = p.clone();
            }
            // ⚠️ **O passo que faltava, e sem ele a sonda media ZERO.** O `pre` só avança
            // quando o quadro FECHA; um laço que só `cook`a lê o mesmo tique cento e
            // vinte vezes. Medido contra a cena `=71`, que o Enio já aprovara: ela também
            // dava 0,0000, e foi isso que provou que o erro era do harness.
            cook.advance_tick(&doc.graph, &reg, t)
                .expect("avança o quadro");
        }
        (inicio, fim)
    };
    let percurso = |a: &[[f32; 2]], b: &[[f32; 2]], faixa: std::ops::Range<usize>| -> f32 {
        a[faixa.clone()]
            .iter()
            .zip(&b[faixa])
            .map(|(p, q)| (q[0] - p[0]).abs() + (q[1] - p[1]).abs())
            .fold(0.0_f32, f32::max)
    };

    let (a0, a1) = run(sinks[0]);
    assert_eq!(
        a1[first..first + count],
        a0[first..first + count],
        "a fileira PREGADA da esquerda tem de ficar onde nasceu, ao bit"
    );
    // CONTROLE: e o resto da folha tem de BALANÇAR — senão *"a esquerda ficou parada"*
    // seria verdade sobre uma cena em que o vento não chega a ninguém.
    let balanco = percurso(&a0, &a1, 0..count);
    assert!(
        balanco > spacing * 0.5,
        "a barra da folha pendurada tem de oscilar (>{:.3}); oscilou {balanco:.3}",
        spacing * 0.5
    );

    let (b0, b1) = run(sinks[1]);
    let saiu = percurso(&b0, &b1, first..first + count);
    assert!(
        saiu > spacing * 4.0,
        "a fileira da direita tinha de ter RASGADO e ido embora (>{:.3}); andou {saiu:.3}",
        spacing * 4.0
    );
    assert!(
        saiu > balanco * 4.0,
        "e ela tem de andar MUITO mais que o balanço da esquerda ({saiu:.3} vs {balanco:.3})"
    );
}
