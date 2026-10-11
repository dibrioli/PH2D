use super::*;

/// ⭐ Sem `fx.filter` no fio os filtros não correm — e o passe fundido leva o Motion como sempre.
#[test]
fn without_a_filter_nothing_changes() {
    let mut f = MotionFiltro::default();
    f.prepara(&Graph::new(), &[]);
    assert!(!f.liga(true));
    assert!(f.pilha().is_empty());
}

/// ⭐ **O tecto DIGITADO do comprimento é honrado pelo passe em qualquer janela e zoom** (doc 129
/// §5.5 A8): o σ que ele pede numa cena de 8K (4 320 px de altura) no zoom máximo da câmera cabe no
/// `SIGMA_ALCANCAVEL` da pirâmide. Antes a régua era o raio de SATURAÇÃO medido com os `534` px de
/// UMA janela como entrada — verde por construção, e a 4K a mesma régua saturava em ~0,27.
#[test]
fn the_typed_length_ceiling_is_honoured_at_8k_and_max_zoom() {
    let camera = ph2d_render::Camera2d {
        height_world: ph2d_render::Camera2d::ZOOM_MIN_HEIGHT_WORLD,
        ..ph2d_render::Camera2d::default()
    };
    let janela = ph2d_host::WindowSize {
        width: 7680,
        height: 4320,
    };
    let [a, ..] = mundo_para_pixel(&camera, janela, None);
    let mut op = ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::BLUR);
    op.radius = ph2d_node_fx_filter::COMPRIMENTO_DIGITADO;
    let sigma = FxOpGpu::resolve(&op, [a, 0.0, 0.0, -a]).sigma_px;
    assert!(
        sigma <= ph2d_render::fx_stack::SIGMA_ALCANCAVEL,
        "o tecto digitado pede σ {sigma} px e a pirâmide honra até {}",
        ph2d_render::fx_stack::SIGMA_ALCANCAVEL
    );
    assert!(
        sigma > 32.0 * 1000.0,
        "controlo: a fixtura está MUITO acima do kernel de sempre ({sigma})"
    );
}

/// O Y do mundo sobe e o do pixel desce: uma sombra para BAIXO no mundo cai para baixo na tela.
#[test]
fn world_down_is_screen_down() {
    let camera = ph2d_render::Camera2d::default();
    let janela = ph2d_host::WindowSize {
        width: 800,
        height: 600,
    };
    let [a, b, c, d] = mundo_para_pixel(&camera, janela, None);
    assert!((a - 60.0).abs() < 1e-9 && b == 0.0 && c == 0.0 && (d + 60.0).abs() < 1e-9);
    let mut op = ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::DROP_SHADOW);
    op.offset = [0.0, -0.5];
    assert_eq!(FxOpGpu::resolve(&op, [a, b, c, d]).offset_px, [0.0, 30.0]);
}

/// ⭐ **O ruído da camada inteira anda com o MUNDO** (doc 129 §5.5 A6): um pan da câmera de uma
/// unidade leva a origem do padrão `k` pixels (a escala da câmera), no sentido oposto; antes ela
/// era a margem da pilha — um ponto fixo da JANELA, e o padrão ficava parado com as formas a andar.
#[test]
fn the_layer_noise_is_anchored_to_the_world() {
    let janela = ph2d_host::WindowSize {
        width: 800,
        height: 600,
    };
    let vp = Some([100.0, 40.0, 600.0, 500.0]);
    let mut camera = ph2d_render::Camera2d::default();
    let o0 = origem_do_mundo(&camera, janela, vp);
    camera.center[0] += 1.0;
    camera.center[1] += 1.0;
    let o1 = origem_do_mundo(&camera, janela, vp);
    let k = 500.0 / 10.0;
    assert!(
        (o0[0] - (100.0 + 300.0)).abs() < 1e-3 && (o0[1] - (40.0 + 250.0)).abs() < 1e-3,
        "{o0:?}"
    );
    assert!(
        (o1[0] - o0[0] + k).abs() < 1e-3 && (o1[1] - o0[1] - k).abs() < 1e-3,
        "{o0:?} → {o1:?}"
    );
}

