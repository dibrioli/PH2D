//! **O PULSO VIRA UM NÚMERO, E É ISSO QUE DESTRAVA A METADE QUE FALTAVA** (doc 89, folha
//! 12 · W6 PULSE).
//!
//! A conferência dos seis `pulse.*` achou **uma** causa por trás de metade da tabela — um
//! pulso não tinha nível —, e a folha marcou as consequências como P1 *"colapsa na P0"*.
//! Estes gates são a prova executável de que ela colapsou: cada um monta a cadeia **no
//! registry REAL** (`register_all_nodes`) e cozinha quadro a quadro, em vez de fabricar
//! streams à mão.
//!
//! ⚠️ **Rodam AQUI e não na `ph2d-node-pulse-level`**, pela mesma razão do
//! `generators_consume_accel` e do `param_census`: esta é a crate onde TODO nó é
//! registrado, então é o build mais barato que enxerga um `pulse.beat`, um `value.math` e
//! um `pulse.compare` ao mesmo tempo. Um gate dentro da crate do nó provaria a aritmética
//! de uma coluna e **não** provaria que a lógica que o artista monta funciona.
//!
//! O último gate é de sinal trocado: ele **mede o que NÃO foi construído**, para que a
//! ausência dos params continue sendo uma decisão em vez de um esquecimento.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O registry REAL — as cadeias sob teste são as que o app ship.
fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Uma fonte de `n` instâncias numa linha — o `in` que todo pulso precisa só para
/// saber quantas linhas paceia.
fn rows(g: &mut Graph, n: f32) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", n);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    grid
}

/// Um metrônomo com o `pre` self-loop que o editor plumba (`out --pre--> state`).
fn beat(g: &mut Graph, src: NodeId, period: f32) -> NodeId {
    let b = g.add_node("pulse.beat");
    g.set_param(b, "period", period);
    connect(g, src, 0, b, 0);
    self_loop(g, b, 1);
    b
}

fn connect(g: &mut Graph, from: NodeId, from_port: u16, to: NodeId, to_port: u16) {
    g.connect(Edge {
        from: (from, from_port),
        to: (to, to_port),
        delayed: false,
    })
    .expect("edge");
}

/// A aresta DELAYED que sai do nó e volta para a própria porta de estado.
fn self_loop(g: &mut Graph, node: NodeId, port: u16) {
    g.connect(Edge {
        from: (node, 0),
        to: (node, port),
        delayed: true,
    })
    .expect("out --pre--> state");
}

fn chain(g: &mut Graph, ty: &str, src: NodeId, params: &[(&str, f32)]) -> NodeId {
    let n = g.add_node(ty);
    for (k, v) in params {
        g.set_param(n, *k, *v);
    }
    connect(g, src, 0, n, 0);
    n
}

/// Cozinha `ticks` quadros a 60 fps e devolve a coluna `col` da PRIMEIRA linha em
/// cada quadro — a leitura de um trem no tempo.
fn train(g: &Graph, reg: &NodeRegistry, node: NodeId, col: &str, ticks: usize) -> Vec<f32> {
    let mut cook = Cook::new();
    let mut out = Vec::with_capacity(ticks);
    for k in 0..ticks {
        let t = k as f64 / 60.0;
        let s = cook.cook(g, reg, node, t).expect("cooks")[0]
            .as_stream()
            .clone();
        out.push(first(&s, col));
        cook.advance_tick(g, reg, t).expect("advances");
    }
    out
}

