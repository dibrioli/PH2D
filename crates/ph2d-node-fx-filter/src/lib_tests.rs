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

/// O cartão diz o que se aplica: um nó largado dá o degrau dos DEFAULTS do manifesto.
#[test]
fn a_dropped_filter_is_the_manifest_defaults() {
    let (g, ids) = cadeia(&["source.grid", TYPE_NAME, "motion.output"]);
    let op = op_of(&g, ids[1]);
    assert_eq!(op.kind, FxOp::BLUR);
    assert!((op.radius - 0.1).abs() < 1e-6);
    assert_eq!(op.offset, [0.12, -0.12]);
    assert!(op.is_active());
    assert_eq!(op.blend_code(), FxOp::BLEND_NORMAL);
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
    assert_eq!(op.ramp_for_device(), novo.ramp_for_device());
}

#[test]
fn it_registers_as_a_device_passthrough() {
    use ph2d_nodegraph::gpu::KernelResolver as _;
    assert!(reg().gpu_kernel(MANIFEST.id).is_some());
    assert_eq!(MANIFEST.effect, Effect::Pure);
}