/// ⚠️ A camada não atravessa o tonemap. Hoje ele é `clamp` (bypass) e a conta fecha; se o AgX
/// ligar, a camada filtrada sai com outra cor que o resto da cena — este gate reprova primeiro.
#[test]
fn the_layer_skips_a_tonemap_that_is_a_bypass() {
    // O mapeamento mudou para o `tonemap_conta.wgsl` (Luz 2D W3, no `main`).
    let wgsl = include_str!("../../ph2d-render/src/shaders/tonemap_conta.wgsl");
    assert!(
        wgsl.contains("const BYPASS_LUT: bool = true;"),
        "o tonemap deixou o bypass: a camada filtrada (motion_filter) tem de passar por ele"
    );
}

/// ⭐ **Os rótulos do `fx.filter` não colidem onde se encontram** (doc 129 §5.5 U1: «três Filter e
/// dois Glow»): o nome do nó é único no catálogo de nós; as linhas do cartão são únicas e nenhuma
/// repete o nome do nó; e nenhum EFEITO do seletor `Effect` é o NOME de outro nó (o nó `fx.glow` e
/// a opção «Glow» eram dois efeitos diferentes com o mesmo nome; idem o `fx.drop_shadow`, que faz
/// CÓPIAS e passou a «Shadow Copies»). ⚠️ Os MODOS (Proximity, Smooth…) são palavras de modo, não
/// efeitos: o «Proximity» da queda e o nó de campo «Proximity» não se confundem.
#[test]
fn the_filter_labels_are_unique_where_they_meet() {
    use ph2d_node_registry::{NodeRegistry, ParamWidget};
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nós registam");
    let nomes: Vec<(String, &str)> = reg
        .manifests()
        .filter_map(|m| {
            reg.ui_manifest(m.id)
                .map(|u| (ph2d_i18n::tr(u.display_key).to_lowercase(), m.name))
        })
        .collect();
    let id = ph2d_node_fx_filter::MANIFEST.id;
    let nome = ph2d_i18n::tr(reg.ui_manifest(id).expect("ficha").display_key).to_lowercase();
    let iguais: Vec<&str> = nomes
        .iter()
        .filter(|(n, _)| *n == nome)
        .map(|(_, t)| *t)
        .collect();
    assert_eq!(
        iguais,
        vec![ph2d_node_fx_filter::TYPE_NAME],
        "o nome «{nome}» repete-se"
    );
    let fichas = reg.param_ui(id).expect("fichas");
    let mut linhas: Vec<String> = fichas
        .iter()
        .map(|h| ph2d_i18n::tr(h.label).to_lowercase())
        .collect();
    assert!(
        !linhas.contains(&nome),
        "uma linha do cartão diz o nome do nó: {linhas:?}"
    );
    linhas.sort();
    let n = linhas.len();
    linhas.dedup();
    assert_eq!(linhas.len(), n, "duas linhas do cartão com o mesmo rótulo");
    for h in fichas
        .iter()
        .filter(|h| h.param == ph2d_node_fx_filter::param::KIND)
    {
        let ParamWidget::Enum { labels } = h.widget else {
            continue;
        };
        for chave in labels {
            let opcao = ph2d_i18n::tr(chave).to_lowercase();
            let outro: Vec<&str> = nomes
                .iter()
                .filter(|(n, _)| *n == opcao)
                .map(|(_, t)| *t)
                .collect();
            assert!(
                outro.is_empty(),
                "a opção «{opcao}» de `{}` é o nome do nó {outro:?}",
                h.param
            );
        }
    }
}

