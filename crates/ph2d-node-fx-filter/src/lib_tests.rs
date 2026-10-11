use super::*;
use ph2d_node_registry::ParamWidget;

fn reg() -> NodeRegistry {
    let mut r = NodeRegistry::new();
    register(&mut r).expect("regista");
    r
}

/// Liga `de → para` na porta 0.
fn liga(g: &mut Graph, de: NodeId, para: NodeId) {
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (de, 0),
        to: (para, 0),
        delayed: false,
    })
    .expect("liga");
}

fn cadeia(tipos: &[&str]) -> (Graph, Vec<NodeId>) {
    let mut g = Graph::new();
    let ids: Vec<NodeId> = tipos.iter().map(|t| g.add_node(*t)).collect();
    for par in ids.windows(2) {
        liga(&mut g, par[0], par[1]);
    }
    (g, ids)
}

#[test]
fn without_a_filter_on_the_wire_the_stack_is_empty() {
    let (g, _) = cadeia(&["source.grid", "motion.output"]);
    assert!(from_graph(&g).is_empty());
}

/// ⭐ A ordem do FIO é a ordem da pilha — não a de inserção no grafo.
#[test]
fn the_stack_follows_the_wire_upstream_to_downstream() {
    let mut g = Graph::new();
    let out = g.add_node("motion.output");
    let b = g.add_node(TYPE_NAME);
    let a = g.add_node(TYPE_NAME);
    let src = g.add_node("source.grid");
    liga(&mut g, src, a);
    liga(&mut g, a, b);
    liga(&mut g, b, out);
    g.set_param(a, param::KIND, f32::from(FxOp::DROP_SHADOW));
    let pilha = from_graph(&g);
    assert_eq!(pilha.iter().map(|d| d.no).collect::<Vec<_>>(), vec![a, b]);
    assert_eq!(pilha[0].op.kind, FxOp::DROP_SHADOW);
    assert_eq!(pilha[1].op.kind, FxOp::BLUR);
    assert!(fora_do_fio(&g, &pilha).is_empty());
}

/// Um filtro largado fora do fio não filtra — e é nomeado, para o cartão o dizer.
#[test]
fn a_filter_off_the_wire_is_named_not_applied() {
    let (mut g, _) = cadeia(&["source.grid", "motion.output"]);
    let solto = g.add_node(TYPE_NAME);
    let pilha = from_graph(&g);
    assert!(pilha.is_empty());
    assert_eq!(fora_do_fio(&g, &pilha), vec![solto]);
}

/// Cada knob aparece EXACTAMENTE para os tipos que a `SPECS` diz que o leem — o gate deriva-se da
/// tabela e confere-se contra ela, tipo a tipo.
#[test]
fn every_gate_is_the_specs_own_answer() {
    let pares: [(&str, Knob); 7] = [
        (param::RADIUS, Knob::Radius),
        (param::OFFSET_X, Knob::Offset),
        (param::COLOR[0], Knob::Color),
        (param::COLOR2[0], Knob::Color2),
        (param::SIZE, Knob::Noise),
        (param::GROW, Knob::Grow),
        (param::HUE, Knob::Adjust),
    ];
    for (nome, knob) in pares {
        let g = PARAM_GATES.iter().find(|g| g.param == nome).expect(nome);
        for k in 0..FxOp::KINDS {
            let mostra = g.values.contains(&i32::try_from(k).unwrap());
            assert_eq!(
                mostra,
                takes(k, knob),
                "{nome} no tipo {}",
                FxOp::SPECS[k].name
            );
        }
        // Só tipos reais (compacta: o gate esconde o que não está na lista).
        assert!(g.values.iter().all(|&v| (0..15).contains(&v)));
    }
    let falloff = PARAM_GATES
        .iter()
        .find(|g| g.param == param::FALLOFF)
        .unwrap();
    assert_eq!(
        falloff.values,
        [FxOp::GLOW, FxOp::INNER_SHADOW, FxOp::INNER_GLOW].map(i32::from)
    );
}

/// Todo tipo com modos é servido por UM param de modo — um tipo novo com uma lista nova reprova.
#[test]
fn every_kind_with_modes_has_exactly_one_mode_param() {
    for k in 0..FxOp::KINDS {
        let modos = FxOp::SPECS[k].modes;
        let n = MODE_PARAMS
            .iter()
            .filter(|(_, l)| same_list(l, modos))
            .count();
        assert_eq!(n, usize::from(!modos.is_empty()), "{}", FxOp::SPECS[k].name);
    }
}

