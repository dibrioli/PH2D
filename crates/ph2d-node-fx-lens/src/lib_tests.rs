use super::*;

fn grafo_com_lente() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let no = g.add_node(TYPE_NAME);
    (g, no)
}

/// Sem o nó, nada: o passe não corre.
#[test]
fn without_the_node_there_is_no_lens() {
    assert_eq!(from_graph(&Graph::new()), None);
}

/// ⭐ **Largado, o nó mostra a VINHETA e nada mais** — o artista vê a lente sem mexer em nada, e
/// os outros três efeitos nascem neutros.
/// (Mutação: o default de `vignette` a `0` ⇒ a lente nasce neutra — RED.)
#[test]
fn a_dropped_lens_shows_only_the_vignette() {
    let (g, no) = grafo_com_lente();
    let l = from_graph(&g).expect("o nó está no grafo");
    assert_eq!(l.no, no);
    assert_eq!(l.lut, None);
    assert!((l.lente.vinheta.amount - 0.4).abs() < 1e-6);
    assert!(!l.lente.vinheta.neutra());
    assert!(l.lente.aberracao.neutra());
    assert!(l.lente.grade.neutra(false));
    assert!(l.lente.grao.neutra());
    assert!(!l.lente.neutra(false));
}

/// Os overrides chegam à lei — o ângulo de GRAUS para voltas, os interruptores e a semente.
#[test]
fn overrides_reach_the_law() {
    let (mut g, no) = grafo_com_lente();
    g.set_param(no, param::AB, 0.03);
    g.set_param(no, param::AB_MODE, 1.0);
    g.set_param(no, param::AB_ANGLE, 90.0);
    g.set_param(no, param::EXPOSURE, 0.5);
    g.set_param(no, param::GRAIN_COLOR, 1.0);
    g.set_param(no, param::GRAIN_ANIMATED, 0.0);
    g.set_param(no, param::GRAIN_SEED, 42.0);
    g.set_text_param(no, LUT, "  /tmp/look.cube ");
    let l = from_graph(&g).unwrap();
    assert!(l.lente.aberracao.linear);
    assert!((l.lente.aberracao.angle - 0.25).abs() < 1e-6);
    assert!((l.lente.grade.exposure - 0.5).abs() < 1e-6);
    assert!(l.lente.grao.colorido);
    assert!(!l.lente.grao.animado);
    assert_eq!(l.lente.grao.semente, 42);
    assert_eq!(l.lut.as_deref(), Some("/tmp/look.cube"));
    g.set_text_param(no, LUT, "   ");
    assert_eq!(
        from_graph(&g).unwrap().lut,
        None,
        "um caminho em branco é sem LUT"
    );
}

/// **Todo param tem hint e secção** — um param sem hint não existe no cartão (controlo morto), e
/// um sem secção pintaria solto antes das quatro.
#[test]
fn every_param_has_a_hint_and_a_section() {
    let mut nomes: Vec<&str> = MANIFEST.params.iter().map(|p| p.name).collect();
    nomes.push(LUT);
    for n in nomes {
        let canal_de_cor = param::VIN_COR[1..].contains(&n);
        assert!(
            canal_de_cor || PARAM_HINTS.iter().any(|h| h.param == n),
            "{n} sem hint"
        );
        assert!(
            canal_de_cor || PARAM_GROUPS.iter().any(|g| g.param == n),
            "{n} sem secção"
        );
    }
}

/// A ordem das secções é a ordem FIXA da lente, e só a vinheta nasce aberta.
#[test]
fn the_sections_follow_the_fixed_order_and_only_the_vignette_is_open() {
    let mut ordem: Vec<&str> = Vec::new();
    for g in PARAM_GROUPS {
        if ordem.last() != Some(&g.group_key) {
            ordem.push(g.group_key);
        }
        assert_eq!(
            g.folded,
            g.group_key != "node.group.vignette",
            "{}",
            g.param
        );
    }
    assert_eq!(
        ordem,
        [
            "node.group.aberration",
            "node.group.vignette",
            "node.group.grade",
            "node.group.grain"
        ]
    );
}

/// O nó é passthrough e regista-se com o kernel do dispositivo.
#[test]
fn it_registers_as_a_device_passthrough() {
    use ph2d_nodegraph::gpu::KernelResolver as _;
    let mut reg = NodeRegistry::new();
    register(&mut reg).expect("regista");
    assert!(reg.gpu_kernel(MANIFEST.id).is_some());
    assert_eq!(MANIFEST.effect, Effect::Pure);
}