/// ⭐ **Formas pelo Vello: o cartão de cada `Each Shape` di-lo** (doc 129 §3.5) — desde a W4b elas
/// estão NA camada (o `Whole Layer` filtra-as), mas fora das células: um `Each Shape` vê-as como uma
/// peça só. A lei: há cópias vectoriais e a placa não as desenhou; só os cartões `Each Shape` falam.
#[test]
fn shapes_drawn_by_the_vector_path_are_named_on_every_each_shape_card() {
    assert!(formas_no_vello(3, false));
    assert!(!formas_no_vello(3, true), "a placa desenhou-as");
    assert!(!formas_no_vello(0, false), "sem formas, nada a dizer");

    let mut g = Graph::new();
    let um = g.add_node(ph2d_node_fx_filter::TYPE_NAME.to_string());
    let dois = g.add_node(ph2d_node_fx_filter::TYPE_NAME.to_string());
    let inteira = g.add_node(ph2d_node_fx_filter::TYPE_NAME.to_string());
    g.set_param(inteira, ph2d_node_fx_filter::param::SCOPE, 1.0);
    let solto = g.add_node(ph2d_node_fx_filter::TYPE_NAME.to_string());
    let saida = g.add_node("motion.output".to_string());
    let liga = |g: &mut Graph, de, para| {
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (de, 0),
            to: (para, 0),
            delayed: false,
        })
        .expect("liga")
    };
    liga(&mut g, um, dois);
    liga(&mut g, dois, inteira);
    liga(&mut g, inteira, saida);
    let mut f = MotionFiltro::default();
    f.prepara(&g, &[]);
    assert!(f.liga(true));
    assert!(
        f.queixa(um).is_none(),
        "sem formas pelo Vello, o cartão cala"
    );
    f.formas_no_vello = true;
    for no in [um, dois] {
        let q = f.queixa(no).expect("o cartão diz");
        assert!(q.contains("Blend") && q.contains("one piece"), "{q}");
    }
    assert!(
        f.queixa(inteira).is_none(),
        "o `Whole Layer` filtra as formas do Vello — o cartão dele cala"
    );
    let q = f.queixa(solto).expect("o solto diz o seu");
    assert!(
        q.contains("Not on the wire"),
        "a queixa de fora do fio vem primeiro: {q}"
    );
    // O quadro seguinte sem o Motion na cena apaga-a.
    assert!(!f.liga(false));
    assert!(f.queixa(um).is_none());
}

/// ⭐⭐ **A pousa e a camada concordam em todo tag** (doc 129 §3.5): um modo com pousa é um modo com
/// camada ([`ph2d_eval_motion::MisturaDoSink::tem_camada`]) e o MESMO do Vello (a tradução do
/// codificador das formas) — senão a camada pousava num modo e as cópias misturavam-se noutro.
/// `Copies` pousa sempre em `Normal` (mistura só entre as cópias).
#[test]
fn the_landing_mode_and_the_group_layer_agree_on_every_tag() {
    for tag in 0..=u8::MAX {
        let m = ph2d_eval_motion::MisturaDoSink {
            blend: tag,
            ..Default::default()
        };
        for com in BlendWith::ALL {
            let p = pousa_do_sink(tag, com);
            let esperado = m.tem_camada() && com != BlendWith::Copies;
            assert_eq!(p != Pousa::Normal, esperado, "tag {tag} {com:?}: {p:?}");
        }
    }
    let vello = |tag| crate::motion_shape_gen::mistura::mistura_vello(tag);
    let par = |p: Pousa| match p {
        Pousa::Soma => {
            ph2d_vector::VelloBlend::new(ph2d_vector::Mix::Normal, ph2d_vector::Compose::Plus)
        }
        Pousa::Multiplica => {
            ph2d_vector::VelloBlend::new(ph2d_vector::Mix::Multiply, ph2d_vector::Compose::SrcOver)
        }
        Pousa::Tela => {
            ph2d_vector::VelloBlend::new(ph2d_vector::Mix::Screen, ph2d_vector::Compose::SrcOver)
        }
        Pousa::Normal => {
            ph2d_vector::VelloBlend::new(ph2d_vector::Mix::Normal, ph2d_vector::Compose::SrcOver)
        }
    };
    for tag in [1u8, 3, 4] {
        assert_eq!(
            vello(tag),
            Some(par(pousa_do_sink(tag, BlendWith::Scene))),
            "tag {tag}: a pousa e o Vello traduzem o modo de maneiras diferentes"
        );
    }
}

/// ⭐⭐ **Só o `Scene` deixa de agrupar, e só com a camada ligada** (doc 129 §3.5) — a regra que a
/// bomba e a rota da placa perguntam ([`ph2d_eval_motion::MisturaDoSink::pousa_na_camada`]).
#[test]
fn only_a_scene_blend_goes_to_the_landing_and_only_with_the_layer() {
    for tag in 0..=u8::MAX {
        for com in BlendWith::ALL {
            let m = ph2d_eval_motion::MisturaDoSink {
                blend: tag,
                com,
                sink: 0,
            };
            assert!(!m.pousa_na_camada(false), "sem camada nada muda");
            assert_eq!(
                m.pousa_na_camada(true),
                m.tem_camada() && com == BlendWith::Scene,
                "tag {tag} {com:?}"
            );
        }
    }
}