/// ⭐ Os rótulos do seletor DIZEM o nome da `SPECS`, pela tabela de strings — a lista não é escrita
/// à mão: um tipo trocado de lugar reprova aqui.
#[test]
fn the_labels_say_the_specs_names() {
    for (k, chave) in KIND_LABELS.iter().enumerate() {
        assert_eq!(ph2d_i18n::tr(chave), FxOp::SPECS[k].name, "tipo {k}");
    }
    for ((_, modos), rotulos) in
        MODE_PARAMS
            .iter()
            .zip([&FALLOFF_LABELS, &NOISE_LABELS, &RAMP_LABELS])
    {
        let palavras: Vec<&str> = rotulos.iter().map(|c| ph2d_i18n::tr(c)).collect();
        assert_eq!(palavras, modos.to_vec());
    }
}

#[test]
fn every_param_has_a_hint_or_is_a_colour_channel() {
    for s in MANIFEST.params {
        let tem = PARAM_HINTS.iter().any(|h| h.param == s.name);
        let canal = PARAM_HINTS.iter().any(
            |h| matches!(h.widget, ParamWidget::Color { channels } if channels.contains(&s.name)),
        );
        assert!(tem || canal, "{} sem ficha", s.name);
    }
}

/// ⭐ Um nó largado É o `FxOp::new(BLUR)` do Vector — o manifesto contra a LEI do outro autor,
/// nunca contra si próprio (o verde antigo comparava o nó com o próprio manifesto, doc 129 §5.5 A2).
#[test]
fn the_dropped_filter_is_the_vectors_blur() {
    let (g, ids) = cadeia(&["source.grid", TYPE_NAME, "motion.output"]);
    assert_eq!(op_of(&g, ids[1]), FxOp::new(FxOp::BLUR));
    assert_eq!(from_graph(&g)[0].scope, Scope::EachShape, "nasce em cada forma");
}

/// Igual a menos do arredondamento da ida-e-volta da cor (linear ↔ sRGB em `f32`).
fn same_op(a: &FxOp, b: &FxOp) -> bool {
    let perto = |x: &[f32], y: &[f32]| x.iter().zip(y).all(|(p, q)| (p - q).abs() < 1e-5);
    let (mut a2, mut b2) = (a.clone(), b.clone());
    let cores = perto(&a.color, &b.color)
        && perto(&a.color_b, &b.color_b)
        && perto(a.stops.as_flattened(), b.stops.as_flattened());
    (a2.color, a2.color_b, a2.stops) = (b.color, b.color_b, b.stops);
    b2.stops = b.stops;
    cores && a2 == b2
}

/// ⭐⭐ **Um tipo escolhido no cartão = o `FxOp::new(tipo)` do Vector** (doc 129 §5.5 A2/A5): o
/// Glow nasce BRANCO e em Proximity, o Color Overlay rosa, a luz do bevel a `(-0,1, 0,1)`.
/// Controlo: sem o gesto (só o `kind`), o Glow nascia preto — o defeito que o dono viu.
#[test]
fn a_kind_chosen_on_the_card_is_the_vectors_new() {
    for k in 0..FxOp::KINDS {
        #[expect(clippy::cast_possible_truncation, reason = "KINDS = 15")]
        let k = k as u8;
        let (mut g, ids) = cadeia(&["source.grid", TYPE_NAME, "motion.output"]);
        switch_kind(&mut g, ids[1], k);
        let op = op_of(&g, ids[1]);
        assert!(same_op(&op, &FxOp::new(k)), "{}: {op:?}", FxOp::kind_name(k));
    }
    let (mut g, ids) = cadeia(&["source.grid", TYPE_NAME, "motion.output"]);
    g.set_param(ids[1], param::KIND, f32::from(FxOp::GLOW));
    assert_eq!(op_of(&g, ids[1]).color, [0.0, 0.0, 0.0, 1.0], "controlo: o glow preto");
}

/// O que o artista mexeu SOBREVIVE à troca; o que não mexeu segue o tipo novo — ida e volta.
#[test]
fn switching_kind_keeps_what_the_artist_moved() {
    let (mut g, ids) = cadeia(&["source.grid", TYPE_NAME, "motion.output"]);
    let no = ids[1];
    switch_kind(&mut g, no, FxOp::DROP_SHADOW);
    g.set_param(no, param::RADIUS, 0.3);
    switch_kind(&mut g, no, FxOp::INNER_SHADOW);
    let op = op_of(&g, no);
    assert!((op.radius - 0.3).abs() < 1e-6, "o raio mexido fica");
    assert_eq!(op.offset, FxOp::new(FxOp::INNER_SHADOW).offset, "o deslocamento segue o tipo");
    assert!((op.opacity - FxOp::new(FxOp::INNER_SHADOW).opacity).abs() < 1e-6);
    switch_kind(&mut g, no, FxOp::DROP_SHADOW);
    assert_eq!(op_of(&g, no).offset, FxOp::new(FxOp::DROP_SHADOW).offset);
}