fn first(s: &Stream, col: &str) -> f32 {
    match s.get(col) {
        Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Os índices dos quadros em que o trem disparou.
fn fired(train: &[f32]) -> Vec<usize> {
    train
        .iter()
        .enumerate()
        .filter(|(_, v)| **v > 0.5)
        .map(|(i, _)| i)
        .collect()
}

/// **O pulso vira um número que o domínio de VALOR consegue ler.**
///
/// FALSIFICAÇÃO da ponte: antes deste nó, `param_source::driven_value` lia
/// `attr::VALUE_COLUMN` (`"v"`) e um pulso emitia `"pulse"` ⇒ nenhum `value.*`, nenhum
/// `motion.drive` e nenhum param dirigido conseguia ouvir um disparo. O gate exige as duas
/// metades no MESMO trem: o `1.0` nos quadros do beat **e** o `0.0` nos outros (uma ponte
/// que devolvesse a constante 1 passaria só com a primeira).
#[test]
fn the_pulse_becomes_a_number_the_value_domain_can_read() {
    let reg = registry();
    let mut g = Graph::new();
    let src = rows(&mut g, 1.0);
    let b = beat(&mut g, src, 0.5);
    let lvl = chain(&mut g, "pulse.level", b, &[]);
    g.validate(&reg).expect("well-typed");

    let v = train(&g, &reg, lvl, "v", 60);
    assert_eq!(fired(&v), vec![0, 30], "o beat de 0,5 s bate no 0 e no 0,5");
    assert!(
        v.iter().all(|x| *x == 0.0 || *x == 1.0),
        "máscara 0/1: {v:?}"
    );
}

/// **A LÓGICA entre pulsos, pelo `value.math` que já existia** — a P1 que a folha 12
/// declara colapsar nesta P0.
///
/// Dois metrônomos (0,5 s e 0,25 s) coincidem no 0 e no 0,5 e divergem no 0,25. Sobre
/// `{0,1}`, `Min` **é** o AND e `Max` **é** o OR, e o par é o oráculo um do outro: um AND
/// que na verdade fosse um OR dispararia nos quadros 15 e 45 também.
#[test]
fn two_pulses_are_anded_by_the_value_math_that_already_exists() {
    let reg = registry();
    let mut both = Vec::new();
    // op: 4 Min (AND) · 5 Max (OR).
    for op in [4.0, 5.0] {
        let mut g = Graph::new();
        let src = rows(&mut g, 1.0);
        let slow = beat(&mut g, src, 0.5);
        let fast = beat(&mut g, src, 0.25);
        let a = chain(&mut g, "pulse.level", slow, &[]);
        let b = chain(&mut g, "pulse.level", fast, &[]);
        let m = g.add_node("value.math");
        g.set_param(m, "op", op);
        connect(&mut g, a, 0, m, 0);
        connect(&mut g, b, 0, m, 1);
        g.validate(&reg).expect("well-typed");
        both.push(fired(&train(&g, &reg, m, "v", 60)));
    }
    assert_eq!(both[0], vec![0, 30], "AND: só onde os DOIS bateram");
    assert_eq!(both[1], vec![0, 15, 30, 45], "OR: onde qualquer um bateu");
}

/// **O PORTÃO** — o item 3 do `SUPERAR:` da folha 12, e a janela de atividade do
/// `pulse.beat`, na mesma cadeia: `pulse → level → value.math(Multiply, condição) →
/// pulse.compare`.
///
/// ⚠️ **A fonte é um pulso ESCALONADO por linha**, não o metrônomo — e a diferença é o que
/// torna este gate honesto. O `pulse.beat` é UNIFORME por natureza (*"every instance beats
/// together"*, o gate dele), então um `pulse.level` que colapsasse na primeira linha
/// passaria: **medido**, essa mutação sobrevive à versão com beat. O escalonamento vem de
/// `value.lfo(Saw, phase_stagger) → pulse.compare`, que é exatamente a cadeia de 2 nós que
/// a folha 12 dá como resposta ao P2 de *swing/fase por linha* — conferida aqui de
/// passagem.
///
/// A condição é POR LINHA (`value.instance_field(Index)` + `value.step` = *"da linha 2 em
/// diante"*), que é a forma que nenhuma referência tem — Max/Pd/TD são canais escalares e o
/// Niagara não tem editor de sinal. As duas metades: as linhas que a condição **não** nomeia
/// ficam mudas, e as que ela nomeia disparam em quadros **DIFERENTES**.
#[test]
fn a_pulse_is_gated_by_a_condition_the_value_domain_names() {
    let reg = registry();
    let mut g = Graph::new();
    let src = rows(&mut g, 4.0);
    // O pulso escalonado: uma rampa por linha, cada uma com a própria fase.
    let saw = chain(
        &mut g,
        "value.lfo",
        src,
        &[
            ("wave", 3.0),
            ("period", 1.0),
            ("amplitude", 0.5),
            ("offset", 0.5),
            ("phase_stagger", 0.2),
        ],
    );
    let src_pulse = g.add_node("pulse.compare");
    g.set_param(src_pulse, "rise", 0.5);
    g.set_param(src_pulse, "fall", 0.25);
    connect(&mut g, saw, 0, src_pulse, 0);
    self_loop(&mut g, src_pulse, 1);

    let lvl = chain(&mut g, "pulse.level", src_pulse, &[]);
    // A máscara por linha: índice 0..3, corte duro em 2 ⇒ [0, 0, 1, 1].
    let idx = chain(&mut g, "value.instance_field", src, &[("mode", 0.0)]);
    let mask = chain(
        &mut g,
        "value.step",
        idx,
        &[("threshold", 2.0), ("mode", 0.0)],
    );
    let m = g.add_node("value.math");
    g.set_param(m, "op", 2.0); // Multiply
    connect(&mut g, lvl, 0, m, 0);
    connect(&mut g, mask, 0, m, 1);
    let cmp = g.add_node("pulse.compare");
    g.set_param(cmp, "rise", 0.5);
    g.set_param(cmp, "fall", 0.25);
    connect(&mut g, m, 0, cmp, 0);
    self_loop(&mut g, cmp, 1);
    g.validate(&reg).expect("well-typed");

    // Quando cada linha disparou, ao longo de um período inteiro do saw.
    let mut when: Vec<Vec<usize>> = vec![Vec::new(); 4];
    let mut cook = Cook::new();
    for k in 0..60 {
        let t = k as f64 / 60.0;
        let s = cook.cook(&g, &reg, cmp, t).expect("cooks")[0]
            .as_stream()
            .clone();
        if let Some(Column::Scalar(v)) = s.get("pulse") {
            for (row, x) in v.iter().enumerate() {
                if *x > 0.5 {
                    when[row].push(k);
                }
            }
        }
        cook.advance_tick(&g, &reg, t).expect("advances");
    }
    assert!(
        when[0].is_empty() && when[1].is_empty(),
        "o portão cala quem a condição não nomeia: {when:?}"
    );
    assert!(
        !when[2].is_empty() && !when[3].is_empty(),
        "e deixa passar quem ela nomeia: {when:?}"
    );
    assert_ne!(
        when[2], when[3],
        "o pulso é POR LINHA: um nível colapsado na 1ª linha daria os mesmos quadros"
    );
}

/// **O ida-e-volta é a identidade.** O nível normaliza para `0/1` e o
/// `pulse.compare(rise = 0.5)` o lê de volta como borda — então atravessar o domínio de
/// valor não inventa nem come um disparo, que é o que torna a composição do portão segura.
#[test]
fn the_round_trip_through_the_value_domain_is_the_identity() {
    let reg = registry();
    let mut g = Graph::new();
    let src = rows(&mut g, 1.0);
    let b = beat(&mut g, src, 0.25);
    let lvl = chain(&mut g, "pulse.level", b, &[]);
    let cmp = g.add_node("pulse.compare");
    g.set_param(cmp, "rise", 0.5);
    g.set_param(cmp, "fall", 0.25);
    connect(&mut g, lvl, 0, cmp, 0);
    self_loop(&mut g, cmp, 1);
    g.validate(&reg).expect("well-typed");

    let direto = fired(&train(&g, &reg, b, "pulse", 60));
    let ida_e_volta = fired(&train(&g, &reg, cmp, "pulse", 60));
    assert_eq!(ida_e_volta, direto, "o desvio pelo valor não muda o trem");
    assert!(!direto.is_empty(), "e o trem não pode estar vazio");
}

/// **A CERCA: o toggle e o latch são o CONTADOR, e é por isso que o `pulse.level` tem zero
/// params.**
///
/// A folha 12 escreve que *"`pulse.counter` acumula (monotônico, nunca volta a 0)"* — o que
/// vale para o `count_tick` no `pre` e **não** para o que ele emite. Com `count_max = 2` o
/// valor exibido é exatamente o par que um `mode` no nível duplicaria. Este gate MEDE a
/// tabela do doc-comment do `pulse.level`; se um dia ela ficar falsa, ela cai aqui em vez
/// de virar um knob a mais.
#[test]
fn the_toggle_and_the_latch_are_the_counter() {
    let reg = registry();
    // mode: 0 Wrap (toggle) · 1 Clamp (latch).
    let mut seen = Vec::new();
    for mode in [0.0, 1.0] {
        let mut g = Graph::new();
        let src = rows(&mut g, 1.0);
        let b = beat(&mut g, src, 0.25);
        let c = g.add_node("pulse.counter");
        g.set_param(c, "count_max", 2.0);
        g.set_param(c, "mode", mode);
        connect(&mut g, b, 0, c, 0);
        self_loop(&mut g, c, 1);
        g.validate(&reg).expect("well-typed");
        // Os quadros dos beats: 0, 15, 30, 45 — leia o valor logo depois de cada um.
        let v = train(&g, &reg, c, "v", 60);
        seen.push(vec![v[0], v[15], v[30], v[45]]);
    }
    assert_eq!(seen[0], vec![1.0, 0.0, 1.0, 0.0], "Wrap = toggle");
    assert_eq!(seen[1], vec![1.0, 1.0, 1.0, 1.0], "Clamp = latch");
}

/// **"DISPARE SÓ QUEM ESTÁ DENTRO DA CAIXA"** — o `SUPERAR:` item 3 da folha 12, agora uma
/// aresta em vez de um script.
///
/// A folha nomeava a combinação que **nenhuma referência tem**: o C4D tem Fields e zero
/// eventos, o Niagara tem eventos e zero campos componíveis, e nós temos os dois — e eles
/// nunca se encontraram, por **uma linha de tabela**. A família `field.*` escreve a coluna
/// `falloff` no stream de instâncias e o `READ_CHANNELS` do `value.attribute` não a listava,
/// então o peso de um campo era consumido por seis `motion.*` e **ilegível** no domínio de
/// valor.
///
/// Este gate é o produto: a caixa cobre metade da fileira, e só essa metade dispara.
#[test]
fn a_pulse_fires_only_where_a_spatial_field_says_it_may() {
    let reg = registry();
    let mut g = Graph::new();
    let src = rows(&mut g, 4.0);
    // O pulso escalonado por linha (a mesma fonte do gate do portão ordinal).
    let saw = chain(
        &mut g,
        "value.lfo",
        src,
        &[
            ("wave", 3.0),
            ("period", 1.0),
            ("amplitude", 0.5),
            ("offset", 0.5),
            ("phase_stagger", 0.2),
        ],
    );
    let src_pulse = g.add_node("pulse.compare");
    g.set_param(src_pulse, "rise", 0.5);
    g.set_param(src_pulse, "fall", 0.25);
    connect(&mut g, saw, 0, src_pulse, 0);
    self_loop(&mut g, src_pulse, 1);
    let lvl = chain(&mut g, "pulse.level", src_pulse, &[]);

    // A CAIXA: borda dura, centrada à direita, larga o bastante para o par da direita.
    let boxf = chain(
        &mut g,
        "field.box",
        src,
        &[
            ("width", 2.5),
            ("height", 100.0),
            ("soft", 0.0),
            ("center_x", 1.5),
        ],
    );
    let mask = g.add_node("value.attribute");
    g.set_text_param(mask, "attr", "falloff");
    g.set_param(mask, "mode", 0.0);
    connect(&mut g, boxf, 0, mask, 0);

    let m = g.add_node("value.math");
    g.set_param(m, "op", 2.0); // Multiply
    connect(&mut g, lvl, 0, m, 0);
    connect(&mut g, mask, 0, m, 1);
    let cmp = g.add_node("pulse.compare");
    g.set_param(cmp, "rise", 0.5);
    g.set_param(cmp, "fall", 0.25);
    connect(&mut g, m, 0, cmp, 0);
    self_loop(&mut g, cmp, 1);
    g.validate(&reg).expect("well-typed");

    let mut cook = Cook::new();
    let mut when: Vec<Vec<usize>> = vec![Vec::new(); 4];
    let mut weights: Vec<f32> = Vec::new();
    for k in 0..60 {
        let t = k as f64 / 60.0;
        let s = cook.cook(&g, &reg, cmp, t).expect("cooks")[0]
            .as_stream()
            .clone();
        if k == 0 {
            let w = cook.cook(&g, &reg, mask, t).expect("cooks")[0]
                .as_stream()
                .clone();
            weights = match w.get("v") {
                Some(Column::Scalar(v)) => v.clone(),
                _ => panic!("o attribute tem de emitir `v`"),
            };
        }
        if let Some(Column::Scalar(v)) = s.get("pulse") {
            for (row, x) in v.iter().enumerate() {
                if *x > 0.5 {
                    when[row].push(k);
                }
            }
        }
        cook.advance_tick(&g, &reg, t).expect("advances");
    }
    // A fileira mora em x = -1.5, -0.5, 0.5, 1.5; a caixa de largura 2,5 centrada em 1,5
    // com borda DURA cobre o par da direita e nada mais.
    assert_eq!(
        weights,
        vec![0.0, 0.0, 1.0, 1.0],
        "o peso que a caixa deixou, lido pelo canal Falloff do picker"
    );
    assert!(
        when[0].is_empty() && when[1].is_empty(),
        "quem está FORA da caixa não dispara: {when:?}"
    );
    assert!(
        !when[2].is_empty() && !when[3].is_empty(),
        "quem está DENTRO dispara: {when:?}"
    );
    assert_ne!(
        when[2], when[3],
        "e continua sendo por LINHA: um nível colapsado daria os mesmos quadros"
    );
}