/// A pousa vem dos SINKS: um `Output` em `Add` com o cenário pousa em `Soma`; sem sinks, `Normal`.
#[test]
fn the_landing_comes_from_the_sinks() {
    let mut g = Graph::new();
    let saida = g.add_node("motion.output".to_string());
    let mut f = MotionFiltro::default();
    f.prepara(&g, &[saida]);
    assert_eq!(f.pousa(), Pousa::Normal, "o `Output` nasce sem mistura");
    g.set_param(saida, ph2d_eval_motion::SINK_BLEND_PARAM, 1.0);
    f.prepara(&g, &[saida]);
    assert_eq!(f.pousa(), Pousa::Soma, "`Add` com tudo pousa somando");
    g.set_param(saida, ph2d_eval_motion::SINK_BLEND_WITH_PARAM, 1.0);
    f.prepara(&g, &[saida]);
    assert_eq!(
        f.pousa(),
        Pousa::Normal,
        "`Copies` mistura dentro da camada"
    );
    f.prepara(&g, &[]);
    assert_eq!(f.pousa(), Pousa::Normal);
}

/// ⭐ **Duas saídas, dois modos, UMA camada: o cartão de cada filtro di-lo** (doc 129 §3.5, item 4) —
/// a camada pousa com o modo da primeira. ⚠️ O controlo: duas saídas com o MESMO modo calam.
#[test]
fn two_outputs_that_land_differently_are_named_on_every_filter_card() {
    let mut g = Graph::new();
    let filtro = g.add_node(ph2d_node_fx_filter::TYPE_NAME.to_string());
    let a = g.add_node("motion.output".to_string());
    let b = g.add_node("motion.output".to_string());
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (filtro, 0),
        to: (a, 0),
        delayed: false,
    })
    .expect("liga");
    for s in [a, b] {
        g.set_param(s, ph2d_eval_motion::SINK_BLEND_PARAM, 1.0);
    }
    let mut f = MotionFiltro::default();
    f.prepara(&g, &[a, b]);
    assert!(f.liga(true));
    assert!(
        f.queixa(filtro).is_none(),
        "o mesmo modo nas duas: o cartão cala"
    );
    g.set_param(b, ph2d_eval_motion::SINK_BLEND_PARAM, 3.0);
    f.prepara(&g, &[a, b]);
    assert_eq!(f.pousa(), Pousa::Soma, "a primeira manda");
    let q = f.queixa(filtro).expect("o cartão diz");
    assert!(q.contains("different modes"), "{q}");
}

/// ⭐⭐ **A rota da PLACA pergunta a mesma regra** (doc 129 §3.5): com a camada isolada, um `Output`
/// em `Add` com o `Scene` não recusa a placa e o estilo dele vai sem modo; o `Everything` continua a
/// recusar e a levar o modo. ⚠️ O controlo: sem a camada, o grupo de sempre.
#[test]
fn the_device_route_asks_the_same_landing_rule() {
    let mut m = crate::motion_state::MotionState::new();
    let saida = m.doc.graph.add_node("motion.output".to_string());
    m.sinks = vec![saida];
    m.doc
        .graph
        .set_param(saida, ph2d_eval_motion::SINK_BLEND_PARAM, 1.0);
    for (alcance, isolada, agrupa) in [(2.0, false, true), (2.0, true, false), (0.0, true, true)] {
        m.doc
            .graph
            .set_param(saida, ph2d_eval_motion::SINK_BLEND_WITH_PARAM, alcance);
        m.pump.isola_na_camada(isolada);
        let caso = format!("alcance {alcance}, camada {isolada}");
        assert_eq!(
            crate::motion_bridge::gpu::sink_mistura_em_grupo(&m.doc.graph, saida, isolada),
            agrupa,
            "{caso}: a cerca da placa"
        );
        let estilos = crate::motion_bridge::gpu::estilos_das_saidas(&m);
        assert_eq!(
            estilos[0].blend,
            if agrupa { 1 } else { 0 },
            "{caso}: o estilo da placa"
        );
    }
}