/// ⭐ **A cor escolhida no cartão chega à pilha como o artista a vê** (doc 129 §5.5 A4): o cartão
/// guarda LINEAR, a `FxOp` é sRGB reta. ⚠️ A fixtura é um cinzento MÉDIO — preto e branco não se
/// movem pela transferência e foram o verde que escondia o defeito.
#[test]
fn a_card_colour_reaches_the_op_as_the_artist_sees_it() {
    let (mut g, ids) = cadeia(&["source.grid", TYPE_NAME, "motion.output"]);
    let no = ids[1];
    switch_kind(&mut g, no, FxOp::DROP_SHADOW);
    let cinza = 128.0 / 255.0;
    let lin = ph2d_color::srgb::srgb_to_linear_unit(cinza);
    for c in &param::COLOR[..3] {
        g.set_param(no, *c, lin);
    }
    let op = op_of(&g, no);
    for v in &op.color[..3] {
        assert!((v - cinza).abs() < 1e-5, "sRGB 128 chega como {}", v * 255.0);
    }
    assert!((lin - cinza).abs() > 0.2, "controlo: a fixtura move-se pela transferência");
}

/// ⭐ **A pilha parte-se no primeiro `Whole Layer`** (doc 129 §3.4): o prefixo `Each Shape` corre
/// por forma; um `Each Shape` depois dele corre na camada e é nomeado para o cartão o dizer.
#[test]
fn the_stack_splits_at_the_first_whole_layer() {
    let tipos = ["source.grid", TYPE_NAME, TYPE_NAME, TYPE_NAME, TYPE_NAME, "motion.output"];
    let (mut g, ids) = cadeia(&tipos);
    let [a, b, c, d] = [ids[1], ids[2], ids[3], ids[4]];
    g.set_param(c, param::SCOPE, 1.0);
    let pilha = from_graph(&g);
    let (forma, camada) = partition(&pilha);
    assert_eq!(forma.iter().map(|x| x.no).collect::<Vec<_>>(), vec![a, b]);
    assert_eq!(camada.iter().map(|x| x.no).collect::<Vec<_>>(), vec![c, d]);
    assert_eq!(na_camada_apesar_do_scope(&pilha), vec![d]);
    g.set_param(a, param::SCOPE, 1.0);
    let pilha = from_graph(&g);
    assert!(partition(&pilha).0.is_empty(), "um Whole Layer à cabeça = tudo na camada");
    assert_eq!(na_camada_apesar_do_scope(&pilha), vec![b, d]);
}

/// A matiz do cartão é em graus; a `FxOp` guarda voltas. O modo vem do param do tipo escolhido.
#[test]
fn overrides_reach_the_op() {
    let (mut g, ids) = cadeia(&["source.grid", TYPE_NAME, "motion.output"]);
    let no = ids[1];
    g.set_param(no, param::KIND, f32::from(FxOp::COLOR_ADJUST));
    g.set_param(no, param::HUE, 90.0);
    g.set_param(no, param::FALLOFF, 1.0);
    let op = op_of(&g, no);
    assert!((op.hue - 0.25).abs() < 1e-6);
    // Color Adjust não tem modos: o `falloff` mexido não o alcança.
    assert_eq!(op.mode, FxOp::new(FxOp::COLOR_ADJUST).mode);
    g.set_param(no, param::KIND, f32::from(FxOp::GLOW));
    assert_eq!(op_of(&g, no).mode, FxOp::MODE_CONTOUR);
    g.set_param(no, param::KIND, f32::from(FxOp::TURBULENCE));
    g.set_param(no, param::NOISE_MODE, 1.0);
    assert_eq!(op_of(&g, no).mode, FxOp::MODE_CREASED);
}

/// A rampa de dois stops sai das duas cores — e é a do `FxOp::new` com os defaults.
#[test]
fn the_ramp_is_the_two_colours() {
    let (mut g, ids) = cadeia(&["source.grid", TYPE_NAME, "motion.output"]);
    g.set_param(ids[1], param::KIND, f32::from(FxOp::GRADIENT_MAP));
    let op = op_of(&g, ids[1]);
    let novo = FxOp::new(FxOp::GRADIENT_MAP);
    assert_eq!(op.stop_count, 2);
    // ⚠️ O branco volta da transferência linear → sRGB a `0,99999994` (o `f32` da curva única).
    let (a, b) = (op.ramp_for_device(), novo.ramp_for_device());
    let perto = |x: &[f32], y: &[f32]| x.iter().zip(y).all(|(p, q)| (p - q).abs() < 1e-6);
    assert!(perto(a.0.as_flattened(), b.0.as_flattened()) && a.1 == b.1 && a.2 == b.2);
}

#[test]
fn it_registers_as_a_device_passthrough() {
    use ph2d_nodegraph::gpu::KernelResolver as _;
    assert!(reg().gpu_kernel(MANIFEST.id).is_some());
    assert_eq!(MANIFEST.effect, Effect::Pure);
}
